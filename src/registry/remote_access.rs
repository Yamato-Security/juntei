// Copyright (C) 2026 Shirofune Security Co., Ltd.
//
// Unless explicitly acquired and licensed from Licensor under another license, the contents of this
// file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
// allowed by the RPL, and You may not copy or use this file in either source code or executable
// form, except in compliance with the terms and conditions of the RPL.
//
// All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
// OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
// INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
// and limitations under the RPL.

//! `parse-registry remote-access`: the remote connections users made, from `NTUSER.DAT` hives,
//! read by ys-registry-parser's `remote_access` module, with Hayabusa's input options, progress
//! bar and results summary. One record per Remote Desktop computer (`Terminal Server Client`),
//! PuTTY and WinSCP saved session and accepted SSH host key (with its OpenSSH fingerprint),
//! mapped drive (`Network`, `Map Network Drive MRU`) and share Explorer mounted (`MountPoints2`).
//! Stored passwords are reported as present, never decoded. The summary lists the connections
//! written last, and the sessions that keep a password.

use super::{
    Hosts, Opened, Profiles, RecordWriter, choose_logs, files_read, filetime, find_hives_of_type,
    input_files, open, profile_of_path, sink,
};
use crate::cli::RegistryRemoteAccessOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::ntuser::profile_name;
use registry::remote_access::{Connection, connections};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems, connections and sessions listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryRemoteAccessOptions) -> i32 {
    let no_color = opts.common.no_color;
    let data_on_stdout = opts.data_on_stdout();
    let report_error = |bar: Option<&ProgressBar>, msg: &str| {
        if data_on_stdout {
            eprintln!("{msg}");
        } else {
            progress::suspended(bar, || error_msg(no_color, msg));
        }
    };

    // --- inputs ---------------------------------------------------------------------------
    let given_logs: Vec<PathBuf> = opts.log1.iter().chain(&opts.log2).cloned().collect();
    let inputs = input_files(
        opts.filepath.as_ref(),
        opts.directory.as_ref(),
        "NTUSER.DAT hives",
        |d| find_hives_of_type(d, "NTUSER.DAT"),
    )
    .and_then(|files| Ok((files, choose_logs(opts.no_logs, &given_logs)?)))
    .and_then(|(files, logs)| {
        let sink = sink(opts.output.as_ref(), opts.clobber, data_on_stdout, || {
            files_read(&files, &given_logs)
        })?;
        Ok((files, logs, sink))
    });
    let (files, logs, sink) = match inputs {
        Ok(inputs) => inputs,
        Err(msg) => {
            report_error(None, &msg);
            return 1;
        }
    };
    let mut writer = match sink.map(|w| RecordWriter::new(w, opts.output_type(), &CSV_HEADER)) {
        Some(Ok(w)) => Some(w),
        Some(Err(e)) => {
            report_error(None, &format!("Could not write the records: {e}"));
            return 1;
        }
        None => None,
    };
    if !data_on_stdout {
        let total: u64 = files
            .iter()
            .map(|p| p.metadata().map_or(0, |m| m.len()))
            .sum();
        p_label(no_color, "NTUSER.DAT hives: ", &fmt_num(files.len()));
        p_label(no_color, "Total file size: ", &util::size_display(total));
        p(None, "", true);
    }
    let bar = (!data_on_stdout).then(|| {
        p(
            JunteiColor::Orange.rdg(no_color),
            "Parsing in progress. Please wait.",
            true,
        );
        p(None, "", true);
        progress::progress_bar(files.len(), no_color)
    });

    // --- parse and write -------------------------------------------------------------------
    let mut hosts = Hosts::new(!opts.no_logs);
    let mut profiles = Profiles::new(!opts.no_logs);
    let mut stats = Stats {
        to_stderr: data_on_stdout,
        ..Stats::default()
    };
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let parsed = open(path, logs).and_then(|mut opened| {
            if !opened.hive_type.is_empty() && opened.hive_type != "NTUSER.DAT" {
                return Err(format!(
                    "a {} hive, not an NTUSER.DAT hive",
                    opened.hive_type
                ));
            }
            opened.host = hosts.of(path, &opened);
            let username = profile_of_path(path).or_else(|| profile_name(&opened.hive));
            let sid = username.as_deref().and_then(|u| profiles.sid(path, u));
            Ok((opened, User { username, sid }))
        });
        match parsed {
            Ok((opened, user)) => {
                if let Err(e) = stats.hive(&opened, &user, writer.as_mut()) {
                    report_error(
                        bar.as_ref(),
                        &format!("Could not write the records of {}: {e}", path.display()),
                    );
                    return 1;
                }
            }
            Err(e) => {
                stats.failed += 1;
                if stats.failed <= MAX_PRINTED_ERRORS {
                    report_error(
                        bar.as_ref(),
                        &format!("Could not parse {}: {e}", path.display()),
                    );
                }
            }
        }
        if let Some(b) = &bar {
            b.inc(1);
        }
    }
    if stats.failed > MAX_PRINTED_ERRORS {
        report_error(
            bar.as_ref(),
            &format!(
                "{} more files could not be parsed",
                fmt_num(stats.failed - MAX_PRINTED_ERRORS)
            ),
        );
    }
    if let Some(w) = writer
        && let Err(e) = w.finish()
    {
        report_error(None, &format!("Could not write the records: {e}"));
        return 1;
    }
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(no_color));
    }

    // --- summary ---------------------------------------------------------------------------
    if !data_on_stdout {
        stats.print(no_color);
        if let Some(out) = &opts.output {
            p_label(
                no_color,
                "Results saved: ",
                &format!("{} ({})", out.display(), util::file_size_display(out)),
            );
        } else {
            p(
                None,
                "To save every record, add -o <FILE> (-t csv, json or jsonl).",
                true,
            );
        }
    }
    i32::from(stats.hives == 0)
}

// --- records ----------------------------------------------------------------------------------

/// The user a hive belongs to: its profile folder's name, and its SID from its system's
/// `ProfileList`.
struct User {
    username: Option<String>,
    sid: Option<String>,
}

/// A connection's details, as a JSON object in their order.
struct Details<'a>(&'a [(String, String)]);

impl Serialize for Details<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.0.iter().map(|(name, value)| (name, value)))
    }
}

/// A connection's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    host: Option<&'a str>,
    username: Option<&'a str>,
    user_sid: Option<&'a str>,
    source_path: &'a str,
    client: &'static str,
    kind: &'static str,
    target: Option<&'a str>,
    port: Option<u32>,
    remote_username: Option<&'a str>,
    protocol: Option<&'a str>,
    mru_position: Option<u32>,
    session_name: Option<&'a str>,
    host_key_type: Option<&'a str>,
    host_key_fingerprint: Option<&'a str>,
    host_key: Option<&'a str>,
    drive_letter: Option<&'a str>,
    password_stored: Option<bool>,
    details: Details<'a>,
    key_path: &'a str,
    value_name: Option<&'a str>,
    key_last_written: Option<String>,
    data_raw: Option<String>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

const CSV_HEADER: [&str; 25] = [
    "Host",
    "Username",
    "UserSid",
    "SourcePath",
    "Client",
    "Kind",
    "Target",
    "Port",
    "RemoteUsername",
    "Protocol",
    "MruPosition",
    "SessionName",
    "HostKeyType",
    "HostKeyFingerprint",
    "HostKey",
    "DriveLetter",
    "PasswordStored",
    "Details",
    "KeyPath",
    "ValueName",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl<'a> Record<'a> {
    fn new(opened: &'a Opened, user: &'a User, connection: &'a Connection) -> Self {
        Record {
            host: opened.host.as_deref(),
            username: user.username.as_deref(),
            user_sid: user.sid.as_deref(),
            source_path: &opened.source,
            client: connection.kind.client().name(),
            kind: connection.kind.name(),
            target: connection.target.as_deref(),
            port: connection.port,
            remote_username: connection.username.as_deref(),
            protocol: connection.protocol.as_deref(),
            mru_position: connection.mru_position,
            session_name: connection.session_name.as_deref(),
            host_key_type: connection.host_key_type.as_deref(),
            host_key_fingerprint: connection.host_key_fingerprint.as_deref(),
            host_key: connection.host_key.as_deref(),
            drive_letter: connection.drive_letter.as_deref(),
            password_stored: connection.password_stored,
            details: Details(&connection.details),
            key_path: &connection.key_path,
            value_name: connection.value_name.as_deref(),
            key_last_written: filetime(connection.key_last_written),
            data_raw: connection.data_raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(connection.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &connection.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let num = |v: Option<u32>| v.map(|n| n.to_string()).unwrap_or_default();
        let flag = |v: Option<bool>| match v {
            Some(true) => "True".to_string(),
            Some(false) => "False".to_string(),
            None => String::new(),
        };
        let details: Vec<String> = self
            .details
            .0
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        vec![
            text(self.host),
            text(self.username),
            text(self.user_sid),
            self.source_path.to_string(),
            self.client.to_string(),
            self.kind.to_string(),
            text(self.target),
            num(self.port),
            text(self.remote_username),
            text(self.protocol),
            num(self.mru_position),
            text(self.session_name),
            text(self.host_key_type),
            text(self.host_key_fingerprint),
            text(self.host_key),
            text(self.drive_letter),
            flag(self.password_stored),
            details.join("; "),
            self.key_path.to_string(),
            text(self.value_name),
            text(self.key_last_written.as_deref()),
            text(self.data_raw.as_deref()),
            self.transaction_log_status.to_string(),
            self.logs_applied.join("|"),
            self.problems.join("; "),
        ]
    }
}

// --- summary ----------------------------------------------------------------------------------

/// Counts for the results summary.
#[derive(Default)]
struct Stats {
    to_stderr: bool,
    hives: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    /// Hives without any of the keys read.
    without: usize,
    records: usize,
    clients: BTreeMap<&'static str, usize>,
    /// The connections whose keys were written last: (key time, line), each line once. Only
    /// records whose key is their own, or that are first in their list: the key's time is
    /// theirs.
    latest: BTreeSet<(u64, String)>,
    /// Sessions that keep a password.
    passwords: Vec<String>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s connections (when there is a writer) and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        user: &User,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        self.hives += 1;
        self.hosts.extend(opened.host.clone());
        self.users.extend(user.username.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        let (records, problems) = connections(&opened.hive);
        if records.is_empty() {
            self.without += 1;
        }
        for problem in &problems {
            if self.to_stderr {
                eprintln!("{}: {problem}", opened.source);
            }
            self.problem(&opened.source, problem);
        }
        let who = user.username.as_deref().unwrap_or("?");
        let host = opened.host.as_deref().unwrap_or(&opened.source);
        for connection in &records {
            self.records += 1;
            let client = connection.kind.client().name();
            *self.clients.entry(client).or_default() += 1;
            for problem in &connection.problems {
                let at = connection.value_name.as_ref().map_or_else(
                    || connection.key_path.clone(),
                    |v| format!(r"{}\{v}", connection.key_path),
                );
                self.problem(&opened.source, &format!("{at}: {problem}"));
            }
            // the key's time is a connection's own for its own key, and for the first of a list
            let own_time = connection.value_name.is_none() || connection.mru_position == Some(0);
            if own_time
                && let (Some(target), Some(at)) = (
                    connection.target.as_deref(),
                    filetime(connection.key_last_written),
                )
            {
                let port = connection.port.map(|p| format!(":{p}")).unwrap_or_default();
                self.latest.insert((
                    connection.key_last_written,
                    format!("{host}  {at}  {who}  {client}  {target}{port}"),
                ));
                // keep the most recent only
                while self.latest.len() > MAX_PRINTED {
                    self.latest.pop_first();
                }
            }
            if connection.password_stored == Some(true) {
                let session = connection
                    .session_name
                    .as_deref()
                    .or(connection.target.as_deref())
                    .unwrap_or_default();
                let which = connection
                    .details
                    .iter()
                    .find(|(name, _)| name == "StoredPasswords")
                    .map_or("", |(_, value)| value.as_str());
                self.passwords
                    .push(format!("{host}  {who}  {client}  {session} ({which})"));
            }
            let record = Record::new(opened, user, connection);
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv())?,
                Some(w) => w.json(&record)?,
                None => {}
            }
        }
        Ok(())
    }

    fn problem(&mut self, source: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {problem}"));
        }
    }

    fn print(&self, no_color: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(no_color, "NTUSER.DAT hives parsed: ", &fmt_num(self.hives));
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        let list = |set: &BTreeSet<String>| set.iter().cloned().collect::<Vec<_>>().join(", ");
        if !self.hosts.is_empty() {
            p_label(no_color, "Hosts: ", &list(&self.hosts));
        }
        if !self.users.is_empty() {
            p_label(no_color, "Users: ", &list(&self.users));
        }
        p_label(
            no_color,
            "Dirty hives recovered with their transaction logs: ",
            &fmt_num(self.recovered),
        );
        if !self.dirty.is_empty() {
            p_label(
                no_color,
                "Dirty hives whose transaction logs were not applied (connections may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if self.without > 0 {
            p_label(
                no_color,
                "Hives without remote connections: ",
                &fmt_num(self.without),
            );
        }
        let clients: Vec<String> = self
            .clients
            .iter()
            .map(|(client, n)| format!("{client} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Connections: ",
            &if clients.is_empty() {
                fmt_num(self.records)
            } else {
                format!("{} ({})", fmt_num(self.records), clients.join(", "))
            },
        );
        if !self.latest.is_empty() {
            p_label(
                no_color,
                "Written last (key last written UTC, user, client, target): ",
                "",
            );
            for (_, line) in self.latest.iter().rev() {
                p(None, &format!("  {line}"), true);
            }
        }
        p_label(
            no_color,
            "Sessions with a stored password (not decoded): ",
            &fmt_num(self.passwords.len()),
        );
        for session in self.passwords.iter().take(MAX_PRINTED) {
            p(None, &format!("  {session}"), true);
        }
        p_label(
            no_color,
            "Keys or values that could not be read: ",
            &fmt_num(self.problem_count),
        );
        for problem in &self.problems {
            p(None, &format!("  {problem}"), true);
        }
        if self.problem_count > self.problems.len() {
            p(
                None,
                &format!(
                    "  … {} more in the Problems column (-o <FILE>)",
                    fmt_num(self.problem_count - self.problems.len())
                ),
                true,
            );
        }
        p(None, "", true);
    }
}
