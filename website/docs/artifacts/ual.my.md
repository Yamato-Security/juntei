# အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း (UAL) { #user-access-logging-ual }

## UAL ဆိုသည်မှာ { #what-ual-is }

အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း (User Access Logging, UAL) သည် Windows Server 2012 မှစ၍ မူလအတိုင်း ထည့်သွင်းပြီး
ဖွင့်ထားသော Windows Server feature တစ်ခု ဖြစ်ပြီး client များက server ၏ role များထံ ပြုလုပ်သော request များကို ESE
database များတွင် စုစည်းထားပါသည်။ Role၊ client address နှင့် account တစ်ခုစီအတွက် ၎င်းသည် နှစ်အတွင်း client ကို ပထမဆုံးနှင့်
နောက်ဆုံး တွေ့ခဲ့သည့်အချိန်၊ request စုစုပေါင်း အရေအတွက်နှင့် နှစ်၏ နေ့တစ်နေ့ချင်းစီအတွက် အရေအတွက်ကို သိမ်းဆည်းပါသည်။ Role များတွင်
File Server (SMB)၊ Active Directory Domain Services၊ Active Directory Certificate Services၊ DHCP
Server၊ Print and Document Services၊ Remote Access၊ Web Server (IIS) နှင့် Windows Server Update
Services တို့ ပါဝင်ပါသည်။

UAL သည် ကွန်ရက်အတွင်း ဘေးတိုက်ရွေ့လျားမှု (lateral movement) စုံစမ်းစစ်ဆေးမှုများအတွက် တန်ဖိုးရှိပါသည်။ ၎င်း၏ client row များသည်
မည်သည့် address များနှင့် account များက server တစ်ခုကို အသုံးပြုခဲ့သည်၊ မည်သည့်နေ့များတွင် အသုံးပြုခဲ့သည်ကို ပြသပြီး event log များ
လှည့်ပတ်ထပ်ရေးခံရပြီးနောက် ကြာမြင့်စွာ ကျော်လွန်၍ပင် ပြသနိုင်လေ့ ရှိပါသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> မှတ်တမ်းတင်ထားသော address ရှိ client တစ်ခုသည်
မှတ်တမ်းတင်ထားသော account အဖြစ် authenticate လုပ်၍ အရေအတွက် ရှိသော နေ့တိုင်းတွင် role ကို အသုံးပြုခဲ့ပါသည်။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` တွင် နောက်ဆုံး လှုပ်ရှားမှုများ ရှိပါသည်။ `SystemIdentity.mdb` သည် server ကို ဖော်ပြပြီး role များကို
အမည်ပေးကာ နှစ်တစ်နှစ်စီကို သိမ်းထားသော `{GUID}.mdb` ဖိုင်ကို စာရင်းပြုစုထားပါသည် (`CHAINED_DATABASES`)။ UAL service သည်
database များကို ဖွင့်ထားဆဲ ဖြစ်သောကြောင့် `Sum` ဖိုင်တွဲတစ်ခုလုံးကို forensic collector ဖြင့် သို့မဟုတ် volume snapshot မှ
စုဆောင်းပါ။

!!! warning "Dirty database များ"
    အလုပ်လုပ်နေသော server မှ စုဆောင်းသည့်အခါ `Current.mdb` သည် dirty ဖြစ်နေလေ့ ရှိပါသည်။ Juntei သည် ၎င်းကို စုဆောင်းထားသည့်အတိုင်း
    ဖတ်ပြီး ၎င်းကို အစီရင်ခံသော်လည်း `Svc*.log` ကို replay မလုပ်ပါ။ Transaction log များတွင်သာ ရှိသော လှုပ်ရှားမှုများ
    ပျောက်နေပါသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- UAL သည် request များကို နေ့အလိုက် ရေတွက်ပြီး request တစ်ခုချင်းစီကို log မလုပ်ပါ။ Client row တစ်ခု၏ တစ်ခုတည်းသော အချိန်များမှာ
  `InsertDate` နှင့် `LastAccess` ဖြစ်ပါသည်။
- `DailyAccesses` ၏ နေ့များသည် UTC နေ့များ ဖြစ်ပါသည်။
- `Address` သည် server မြင်ခဲ့သော address ဖြစ်ပါသည်။ NAT gateway၊ proxy သို့မဟုတ် VPN သည် server နှင့်
  တကယ့် client ကြားတွင် ရှိနေနိုင်ပါသည်။ `::1` နှင့် `127.0.0.1` သည် server ကိုယ်တိုင် ဖြစ်ပါသည်။
- `Sum` ဖိုင်တွဲရှိ database တိုင်းကို ခွဲခြမ်းစိတ်ဖြာပါ။ တူညီသော client တွင် `Current.mdb` ၌ရော
  ၎င်း၏ နှစ်အတွက် `{GUID}.mdb` ၌ပါ row များ ရှိနိုင်ပါသည်။
- Row များ မရှိခြင်းသည် မည်သူမျှ မချိတ်ဆက်ခဲ့ကြောင်း သက်သေမပြပါ။ UAL ကို ပိတ်ထားနိုင်ပြီး (`Disable-Ual` ဖြင့် သို့မဟုတ်
  User Access Logging Service ကို ရပ်တန့်ခြင်းဖြင့်) dirty database တွင် ၎င်း၏ နောက်ဆုံး ပြောင်းလဲမှုများ ပျောက်နေပါသည်။
- `SystemIdentity.mdb` မရှိလျှင် `RoleName` သည် Windows Server က ၎င်း၏ role များကို ပေးထားသော အမည်များမှ လာပါသည်။
- `LastAccess` သည် သုည သို့မဟုတ် မမှန်ကန်ပါက နှစ်ကို သတ်မှတ်ရန် မသုံးပါ။ Juntei သည် မှန်ကန်သော `InsertDate`၊ ထို့နောက် နှစ်အလိုက် database ၏ metadata ကို သုံးပြီး နှစ်ကို မသိနိုင်ပါက `DayOfYear` ကို ထိန်းသိမ်းထားသည်။
- အနှစ်ချုပ် စုစုပေါင်းသည် source row အားလုံးကို ပေါင်းထားခြင်းဖြစ်သည်။ `Current.mdb`၊ နှစ်အလိုက် database များနှင့် snapshot များတွင် အချက်အလက် ထပ်နေနိုင်သဖြင့် ထပ်နေမှုဖယ်ရှားထားသော connection သို့မဟုတ် logon အရေအတွက်ဟု မယူဆသင့်ပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

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

## Output table များ { #output-tables }

CSV သည် table တစ်ခုလျှင် ဖိုင်တစ်ခု ဖန်တီးပြီး table ဗလာဖြစ်နေလျှင် ၎င်း၏ header သာ ပါဝင်ပါသည်။ JSON နှင့် JSONL row များတွင်
`SourceFile` (database) နှင့် `Table` (database ထဲရှိ table ၏ အမည်) ပါဝင်ပါသည်။

| CSV ဖိုင် | Table | အကြောင်းအရာ |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | Role တစ်ခုစီ၏ client များ: address၊ account၊ နှစ်အတွင်း ပထမဆုံးနှင့် နောက်ဆုံး ဝင်ရောက်မှု၊ စုစုပေါင်းနှင့် နေ့စဉ် request အရေအတွက်များ။ |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | Role တစ်ခုစီကို ပထမဆုံးနှင့် နောက်ဆုံး အသုံးပြုခဲ့သည့်အချိန်။ |
| `UAL-DNS.csv` | `DNS` | Address နှင့် host name အတွဲများ၊ တစ်ခုစီကို နောက်ဆုံး တွေ့ခဲ့သည့်အချိန်နှင့်အတူ။ |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Hyper-V virtual machine များ: identifier များ၊ serial number၊ ဖန်တီးချိန်နှင့် နောက်ဆုံး active ဖြစ်ခဲ့ချိန်။ |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | Server: အမည်၊ domain၊ Windows ဗားရှင်း၊ product type၊ hardware၊ time zone နှင့် နောက်ဆုံး boot။ Row အသစ်တစ်ခုသည် အမည်ပြောင်းခြင်း၊ domain သို့ join ခြင်း သို့မဟုတ် domain controller အဖြစ် promote လုပ်ခြင်းကို ပြသနိုင်ပါသည်။ |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | နှစ်တစ်နှစ်စီ၏ `{GUID}.mdb` ဖိုင်။ |
| `UAL-Roles.csv` | `ROLE_IDS` | Role GUID များနှင့် ၎င်းတို့၏ အမည်များ။ |
| `UAL-<table>.csv` | အခြား table | Juntei မသိသော table တစ်ခု၏ column တိုင်း။ |

## Field များ { #fields }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `SourceFile` | Row ကို ဖတ်ယူခဲ့သော database။ |
| `RoleGUID`, `RoleName` | Role နှင့် database ဘေးရှိ `SystemIdentity.mdb` မှ ၎င်း၏ အမည် (ထိုဖိုင် မရှိလျှင် built-in အမည်များ)။ |
| `TenantID` | Role က အစီရင်ခံလျှင် client ၏ tenant GUID။ |
| `TotalAccesses` | Database ၏ နှစ်အတွင်း client ၏ request များ။ `DailyAccesses` ၏ ပေါင်းလဒ်။ |
| `InsertDate` | Row ကို ဖန်တီးခဲ့သည့်အချိန်: ဤ database ရှိ နှစ်အတွင်း client ၏ ပထမဆုံး request (UTC)။ |
| `LastAccess` | Client ၏ နောက်ဆုံး request (UTC)။ |
| `Address` | Client ၏ IPv4 သို့မဟုတ် IPv6 address။ |
| `AuthenticatedUserName` | Client က authenticate လုပ်ခဲ့သော account (`DOMAIN\user`၊ computer account များသည် `$` ဖြင့် ဆုံးပါသည်)။ |
| `ClientName` | Role က အစီရင်ခံလျှင် client ၏ အမည်။ |
| `DailyAccesses` | Request ရှိသော နေ့များနှင့် ၎င်းတို့၏ အရေအတွက်များ၊ `Day1` … `Day366` column များမှ ယူပြီး `LastAccess` ၏ နှစ်ဖြင့် ရက်စွဲတပ်ထားပါသည်: CSV တွင် `2023-01-12: 5, 2023-01-13: 2`၊ JSON တွင် `Date` နှင့် `Accesses` ပါသော object များ။ နှစ်ကို မသိသော နေ့တွင် `Date` အစား `DayOfYear` ပါရှိပါသည်။ |
| `FirstSeen`, `LastSeen` | Role တစ်ခု၏ ပထမဆုံးနှင့် နောက်ဆုံး အသုံးပြုမှု (`ROLE_ACCESS`)၊ DNS အတွဲတစ်ခုကို နောက်ဆုံး တွေ့ခဲ့ချိန် (`DNS`)၊ UTC ဖြင့်။ |
| `CreationTime`, `LastSeenActive` | Virtual machine တစ်ခု သို့မဟုတ် `SYSTEM_IDENTITY` row တစ်ခုကို ဖန်တီးခဲ့သည့်အချိန်နှင့် virtual machine တစ်ခု နောက်ဆုံး active ဖြစ်ခဲ့သည့်အချိန်၊ UTC ဖြင့်။ |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server`။ |
| `OS-CurrentTimeZone` | UTC မှ server ၏ offset ကို မိနစ်ဖြင့် (`-420` သည် UTC-07:00 ဖြစ်သည်)။ |
| `OS-LastBootUpTime` | Server ၏ နောက်ဆုံး boot ကို UTC ဖြင့် (ဖတ်မရလျှင် သိမ်းဆည်းထားသည့်အတိုင်း WMI စာသား)။ |
| အခြား column များ | Table ၏ အခြား column တိုင်းကို Juntei ၏ PascalCase field label များဖြင့်၊ ဥပမာ `SystemDNS-HostName`၊ `SystemDomainName`၊ `OS-BuildNumber` နှင့် `SystemSerialNumber`။ |

## အလိုအလျောက် ခွဲခြမ်းစိတ်ဖြာခြင်း { #automatic-parsing }

`auto-parse` သည် table များကို `LateralMovement` ထဲသို့ `UAL-Clients.csv`၊ `UAL-RoleAccess.csv` စသည်ဖြင့်
(`-t json` သို့မဟုတ် `-t jsonl` ဖြင့်ဆိုလျှင် `UAL.json` သို့မဟုတ် `UAL.jsonl` ဖိုင်တစ်ခုတည်း)
`LateralMovement/UAL-logs.txt` နှင့်အတူ ရေးသားပါသည်။ ပေါင်းစပ်ထားသော `-j` timeline တွင် row တိုင်းကို `Comp` နှင့်
`Artifact` identifier `ual` တို့နှင့်အတူ ထည့်သွင်းပါသည်။ `-Z` သည် `InsertDate`၊ `LastAccess`၊ `FirstSeen`၊ `LastSeen`၊
`CreationTime`၊ `LastSeenActive` နှင့် `OS-LastBootUpTime` တို့ကို ပြောင်းလဲပေးပါသည်။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft — User Access Logging](https://learn.microsoft.com/en-us/windows-server/administration/user-access-logging/get-started-with-user-access-logging)
- [Plaso — User Access Logging parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/user_access_logging.py)
- [Velociraptor — Windows.Forensics.UserAccessLogs](https://docs.velociraptor.app/artifact_references/pages/windows.forensics.useraccesslogs/)
- [CrowdStrike — How to Leverage User Access Logging for Forensic Investigations](https://www.crowdstrike.com/en-us/blog/user-access-logging-ual-overview/)
