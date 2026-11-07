# उपयोगकर्ता एक्सेस लॉगिंग (UAL) { #user-access-logging-ual }

## UAL क्या है { #what-ual-is }

User Access Logging (UAL) एक Windows Server feature है, जो Windows Server 2012 से default रूप से installed और
enabled रहता है। यह server के roles को clients द्वारा भेजे गए requests को ESE databases में aggregate करता है। हर
role, client address और account के लिए, यह रखता है कि client वर्ष में पहली और आखिरी बार कब देखा गया, requests
की कुल संख्या, और वर्ष के हर दिन की एक गिनती। Roles में File Server (SMB), Active Directory Domain Services,
Active Directory Certificate Services, DHCP Server, Print and Document Services, Remote Access, Web Server (IIS)
और Windows Server Update Services शामिल हैं।

Lateral movement की जाँच में UAL मूल्यवान है: इसकी client rows दिखाती हैं कि किन addresses और accounts ने किसी
server का उपयोग किया, और किन दिनों में, अक्सर event logs के roll over हो जाने के काफ़ी समय बाद भी।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> दर्ज address पर मौजूद, दर्ज account के रूप में
authenticated किसी client ने हर उस दिन role का उपयोग किया जिसकी गिनती मौजूद है।

## स्थान और संग्रह { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` सबसे हाल की गतिविधि रखता है। `SystemIdentity.mdb` server का वर्णन करता है, roles के नाम बताता है,
और वह `{GUID}.mdb` file बताता है जिसमें हर वर्ष रखा जाता है (`CHAINED_DATABASES`)। UAL service databases को खुला
रखती है: पूरी `Sum` directory को forensic collector से या volume snapshot से संग्रहित करें।

!!! warning "Dirty databases"
    चल रहे server से संग्रहित करने पर `Current.mdb` अक्सर dirty होता है। Juntei इसे जैसा संग्रहित किया गया वैसा
    पढ़ता है और इसकी रिपोर्ट करता है, लेकिन `Svc*.log` को replay नहीं करता: केवल transaction logs में मौजूद
    गतिविधि अनुपस्थित रहती है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- UAL प्रति दिन requests गिनता है; यह हर request को अलग से log नहीं करता। किसी client row के केवल दो समय
  `InsertDate` और `LastAccess` हैं।
- `DailyAccesses` के दिन UTC दिन हैं।
- `Address` वह address है जो server ने देखा: server और असली client के बीच कोई NAT gateway, proxy या VPN हो सकता है।
  `::1` और `127.0.0.1` स्वयं server हैं।
- `Sum` directory के हर database को parse करें: एक ही client की rows `Current.mdb` में भी और उसके वर्ष के
  `{GUID}.mdb` में भी हो सकती हैं।
- Rows का न होना यह सिद्ध नहीं करता कि किसी ने connect नहीं किया: UAL बंद किया जा सकता है (`Disable-Ual` से, या
  User Access Logging Service को रोककर), और dirty database में उसके सबसे नए बदलाव नहीं होते।
- `SystemIdentity.mdb` के बिना, `RoleName` उन नामों से आता है जो Windows Server अपने roles को देता है।

## Juntei से parse करें { #parse-with-juntei }

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

## आउटपुट tables { #output-tables }

CSV हर table के लिए एक file बनाता है; table खाली होने पर उस file में केवल header होता है। JSON और JSONL rows में
`SourceFile` (database) और `Table` (database में table का नाम) होते हैं।

| CSV file | Table | सामग्री |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | हर role के clients: address, account, वर्ष का पहला और आखिरी access, requests की कुल और दैनिक गिनती। |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | हर role का पहली और आखिरी बार उपयोग कब हुआ। |
| `UAL-DNS.csv` | `DNS` | Address और host name के जोड़े, साथ में यह कि हर जोड़ा आखिरी बार कब देखा गया। |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Hyper-V virtual machines: identifiers, serial number, creation और आखिरी सक्रियता के समय। |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | Server: नाम, domain, Windows संस्करण, product type, hardware, time zone और आखिरी boot; एक नई row rename, domain join या domain controller में promotion दिखा सकती है। |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | हर वर्ष की `{GUID}.mdb` file। |
| `UAL-Roles.csv` | `ROLE_IDS` | Role GUIDs और उनके नाम। |
| `UAL-<table>.csv` | कोई अन्य table | ऐसी table का हर column जिसे Juntei नहीं जानता। |

## Fields { #fields }

| Field | व्याख्या |
|---|---|
| `SourceFile` | वह database जिससे row पढ़ी गई। |
| `RoleGUID`, `RoleName` | Role, और database के बगल वाले `SystemIdentity.mdb` से उसका नाम (उसके न होने पर built-in नाम)। |
| `TenantID` | Client के tenant का GUID, जब role इसे बताता है। |
| `TotalAccesses` | Database के वर्ष में client के requests; `DailyAccesses` का योग। |
| `InsertDate` | Row कब बनी: इस database में वर्ष का client का पहला request (UTC)। |
| `LastAccess` | Client का सबसे हाल का request (UTC)। |
| `Address` | Client का IPv4 या IPv6 address। |
| `AuthenticatedUserName` | वह account जिसके रूप में client authenticated हुआ (`DOMAIN\user`; computer accounts के अंत में `$` होता है)। |
| `ClientName` | Client का नाम, जब role इसे बताता है। |
| `DailyAccesses` | Requests वाले दिन और उनकी गिनती, `Day1` … `Day366` columns से, `LastAccess` के वर्ष की तारीखों के साथ: CSV में `2023-01-12: 5, 2023-01-13: 2`, JSON में `Date` और `Accesses` वाले objects। जिस दिन का वर्ष अज्ञात हो, उसमें `Date` के बजाय `DayOfYear` होता है। |
| `FirstSeen`, `LastSeen` | किसी role का पहला और आखिरी उपयोग (`ROLE_ACCESS`), किसी DNS जोड़े को आखिरी बार देखे जाने का समय (`DNS`), UTC में। |
| `CreationTime`, `LastSeenActive` | कोई virtual machine या `SYSTEM_IDENTITY` row कब बनी, और कोई virtual machine आखिरी बार कब सक्रिय थी, UTC में। |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server`। |
| `OS-CurrentTimeZone` | UTC से server का offset, minutes में (`-420` का अर्थ UTC-07:00 है)। |
| `OS-LastBootUpTime` | Server का आखिरी boot, UTC में (जब इसे पढ़ा न जा सके, तो WMI text जैसा stored है वैसा)। |
| अन्य columns | Table का हर अन्य column, Juntei के PascalCase field labels के साथ, उदाहरण के लिए `SystemDNS-HostName`, `SystemDomainName`, `OS-BuildNumber` और `SystemSerialNumber`। |

## स्वचालित विश्लेषण { #automatic-parsing }

`auto-parse` tables को `LateralMovement` में `UAL-Clients.csv`, `UAL-RoleAccess.csv` आदि नामों से लिखता है
(`-t json` या `-t jsonl` के साथ एक ही `UAL.json` या `UAL.jsonl` file), साथ में `LateralMovement/UAL-logs.txt`
भी। संयुक्त `-j` timeline में हर row शामिल होती है, `Comp` और `Artifact` identifier `ual` के साथ। `-Z`
`InsertDate`, `LastAccess`, `FirstSeen`, `LastSeen`, `CreationTime`, `LastSeenActive` और `OS-LastBootUpTime` को
convert करता है।
