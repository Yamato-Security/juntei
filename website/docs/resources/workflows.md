# Cross-Artifact Workflows

No artifact should carry an important conclusion alone. These workflows show how to turn a lead
into a defensible chain of evidence.

## Suspicious executable execution

1. **Prefetch:** establish retained run time, path/hash context, run count, and loaded files.
2. **`$MFT`:** find the path, size, timestamps, ADS, deleted state, and MFT reference.
3. **USN Journal:** reconstruct creation, writes, rename, and deletion.
4. **Amcache:** add SHA-1, PE metadata, version, and historical path.
5. **ShimCache:** add compatibility-cache path/order and supported execution flag.
6. **SRUM:** look for resource/network use by the same executable/user.
7. **Defender:** check quarantine time, original path, hash, and download origin.
8. **Event logs/EDR:** identify process parent, command line, account, network, and security context.

The conclusion should distinguish file presence, execution, and attribution to a person or intrusion.

## Ransomware file activity

1. Identify encrypted extensions and victim paths in `$MFT`.
2. Use USN reasons and order to reconstruct mass writes/renames/deletes.
3. Use Prefetch to identify candidate encryptors and run times.
4. Use SRUM for CPU/disk/network spikes by the candidate process.
5. Check services and TaskCache for deployment/persistence.
6. Check Defender quarantine and event logs for detection/remediation.

MFT/USN can prove file-system impact even when no execution artifact survives. They generally do
not identify the responsible process without process telemetry or correlation.

## Malicious document / shortcut initial access

1. RecentDocs and Jump Lists identify user/application interaction and MRU context.
2. Parse the matching Recent-folder LNK for target, volume/share, MFT reference, and arguments.
3. Use Zone.Identifier from `$MFT` resident ADS or Defender content for origin URLs/zones.
4. Use Prefetch/event logs for the document application and child processes.
5. Use USN/MFT to follow extracted payload creation and deletion.

## Service persistence

1. Decode service name, `ImagePath`, `ServiceDll`, account, start type, and failure actions.
2. Resolve environment variables and quoted/unquoted path behavior.
3. Locate the executable/DLL in `$MFT` and USN history.
4. Check Prefetch and event logs for actual service/process start.
5. Compare non-current control sets and recovered registry records for earlier configuration.

## Scheduled-task persistence

1. Decode TaskCache actions, principal/context, registration/runtime metadata, and hidden state.
2. Compare with `C:\Windows\System32\Tasks` XML.
3. Correlate task name/GUID with Security and Task Scheduler Operational events.
4. Locate and hash the executable/script; inspect Prefetch and file-system history.
5. Recover deleted TaskCache records if the task was removed.

## Suspected exfiltration

1. Use SRUM to identify unusual bytes sent by app/user/profile and the measurement period.
2. Confirm the executable identity with Prefetch, Amcache, and `$MFT`.
3. Identify files accessed/opened through Jump Lists, LNK, RecentDocs, and event/EDR data.
4. Use proxy, firewall, DNS, packet, or EDR telemetry for destination/protocol.

SRUM volume supports an exfiltration hypothesis but does not identify transferred content or prove
malicious intent by itself.

## Correlation keys worth preserving

| Key | Artifacts |
|---|---|
| Full path / file name | Almost every artifact |
| SHA-1 / SHA-256 | Amcache, Defender, recovered files |
| MFT entry + sequence | `$MFT`, Prefetch, LNK, Jump Lists, USN |
| USN | Amcache, `$MFT`, USN Journal |
| Volume serial / droids | Prefetch, LNK, Jump Lists, MFT object IDs |
| SID / user name | SRUM, SAM, registry, event logs |
| Task GUID / task name | TaskCache, task XML, event logs |
| Service name | Services, event logs, process/driver data |
| Defender EntryId / ScanId | Quarantine, Defender history, event logs |
