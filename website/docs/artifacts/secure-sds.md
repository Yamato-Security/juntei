# Security Descriptors (`$Secure:$SDS`)

## What `$SDS` is

Every file and directory on NTFS has a **security descriptor**: its owner, its primary group, a
**DACL** (who may do what with it) and a **SACL** (what access is audited, and its integrity label).
NTFS keeps each distinct descriptor once, in the `$SDS` stream of the `$Secure` file (MFT entry 9),
and gives it a **security ID**. A file's `$STANDARD_INFORMATION` records only that security ID: the
`SecurityID` column of [`parse-ntfs mft`](mft.md). `$SDS` turns it into an owner and ACLs.

`$SDS` is in 256 KiB blocks, each followed by a copy of it (its mirror). Each entry has a hash of
its descriptor, which NTFS uses to find a descriptor that already exists (`$SDH`), and the
security ID (`$SII` indexes it).

<span class="evidence-strength">Best conclusion:</span> the files whose `SecurityID` names a
descriptor had, when their `$MFT` record was last written, that owner and those ACLs. A file's
owner is usually the account that created it, but an administrator, an installer, or a program
taking ownership can change it.

## Location and collection

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

Collect it with a raw-aware forensic tool; collections usually name it `$Secure_$SDS`, since `:`
cannot be in a file name. Collect the same volume's `$MFT` with it.

## Analyst cautions

- An owner is a SID. Only well-known SIDs (`NT AUTHORITY\SYSTEM`, `BUILTIN\Administrators`,
  `NT SERVICE\TrustedInstaller`, …) and the built-in accounts of a machine or domain (`-500`
  `Administrator`, …) are named. Resolve the others with the host's `SAM`
  ([SAM accounts](sam.md)), its profile list ([System Information](system-info.md)), or the domain.
- A descriptor is shared by every file with the same owner and ACLs, and usually stays in `$SDS`
  after those files are deleted. A descriptor without files in the `$MFT` is not by itself a
  deleted file.
- An ACE is not access history. The DACL says who *may* access a file; event logs and other
  artifacts say who did. The SACL's audit entries describe configuration, not what was logged.
- ACEs are listed in their order in the ACL. Juntei does not calculate effective access: deny
  entries, inheritance, group membership, privileges, and the owner's implicit rights all count.
- `DACL` and `SACL` write each ACE as `icacls` does. `SDDL` represents the supported permissions, with masks in hexadecimal and accounts as SIDs. It is not a byte-for-byte copy of the binary descriptor; unrepresentable conditional text leaves it empty and is diagnosed.

## Parse with Juntei

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

The summary counts the descriptors and lists their owners, with how many descriptors each owns,
the descriptors with a NULL DACL (no restriction at all), and any damage.

## CSV output fields

| Field | Interpretation |
|---|---|
| `SourcePath` | Input file. |
| `SecurityID` | The security ID: join it with the `SecurityID` of [`parse-ntfs mft`](mft.md) for the same volume. |
| `Hash` | The hash recorded with the descriptor. |
| `EntryOffset` | Where the entry is in the stream. |
| `FromMirror` | `True` for a copy read from the mirror block: one that differs from its entry (listed after it), or the copy of a damaged entry. |
| `Size` | The entry's size, its 20-byte header included. |
| `Owner`, `OwnerName` | The owner's SID, and its name if well known. |
| `Group`, `GroupName` | The primary group's SID, and its name if well known. Windows does not use it for access checks. |
| `Control` | The descriptor's control flags: `SE_DACL_PRESENT`, `SE_DACL_PROTECTED` (inheritance blocked), `SE_DACL_AUTO_INHERITED`, … joined by `|`. |
| `DACL-ACE-Count`, `DACL` | The DACL's entries, separated by `; `. Empty if there is no DACL; `NULL DACL: no restriction` if the descriptor grants everyone everything. |
| `SACL-ACE-Count`, `SACL` | The SACL's entries: audit entries, the mandatory integrity label, process trust labels, and resource attributes. |
| `SDDL` | The whole descriptor in SDDL. Empty only if something in it is not decoded. |
| `Problems` | A hash that is not the descriptor's, parts that cannot be decoded, ACEs whose type is not known. |

Each entry of `DACL` and `SACL` is written as `icacls` writes it: the account, the type if it is
not an allow entry, the inheritance flags, and the rights.

| Part | Meaning |
|---|---|
| Account | The well-known name, or the SID. |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | The ACE type, when not an allow entry. A mandatory label has no type shown. |
| `(I)` | Inherited from the parent directory. |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | Inherited by files (object inherit), by directories (container inherit), only by them and not this object (inherit only), and only one level down (no propagate). |
| `(SA)`, `(FA)` | An audit entry for successful or failed access. |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | Full control, modify, read and execute, read, write. |
| `(RD,WD,AD,…)` | Specific rights otherwise: read data, write data, append data, read and write extended attributes (`REA`, `WEA`), execute (`X`), delete child (`DC`), read and write attributes (`RA`, `WA`), delete (`DE`), read control (`RC`), write DAC (`WDAC`), write owner (`WO`), synchronize (`S`), and the generic rights `GA`, `GR`, `GW`, `GE`. `N` is no access. |
| `(NW)`, `(NR)`, `(NX)` | A mandatory label's policy: no write up, no read up, no execute up. |
| `if (…)` | A callback ACE's condition, such as `(WIN://SYSAPPID Contains "…")`: the entry applies only to processes for which it is true. Windows uses them to give app container packages access. |

Process trust labels and resource attributes show their mask in hexadecimal, and a resource
attribute its claim (`("IMAGELOAD",TU,0x0,1)`).

JSON/JSONL have the same fields and names, with numbers as numbers, `null` for empty values and
`Problems` as a list.

## Investigation strategy

1. Parse the same volume's `$MFT` and join its `SecurityID` with this output's: each file gets its
   owner and ACLs.
2. Look at the owners that are user accounts (`S-1-5-21-…`): which files a suspicious account
   owns, and which accounts own files in system directories.
3. Look for permissive ACLs on directories where programs run or load from: `Everyone`,
   `Authenticated Users` or `User` with `(F)`, `(M)` or `(W)`, and NULL DACLs.
4. A descriptor used by one file only, or by an unusual set of files, can show files that were
   created or changed outside normal installation.
5. Corroborate ownership and permission changes with Security event log entries (4670, 4663 with a
   SACL) and the [USN Journal](usn-journal.md)'s `SECURITY_CHANGE` records.

## Further Details

- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (security descriptors, ACEs, conditional expressions)
- [Microsoft: Security Descriptor String Format (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: NTFS format documentation](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (its `secure` crate)

With structured output on stdout (`-t` without `-o`), stream-level recovery warnings go to stderr, so pipes retain readable CSV/JSON/JSONL without concealing damaged or recovered regions.
