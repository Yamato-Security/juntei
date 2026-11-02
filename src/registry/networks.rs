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

//! `parse-registry networks`: the networks a system joined and how it was set up, from SOFTWARE
//! and SYSTEM hives, read by ys-registry-parser's `networks` module, with Hayabusa's input
//! options, progress bar and results summary. One record per network profile (with its first and
//! last connection times, in UTC with the system's time zone and as found, and the gateway's MAC
//! address and DNS suffix of its signatures), signature no profile has, network card, TCP/IP
//! interface (addresses, DHCP server and lease), share, port proxy and persistent route. The
//! summary lists the networks connected last, the shares, the port proxies and the routes.

use super::{
    Hosts, Logs, Opened, RecordWriter, choose_logs, files_read, filetime, find_hives, find_in,
    hive_type_of_file, input_files, open, sink, software_of, system_root,
};
use crate::cli::RegistryNetworksOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::networks::{
    self, Interface, NetworkCard, PortProxy, Profile, Route, Share, Signature, SystemTime,
    TimeZone, category_name, connection_type_name, share_type_name,
};
use registry::system::{ControlSets, control_set_name};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::path::{Path, PathBuf};

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and items listed in the summary.
const MAX_PRINTED: usize = 20;
/// The hives read, as their base blocks name them.
const HIVE_TYPES: [&str; 2] = ["SOFTWARE", "SYSTEM"];

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryNetworksOptions) -> i32 {
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
        "SOFTWARE or SYSTEM hives",
        |d| {
            find_hives(d)
                .into_iter()
                .filter(|p| hive_type_of_file(p).is_some_and(|t| HIVE_TYPES.contains(&t.as_str())))
                .collect()
        },
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
        p_label(no_color, "Registry hives: ", &fmt_num(files.len()));
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
    let mut systems = Systems::new(!opts.no_logs);
    let mut stats = Stats {
        to_stderr: data_on_stdout,
        ..Stats::default()
    };
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let parsed = open(path, logs).and_then(|mut opened| {
            if !HIVE_TYPES.contains(&opened.hive_type.as_str()) {
                return Err(if opened.hive_type.is_empty() {
                    "not a SOFTWARE or SYSTEM hive (neither its base block nor its file name says which)".to_string()
                } else {
                    format!("a {} hive, not a SOFTWARE or SYSTEM hive", opened.hive_type)
                });
            }
            opened.host = hosts.of(path, &opened);
            let read = Read::of(path, &opened, opts, &mut systems)?;
            Ok((opened, read))
        });
        match parsed {
            Ok((opened, read)) => {
                if let Err(e) = stats.hive(&opened, &read, writer.as_mut()) {
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
        stats.print(opts, no_color);
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
    i32::from(stats.hives.values().sum::<usize>() == 0)
}

// --- the system's other hive -------------------------------------------------------------------

/// The SYSTEM hive of the system the SOFTWARE hive at `path` comes from, as [`Hosts`] finds it:
/// the system's `Windows\System32\config\SYSTEM`, else a SYSTEM hive next to it.
fn system_of(path: &Path) -> Option<PathBuf> {
    system_root(path)
        .and_then(|root| find_in(&root, &["Windows", "System32", "config", "SYSTEM"]))
        .or_else(|| find_in(path.parent()?, &["SYSTEM"]))
        .filter(|found| found != path)
}

/// What one hive's records take from the other hive of its system: a SOFTWARE hive's profiles
/// the time zone of its SYSTEM; a SYSTEM hive's interfaces the adapters of its SOFTWARE.
struct Systems {
    logs: bool,
    time_zones: HashMap<PathBuf, Option<TimeZone>>,
    adapters: HashMap<PathBuf, HashMap<String, String>>,
}

impl Systems {
    fn new(logs: bool) -> Self {
        Systems {
            logs,
            time_zones: HashMap::new(),
            adapters: HashMap::new(),
        }
    }

    fn logs(&self) -> Logs<'static> {
        if self.logs {
            Logs::NextToIt
        } else {
            Logs::Without
        }
    }

    /// The time zone of the control set Windows used last in the SYSTEM hive of the SOFTWARE
    /// hive at `path`.
    fn time_zone(&mut self, path: &Path) -> Option<TimeZone> {
        let system = system_of(path)?;
        let logs = self.logs();
        self.time_zones
            .entry(system)
            .or_insert_with_key(|system| {
                let opened = open(system, logs)
                    .ok()
                    .filter(|o| o.hive_type == "SYSTEM")?;
                let sets = ControlSets::read(&opened.hive);
                let set = sets.current.filter(|set| sets.present.contains(set))?;
                TimeZone::read(&opened.hive, set)
            })
            .clone()
    }

    /// The network cards' adapters, by interface GUID (upper case), in the SOFTWARE hive of the
    /// SYSTEM hive at `path`.
    fn adapters(&mut self, path: &Path) -> HashMap<String, String> {
        let Some(software) = software_of(path) else {
            return HashMap::new();
        };
        let logs = self.logs();
        self.adapters
            .entry(software)
            .or_insert_with_key(|software| {
                open(software, logs)
                    .ok()
                    .filter(|o| o.hive_type == "SOFTWARE")
                    .map(|o| {
                        networks::software(&o.hive)
                            .cards
                            .into_iter()
                            .filter_map(|c| {
                                Some((c.service_name?.to_ascii_uppercase(), c.description?))
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .clone()
    }
}

// --- reading ----------------------------------------------------------------------------------

/// The records of one hive.
struct Read {
    records: Vec<Item>,
    problems: Vec<String>,
    /// A SYSTEM hive without a usable `Select\Current`: its first control set was read.
    without_select: bool,
    control_sets: usize,
}

/// One record's source, with the control set of a SYSTEM hive's.
enum Item {
    Profile(Box<Profile>, Option<TimeZone>),
    Signature(Signature),
    Card(NetworkCard),
    Interface(Interface, ControlSet, Option<String>),
    Share(Share, ControlSet),
    PortProxy(PortProxy, ControlSet),
    Route(Route, ControlSet),
}

/// A SYSTEM hive's control set, and whether it is the current one.
#[derive(Clone, Copy)]
struct ControlSet {
    set: u32,
    current: Option<bool>,
}

impl Read {
    fn of(
        path: &Path,
        opened: &Opened,
        opts: &RegistryNetworksOptions,
        systems: &mut Systems,
    ) -> Result<Self, String> {
        let hive = &opened.hive;
        let mut read = Read {
            records: Vec::new(),
            problems: Vec::new(),
            without_select: false,
            control_sets: 0,
        };
        if opened.hive_type == "SOFTWARE" {
            let found = networks::software(hive);
            let tz = if found.profiles.is_empty() {
                None
            } else {
                systems.time_zone(path)
            };
            read.records.extend(
                found
                    .profiles
                    .into_iter()
                    .map(|p| Item::Profile(Box::new(p), tz.clone())),
            );
            read.records
                .extend(found.signatures.into_iter().map(Item::Signature));
            read.records.extend(found.cards.into_iter().map(Item::Card));
            read.problems = found.problems;
            return Ok(read);
        }
        let sets = ControlSets::read(hive);
        if sets.present.is_empty() {
            return Err("no control set (ControlSet001, …): not a SYSTEM hive".to_string());
        }
        read.without_select = sets
            .current
            .is_none_or(|current| !sets.present.contains(&current));
        if read.without_select {
            read.problems.push(
                "Select\\Current is missing or invalid; the current control set is unknown"
                    .to_owned(),
            );
        }
        let chosen: Vec<u32> = if opts.all_control_sets {
            sets.present.clone()
        } else {
            sets.current_or_first().into_iter().collect()
        };
        let adapters = systems.adapters(path);
        for set in chosen {
            let Some(found) = networks::system(hive, set) else {
                continue;
            };
            read.control_sets += 1;
            let at = ControlSet {
                set,
                current: sets
                    .current
                    .filter(|current| sets.present.contains(current))
                    .map(|current| current == set),
            };
            read.records.extend(found.interfaces.into_iter().map(|i| {
                let adapter = adapters.get(&i.guid.to_ascii_uppercase()).cloned();
                Item::Interface(i, at, adapter)
            }));
            read.records
                .extend(found.shares.into_iter().map(|s| Item::Share(s, at)));
            read.records.extend(
                found
                    .port_proxies
                    .into_iter()
                    .map(|p| Item::PortProxy(p, at)),
            );
            read.records
                .extend(found.routes.into_iter().map(|r| Item::Route(r, at)));
            read.problems.extend(found.problems);
        }
        Ok(read)
    }
}

// --- records ----------------------------------------------------------------------------------

/// A record's details, as a JSON object in their order.
#[derive(Default)]
struct Details(Vec<(String, String)>);

impl Details {
    fn of(pairs: Vec<(&str, String)>) -> Self {
        Details(pairs.into_iter().map(|(n, v)| (n.to_owned(), v)).collect())
    }
}

impl Serialize for Details {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.0.iter().map(|(name, value)| (name, value)))
    }
}

/// A record, its fields in this order.
#[derive(Serialize, Default)]
struct Record<'a> {
    host: Option<&'a str>,
    source_path: &'a str,
    control_set: Option<String>,
    is_current_control_set: Option<bool>,
    source: &'static str,
    profile_name: Option<String>,
    category: Option<String>,
    connection_type: Option<String>,
    first_connected: Option<String>,
    last_connected: Option<String>,
    first_connected_local: Option<String>,
    last_connected_local: Option<String>,
    time_zone: Option<String>,
    gateway_mac: Option<String>,
    dns_suffix: Option<String>,
    interface_guid: Option<String>,
    ip_address: Option<String>,
    dhcp_server: Option<String>,
    lease_obtained: Option<String>,
    lease_expires: Option<String>,
    share_name: Option<String>,
    share_path: Option<String>,
    listen_address: Option<String>,
    listen_port: Option<u16>,
    connect_address: Option<String>,
    connect_port: Option<u16>,
    route: Option<String>,
    details: Details,
    key_path: String,
    value_name: Option<String>,
    key_last_written: Option<String>,
    data_raw: Option<String>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: Vec<String>,
}

const CSV_HEADER: [&str; 35] = [
    "Host",
    "SourcePath",
    "ControlSet",
    "IsCurrentControlSet",
    "Source",
    "ProfileName",
    "Category",
    "ConnectionType",
    "FirstConnectedUTC",
    "LastConnectedUTC",
    "FirstConnectedLocal",
    "LastConnectedLocal",
    "TimeZone",
    "GatewayMac",
    "DnsSuffix",
    "InterfaceGuid",
    "IpAddress",
    "DhcpServer",
    "LeaseObtainedUTC",
    "LeaseExpiresUTC",
    "ShareName",
    "SharePath",
    "ListenAddress",
    "ListenPort",
    "ConnectAddress",
    "ConnectPort",
    "Route",
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

/// A Unix time in UTC (RFC 3339); `None` for 0.
fn unix(seconds: u32) -> Option<String> {
    (seconds != 0)
        .then(|| filetime(u64::from(seconds) * 10_000_000 + 116_444_736_000_000_000))
        .flatten()
}

/// Distinct non-empty values, joined.
fn joined<'v>(values: impl Iterator<Item = Option<&'v str>>) -> Option<String> {
    let mut seen: Vec<&str> = Vec::new();
    for value in values.flatten() {
        if !seen.contains(&value) {
            seen.push(value);
        }
    }
    (!seen.is_empty()).then(|| seen.join("|"))
}

impl<'a> Record<'a> {
    /// A record of `source` from `opened`, with nothing else filled in.
    fn base(opened: &'a Opened, source: &'static str, at: Option<ControlSet>) -> Self {
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            control_set: at.map(|a| control_set_name(a.set)),
            is_current_control_set: at.and_then(|a| a.current),
            source,
            transaction_log_status: opened.log_status(false),
            logs_applied: &opened.recovery.logs_applied,
            ..Record::default()
        }
    }

    fn of(opened: &'a Opened, item: &Item) -> Self {
        let (mut r, replayed) = match item {
            Item::Profile(p, tz) => {
                let mut r = Record::base(opened, "network_profile", None);
                r.profile_name.clone_from(&p.profile_name);
                r.category = p
                    .category
                    .map(|c| category_name(c).map_or_else(|| c.to_string(), str::to_owned));
                r.connection_type = p
                    .name_type
                    .map(|t| connection_type_name(t).map_or_else(|| t.to_string(), str::to_owned));
                let utc = |t: &Option<SystemTime>| {
                    t.as_ref()
                        .and_then(|t| tz.as_ref()?.to_utc(t))
                        .and_then(filetime)
                };
                r.first_connected = utc(&p.date_created);
                r.last_connected = utc(&p.date_last_connected);
                r.first_connected_local = p.date_created.map(|t| t.text());
                r.last_connected_local = p.date_last_connected.map(|t| t.text());
                r.time_zone = tz.as_ref().map(|tz| {
                    tz.name
                        .clone()
                        .unwrap_or_else(|| format!("UTC bias {} minutes", tz.bias))
                });
                r.gateway_mac = joined(p.signatures.iter().map(|s| s.gateway_mac.as_deref()));
                r.dns_suffix = joined(p.signatures.iter().map(|s| s.dns_suffix.as_deref()));
                let mut details = Vec::new();
                if let Some(d) = &p.description {
                    details.push(("Description", d.clone()));
                }
                if let Some(m) = p.managed {
                    details.push(("Managed", if m { "True" } else { "False" }.to_owned()));
                }
                if let Some(first) = joined(p.signatures.iter().map(|s| s.first_network.as_deref()))
                {
                    details.push(("FirstNetwork", first));
                }
                if !p.signatures.is_empty() {
                    let keys: Vec<&str> =
                        p.signatures.iter().map(|s| s.key_path.as_str()).collect();
                    details.push(("Signatures", keys.join("|")));
                }
                r.details = Details::of(details);
                r.key_path.clone_from(&p.key_path);
                r.key_last_written = filetime(p.key_last_written);
                r.data_raw = p.data_raw.as_deref().map(hex);
                r.problems.clone_from(&p.problems);
                for (label, local, utc) in [
                    ("DateCreated", &p.date_created, &r.first_connected),
                    (
                        "DateLastConnected",
                        &p.date_last_connected,
                        &r.last_connected,
                    ),
                ] {
                    if local.is_some() && utc.is_none() {
                        r.problems.push(format!("{label}: UTC conversion unavailable (missing/invalid time zone, DST gap/fold, or unsupported date); local time retained"));
                    }
                }
                for s in &p.signatures {
                    r.problems
                        .extend(s.problems.iter().map(|w| format!("{}: {w}", s.key_path)));
                }
                (r, p.replayed || p.signatures.iter().any(|s| s.replayed))
            }
            Item::Signature(s) => {
                let mut r = Record::base(opened, "network_signature", None);
                r.profile_name.clone_from(&s.description);
                r.gateway_mac.clone_from(&s.gateway_mac);
                r.dns_suffix.clone_from(&s.dns_suffix);
                let mut details = vec![(
                    "Managed",
                    if s.managed { "True" } else { "False" }.to_owned(),
                )];
                if let Some(g) = &s.profile_guid {
                    details.push(("ProfileGuid", g.clone()));
                }
                if let Some(f) = &s.first_network {
                    details.push(("FirstNetwork", f.clone()));
                }
                r.details = Details::of(details);
                r.key_path.clone_from(&s.key_path);
                r.key_last_written = filetime(s.key_last_written);
                r.problems.clone_from(&s.problems);
                (r, s.replayed)
            }
            Item::Card(c) => {
                let mut r = Record::base(opened, "network_card", None);
                r.interface_guid.clone_from(&c.service_name);
                if let Some(d) = &c.description {
                    r.details = Details::of(vec![("Adapter", d.clone())]);
                }
                r.key_path.clone_from(&c.key_path);
                r.key_last_written = filetime(c.key_last_written);
                r.problems.clone_from(&c.problems);
                (r, c.replayed)
            }
            Item::Interface(i, at, adapter) => {
                let mut r = Record::base(opened, "interface", Some(*at));
                r.interface_guid = Some(i.guid.clone());
                r.ip_address = (!i.ip_addresses.is_empty()).then(|| i.ip_addresses.join(","));
                r.dhcp_server.clone_from(&i.dhcp_server);
                r.lease_obtained = i.lease_obtained.and_then(unix);
                r.lease_expires = i.lease_terminates.and_then(unix);
                let mut details = Vec::new();
                if let Some(a) = adapter {
                    details.push(("Adapter", a.clone()));
                }
                if let Some(s) = &i.subkey {
                    details.push(("Subkey", s.clone()));
                }
                if let Some(d) = i.dhcp_enabled {
                    details.push(("DhcpEnabled", if d { "True" } else { "False" }.to_owned()));
                }
                for (name, values) in [
                    ("SubnetMask", &i.subnet_masks),
                    ("DefaultGateway", &i.default_gateways),
                    ("DnsServers", &i.name_servers),
                ] {
                    if !values.is_empty() {
                        details.push((name, values.join(",")));
                    }
                }
                if let Some(d) = &i.domain {
                    details.push(("Domain", d.clone()));
                }
                r.details = Details::of(details);
                r.key_path.clone_from(&i.key_path);
                r.key_last_written = filetime(i.key_last_written);
                r.problems.clone_from(&i.problems);
                (r, i.replayed)
            }
            Item::Share(s, at) => {
                let mut r = Record::base(opened, "share", Some(*at));
                r.share_name = Some(s.name.clone());
                r.share_path.clone_from(&s.path);
                let mut details = Vec::new();
                if let Some(t) = s.share_type {
                    details.push(("Type", format!("{t} ({})", share_type_name(t))));
                }
                if let Some(remark) = &s.remark {
                    details.push(("Remark", remark.clone()));
                }
                for (name, value) in &s.fields {
                    if !["Path", "Remark", "Type", "ShareName"]
                        .iter()
                        .any(|n| n.eq_ignore_ascii_case(name))
                    {
                        details.push((name.as_str(), value.clone()));
                    }
                }
                r.details = Details::of(details);
                r.key_path.clone_from(&s.key_path);
                r.value_name = Some(s.name.clone());
                r.key_last_written = filetime(s.key_last_written);
                r.data_raw = s.data_raw.as_deref().map(hex);
                r.problems.clone_from(&s.problems);
                (r, s.replayed)
            }
            Item::PortProxy(p, at) => {
                let mut r = Record::base(opened, "port_proxy", Some(*at));
                r.listen_address = Some(p.listen_address.clone());
                r.listen_port = p.listen_port;
                r.connect_address.clone_from(&p.connect_address);
                r.connect_port = p.connect_port;
                r.details = Details::of(vec![
                    ("Kind", p.kind.clone()),
                    ("Protocol", "tcp".to_owned()),
                ]);
                r.key_path.clone_from(&p.key_path);
                r.value_name = Some(p.value_name.clone());
                r.key_last_written = filetime(p.key_last_written);
                r.data_raw = p.data_raw.as_deref().map(hex);
                r.problems.clone_from(&p.problems);
                (r, p.replayed)
            }
            Item::Route(route, at) => {
                let mut r = Record::base(opened, "route", Some(*at));
                r.route = Some(route.value_name.clone());
                let mut details = Vec::new();
                for (name, value) in [
                    ("Destination", &route.destination),
                    ("Netmask", &route.netmask),
                    ("Gateway", &route.gateway),
                    ("Metric", &route.metric),
                ] {
                    if let Some(v) = value {
                        details.push((name, v.clone()));
                    }
                }
                r.details = Details::of(details);
                r.key_path.clone_from(&route.key_path);
                r.value_name = Some(route.value_name.clone());
                r.key_last_written = filetime(route.key_last_written);
                r.problems.clone_from(&route.problems);
                (r, route.replayed)
            }
        };
        r.transaction_log_status = opened.log_status(replayed);
        r
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: &Option<String>| v.clone().unwrap_or_default();
        let num = |v: Option<u16>| v.map(|n| n.to_string()).unwrap_or_default();
        let flag = |b: Option<bool>| match b {
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
            self.host.unwrap_or_default().to_string(),
            self.source_path.to_string(),
            text(&self.control_set),
            flag(self.is_current_control_set),
            self.source.to_string(),
            text(&self.profile_name),
            text(&self.category),
            text(&self.connection_type),
            text(&self.first_connected),
            text(&self.last_connected),
            text(&self.first_connected_local),
            text(&self.last_connected_local),
            text(&self.time_zone),
            text(&self.gateway_mac),
            text(&self.dns_suffix),
            text(&self.interface_guid),
            text(&self.ip_address),
            text(&self.dhcp_server),
            text(&self.lease_obtained),
            text(&self.lease_expires),
            text(&self.share_name),
            text(&self.share_path),
            text(&self.listen_address),
            num(self.listen_port),
            text(&self.connect_address),
            num(self.connect_port),
            text(&self.route),
            details.join("; "),
            self.key_path.clone(),
            text(&self.value_name),
            text(&self.key_last_written),
            text(&self.data_raw),
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
    /// Hives read, by type.
    hives: BTreeMap<String, usize>,
    failed: usize,
    hosts: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without_select: Vec<String>,
    control_sets: usize,
    by_source: BTreeMap<&'static str, usize>,
    /// Profiles without their system's time zone: their times stay local.
    without_time_zone: usize,
    /// The networks connected last: (time, line), the most recent kept.
    latest: BTreeSet<(String, String)>,
    /// Shares, port proxies and routes: lines, each once across control sets.
    shares: BTreeSet<String>,
    proxies: BTreeSet<String>,
    routes: BTreeSet<String>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `read` (when there is a writer) and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        read: &Read,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        *self.hives.entry(opened.hive_type.clone()).or_default() += 1;
        self.hosts.extend(opened.host.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        if read.without_select {
            self.without_select.push(opened.source.clone());
        }
        self.control_sets += read.control_sets;
        for problem in &read.problems {
            if self.to_stderr {
                eprintln!("{}: {problem}", opened.source);
            }
            self.problem(&opened.source, problem);
        }
        let host = opened.host.as_deref().unwrap_or(&opened.source);
        for item in &read.records {
            let record = Record::of(opened, item);
            *self.by_source.entry(record.source).or_default() += 1;
            for problem in &record.problems {
                self.problem(&opened.source, &format!("{}: {problem}", record.key_path));
            }
            match item {
                Item::Profile(p, tz) => {
                    if tz.is_none() {
                        self.without_time_zone += 1;
                    }
                    let when = record
                        .last_connected
                        .clone()
                        .or_else(|| record.last_connected_local.clone().map(|t| t + " (local)"));
                    if let Some(when) = when {
                        let line = format!(
                            "{host}  {when}  {}  {}{}",
                            p.profile_name.as_deref().unwrap_or("?"),
                            record.connection_type.as_deref().unwrap_or("?"),
                            record
                                .gateway_mac
                                .as_deref()
                                .map_or_else(String::new, |m| format!("  gateway {m}"))
                        );
                        self.latest.insert((when, line));
                        while self.latest.len() > MAX_PRINTED {
                            self.latest.pop_first();
                        }
                    }
                }
                Item::Share(s, _) => {
                    self.shares.insert(format!(
                        "{host}  {}  {}",
                        s.name,
                        s.path.as_deref().unwrap_or("?")
                    ));
                }
                Item::PortProxy(p, _) => {
                    self.proxies.insert(format!(
                        "{host}  {}  {}:{} -> {}:{}",
                        p.kind,
                        p.listen_address,
                        p.listen_port.map_or("?".to_owned(), |n| n.to_string()),
                        p.connect_address.as_deref().unwrap_or("?"),
                        p.connect_port.map_or("?".to_owned(), |n| n.to_string())
                    ));
                }
                Item::Route(r, _) => {
                    self.routes.insert(format!("{host}  {}", r.value_name));
                }
                _ => {}
            }
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

    /// A labelled list, at most [`MAX_PRINTED`] of it.
    fn list(no_color: bool, label: &str, lines: &BTreeSet<String>) {
        p_label(no_color, label, &fmt_num(lines.len()));
        for line in lines.iter().take(MAX_PRINTED) {
            p(None, &format!("  {line}"), true);
        }
        if lines.len() > MAX_PRINTED {
            p(
                None,
                &format!(
                    "  … {} more (-o <FILE>)",
                    fmt_num(lines.len() - MAX_PRINTED)
                ),
                true,
            );
        }
    }

    fn print(&self, opts: &RegistryNetworksOptions, no_color: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        let hives: Vec<String> = self
            .hives
            .iter()
            .map(|(kind, n)| format!("{kind} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Hives parsed: ",
            &format!(
                "{}{}",
                fmt_num(self.hives.values().sum::<usize>()),
                if hives.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", hives.join(", "))
                }
            ),
        );
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        if !self.hosts.is_empty() {
            let hosts: Vec<&str> = self.hosts.iter().map(String::as_str).collect();
            p_label(no_color, "Hosts: ", &hosts.join(", "));
        }
        p_label(
            no_color,
            "Dirty hives recovered with their transaction logs: ",
            &fmt_num(self.recovered),
        );
        if !self.dirty.is_empty() {
            p_label(
                no_color,
                "Dirty hives whose transaction logs were not applied (records may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if !self.without_select.is_empty() {
            p_label(
                no_color,
                "Hives without usable Select\\Current (the first control set was read): ",
                &fmt_num(self.without_select.len()),
            );
        }
        if self.control_sets > 0 {
            p_label(
                no_color,
                "SYSTEM control sets read: ",
                &format!(
                    "{}{}",
                    fmt_num(self.control_sets),
                    if opts.all_control_sets {
                        ""
                    } else {
                        " (the one in use; --all-control-sets for every one)"
                    }
                ),
            );
        }
        let sources: Vec<String> = self
            .by_source
            .iter()
            .map(|(s, n)| format!("{s} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Records: ",
            &if sources.is_empty() {
                "0".to_owned()
            } else {
                format!(
                    "{} ({})",
                    fmt_num(self.by_source.values().sum::<usize>()),
                    sources.join(", ")
                )
            },
        );
        if self.without_time_zone > 0 {
            p_label(
                no_color,
                "Profiles without their system's SYSTEM hive (times local only): ",
                &fmt_num(self.without_time_zone),
            );
        }
        if !self.latest.is_empty() {
            p_label(
                no_color,
                "Connected last (UTC, network, type, gateway): ",
                "",
            );
            for (_, line) in self.latest.iter().rev() {
                p(None, &format!("  {line}"), true);
            }
        }
        Self::list(no_color, "Shares: ", &self.shares);
        Self::list(
            no_color,
            "Port proxies (netsh interface portproxy): ",
            &self.proxies,
        );
        Self::list(no_color, "Persistent routes: ", &self.routes);
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
