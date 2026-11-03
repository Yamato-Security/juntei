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

//! `parse-registry system-info`: what every timeline needs to know about a system, read from its
//! `SYSTEM`, `SOFTWARE`, `SAM` and `SECURITY` hives by ys-registry-parser's `system_info` module,
//! with Hayabusa's input options, progress bar and results summary. One record per system: its
//! names, domain or workgroup, Windows version, install time, time zone (and so the local times
//! it kept), last shutdown, processor architecture, code pages, the user who last signed in, its
//! SIDs, the Windows versions it was upgraded from, and its profiles.
//!
//! A system's hives are those in one directory (`Windows\System32\config`, or hives exported
//! together); with `-f`, the hives given, and the others of the four found next to them.

use super::{
    Hosts, Logs, Opened, RecordWriter, files_read, filetime, find_hives, find_in,
    hive_type_of_file, open, sink,
};
use crate::cli::RegistrySystemInfoOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::system::control_set_name;
use registry::system_info::{
    Domains, PreviousVersion, Profile, SoftwareDetails, SystemDetails, TimeZone, WindowsVersion,
    domains, machine_sid, software, system, utc_offset,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::path::{Path, PathBuf};

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Systems and problems listed in the summary.
const MAX_PRINTED: usize = 20;
/// The hives a system's details come from, in this order.
const KINDS: [&str; 4] = ["SYSTEM", "SOFTWARE", "SAM", "SECURITY"];
/// The first build of Windows 11, which keeps `Windows 10` in its `ProductName`.
const WINDOWS_11_BUILD: u32 = 22_000;

/// The hives of one system: at most one of each of [`KINDS`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct System {
    hives: BTreeMap<usize, PathBuf>,
}

impl System {
    #[cfg(test)]
    fn get(&self, kind: &str) -> Option<&PathBuf> {
        let i = KINDS.iter().position(|k| *k == kind)?;
        self.hives.get(&i)
    }

    /// The hive its record names in `SourcePath`: its `SYSTEM`, else the first it has.
    fn source(&self) -> &Path {
        self.hives
            .values()
            .next()
            .map_or(Path::new(""), PathBuf::as_path)
    }

    fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.hives.values()
    }
}

/// The index in [`KINDS`] of the hive at `path` (by its base block's name, else its file name).
fn kind_of(path: &Path) -> Option<usize> {
    let kind = hive_type_of_file(path)?;
    KINDS.iter().position(|k| *k == kind)
}

/// Whether the hive at `path` is named as its kind is (`SYSTEM`, in any case).
fn named_as(path: &Path, kind: usize) -> bool {
    path.file_name()
        .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(KINDS[kind]))
}

/// The systems of the hives under `dir`: those in one directory are one system's. A second hive
/// of a kind in a directory (`SYSTEM.old`) is a system of its own; the one named as its kind is
/// the directory's.
fn systems_in(dir: &Path) -> Vec<System> {
    let mut by_dir: BTreeMap<PathBuf, Vec<(usize, PathBuf)>> = BTreeMap::new();
    for path in find_hives(dir) {
        if let Some(kind) = kind_of(&path) {
            let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
            by_dir.entry(parent).or_default().push((kind, path));
        }
    }
    let mut systems = Vec::new();
    for (_, mut hives) in by_dir {
        // the hives named as their kinds first
        hives.sort_by_key(|(kind, path)| (!named_as(path, *kind), path.clone()));
        let mut system = System::default();
        for (kind, path) in hives {
            match system.hives.entry(kind) {
                Entry::Occupied(_) => systems.push(System {
                    hives: BTreeMap::from([(kind, path)]),
                }),
                Entry::Vacant(slot) => {
                    slot.insert(path);
                }
            }
        }
        systems.push(system);
    }
    systems.sort_by(|a, b| a.source().cmp(b.source()));
    systems
}

/// The system of the hives given (`-f`), with the other kinds found next to them.
fn system_of(files: &[PathBuf]) -> Result<System, String> {
    let mut system = System::default();
    for path in files {
        if !path.is_file() {
            return Err(format!("The file does not exist: {}", path.display()));
        }
        let kind = kind_of(path).ok_or_else(|| {
            format!(
                "{} is not a SYSTEM, SOFTWARE, SAM or SECURITY hive",
                path.display()
            )
        })?;
        if let Some(other) = system.hives.insert(kind, path.clone()) {
            return Err(format!(
                "Two {} hives were given ({} and {}): give the hives of one system",
                KINDS[kind],
                other.display(),
                path.display()
            ));
        }
    }
    let dirs: Vec<PathBuf> = system
        .paths()
        .map(|p| {
            p.parent()
                .filter(|d| !d.as_os_str().is_empty())
                .unwrap_or(Path::new("."))
                .to_path_buf()
        })
        .collect();
    for (i, kind) in KINDS.iter().enumerate() {
        if system.hives.contains_key(&i) {
            continue;
        }
        if let Some(found) = dirs
            .iter()
            .filter_map(|dir| find_in(dir, &[kind]))
            .find(|found| kind_of(found) == Some(i))
        {
            system.hives.insert(i, found);
        }
    }
    Ok(system)
}

/// Run the command; returns the exit status.
pub fn run(opts: &RegistrySystemInfoOptions) -> i32 {
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
    let systems = match (&opts.directory, opts.filepath.as_slice()) {
        (Some(dir), _) if !dir.is_dir() => {
            Err(format!("The directory does not exist: {}", dir.display()))
        }
        (Some(dir), _) => {
            let found = systems_in(dir);
            if found.is_empty() {
                Err(format!(
                    "No SYSTEM, SOFTWARE, SAM or SECURITY hives were found in {}",
                    dir.display()
                ))
            } else {
                Ok(found)
            }
        }
        (None, files) => system_of(files).map(|s| vec![s]),
    };
    let inputs = systems.and_then(|systems| {
        let hives: Vec<PathBuf> = systems.iter().flat_map(System::paths).cloned().collect();
        // the hives, their logs, and the SYSTEM and SOFTWARE hives found for their hosts
        let sink = sink(opts.output.as_ref(), opts.clobber, data_on_stdout, || {
            files_read(&hives, &[])
        })?;
        Ok((systems, sink))
    });
    let (systems, sink) = match inputs {
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
    let hive_count: usize = systems.iter().map(|s| s.hives.len()).sum();
    if !data_on_stdout {
        let total: u64 = systems
            .iter()
            .flat_map(System::paths)
            .map(|p| p.metadata().map_or(0, |m| m.len()))
            .sum();
        p_label(no_color, "Systems: ", &fmt_num(systems.len()));
        p_label(no_color, "Hives: ", &fmt_num(hive_count));
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
        progress::progress_bar(hive_count, no_color)
    });

    // --- parse and write -------------------------------------------------------------------
    let logs = if opts.no_logs {
        Logs::Without
    } else {
        Logs::NextToIt
    };
    let mut hosts = Hosts::new(!opts.no_logs);
    let mut stats = Stats::default();
    for system in &systems {
        let mut read = Read::default();
        for (&i, path) in &system.hives {
            if let Some(b) = &bar {
                b.set_message(progress::message(path));
            }
            let opened = open(path, logs).and_then(|opened| {
                if opened.hive_type == KINDS[i] {
                    Ok(opened)
                } else {
                    Err(format!(
                        "a {} hive, not a {} hive",
                        opened.hive_type, KINDS[i]
                    ))
                }
            });
            match opened {
                Ok(mut opened) => {
                    if read.host.is_none() {
                        read.host = hosts.of(path, &opened);
                    }
                    opened.host.clone_from(&read.host);
                    read.add(i, opened);
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
        if read.opened.is_empty() {
            continue;
        }
        let record = Record::new(&read);
        stats.system(&read, &record);
        let written = match writer.as_mut() {
            Some(w) if w.is_csv() => w.csv(&record.csv()),
            Some(w) => w.json(&record),
            None => Ok(()),
        };
        if let Err(e) = written {
            report_error(
                bar.as_ref(),
                &format!(
                    "Could not write the record of {}: {e}",
                    system.source().display()
                ),
            );
            return 1;
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
    i32::from(stats.systems == 0)
}

// --- one system's hives -----------------------------------------------------------------------

/// What one system's hives say.
#[derive(Default)]
struct Read {
    host: Option<String>,
    /// Each hive read, by its index in [`KINDS`].
    opened: BTreeMap<usize, Opened>,
    system: Option<SystemDetails>,
    software: Option<SoftwareDetails>,
    sam_sid: Option<String>,
    domains: Option<Domains>,
}

impl Read {
    fn add(&mut self, kind: usize, opened: Opened) {
        match KINDS[kind] {
            "SYSTEM" => self.system = Some(system(&opened.hive)),
            "SOFTWARE" => self.software = Some(software(&opened.hive)),
            "SAM" => self.sam_sid = machine_sid(&opened.hive),
            _ => self.domains = domains(&opened.hive),
        }
        self.opened.insert(kind, opened);
    }

    fn source(&self, kind: &str) -> Option<&str> {
        let i = KINDS.iter().position(|k| *k == kind)?;
        self.opened.get(&i).map(|o| o.source.as_str())
    }

    /// Whether the replay wrote what was read from the hive of `kind`.
    fn replayed(&self, kind: usize) -> bool {
        match KINDS[kind] {
            "SYSTEM" => self.system.as_ref().is_some_and(|s| s.replayed),
            "SOFTWARE" => self.software.as_ref().is_some_and(|s| {
                s.version.as_ref().is_some_and(|v| v.replayed)
                    || s.profiles.iter().any(|p| p.replayed)
                    || s.last_logged_on.as_ref().is_some_and(|l| l.replayed)
            }),
            "SAM" => {
                self.sam_sid.is_some()
                    && self.opened.get(&kind).is_some_and(|opened| {
                        opened.hive.key(r"SAM\Domains\Account").is_some_and(|key| {
                            key.status() == registry::Status::Replayed
                                || key
                                    .value("V")
                                    .is_some_and(|v| v.status() == registry::Status::Replayed)
                        })
                    })
            }
            _ => self.domains.as_ref().is_some_and(|d| d.replayed),
        }
    }

    /// How the transaction logs bear on the record, over all its hives: `not_applied` when one
    /// is dirty without logs that apply, else `replayed` when the replay wrote something read,
    /// else `unchanged` when logs were applied, else `clean`.
    fn log_status(&self) -> &'static str {
        let statuses: Vec<&str> = self
            .opened
            .iter()
            .map(|(&i, o)| o.log_status(self.replayed(i)))
            .collect();
        ["not_applied", "replayed", "unchanged"]
            .into_iter()
            .find(|s| statuses.contains(s))
            .unwrap_or("clean")
    }

    /// Every problem, each with the hive it is in, and what the hives disagree on.
    fn problems(&self) -> Vec<String> {
        let mut all = Vec::new();
        let mut add = |kind: &str, problems: &[String]| {
            all.extend(problems.iter().map(|p| format!("{kind}: {p}")));
        };
        if let Some(s) = &self.system {
            add("SYSTEM", &s.problems);
            for previous in &s.previous_versions {
                add("SYSTEM", &previous.version.problems);
            }
        }
        if let Some(s) = &self.software {
            add("SOFTWARE", &s.problems);
            if let Some(v) = &s.version {
                add("SOFTWARE", &v.problems);
            }
            for profile in &s.profiles {
                add("SOFTWARE", &profile.problems);
            }
            if let Some(l) = &s.last_logged_on {
                add("SOFTWARE", &l.problems);
            }
        }
        if let Some(d) = &self.domains {
            add("SECURITY", &d.problems);
        }
        // hives of different systems, or a SID changed in one of them
        if let (Some(sam), Some(security)) = (
            &self.sam_sid,
            self.domains
                .as_ref()
                .and_then(|d| d.account_domain_sid.as_ref()),
        ) && !sam.eq_ignore_ascii_case(security)
        {
            all.push(format!(
                "the SAM's computer SID {sam} and the SECURITY hive's (PolAcDmS) {security} differ: hives of different systems?"
            ));
        }
        all
    }
}

/// `Bias` and another bias added to it, as an offset from UTC.
fn offset(bias: Option<i32>, plus: Option<i32>) -> Option<String> {
    Some(utc_offset(bias?.checked_add(plus.unwrap_or(0))?))
}

/// The time zone in words, the local times the system kept: `Eastern Standard Time, UTC-05:00;
/// daylight time UTC-04:00 from the second Sunday of March, 02:00 to the first Sunday of
/// November, 02:00`.
pub fn describe_time_zone(zone: &TimeZone) -> String {
    let mut text = zone.name().unwrap_or("(unnamed)").to_string();
    if let Some(standard) = offset(zone.bias, zone.standard_bias) {
        text.push_str(&format!(", {standard}"));
    }
    let starts = (
        zone.daylight_start.and_then(|d| d.describe()),
        zone.standard_start.and_then(|d| d.describe()),
    );
    match (starts, offset(zone.bias, zone.daylight_bias)) {
        ((Some(daylight), Some(standard)), Some(daylight_offset)) => text.push_str(&format!(
            "; daylight time {daylight_offset} from the {daylight} to the {standard}"
        )),
        ((None, None), _) => text.push_str("; no daylight saving time"),
        _ => {}
    }
    if zone.dynamic_daylight_time_disabled == Some(true) {
        text.push_str(" (DynamicDaylightTimeDisabled: dynamic daylight saving time off)");
    }
    text
}

/// A Windows version in words: `Windows 10 Pro (Professional) 21H2, build 22000.434`.
fn describe_version(v: &WindowsVersion) -> String {
    let mut text = v.product_name.clone().unwrap_or_else(|| "Windows".into());
    if let Some(edition) = &v.edition_id {
        text.push_str(&format!(" ({edition})"));
    }
    if let Some(release) = display_version(v) {
        text.push_str(&format!(" {release}"));
    }
    if let Some(sp) = &v.csd_version {
        text.push_str(&format!(" {sp}"));
    }
    if let Some(build) = &v.current_build {
        text.push_str(&format!(", build {build}"));
        if let Some(ubr) = v.ubr {
            text.push_str(&format!(".{ubr}"));
        }
    }
    text
}

/// `DisplayVersion` (`21H2`), else `ReleaseId` (`1809`).
fn display_version(v: &WindowsVersion) -> Option<&str> {
    v.display_version.as_deref().or(v.release_id.as_deref())
}

/// `10.0` (`CurrentMajorVersionNumber` and `…Minor…`), else `CurrentVersion` (`6.1`).
fn version_number(v: &WindowsVersion) -> Option<String> {
    match (v.major_version, v.minor_version) {
        (Some(major), Some(minor)) => Some(format!("{major}.{minor}")),
        _ => v.current_version.clone(),
    }
}

/// Whether a version calls itself Windows 10 with a build of Windows 11.
fn is_windows_11(v: &WindowsVersion) -> bool {
    v.product_name
        .as_deref()
        .is_some_and(|p| p.contains("Windows 10"))
        && v.current_build
            .as_deref()
            .and_then(|b| b.parse::<u32>().ok())
            .is_some_and(|b| b >= WINDOWS_11_BUILD)
}

/// A Unix time in seconds (`InstallDate`) as RFC 3339.
fn unix_time(seconds: u32) -> Option<String> {
    // seconds since 1970 in FILETIME ticks
    filetime(116_444_736_000_000_000 + u64::from(seconds) * 10_000_000)
}

// --- records ----------------------------------------------------------------------------------

/// A Windows version the system was upgraded from, in a record.
#[derive(Serialize)]
struct PreviousRecord<'a> {
    #[serde(rename = "KeyName")]
    key_name: &'a str,
    #[serde(rename = "UpdatedOn")]
    updated_on: Option<&'a str>,
    #[serde(rename = "ProductName")]
    product_name: Option<&'a str>,
    #[serde(rename = "Edition")]
    edition: Option<&'a str>,
    #[serde(rename = "DisplayVersion")]
    display_version: Option<&'a str>,
    #[serde(rename = "Build")]
    build: Option<&'a str>,
    #[serde(rename = "Ubr")]
    ubr: Option<u32>,
    #[serde(rename = "InstallDate")]
    install_date: Option<String>,
    #[serde(rename = "InstallTime")]
    install_time: Option<String>,
    #[serde(rename = "KeyLastWritten")]
    key_last_written: Option<String>,
}

impl<'a> PreviousRecord<'a> {
    fn new(p: &'a PreviousVersion) -> Self {
        let v = &p.version;
        PreviousRecord {
            key_name: &p.key_name,
            updated_on: p.updated_on.as_deref(),
            product_name: v.product_name.as_deref(),
            edition: v.edition_id.as_deref(),
            display_version: display_version(v),
            build: v.current_build.as_deref(),
            ubr: v.ubr,
            install_date: v.install_date.and_then(unix_time),
            install_time: v.install_time.and_then(filetime),
            key_last_written: filetime(v.key_last_written),
        }
    }

    /// `Windows 10 Pro (Professional) 1803, build 17134.765, installed 2019-01-27T10:39:32Z,
    /// upgraded on 2019-05-16T00:55:20`.
    fn csv(p: &PreviousVersion) -> String {
        let mut text = describe_version(&p.version);
        if let Some(date) = p.version.install_date.and_then(unix_time) {
            text.push_str(&format!(", installed {date}"));
        }
        match &p.updated_on {
            Some(on) => text.push_str(&format!(", upgraded on {on}")),
            None => text.push_str(&format!(" ({})", p.key_name)),
        }
        text
    }
}

/// A profile, in a record.
#[derive(Serialize)]
struct ProfileRecord<'a> {
    #[serde(rename = "SID")]
    sid: &'a str,
    #[serde(rename = "Path")]
    path: Option<&'a str>,
    #[serde(rename = "KeyLastWritten")]
    key_last_written: Option<String>,
    #[serde(rename = "LoadTime")]
    load_time: Option<String>,
    #[serde(rename = "LocalLoadTime")]
    local_load_time: Option<String>,
    #[serde(rename = "LocalUnloadTime")]
    local_unload_time: Option<String>,
    #[serde(rename = "State")]
    state: Option<u32>,
}

impl<'a> ProfileRecord<'a> {
    fn new(p: &'a Profile) -> Self {
        ProfileRecord {
            sid: &p.sid,
            path: p.path.as_deref(),
            key_last_written: filetime(p.key_last_written),
            load_time: p.load_time.and_then(filetime),
            local_load_time: p.local_load_time.and_then(filetime),
            local_unload_time: p.local_unload_time.and_then(filetime),
            state: p.state,
        }
    }

    /// `S-1-5-21-…-1001 C:\Users\alice (key 2022-02-07T07:38:00Z, loaded …, unloaded …)`.
    fn csv(&self) -> String {
        let times: Vec<String> = [
            ("key", &self.key_last_written),
            ("loaded", &self.local_load_time),
            ("unloaded", &self.local_unload_time),
            ("profile loaded", &self.load_time),
        ]
        .iter()
        .filter_map(|(what, t)| t.as_ref().map(|t| format!("{what} {t}")))
        .collect();
        format!(
            "{} {} ({})",
            self.sid,
            self.path.unwrap_or("-"),
            times.join(", ")
        )
    }
}

/// A system's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    #[serde(rename = "Host")]
    host: Option<&'a str>,
    #[serde(rename = "SourcePath")]
    source_path: &'a str,
    #[serde(rename = "SoftwarePath")]
    software_path: Option<&'a str>,
    #[serde(rename = "SamPath")]
    sam_path: Option<&'a str>,
    #[serde(rename = "SecurityPath")]
    security_path: Option<&'a str>,
    #[serde(rename = "CompName")]
    computer_name: Option<&'a str>,
    #[serde(rename = "Hostname")]
    hostname: Option<&'a str>,
    #[serde(rename = "Domain")]
    domain: Option<&'a str>,
    #[serde(rename = "PrimaryDomain")]
    primary_domain: Option<&'a str>,
    #[serde(rename = "DNS-Domain")]
    dns_domain: Option<&'a str>,
    #[serde(rename = "ProductName")]
    product_name: Option<&'a str>,
    #[serde(rename = "Edition")]
    edition: Option<&'a str>,
    #[serde(rename = "DisplayVersion")]
    display_version: Option<&'a str>,
    #[serde(rename = "Version")]
    version: Option<String>,
    #[serde(rename = "Build")]
    build: Option<&'a str>,
    #[serde(rename = "Ubr")]
    ubr: Option<u32>,
    #[serde(rename = "ServicePack")]
    service_pack: Option<&'a str>,
    #[serde(rename = "InstallationType")]
    installation_type: Option<&'a str>,
    #[serde(rename = "InstallDate")]
    install_date: Option<String>,
    #[serde(rename = "InstallTime")]
    install_time: Option<String>,
    #[serde(rename = "RegisteredOwner")]
    registered_owner: Option<&'a str>,
    #[serde(rename = "RegisteredOrganization")]
    registered_organization: Option<&'a str>,
    #[serde(rename = "SystemRoot")]
    system_root: Option<&'a str>,
    #[serde(rename = "TimeZone")]
    time_zone: Option<&'a str>,
    #[serde(rename = "TimeZoneDescription")]
    time_zone_description: Option<String>,
    #[serde(rename = "UTC-Offset")]
    utc_offset: Option<String>,
    #[serde(rename = "DaylightUTC-Offset")]
    daylight_utc_offset: Option<String>,
    #[serde(rename = "BiasMinutes")]
    bias_minutes: Option<i32>,
    #[serde(rename = "ActiveBiasMinutes")]
    active_bias_minutes: Option<i32>,
    #[serde(rename = "StandardBiasMinutes")]
    standard_bias_minutes: Option<i32>,
    #[serde(rename = "DaylightBiasMinutes")]
    daylight_bias_minutes: Option<i32>,
    #[serde(rename = "StandardName")]
    standard_name: Option<&'a str>,
    #[serde(rename = "DaylightName")]
    daylight_name: Option<&'a str>,
    #[serde(rename = "StandardStart")]
    standard_start: Option<String>,
    #[serde(rename = "DaylightStart")]
    daylight_start: Option<String>,
    #[serde(rename = "DynamicDaylightTimeDisabled")]
    dynamic_daylight_time_disabled: Option<bool>,
    #[serde(rename = "RealTimeIsUniversal")]
    real_time_is_universal: Option<bool>,
    #[serde(rename = "TimeZoneLastWritten")]
    time_zone_last_written: Option<String>,
    #[serde(rename = "LastShutdown")]
    last_shutdown: Option<String>,
    #[serde(rename = "CurrentControlSet")]
    current_control_set: Option<String>,
    #[serde(rename = "ProcessorArchitecture")]
    processor_architecture: Option<&'a str>,
    #[serde(rename = "AnsiCodePage")]
    ansi_code_page: Option<&'a str>,
    #[serde(rename = "OemCodePage")]
    oem_code_page: Option<&'a str>,
    #[serde(rename = "LastLoggedOnUser")]
    last_logged_on_user: Option<&'a str>,
    #[serde(rename = "LastLoggedOnSamUser")]
    last_logged_on_sam_user: Option<&'a str>,
    #[serde(rename = "LastLoggedOnUserSID")]
    last_logged_on_user_sid: Option<&'a str>,
    #[serde(rename = "LastLoggedOnDisplayName")]
    last_logged_on_display_name: Option<&'a str>,
    #[serde(rename = "LogonUiLastWritten")]
    logon_ui_last_written: Option<String>,
    #[serde(rename = "MachineSID")]
    machine_sid: Option<&'a str>,
    #[serde(rename = "DomainSID")]
    domain_sid: Option<&'a str>,
    #[serde(rename = "MachineAccountSID")]
    machine_account_sid: Option<&'a str>,
    #[serde(rename = "CompNameLastWritten")]
    computer_name_last_written: Option<String>,
    #[serde(rename = "PrimaryDomainLastWritten")]
    primary_domain_last_written: Option<String>,
    #[serde(rename = "VersionLastWritten")]
    version_last_written: Option<String>,
    #[serde(rename = "PreviousVersions")]
    previous_versions: Vec<PreviousRecord<'a>>,
    #[serde(rename = "Profiles")]
    profiles: Vec<ProfileRecord<'a>>,
    #[serde(rename = "TransactionLogStatus")]
    transaction_log_status: &'static str,
    #[serde(rename = "LogsApplied")]
    logs_applied: Vec<String>,
    #[serde(rename = "Problems")]
    problems: Vec<String>,
}

const CSV_HEADER: [&str; 59] = [
    "Host",
    "SourcePath",
    "SoftwarePath",
    "SamPath",
    "SecurityPath",
    "ComputerName",
    "Hostname",
    "Domain",
    "PrimaryDomain",
    "DNS-Domain",
    "ProductName",
    "Edition",
    "DisplayVersion",
    "Version",
    "Build",
    "Ubr",
    "ServicePack",
    "InstallationType",
    "InstallDateUTC",
    "InstallTimeUTC",
    "RegisteredOwner",
    "RegisteredOrganization",
    "SystemRoot",
    "TimeZone",
    "TimeZoneDescription",
    "UTC-Offset",
    "DaylightUTC-Offset",
    "BiasMinutes",
    "ActiveBiasMinutes",
    "StandardBiasMinutes",
    "DaylightBiasMinutes",
    "StandardName",
    "DaylightName",
    "StandardStart",
    "DaylightStart",
    "DynamicDaylightTimeDisabled",
    "RealTimeIsUniversal",
    "TimeZoneLastWrittenUTC",
    "LastShutdownUTC",
    "CurrentControlSet",
    "ProcessorArchitecture",
    "AnsiCodePage",
    "OemCodePage",
    "LastLoggedOnUser",
    "LastLoggedOnSamUser",
    "LastLoggedOnUserSID",
    "LastLoggedOnDisplayName",
    "LogonUiLastWrittenUTC",
    "MachineSID",
    "DomainSID",
    "MachineAccountSID",
    "ComputerNameLastWrittenUTC",
    "PrimaryDomainLastWrittenUTC",
    "VersionLastWrittenUTC",
    "PreviousVersions",
    "Profiles",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

impl<'a> Record<'a> {
    fn new(read: &'a Read) -> Self {
        let s = read.system.as_ref();
        let zone = s.and_then(|s| s.time_zone.as_ref());
        let sw = read.software.as_ref();
        let v = sw.and_then(|sw| sw.version.as_ref());
        let logon = sw.and_then(|sw| sw.last_logged_on.as_ref());
        let d = read.domains.as_ref();
        let text = |o: Option<&'a Option<String>>| o.and_then(Option::as_deref);
        let daylight = zone
            .filter(|z| z.daylight_start.is_some_and(|d| d.is_set()))
            .and_then(|z| offset(z.bias, z.daylight_bias));
        Record {
            host: read.host.as_deref(),
            source_path: read
                .opened
                .values()
                .next()
                .map_or("", |o| o.source.as_str()),
            software_path: read.source("SOFTWARE"),
            sam_path: read.source("SAM"),
            security_path: read.source("SECURITY"),
            // the SYSTEM's name, else LSA's (the account domain is the computer)
            computer_name: text(s.map(|s| &s.computer_name))
                .or(text(d.map(|d| &d.account_domain_name))),
            hostname: text(s.map(|s| &s.hostname)),
            domain: text(s.map(|s| &s.domain)),
            primary_domain: text(d.map(|d| &d.primary_domain_name)),
            dns_domain: text(d.map(|d| &d.dns_domain_name)),
            product_name: text(v.map(|v| &v.product_name)),
            edition: text(v.map(|v| &v.edition_id)),
            display_version: v.and_then(display_version),
            version: v.and_then(version_number),
            build: text(v.map(|v| &v.current_build)),
            ubr: v.and_then(|v| v.ubr),
            service_pack: text(v.map(|v| &v.csd_version)),
            installation_type: text(v.map(|v| &v.installation_type)),
            install_date: v.and_then(|v| v.install_date).and_then(unix_time),
            install_time: v.and_then(|v| v.install_time).and_then(filetime),
            registered_owner: text(v.map(|v| &v.registered_owner)),
            registered_organization: text(v.map(|v| &v.registered_organization)),
            system_root: text(v.map(|v| &v.system_root)),
            time_zone: zone.and_then(TimeZone::name),
            time_zone_description: zone.map(describe_time_zone),
            utc_offset: zone.and_then(|z| offset(z.bias, z.standard_bias)),
            daylight_utc_offset: daylight,
            bias_minutes: zone.and_then(|z| z.bias),
            active_bias_minutes: zone.and_then(|z| z.active_time_bias),
            standard_bias_minutes: zone.and_then(|z| z.standard_bias),
            daylight_bias_minutes: zone.and_then(|z| z.daylight_bias),
            standard_name: text(zone.map(|z| &z.standard_name)),
            daylight_name: text(zone.map(|z| &z.daylight_name)),
            standard_start: zone.and_then(|z| z.standard_start?.describe()),
            daylight_start: zone.and_then(|z| z.daylight_start?.describe()),
            dynamic_daylight_time_disabled: zone.and_then(|z| z.dynamic_daylight_time_disabled),
            real_time_is_universal: zone.and_then(|z| z.real_time_is_universal),
            time_zone_last_written: zone.and_then(|z| filetime(z.key_last_written)),
            last_shutdown: s.and_then(|s| s.last_shutdown).and_then(filetime),
            current_control_set: s.and_then(|s| s.control_set).map(control_set_name),
            processor_architecture: text(s.map(|s| &s.processor_architecture)),
            ansi_code_page: text(s.map(|s| &s.ansi_code_page)),
            oem_code_page: text(s.map(|s| &s.oem_code_page)),
            last_logged_on_user: text(logon.map(|l| &l.user)),
            last_logged_on_sam_user: text(logon.map(|l| &l.sam_user)),
            last_logged_on_user_sid: text(logon.map(|l| &l.user_sid)),
            last_logged_on_display_name: text(logon.map(|l| &l.display_name)),
            logon_ui_last_written: logon.and_then(|l| filetime(l.key_last_written)),
            // the SAM's, else LSA's account domain SID: the same on every public pair read
            machine_sid: read
                .sam_sid
                .as_deref()
                .or(text(d.map(|d| &d.account_domain_sid))),
            domain_sid: text(d.map(|d| &d.primary_domain_sid)),
            machine_account_sid: text(d.map(|d| &d.machine_account_sid)),
            computer_name_last_written: s
                .and_then(|s| s.computer_name_last_written)
                .and_then(filetime),
            primary_domain_last_written: d
                .and_then(|d| d.primary_domain_last_written)
                .and_then(filetime),
            version_last_written: v.and_then(|v| filetime(v.key_last_written)),
            previous_versions: s
                .map(|s| {
                    s.previous_versions
                        .iter()
                        .map(PreviousRecord::new)
                        .collect()
                })
                .unwrap_or_default(),
            profiles: sw
                .map(|sw| sw.profiles.iter().map(ProfileRecord::new).collect())
                .unwrap_or_default(),
            transaction_log_status: read.log_status(),
            logs_applied: read
                .opened
                .values()
                .flat_map(|o| o.recovery.logs_applied.iter().cloned())
                .collect(),
            problems: read.problems(),
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let num = |v: Option<i64>| v.map(|n| n.to_string()).unwrap_or_default();
        let flag = |b: Option<bool>| {
            b.map(|b| if b { "True" } else { "False" }.to_string())
                .unwrap_or_default()
        };
        let previous: Vec<String> = self
            .previous_versions
            .iter()
            .map(|p| {
                let mut text = format!(
                    "{}{}",
                    p.product_name.unwrap_or("Windows"),
                    p.display_version
                        .map(|d| format!(" {d}"))
                        .unwrap_or_default()
                );
                if let Some(build) = p.build {
                    text.push_str(&format!(", build {build}"));
                    if let Some(ubr) = p.ubr {
                        text.push_str(&format!(".{ubr}"));
                    }
                }
                if let Some(date) = &p.install_date {
                    text.push_str(&format!(", installed {date}"));
                }
                match p.updated_on {
                    Some(on) => text.push_str(&format!(", upgraded on {on}")),
                    None => text.push_str(&format!(" ({})", p.key_name)),
                }
                text
            })
            .collect();
        vec![
            text(self.host),
            self.source_path.to_string(),
            text(self.software_path),
            text(self.sam_path),
            text(self.security_path),
            text(self.computer_name),
            text(self.hostname),
            text(self.domain),
            text(self.primary_domain),
            text(self.dns_domain),
            text(self.product_name),
            text(self.edition),
            text(self.display_version),
            text(self.version.as_deref()),
            text(self.build),
            num(self.ubr.map(i64::from)),
            text(self.service_pack),
            text(self.installation_type),
            text(self.install_date.as_deref()),
            text(self.install_time.as_deref()),
            text(self.registered_owner),
            text(self.registered_organization),
            text(self.system_root),
            text(self.time_zone),
            text(self.time_zone_description.as_deref()),
            text(self.utc_offset.as_deref()),
            text(self.daylight_utc_offset.as_deref()),
            num(self.bias_minutes.map(i64::from)),
            num(self.active_bias_minutes.map(i64::from)),
            num(self.standard_bias_minutes.map(i64::from)),
            num(self.daylight_bias_minutes.map(i64::from)),
            text(self.standard_name),
            text(self.daylight_name),
            text(self.standard_start.as_deref()),
            text(self.daylight_start.as_deref()),
            flag(self.dynamic_daylight_time_disabled),
            flag(self.real_time_is_universal),
            text(self.time_zone_last_written.as_deref()),
            text(self.last_shutdown.as_deref()),
            text(self.current_control_set.as_deref()),
            text(self.processor_architecture),
            text(self.ansi_code_page),
            text(self.oem_code_page),
            text(self.last_logged_on_user),
            text(self.last_logged_on_sam_user),
            text(self.last_logged_on_user_sid),
            text(self.last_logged_on_display_name),
            text(self.logon_ui_last_written.as_deref()),
            text(self.machine_sid),
            text(self.domain_sid),
            text(self.machine_account_sid),
            text(self.computer_name_last_written.as_deref()),
            text(self.primary_domain_last_written.as_deref()),
            text(self.version_last_written.as_deref()),
            previous.join(" | "),
            self.profiles
                .iter()
                .map(ProfileRecord::csv)
                .collect::<Vec<_>>()
                .join(" | "),
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
    systems: usize,
    failed: usize,
    hives: BTreeMap<&'static str, usize>,
    recovered: usize,
    dirty: Vec<String>,
    /// Each system's lines, the first `MAX_PRINTED`.
    listed: Vec<Vec<String>>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Count a system's record, and list it.
    fn system(&mut self, read: &Read, record: &Record) {
        self.systems += 1;
        for (&i, opened) in &read.opened {
            *self.hives.entry(KINDS[i]).or_default() += 1;
            if opened.recovery.recovered {
                self.recovered += 1;
            } else if opened.recovery.dirty {
                self.dirty.push(opened.source.clone());
            }
        }
        for problem in &record.problems {
            self.problem_count += 1;
            if self.problems.len() < MAX_PRINTED {
                self.problems
                    .push(format!("{}: {problem}", record.source_path));
            }
        }
        if self.listed.len() >= MAX_PRINTED {
            return;
        }
        let name = record
            .host
            .or(record.computer_name)
            .unwrap_or("(unknown host)");
        let dir = Path::new(record.source_path)
            .parent()
            .map_or_else(String::new, |d| d.display().to_string());
        let mut lines = vec![format!("{name}  ({dir})")];
        let mut line = |label: &str, value: Option<String>| {
            if let Some(value) = value {
                lines.push(format!("  {label:<16}{value}"));
            }
        };
        let version = read.software.as_ref().and_then(|s| s.version.as_ref());
        line(
            "Windows:",
            version.map(|v| {
                let mut text = describe_version(v);
                if is_windows_11(v) {
                    text.push_str(" (Windows 11: build 22000 or later)");
                }
                text
            }),
        );
        line(
            "Installed:",
            record
                .install_time
                .clone()
                .or_else(|| record.install_date.clone()),
        );
        line("Time zone:", record.time_zone_description.clone());
        if let (Some(active), Some(when)) =
            (record.active_bias_minutes, &record.time_zone_last_written)
        {
            line(
                "Active offset:",
                Some(format!("{} when last written ({when})", utc_offset(active))),
            );
        }
        line("Last shutdown:", record.last_shutdown.clone());
        line(
            "Domain:",
            record.primary_domain.map(|domain| match record.domain_sid {
                Some(sid) => format!(
                    "{domain}{} (joined, {sid})",
                    record
                        .dns_domain
                        .map(|d| format!(", {d}"))
                        .unwrap_or_default()
                ),
                None => format!("{domain} (workgroup)"),
            }),
        );
        line(
            "Last logged on:",
            record.last_logged_on_user.map(str::to_string),
        );
        line("Computer SID:", record.machine_sid.map(str::to_string));
        line(
            "Profiles:",
            (!record.profiles.is_empty()).then(|| fmt_num(record.profiles.len())),
        );
        if let Some(system) = &read.system {
            for previous in &system.previous_versions {
                line("Upgraded from:", Some(PreviousRecord::csv(previous)));
            }
        }
        self.listed.push(lines);
    }

    fn print(&self, no_color: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(no_color, "Systems: ", &fmt_num(self.systems));
        let hives: Vec<String> = KINDS
            .iter()
            .filter_map(|k| self.hives.get(k).map(|n| format!("{k} {}", fmt_num(*n))))
            .collect();
        p_label(no_color, "Hives parsed: ", &hives.join(", "));
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        p_label(
            no_color,
            "Dirty hives recovered with their transaction logs: ",
            &fmt_num(self.recovered),
        );
        if !self.dirty.is_empty() {
            p_label(
                no_color,
                "Dirty hives whose transaction logs were not applied (values may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        p(None, "", true);
        for lines in &self.listed {
            for line in lines {
                p(None, line, true);
            }
            p(None, "", true);
        }
        if self.systems > self.listed.len() {
            p(
                None,
                &format!(
                    "… {} more systems in the records (-o <FILE>)",
                    fmt_num(self.systems - self.listed.len())
                ),
                true,
            );
        }
        p_label(
            no_color,
            "Values that could not be read or don't match: ",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn hive(path: &Path, name: &str) {
        // a base block naming the hive, enough for its kind
        let mut base = vec![0u8; 4096];
        base[..4].copy_from_slice(b"regf");
        let units: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
        base[0x30..0x30 + units.len()].copy_from_slice(&units);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, base).unwrap();
    }

    #[test]
    fn systems_of_a_collection() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("H1/C/Windows/System32/config");
        for kind in KINDS {
            hive(&config.join(kind), kind);
        }
        hive(&config.join("DEFAULT"), "DEFAULT");
        hive(&config.join("RegBack/SYSTEM"), "SYSTEM");
        hive(&config.join("SYSTEM.old"), r"\REGISTRY\MACHINE\SYSTEM");
        hive(&dir.path().join("exported/software.hiv"), "SOFTWARE");
        let systems = systems_in(dir.path());
        let names: Vec<Vec<String>> = systems
            .iter()
            .map(|s| {
                s.paths()
                    .map(|p| {
                        p.strip_prefix(dir.path())
                            .unwrap()
                            .to_string_lossy()
                            .replace('\\', "/")
                    })
                    .collect()
            })
            .collect();
        assert_eq!(
            names,
            [
                vec!["H1/C/Windows/System32/config/RegBack/SYSTEM".to_string()],
                vec![
                    "H1/C/Windows/System32/config/SYSTEM".to_string(),
                    "H1/C/Windows/System32/config/SOFTWARE".to_string(),
                    "H1/C/Windows/System32/config/SAM".to_string(),
                    "H1/C/Windows/System32/config/SECURITY".to_string(),
                ],
                vec!["H1/C/Windows/System32/config/SYSTEM.old".to_string()],
                vec!["exported/software.hiv".to_string()],
            ]
        );
        // -f: the hives given, and the others next to them
        let given = system_of(&[config.join("SOFTWARE")]).unwrap();
        assert_eq!(given.paths().count(), 4);
        assert_eq!(given.get("SYSTEM"), Some(&config.join("SYSTEM")));
        let err = system_of(&[config.join("SYSTEM"), config.join("SYSTEM.old")]).unwrap_err();
        assert!(err.contains("Two SYSTEM hives"), "{err}");
        let err = system_of(&[config.join("DEFAULT")]).unwrap_err();
        assert!(
            err.contains("is not a SYSTEM, SOFTWARE, SAM or SECURITY hive"),
            "{err}"
        );
    }

    #[test]
    fn time_zones_in_words() {
        use registry::system_info::TransitionDate;
        let date = |month, day, hour, weekday| TransitionDate {
            month,
            day,
            hour,
            weekday,
            ..TransitionDate::default()
        };
        let mut zone = TimeZone {
            key_name: Some("Eastern Standard Time".into()),
            bias: Some(300),
            standard_bias: Some(0),
            daylight_bias: Some(-60),
            standard_start: Some(date(11, 1, 2, 0)),
            daylight_start: Some(date(3, 2, 2, 0)),
            ..TimeZone::default()
        };
        assert_eq!(
            describe_time_zone(&zone),
            "Eastern Standard Time, UTC-05:00; daylight time UTC-04:00 from the second Sunday of March, 02:00 to the first Sunday of November, 02:00"
        );
        zone.dynamic_daylight_time_disabled = Some(true);
        assert!(
            describe_time_zone(&zone)
                .ends_with("(DynamicDaylightTimeDisabled: dynamic daylight saving time off)")
        );
        let tokyo = TimeZone {
            key_name: Some("Tokyo Standard Time".into()),
            bias: Some(-540),
            standard_bias: Some(0),
            daylight_bias: Some(-60),
            standard_start: Some(TransitionDate::default()),
            daylight_start: Some(TransitionDate::default()),
            ..TimeZone::default()
        };
        assert_eq!(
            describe_time_zone(&tokyo),
            "Tokyo Standard Time, UTC+09:00; no daylight saving time"
        );
        assert_eq!(
            describe_time_zone(&TimeZone::default()),
            "(unnamed); no daylight saving time"
        );
    }

    #[test]
    fn versions_in_words() {
        let v = WindowsVersion {
            product_name: Some("Windows 10 Pro".into()),
            edition_id: Some("Professional".into()),
            display_version: Some("21H2".into()),
            release_id: Some("2009".into()),
            current_build: Some("22000".into()),
            ubr: Some(434),
            major_version: Some(10),
            minor_version: Some(0),
            current_version: Some("6.3".into()),
            ..WindowsVersion::default()
        };
        assert_eq!(
            describe_version(&v),
            "Windows 10 Pro (Professional) 21H2, build 22000.434"
        );
        assert!(is_windows_11(&v));
        assert_eq!(version_number(&v).as_deref(), Some("10.0"));
        let seven = WindowsVersion {
            product_name: Some("Windows 7 Ultimate".into()),
            csd_version: Some("Service Pack 1".into()),
            current_build: Some("7601".into()),
            current_version: Some("6.1".into()),
            ..WindowsVersion::default()
        };
        assert_eq!(
            describe_version(&seven),
            "Windows 7 Ultimate Service Pack 1, build 7601"
        );
        assert!(!is_windows_11(&seven));
        assert_eq!(version_number(&seven).as_deref(), Some("6.1"));
        assert_eq!(
            unix_time(1_644_087_066).as_deref(),
            Some("2022-02-05T18:51:06Z")
        );
    }
}
