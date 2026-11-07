# User Access Logging (UAL)

## What UAL is

User Access Logging (UAL) is a Windows Server feature, installed and enabled by default since
Windows Server 2012, that aggregates the requests clients make to the server's roles in ESE
databases. For each role, client address and account, it keeps when the client was first and last
seen in the year, the total number of requests, and a count for each day of the year. Roles include
File Server (SMB), Active Directory Domain Services, Active Directory Certificate Services, DHCP
Server, Print and Document Services, Remote Access, Web Server (IIS) and Windows Server Update
Services.

UAL is valuable for lateral movement investigations: its client rows show which addresses and
accounts used a server, and on which days, often long after the event logs have rolled over.

<span class="evidence-strength">Best conclusion:</span> a client at the recorded address,
authenticated as the recorded account, used the role on each day that has a count.

## Location and collection

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` holds the most recent activity. `SystemIdentity.mdb` describes the server, names the
roles, and lists the `{GUID}.mdb` file that holds each year (`CHAINED_DATABASES`). The UAL service
keeps the databases open: collect the whole `Sum` directory with a forensic collector or from a
volume snapshot.

!!! warning "Dirty databases"
    `Current.mdb` is often dirty when collected from a running server. Juntei reads it as collected
    and reports it, but does not replay `Svc*.log`: activity existing only in the transaction logs
    is missing.

## Analyst cautions

- UAL counts requests per day; it does not log each request. The only times of a client row are
  `InsertDate` and `LastAccess`.
- The days of `DailyAccesses` are UTC days.
- `Address` is the address the server saw: a NAT gateway, proxy or VPN can stand between it and the
  real client. `::1` and `127.0.0.1` are the server itself.
- Parse every database of the `Sum` directory: the same client can have rows in `Current.mdb` and in
  the `{GUID}.mdb` of its year.
- Missing rows do not prove that nothing connected: UAL can be turned off (`Disable-Ual`, or by
  stopping the User Access Logging Service), and a dirty database misses its newest changes.
- Without `SystemIdentity.mdb`, `RoleName` comes from the names Windows Server gives its roles.
- A zero or invalid `LastAccess` cannot establish the year. Juntei then uses a valid `InsertDate`, followed by the yearly database metadata; without a valid year it retains `DayOfYear`.
- Summary totals add all source rows. `Current.mdb`, yearly databases and snapshots can overlap, so these are not deduplicated connection or logon counts.

## Parse with Juntei

```text
Usage: juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Current.mdb, {GUID}.mdb and SystemIdentity.mdb
  -f, --file <FILE>      Parse one UAL database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl \
  | jq 'select(.Table == "CLIENTS") | {Address, AuthenticatedUserName, RoleName, LastAccess}'
```

## Output tables

CSV creates one file per table, with only its header when the table is empty. JSON and JSONL rows
carry `SourceFile` (the database) and `Table` (the table's name in the database).

| CSV file | Table | Contents |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | Each role's clients: address, account, first and last access of the year, total and daily request counts. |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | When each role was first and last used. |
| `UAL-DNS.csv` | `DNS` | Address and host name pairs, with when each was last seen. |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Hyper-V virtual machines: identifiers, serial number, creation and last active times. |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | The server: name, domain, Windows version, product type, hardware, time zone and last boot; a new row can show a rename, a domain join or a promotion to domain controller. |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | The `{GUID}.mdb` file of each year. |
| `UAL-Roles.csv` | `ROLE_IDS` | Role GUIDs with their names. |
| `UAL-<table>.csv` | Another table | Every column of a table Juntei does not know. |

## Fields

| Field | Interpretation |
|---|---|
| `SourceFile` | The database the row was read from. |
| `RoleGUID`, `RoleName` | The role, and its name from the `SystemIdentity.mdb` beside the database (built-in names without one). |
| `TenantID` | The GUID of the client's tenant, when the role reports one. |
| `TotalAccesses` | Requests of the client in the database's year; the sum of `DailyAccesses`. |
| `InsertDate` | When the row was created: the client's first request of the year in this database (UTC). |
| `LastAccess` | The client's latest request (UTC). |
| `Address` | The client's IPv4 or IPv6 address. |
| `AuthenticatedUserName` | The account the client authenticated as (`DOMAIN\user`; computer accounts end with `$`). |
| `ClientName` | The client's name, when the role reports one. |
| `DailyAccesses` | The days with requests and their counts, from the `Day1` … `Day366` columns, dated in the year of `LastAccess`: `2023-01-12: 5, 2023-01-13: 2` in CSV, objects with `Date` and `Accesses` in JSON. A day whose year is unknown has `DayOfYear` instead of `Date`. |
| `FirstSeen`, `LastSeen` | First and last use of a role (`ROLE_ACCESS`), last sighting of a DNS pair (`DNS`), in UTC. |
| `CreationTime`, `LastSeenActive` | When a virtual machine or a `SYSTEM_IDENTITY` row was created, and when a virtual machine was last active, in UTC. |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server`. |
| `OS-CurrentTimeZone` | The server's offset from UTC in minutes (`-420` is UTC-07:00). |
| `OS-LastBootUpTime` | The server's last boot in UTC (the WMI text as stored when it can't be read). |
| Other columns | Every other column of the table under Juntei's PascalCase field labels, for example `SystemDNS-HostName`, `SystemDomainName`, `OS-BuildNumber` and `SystemSerialNumber`. |

## Automatic parsing

`auto-parse` writes the tables to `LateralMovement` as `UAL-Clients.csv`, `UAL-RoleAccess.csv` and
so on (one `UAL.json` or `UAL.jsonl` file with `-t json` or `-t jsonl`), with
`LateralMovement/UAL-logs.txt`. The combined `-j` timeline includes every row, with `Comp` and the
`Artifact` identifier `ual`. `-Z` converts `InsertDate`, `LastAccess`, `FirstSeen`, `LastSeen`,
`CreationTime`, `LastSeenActive` and `OS-LastBootUpTime`.

## Further Details

- [Microsoft — User Access Logging](https://learn.microsoft.com/en-us/windows-server/administration/user-access-logging/get-started-with-user-access-logging)
- [Plaso — User Access Logging parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/user_access_logging.py)
- [Velociraptor — Windows.Forensics.UserAccessLogs](https://docs.velociraptor.app/artifact_references/pages/windows.forensics.useraccesslogs/)
- [CrowdStrike — How to Leverage User Access Logging for Forensic Investigations](https://www.crowdstrike.com/en-us/blog/user-access-logging-ual-overview/)
