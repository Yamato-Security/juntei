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

use super::adapters::{Policy, project};
use super::model::{Context, Point};
use serde_json::{Value, json};
use std::path::PathBuf;

fn ctx(artifact: &str) -> Context {
    Context {
        parsed_file: PathBuf::from("parsed.jsonl"),
        artifact: artifact.into(),
        computer: "HOST".into(),
        raw_source: Some("/evidence/source".into()),
        input_index: 0,
        record: 1,
        location: String::new(),
    }
}
fn policy() -> Policy {
    Policy {
        assumed: None,
        start: None,
        end: None,
        collection: false,
        interpreted: false,
    }
}
const DATE: &str = "2026-10-20T08:46:38.200166800Z";

#[test]
fn mft_si_is_shared_by_entry_but_fn_names_remain_distinct() {
    let row = json!({"EntryNum":"10","SequenceNumber":"3","ParentEntryNumber":"5","ParentPath":"Windows",
        "FileName":"first.exe","NameType":"Win32","InUse":"True","Created (SI)":DATE,"Created (FN)":DATE});
    let a = project(&row, &ctx("mft"), &policy());
    let mut other = row.clone();
    other["FileName"] = json!("second.exe");
    let b = project(&other, &ctx("mft"), &policy());
    assert_eq!(a.events.len(), 2);
    let si = |p: &super::adapters::Projection| {
        p.events
            .iter()
            .find(|e| e.event.timestamp_type.contains("SI"))
            .unwrap()
            .identity
            .clone()
    };
    let filename = |p: &super::adapters::Projection| {
        p.events
            .iter()
            .find(|e| e.event.timestamp_type.contains("FN"))
            .unwrap()
            .identity
            .clone()
    };
    assert_eq!(si(&a), si(&b));
    assert_ne!(filename(&a), filename(&b));
    other["IsADS"] = json!("True");
    assert!(project(&other, &ctx("mft"), &policy()).events.is_empty());
    other["IsADS"] = json!("False");
    other["NameType"] = json!("DOS");
    let dos = project(&other, &ctx("mft"), &policy());
    assert_eq!(dos.events.len(), 1);
    assert!(dos.events[0].event.timestamp_type.contains("FN"));
}

#[test]
fn native_mft_attributes_keep_si_and_fn_even_when_times_match() {
    let v = json!({"FullPath":"Windows\\x.exe","Header":{"RecordNumber":10,"Sequence":3},"Attributes":[
        {"Header":{"TypeCode":"StandardInformation"},"Data":{"Modified":DATE,"Created":DATE}},
        {"Header":{"TypeCode":"FileName"},"Data":{"Created":DATE,"Name":"x.exe","Namespace":"Win32","Parent":{"Entry":5}}}
    ]});
    let p = project(&v, &ctx("mft"), &policy());
    assert_eq!(p.events.len(), 3);
    assert!(p.events.iter().all(|e| e.time.nanos == 200166800));
    assert_eq!(
        p.events
            .iter()
            .map(|e| &e.identity)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
}

#[test]
fn prefetch_native_csv_and_timeline_share_execution_identity() {
    let native =
        json!({"SourceFile":"/evidence/x.pf","ExecutablePath":"C:\\x.exe","LastRunTimes":[DATE]});
    let flat = json!({"SourceFile":"/evidence/x.pf","ExecutablePath":"C:\\x.exe","LastRun":DATE});
    let timeline =
        json!({"SourceFile":"/evidence/x.pf","ExecutablePath":"C:\\x.exe","RunTime":DATE});
    let a = project(&native, &ctx("prefetch"), &policy());
    let b = project(&flat, &ctx("prefetch"), &policy());
    let c = project(&timeline, &ctx("prefetch-timeline"), &policy());
    assert_eq!(a.events[0].identity, b.events[0].identity);
    assert_eq!(a.events[0].identity, c.events[0].identity);
    assert_eq!(c.events[0].event.artifact, "prefetch");
}

#[test]
fn prefetch_csv_volume_lists_use_the_actual_export_separator() {
    let row = json!({"LastRun":DATE,"VolumeCreated":format!("{DATE}, {DATE}"),"VolumeDevicePaths":"vol1, vol2"});
    let p = project(&row, &ctx("prefetch"), &policy());
    assert_eq!(p.events.len(), 3);
    assert_eq!(p.counts.invalid_timestamps, 0);
    assert!(p.events.iter().any(|e| e.event.path == "vol2"));
}

#[test]
fn registry_values_do_not_become_key_write_events_and_qword_times_are_opt_in() {
    let value = json!({"RecordType":"value","KeyPath":"A\\B","ValueName":"Counter","ValueFiletimeUTC":DATE,"KeyLastWritten":DATE});
    let p = project(&value, &ctx("registry"), &policy());
    assert!(p.events.is_empty());
    assert_eq!(p.counts.interpreted_times_omitted, 1);
    let mut settings = policy();
    settings.interpreted = true;
    let p = project(&value, &ctx("registry"), &settings);
    assert_eq!(p.events.len(), 1);
    assert_eq!(p.events[0].event.details["TimeRole"], "interpreted_value");
    let key = json!({"RecordType":"key","KeyPath":"A\\B","KeyLastWritten":DATE});
    assert_eq!(project(&key, &ctx("registry"), &policy()).events.len(), 1);
}

#[test]
fn registry_key_rows_consolidate_without_collapsing_distinct_accounts_or_recovery() {
    let row = json!({"SourcePath":"/evidence/SYSTEM","ServiceName":"Example","ControlSet":1,"KeyLastWritten":DATE,"FailureActionType":"restart"});
    let mut other = row.clone();
    other["FailureActionType"] = json!("run_command");
    assert_eq!(
        project(&row, &ctx("services"), &policy()).events[0].identity,
        project(&other, &ctx("services"), &policy()).events[0].identity
    );
    let a = json!({"SourcePath":"/evidence/SAM","RID":500,"Username":"A","KeyLastWritten":DATE});
    let b = json!({"SourcePath":"/evidence/SAM","RID":501,"Username":"B","KeyLastWritten":DATE});
    assert_ne!(
        project(&a, &ctx("sam"), &policy()).events[0].identity,
        project(&b, &ctx("sam"), &policy()).events[0].identity
    );
    let active = json!({"RecordType":"key","KeyPath":"K","KeyLastWritten":DATE,"AllocationStatus":"allocated"});
    let mut deleted = active.clone();
    deleted["AllocationStatus"] = json!("deleted_complete");
    assert_ne!(
        project(&active, &ctx("registry"), &policy()).events[0].identity,
        project(&deleted, &ctx("registry"), &policy()).events[0].identity
    );
}

#[test]
fn amcache_times_are_inventory_and_pe_metadata_not_execution() {
    let v = json!({"RecordType":"file","Source":"/evidence/Amcache.hve","SourceKeyPath":"Inventory\\X","Name":"x.exe","KeyLastWrittenUTC":DATE,"LinkDate":"10/20/2026 12:09:16"});
    let p = project(&v, &ctx("amcache"), &policy());
    assert_eq!(p.events.len(), 2);
    assert!(
        p.events
            .iter()
            .all(|e| e.event.details["TimeRole"] != "execution")
    );
    assert!(
        p.events
            .iter()
            .any(|e| e.event.timestamp == "2026-10-20T12:09:16.000000000Z")
    );
    let summary = json!({"RecordType":"summary","Source":"/evidence/Amcache.hve"});
    assert!(
        project(&summary, &ctx("amcache"), &policy())
            .events
            .is_empty()
    );
}

#[test]
fn srum_buckets_preserve_counters_and_intervals_without_inventing_a_duration_timestamp() {
    let v = json!({"Provider":"NetworkUsages","ID":9,"Timestamp":DATE,"EXE-Info":"\\device\\x.exe","BytesSent":123544,"ConnectedTime":123456});
    let p = project(&v, &ctx("srum"), &policy());
    assert_eq!(p.events.len(), 1);
    assert_eq!(p.events[0].event.details["TimeRole"], "usage_bucket");
    assert_eq!(p.events[0].event.details["BytesSent"], 123544);
    assert!(p.events[0].event.timestamp_type.contains("NetworkUsages"));
}

#[test]
fn shortcut_target_times_and_collection_times_have_different_roles() {
    let v = json!({"SourceFile":"/evidence/x.lnk","SourceModified":DATE,"TargetPath":"C:\\target","Header":{"TargetCreated":DATE,"TargetAccessed":DATE}});
    let p = project(&v, &ctx("lnk"), &policy());
    assert_eq!(p.events.len(), 2);
    assert!(
        p.events
            .iter()
            .all(|e| e.event.details["TimeRole"] == "target_metadata")
    );
    let mut settings = policy();
    settings.collection = true;
    let p = project(&v, &ctx("lnk"), &settings);
    assert_eq!(p.events.len(), 3);
    assert!(
        p.events
            .iter()
            .any(|e| e.event.details["TimeRole"] == "collection_filesystem")
    );
}

#[test]
fn jump_list_native_entries_and_custom_links_are_projected() {
    let v = json!({"Type":"automatic","SourceFile":"/evidence/list","Entries":[{"EntryNumber":10,"Path":"C:\\target","LastUsed":DATE,"Created":DATE,"Link":{"Header":{"TargetCreated":DATE}}}]});
    let p = project(&v, &ctx("jump-lists"), &policy());
    assert_eq!(p.events.len(), 3);
    assert!(
        p.events
            .iter()
            .any(|e| e.event.source_field == "Entries[0].Link.Header.TargetCreated")
    );
    let custom = json!({"SourceFile":"/evidence/custom","Type":"custom","Categories":[{"Links":[{"TargetPath":"C:\\target","Header":{"TargetModified":DATE}}]}]});
    assert_eq!(
        project(&custom, &ctx("jump-lists"), &policy()).events.len(),
        1
    );
}

#[test]
fn logfile_lsn_alone_has_no_wall_clock_and_redo_undo_are_preserved() {
    let v = json!({"LSN":100,"SourceFile":"/evidence/$LogFile","RedoTimes":{"Created":{"UTC":DATE}},"UndoTimes":{"Created":{"UTC":DATE}}});
    let p = project(&v, &ctx("logfile"), &policy());
    assert_eq!(p.events.len(), 2);
    assert_ne!(p.events[0].identity, p.events[1].identity);
    let row = json!({"LSN":134369688384234399u64,"SourceFile":"/evidence/$LogFile"});
    let p = project(&row, &ctx("logfile"), &policy());
    assert!(p.events.is_empty());
    assert_eq!(p.counts.untimed_records, 1);
}

#[test]
fn i30_csv_and_native_time_names_are_both_supported() {
    let csv = json!({"RecordOffset":4096,"EntryOffset":4200,"FileName":"x","CreatedUTC":DATE,"MFT-ModifiedUTC":DATE,"FromSlack":true});
    let native = json!({"RecordOffset":4096,"EntryOffset":4200,"FileName":"x","Created":DATE,"MFT-Modified":DATE,"FromSlack":true});
    assert_eq!(project(&csv, &ctx("i30"), &policy()).events.len(), 2);
    assert_eq!(project(&native, &ctx("i30"), &policy()).events.len(), 2);
}

#[test]
fn all_registry_adapter_families_accept_the_exported_key_time_names() {
    for artifact in [
        "app-history",
        "appcompat",
        "auditpol",
        "bam",
        "bcd",
        "featureusage",
        "installed-programs",
        "lsa-secrets",
        "mru",
        "muicache",
        "networks",
        "office",
        "opensavemru",
        "persistence",
        "recentdocs",
        "remote-access",
        "sam",
        "security-settings",
        "services",
        "shellbags",
        "system-info",
        "taskcache",
        "usb",
        "userassist",
    ] {
        let v = json!({"SourcePath":"/evidence/SYSTEM","KeyPath":"A\\B","KeyLastWritten":DATE});
        let p = project(&v, &ctx(artifact), &policy());
        assert_eq!(p.events.len(), 1, "{artifact}");
        assert_eq!(p.counts.invalid_timestamps, 0);
    }
}

#[test]
fn hayabusa_info_details_strings_are_valid_and_correlation_rows_stay_distinct() {
    let v = json!({"Timestamp":DATE,"Computer":"HOST","RuleID":"rule","RuleTitle":"Information","EventID":3,"Level":"info","EvtxFile":"/evidence/events.evtx","Details":"text"});
    let p = project(&v, &ctx("hayabusa"), &policy());
    assert_eq!(p.events.len(), 1);
    assert_eq!(p.events[0].event.source_file, "/evidence/events.evtx");
    assert_eq!(p.events[0].event.details["RawSourceAvailable"], true);
    let mut next = ctx("hayabusa");
    next.record = 2;
    assert_ne!(
        p.events[0].identity,
        project(&v, &next, &policy()).events[0].identity
    );
}

#[test]
fn invalid_missing_filtered_and_unknown_times_are_reported_separately() {
    let v = json!({"LastRun":"not a timestamp","PreviousRun1":null,"PreviousRun2":"0","PreviousRun3":"1970-01-01T00:00:00Z"});
    let p = project(&v, &ctx("prefetch"), &policy());
    assert_eq!(p.events.len(), 1);
    assert_eq!(p.counts.invalid_timestamps, 1);
    assert_eq!(p.counts.untimed_records, 0);
    let mut settings = policy();
    settings.start = Some(Point {
        seconds: 1,
        nanos: 0,
    });
    let p = project(&v, &ctx("prefetch"), &settings);
    assert!(p.events.is_empty());
    assert_eq!(p.counts.filtered_events, 1);
    assert_eq!(p.counts.untimed_records, 0);
    let p = project(&Value::Null, &ctx("unknown"), &policy());
    assert_eq!(p.counts.unsupported_records, 1);
}

#[test]
fn native_usb_user_time_uses_the_user_hive_source() {
    let v = json!({"SourcePath":"/evidence/SYSTEM", "Path":"USB\\disk", "Users":[{"Username":"analyst","SourcePath":"/evidence/NTUSER.DAT","MountPoints2LastWritten":DATE}]});
    let p = project(&v, &ctx("usb"), &policy());
    assert_eq!(p.events.len(), 1);
    assert_eq!(p.events[0].event.source_file, "/evidence/NTUSER.DAT");
    assert_eq!(p.events[0].event.user, "analyst");
}
#[test]
fn different_control_sets_keep_distinct_service_key_times() {
    let a = json!({"ServiceName":"Example","ControlSet":1,"KeyLastWritten":DATE});
    let mut b = a.clone();
    b["ControlSet"] = json!(2);
    assert_ne!(
        project(&a, &ctx("services"), &policy()).events[0].identity,
        project(&b, &ctx("services"), &policy()).events[0].identity
    );
}
#[test]
fn native_mft_recovered_slack_is_separate_from_live_fn() {
    let v = json!({"Header":{"RecordNumber":10,"Sequence":1,"Flags":"ALLOCATED"},"Attributes":[],"SlackFileNames":[{"Offset":512,"Name":"old.exe","Created":134054235982001668u64}]});
    let p = project(&v, &ctx("mft"), &policy());
    assert_eq!(p.events.len(), 1);
    assert_eq!(
        p.events[0].event.details["TimeRole"],
        "recovered_file_metadata"
    );
    assert_eq!(p.events[0].event.source_field, "SlackFileNames[0].Created");
    assert_eq!(p.events[0].event.details["Header"]["Flags"], "ALLOCATED");
    assert_eq!(p.events[0].event.details["FromSlack"], true);
    assert_eq!(
        p.events[0].event.details["OriginalTimestamp"],
        "134054235982001668"
    );
}

#[test]
fn abbreviated_fields_keep_current_spelling_and_accept_legacy_inputs() {
    use super::model::field;
    for (legacy, current) in [
        ("FileName", "Filename"),
        ("Arguments", "Args"),
        ("Computer", "Comp"),
        ("BytesRecvd", "BytesReceived"),
        ("ComputerNameLastWrittenUTC", "CompNameLastWritten"),
    ] {
        let mut value = json!({});
        value[legacy] = json!("old");
        assert_eq!(field(&value, legacy).unwrap(), (legacy, &json!("old")));
        value[current] = json!("new");
        let (actual, data) = field(&value, legacy).unwrap();
        // A current UTC-less time is the fallback only when the explicit UTC field is absent.
        if current == "CompNameLastWritten" {
            value.as_object_mut().unwrap().remove(legacy);
            assert_eq!(field(&value, legacy).unwrap(), (current, &json!("new")));
        } else {
            assert_eq!((actual, data), (current, &json!("new")));
        }
    }
    let old = json!({"FullPath":"Windows\\x.exe","Header":{"RecordNumber":10,"Sequence":3},"Attributes":[
        {"Header":{"TypeCode":"StandardInformation"},"Data":{"Created":DATE}}
    ]});
    let mut new = old.clone();
    crate::output_schema::rename_fields(&mut new).unwrap();
    let a = project(&old, &ctx("mft"), &policy());
    let b = project(&new, &ctx("mft"), &policy());
    assert_eq!(a.events.len(), 1);
    assert_eq!(b.events.len(), 1);
    assert_eq!(a.events[0].identity, b.events[0].identity);
    assert_eq!(a.events[0].event.source_field, "Attributes[0].Data.Created");
    assert_eq!(b.events[0].event.source_field, "Attr[0].Data.Created");
    let exported = serde_json::to_value(&b.events[0].event).unwrap();
    assert_eq!(exported["Comp"], "HOST");
    assert!(exported.get("Computer").is_none());
}

#[test]
fn ntfs_metadata_without_timestamps_is_untimed_and_supported() {
    for artifact in ["boot", "secure-sds"] {
        let result = project(
            &json!({"SourcePath":"capture/$Boot","SecurityID":256}),
            &ctx(artifact),
            &policy(),
        );
        assert!(result.events.is_empty());
        assert_eq!(result.counts.untimed_records, 1);
        assert_eq!(result.counts.unsupported_records, 0);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.reason.contains("No usable mapped event timestamp"))
        );
    }
}
