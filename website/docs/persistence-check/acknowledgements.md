# Persistence Check Acknowledgements

Juntei's `check-persistence` command builds on two generations of community persistence hunting.
Thank you to the authors and contributors who made their techniques, implementations, and research
available to investigators.

## PyrsistenceSniper: the Python project

[**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper), by
[Hexastrike](https://github.com/Hexastrike), is the direct reference for this feature. Juntei ports
its 130-check catalogue to native Rust, along with the default allowlist, LOLBAS snapshot, and
classification behavior. The reference version is pinned to
[commit `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f).

The Python project is MIT licensed. Juntei retains its copyright and license notice in
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt).
Juntei's implementation runs offline without a Python interpreter. Its collection handling,
coverage reporting, and conservative interpretation of incomplete evidence are documented in the
[command guide](../commands/check-persistence.md).

## PersistenceSniper: the original PowerShell tool

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) is the original PowerShell
persistence hunting project that inspired PyrsistenceSniper. PyrsistenceSniper credits
Federico Lagrasta; the original repository also credits Riccardo Ancarani and other
contributors. We thank the original project and its contributors for making Windows persistence
mechanisms easier to investigate.

Juntei's direct porting reference is the Python project. The original PowerShell tool is credited
for that lineage. Juntei also independently implements the mechanisms proposed by
[hongson11698](https://github.com/hongson11698) in the original project's
[scheduled-task COM resolution PR #31](https://github.com/last-byte/PersistenceSniper/pull/31)
and [Start Menu library/CLSID-folder PR #32](https://github.com/last-byte/PersistenceSniper/pull/32).
These add to the Python reference catalogue; this does not imply that every live-system feature in either upstream project
is implemented by Juntei's offline command.

## Supporting research and tools

- [MITRE ATT&CK](https://attack.mitre.org/) provides the technique identifiers carried in the
  catalogue. A technique mapping describes a possible use of a mechanism; a matching registry value
  or file is not, by itself, an ATT&CK-confirmed attack.
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  is also acknowledged by PyrsistenceSniper as an inspiration for examining Windows autostarts.
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) documents legitimate Windows binaries that can
  be abused. Recognizing one of these binaries helps interpret a command, but does not make that
  command malicious automatically.

For the registry, NTFS, Shell Link, event-log, and other parser foundations used throughout Juntei,
see the [project-wide acknowledgements](../acknowledgements.md).

Return to the [persistence mechanisms guide](index.md).
