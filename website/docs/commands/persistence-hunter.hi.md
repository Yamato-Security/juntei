# Persistence Hunter { #persistence-hunter }

Persistence किसी प्रोग्राम को reboot, sign-in, scheduled trigger या किसी अन्य
event के बाद फिर से चलने देता है। हमलावर इस उद्देश्य के लिए सामान्य Windows configuration का उपयोग कर सकता है। कई वैध प्रोग्राम
ठीक यही तंत्र उपयोग करते हैं, इसलिए persistence entry मिलना विश्लेषण का शुरुआती बिंदु है।

`juntei persistence-hunter` [Hexastrike के PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper)
का native Rust port है, जिसमें 131 checks हैं:
130-check वाला PyrsistenceSniper catalogue और एक native Start Menu COM check।

हर तंत्र, उसके साक्ष्य स्थानों, और किसी finding की जाँच कैसे करें, इसकी व्याख्या के लिए
[131-check persistence मार्गदर्शिका](../persistence-hunter/index.md) देखें।
[Persistence आभार](../persistence-hunter/acknowledgements.md) catalogue के पीछे के Python और
PowerShell projects का वर्णन करते हैं।

## उपयोग { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## पहला scan { #first-scan }

```bash
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

साक्ष्य कोई mount किया गया Windows volume, Velociraptor संग्रह, या कई एकत्र किए गए computers वाली
directory हो सकती है। विश्लेषण के दौरान computers अलग रखे जाते हैं, और हर परिणाम में
`Comp` शामिल होता है। Windows paths resolve करते समय संग्रह के filename escaping को decode किया जाता है।

एक hive के लिए:

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

एक अकेला hive उन checks के लिए data नहीं दे सकता जिन्हें किसी अन्य hive या filesystem साक्ष्य की आवश्यकता है। खाली परिणाम को
सार्थक मानने से पहले coverage report पढ़ें।

Check IDs और उनके ATT&CK mappings सूचीबद्ध करें:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

`--list-checks` द्वारा print किए गए check IDs, या `T1547.001` जैसे ATT&CK ID का उपयोग करें।
`T1547` जैसा parent ID उसकी sub-techniques चुनता है। `--technique` `--check` का alias है।
हर check findings में अपनी technique, विवरण और reference links शामिल करता है।

## समानांतर scans { #parallel-scans }

Command एक साझा worker pool का उपयोग करता है, जो default रूप से logical CPUs की संख्या के बराबर होता है। Worker count चुनने के लिए
`-T, --threads <N>` set करें, या single-thread scan के लिए `-T 1`:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

साक्ष्य की खोज, स्वतंत्र checks, payload metadata, और MFT/EVTX timeline स्रोत समानांतर चलते हैं। किसी host के
load किए गए registry hives read-only साझा किए जाते हैं, और एक ही payload का cached metadata एक बार initialize होता है।
पूरे संग्रह में hive memory को कई गुना होने से बचाने के लिए hosts एक-एक करके load किए जाते हैं। Findings,
coverage, timestamp चयन और warning क्रम अलग-अलग thread counts पर समान रहते हैं।

`auto-parse` पहले से ही parser processes को समानांतर चलाता है, इसलिए यह हर `persistence-hunter` child को
एक worker देता है। उसका अपना `-T` option समवर्ती parser processes की संख्या नियंत्रित करता है।

## एकत्र करने योग्य साक्ष्य { #evidence-to-collect }

| स्रोत | Checks के उदाहरण |
|---|---|
| `SYSTEM` और पास के transaction logs | सेवाएँ, service recovery commands, LSA packages, boot settings, authentication providers |
| `SOFTWARE` और पास के transaction logs | Run keys, Winlogon, IFEO, COM registrations, TaskCache, Office settings |
| हर उपयोगकर्ता का `NTUSER.DAT` और `UsrClass.dat` | User autostarts, environment variables, COM overrides, Office settings |
| `SAM` और `SOFTWARE` | RID विसंगतियाँ, hidden accounts, local Administrators की असामान्य सदस्यता |
| `Windows/System32/Tasks` | Scheduled-task commands और COM handlers; TaskCache से तुलना |
| प्रति-उपयोगकर्ता Start Menu `.library-ms` फ़ाइलें और CLSID-नाम वाले फ़ोल्डर | COM activation-trigger candidates, मेल खाते user/machine class registrations से जोड़े गए |
| Startup folders, PowerShell profiles, Office templates, Terminal settings | User या system triggers द्वारा चलाई गई फ़ाइलें और commands |
| WMI `OBJECTS.DATA` | Event consumer instances और filter-to-consumer bindings |
| BITS `qmgr.db` या `qmgr*.dat` | Recover किए गए notification commands और owner SIDs |
| `$MFT` और प्रासंगिक EVTX फ़ाइलें | स्वतंत्र timestamp correlation |
| संदर्भित executables, DLLs और catalog फ़ाइलें | SHA256 hashes और वर्णनात्मक PE/certificate metadata |

उपलब्ध होने पर registry transaction logs replay किए जाते हैं। `--no-logs` hives को एकत्र किए गए रूप में पढ़ता है;
replay न किया गया dirty hive हाल के बदलाव छोड़ सकता है और coverage में report किया जाता है। Active-system checks के लिए
वर्तमान control set का उपयोग होता है। DSRM backdoor check मौजूद होने पर अतिरिक्त रूप से
`ControlSet001`, `ControlSet002` और `CurrentControlSet` की जाँच करता है; उस setting को सक्रिय बताने से पहले
उसके report किए गए path की जाँच `SYSTEM\\Select` से करें। अन्य ऐतिहासिक control sets
की जाँच अलग-अलग registry parser commands से की जा सकती है।

Hive reads प्रति-computer 1 GiB input budget साझा करते हैं। Machine hives का प्रयास user hives से पहले किया जाता है;
जो inputs या transaction logs समा नहीं पाते उनका नाम partial coverage में दिया जाता है। अन्य पढ़ने योग्य hives और
file-backed checks जारी रहते हैं। केवल canonical hive स्थान या स्पष्ट रूप से दिए गए exports ही
hive की पहचान स्थापित करते हैं; किसी उपयोगकर्ता की temporary directory के भीतर की copy उस उपयोगकर्ता के वास्तविक hive की जगह नहीं ले सकती।
SOFTWARE में दर्ज मूल system drive को captured system volume से map किया जाता है, इसलिए
मूल `C:` volume को किसी अलग evidence drive label के अंतर्गत mount करने से payload metadata नहीं छूटता।

`auto-parse` भी hunter चलाता है और हर computer directory के भीतर `persistence/PersistenceHunter.csv` (या `.json`/`.jsonl`)
सहेजता है, उसके पास `PersistenceHunter-logs.txt` के साथ। इसके findings वैकल्पिक
`-j <FILE>` SIEM timeline में भी जाते हैं। Automatic parsing पूर्ण configuration inventory, जिसमें allow rules से मेल खाने वाली
entries भी शामिल हैं, रखने के लिए `--min-severity info` का उपयोग करती है।

## COM registrations और activation triggers { #com-registrations-and-activation-triggers }

`scheduled_task_files` check task XML के `Actions Context` और मेल खाते
`Principal id` का उपयोग करके हर action का principal चुनता है। Group principal को group के रूप में ही रखा जाता है;
इससे यह स्थापित नहीं होता कि किस व्यक्तिगत उपयोगकर्ता के class registrations उपयोग होंगे।

COM-handler candidates को native और 32-bit class views में अलग-अलग resolve किया जाता है। केवल
मेल खाते उपयोगकर्ता के captured `UsrClass.dat`/`NTUSER.DAT` और machine `SOFTWARE` पर विचार किया जाता है।
SID principals को captured `ProfileList` paths के माध्यम से जोड़ा जाता है। Account नाम, जिनमें
`HOST\user` और `DOMAIN\user` शामिल हैं, साधारण `user` profile की तुलना में मेल खाते `user.DOMAIN` profile को
प्राथमिकता देते हैं। Local host aliases और `.\user` suffix matching के लिए captured host नामों का उपयोग करते हैं;
सटीक suffix अन्य host aliases पर प्राथमिकता लेता है। साधारण profile पर विचार
केवल तब किया जाता है जब कोई qualified profile मेल नहीं खाता। हर ऐसे मिलान को `Description` में स्पष्ट रूप से
**folder-name heuristic** label किया जाता है; यह account ownership या domain सदस्यता सिद्ध नहीं करता। अस्पष्ट मिलान
अनसुलझे रहते हैं और finding विवरण तथा coverage दोनों में report किए जाते हैं; qualified profiles के बीच की
अस्पष्टता साधारण profile पर fallback को trigger नहीं करती। यह
`Volatile Environment` पर निर्भर नहीं करता, जो सामान्यतः केवल memory में होता है और registry hives के साथ सहेजा नहीं जाता। Duplicate user CLSID
machine CLSID के children को ढक (shadow) देता है। Group principals machine COM candidates का उपयोग करते हैं और
यह सीमा finding विवरण में रखते हैं; सामान्य group principal अपने-आप में
coverage को partial नहीं बनाता या error-summary entries का उपभोग नहीं करता।

Resolver `InprocServer32` और `LocalServer32` का अनुसरण करता है, managed `CodeBase` paths का उपयोग करता है
(version subkeys सहित), और AppID `LocalService` को एकत्र किए गए SYSTEM service
`ImagePath` से जोड़ता है। AppID executable-name associations रखे जाते हैं, लेकिन केवल basename से
किसी executable का पूरा path स्थापित नहीं होता। Local `file:` CodeBase URLs decode किए जाते हैं; remote URLs को
access किए बिना दर्ज किया जाता है। Missing registrations CLSID को
`ResolutionStatus: unresolved_com_class`, खाली path/hash fields और अज्ञात `Exists` के साथ रखते हैं।

`start_menu_com` check प्रति-उपयोगकर्ता Start Menu `.library-ms` XML URLs पढ़ता है और
CLSID-नाम वाले फ़ोल्डर एकत्र करता है, खाली directories सहित। यह trigger path, CLSID, candidate
server और registry provenance report करता है। केवल-फ़ाइल acquisition खाली directory सुरक्षित नहीं रख सकता,
और missing user hives प्रति-उपयोगकर्ता registration lookup को पूरा होने से रोकते हैं। Symbolic links का
अनुसरण नहीं किया जाता। ये configuration candidates हैं: architecture, elevation, UAC, policy और
activation context यह प्रभावित कर सकते हैं कि कोई registration वास्तव में उपयोग होता है या नहीं। निष्पादन की पुष्टि
process, module-load या अन्य स्वतंत्र साक्ष्य से करें।

## वर्गीकरण और profiles { #classification-and-profiles }

Default न्यूनतम वर्गीकरण `medium` है। पूर्ण inventory की जाँच करने के लिए, जिसमें
bundled allowlist से मेल खाने वाली entries भी शामिल हैं:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "अधूरे संग्रहों की समीक्षा करते समय सभी severities का उपयोग करें"
    कोई व्यापक allow rule किसी हमलावर के command से आंशिक रूप से मेल खा सकता है, तब भी जब उसका signer missing
    या अलग हो। Upstream आंशिक मिलानों को `LOW` के रूप में वर्गीकृत करता है, जिसे manual command का default
    `medium` threshold छिपा देता है। यह Windows directories के अंतर्गत दुर्भावनापूर्ण scheduled tasks को प्रभावित कर सकता है।
    पूर्ण जाँच के लिए `--min-severity info` का उपयोग करें और `PolicyReason`, `SignerStatus`,
    command और स्रोत साक्ष्य की एक साथ समीक्षा करें। `auto-parse` पहले से ही इन entries को रखता है।

| वर्गीकरण | अर्थ |
|---|---|
| `HIGH` | कोई configure किया गया block rule पूरी तरह मेल खाया |
| `MEDIUM` | कोई allow या block rule मेल नहीं खाया |
| `LOW` | किसी allow rule की hard conditions मेल खाईं, लेकिन signer metadata मेल नहीं खाया या उपलब्ध नहीं था |
| `INFO` | कोई allow rule पूरी तरह मेल खाया |

ये policy वर्गीकरण हैं, compromise की मापी गई संभावनाएँ नहीं। Default `MEDIUM`
entry malware का प्रमाण नहीं है। `INFO` यह स्थापित नहीं करता कि किसी फ़ाइल का signature मान्य है या कि
किसी हमलावर ने उसका दुरुपयोग नहीं किया। आंशिक allow मिलान default threshold से नीचे है, इसलिए विरल संग्रहों की समीक्षा
या allowlist को tune करते समय `--min-severity low` या `info` शामिल करें।

`--profile profile.yaml` से YAML profile दें। यह bundled profile को **बदल देता है**। चुनी गई फ़ाइल के भीतर के global
और per-check rules संयुक्त होते हैं; कोई भी पूरी तरह मेल खाने वाला block rule प्रबल होता है।
उदाहरण के लिए:

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

समर्थित rule conditions `path_matches`, `value_matches`, `hash` (64 hexadecimal SHA256
अंक), `not_lolbin` और `signer` (case-insensitive substring) हैं। सभी बताई गई hard conditions का
मेल खाना आवश्यक है। Signer metadata ऊपर वर्णित soft condition है। Regexes Rust के regular-expression
syntax का उपयोग करते हैं: lookarounds और backreferences समर्थित नहीं हैं। अज्ञात check IDs, अमान्य expressions
और अज्ञात profile fields चुपचाप policy को कमज़ोर करने के बजाय error उत्पन्न करते हैं।

## आउटपुट fields { #output-fields }

CSV default file format है। पठनीय JSON objects के लिए `-t json` या compact,
प्रति-line-एक-object आउटपुट के लिए `-t jsonl` का उपयोग करें। `-o` के बिना, स्पष्ट `-t` stdout पर केवल data लिखता है; coverage
warnings stderr पर जाती हैं। दोनों में से किसी option के बिना, Juntei एक summary और अधिकतम 30 candidates print करता है।

| Fields | इनकी व्याख्या कैसे करें |
|---|---|
| `Comp`, `CheckID`, `Technique`, `MitreID` | Row किस computer और persistence तंत्र का वर्णन करती है |
| `Path`, `Value` | Registry स्थान, फ़ाइल, task या recover किया गया record और उसकी configure की गई सामग्री |
| `Description`, `References` | तंत्र क्यों महत्वपूर्ण है और अधिक कहाँ पढ़ें |
| `AccessGained` | तंत्र का सामान्य दायरा (`USER` या `SYSTEM`; BITS `LOCAL_SERVICE`/`NETWORK_SERVICE` की भी पहचान कर सकता है); यह प्रमाण नहीं कि किसी विशिष्ट command ने वह privilege प्राप्त किया |
| `Severity`, `PolicyReason` | Profile वर्गीकरण, मेल खाने वाली policy की व्याख्या, और कोई भी partial-match या unverified-signer योग्यता |
| `SourceFile`, `RecordStatus` | सटीक एकत्र किया गया स्रोत, दिए गए input से शुरू होने वाला उसका path (auto-parse से `./HOST/...`), और allocation/replay/recovery context; मूल साक्ष्य तक वापस पहुँचने के लिए इसका अनुसरण करें |
| `KeyLastWrite` | Registry **key** का timestamp। यह किसी व्यक्तिगत value का समय नहीं बताता |
| `LastChange`, `ChangeSource`, `ChangeEvidence` | स्वतंत्र रूप से correlate किया गया बदलाव समय, स्रोत प्रमाण, और क्या correlation सफल हुआ |
| `ResolvedPath`, `ResolutionStatus`, `Exists` | Payload path और offline resolution परिणाम; एकत्र न किया गया साक्ष्य अज्ञात है, `false` नहीं |
| `SHA256` | पढ़ने योग्य होने पर, एकत्र किए गए payload bytes का SHA256 |
| `Signer`, `SignerStatus` | वर्णनात्मक embedded certificate/catalog metadata और उसकी उपलब्धता; certificate trust और signature की वैधता सत्यापित नहीं की जाती |
| `IsLolbin`, `Launcher` | क्या configure किया गया launcher या payload bundled LOLBAS list में है, और proxied payload के लिए उपयोग किया गया launcher |
| `IsBuiltin`, `IsInOS-Directory` | सामान्य Windows commands और system directories के लिए context; कोई भी field वैधता स्थापित नहीं करता |

`rundll32.exe` command किसी DLL payload का नाम ले सकता है। Juntei payload को resolve और
hash करते समय launcher को सुरक्षित रखता है। एकत्र किए गए computer के बाहर का reference, missing drive, अस्पष्ट
case-insensitive मिलान, या अनसुलझा environment variable विश्लेषक की machine की किसी फ़ाइल से
प्रतिस्थापित नहीं किया जाता।

## Timestamps और सीमाएँ { #timestamps-and-limitations }

Hunter विश्लेषक VM की copy की गई फ़ाइलों के modification times को attack timestamps के रूप में उपयोग नहीं करता।
जहाँ उपलब्ध हो, यह registry writes को Sysmon event 13 से, task installation को
Security 4698 या Task Scheduler 106 से, service installation को System 7045 से, और file-backed
configuration को मूल `$MFT` timestamps से correlate करता है। Provenance `ChangeSource` में दिखता है।
कोई event configuration बदलाव का समय बता सकता है, बिना यह सिद्ध किए कि configure किया गया payload चला।
Local System event 6011 किसी पहले के computer नाम को सिद्ध कर सकता है। Hunter एकत्र किए गए computer से anchored सत्यापित rename
chains का अनुसरण करता है, उसका वर्तमान `Comp` value सुरक्षित रखते हुए मूल event computer और rename-record प्रमाण को
`ChangeSource` में दर्ज करता है। असिद्ध विदेशी नाम,
archived logs, forwarded logs और कोई अन्य Windows installation इस नियम के माध्यम से स्वीकार नहीं किए जाते।

Captured ComputerName/ActiveComputerName, TCP/IP Hostname/NV Hostname, और client-info hostname/FQDN
host aliases प्रदान करते हैं, जो DNS/NetBIOS अंतरों और pending renames को कवर करते हैं। Captured पहचान के बिना,
primary logs में एक अकेला नाम स्पष्ट unverified-identity provenance
और coverage warning के साथ समय-निर्धारण का समर्थन कर सकता है। कई असत्यापित नाम host से नहीं जोड़े जाते। अस्वीकृत Computer
नाम correlation coverage सीमा के रूप में report किए जाते हैं; उनके channels फिर भी एकत्रित के रूप में चिह्नित रहते हैं।

किसी एक computer के लिए बाहरी रूप से एकत्र किया गया MFT चुनने के लिए `--mft <FILE>` का उपयोग करें। Timestamp correlation छोड़ने के लिए
`--no-timeline` का उपयोग करें। `NOT_RUN`, `NOT_APPLICABLE`, `NO_ARTIFACT`, `NO_MATCH` और `REJECTED`
खाली `LastChange` की व्याख्या करते हैं; खाली field इस बात का साक्ष्य नहीं है कि entry हानिरहित है।

WMI और BITS recovery पूर्ण database transaction parser के बजाय सीमित carving है।
Recover किए गए strings या bindings में पुराना/हटाया गया data हो सकता है; इन records की पुष्टि आवश्यक है।
केवल WMI class schemas को subscriptions के रूप में नहीं गिना जाता। एकत्र की गई task XML फ़ाइल के बिना TaskCache entry
समीक्षा योग्य विसंगति है; हो सकता है triage संग्रह ने बस वह फ़ाइल छोड़ दी हो।
Catalog और PE metadata असत्यापित हैं और संग्रह से अनुपस्थित हो सकते हैं। Accessibility-file
replacement checks उपयुक्त reference signature metadata पर निर्भर करते हैं; missing catalog साक्ष्य के कारण
हर accessibility executable flag नहीं होना चाहिए।

हर check के लिए coverage दर्ज किया जाता है, शून्य findings वाले checks सहित। `complete` का अर्थ है कि
लागू एकत्र किया गया स्रोत पढ़ा गया; यह सिद्ध नहीं करता कि संग्रह पूर्ण है या कि
computer में कोई अन्य persistence नहीं है। `partial`, `not_collected`, `failed` और `disabled`
सीमाओं की व्याख्या करते हैं। अपने findings के साथ `--coverage Coverage.csv` और parser log रखें।

यह port Juntei के CSV/JSON/JSONL formats का उपयोग करता है। यह upstream Python प्रोग्राम को call नहीं करता, online
enrichment services से query नहीं करता, या upstream HTML/XLSX UI को पुन: प्रस्तुत नहीं करता।

## अधिक जानकारी { #further-details }

- [PyrsistenceSniper implementation और profiles](https://github.com/hexastrike/PyrsistenceSniper)
- [MITRE ATT&CK persistence techniques](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft: Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [रेजिस्ट्री autostart inventory](../artifacts/persistence.md)
- [सेवाएँ](../artifacts/services.md) और [TaskCache](../artifacts/taskcache.md)
