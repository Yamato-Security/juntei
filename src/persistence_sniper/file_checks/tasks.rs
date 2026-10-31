use super::{Check, decode_text, registry_text};
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
}

fn parse_task(bytes: &[u8]) -> Result<(Vec<Action>, String), String> {
    let text = decode_text(bytes, false)?;
    let mut reader = NsReader::from_str(&text);
    reader.config_mut().check_end_names = true;
    let mut stack = Vec::<String>::new();
    let mut actions = Vec::new();
    let mut current: Option<Action> = None;
    let mut user = String::new();
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
                if matches!(tag.as_str(), "Exec" | "ComHandler")
                    && stack.last().is_some_and(|s| s == "Actions")
                {
                    current = Some(Action {
                        kind: tag.clone(),
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
                add_text(&stack, &mut current, &mut user, &decoded);
            }
            Event::CData(e) => {
                let decoded = e.decode().map_err(|e| e.to_string())?;
                add_text(&stack, &mut current, &mut user, &decoded);
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
                add_text(&stack, &mut current, &mut user, &character.to_string());
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !root_seen || !stack.is_empty() {
        return Err("truncated or empty task XML".into());
    }
    Ok((actions, user.trim().to_string()))
}

fn add_text(stack: &[String], action: &mut Option<Action>, user: &mut String, text: &str) {
    if stack.last().is_some_and(|s| s == "UserId") && stack.iter().any(|s| s == "Principals") {
        user.push_str(text);
    }
    if let Some(action) = action {
        let target = match stack.last().map(String::as_str) {
            Some("Command") => &mut action.command,
            Some("Arguments") => &mut action.arguments,
            Some("ClassId") => &mut action.clsid,
            Some("WorkingDirectory") => &mut action.working_directory,
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
        let (actions, user) = match parse_task(&bytes) {
            Ok(actions) => actions,
            Err(e) => {
                check.issue("malformed", format!("{}: {e}", file.physical.display()));
                continue;
            }
        };
        let task_name = format!("\\{}", file.logical[TASK_ROOT.len()..].replace('/', "\\"));
        let access = if matches!(
            user.to_uppercase().as_str(),
            "S-1-5-18" | "NT AUTHORITY\\SYSTEM" | "SYSTEM"
        ) {
            "SYSTEM"
        } else if user.is_empty() {
            "UNKNOWN"
        } else {
            "USER"
        };
        for action in actions {
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
                    if user.is_empty() {
                        "not recorded"
                    } else {
                        &user
                    },
                    action.working_directory.trim()
                );
                f
            } else {
                let clsid = action.clsid.trim();
                if clsid.is_empty() {
                    continue;
                }
                let image = check.ctx.machine_hive("SOFTWARE").and_then(|h| {
                    registry_text(h, &format!("Classes\\CLSID\\{clsid}\\InprocServer32"), "")
                        .or_else(|| {
                            registry_text(h, &format!("Classes\\CLSID\\{clsid}\\LocalServer32"), "")
                        })
                });
                let mut f = check.file_finding(&file, image.as_deref().unwrap_or(clsid), access);
                if let Some(image) = image.as_deref() {
                    let (payload, launcher) =
                        crate::persistence_sniper::resolver::command_target(image);
                    f.resolve_target = payload;
                    f.launcher = launcher;
                } else {
                    f.resolve_target.clear();
                }
                f.description = format!(
                    "Task {task_name} ComHandler class {clsid}; principal: {}. {}",
                    if user.is_empty() {
                        "not recorded"
                    } else {
                        &user
                    },
                    if image.is_none() {
                        "COM server was not resolved from collected SOFTWARE; target/signature coverage is incomplete."
                    } else {
                        "Configuration is not proof of execution."
                    }
                );
                if image.is_none() {
                    f.resolution_status = "unresolved_com_class".into();
                    check.issue(
                        "unresolved",
                        format!(
                            "{} ComHandler {clsid} has no collected COM server registration",
                            file.physical.display()
                        ),
                    );
                }
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
        let (actions, user) = parse_task(&bytes).unwrap();
        assert_eq!(actions.len(), 2);
        assert_eq!(user, "S-1-5-18");
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
        let (actions, _) = parse_task(xml).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].command, "real.exe");
    }
}
