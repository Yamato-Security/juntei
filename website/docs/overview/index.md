# About Juntei

Juntei is a Windows forensic artifact parser written in memory-safe Rust. It complements
[Hayabusa](https://github.com/Yamato-Security/hayabusa): Hayabusa analyzes Windows event logs,
while Juntei parses persistent file-system, registry, execution, user-activity, resource-usage,
and security-product artifacts.

The same binary runs on Windows, macOS, and Linux. Analysts can use one consistent interface to examine collected Windows evidence and disk images instead of a separate program for every artifact.

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

Juntei does not acquire a live disk or replace evidence preservation, and the only disk images it
mounts are VHDX images, read-only. It does not automatically combine every artifact into a single
incident narrative. It parses the
evidence you provide and produces records suitable for investigation and correlation.

Juntei will also not do the analysis for you. A DFIR specialist is required to perform the analysis on the evidence.

## License

Juntei is released under the
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) and copyright Shirofune Security Co., Ltd. Its source code was published under the
AGPLv3 before version 1.0.0. The licenses of the crates and other third-party code and data it is
built with are in
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
The projects Juntei builds on and was checked against are listed in
[Acknowledgements](../acknowledgements.md).
