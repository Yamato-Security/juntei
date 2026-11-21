# YARA Scan { #yara-scan }

Scan collected files with the portable Rust engine. Windows, macOS and Linux are supported. Files are read as data and never executed. This command is independent of automatic artifact parsing.

## Choose rules { #choose-rules }

Core is the default and prioritizes accuracy and performance. Extended broadens hunting coverage. Full is the broadest package and can produce more false positives.

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
```

`--rules <FILE_OR_DIRECTORY>`: Local UTF-8 rule file or directory; no download. Directories are searched recursively. Includes stay within the selected rule directory, expand once, and preserve dependency order. Invalid or unsupported rules stop compilation rather than being silently dropped.

## Limits and scope { #limits-and-scope }

All regular files are eligible, including recovered quarantine payloads. Symbolic links and special files are skipped. Archives and disk images are not unpacked; live memory is not scanned. Use stable extracted files and a new output directory outside the evidence.

- `-T, --threads <N>`: Parallel scans (default: CPU count, capped at 8).
- `--timeout <SECONDS>`: Per-file evaluation timeout in seconds (default: 60).
- `--max-file-mib <MIB>`: Maximum file size in MiB (default: 64). Larger files are reported as skipped. More workers and larger buffers increase memory use.
- `--rules-release YYYYMMDD`: Forge release date, or latest (default). The downloaded source and hashes are saved for reproducibility.

## Results { #results }

- `YaraDetections.csv` / `YaraDetections.jsonl`: Rule/file matches, with relative paths, evidence computer names, hashes, metadata and limited match offsets. No matched bytes are exported.
- `YaraFiles.csv` / `YaraFiles.jsonl`: Per-file coverage, including skips, read errors, timeouts and partial results.
- `Summary.json`: Completion status, settings, counts, engine version and source hashes.
- `Rules.yar`: Exact downloaded rules, or a flattened snapshot of local rules.
- `CompilationWarnings.txt` / `CompilationError.txt`: Compilation warnings, or an error when compilation fails.

Exit codes: 0 means complete, 3 incomplete coverage, 2 setup/output failure, and 130 cancellation. A match is a hunting lead, not proof of compromise; no matches do not prove safety. Review rule titles and artifact context. Quarantine must be decoded before its payloads can be identified.

## Further Details { #further-details }

- [Engine: Boreal](https://github.com/vthib/boreal)
- [Rule packages: YARA Forge](https://github.com/YARAHQ/yara-forge)
- [Detailed usage and field reference (English)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.md)
- [Detailed usage and field reference (Japanese)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.ja.md)
