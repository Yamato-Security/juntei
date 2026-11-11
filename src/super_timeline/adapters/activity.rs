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

//! Explicit timestamp maps for browser, WebCache, Outlook and UAL exports.
//! Scheduled, expiry, inventory and housekeeping clocks never imply an observed action.
use super::*;

/// JSON retains Table; flat CSV identifies it through its parser's columns, regardless of the
/// analyst's filename or SourceN suffix. All signatures name columns, not date-looking values.
fn table(b: &Builder<'_, '_>, signatures: &[(&str, &[&str])]) -> String {
    let explicit = text(b.value, &["Table"]);
    if !explicit.is_empty() {
        return explicit;
    }
    signatures
        .iter()
        .find(|(_, fields)| fields.iter().all(|key| field(b.value, key).is_some()))
        .map_or_else(String::new, |(name, _)| (*name).into())
}

fn structured(value: &Value) -> Value {
    if let Value::String(text) = value
        && (text.trim_start().starts_with('[') || text.trim_start().starts_with('{'))
        && let Ok(decoded) = serde_json::from_str(text)
    {
        return decoded;
    }
    value.clone()
}

fn context(b: &mut Builder<'_, '_>, table: &str, keys: &[&str]) {
    b.extra.insert("Table".into(), json!(table));
    for key in keys {
        if let Some((actual, value)) = field(b.value, key).filter(|(_, value)| !value.is_null()) {
            let value = if matches!(*key, "DailyAccesses" | "GoogleAnalytics" | "RedirectChain") {
                structured(value)
            } else {
                value.clone()
            };
            b.extra.insert(actual.into(), compact(&value, 0));
        }
    }
}

fn identity(b: &mut Builder<'_, '_>, kind: &str, keys: &[&str]) {
    let parts = keys
        .iter()
        .filter_map(|key| {
            let v = text(b.value, &[*key]);
            (!v.is_empty()).then(|| format!("{key}={v}"))
        })
        .collect::<Vec<_>>();
    b.id = format!(
        "{}:{kind}:{}",
        b.artifact,
        if parts.is_empty() {
            b.id.clone()
        } else {
            fingerprint(&parts.iter().map(String::as_str).collect::<Vec<_>>())
        }
    );
}

fn times(b: &mut Builder<'_, '_>, fields: &[(&str, &str, &str)]) {
    for (field, label, role) in fields {
        b.named(&[*field], label, role, true);
    }
}

pub(super) fn browser(b: &mut Builder<'_, '_>) {
    let table = table(
        b,
        &[
            ("History", &["VisitTime", "LastVisitTime"]),
            ("Downloads", &["DownloadID", "StartTime"]),
            ("Cookies", &["Host", "Created", "LastAccessed"]),
            ("FormHistory", &["FirstUsed", "RemovedTime"]),
        ],
    );
    context(
        b,
        &table,
        &[
            "Browser",
            "Profile",
            "URL",
            "Title",
            "RecordType",
            "Transition",
            "VisitCount",
            "TypedCount",
            "VisitDuration",
            "PageSource",
            "TargetPath",
            "CurrentPath",
            "RedirectChain",
            "Referrer",
            "MimeType",
            "ReceivedBytes",
            "TotalBytes",
            "State",
            "DangerType",
            "InterruptReason",
            "Opened",
            "FileDeletedByUser",
            "Host",
            "Name",
            "Secure",
            "HttpOnly",
            "Persistent",
            "GoogleAnalytics",
            "FieldName",
            "Count",
            "GUID",
        ],
    );
    match table.as_str() {
        "History" => {
            b.path = text(b.value, &["URL"]);
            identity(
                b,
                &format!("History:{}", text(b.value, &["RecordType"])),
                &["VisitID", "URL"],
            );
            times(
                b,
                &[
                    ("VisitTime", "Browser recorded visit", "browser_visit"),
                    (
                        "LastVisitTime",
                        "Browser page last-visit metadata",
                        "browser_page_metadata",
                    ),
                ],
            );
        }
        "Downloads" => {
            b.path = text(b.value, &["TargetPath", "CurrentPath", "URL"]);
            identity(b, "Downloads", &["DownloadID", "TargetPath", "URL"]);
            times(
                b,
                &[
                    ("StartTime", "Browser download start", "download_activity"),
                    (
                        "EndTime",
                        "Browser download end metadata",
                        "download_activity",
                    ),
                ],
            );
        }
        "Cookies" => {
            b.path = format!(
                "{}{}; {}",
                text(b.value, &["Host"]),
                text(b.value, &["Path"]),
                text(b.value, &["Name"])
            );
            identity(b, "Cookies", &["RowID", "Host", "Name", "Path"]);
            times(
                b,
                &[
                    ("Created", "Browser cookie creation", "cookie_metadata"),
                    (
                        "LastAccessed",
                        "Browser cookie last-access metadata",
                        "cookie_metadata",
                    ),
                    (
                        "Expires",
                        "Browser cookie configured expiry",
                        "configuration_metadata",
                    ),
                ],
            );
        }
        "FormHistory" => {
            b.path = text(b.value, &["FieldName", "GUID"]);
            context(b, &table, &["Value"]);
            identity(b, "FormHistory", &["RowID", "GUID", "FieldName", "Value"]);
            times(
                b,
                &[
                    (
                        "FirstUsed",
                        "Browser form recorded first use",
                        "form_activity",
                    ),
                    (
                        "LastUsed",
                        "Browser form recorded last use",
                        "form_activity",
                    ),
                    (
                        "RemovedTime",
                        "Browser form deletion tombstone",
                        "deletion_metadata",
                    ),
                ],
            );
        }
        _ => {}
    }
}

pub(super) fn webcache(b: &mut Builder<'_, '_>) {
    let table = table(
        b,
        &[
            ("Downloads", &["DownloadID"]),
            ("Cookies", &["CookieTable"]),
            ("Containers", &["LastScavengeTime"]),
            ("Entries", &["ResponseHeaders"]),
            ("History", &["AccessedTime", "Title"]),
        ],
    );
    context(
        b,
        &table,
        &[
            "Package",
            "Container",
            "ContainerID",
            "URL",
            "Title",
            "Account",
            "Filename",
            "FilePath",
            "TargetPath",
            "CurrentPath",
            "Referrer",
            "Publisher",
            "AccessCount",
            "SyncCount",
            "EntryType",
            "Flags",
            "RedirectURL",
            "Host",
            "Name",
            "Directory",
            "CookieTable",
            "TotalBytes",
        ],
    );
    b.path = text(
        b.value,
        &["TargetPath", "FilePath", "URL", "Host", "Directory", "Name"],
    );
    identity(
        b,
        &table,
        &[
            "EntryID",
            "DownloadID",
            "ContainerID",
            "URL",
            "Host",
            "Name",
            "Path",
        ],
    );
    match table.as_str() {
        "History" | "Entries" => times(
            b,
            &[
                (
                    "AccessedTime",
                    "WebCache stored access time",
                    "webcache_metadata",
                ),
                (
                    "ModifiedTime",
                    "WebCache stored modification time",
                    "webcache_metadata",
                ),
                (
                    "CreationTime",
                    "WebCache entry creation",
                    "webcache_metadata",
                ),
                (
                    "ExpiryTime",
                    "WebCache configured expiry",
                    "configuration_metadata",
                ),
                (
                    "SyncTime",
                    "WebCache stored sync time",
                    "cache_housekeeping",
                ),
            ],
        ),
        "Downloads" => times(
            b,
            &[
                ("StartTime", "WebCache download start", "download_activity"),
                (
                    "AccessedTime",
                    "WebCache download entry access metadata",
                    "webcache_metadata",
                ),
            ],
        ),
        "Cookies" => times(
            b,
            &[
                (
                    "LastModified",
                    "WebCache cookie modification",
                    "cookie_metadata",
                ),
                (
                    "Expires",
                    "WebCache cookie configured expiry",
                    "configuration_metadata",
                ),
            ],
        ),
        "Containers" => times(
            b,
            &[
                (
                    "LastAccessTime",
                    "WebCache container last access",
                    "cache_housekeeping",
                ),
                (
                    "LastScavengeTime",
                    "WebCache container scavenging",
                    "cache_housekeeping",
                ),
            ],
        ),
        _ => {}
    }
}

pub(super) fn outlook(b: &mut Builder<'_, '_>) {
    let table = table(
        b,
        &[
            ("Properties", &["PropertyTag", "Type", "Value"]),
            ("Folders", &["FolderID"]),
            ("Attachments", &["Method", "Index", "DataSize"]),
            ("Recipients", &["RecipientType"]),
            ("Contacts", &["GivenName"]),
            ("Calendar", &["GlobalObjectID"]),
            ("Tasks", &["PercentComplete"]),
            ("Items", &["MessageClass"]),
            ("Files", &["FileType"]),
        ],
    );
    context(
        b,
        &table,
        &[
            "FolderPath",
            "ItemID",
            "FolderID",
            "ParentID",
            "Index",
            "Scope",
            "Hidden",
            "EmbeddedIn",
            "ItemType",
            "MessageClass",
            "Subject",
            "SenderName",
            "SenderAddress",
            "SenderSMTP-Address",
            "To",
            "Cc",
            "Bcc",
            "InternetMessageID",
            "InReplyTo",
            "Read",
            "Flags",
            "Filename",
            "Pathname",
            "SHA256",
            "MimeType",
            "EmbeddedSubject",
            "Method",
            "PropertyTag",
            "PropertyName",
            "Type",
            "DisplayName",
            "Organizer",
            "Recurring",
            "AllDay",
            "BusyStatus",
            "Complete",
            "Owner",
            "Status",
        ],
    );
    b.path = text(
        b.value,
        &[
            "Pathname",
            "Filename",
            "Subject",
            "DisplayName",
            "FolderPath",
            "Name",
        ],
    );
    let item = matches!(table.as_str(), "Items" | "Contacts" | "Calendar" | "Tasks");
    identity(
        b,
        if item { "Item" } else { &table },
        &[
            "ItemID",
            "FolderID",
            "Index",
            "Scope",
            "PropertyTag",
            "EmbeddedIn",
        ],
    );
    if matches!(table.as_str(), "Folders" | "Attachments") || item {
        let object = match table.as_str() {
            "Folders" => "folder",
            "Attachments" => "attachment",
            _ => "item",
        };
        b.named(
            &["CreationTime"],
            &format!("Outlook {object} creation"),
            "outlook_object_metadata",
            true,
        );
        b.named(
            &["LastModifiedTime"],
            &format!("Outlook {object} last modification"),
            "outlook_object_metadata",
            true,
        );
    }
    match table.as_str() {
        "Items" => times(
            b,
            &[
                (
                    "SubmitTime",
                    "Outlook message submission",
                    "message_submission",
                ),
                (
                    "DeliveryTime",
                    "Outlook message delivery",
                    "message_delivery",
                ),
            ],
        ),
        "Calendar" => times(
            b,
            &[
                (
                    "StartTime",
                    "Outlook calendar scheduled start",
                    "scheduled_time",
                ),
                (
                    "EndTime",
                    "Outlook calendar scheduled end",
                    "scheduled_time",
                ),
                (
                    "ReminderTime",
                    "Outlook calendar scheduled reminder",
                    "scheduled_time",
                ),
            ],
        ),
        "Tasks" => times(
            b,
            &[
                (
                    "StartTime",
                    "Outlook task configured start",
                    "scheduled_time",
                ),
                ("DueTime", "Outlook task due date", "scheduled_time"),
                (
                    "CompletedTime",
                    "Outlook task recorded completion",
                    "task_metadata",
                ),
                (
                    "ReminderTime",
                    "Outlook task scheduled reminder",
                    "scheduled_time",
                ),
            ],
        ),
        "Properties" => {
            let ty = text(b.value, &["Type"]);
            if matches!(ty.as_str(), "PtypTime" | "PtypFloatingTime") {
                b.named(
                    &["Value"],
                    "Outlook typed time property",
                    "outlook_property_metadata",
                    true,
                );
            } else if matches!(ty.as_str(), "PtypMultipleTime" | "PtypMultipleFloatingTime") {
                let raw = get(b.value, &["Value"])
                    .map(structured)
                    .unwrap_or(Value::Null);
                if let Some(values) = raw.as_array() {
                    for (index, value) in values.iter().enumerate() {
                        b.emit(
                            value,
                            &format!("Value[{index}]"),
                            "Outlook typed time property",
                            "outlook_property_metadata",
                            &b.path.clone(),
                            &format!("{}:{index}", b.id),
                            true,
                        );
                    }
                }
            }
        }
        _ => {}
    }
}

pub(super) fn ual(b: &mut Builder<'_, '_>) {
    let mut table = text(b.value, &["Table"]);
    if table.is_empty() {
        let stem = b
            .ctx
            .parsed_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if let Some(name) = stem.strip_prefix("UAL-") {
            table = name
                .rsplit_once("-Source")
                .filter(|(_, suffix)| {
                    !suffix.is_empty() && suffix.bytes().all(|c| c.is_ascii_digit())
                })
                .map_or(name, |(name, _)| name)
                .into();
        }
    }
    table = match table.as_str() {
        "Clients" => "CLIENTS",
        "RoleAccess" => "ROLE_ACCESS",
        "DNS" => "DNS",
        "VirtualMachines" => "VIRTUALMACHINES",
        "SystemIdentity" => "SYSTEM_IDENTITY",
        "ChainedDatabases" => "CHAINED_DATABASES",
        "Roles" => "ROLE_IDS",
        other => other,
    }
    .into();
    context(
        b,
        &table,
        &[
            "RoleGuid",
            "RoleName",
            "TenantId",
            "Address",
            "ClientName",
            "AuthenticatedUserName",
            "TotalAccesses",
            "DailyAccesses",
            "HostName",
            "VMName",
            "VMID",
            "VmGuid",
            "BiosGuid",
            "OSVersion",
            "OSProductType",
        ],
    );
    b.user = text(b.value, &["AuthenticatedUserName", "User", "UserName"]);
    b.path = text(b.value, &["Address", "ClientName", "HostName", "VMName"]);
    identity(
        b,
        &table,
        &[
            "RoleGuid",
            "TenantId",
            "Address",
            "AuthenticatedUserName",
            "ClientName",
            "VMID",
            "VmGuid",
            "BiosGuid",
        ],
    );
    times(
        b,
        &[
            (
                "LastSeenActive",
                "UAL virtual machine last-seen metadata",
                "virtual_machine_metadata",
            ),
            (
                "FirstAccess",
                "UAL recorded first access",
                "ual_access_metadata",
            ),
            (
                "LastAccess",
                "UAL recorded last access",
                "ual_access_metadata",
            ),
            (
                "InsertDate",
                "UAL database record insertion",
                "database_metadata",
            ),
            (
                "CreationTime",
                "UAL record creation metadata",
                "database_metadata",
            ),
            (
                "FirstSeen",
                "UAL first-seen metadata",
                "ual_access_metadata",
            ),
            ("LastSeen", "UAL last-seen metadata", "ual_access_metadata"),
            (
                "OSLastBootUpTime",
                "UAL stored operating-system boot time",
                "system_metadata",
            ),
        ],
    );
    // DailyAccesses contains dates and counts, not precise access instants. Preserve it in
    // Details instead of inventing midnight logons or treating access counts as FILETIMEs.
}

/// Recycle Bin: when an item was moved to the bin, from its `$I` file or INFO2 record. The
/// bin's user (or SID) is the account it was deleted from; the recycled content's presence,
/// the size and INFO2's record state remain context.
pub(super) fn recycle_bin(b: &mut Builder<'_, '_>) {
    for key in [
        "Format",
        "FormatVersion",
        "FileSize",
        "SID",
        "RecycledFile",
        "RecycledFileFound",
        "RecordIndex",
        "Drive",
        "RemovedFromBin",
        "ShortPath",
        "Problems",
    ] {
        if let Some((actual, value)) = field(b.value, key).filter(|(_, value)| {
            !value.is_null()
                && value.as_str() != Some("")
                && value.as_array().is_none_or(|items| !items.is_empty())
        }) {
            b.extra.insert(actual.into(), compact(value, 0));
        }
    }
    b.path = text(b.value, &["OriginalPath"]);
    let offset = text(b.value, &["RecordOffset"]);
    b.id = if offset.is_empty() {
        "IndexFile=$I".into()
    } else {
        format!(
            "RecordIndex={};RecordOffset={offset}",
            text(b.value, &["RecordIndex"])
        )
    };
    b.named(
        &["DeletedTime"],
        "Recycle Bin deletion (moved to the bin)",
        "deletion",
        true,
    );
}
