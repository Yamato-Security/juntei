//! COM activation triggers placed in a user's Start Menu. Inspired by the
//! mechanism described in PersistenceSniper #32; independently implemented.
use super::{Check, decode_text};
use crate::persistence_sniper::com::{self, Identity};
use quick_xml::{NsReader, events::Event, name::ResolveResult};
use std::collections::BTreeSet;

fn folder_clsid(path: &str) -> Option<String> {
    let name = path.rsplit('/').next()?;
    let at = name.rfind(".{")?;
    com::guid(&name[at + 1..])
}
fn url_clsid(url: &str) -> Option<String> {
    let url = url.trim();
    if let Some(at) = url.rfind('{') {
        let prefix = url[..at].to_ascii_lowercase();
        if prefix.ends_with("shell:::")
            || prefix.ends_with("knownfolder:::")
            || prefix.ends_with('.')
        {
            return com::guid(&url[at..]);
        }
    }
    None
}
fn library_classes(bytes: &[u8]) -> Result<BTreeSet<String>, String> {
    let text = decode_text(bytes, false)?;
    let mut reader = NsReader::from_str(&text);
    let mut stack = Vec::<String>::new();
    let mut url = String::new();
    let mut classes = BTreeSet::new();
    let mut root_seen = false;
    loop {
        let (namespace, event) = reader.read_resolved_event().map_err(|e| e.to_string())?;
        let own = match namespace {
            ResolveResult::Unbound => true,
            ResolveResult::Bound(ns) => {
                ns.as_ref() == b"http://schemas.microsoft.com/windows/2009/library"
            }
            ResolveResult::Unknown(_) => return Err("unbound library XML namespace".into()),
        };
        match event {
            Event::DocType(_) => {
                return Err("DTD declarations are not accepted in library evidence".into());
            }
            Event::Start(e) => {
                let tag = if own {
                    String::from_utf8_lossy(e.local_name().as_ref()).into_owned()
                } else {
                    "__foreign".into()
                };
                if stack.is_empty() {
                    if root_seen || tag != "libraryDescription" {
                        return Err("expected one libraryDescription root".into());
                    }
                    root_seen = true;
                }
                if stack.len() >= 128 {
                    return Err("library XML exceeds 128 nesting levels".into());
                }
                if tag == "url" {
                    url.clear();
                }
                stack.push(tag);
            }
            Event::Empty(e) if stack.is_empty() => {
                if root_seen || !own || e.local_name().as_ref() != b"libraryDescription" {
                    return Err("expected one libraryDescription root".into());
                }
                root_seen = true;
            }
            Event::Text(e) if stack.last().is_some_and(|s| s == "url") => {
                let text = e.decode().map_err(|e| e.to_string())?;
                url.push_str(&quick_xml::escape::unescape(&text).map_err(|e| e.to_string())?);
            }
            Event::CData(e) if stack.last().is_some_and(|s| s == "url") => {
                url.push_str(&e.decode().map_err(|e| e.to_string())?)
            }
            Event::GeneralRef(e) if stack.last().is_some_and(|s| s == "url") => {
                let text = format!("&{};", e.decode().map_err(|e| e.to_string())?);
                url.push_str(&quick_xml::escape::unescape(&text).map_err(|e| e.to_string())?);
            }
            Event::End(_) => {
                if stack.last().is_some_and(|s| s == "url")
                    && stack.len() >= 3
                    && stack[stack.len() - 2] == "simpleLocation"
                    && stack.iter().any(|s| s == "searchConnectorDescription")
                    && let Some(clsid) = url_clsid(&url)
                {
                    classes.insert(clsid);
                }
                stack.pop();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !root_seen || !stack.is_empty() {
        return Err("empty or truncated library XML".into());
    }
    Ok(classes)
}

fn start_menu_user(path: &str) -> Option<&str> {
    let mut parts = path.split('/');
    if !parts.next()?.eq_ignore_ascii_case("Users") {
        return None;
    }
    let user = parts.next()?;
    for wanted in ["AppData", "Roaming", "Microsoft", "Windows", "Start Menu"] {
        if !parts.next()?.eq_ignore_ascii_case(wanted) {
            return None;
        }
    }
    // The Start Menu directory itself is not a candidate.
    parts.next()?;
    Some(user)
}

pub(super) fn run(check: &mut Check<'_>) {
    let files = check.files(|path| start_menu_user(path).is_some());
    let directories: Vec<_> = check
        .ctx
        .directories
        .iter()
        .filter(|d| start_menu_user(&d.logical).is_some())
        .cloned()
        .collect();
    for (entry, directory) in files
        .into_iter()
        .map(|f| (f, false))
        .chain(directories.into_iter().map(|d| (d, true)))
    {
        check.sources += 1;
        let classes = if directory {
            folder_clsid(&entry.logical).into_iter().collect()
        } else if entry.logical.to_ascii_lowercase().ends_with(".library-ms") {
            let Some(bytes) = check.read(&entry, 4 * 1024 * 1024) else {
                continue;
            };
            match library_classes(&bytes) {
                Ok(classes) => classes,
                Err(e) => {
                    check.issue("malformed", format!("{}: {e}", entry.physical.display()));
                    continue;
                }
            }
        } else {
            continue;
        };
        for clsid in classes {
            let user = start_menu_user(&entry.logical).unwrap_or_default();
            let resolution = com::resolve(check.ctx, &clsid, Identity::User(user));
            for issue in resolution.issues {
                check.issue(
                    "unresolved",
                    format!("{} {clsid}: {issue}", entry.physical.display()),
                );
            }
            let trigger = if directory {
                "CLSID-named Start Menu folder"
            } else {
                "Start Menu library URL"
            };
            let description = format!(
                "{trigger} references {clsid} for user {user}. This is an activation-trigger candidate, not proof of execution or maliciousness. {}",
                resolution.notes.join("; ")
            );
            if resolution.servers.is_empty() {
                let mut finding = check.file_finding(&entry, &clsid, "USER");
                finding.resolve_target.clear();
                finding.resolution_status = "unresolved_com_class".into();
                finding.description = description;
                check.scan.findings.push(finding);
            } else {
                for server in resolution.servers {
                    let mut finding = check.file_finding(&entry, &server.command, "USER");
                    finding.resolve_target = server.target;
                    finding.launcher = server.launcher;
                    if server.association_only {
                        finding.resolution_status = "unresolved_com_class".into();
                    }
                    finding.description = format!("{description} Registration: {}", server.source);
                    check.scan.findings.push(finding);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ID: &str = "{00112233-4455-6677-8899-AABBCCDDEEFF}";
    #[test]
    fn urls_and_folders_require_real_guids() {
        for prefix in ["shell:::", "KNOWNFOLDER:::", "C:/Start/Name."] {
            assert_eq!(url_clsid(&format!("{prefix}{ID}")), Some(ID.into()));
        }
        assert!(url_clsid(&format!("https://example.test/{ID}")).is_none());
        assert!(folder_clsid(&format!("Folder.{ID}.bak")).is_none());
        assert!(com::guid("{not-a-clsid}").is_none());
        assert!(com::guid("{{00112233-4455-6677-8899-AABBCCDDEEFF}").is_none());
    }
    #[test]
    fn library_xml_is_parsed_without_comment_or_foreign_namespace_false_positives() {
        let xml = format!(
            r#"<libraryDescription xmlns="http://schemas.microsoft.com/windows/2009/library"><!-- <url>shell:::{ID}</url> --><searchConnectorDescription><simpleLocation><url>shell:::{ID}</url></simpleLocation></searchConnectorDescription></libraryDescription>"#
        );
        assert_eq!(
            library_classes(xml.as_bytes()).unwrap(),
            BTreeSet::from([ID.into()])
        );
        let foreign = xml.replace("<url>", "<url xmlns=\"https://invalid.test\">");
        assert!(library_classes(foreign.as_bytes()).unwrap().is_empty());
        let mut utf16 = vec![0xff, 0xfe];
        utf16.extend(xml.encode_utf16().flat_map(u16::to_le_bytes));
        assert_eq!(
            library_classes(&utf16).unwrap(),
            BTreeSet::from([ID.into()])
        );
        assert!(library_classes(b"<!DOCTYPE libraryDescription><libraryDescription/>").is_err());
        assert!(library_classes(b"<libraryDescription><url>").is_err());
    }
}
