use super::{Check, decode_text};
use crate::persistence_sniper::com::{self, Identity};
use crate::persistence_sniper::context::Resolution;
use crate::persistence_sniper::{Finding, TimeEvidence};
use quick_xml::NsReader;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;

const TASK_ROOT: &str = "windows/system32/tasks/";
const CACHE: &str = "Microsoft\\Windows NT\\CurrentVersion\\Schedule\\TaskCache";

#[derive(Default, Debug)]
struct Action {
    kind: String,
    command: String,
    arguments: String,
    clsid: String,
    working_directory: String,
    context: String,
    principal: Principal,
}

#[derive(Default, Debug, Clone)]
struct Principal {
    id: String,
    user: String,
    group: String,
    run_level: String,
}

fn parse_task(bytes: &[u8]) -> Result<Vec<Action>, String> {
    let text = decode_text(bytes, false)?;
    let mut reader = NsReader::from_str(&text);
    reader.config_mut().check_end_names = true;
    let mut stack = Vec::<String>::new();
    let mut actions = Vec::new();
    let mut current: Option<Action> = None;
    let mut principals = Vec::<Principal>::new();
    let mut principal: Option<Principal> = None;
    let mut actions_context = String::new();
    let mut root_seen = false;
    loop {
        let (namespace, event) = reader.read_resolved_event().map_err(|e| e.to_string())?;
        let scheduler_namespace = match namespace {
            ResolveResult::Unbound => true,
            ResolveResult::Bound(ns) => {
                ns.as_ref() == b"http://schemas.microsoft.com/windows/2004/02/mit/task"
            }
            ResolveResult::Unknown(prefix) => {
                return Err(format!(
                    "unbound XML namespace prefix: {}",
                    String::from_utf8_lossy(&prefix)
                ));
            }
        };
        match event {
            Event::Start(e) => {
                let tag = if scheduler_namespace {
                    String::from_utf8_lossy(e.local_name().as_ref()).to_string()
                } else {
                    "__foreign_namespace".into()
                };
                if stack.is_empty() {
                    if root_seen || tag != "Task" {
                        return Err("expected a single Task XML root".into());
                    }
                    root_seen = true;
                }
                if stack.len() >= 128 {
                    return Err("task XML nesting exceeds 128 levels".into());
                }
                if tag == "Principal" && stack == ["Task", "Principals"] {
                    let id = e
                        .try_get_attribute("id")
                        .map_err(|e| e.to_string())?
                        .map(|a| {
                            a.decode_and_unescape_value(reader.decoder())
                                .map(|v| v.into_owned())
                        })
                        .transpose()
                        .map_err(|e| e.to_string())?
                        .unwrap_or_default();
                    principal = Some(Principal {
                        id,
                        ..Default::default()
                    });
                }
                if tag == "Actions" && stack.last().is_some_and(|s| s == "Task") {
                    actions_context = e
                        .try_get_attribute("Context")
                        .map_err(|e| e.to_string())?
                        .map(|a| {
                            a.decode_and_unescape_value(reader.decoder())
                                .map(|v| v.into_owned())
                        })
                        .transpose()
                        .map_err(|e| e.to_string())?
                        .unwrap_or_default();
                }
                if matches!(tag.as_str(), "Exec" | "ComHandler")
                    && stack.last().is_some_and(|s| s == "Actions")
                {
                    current = Some(Action {
                        kind: tag.clone(),
                        context: actions_context.clone(),
                        ..Default::default()
                    });
                }
                stack.push(tag);
            }
            Event::End(e) => {
                let tag = if scheduler_namespace {
                    String::from_utf8_lossy(e.local_name().as_ref()).to_string()
                } else {
                    "__foreign_namespace".into()
                };
                if matches!(tag.as_str(), "Exec" | "ComHandler")
                    && let Some(action) = current.take()
                {
                    actions.push(action);
                }
                if tag == "Principal"
                    && let Some(item) = principal.take()
                {
                    principals.push(item);
                }
                stack.pop();
            }
            Event::Empty(e) if stack.is_empty() => {
                if root_seen || !scheduler_namespace || e.local_name().as_ref() != b"Task" {
                    return Err("expected a single Task XML root".into());
                }
                root_seen = true;
            }
            Event::Text(e) => {
                let decoded = e.decode().map_err(|e| e.to_string())?;
                let decoded = quick_xml::escape::unescape(&decoded).map_err(|e| e.to_string())?;
                add_text(&stack, &mut current, &mut principal, &decoded);
            }
            Event::CData(e) => {
                let decoded = e.decode().map_err(|e| e.to_string())?;
                add_text(&stack, &mut current, &mut principal, &decoded);
            }
            Event::DocType(_) => {
                return Err("DTD declarations are not accepted in task evidence".into());
            }
            Event::GeneralRef(e) => {
                let name = e.decode().map_err(|e| e.to_string())?;
                let resolved = match name.as_ref() {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    _ => e.resolve_char_ref().map_err(|e| e.to_string())?,
                };
                let Some(character) = resolved else {
                    return Err(format!("unresolved entity reference in task XML: {name}"));
                };
                add_text(&stack, &mut current, &mut principal, &character.to_string());
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !root_seen || !stack.is_empty() {
        return Err("truncated or empty task XML".into());
    }
    for action in &mut actions {
        let eligible: Vec<_> = principals
            .iter()
            .filter(|p| {
                if action.context.is_empty() {
                    principals.len() == 1
                } else {
                    p.id == action.context
                }
            })
            .collect();
        if eligible.len() == 1 {
            action.principal = eligible[0].clone();
        }
    }
    Ok(actions)
}

fn add_text(
    stack: &[String],
    action: &mut Option<Action>,
    principal: &mut Option<Principal>,
    text: &str,
) {
    if stack.len() == 4
        && stack[0] == "Task"
        && stack[1] == "Principals"
        && stack[2] == "Principal"
        && let Some(principal) = principal
    {
        let target = match stack[3].as_str() {
            "UserId" => Some(&mut principal.user),
            "GroupId" => Some(&mut principal.group),
            "RunLevel" => Some(&mut principal.run_level),
            _ => None,
        };
        if let Some(target) = target {
            target.push_str(text);
        }
    }
    if stack.len() == 4
        && stack[0] == "Task"
        && stack[1] == "Actions"
        && let Some(action) = action
    {
        let target = match stack[3].as_str() {
            "Command" if stack[2] == "Exec" => &mut action.command,
            "Arguments" if stack[2] == "Exec" => &mut action.arguments,
            "ClassId" if stack[2] == "ComHandler" => &mut action.clsid,
            "WorkingDirectory" if stack[2] == "Exec" => &mut action.working_directory,
            _ => return,
        };
        target.push_str(text);
    }
}

fn event_times(finding: &mut Finding, task_name: &str) {
    finding.time_evidence.push(TimeEvidence::EventLog {
        channel: "Security".into(),
        event_ids: vec![4698, 4699],
        match_field: "TaskName".into(),
        match_value: task_name.into(),
    });
    finding.time_evidence.push(TimeEvidence::EventLog {
        channel: "Microsoft-Windows-TaskScheduler/Operational".into(),
        event_ids: vec![106, 140, 141],
        match_field: "TaskName".into(),
        match_value: task_name.into(),
    });
}

pub(super) fn scheduled(check: &mut Check<'_>) {
    for file in check.files(|p| p.starts_with(TASK_ROOT)) {
        check.sources += 1;
        let Some(bytes) = check.read(&file, 8 * 1024 * 1024) else {
            continue;
        };
        let actions = match parse_task(&bytes) {
            Ok(actions) => actions,
            Err(e) => {
                check.issue("malformed", format!("{}: {e}", file.physical.display()));
                continue;
            }
        };
        let task_name = format!("\\{}", file.logical[TASK_ROOT.len()..].replace('/', "\\"));
        for action in actions {
            let user = action.principal.user.trim();
            let group = action.principal.group.trim();
            let access = if matches!(
                user.to_uppercase().as_str(),
                "S-1-5-18" | "NT AUTHORITY\\SYSTEM" | "SYSTEM"
            ) {
                "SYSTEM"
            } else if user.is_empty() && group.is_empty() {
                "UNKNOWN"
            } else {
                "USER"
            };
            let principal = if !user.is_empty() {
                format!("user {user}")
            } else if !group.is_empty() {
                format!("group {group} (individual user unknown)")
            } else {
                "not recorded or ambiguous".into()
            };
            let mut finding = if action.kind == "Exec" {
                let command = action.command.trim();
                if command.is_empty() {
                    continue;
                }
                let value = format!("{command} {}", action.arguments.trim())
                    .trim()
                    .to_string();
                let mut f = check.file_finding(&file, &value, access);
                f.resolve_target.clear();
                f.description = format!(
                    "Task {task_name} Exec action; principal: {}; working directory: {}. Configuration is not proof of execution.",
                    principal,
                    action.working_directory.trim()
                );
                f
            } else {
                let clsid = action.clsid.trim();
                if clsid.is_empty() {
                    continue;
                }
                let identity = if access == "SYSTEM" {
                    Identity::Machine
                } else if !user.is_empty() {
                    Identity::Principal(user)
                } else if !group.is_empty() {
                    Identity::Group(group)
                } else {
                    Identity::Unknown
                };
                let resolution = com::resolve(check.ctx, clsid, identity);
                for issue in &resolution.issues {
                    check.issue(
                        "unresolved",
                        format!("{} ComHandler {clsid}: {issue}", file.physical.display()),
                    );
                }
                let notes = resolution.notes.join("; ");
                let description = format!(
                    "Task {task_name} ComHandler class {clsid}; principal: {principal}; run level: {}. Configuration is not proof of execution. Architecture, elevation and policy affect activation. {notes}",
                    action.principal.run_level.trim()
                );
                if !resolution.servers.is_empty() {
                    for server in resolution.servers {
                        let mut f = check.file_finding(&file, &server.command, access);
                        f.resolve_target = server.target;
                        f.launcher = server.launcher;
                        if server.association_only {
                            f.resolution_status = "unresolved_com_class".into();
                        }
                        f.description = format!("{description} Registration: {}", server.source);
                        event_times(&mut f, &task_name);
                        check.scan.findings.push(f);
                    }
                    continue;
                }
                let mut f = check.file_finding(&file, clsid, access);
                f.resolve_target.clear();
                f.resolution_status = "unresolved_com_class".into();
                f.description = format!(
                    "{description} COM server unresolved; see coverage for missing or damaged evidence."
                );
                f
            };
            event_times(&mut finding, &task_name);
            check.scan.findings.push(finding);
        }
    }
}

pub(super) fn cache(check: &mut Check<'_>) {
    let Some(hive) = check.ctx.machine_hive("SOFTWARE") else {
        return;
    };
    check.sources += 1;
    let Some((tasks, problems)) = registry::software::task_cache(&hive.opened.hive) else {
        return;
    };
    for problem in problems {
        check.issue("malformed", format!("{}: {problem}", hive.path.display()));
    }
    let tasks_present = hive.opened.hive.key(&format!("{CACHE}\\Tasks")).is_some();
    let tree_present = hive.opened.hive.key(&format!("{CACHE}\\Tree")).is_some();
    let any_xml = check
        .ctx
        .files
        .iter()
        .any(|f| f.logical.to_lowercase().starts_with(TASK_ROOT));
    if check.definition.id == "ghost_task" && !any_xml {
        check.issue("not_collected","No task XML files were collected; filesystem ghost-task comparison is unavailable, so only registry counterpart inconsistencies are examined");
    }
    for task in tasks {
        for problem in &task.problems {
            check.issue(
                "malformed",
                format!("{} {}: {problem}", hive.path.display(), task.name),
            );
        }
        let task_name = task.name.trim_start_matches('\\');
        let key = if task.in_tree {
            format!("{CACHE}\\Tree\\{task_name}")
        } else {
            format!(
                "{CACHE}\\Tasks\\{}",
                task.guid.as_deref().unwrap_or("unknown")
            )
        };
        let mut anomalies = Vec::new();
        if check.definition.id == "hidden_scheduled_task" {
            if !task.in_tree {
                continue;
            }
            let Some(rawkey) = hive.opened.hive.key(&key) else {
                continue;
            };
            match rawkey.value("SD") {
                None => anomalies.push("TaskCache Tree registration has no SD value".to_string()),
                Some(v) if v.data_type() == 3 => match v.data() {
                    Ok(data) if data.is_empty() => anomalies
                        .push("TaskCache Tree registration has an empty SD value".to_string()),
                    Err(e) => {
                        check.issue("malformed", format!("{key} SD could not be decoded: {e}"))
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            if task.in_tree && !task.in_tasks && tasks_present {
                anomalies.push("no matching TaskCache Tasks registration".to_string());
            }
            if !task.in_tree && task.in_tasks && tree_present {
                anomalies.push("no matching TaskCache Tree registration".to_string());
            }
            if any_xml {
                let path = task
                    .path
                    .as_deref()
                    .unwrap_or(&task.name)
                    .trim_start_matches('\\');
                match check.ctx.resolve(&format!("Windows/System32/Tasks/{path}")) {
                    Resolution::NotCollected=>anomalies.push("no matching XML in collected task files (may be an incomplete collection or stale built-in registration)".to_string()),
                    Resolution::Invalid(e)=>check.issue("unresolved",format!("{key}: unsafe/unreadable task path: {e}")),
                    Resolution::Ambiguous(_)=>check.issue("unresolved",format!("{key}: multiple collected XML paths match")),
                    _=>{},
                }
            }
        }
        if anomalies.is_empty() {
            continue;
        }
        let mut finding = check.finding(
            &format!("HKLM\\SOFTWARE\\{key}"),
            task.guid.as_deref().unwrap_or(&task.name),
            "SYSTEM",
        );
        finding.source_file = hive.path.display().to_string();
        finding.record_status = hive.opened.log_status(task.replayed).into();
        finding.description = format!(
            "Scheduled task \\{task_name}: {}. Configuration discrepancy does not prove maliciousness or execution.",
            anomalies.join("; ")
        );
        if let Some(time) = task.tree_last_written.or(task.task_last_written) {
            finding.key_last_write = crate::registry::filetime(time).unwrap_or_default();
        }
        event_times(&mut finding, &format!("\\{task_name}"));
        check.scan.findings.push(finding);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utf16_and_multiple_action_types_are_preserved() {
        let xml = r#"<?xml version="1.0" encoding="UTF-16"?><Task xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task"><Principals><Principal><UserId>S-1-5-18</UserId></Principal></Principals><Actions><Exec><Command>C:\A B\tool.exe</Command><Arguments>-x &amp; y</Arguments></Exec><ComHandler><ClassId>{ABC}</ClassId></ComHandler></Actions></Task>"#;
        let mut bytes = vec![0xff, 0xfe];
        bytes.extend(xml.encode_utf16().flat_map(u16::to_le_bytes));
        let actions = parse_task(&bytes).unwrap();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].principal.user, "S-1-5-18");
        assert_eq!(actions[0].command, r"C:\A B\tool.exe");
        assert_eq!(actions[0].arguments, "-x & y");
        assert_eq!(actions[1].clsid, "{ABC}");
    }
    #[test]
    fn malformed_and_external_entity_tasks_do_not_silently_pass() {
        assert!(parse_task(b"<Task><Actions><Exec>").is_err());
        assert!(
            parse_task(
                b"<!DOCTYPE Task [<!ENTITY x SYSTEM 'file:///etc/passwd'>]><Task>&x;</Task>"
            )
            .is_err()
        );
        assert!(parse_task(b"<Other/>").is_err());
    }
    #[test]
    fn foreign_namespace_actions_are_not_misreported_as_executable_tasks() {
        let xml=br#"<Task xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task" xmlns:other="https://example.invalid"><Actions><other:Exec><other:Command>fake.exe</other:Command></other:Exec><Exec><Command>real.exe</Command></Exec></Actions></Task>"#;
        let actions = parse_task(xml).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].command, "real.exe");
    }
    #[test]
    fn task_actions_select_their_principal_by_context_without_concatenating_accounts() {
        let xml = br#"<Task><Principals><Principal id="System"><UserId>S-1-5-18</UserId></Principal><Principal id="User"><UserId>LAB\alice</UserId></Principal></Principals><Actions Context="User"><ComHandler><ClassId>{ABC}</ClassId></ComHandler></Actions></Task>"#;
        let actions = parse_task(xml).unwrap();
        assert_eq!(actions[0].principal.user, r"LAB\alice");
        let xml = String::from_utf8(xml.to_vec())
            .unwrap()
            .replace("Context=\"User\"", "Context=\"Missing\"");
        assert!(
            parse_task(xml.as_bytes()).unwrap()[0]
                .principal
                .user
                .is_empty()
        );
    }
    #[test]
    fn group_principals_are_kept_separate_from_individual_users() {
        let xml = br#"<Task><Principals><Principal><GroupId>S-1-5-32-544</GroupId><RunLevel>HighestAvailable</RunLevel></Principal></Principals><Actions><ComHandler><ClassId>{ABC}</ClassId></ComHandler></Actions></Task>"#;
        let actions = parse_task(xml).unwrap();
        assert!(actions[0].principal.user.is_empty());
        assert_eq!(actions[0].principal.group, "S-1-5-32-544");
        assert_eq!(actions[0].principal.run_level, "HighestAvailable");
    }
}
