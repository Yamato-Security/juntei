---
hide:
  - navigation
  - toc
---

<div class="jt-hero" markdown>

<img class="jt-hero-art" src="assets/juntei-goddess.png"
     alt="Juntei, Goddess of Digital Forensics, surrounded by Windows forensic artifacts"
     width="1448" height="1086">

<p class="tagline">
<strong>Juntei</strong> is Yamato Security's cross-platform parser for Windows forensic artifacts
that are <em>not</em> event logs. It turns collected evidence into consistent CSV, JSON, and JSONL
records while preserving the context an investigator needs to avoid false conclusions.
</p>

<div class="jt-cta" markdown>
[Start here :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Learn the artifacts :material-archive-search:](artifacts/index.md){ .md-button }
[Command reference :material-console:](commands/index.md){ .md-button }
[GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/juntei){ .md-button }
</div>

</div>

## From raw artifact to defensible conclusion

Juntei can tell you what an artifact records; this site explains what that record **does and does
not prove**. Start with an artifact page even if you have never performed Windows DFIR before.
Each guide covers its purpose, on-disk location, retention and collection caveats, parsing command,
output fields, interpretation, correlations, and deeper reading.

<div class="grid cards" markdown>

-   :material-file-tree:{ .lg .middle } __File-system history__

    ---

    Use the `$MFT` and USN Journal to reconstruct file existence, names, movement, deletion,
    metadata changes, and alternate data streams.

-   :material-play-circle-outline:{ .lg .middle } __Program evidence__

    ---

    Prefetch can strongly support execution. Amcache and ShimCache add inventory, path, hash,
    compatibility, and historical context—but require careful interpretation.

-   :material-account-clock:{ .lg .middle } __User activity__

    ---

    LNK files, Jump Lists, RecentDocs, and SRUM expose opened items, application activity,
    removable or network locations, resource use, and historical user context.

-   :material-shield-bug:{ .lg .middle } __Security evidence__

    ---

    Defender quarantine can preserve threat labels, original paths, download origins, hashes,
    owners, and even safely recover quarantined bytes for isolated analysis.

-   :material-database-search:{ .lg .middle } __Registry state__

    ---

    Read allocated and recoverable registry data, replay transaction logs, and inspect services,
    scheduled tasks, local accounts, and recent-document state.

-   :material-link-variant:{ .lg .middle } __Correlate, do not guess__

    ---

    No single artifact tells the whole story. The guides show how to combine independent sources
    into an evidence-backed timeline.

</div>

!!! warning "Forensic output is evidence, not a verdict"
    A path that looks suspicious is a lead. A timestamp may describe a file's metadata rather than
    execution. A missing record does not prove an action did not happen. Preserve source evidence,
    record tool versions and commands, and corroborate important conclusions.
