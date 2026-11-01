use super::{Check, decode_text};
use serde_json::Value;

fn script_config(path: &str) -> bool {
    let parts: Vec<_> = path.split('/').collect();
    let name = parts.last().copied().unwrap_or("");
    if !matches!(name, "scripts.ini" | "psscripts.ini") {
        return false;
    }
    if parts.len() == 6 && parts[..3] == ["windows", "system32", "grouppolicy"] {
        return matches!(parts[3], "machine" | "user") && parts[4] == "scripts";
    }
    parts.len() == 7
        && parts[..3] == ["windows", "system32", "grouppolicyusers"]
        && parts[4] == "user"
        && parts[5] == "scripts"
}

pub(super) fn gp_scripts(check: &mut Check<'_>) {
    for file in check.files(script_config) {
        check.sources += 1;
        let Some(bytes) = check.read(&file, 4 * 1024 * 1024) else {
            continue;
        };
        let text = match decode_text(&bytes, true) {
            Ok(s) => s,
            Err(e) => {
                check.issue("malformed", format!("{}: {e}", file.physical.display()));
                continue;
            }
        };
        let (commands, damaged) = parse_ini(&text);
        if !damaged.is_empty() {
            check.issue(
                "malformed",
                format!(
                    "{}: invalid INI line(s) {} (valid siblings retained)",
                    file.physical.display(),
                    damaged
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
        }
        let access = if file.logical.to_lowercase().contains("/machine/") {
            "SYSTEM"
        } else {
            "USER"
        };
        for (section, command, parameters) in commands {
            let value = format!("{command} {parameters}").trim().to_string();
            let mut f = check.file_finding(&file, &value, access);
            f.resolve_target.clear();
            f.description = format!(
                "Group Policy {section} script; command and parameters recovered from configuration. Configuration does not prove execution."
            );
            check.scan.findings.push(f);
        }
    }
}

/// Preserve duplicate sections/keys instead of dropping every valid sibling.
fn parse_ini(text: &str) -> (Vec<(String, String, String)>, Vec<usize>) {
    let mut section = String::new();
    let mut entries = Vec::<(String, String, String)>::new();
    let mut damaged = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with([';', '#']) {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].to_lowercase();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            damaged.push(line_no + 1);
            continue;
        };
        if key.trim().is_empty() {
            damaged.push(line_no + 1);
            continue;
        }
        entries.push((
            section.clone(),
            key.trim().to_lowercase(),
            value.trim().to_string(),
        ));
    }
    let mut commands = Vec::new();
    for (index, (section, key, value)) in entries.iter().enumerate() {
        if !matches!(
            section.as_str(),
            "startup" | "shutdown" | "logon" | "logoff"
        ) || value.is_empty()
        {
            continue;
        }
        let Some(prefix) = key.strip_suffix("cmdline") else {
            continue;
        };
        if prefix.is_empty() || !prefix.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let parameter_key = format!("{prefix}parameters");
        let next_command = entries
            .iter()
            .enumerate()
            .skip(index + 1)
            .find(|(_, e)| e.0 == *section && e.1 == *key)
            .map(|(n, _)| n)
            .unwrap_or(entries.len());
        let parameters = entries[index + 1..next_command]
            .iter()
            .find(|e| e.0 == *section && e.1 == parameter_key)
            .or_else(|| {
                entries[..index]
                    .iter()
                    .rev()
                    .find(|e| e.0 == *section && e.1 == parameter_key)
            })
            .map(|e| e.2.clone())
            .unwrap_or_default();
        commands.push((section.clone(), value.clone(), parameters));
    }
    (commands, damaged)
}

fn terminal_settings(path: &str) -> bool {
    path.starts_with("users/") && (
        path.ends_with("/appdata/local/packages/microsoft.windowsterminal_8wekyb3d8bbwe/localstate/settings.json")
        || path.ends_with("/appdata/local/packages/microsoft.windowsterminalpreview_8wekyb3d8bbwe/localstate/settings.json")
        || path.ends_with("/appdata/local/microsoft/windows terminal/settings.json")
    )
}

pub(super) fn terminal(check: &mut Check<'_>) {
    for file in check.files(terminal_settings) {
        check.sources += 1;
        let Some(bytes) = check.read(&file, 8 * 1024 * 1024) else {
            continue;
        };
        let parsed = decode_text(&bytes, false)
            .and_then(|s| strip_jsonc(&s))
            .and_then(|s| serde_json::from_str::<Value>(&s).map_err(|e| e.to_string()));
        let data = match parsed {
            Ok(data) => data,
            Err(e) => {
                check.issue("malformed", format!("{}: {e}", file.physical.display()));
                continue;
            }
        };
        let Some(profiles) = data.get("profiles") else {
            continue;
        };
        let mut entries = Vec::new();
        if let Some(array) = profiles.as_array() {
            entries.extend(array);
        } else if profiles.is_object() {
            if let Some(defaults) = profiles.get("defaults") {
                entries.push(defaults);
            }
            if let Some(list) = profiles.get("list").and_then(Value::as_array) {
                entries.extend(list);
            }
        }
        for entry in entries {
            let Some(command) = entry
                .get("commandline")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            else {
                continue;
            };
            let mut f = check.file_finding(&file, command, "USER");
            f.resolve_target.clear();
            f.description="Windows Terminal configured profile command; ordinary shells also use this setting. Configuration does not prove execution.".into();
            check.scan.findings.push(f);
        }
    }
}

/// Strip comments/trailing commas while preserving JSON string literals and escapes.
fn strip_jsonc(text: &str) -> Result<String, String> {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut string = false;
    while let Some(c) = chars.next() {
        if string {
            result.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    result.push(next);
                } else {
                    return Err("unterminated JSON escape".into());
                }
            } else if c == '"' {
                string = false;
            }
        } else if c == '"' {
            string = true;
            result.push(c);
        } else if c == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for c in chars.by_ref() {
                if c == '\n' {
                    result.push('\n');
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut closed = false;
            while let Some(c) = chars.next() {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    closed = true;
                    break;
                }
            }
            if !closed {
                return Err("unterminated JSON block comment".into());
            }
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    if string {
        return Err("unterminated JSON string".into());
    }
    let mut out = String::with_capacity(result.len());
    let mut chars = result.chars().peekable();
    let mut string = false;
    while let Some(c) = chars.next() {
        if string {
            out.push(c);
            if c == '\\' {
                if let Some(n) = chars.next() {
                    out.push(n);
                }
            } else if c == '"' {
                string = false;
            }
        } else if c == '"' {
            string = true;
            out.push(c);
        } else if c == ','
            && chars
                .clone()
                .find(|c| !c.is_whitespace())
                .is_some_and(|c| matches!(c, '}' | ']'))
        {
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jsonc_preserves_url_and_comment_markers_in_strings() {
        let raw = r#"// header
        {"profiles":{"defaults":{"commandline":"cmd /c https://site/a/*b*/"},"list":[{"commandline":"pwsh.exe",},],},}"#;
        let parsed: Value = serde_json::from_str(&strip_jsonc(raw).unwrap()).unwrap();
        assert_eq!(
            parsed["profiles"]["defaults"]["commandline"],
            "cmd /c https://site/a/*b*/"
        );
        assert!(strip_jsonc("{/* unterminated").is_err());
    }
    #[test]
    fn ini_keeps_duplicates_and_reports_damage() {
        let (commands, damage) = parse_ini(
            "[Startup]\n0CmdLine=a.cmd\n0Parameters=-a\n0CmdLine=b.cmd\n0Parameters=-b\n[Startup]\n1CmdLine=c.cmd\nbad line",
        );
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].2, "-a");
        assert_eq!(commands[1].2, "-b");
        assert_eq!(damage, vec![8]);
    }
    #[test]
    fn only_group_policy_registration_files_are_configuration_sources() {
        assert!(script_config(
            "windows/system32/grouppolicy/machine/scripts/scripts.ini"
        ));
        assert!(script_config(
            "windows/system32/grouppolicy/user/scripts/psscripts.ini"
        ));
        assert!(script_config(
            "windows/system32/grouppolicyusers/s-1-5-21-1-2-3-1001/user/scripts/scripts.ini"
        ));
        assert!(!script_config(
            "windows/system32/grouppolicy/machine/scripts/startup/scripts.ini"
        ));
        assert!(!script_config(
            "windows/system32/grouppolicy/other/scripts/scripts.ini"
        ));
        assert!(!script_config(
            "windows/system32/grouppolicyusers/s-1-5-21-1-2-3-1001/user/scripts/deeper/scripts.ini"
        ));
    }
}
