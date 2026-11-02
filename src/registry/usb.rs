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

//! `parse-registry usb`: USB storage, USB and portable devices and the volumes they held, read
//! by ys-registry-parser's `usb` module from each SYSTEM hive and its system's SOFTWARE and
//! users' NTUSER.DAT hives, with Hayabusa's input options, progress bar and results summary. One
//! record per device and volume of its: its names, IDs and serial number, its install, first
//! install, last arrival and last removal times, the drive letters and volume GUID
//! `MountedDevices` gives it, its volume's names and serial numbers, and the users whose
//! `MountPoints2` has the volume. Then the volumes no device key names (an internal disk's, or a
//! removed device's), and the devices only SOFTWARE names. The summary lists the USB storage
//! devices that arrived last.

use super::{
    Hosts, Logs, Opened, RecordWriter, choose_logs, files_read, filetime, find_hives_of_type,
    input_files, open, profile_of_path, sink, software_of, users_of,
};
use crate::cli::RegistryUsbOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::ntuser::profile_name;
use registry::system::ControlSets;
use registry::usb::{
    self, DeviceClass, DeviceRef, History, MountTarget, Sources, serial, storage_name, vid_pid,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and devices listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryUsbOptions) -> i32 {
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
        "SYSTEM hives",
        |d| find_hives_of_type(d, "SYSTEM"),
    )
    .and_then(|files| Ok((files, choose_logs(opts.no_logs, &given_logs)?)))
    .and_then(|(files, logs)| {
        let sink = sink(opts.output.as_ref(), opts.clobber, data_on_stdout, || {
            // the SOFTWARE and user hives of each system are read too
            let mut read = files.clone();
            for system in &files {
                read.extend(software_of(system));
                read.extend(users_of(system));
            }
            files_read(&read, &given_logs)
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
        p_label(no_color, "SYSTEM hives: ", &fmt_num(files.len()));
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
    // The SYSTEM hive's own logs are those given or next to it; its system's other hives use
    // the logs next to them (or none, with --no-logs).
    let companion_logs = if opts.no_logs {
        Logs::Without
    } else {
        Logs::NextToIt
    };
    let mut hosts = Hosts::new(!opts.no_logs);
    let mut stats = Stats {
        to_stderr: data_on_stdout,
        ..Stats::default()
    };
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let parsed = open(path, logs).and_then(|mut opened| {
            if !opened.hive_type.is_empty() && opened.hive_type != "SYSTEM" {
                return Err(format!("a {} hive, not a SYSTEM hive", opened.hive_type));
            }
            opened.host = hosts.of(path, &opened);
            Ok(opened)
        });
        match parsed {
            Ok(system) => {
                let system_of = SystemOf::read(path, companion_logs, &mut stats);
                if let Err(e) = stats.system(&system, &system_of, opts, writer.as_mut()) {
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

// --- the hives of a system ----------------------------------------------------------------------

/// The other hives of a SYSTEM hive's system: its SOFTWARE hive and its users' NTUSER.DAT
/// hives, each with the name of its user (its profile folder's, else the one its shell folders
/// give).
struct SystemOf {
    software: Option<Opened>,
    users: Vec<(String, Opened)>,
}

impl SystemOf {
    fn read(system: &Path, logs: Logs, stats: &mut Stats) -> Self {
        let mut open_as = |path: &Path, kind: &str| match open(path, logs) {
            Ok(opened) if opened.hive_type.is_empty() || opened.hive_type == kind => Some(opened),
            Ok(opened) => {
                stats.problem(
                    &path.to_string_lossy(),
                    "hive",
                    &format!("a {} hive, not a {kind} hive", opened.hive_type),
                );
                None
            }
            Err(e) => {
                stats.problem(&path.to_string_lossy(), "hive", &e);
                None
            }
        };
        let software = software_of(system).and_then(|p| open_as(&p, "SOFTWARE"));
        let users = users_of(system)
            .iter()
            .filter_map(|path| {
                let opened = open_as(path, "NTUSER.DAT")?;
                let name = profile_of_path(path)
                    .or_else(|| profile_name(&opened.hive))
                    .unwrap_or_else(|| opened.source.clone());
                Some((name, opened))
            })
            .collect();
        SystemOf { software, users }
    }

    /// The opened user hive of user `user`.
    fn user(&self, source: &str) -> Option<(&str, &Opened)> {
        self.users
            .iter()
            .find(|(_, opened)| opened.source == source)
            .map(|(name, opened)| (name.as_str(), opened))
    }
}

// --- records ----------------------------------------------------------------------------------

/// A user who had a record's volume mounted.
#[derive(Serialize)]
struct UserRecord<'a> {
    #[serde(rename = "Username")]
    username: &'a str,
    #[serde(rename = "MountPoints2LastWritten")]
    mount_points2_last_written: Option<String>,
    #[serde(rename = "SourcePath")]
    source_path: Option<&'a str>,
}

/// A device's or volume's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    #[serde(rename = "Host")]
    host: Option<&'a str>,
    #[serde(rename = "DeviceClass")]
    device_class: &'static str,
    #[serde(rename = "ControlSet")]
    control_set: Option<u32>,
    #[serde(rename = "Vendor")]
    vendor: Option<String>,
    #[serde(rename = "Product")]
    product: Option<String>,
    #[serde(rename = "Revision")]
    revision: Option<String>,
    #[serde(rename = "Vid")]
    vid: Option<String>,
    #[serde(rename = "PID")]
    pid: Option<String>,
    #[serde(rename = "Serial")]
    serial: Option<String>,
    #[serde(rename = "Instance")]
    instance: Option<&'a str>,
    #[serde(rename = "FriendlyName")]
    friendly_name: Option<&'a str>,
    #[serde(rename = "DeviceDesc")]
    device_desc: Option<&'a str>,
    #[serde(rename = "Service")]
    service: Option<&'a str>,
    #[serde(rename = "ContainerID")]
    container_id: Option<&'a str>,
    #[serde(rename = "DeviceFirstInstall")]
    device_first_install: Option<String>,
    #[serde(rename = "DriverLastInstalled")]
    driver_last_installed: Option<String>,
    #[serde(rename = "LastArrival")]
    last_arrival: Option<String>,
    #[serde(rename = "LastRemoval")]
    last_removal: Option<String>,
    #[serde(rename = "DriveLetters")]
    drive_letters: Vec<String>,
    #[serde(rename = "VolumeGUID")]
    volume_guid: Option<&'a str>,
    #[serde(rename = "VolumeData")]
    volume_data: Option<String>,
    #[serde(rename = "VolumeNames")]
    volume_names: &'a [String],
    #[serde(rename = "VolumeSerials")]
    volume_serials: Vec<String>,
    #[serde(rename = "Users")]
    users: Vec<UserRecord<'a>>,
    #[serde(rename = "FoundIn")]
    found_in: &'a [&'static str],
    #[serde(rename = "SourceKeys")]
    source_keys: &'a [String],
    #[serde(rename = "KeyPath")]
    key_path: Option<&'a str>,
    #[serde(rename = "KeyLastWritten")]
    key_last_written: Option<String>,
    #[serde(rename = "SourcePath")]
    source_path: &'a str,
    #[serde(rename = "SoftwarePath")]
    software_path: Option<&'a str>,
    #[serde(rename = "TransactionLogStatus")]
    transaction_log_status: &'static str,
    #[serde(rename = "LogsApplied")]
    logs_applied: Vec<String>,
    #[serde(rename = "Problems")]
    problems: &'a [String],
}

const CSV_HEADER: [&str; 33] = [
    "Host",
    "DeviceClass",
    "ControlSet",
    "Vendor",
    "Product",
    "Revision",
    "Vid",
    "PID",
    "Serial",
    "Instance",
    "FriendlyName",
    "DeviceDesc",
    "Service",
    "ContainerID",
    "DeviceFirstInstallUTC",
    "DriverLastInstalledUTC",
    "LastArrivalUTC",
    "LastRemovalUTC",
    "DriveLetters",
    "VolumeGUID",
    "VolumeData",
    "VolumeNames",
    "VolumeSerials",
    "Users",
    "FoundIn",
    "SourceKeys",
    "KeyPath",
    "KeyLastWrittenUTC",
    "SourcePath",
    "SoftwarePath",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// A volume serial number as Windows shows it: `1234-ABCD`.
fn volume_serial(n: u32) -> String {
    format!("{:04X}-{:04X}", n >> 16, n & 0xffff)
}

/// What `MountedDevices` data says a volume is on.
fn volume_data(target: &MountTarget) -> Option<String> {
    match target {
        MountTarget::Mbr { signature, offset } => Some(format!(
            "MBR disk signature {signature:08X}, partition offset {offset}"
        )),
        MountTarget::Gpt(guid) => Some(format!("GPT partition {guid}")),
        MountTarget::Path(path) => Some(path.clone()),
        MountTarget::Other => None,
    }
}

/// The class of a record: its device's, else `volume` for a volume or drive letter no device
/// key names, `portable_device` or `readyboost` for a device only SOFTWARE names.
fn class(h: &History) -> &'static str {
    if let Some(d) = &h.device {
        d.class.name()
    } else if h
        .found_in
        .iter()
        .any(|f| matches!(*f, "MountedDevices" | "MountPoints2"))
    {
        "volume"
    } else if h.found_in.contains(&"Windows Portable Devices") {
        "portable_device"
    } else {
        "readyboost"
    }
}

/// The name, IDs and serial number of a device a key or value names.
fn named(r: &DeviceRef) -> [Option<String>; 6] {
    if r.enumerator.eq_ignore_ascii_case("USBSTOR") {
        let [_, vendor, product, revision] = storage_name(&r.device);
        [
            vendor,
            product,
            revision,
            None,
            None,
            serial(&r.enumerator, &r.instance),
        ]
    } else {
        let (vid, pid) = vid_pid(&r.device);
        let serial = vid
            .is_some()
            .then(|| serial(&r.enumerator, &r.instance))
            .flatten();
        [None, None, None, vid, pid, serial]
    }
}

/// The most telling of the transaction log statuses of the hives a record was joined from.
fn combined_status(statuses: &[&'static str]) -> &'static str {
    ["not_applied", "replayed", "unchanged", "clean"]
        .into_iter()
        .find(|s| statuses.contains(s))
        .unwrap_or("clean")
}

impl<'a> Record<'a> {
    fn new(system: &'a Opened, of: &'a SystemOf, h: &'a History) -> Self {
        let d = h.device.as_ref();
        let [vendor, product, revision, vid, pid, serial] = match (d, &h.named) {
            (Some(d), _) => {
                let (vid, pid) = h.vid_pid.clone().unwrap_or((d.vid.clone(), d.pid.clone()));
                [
                    d.vendor.clone(),
                    d.product.clone(),
                    d.revision.clone(),
                    vid,
                    pid,
                    d.serial.clone(),
                ]
            }
            (None, Some(r)) => named(r),
            (None, None) => Default::default(),
        };
        let instance = d
            .and_then(|d| d.reference.as_ref())
            .or(h.named.as_ref())
            .map(|r| r.instance.as_str());
        // the statuses and logs of the hives it was joined from
        let mut statuses = vec![system.log_status(h.replayed_in.contains(&"SYSTEM"))];
        let mut logs: Vec<String> = system.recovery.logs_applied.clone();
        let software = of.software.as_ref().filter(|_| {
            h.found_in
                .iter()
                .any(|f| matches!(*f, "Windows Portable Devices" | "EMDMgmt"))
        });
        if let Some(s) = software {
            statuses.push(s.log_status(h.replayed_in.contains(&"SOFTWARE")));
            logs.extend(s.recovery.logs_applied.iter().cloned());
        }
        let users: Vec<UserRecord> = h
            .users
            .iter()
            .map(|u| {
                let source = of.user(&u.user);
                let hive = source.map(|(_, opened)| opened);
                if let Some(hive) = hive {
                    statuses.push(hive.log_status(h.replayed_in.contains(&"NTUSER.DAT")));
                    logs.extend(hive.recovery.logs_applied.iter().cloned());
                }
                UserRecord {
                    username: source.map_or(u.user.as_str(), |(name, _)| name),
                    mount_points2_last_written: filetime(u.last_written),
                    source_path: hive.map(|o| o.source.as_str()),
                }
            })
            .collect();
        logs.sort();
        logs.dedup();
        Record {
            host: system.host.as_deref(),
            device_class: class(h),
            control_set: d.map(|d| d.control_set),
            vendor,
            product,
            revision,
            vid,
            pid,
            serial,
            instance,
            friendly_name: d.and_then(|d| d.friendly_name.as_deref()),
            device_desc: d.and_then(|d| d.device_desc.as_deref()),
            service: d.and_then(|d| d.service.as_deref()),
            container_id: d.and_then(|d| d.container_id.as_deref()),
            device_first_install: d.and_then(|d| d.device_first_install).and_then(filetime),
            driver_last_installed: d.and_then(|d| d.driver_last_installed).and_then(filetime),
            last_arrival: d.and_then(|d| d.last_arrival).and_then(filetime),
            last_removal: d.and_then(|d| d.last_removal).and_then(filetime),
            drive_letters: h.drive_letters.iter().map(|l| format!("{l}:")).collect(),
            volume_guid: h.volume_guid.as_deref(),
            volume_data: h.mount_target.as_ref().and_then(volume_data),
            volume_names: &h.volume_names,
            volume_serials: h.volume_serials.iter().map(|&n| volume_serial(n)).collect(),
            users,
            found_in: &h.found_in,
            source_keys: &h.keys,
            key_path: d.map(|d| d.path.as_str()),
            key_last_written: d.and_then(|d| filetime(d.last_written)),
            source_path: &system.source,
            software_path: software.map(|s| s.source.as_str()),
            transaction_log_status: combined_status(&statuses),
            logs_applied: logs,
            problems: &h.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let owned = |v: &Option<String>| v.clone().unwrap_or_default();
        vec![
            text(self.host),
            self.device_class.to_string(),
            self.control_set.map(|n| n.to_string()).unwrap_or_default(),
            owned(&self.vendor),
            owned(&self.product),
            owned(&self.revision),
            owned(&self.vid),
            owned(&self.pid),
            owned(&self.serial),
            text(self.instance),
            text(self.friendly_name),
            text(self.device_desc),
            text(self.service),
            text(self.container_id),
            owned(&self.device_first_install),
            owned(&self.driver_last_installed),
            owned(&self.last_arrival),
            owned(&self.last_removal),
            self.drive_letters.join("|"),
            text(self.volume_guid),
            owned(&self.volume_data),
            self.volume_names.join("|"),
            self.volume_serials.join("|"),
            self.users
                .iter()
                .map(|u| match &u.mount_points2_last_written {
                    Some(t) => format!("{} ({t})", u.username),
                    None => u.username.to_owned(),
                })
                .collect::<Vec<_>>()
                .join("|"),
            self.found_in.join("|"),
            self.source_keys.join(" | "),
            text(self.key_path),
            owned(&self.key_last_written),
            self.source_path.to_string(),
            text(self.software_path),
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
    /// Whether the records go to stdout (`-t` without `-o`): no summary, so what concerns a hive
    /// rather than one record goes to stderr.
    to_stderr: bool,
    hives: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without_software: usize,
    user_hives: usize,
    users: BTreeSet<String>,
    records: usize,
    classes: BTreeMap<&'static str, usize>,
    /// Volumes whose `MountedDevices` data names a USB storage device without a device key.
    removed: usize,
    /// The USB storage devices that arrived last: (time, line).
    latest: Vec<(u64, String)>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of the SYSTEM hive `system` and its system's other hives (when there
    /// is a writer) and count them.
    fn system(
        &mut self,
        system: &Opened,
        of: &SystemOf,
        opts: &RegistryUsbOptions,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        self.hives += 1;
        self.hosts.extend(system.host.clone());
        for opened in std::iter::once(system)
            .chain(&of.software)
            .chain(of.users.iter().map(|(_, o)| o))
        {
            if opened.recovery.recovered {
                self.recovered += 1;
            } else if opened.recovery.dirty {
                self.dirty.push(opened.source.clone());
            }
        }
        if of.software.is_none() {
            self.without_software += 1;
        }
        self.user_hives += of.users.len();
        let sets = ControlSets::read(&system.hive);
        let read: Vec<u32> = if opts.all_control_sets {
            sets.present.clone()
        } else {
            sets.current_or_first().into_iter().collect()
        };
        let mut devices = Vec::new();
        for set in read {
            if let Some((found, problems)) = usb::devices(&system.hive, set) {
                for problem in &problems {
                    self.hive_problem(&system.source, "Enum", problem);
                }
                devices.extend(found);
            }
        }
        let mounted = self.list(
            &system.source,
            "MountedDevices",
            usb::mounted_devices(&system.hive),
        );
        let (portable, ready_boost) = match &of.software {
            Some(s) => (
                self.list(
                    &s.source,
                    "Windows Portable Devices",
                    usb::portable_devices(&s.hive),
                ),
                self.list(&s.source, "EMDMgmt", usb::ready_boost(&s.hive)),
            ),
            None => (Vec::new(), Vec::new()),
        };
        let users: Vec<(String, Vec<usb::MountPoint>)> = of
            .users
            .iter()
            .map(|(_, opened)| {
                let points = self.list(
                    &opened.source,
                    "MountPoints2",
                    usb::mount_points(&opened.hive),
                );
                // The parser joins on a unique source identity, never a display username.
                (opened.source.clone(), points)
            })
            .collect();
        let history = usb::history(&Sources {
            devices: &devices,
            mounted: &mounted,
            portable: &portable,
            ready_boost: &ready_boost,
            users: &users,
        });
        for h in &history {
            self.count(system, of, h);
            let record = Record::new(system, of, h);
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv())?,
                Some(w) => w.json(&record)?,
                None => {}
            }
        }
        Ok(())
    }

    /// The items a reader found, its problems counted.
    fn list<T>(&mut self, source: &str, at: &str, read: Option<(Vec<T>, Vec<String>)>) -> Vec<T> {
        let (items, problems) = read.unwrap_or_default();
        for problem in &problems {
            self.hive_problem(source, at, problem);
        }
        items
    }

    /// Count record `h` of `system`.
    fn count(&mut self, system: &Opened, of: &SystemOf, h: &History) {
        self.records += 1;
        *self.classes.entry(class(h)).or_default() += 1;
        self.users.extend(h.users.iter().map(|u| {
            of.user(&u.user)
                .map_or(u.user.clone(), |(name, _)| name.to_owned())
        }));
        if h.device.is_none()
            && h.volume_guid.is_some()
            && h.named
                .as_ref()
                .is_some_and(|r| r.enumerator.eq_ignore_ascii_case("USBSTOR"))
        {
            self.removed += 1;
        }
        let where_ = h.device.as_ref().map_or_else(
            || h.volume_guid.clone().unwrap_or_default(),
            |d| d.path.clone(),
        );
        for problem in &h.problems {
            self.problem(&system.source, &where_, problem);
        }
        // USB storage devices by their last arrival (else their driver's last install), each volume once
        if let Some(d) = h
            .device
            .as_ref()
            .filter(|d| d.class == DeviceClass::UsbStorage)
            && let Some((when, at)) = d
                .last_arrival
                .or(d.driver_last_installed)
                .and_then(|when| Some((when, filetime(when)?)))
        {
            let name = d
                .friendly_name
                .clone()
                .or_else(|| d.product.clone())
                .unwrap_or_default();
            let serial = d.serial.as_deref().unwrap_or("-");
            let letters: String = h.drive_letters.iter().map(|l| format!(" {l}:")).collect();
            let line = format!("{at}  {name} (serial {serial}){letters}");
            if !self.latest.iter().any(|(_, l)| *l == line) {
                self.latest.push((when, line));
            }
            if self.latest.len() > MAX_PRINTED * 4 {
                self.trim_latest();
            }
        }
    }

    /// The most recent arrivals, newest first, at most [`MAX_PRINTED`].
    fn trim_latest(&mut self) {
        self.latest.sort_by(|a, b| b.cmp(a));
        self.latest.truncate(MAX_PRINTED);
    }

    /// A problem with a key of a hive, not with one record.
    fn hive_problem(&mut self, source: &str, at: &str, problem: &str) {
        if self.to_stderr {
            eprintln!("{source}: {at}: {problem}");
        }
        self.problem(source, at, problem);
    }

    fn problem(&mut self, source: &str, at: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {at}: {problem}"));
        }
    }

    fn print(&mut self, no_color: bool) {
        self.trim_latest();
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(no_color, "SYSTEM hives parsed: ", &fmt_num(self.hives));
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        let list = |set: &BTreeSet<String>| set.iter().cloned().collect::<Vec<_>>().join(", ");
        if !self.hosts.is_empty() {
            p_label(no_color, "Hosts: ", &list(&self.hosts));
        }
        if self.without_software > 0 {
            p_label(
                no_color,
                "Systems without their SOFTWARE hive (no portable device names or ReadyBoost volumes): ",
                &fmt_num(self.without_software),
            );
        }
        p_label(no_color, "User hives read: ", &fmt_num(self.user_hives));
        if !self.users.is_empty() {
            p_label(
                no_color,
                "Users with volumes in MountPoints2: ",
                &list(&self.users),
            );
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
        let classes: Vec<String> = self
            .classes
            .iter()
            .map(|(class, n)| format!("{class} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Records: ",
            &if classes.is_empty() {
                fmt_num(self.records)
            } else {
                format!("{} ({})", fmt_num(self.records), classes.join(", "))
            },
        );
        p_label(
            no_color,
            "Volumes of USB storage devices without a device key (removed devices): ",
            &fmt_num(self.removed),
        );
        if !self.latest.is_empty() {
            p_label(
                no_color,
                "USB storage devices that arrived last (UTC, device, letters): ",
                "",
            );
            for (_, line) in &self.latest {
                p(None, &format!("  {line}"), true);
            }
        }
        p_label(
            no_color,
            "Records or keys that could not be read: ",
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
