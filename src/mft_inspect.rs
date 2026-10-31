//! `parse-mft --dump-entry` and `--list-directory`: one entry in full detail, or what a directory
//! holds, on the terminal.

use crate::cli::{EntrySpec, MftOptions};
use crate::mft::JsonEntry;
use crate::mft_csv::{self, NamesInUse};
use crate::util::{error_msg, fmt_num};
use mft::MftParser;
use mft::entry::ZERO_HEADER;
use std::fmt::Write;
use std::fs::File;
use std::io::BufReader;

/// Run `--dump-entry` or `--list-directory`; returns the exit status.
pub fn run(opts: &MftOptions) -> i32 {
    let no_color = opts.common.no_color;
    let Some(path) = &opts.filepath else {
        error_msg(
            no_color,
            "--dump-entry and --list-directory read one $MFT: give it with -f.",
        );
        return 1;
    };
    if opts.output.is_some()
        || opts.output_type.is_some()
        || opts.extract.is_some()
        || opts.ranges.is_some()
        || opts.recover_slack
        || opts.has_csv_options()
    {
        error_msg(
            no_color,
            "--dump-entry and --list-directory show one entry or directory on the terminal: they don't take -o, -t, -e, -r, -x or the CSV options.",
        );
        return 1;
    }
    if !path.is_file() {
        error_msg(
            no_color,
            &format!("The file does not exist: {}", path.display()),
        );
        return 1;
    }
    let mut parser = match MftParser::from_path(path) {
        Ok(parser) => parser,
        Err(e) => {
            error_msg(
                no_color,
                &format!("Could not parse {}: {e}", path.display()),
            );
            return 1;
        }
    };
    let shown = match (&opts.dump_entry, &opts.list_directory) {
        (Some(spec), _) => dump_entry(&mut parser, *spec),
        (None, Some(target)) => list_directory(&mut parser, target),
        (None, None) => unreachable!("run only with --dump-entry or --list-directory"),
    };
    match shown {
        Ok(text) => {
            print!("{text}");
            0
        }
        Err(msg) => {
            error_msg(no_color, &msg);
            1
        }
    }
}

/// Whether `sequence` names the record whose header has `actual`: references to an entry in use
/// carry its sequence number; those to a freed entry, the one before (NTFS raises it when the
/// record is freed).
fn same_sequence(sequence: u16, actual: u16, in_use: bool) -> bool {
    sequence == actual || (!in_use && sequence.wrapping_add(1) == actual)
}

/// One entry: its header, its attributes and slack as `parse-mft -t json` writes them, and its
/// bytes.
fn dump_entry(parser: &mut MftParser<BufReader<File>>, spec: EntrySpec) -> Result<String, String> {
    let count = parser.get_entry_count();
    if spec.entry >= count {
        return Err(format!(
            "The $MFT has {} entries: there is no entry {}.",
            fmt_num(usize::try_from(count).unwrap_or(usize::MAX)),
            spec.entry
        ));
    }
    let entry = parser
        .get_entry(spec.entry)
        .map_err(|e| format!("Entry {} could not be read: {e}", spec.entry))?;
    if &entry.header.signature == ZERO_HEADER {
        return Err(format!("Entry {} is empty (never used).", spec.entry));
    }
    let header = &entry.header;
    let in_use = entry.is_allocated();
    if let Some(sequence) = spec.sequence
        && !same_sequence(sequence, header.sequence, in_use)
    {
        return Err(format!(
            "Entry {} has sequence number {}, not {sequence}.",
            spec.entry, header.sequence
        ));
    }
    let full_path = parser.get_full_path_for_entry(&entry).ok().flatten();
    let names = NamesInUse::of(parser);
    let found = mft_csv::slack_names(&entry, spec.entry);
    let mut slack = found.clone().unwrap_or_default();
    for name in &mut slack {
        name.copy_of_entry_in_use = names.has(name);
    }

    let mut out = String::new();
    let size = entry.data.len() as u64;
    let flags = header.flags.bits();
    let described: Vec<&str> = [
        (0x1, "in use"),
        (0x2, "directory"),
        (0x4, "in $Extend"),
        (0x8, "view index"),
    ]
    .iter()
    .filter(|(bit, _)| flags & bit != 0)
    .map(|(_, name)| *name)
    .collect();
    let _ = writeln!(
        out,
        "Entry {}-{} at offset 0x{:X} of the $MFT ({} bytes)",
        spec.entry,
        header.sequence,
        spec.entry * size,
        size
    );
    let _ = writeln!(
        out,
        "  Path:               {}",
        full_path.as_ref().map_or_else(
            || "(unknown)".to_string(),
            |p| p.to_string_lossy().replace('/', "\\")
        )
    );
    let _ = writeln!(
        out,
        "  Signature:          {}",
        String::from_utf8_lossy(&header.signature)
    );
    let _ = writeln!(
        out,
        "  Flags:              0x{flags:04X} ({})",
        if described.is_empty() {
            "free".to_string()
        } else {
            described.join(", ")
        }
    );
    let _ = writeln!(out, "  Hard links:         {}", header.hard_link_count);
    let base = &header.base_reference;
    let _ = writeln!(
        out,
        "  Base record:        {}",
        if base.entry == 0 && base.sequence == 0 {
            "(this is a base record)".to_string()
        } else {
            format!("{}-{}", base.entry, base.sequence)
        }
    );
    let _ = writeln!(
        out,
        "  $LogFile LSN:       {}",
        header.metadata_transaction_journal
    );
    let _ = writeln!(
        out,
        "  Size in use:        {} of {} bytes",
        header.used_entry_size, header.total_entry_size
    );
    let _ = writeln!(
        out,
        "  Update sequence:    {}",
        match entry.valid_fixup {
            Some(true) => "applied",
            Some(false) => "applied, but a sector did not hold its value (written in part)",
            None => "not applied (not a FILE record)",
        }
    );
    match &found {
        Ok(_) => {
            let _ = writeln!(
                out,
                "  Names in slack:     {} ({} that no file in use has)",
                slack.len(),
                slack.iter().filter(|n| !n.copy_of_entry_in_use).count()
            );
        }
        Err(e) => {
            let _ = writeln!(out, "  Names in slack:     not read: {e}");
        }
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Attributes and slack, as parse-mft -t json -x writes them:"
    );
    let json = JsonEntry {
        full_path,
        entry: &entry,
        slack_file_names: Some(slack),
        slack_problem: found.err(),
    };
    let _ = writeln!(
        out,
        "{}",
        serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Record bytes (update sequence array applied; the size in use ends at 0x{:X}):",
        header.used_entry_size
    );
    hex_dump(&mut out, &entry.data);
    Ok(out)
}

/// `offset: 16 bytes in hex |ASCII|` lines.
fn hex_dump(out: &mut String, bytes: &[u8]) {
    for (line, chunk) in bytes.chunks(16).enumerate() {
        let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02X}")).collect();
        let ascii: String = chunk
            .iter()
            .map(|&b| {
                if (b' '..=b'~').contains(&b) {
                    char::from(b)
                } else {
                    '.'
                }
            })
            .collect();
        let _ = writeln!(out, "{:04X}  {:<47}  |{ascii}|", line * 16, hex.join(" "));
    }
}

/// What a directory holds: one line per name in it.
fn list_directory(parser: &mut MftParser<BufReader<File>>, target: &str) -> Result<String, String> {
    let listing = mft_csv::list_directory(parser, target)?;
    let mut out = String::new();
    let (entry, sequence) = listing.directory;
    let _ = writeln!(
        out,
        "Directory {} (entry {entry}-{sequence}, {})",
        listing.path,
        if listing.in_use { "in use" } else { "deleted" }
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "{:<16}{:<8}{:<6}{:>14}  {:<30}Name",
        "Entry-Sequence", "In use", "Type", "Size", "Modified (UTC)"
    );
    for child in &listing.children {
        let _ = writeln!(
            out,
            "{:<16}{:<8}{:<6}{:>14}  {:<30}{}",
            format!("{}-{}", child.entry, child.sequence),
            if child.in_use { "True" } else { "False" },
            if child.is_dir { "Dir" } else { "File" },
            child.size,
            mft_csv::time(child.modified),
            child.name
        );
    }
    let directories = listing.children.iter().filter(|c| c.is_dir).count();
    let deleted = listing.children.iter().filter(|c| !c.in_use).count();
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "{} names: directories {}, files {}, deleted {}",
        fmt_num(listing.children.len()),
        fmt_num(directories),
        fmt_num(listing.children.len() - directories),
        fmt_num(deleted)
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_numbers_of_freed_entries() {
        assert!(same_sequence(5, 5, true));
        assert!(!same_sequence(4, 5, true));
        assert!(same_sequence(4, 5, false));
        assert!(same_sequence(5, 5, false));
        assert!(!same_sequence(3, 5, false));
    }

    #[test]
    fn hex_dump_lines() {
        let mut out = String::new();
        hex_dump(&mut out, b"FILE0\x00\x03\x00abcdefghij");
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines[0],
            "0000  46 49 4C 45 30 00 03 00 61 62 63 64 65 66 67 68  |FILE0...abcdefgh|"
        );
        assert_eq!(lines[1], format!("0010  {:<47}  |ij|", "69 6A"));
        assert_eq!(lines.len(), 2);
    }
}
