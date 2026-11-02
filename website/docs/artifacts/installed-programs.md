# Installed Programs

## What Windows keeps

Windows and installers record installed software in several registry lists. Each list has one key
per program, product, distribution or package; Juntei gives one record per key.

| Source | Hive and key | What it holds |
|---|---|---|
| `uninstall` | `SOFTWARE` and `NTUSER.DAT` `…\CurrentVersion\Uninstall\*`, and `SOFTWARE\Wow6432Node\…` | What Settings and *Programs and Features* list: name, version, publisher, install date, folders, uninstall commands, size, whether it is hidden |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | Windows Installer products: product name and version, the package's file name and the folder it was installed from |
| `app_path` | `SOFTWARE` and `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | Programs Windows starts by name (`Win+R`, `start tool`): the program's path and its folder |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | Windows Subsystem for Linux distributions: name, where its files or disk are, WSL 1 or 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | Packaged (Store) apps: provisioned for every user, Windows' own, staged, deprovisioned, and each user's |

<span class="evidence-strength">Best conclusion:</span> this program, product or package was
installed (or registered) on the system, for every user or for this user. The install date is
what the installer wrote; the key's last write is when the entry last changed.

## Location

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

Collect `SOFTWARE`, every `NTUSER.DAT`, and their transaction logs. `SYSTEM` names the host, and
`SOFTWARE`'s `ProfileList` gives each user's SID.

## Analyst cautions

- **Install date and key time.** `InstallDate` is a date (no time) written by the installer, in
  local time; some installers don't write it, and some write another format (kept as written in
  `InstallDateRaw`). An update can rewrite the entry without changing the date, or change the date
  without the program changing much: compare `InstallDate` with `KeyLastWrittenUTC`.
- **Uninstalled software.** A program removed cleanly loses its keys. Use `parse-registry read
  --recover-deleted` for deleted `Uninstall` keys, and Amcache, Prefetch and the `$MFT` for traces
  left behind.
- **Per-user registrations.** `NTUSER.DAT` entries identify software registered for a profile.
  That scope does not establish who ran the installer or whether administrator rights were used.
- **Hidden entries.** `SystemComponent` hides an entry from *Programs and Features*; entries
  without a `DisplayName` (updates, components) are not shown either. They are all reported.
- **Product codes.** A GUID-shaped `Uninstall` key can be matched against a packed
  `msi_product` key through `ProductCode`. The GUID-shaped name alone does not prove that
  Windows Installer created the entry.
- **App Paths.** These are registrations used to locate programs, rather than execution
  records. The `Path` value can hold multiple search directories and is not necessarily the
  installation directory.
- **Publisher.** This is registry metadata supplied by the installer, not verified signature
  information.
- **Architecture.** `Wow6432Node` holds 32-bit programs on 64-bit Windows (`Architecture` `x86`,
  `Wow64` `True`); other entries are the system's own architecture (empty). A package's
  architecture is its own (`x64`, `neutral`, …).
- **WSL.** `Version` is the WSL version from the distribution's `Flags` (`0x8`: it runs in WSL 2's
  virtual machine). The distribution's own `Version` value (in `Details`) is its file system's
  version, 2 on WSL 1 distributions too. `BasePath` holds its `ext4.vhdx` (WSL 2) or `rootfs`
  (WSL 1): collect it to look inside.
- **Packages.** The `<SID>` lists are the packages registered for each user; most are Windows'
  own. `Deprovisioned` packages were removed for new users, `EndOfLife\<SID>` ones for that user.
- **Damaged values.** Malformed numbers, wrong value types and invalid UTF-16 are reported in
  `Problems` and retained in `DataRaw`, including an empty hex string for a present value with
  zero bytes. A clean empty string in an optional numeric field is treated as absent. A package
  version must contain four decimal components, each within the unsigned 16-bit range.
  Hive-level warnings stay on stderr when `-t` streams records to stdout.

## Parse with Juntei

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.scope == "user" and .source != "package")'
```

The summary lists the programs installed last (by `InstallDate`), what users installed for
themselves, and the WSL distributions.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, hive file, and `SOFTWARE` or `NTUSER.DAT`. |
| `Username`, `UserSid` | A user hive's user (its profile folder and `ProfileList` SID), or the user a package is listed for. |
| `Source`, `Scope` | The list (`uninstall`, `msi_product`, `app_path`, `wsl`, `package`), and `machine` or `user`. |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher`; a product's `ProductName` and `Version`; an App Path's program file name; a distribution's name and WSL version; a package's name and version. |
| `InstallDate`, `InstallDateRaw` | `InstallDate` as a date when written `YYYYMMDD`; otherwise as written. |
| `KeyLastWrittenUTC` | When the entry's key last changed. |
| `InstallLocation`, `InstallSource` | `InstallLocation` (an App Path's `Path`, a distribution's `BasePath`, a package's `Path`); `InstallSource` (a product's last-used source folder). |
| `Program` | An App Path's program. |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`. |
| `SizeKb`, `SystemComponent`, `WindowsInstaller` | `EstimatedSize` in KiB; whether it is hidden; whether Windows Installer installed it. |
| `ProductCode` | The Windows Installer product code (`{GUID}`). |
| `Architecture`, `Wow64` | `x86` under `Wow6432Node`, or a package's architecture; whether the key is under `Wow6432Node`. |
| `Details` | More: `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User` (the account that ran an Inno Setup installer); a product's `PackageCode`, `PackageName`; a distribution's `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName`, `Default`; a package's `List`, `ResourceId`, `PublisherId`. |
| `KeyPath` | The entry's key. |
| `DataRaw` | The bytes (hex) of values that are not what their names say. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Values that could not be read or are not what their names say. |

## Investigation strategy

1. Sort by `InstallDate` and `KeyLastWrittenUTC` around the incident; look at remote access
   tools, tunnelling and file transfer clients, archivers, scripting runtimes and security tools.
2. Review `Scope` `user` records: software installed without administrator rights.
3. Check `InstallLocation`, `InstallSource` and `UninstallCommand` for user-writable folders
   (`Downloads`, `AppData`, `Temp`, `ProgramData`).
4. Look for WSL distributions, and collect their `BasePath`.
5. Correlate with Amcache and Prefetch (the installer and the program running), the `$MFT` (the
   install folder's creation) and the Application event log (MsiInstaller events).

## Further Details

- [Microsoft: Uninstall registry key](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: App registration (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: Package identity overview](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
