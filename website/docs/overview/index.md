# About Juntei

Juntei is a Windows forensic artifact parser written in memory-safe Rust. It complements
[Hayabusa](https://github.com/Yamato-Security/hayabusa): Hayabusa analyzes Windows event logs,
while Juntei parses persistent file-system, registry, execution, user-activity, resource-usage,
and security-product artifacts.

The same binary runs on Windows, macOS, and Linux. This is useful when an analyst receives a KAPE,
Velociraptor, disk-image, or other offline collection and wants one consistent interface instead
of a separate program for every artifact.

## Design goals

- **Defensible parsing:** Preserve raw values and report damage instead of silently inventing data.
- **Useful for all:** Default usage should be easy to use for beginners but customizable and full
  featured for veterans as well.
- **Standalone binary:** Does not require any runtime libraries to execute. It just works out of
  the box.
- **Blazing fast:** Aims to be the fastest parser while also not requiring a large amount of
  memory.
- **Cross-platform use:** Analyze Windows evidence without requiring a Windows workstation.
- **Automation:** CSV for spreadsheets and timeline tools; JSON/JSONL for `jq`, scripts, and data lakes.
- **Safe failure:** Malformed records are reported and skipped where possible.
- **Transparent lineage:** Juntei identifies the open-source parsers and reference tools behind each implementation.

## What Juntei does not do

Juntei does not acquire a live disk, mount a forensic image, or replace evidence preservation. It
does not automatically combine every artifact into a single incident narrative. It parses the
evidence you provide and produces records suitable for investigation and correlation.

Juntei will also not do the analysis for you. A DFIR specialist is required to perform the analysis on the evidence.

## License

Juntei is released under the
[GNU Affero General Public License v3.0](https://github.com/Yamato-Security/juntei/blob/main/LICENSE).
Its parser dependencies and reference projects have their own licenses; see
[Acknowledgements](../acknowledgements.md).
