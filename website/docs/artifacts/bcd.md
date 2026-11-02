# Boot Configuration Data (BCD)

## What the BCD store is

Windows Vista and later keep their boot configuration in a registry hive, the BCD store:
`\Boot\BCD` on BIOS systems, `\EFI\Microsoft\Boot\BCD` on the EFI system partition of UEFI
systems. `bcdedit` and the Windows installer edit it; the boot manager reads it at every boot.

The store holds objects, each named after a GUID:

- the **Windows boot manager** (`{bootmgr}`), with the default entry, the display order and the
  menu timeout; on UEFI systems also the **firmware boot manager** (`{fwbootmgr}`) with the
  firmware's boot order;
- one **Windows boot loader** per Windows installation (including the recovery environment), a
  **resume application** (hibernation) and the **memory tester**;
- **inherited settings** objects (`{globalsettings}`, `{bootloadersettings}`, …) whose settings the
  others inherit;
- **device** objects (ramdisk options of the recovery environment).

Each object has elements: a description, a device and path, and settings such as
`recoveryenabled`, `bootstatuspolicy`, `testsigning`, `nointegritychecks` and `safeboot`.

Two kinds of attack change them:

- **Ransomware** turns off recovery so the system can't be repaired:
  `bcdedit /set {default} recoveryenabled No` and
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`. Some families set `safeboot` so
  that Windows restarts in safe mode, where security products don't run.
- **Rootkits and unsigned drivers** may use `testsigning on` or `nointegritychecks on` to weaken boot-time checks. These settings alone do not prove a rootkit ran.

<span class="evidence-strength">Best conclusion:</span> at the time of collection, this boot entry
had this setting. The element's key's last-write time is when the setting was last written.

## Location

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

An element's type is 8 hexadecimal digits: its class (`1` every application, `2` one kind of
application, `3` devices), its format (`1` device, `2` string, `3` object, `4` object list, `5`
integer, `6` boolean, `7` integer list) and its number. The command names elements as
Microsoft documents them (`BcdLibraryBoolean_AutoRecoveryEnabled`).

Collection tools often don't copy the store: the EFI system partition isn't mounted, and `\Boot`
is hidden. Collect it explicitly, or mount the partition. `-d` also reads `BCD-Template`
(`HiveType` says which); the template is what Windows starts from, not the store it boots with.

## Analyst cautions

- **Settings are inherited.** A boot loader inherits from the objects in its
  `BcdLibraryObjectList_InheritedObjects` (usually `{bootloadersettings}`, which inherits
  `{globalsettings}`). `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks`, `SafeBoot` and
  `BootStatusPolicy` are resolved stored values (own, else inherited), and `Inherited` says which
  come from another object. Inheritance is followed for at most eight levels, with cycles bounded. A referenced object can be absent from a partial store. Empty fields do not establish an effective Windows default. Firmware and Secure Boot may also prevent a stored option from taking effect.
- **Not set is not off.** An empty `RecoveryEnabled` means the element isn't there; Windows'
  default applies.
- **Times are per element.** Each element's key time (`key_last_written` in `Elements`) is when it
  was last written. Windows rewrites some elements at updates and upgrades; compare with the other
  elements' times.
- **Devices are partially decoded.** A GPT partition is given as its partition's and disk's GUIDs;
  a ramdisk as its options object. Other devices keep their bytes only (`data_raw`).
- **Undocumented elements** (`0x15000065`, the `0x4…` class) have no name and keep their bytes.

## Parse with Juntei

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.test_signing == true or .no_integrity_checks == true or .recovery_enabled == false)'
```

The summary lists each store's default entry and every test signing, integrity check, recovery,
boot status policy (any `Ignore…`) and safe mode setting, with the time it was set or the object
it is inherited from.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, the hive, and `BCD` or `BCD-Template`. |
| `ObjectGuid`, `ObjectType`, `ObjectTypeName` | The object's GUID, its type (`0x10200003`) and what it is (`Windows boot loader`, `inherited settings for Windows boot loader`, …). |
| `Description`, `Path`, `Device` | The description, the application's path, and its device (a GPT partition and disk, or a ramdisk's options object). |
| `IsDefault`, `DisplayOrder` | Whether the Windows boot manager boots it by default, and its place in the boot menu. |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | `recoveryenabled`, `testsigning`, `nointegritychecks`, as resolved from the store. |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`, `Network`, `DsRepair`) and `bootstatuspolicy` (`IgnoreAllFailures`, …), with the number. |
| `Inherited` | Which of these come from another object, and which. |
| `Elements` | Every element: its type, name, value, bytes (for devices, undocumented types and data that doesn't decode) and key time. |
| `KeyLastWrittenUTC` | The object's key's last write. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Objects without a type, elements whose data doesn't fit their format. |

## Investigation strategy

1. Check the default entry and every boot loader for `TestSigning`, `NoIntegrityChecks`,
   `RecoveryEnabled = false`, an `Ignore…` `BootStatusPolicy` and `SafeBoot`.
2. Use the element times to date the change, and correlate with `bcdedit.exe` in Prefetch,
   Amcache, process creation events (4688, Sysmon 1) and the System event log.
3. Look for boot entries or devices that shouldn't be there (another loader path, a ramdisk), and
   for a changed default or display order.
4. Compare the store with `BCD-Template` and with another system of the same build.

## Further Details

- [Microsoft: BCD boot options reference](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (object types)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
