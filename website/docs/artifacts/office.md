# Microsoft Office (Recent Files, Trusted Documents, Macro Settings)

## What Office records

Microsoft Office keeps, per user in `NTUSER.DAT` and per version and application:

- **Recent files and places:** `File MRU`, the documents an application opened, and `Place MRU`,
  the folders it opened them from, each item with the time it was last opened. Microsoft 365
  also keeps a list per signed-in account (`User MRU\<account>`).
- **Trusted Documents:** when a user clicks *Enable Editing* or *Enable Content* on a document,
  Office remembers it in `TrustRecords`: the document's path, when it was trusted, and whether
  its macros were enabled.
- **Macro settings:** `VBAWarnings` (which macros run), `BlockContentExecutionFromInternet`
  (whether macros in files from the Internet are blocked), `AccessVBOM` (whether programs may
  change VBA projects) and the **Trusted Locations**, folders whose documents' macros run without
  asking. Group Policy can set the same under `Software\Policies`.

A malicious document usually needs the user to enable its macros: a trust record whose macros
were enabled ties the document, the user and a time together.

<span class="evidence-strength">Best conclusion:</span> the user opened this document in this
application (and last at this time); the user enabled this document's editing or macros at
this time; Office was configured to run macros this way.

## Location

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

Versions: `16.0` is Office 2016 and later (2019, 2021, 2024, Microsoft 365), `15.0` Office 2013,
`14.0` Office 2010, `12.0` Office 2007.

An item looks like `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx`: `T`
is the last-opened time (a FILETIME in hexadecimal), the path follows `*`. A trust record is 24
bytes: the FILETIME the document was trusted, then its flags in the last 4 bytes, `0x7FFFFFFF`
when its macros were enabled and `0x00000001` when only its editing was.

## Analyst cautions

- **Times.** `LastOpenedUTC` is each item's own time. `TrustedAtUTC` is when the user trusted
  the document, the moment *Enable Editing* or *Enable Content* was clicked, which can be before
  the most recent opening.
- **Macros enabled, not run.** `MacrosEnabled` says the user enabled the document's macros; it
  doesn't prove a macro ran or what it did. Correlate with process creation, Prefetch, PowerShell
  logs and the document itself.
- **Paths as Office stores them.** Trust records name the document by a path that may use
  `%USERPROFILE%`, forward slashes, a `file:///` URL and URL encoding (`%20`). MRU items of
  cloud documents are URLs.
- **Defaults and effective policy.** Office creates default trusted locations (`Templates`,
  `XLSTART`, `STARTUP`, …). A location's `Date` is a useful clue, not proof of who created it:
  administrators, scripts, policies and copied registry settings can also populate it. Missing
  `VBAWarnings` alone does not establish the effective macro policy. Office version, Internet
  file marking and other management settings also matter.
  [Microsoft explains Internet macro blocking](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked).
- **Deleted documents.** The lists outlive the documents; a missing file can still be shown
  here. Lists are short and are cleared by Office's *Clear unpinned items* and cleanup tools.

## Parse with Juntei

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

The summary lists the documents whose macros were enabled, settings that let macros run more
freely (`VBAWarnings` 1, `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1,
`AllowNetworkLocations` 1), trusted locations users added or a policy set, and the documents
opened last.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, the profile owner (from the path or the shell folders), the SID from the system's `ProfileList`, and the hive. |
| `OfficeVersion`, `App` | The version's key (`16.0`) and the application (`Word`, `Excel`, …). |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security` or `Trusted Locations`. |
| `Account` | The Microsoft 365 account of a `User MRU` list. |
| `Policy` | Whether the setting or location is set by policy. |
| `MRU-Position`, `IsMostRecent` | An item's place in its list (`Item 1` is 0, the most recent). |
| `Path` | The document or folder; a trusted document's path as stored; a trusted location's `Path`. |
| `LastOpenedUTC` | When the item was last opened. |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | When the document was trusted, whether its macros were enabled, and the record's flags. Unknown flags leave `MacrosEnabled` empty and produce a diagnostic. |
| `Setting`, `Value` | A macro setting and its value with what it means (`1 (all macros enabled)`); a trusted location's key (`Location3`) and its other values (`AllowSubFolders=1; Date=…; Description=…`). |
| `KeyPath`, `KeyLastWrittenUTC` | The key and its last write. |
| `DataRaw` | The value's bytes (hex) when they don't decode. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Items without `*` or with a time that isn't one, trust records that aren't 24 bytes, locations without `Path`, wrong types. |

## Investigation strategy

1. Start with `MacrosEnabled` true: the documents, when, and by whom. Find the files (`$MFT`,
   USN Journal, `Zone.Identifier`), their source (browser and mail history) and what ran next.
2. Check the macro settings and trusted locations: all macros enabled, Internet macros not
   blocked, VBA project access, or a user-added location (in `Downloads`, `Temp`, a network share)
   weaken the defaults.
3. Put `LastOpenedUTC` on the timeline with LNK files, jump lists and RecentDocs for the
   documents' paths, including removable and network locations.

## Further Details

- [Microsoft: Enable or disable macros in Microsoft 365 files](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: Trusted documents](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: Add, remove, or change a trusted location](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
