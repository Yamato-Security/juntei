# Persistence Hunter { #persistence-hunter }

Persistence code को फिर से चलवाने का एक तरीका है: जब Windows शुरू होता है, कोई उपयोगकर्ता sign in करता है, कोई एप्लिकेशन
खुलता है, कोई निर्धारित समय आता है, या कोई अन्य event होता है। हमलावर उन्हीं settings का दुरुपयोग कर सकते हैं
जिनका उपयोग वैध software updates, device support, सुरक्षा और administration के लिए करता है।

`juntei persistence-hunter` एकत्र किए गए Windows साक्ष्य की offline जाँच **131 checks** से करता है, जो
इन तंत्रों और संबंधित account/protection विसंगतियों को कवर करते हैं। यह registry hives, task फ़ाइलें,
scripts, एप्लिकेशन settings और चुनिंदा database records पढ़ता है।

यह पृष्ठ हर implement किए गए check की व्याख्या करता है। पूरे options, output fields, profiles और
timestamp handling के लिए [command मार्गदर्शिका](../commands/persistence-hunter.md) खोलें।
[आभार](acknowledgements.md) पृष्ठ Python PyrsistenceSniper project और मूल
PowerShell PersistenceSniper tool के links देता है।

## जाँच शुरू करें { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

पूरी candidate सूची के लिए `--min-severity info` का उपयोग करें। Manual command का डिफ़ॉल्ट
`medium` है; इसका profile आंशिक allow matches को `LOW` के रूप में वर्गीकृत कर सकता है, भले ही लापता payload या
signer साक्ष्य पूरी तुलना को रोकता हो। न तो कोई allow match और न ही कोई अपरिचित entry यह तय करती है
कि कोई चीज़ दुर्भावनापूर्ण है या नहीं। `auto-parse` पहले से ही सभी severities बनाए रखता है और findings को
हर कंप्यूटर की `persistence/` श्रेणी के अंतर्गत लिखता है।

किसी एक तंत्र पर ध्यान केंद्रित करने के लिए नीचे से उसका **Check ID** कॉपी करें:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

Command डिफ़ॉल्ट रूप से एक साझा worker pool का उपयोग करता है। चार workers चुनने के लिए `-T 4` जोड़ें, या
single-thread scan के लिए `-T 1`। Input में कई कंप्यूटर हो सकते हैं; उनका साक्ष्य अलग-अलग रहता है।

## Findings को साक्ष्य के रूप में पढ़ें { #read-findings-as-evidence }

| प्रश्न | क्या जाँचें |
|---|---|
| क्या configure किया गया था, और कहाँ? | `CheckId`, `Path`, और arguments सहित पूरा `Value`। एकत्र किए गए hive, फ़ाइल या database तक `SourceFile` का अनुसरण करें। |
| क्या target collection में मौजूद है? | `ResolvedPath`, `ResolutionStatus` और `Exists`। एकत्र न किए गए, अस्पष्ट या असमर्थित references अज्ञात हैं; Juntei विश्लेषक की मशीन से कोई फ़ाइल प्रतिस्थापित नहीं करता। |
| क्या configuration प्रभावी है? | वास्तविक trigger, enable flags, चयनित control set, Windows/एप्लिकेशन version, उपयोगकर्ता और architecture। कुछ checks जानबूझकर निष्क्रिय configuration भी बनाए रखते हैं। |
| क्या यह हमला है? | `Severity` और `PolicyReason` profile matching की व्याख्या करते हैं, निर्णय नहीं देते। अपेक्षित software, पूरे command, file hashes, स्वतंत्र निष्पादन साक्ष्य और आसपास की timeline से तुलना करें। |
| इसे कौन चला सकता था? | `AccessGained` एक scope/context संकेत है। Machine-wide configuration अक्सर `SYSTEM` बताता है, लेकिन इससे यह सिद्ध नहीं होता कि launch हुई process SYSTEM के रूप में चली। जहाँ उपलब्ध हों, task principals और BITS owner SIDs अधिक विशिष्ट context देते हैं। |
| यह कब बदला? | `KeyLastWrite` पूरी registry key का समय बताता है। `LastChange`, `ChangeSource` और `ChangeEvidence` उपलब्ध होने पर अलग MFT/EVTX correlation का वर्णन करते हैं। इनमें से कोई भी field स्वचालित रूप से payload निष्पादन का समय नहीं बताता। |
| पुनर्प्राप्त स्थिति कितनी विश्वसनीय है? | Replay किए गए, carve किए गए, क्षतिग्रस्त, लापता या सीमित साक्ष्य के लिए `RecordStatus`, `Coverage.csv` और warnings पढ़ें। पुनर्प्राप्त कोई ऐतिहासिक record अब सक्रिय न भी हो। |
| क्या नामित publisher भरोसेमंद है? | `Signer` और `SignerStatus` वर्णनात्मक PE/certificate/catalog metadata हैं। Juntei Authenticode signatures या certificate trust को सत्यापित नहीं करता। |

Findings की अनुपस्थिति persistence की अनुपस्थिति सिद्ध नहीं करती। `complete` coverage row का अर्थ है
कि लागू होने वाला एकत्र किया गया स्रोत पढ़ा गया, न कि acquisition में हर प्रासंगिक फ़ाइल,
हर उपयोगकर्ता या हर संभव Windows persistence तंत्र शामिल है। अन्य coverage states लापता,
आंशिक, disabled या विफल checks की व्याख्या करती हैं। Hunter कोई live-memory, network या cloud-identity scan नहीं है।

## Catalogue और स्थान संकेतन { #catalogue-and-location-notation }

Catalogue PyrsistenceSniper पर आधारित है, साथ में native `start_menu_com` check भी है;
नीचे दिए गए विवरण Juntei के native व्यवहार का दस्तावेज़ीकरण करते हैं, जिसमें उसकी साक्ष्य संबंधी सीमाएँ भी शामिल हैं।
इसमें **116 registry-आधारित
checks और 15 फ़ाइल/repository checks** हैं। कुछ फ़ाइल checks registry data को भी जोड़ते हैं, इसलिए ये
परस्पर अनन्य acquisition आवश्यकताओं के बजाय implementation की गणनाएँ हैं। नीचे तंत्रों को
जाँच के उद्देश्य के अनुसार समूहित किया गया है, और हर समूह के भीतर Check ID के अनुसार वर्णानुक्रम में रखा गया है।

- `HKLM\SYSTEM`, `HKLM\SOFTWARE` और `HKLM\SAM` एकत्र किए गए machine hives को दर्शाते हैं, जो सामान्यतः
  `Windows\System32\config` से, उपलब्ध होने पर transaction logs के साथ, एकत्र किए जाते हैं।
- `HKU\<user>` का अर्थ है प्रासंगिक एकत्र किया गया user hive। अधिकांश user settings NTUSER.DAT से आती हैं;
  class registrations, जहाँ बताया गया है, `AppData\Local\Microsoft\Windows\UsrClass.dat` से आ सकते हैं।
- `<ControlSet>` का अर्थ है SYSTEM में चयनित सक्रिय control set। DSRM check के अतिरिक्त
  ऐतिहासिक स्थान नीचे स्पष्ट रूप से बताए गए हैं। उनकी समीक्षा करते समय `SYSTEM\Select` जाँचें।
- `<name>` एक परिवर्तनशील path घटक है। Registry target tables में `<child>` का अर्थ है तात्कालिक
  child registrations; इसका अर्थ असीमित recursive walk नहीं है। स्पष्ट रूप से recursive checks
  अपने traversal का वर्णन अलग से करते हैं। Office paths में brackets वैकल्पिक version दर्शाते हैं।
- Tables check द्वारा चुने गए वास्तविक machine और user views सूचीबद्ध करती हैं। जहाँ implement किया गया है वहाँ 32-bit view
  सूचीबद्ध है; यह न मानें कि हर check हर संभव architecture/view की जाँच करता है।
- Generic value checks ख़ाली न होने वाली configure की गई entries देते हैं, जिनमें multi-string elements और
  जहाँ लागू हो, numeric शून्य भी शामिल है। ये अपने आप में किसी दुर्भावनापूर्ण payload, enabled
  trigger या सफल निष्पादन की माँग नहीं करते। अधिक चयनात्मक शर्तें हर check के साथ बताई गई हैं।

ATT&CK IDs catalogue से विरासत में मिली mappings हैं और आगे के अध्ययन के लिए links हैं। ये संभावित
दुरुपयोग का वर्णन करती हैं, यह finding नहीं कि किसी हमलावर ने वह technique अपनाई; protection-setting और diagnostic
checks किसी स्वतंत्र persistence तंत्र के बजाय उसे सक्षम करने वाली परिस्थितियों का वर्णन कर सकते हैं।

| तंत्र समूह | Checks |
|---|---:|
| [Boot, sign-in और startup](#boot-sign-in-and-startup) | 32 |
| [सेवाएँ और सिस्टम extensions](#services-and-system-extensions) | 13 |
| [Scheduled tasks](#scheduled-tasks) | 3 |
| [WMI, BITS और अन्य event triggers](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorer, COM, और फ़ाइल या URL handlers](#explorer-com-and-file-or-url-handlers) | 15 |
| [Office एप्लिकेशन और add-ins](#office-applications-and-add-ins) | 7 |
| [Debuggers, crash handlers और compatibility shims](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [.NET runtime और profiler settings](#net-runtime-and-profiler-settings) | 7 |
| [DLL और executable lookup overrides](#dll-and-executable-lookup-overrides) | 18 |
| [Accounts और sign-in दृश्यता](#accounts-and-sign-in-visibility) | 4 |
| [Authentication घटक और credential सुरक्षा](#authentication-components-and-credential-protection) | 8 |
| **कुल** | **131** |

## Boot, sign-in और startup { #boot-sign-in-and-startup }

ये settings boot, sign-in या desktop initialization के दौरान किसी command या library को चलाने की व्यवस्था करती हैं। Windows version और लागू उपयोगकर्ता की समीक्षा करें: कोई setting अप्रचलित, disabled, या किसी हमलावर के बजाय kiosk के लिए अभिप्रेत हो सकती है।

### Active Setup {#active_setup}

**Check ID:** `active_setup` · **Catalogue में ATT&CK:** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**Juntei क्या जाँचता है:** Native और 32-bit views में machine-wide Active Setup component registrations से `StubPath` पढ़ता है। Windows किसी उपयोगकर्ता की profile तैयार करते समय इन commands का उपयोग कर सकता है; यह check शाब्दिक placeholder values `U` और `/UserInstall` को छोड़ देता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` और संगत `Wow6432Node` tree।

**जाँच कैसे करें:** हर component की तुलना उसके installed product से करें और command arguments की जाँच करें। Machine-wide registration sign in करने वाले उपयोगकर्ता के session में निष्पादन का कारण बन सकता है; output का `SYSTEM` label configuration के scope का वर्णन करता है, परिणामी process token का प्रमाण नहीं है।

### Boot Execute {#boot_execute}

**Check ID:** `boot_execute` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Session Manager का `BootExecute` multi-string पढ़ता है और हर non-empty entry को अलग से सुरक्षित रखता है। यह boot के प्रारंभिक चरण के लिए configure किए गए commands दर्ज करता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**जाँच कैसे करें:** `autocheck autochk *` एक परिचित डिफ़ॉल्ट है, और disk tools वैध entries जोड़ सकते हैं। यह माने बिना कि हर ग़ैर-डिफ़ॉल्ट entry चली है या दुर्भावनापूर्ण है, जोड़े गए commands और उनकी फ़ाइलों की जाँच करें।

### Boot Verification Program {#boot_verification_program}

**Check ID:** `boot_verification_program` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** `BootVerificationProgram` का `ImagePath` पढ़ता है, जो boot verification से जुड़ा एक executable स्थान है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**जाँच कैसे करें:** जाँचें कि यह पुराना तंत्र एकत्र किए गए Windows version और boot configuration पर लागू होता है या नहीं। Target फ़ाइल को resolve करें और उसके उद्देश्य की तुलना अपेक्षित boot-management software से करें।

### Explorer Load Value {#explorer_load}

**Check ID:** `explorer_load` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Machine और user hives से Windows sign-in initialization configuration से जुड़ी `Load` value पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**जाँच कैसे करें:** अपरिचित commands, user-writable गंतव्यों और content को fetch या decode करने वाले arguments की जाँच करें। लागू होने और निष्पादन की पुष्टि अलग से करें; ख़ाली परिणाम अन्य sign-in तंत्रों को कवर नहीं करता।

### Font Drivers {#font_drivers}

**Check ID:** `font_drivers` · **Catalogue में ATT&CK:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei क्या जाँचता है:** Legacy `Font Drivers` registry key की non-empty values को संभावित library registrations के रूप में सूचीबद्ध करता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | सभी non-empty values |

**जाँच कैसे करें:** इसे एक असामान्य legacy configuration surface मानें। यह निष्कर्ष निकालने से पहले कि यह load हो सकती थी, Windows-version समर्थन और संदर्भित library की जाँच करें, और वास्तविक उपयोग की पुष्टि process या image-load साक्ष्य से करें।

### Group Policy Extension DLLs {#gp_extension_dlls}

**Check ID:** `gp_extension_dlls` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** Winlogon के अंतर्गत हर तात्कालिक Group Policy extension registration से `DllName` पढ़ता है। ये DLLs policy processing का विस्तार करती हैं।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**जाँच कैसे करें:** Management और security products extensions install कर सकते हैं। Extension GUID और DLL का मिलान अपेक्षित product से करें, और निष्पादन का दावा करने से पहले policy-processing और image-load साक्ष्य की जाँच करें।

### Group Policy Scripts {#gp_scripts}

**Check ID:** `gp_scripts` · **Catalogue में ATT&CK:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei क्या जाँचता है:** एकत्र की गई local Group Policy `scripts.ini` और `psscripts.ini` registration फ़ाइलों से, per-user local policy stores सहित, command/parameter जोड़े पार्स करता है। Startup/shutdown और logon/logoff sections अपने configure किए गए commands और arguments बनाए रखते हैं, और machine sections को user sections से अलग रखा जाता है।

**जाँचा गया साक्ष्य:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` और समकक्ष `GroupPolicy\User\Scripts` फ़ाइलें।
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`।

**जाँच कैसे करें:** केवल implement किए गए local policy registration स्थानों की जाँच की जाती है; यह पूर्ण SYSVOL/GPO audit नहीं है। किसी policy directory के अंतर्गत कहीं और पड़ी script फ़ाइल पर्याप्त नहीं है। इसके registration को policy processing और निष्पादन साक्ष्य से correlate करें।

### Logon Scripts (UserInitMprLogonScript) {#logon_scripts}

**Check ID:** `logon_scripts` · **Catalogue में ATT&CK:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei क्या जाँचता है:** उपयोगकर्ता की `UserInitMprLogonScript` environment value पढ़ता है, जो उस उपयोगकर्ता के sign-in के लिए कोई script चुन सकती है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**जाँच कैसे करें:** स्वीकृत logon scripts से तुलना करें और वास्तविक script की जाँच करें। यह value बाद के sign-in या निष्पादन को सिद्ध नहीं करती, और domain द्वारा दी गई scripts के लिए अतिरिक्त policy साक्ष्य आवश्यक है।

### Platform Execute {#platform_execute}

**Check ID:** `platform_execute` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Session Manager में हर non-empty `PlatformExecute` entry पढ़ता है। यह platform से संबंधित boot initialization के लिए चुने गए प्रोग्रामों की पहचान करता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**जाँच कैसे करें:** Hardware या provisioning workflows किसी entry की व्याख्या कर सकते हैं। तय करें कि इस Windows build पर boot path इसका उपयोग करता है या नहीं, और value के नाम से हमले का अनुमान लगाने के बजाय संदर्भित image की जाँच करें।

### RDP Client Extension DLL {#rdp_clx_dll}

**Check ID:** `rdp_clx_dll` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Terminal Server के default user configuration में `ClxDllPath` को संभावित RDP-संबंधी extension स्थान के रूप में पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**जाँच कैसे करें:** Windows/RDP version और installed extensions की जाँच करें। Configure की गई DLL image-load विश्लेषण के लिए एक सुराग है; केवल registry value यह नहीं दिखाती कि कोई RDP connection या DLL load हुआ।

### RDP Virtual Channel DLL {#rdp_virtual_channel}

**Check ID:** `rdp_virtual_channel` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Machine-wide Terminal Server Client `Default\AddIns` registrations के नीचे की values सूचीबद्ध करता है। Binary values को संभव होने पर UTF-16LE के रूप में decode किया जाता है, जिससे उपयोगी extension paths सुरक्षित रहते हैं।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: सभी values।

**जाँच कैसे करें:** RDP clients और device-redirection products वैध रूप से add-ins register करते हैं। वास्तविक DLL या command और प्रासंगिक client session की जाँच करें; किसी add-in के अंतर्गत हर value executable path नहीं होती।

### RDP WDS Startup Programs {#rdp_wds_startup}

**Check ID:** `rdp_wds_startup` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** RDP Windows Station driver configuration से `StartupPrograms` पढ़ता है। यह remote-session startup के लिए configure किए गए प्रोग्रामों की पहचान करता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**जाँच कैसे करें:** `rdpclip` एक परिचित entry है। जोड़ी गई entries और host के RDP configuration की जाँच करें, और यह तय करने के लिए session/process records का उपयोग करें कि configure किया गया प्रोग्राम वास्तव में शुरू हुआ या नहीं।

### Registry Run Keys {#run_keys}

**Check ID:** `run_keys` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Machine/user hives से Run, RunOnce, RunEx, RunOnceEx और policy Explorer Run values, साथ ही सूचीबद्ध machine 32-bit keys को सूचीबद्ध करता है। RunEx/RunOnceEx sections का traversal सीमाओं और cycle checks के साथ किया जाता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | सभी non-empty values |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | सभी non-empty values |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | सभी non-empty values |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | सभी non-empty values |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | सभी non-empty values |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | सभी non-empty values |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | सभी non-empty values |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | सभी non-empty values |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | सभी non-empty values |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | सभी non-empty values |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | सभी non-empty values |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | सभी non-empty values |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | सभी non-empty values |

**जाँच कैसे करें:** Software updaters और tray एप्लिकेशन आमतौर पर यहाँ दिखाई देते हैं। Executable, command arguments और user context की तुलना करें। RunOnce entry उपयोग के बाद ग़ायब हो सकती है; एकत्र की गई स्थिति पिछले sign-ins का पूरा इतिहास नहीं है।

### RunServices (Legacy) {#run_services}

**Check ID:** `run_services` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Legacy machine-wide `RunServices` key की values सूचीबद्ध करता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | सभी non-empty values |

**जाँच कैसे करें:** यह एक ऐतिहासिक configuration surface है, इस बात की गारंटी नहीं कि कोई आधुनिक Windows build इसे निष्पादित करता है। Target और operating-system पर लागू होने की जाँच करें, फिर स्वतंत्र निष्पादन साक्ष्य खोजें।

### RunServicesOnce (Legacy) {#run_services_once}

**Check ID:** `run_services_once` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Legacy machine-wide `RunServicesOnce` key की values सूचीबद्ध करता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | सभी non-empty values |

**जाँच कैसे करें:** Non-empty values को configuration सुराग मानें। जाँचें कि स्रोत Windows version इस तंत्र का समर्थन करता है या नहीं, और क्या deployment या compatibility software इस entry की व्याख्या करता है।

### S0 Initial Command {#s0_initial_command}

**Check ID:** `s0_initial_command` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Session Manager से `S0InitialCommand` को संभावित Session 0 initialization command के रूप में पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**जाँच कैसे करें:** एकत्र किए गए Windows build और boot स्थिति पर इसकी प्रयोज्यता स्थापित करें। Configure किया गया command boot निष्पादन का timestamp वाला record नहीं है; उसकी फ़ाइल की जाँच करें और boot/process telemetry से correlate करें।

### Session Manager Execute {#session_manager_execute}

**Check ID:** `session_manager_execute` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Session Manager से non-empty `Execute` entries पढ़ता है, जिससे session initialization के लिए चुने गए प्रोग्राम सामने आते हैं।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**जाँच कैसे करें:** अपेक्षित image baseline से तुलना करें और असामान्य native executables या paths की जाँच करें। निष्पादन का दावा करने से पहले स्थापित करें कि बदलाव के बाद कोई प्रासंगिक boot/session हुआ था या नहीं।

### Session Manager SubSystems {#session_manager_subsystems}

**Check ID:** `session_manager_subsystems` · **Catalogue में ATT&CK:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei क्या जाँचता है:** Session Manager की `SubSystems` key में `Windows` value पढ़ता है, जो Windows subsystem command को configure करती है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**जाँच कैसे करें:** सामान्य command लंबा और version पर निर्भर होता है। पूरी value और arguments की तुलना उसी build के भरोसेमंद baseline से करें; किसी महत्वपूर्ण subsystem setting को बदलने से startup भी टूट सकता है।

### Setup Execute {#setup_execute}

**Check ID:** `setup_execute` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Session Manager में `SetupExecute` entries पढ़ता है, जो startup के लिए configure किए गए setup-संबंधी commands को कवर करती हैं।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**जाँच कैसे करें:** Installation, upgrade और recovery गतिविधि वैध entries बना सकती है। Configuration को persistence से जोड़ने से पहले setup logs और संदर्भित image से तुलना करें।

### Shell Folders Startup Redirect {#shell_folders_startup}

**Check ID:** `shell_folders_startup` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Shell Folders और User Shell Folders दोनों में `Startup` और `Common Startup` की तुलना डिफ़ॉल्ट Startup स्थानों से करता है। ग़ैर-डिफ़ॉल्ट redirections की रिपोर्ट की जाती है, और उनके एकत्र किए गए फ़ोल्डर की सामग्री की जाँच की जाती है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` और `User Shell Folders`: `Common Startup`।
- NTUSER.DAT में समकक्ष `Software\Microsoft\Windows\CurrentVersion\Explorer` settings: `Startup`, साथ ही redirect किए गए स्थानों में एकत्र की गई फ़ाइलें।

**जाँच कैसे करें:** Redirect किया गया Startup फ़ोल्डर autostarts को छिपा सकता है, लेकिन redirection वैध भी हो सकता है। Path को एकत्र किए गए साक्ष्य के भीतर resolve होना चाहिए; एकत्र न किया गया गंतव्य अज्ञात है, इस बात का प्रमाण नहीं कि फ़ोल्डर ख़ाली था।

### Shell Launcher Override {#shell_launcher}

**Check ID:** `shell_launcher` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** SOFTWARE से policy `Shell` value और `system.ini` boot Shell mapping पढ़ता है। ये shell चुनने के वैकल्पिक surfaces हैं।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**जाँच कैसे करें:** Kiosks और application appliances जानबूझकर Explorer को बदल सकते हैं। चुने गए shell की तुलना मशीन की भूमिका से करें और पुष्टि करें कि setting प्रासंगिक session के लिए प्रभावी थी।

### Startup Folder {#startup_folder}

**Check ID:** `startup_folder` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** डिफ़ॉल्ट per-user और common Startup फ़ोल्डरों की सीधी फ़ाइलों की जाँच करता है, और पहले उपलब्ध Startup-folder redirection का अनुसरण करता है। जब एकत्र की गई `.lnk` पार्स की जा सकती है, तो शॉर्टकट targets पुनर्प्राप्त किए जाते हैं।

**जाँचा गया साक्ष्य:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` और `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`।
- Machine/user Shell Folders और User Shell Folders settings में लागू `Common Startup` / `Startup` redirections।

**जाँच कैसे करें:** Startup links और फ़ाइलें sign-in पर वैध एप्लिकेशन launch कर सकती हैं। फ़ाइल नामों के साथ-साथ target arguments की भी जाँच करें। Common फ़ोल्डर एक साझा configuration स्थान है; यह सिद्ध नहीं करता कि launch किए गए प्रोग्राम ने SYSTEM token का उपयोग किया।

### Terminal Services Initial Program {#ts_initial_program}

**Check ID:** `ts_initial_program` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Terminal Services policy `InitialProgram` और machine Terminal Server installation के Run/RunOnce/RunOnceEx configuration को पढ़ता है, जिसमें सीमित nested RunOnceEx sections भी शामिल हैं।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | सभी non-empty values |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | सभी non-empty values |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | सभी non-empty values |

**जाँच कैसे करें:** Remote application deployments जानबूझकर इन settings का उपयोग कर सकते हैं। प्रभावित sessions स्थापित करें और चुने गए commands का process साक्ष्य में अनुसरण करें; केवल registration से remote logon का पता नहीं चलता।

### Winlogon AppSetup {#winlogon_appsetup}

**Check ID:** `winlogon_appsetup` · **Catalogue में ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei क्या जाँचता है:** Winlogon से `AppSetup` पढ़ता है और comma से अलग की गई entries को अलग-अलग candidates में विभाजित करता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**जाँच कैसे करें:** हर component की तुलना अपेक्षित Windows setup/sign-in software से करें। सूची की कोई entry अलग से देखा गया निष्पादन नहीं है, और इसकी प्रासंगिकता Windows version तथा deployment स्थिति पर निर्भर कर सकती है।

### Winlogon GinaDLL {#winlogon_gina_dll}

**Check ID:** `winlogon_gina_dll` · **Catalogue में ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei क्या जाँचता है:** Legacy Winlogon `GinaDLL` replacement setting पढ़ता है। GINA एक पुराना Windows sign-in extension model था।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**जाँच कैसे करें:** यह मुख्य रूप से Vista से पहले के सिस्टमों के लिए प्रासंगिक है। किसी नए सिस्टम पर बची हुई key प्रभावी logon extension का प्रमाण नहीं है; स्रोत version और किसी भी संदर्भित DLL की जाँच करें।

**अधिक विवरण:** [Microsoft दस्तावेज़ीकरण](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina)।

### Winlogon MPNotify {#winlogon_mpnotify}

**Check ID:** `winlogon_mpnotify` · **Catalogue में ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei क्या जाँचता है:** Winlogon की `mpnotify` command/library setting पढ़ता है, जो sign-in से संबंधित एक extension surface है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**जाँच कैसे करें:** Target और arguments की तुलना मेल खाते Windows baseline से करें। यह दावा करने से पहले कि code चला, version-विशिष्ट व्यवहार जाँचें और बाद के sign-in से correlate करें।

### Winlogon Notify Packages {#winlogon_notify_packages}

**Check ID:** `winlogon_notify_packages` · **Catalogue में ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei क्या जाँचता है:** हर तात्कालिक Winlogon `Notify` package registration से `DllName` पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**जाँच कैसे करें:** Notification packages एक legacy extension तंत्र हैं। तय करें कि एकत्र किया गया Windows version इन्हें load करता है या नहीं, और स्वीकृत authentication software को अस्पष्ट libraries से अलग करें।

### Winlogon Shell {#winlogon_shell}

**Check ID:** `winlogon_shell` · **Catalogue में ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei क्या जाँचता है:** Desktop shell के चयन को सामने लाने के लिए machine-wide और per-user Winlogon `Shell` values पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**जाँच कैसे करें:** Explorer सामान्य है, लेकिन kiosks कोई दूसरा shell उपयोग कर सकते हैं। जोड़े गए commands या असामान्य paths की जाँच करें और तय करें कि sign-in पर किस उपयोगकर्ता का प्रभावी configuration लागू हुआ।

### Winlogon System {#winlogon_system}

**Check ID:** `winlogon_system` · **Catalogue में ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei क्या जाँचता है:** Winlogon की `System` value पढ़ता है और comma से अलग की गई entries को अलग-अलग candidates में बाँटता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**जाँच कैसे करें:** इसे version-संवेदनशील sign-in configuration सुराग मानें। सूची को run history मानने के बजाय हर संदर्भित component की जाँच करें और निष्पादन साक्ष्य से पुष्टि करें।

### Winlogon Taskman {#winlogon_taskman}

**Check ID:** `winlogon_taskman` · **Catalogue में ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei क्या जाँचता है:** वैकल्पिक task-management command के लिए machine/user Winlogon `Taskman` values पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**जाँच कैसे करें:** तय करें कि कौन-सा Windows व्यवहार इस setting का उपयोग करेगा और क्या target अपेक्षित है। Configure किया गया replacement इस बात का प्रमाण नहीं कि किसी उपयोगकर्ता ने वह feature चलाया।

### Winlogon Userinit {#winlogon_userinit}

**Check ID:** `winlogon_userinit` · **Catalogue में ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei क्या जाँचता है:** Machine/user Winlogon `Userinit` values पढ़ता है, comma से अलग किए गए प्रोग्रामों को विभाजित करता है और ख़ाली अनुगामी items को अनदेखा करता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**जाँच कैसे करें:** सामान्य entry अक्सर `userinit.exe` की ओर इंगित करती है। अतिरिक्त entries और बदले गए paths की जाँच करें। लागू उपयोगकर्ता और बाद के sign-in की पुष्टि करें; `KeyLastWrite` पूरी key से संबंधित है, विशेष रूप से इस value से नहीं।

### Winlogon VMApplet {#winlogon_vmapplet}

**Check ID:** `winlogon_vmapplet` · **Catalogue में ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei क्या जाँचता है:** Winlogon की `VMApplet` command setting पढ़ता है, जो एक पुराने system applet launch surface को कवर करती है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**जाँच कैसे करें:** Operating-system पर प्रयोज्यता और अपेक्षित applet की जाँच करें। कोई अप्रत्याशित command जाँच का पात्र है, लेकिन यह स्थापित नहीं करता कि trigger करने वाला interface खोला गया था।

## सेवाएँ और सिस्टम extensions { #services-and-system-extensions }

Windows सेवाएँ और विस्तार योग्य system components executables या DLLs को load करने के कई वैध तरीके प्रदान करते हैं। Configure किए गए target की तुलना अपेक्षित product, service account, start settings और स्रोत-समर्थित installation इतिहास से करें।

### DiagTrack DLL {#diagtrack_dll}

**Check ID:** `diagtrack_dll` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** DiagTrack सेवा का `ImagePath` पढ़ता है। Catalogue के DLL-केंद्रित नाम के बावजूद, यह check configure किए गए service image command की जाँच करता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**जाँच कैसे करें:** Service baseline और सामान्य service checks से तुलना करें। एक ही configuration एक से अधिक check परिणाम दे सकता है; candidates की गिनती अलग-अलग हमलों की गिनती नहीं है।

### DiagTrack Listener DLL {#diagtrack_listener_dll}

**Check ID:** `diagtrack_listener_dll` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** DiagTrack-Listener autologger configuration से `FileName` पढ़ता है। यह एक ETW autologger setting है, किसी DLL के load होने का साक्ष्य नहीं।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**जाँच कैसे करें:** कच्ची value और Windows logging configuration की जाँच करें। Catalogue इसे DLL technique के रूप में label करता है, लेकिन यह lookup किसी load होने योग्य DLL को सत्यापित नहीं करता और न ही इस value के माध्यम से निष्पादन सिद्ध करता है।

### Netsh Helper DLL {#netsh_helper}

**Check ID:** `netsh_helper` · **Catalogue में ATT&CK:** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**Juntei क्या जाँचता है:** Machine `NetSh` helper registration key के अंतर्गत non-empty values सूचीबद्ध करता है। Helpers DLL-आधारित commands से `netsh.exe` का विस्तार कर सकते हैं।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | सभी non-empty values |

**जाँच कैसे करें:** Network products helpers install कर सकते हैं। हर value का मिलान किसी अपेक्षित product से करें और resolve की गई library की जाँच करें, फिर प्रासंगिक netsh निष्पादन या image loads खोजें।

### Print Monitors {#print_monitors}

**Check ID:** `print_monitors` · **Catalogue में ATT&CK:** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**Juntei क्या जाँचता है:** सक्रिय control set में हर तात्कालिक Print Monitor registration के नीचे `Driver` पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**जाँच कैसे करें:** Printing software और port monitors वैध रूप से spooler का विस्तार करते हैं। DLL और product metadata की तुलना printer inventory से करें, और spooler/image-load साक्ष्य से loading की पुष्टि करें।

### Print Processors {#print_processors}

**Check ID:** `print_processors` · **Catalogue में ATT&CK:** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**Juntei क्या जाँचता है:** x64 और x86 print-processor registration trees के अंतर्गत `Driver` पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**जाँच कैसे करें:** Third-party printer suites परिचित Windows डिफ़ॉल्ट के अलावा processors जोड़ सकते हैं। दुर्भावनापूर्ण इरादा तय करने से पहले दोनों architectures, वास्तविक DLL और installed printer software की समीक्षा करें।

### Service Control Manager Extension {#scm_extension}

**Check ID:** `scm_extension` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** `ServiceControlManagerExtension` में non-empty values सूचीबद्ध करता है, जो एक service-manager extension surface है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | सभी non-empty values |

**जाँच कैसे करें:** नामित libraries की जाँच करें और देखें कि यह Windows build इस setting का उपयोग करता है या नहीं। Boot/service गतिविधि से correlate करें; यह check किसी load हुए extension को सिद्ध करने के बजाय registrations की सूची बनाता है।

### DNS Server Level Plugin DLL {#server_level_plugin_dll}

**Check ID:** `server_level_plugin_dll` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** सक्रिय SYSTEM control set में DNS सेवा की `ServerLevelPluginDll` setting पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**जाँच कैसे करें:** यह DNS Server role चलाने वाले सिस्टमों के लिए प्रासंगिक है, जिनमें कुछ domain controllers भी शामिल हैं। DLL, service account और service restart/load साक्ष्य की जाँच करें; निष्क्रिय या अनुपस्थित role निष्कर्ष बदल देता है।

### Service Failure Command {#service_failure_command}

**Check ID:** `service_failure_command` · **Catalogue में ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei क्या जाँचता है:** सक्रिय control set में हर सेवा का `FailureCommand` पढ़ता है, जिससे configure किया गया recovery command सामने आता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`।

**जाँच कैसे करें:** Command केवल तभी चलता है जब सेवा की recovery actions उसे चुनती हैं और लागू विफलता होती है। Recovery configuration और failure/process records की जाँच करें; केवल value से सक्रिय recovery action स्थापित नहीं होती।

### SNMP Extension Agent DLL {#snmp_extension_agent}

**Check ID:** `snmp_extension_agent` · **Catalogue में ATT&CK:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei क्या जाँचता है:** SNMP सेवा की `ExtensionAgents` registration key के अंतर्गत values सूचीबद्ध करता है। Values सीधे DLL path के बजाय किसी अन्य registry स्थान की पहचान कर सकती हैं।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | सभी non-empty values |

**जाँच कैसे करें:** किसी अप्रत्यक्ष registration का उसकी library तक manually अनुसरण करें और सत्यापित करें कि SNMP install है और उपयोग में है। इस output में unresolved string इस बात का प्रमाण नहीं कि DLL लापता या दुर्भावनापूर्ण है।

### Time Providers {#time_providers}

**Check ID:** `time_providers` · **Catalogue में ATT&CK:** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**Juntei क्या जाँचता है:** तात्कालिक Windows Time `TimeProviders` registrations से `DllName` पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**जाँच कैसे करें:** Provider enable settings और Windows Time सेवा की स्थिति की समीक्षा अलग से करें; DLL path check के लिए enabled provider आवश्यक नहीं है। Time synchronization products वैध replacements दे सकते हैं।

### Windows Service (ServiceDll) {#windows_service_dll}

**Check ID:** `windows_service_dll` · **Catalogue में ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei क्या जाँचता है:** सक्रिय SYSTEM control set में सेवाओं के नीचे `Parameters\ServiceDll` पढ़ता है और उपलब्ध होने पर service-विशिष्ट timestamp साक्ष्य जोड़ता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`।

**जाँच कैसे करें:** `svchost.exe` जैसा साझा host payload DLL नहीं है। Library, service account, start स्थिति और मेल खाते service installation/image-load साक्ष्य की जाँच करें; registration यह सिद्ध नहीं करता कि सेवा चली।

**अधिक विवरण:** [Microsoft दस्तावेज़ीकरण](https://docs.microsoft.com/en-us/windows/win32/services/service-programs)।

### Windows Service (ImagePath) {#windows_service_image_path}

**Check ID:** `windows_service_image_path` · **Catalogue में ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei क्या जाँचता है:** सक्रिय control set में service `ImagePath` commands पढ़ता है। ImagePath के बिना driver types 1, 2 या 8 एक निहित `System32\drivers\<service>.sys` target बनाए रखते हैं।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath`, `Type` और सहायक service metadata।

**जाँच कैसे करें:** Service account और start settings की समीक्षा output के machine scope label से अलग करें। Disabled या demand-start registrations भी inventory का हिस्सा हैं। Timestamps के लिए अस्पष्ट साझा executable paths के बजाय service-name साक्ष्य को प्राथमिकता दी जाती है।

**अधिक विवरण:** [Microsoft दस्तावेज़ीकरण](https://docs.microsoft.com/en-us/windows/win32/services/service-programs)।

### Windows Update Service Startup DLL {#wu_service_startup_dll}

**Check ID:** `wu_service_startup_dll` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** सक्रिय control set से Windows Update सेवा का `Parameters\ServiceDll` पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**जाँच कैसे करें:** Library की तुलना उसी Windows build और installed servicing स्थिति से करें। यह generic ServiceDll inventory से overlap करता है और किसी दूसरे, अलग compromise का प्रतिनिधित्व नहीं करता।

## Scheduled tasks { #scheduled-tasks }

Task Scheduler configuration को XML फ़ाइलों और SOFTWARE hive दोनों में संग्रहीत करता है। इन स्रोतों को एक साथ पढ़ें। कोई task बिना चले भी registered हो सकता है, और अधूरा collection hidden या ghost task जैसा दिख सकता है।

### Ghost Scheduled Task {#ghost_task}

**Check ID:** `ghost_task` · **Catalogue में ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei क्या जाँचता है:** SOFTWARE TaskCache के `Tree` और `Tasks` registrations की तुलना करता है और, जब collection में task फ़ाइलें मौजूद हों, उनकी संगत task XML फ़ाइलें खोजता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` और `TaskCache\Tasks`।
- Filesystem तुलना के लिए एकत्र की गई `Windows\System32\Tasks\<task path>` फ़ाइलें।

**जाँच कैसे करें:** लापता समकक्ष हटाए गए/पुराने tasks, built-in registrations या अधूरे collection को दर्शा सकते हैं। एकत्र की गई XML न होने का अर्थ है कि filesystem तुलना उपलब्ध नहीं है। किसी task को hidden या दुर्भावनापूर्ण कहने से पहले coverage और सटीक विसंगति की जाँच करें।

### Hidden Scheduled Task {#hidden_scheduled_task}

**Check ID:** `hidden_scheduled_task` · **Catalogue में ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei क्या जाँचता है:** TaskCache Tree registrations में लापता `SD` value या ख़ाली binary security descriptor की जाँच करता है; ये ऐसी स्थितियाँ हैं जो किसी task को सामान्य enumeration से छिपाने से जुड़ी हैं।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: `SD` की उपस्थिति/data, TaskCache समकक्ष metadata के साथ।

**जाँच कैसे करें:** लापता/ख़ाली SD एक विशिष्ट configuration विसंगति है, इस बात का प्रमाण नहीं कि task निष्पादित हुआ या दुर्भावनापूर्ण है। XML की `Hidden` setting और मनमानी ACL समस्याएँ अलग स्थितियाँ हैं और यहाँ प्रयुक्त शर्त नहीं हैं।

**अधिक विवरण:** [Microsoft दस्तावेज़ीकरण](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/)।

### Scheduled Task (XML Files) {#scheduled_task_files}

**Check ID:** `scheduled_task_files` · **Catalogue में ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei क्या जाँचता है:** एकत्र की गई task XML से Exec commands/arguments और COM-handler class IDs/data पार्स करता है। यह `Actions Context`/`Principal id` के आधार पर principal चुनता है, user और group पहचान को अलग-अलग बनाए रखता है, और SYSTEM, USER या UNKNOWN की रिपोर्ट करता है। Unresolved COM actions दिखाई देती रहती हैं।

**जाँचा गया साक्ष्य:**

- `Windows\System32\Tasks\<task path>` XML फ़ाइलें, जिनमें Exec, ComHandler और उपलब्ध principal fields शामिल हैं।
- मेल खाते उपयोगकर्ता की एकत्र की गई `UsrClass.dat` और NTUSER `Software\Classes`, साथ ही machine SOFTWARE `Classes\CLSID`, native और 32-bit views में।
- SID principals के लिए एकत्र किया गया SOFTWARE `ProfileList`। नामित principals पहले किसी अद्वितीय मेल खाते `user.DOMAIN` फ़ोल्डर को, फिर एकत्र किए गए local-host suffixes को, और अंत में साधारण `user` फ़ोल्डर को प्राथमिकता देते हैं। Matches को स्पष्ट रूप से heuristics के रूप में label किया जाता है; वास्तविक अस्पष्टता unresolved रहती है और finding तथा coverage में दर्ज की जाती है। Volatile registry keys आवश्यक नहीं हैं।
- Managed `CodeBase` (version subkeys सहित), AppID associations, और AppID `LocalService` के लिए SYSTEM service `ImagePath`।

COM server candidates अपने registry स्रोत को `Description` में शामिल करते हैं। केवल executable नाम वाले
AppID associations को अनुमानित directory देने के बजाय unresolved रखा जाता है। Local
`file:` URLs decode किए जाते हैं; remote URLs network access के बिना यथावत रखे जाते हैं। User class overrides
उसी profile तक सीमित हैं, और अधूरी user CLSID अपने shadowed
machine registration पर नहीं गिरती। केवल group वाले tasks machine candidates बनाए रखते हैं, जिनके description में व्यक्तिगत-उपयोगकर्ता
सीमा बताई जाती है, और coverage error summary में नियमित notices नहीं जोड़े जाते।

**जाँच कैसे करें:** स्रोत XML और event logs में triggers, enabled स्थिति, principal और run history की जाँच करें। बनाया गया task शायद कभी न चले। Unresolved COM handler फिर भी एक candidate है, और लापता/केवल-group principal data का अर्थ SYSTEM नहीं मानना चाहिए।

## WMI, BITS और अन्य event triggers { #wmi-bits-and-other-event-triggers }

Windows Management Instrumentation (WMI) system management का समर्थन करता है। Background Intelligent Transfer Service (BITS) पृष्ठभूमि transfers का समर्थन करता है। दोनों event-triggered commands बनाए रख सकते हैं। कुछ तंत्र किसी management event, transfer notification, application launch या user action की प्रतीक्षा करते हैं। संग्रहीत trigger उपयोगी configuration साक्ष्य है, लेकिन उसकी उपस्थिति यह स्थापित नहीं करती कि trigger चला।

### Accessibility Features Backdoor {#accessibility_tools}

**Check ID:** `accessibility_tools` · **Catalogue में ATT&CK:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei क्या जाँचता है:** एकत्र किए गए किसी reference binary से उपयोगी Microsoft catalog metadata की आवश्यकता पूरी होने के बाद, सात एकत्र किए गए System32 accessibility binaries की तुलना PE OriginalFilename और certificate/catalog metadata से करता है।

**जाँचा गया साक्ष्य:**

- `Windows\System32\sethc.exe`, `osk.exe`, `narrator.exe`, `magnify.exe`, `utilman.exe`, `atbroker.exe` और `DisplaySwitch.exe`।
- एकत्र किए गए `Windows\System32\winlogon.exe`, `cmd.exe` या `notepad.exe` का reference metadata और लागू एकत्र किए गए catalogs।

**जाँच कैसे करें:** Metadata नाम बदलकर किए गए replacement का संकेत दे सकता है, लेकिन Juntei signatures या certificate trust को सत्यापित नहीं करता। आवश्यक catalogs के बिना विरल collection हर unsigned-जैसी दिखने वाली फ़ाइल को backdoor मानने के बजाय अधूरी तुलना coverage देता है।

### Accessibility Features {#assistive_technology}

**Check ID:** `assistive_technology` · **Catalogue में ATT&CK:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei क्या जाँचता है:** Native/32-bit SOFTWARE views में registered accessibility `StartExe` और `StartParams` commands, तथा user और DEFAULT hives से enabled accessibility `Configuration` entries पढ़ता है। Numeric/non-string StartExe placeholders छोड़ दिए जाते हैं।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` और `StartParams`; साथ ही machine `Wow6432Node` view।
- NTUSER.DAT में `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` और DEFAULT hive में समकक्ष setting।

**जाँच कैसे करें:** Accessibility software वैध है। Enabled feature के नाम को उसके वास्तविक executable registration से अलग करें, फिर target और sign-in/desktop निष्पादन साक्ष्य की जाँच करें। यह स्वयं accessibility executable को बदलने से अलग है।

**अधिक विवरण:** [Technique संदर्भ (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/)।

### BITS Job Notification Command {#bits_notify_command}

**Check ID:** `bits_notify_command` · **Catalogue में ATT&CK:** [T1197](https://attack.mitre.org/techniques/T1197/)

**Juntei क्या जाँचता है:** एकत्र किए गए `qmgr.db`, `qmgr0.dat` या `qmgr1.dat` से counted-string BITS job records पुनर्प्राप्त करता है, और non-empty notification programs, parameters, job IDs तथा owner SIDs बनाए रखता है।

**जाँचा गया साक्ष्य:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`, `qmgr0.dat` और `qmgr1.dat`।

**जाँच कैसे करें:** ये carve किए गए records हैं: allocated/current job स्थिति सत्यापित नहीं की जाती और पुराने/हटाए गए jobs दिखाई दे सकते हैं। Database का MFT write time कमज़ोर container साक्ष्य है, notification के निष्पादन का समय नहीं। Owner SIDs SYSTEM, local service, network service और उपयोगकर्ताओं में अंतर करते हैं।

### Command Processor AutoRun {#cmd_autorun}

**Check ID:** `cmd_autorun` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** Machine/user Command Processor `AutoRun` commands पढ़ता है; यह setting command-prompt के startup को प्रभावित कर सकती है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**जाँच कैसे करें:** Development tools और shell customization किसी value की व्याख्या कर सकते हैं। `cmd.exe /d` AutoRun को bypass करता है, इसलिए केवल configuration यह सिद्ध नहीं करता कि जब भी cmd.exe दिखा, यह चला। Command की सामग्री और trigger करने वाली process के arguments की जाँच करें।

**अधिक विवरण:** [Microsoft दस्तावेज़ीकरण](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd)।

### ErrorHandler.cmd Persistence {#error_handler_cmd}

**Check ID:** `error_handler_cmd` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** एकत्र की गई `Windows\System32\ErrorHandler.cmd` script की उपस्थिति को error-handler persistence candidate के रूप में रिपोर्ट करता है।

**जाँचा गया साक्ष्य:**

- `Windows\System32\ErrorHandler.cmd`।

**जाँच कैसे करें:** केवल उपस्थिति यह सिद्ध नहीं करती कि लागू Windows recovery/error path ने इसे चलाया। Script पढ़ें, इसके उद्गम की जाँच करें, और setup/error/process साक्ष्य से correlate करें।

### Power Automate Desktop Flows {#power_automate}

**Check ID:** `power_automate` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** हर उपयोगकर्ता के Power Automate Desktop `flows` और `scripts` stores के नीचे एकत्र की गई flow/script directories खोजता है, और हर खोजे गए flow की रिपोर्ट एक बार करता है।

**जाँचा गया साक्ष्य:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` और `scripts\<flow>\...`।

**जाँच कैसे करें:** यह संग्रहीत automation की पहचान करता है, हर scheduled trigger या निष्पादित flow की नहीं। इसे स्वचालित या दुर्भावनापूर्ण persistence मानने से पहले flow की सामग्री और बाहरी launch configuration की जाँच करें।

### PowerShell Profile {#powershell_profiles}

**Check ID:** `powershell_profiles` · **Catalogue में ATT&CK:** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**Juntei क्या जाँचता है:** मान्यता प्राप्त Windows PowerShell और PowerShell installation/profile directories में profile.ps1-शैली की फ़ाइलें खोजता है, जिनमें user Documents, खोजे गए OneDrive Documents और एकत्र किए गए Documents redirections शामिल हैं।

**जाँचा गया साक्ष्य:**

- `Windows\System32\WindowsPowerShell\v1.0` और `Windows\SysWOW64\WindowsPowerShell\v1.0` में सीधे मौजूद `profile.ps1` से समाप्त होने वाली फ़ाइलें।
- Per-user `Documents\WindowsPowerShell` और `Documents\PowerShell`, जिनमें एकत्र किए गए Documents redirections और खोजे गए OneDrive Documents फ़ोल्डर शामिल हैं।
- `Program Files\PowerShell` या `Program Files (x86)\PowerShell` के अंतर्गत version directories में सीधे मौजूद `profile.ps1` से समाप्त होने वाली फ़ाइलें।

**जाँच कैसे करें:** Profiles में हानिरहित customization हो सकता है। Script पढ़ें और invoke किए गए payloads का अनुसरण करें। `-NoProfile` के साथ PowerShell launch करने पर ये छूट जाते हैं; केवल फ़ाइल की उपस्थिति profile निष्पादन या दुर्भावनापूर्ण सामग्री सिद्ध नहीं करती।

### Screensaver Hijack {#screensaver}

**Check ID:** `screensaver` · **Catalogue में ATT&CK:** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**Juntei क्या जाँचता है:** हर उपयोगकर्ता की Desktop `SCRNSAVE.EXE` value पढ़ता है, जिससे configure की गई screensaver image की पहचान होती है।

**जाँचा गया साक्ष्य:**

- NTUSER.DAT में `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE`।

**जाँच कैसे करें:** स्वीकृत screensavers से तुलना करें और target की जाँच करें। अन्य desktop settings, session स्थिति और timeout तय करते हैं कि यह trigger हो सकता है या नहीं; सहेजा गया path यह सिद्ध नहीं करता कि screensaver शुरू हुआ।

### Telemetry Controller Command {#telemetry_controller}

**Check ID:** `telemetry_controller` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** TelemetryController और उसके तात्कालिक child registrations से `Binary`, `Command` और `MaintenanceCommand` पढ़ता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` और तात्कालिक children: `Binary`, `Command`, `MaintenanceCommand`।

**जाँच कैसे करें:** चुने गए executable और arguments की तुलना उसी Windows build और maintenance software से करें। Registry command यह प्रदर्शित नहीं करता कि कोई maintenance trigger चला।

### Windows Terminal Custom Profiles {#windows_terminal}

**Check ID:** `windows_terminal` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** Packaged, preview और unpackaged Windows Terminal JSON/JSONC settings पार्स करता है और non-empty profile/default `commandline` settings की रिपोर्ट करता है।

**जाँचा गया साक्ष्य:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` और WindowsTerminalPreview का समकक्ष।
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: legacy profiles arrays, `profiles.list` और `profiles.defaults` के commandline fields।

**जाँच कैसे करें:** सामान्य shells स्वाभाविक रूप से यहाँ दिखाई देते हैं। चुनी गई/डिफ़ॉल्ट profile तय करें और असामान्य commands या launch arguments की जाँच करें। केवल profile सहेजने से वह launch नहीं होती।

### WMI Event Subscription {#wmi_event_subscription}

**Check ID:** `wmi_event_subscription` · **Catalogue में ATT&CK:** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**Juntei क्या जाँचता है:** एकत्र की गई WMI OBJECTS.DATA repository से मान्यता प्राप्त event-consumer instances और filter-to-consumer binding references carve करता है। मान्यता प्राप्त classes में command-line, active-script, log-file, event-log और SMTP consumers शामिल हैं।

**जाँचा गया साक्ष्य:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` और `Repository\FS\OBJECTS.DATA`।

**जाँच कैसे करें:** कच्चा repository hit पुराना, unbound या ऐतिहासिक हो सकता है; output उस योग्यता को सुरक्षित रखता है। Carving पूरा सक्रिय subscription graph पुनर्निर्मित नहीं करता और न ही यह सिद्ध करता है कि trigger चला। Repository का file write time subscription के निर्माण/निष्पादन का समय नहीं है।

## Explorer, COM, और फ़ाइल या URL handlers { #explorer-com-and-file-or-url-handlers }

Component Object Model (COM) एप्लिकेशन को registered software components सक्रिय करने देता है। Windows shell फ़ाइलें, URLs, फ़ोल्डर और विशेष objects खोलने के लिए registered handlers का उपयोग करता है। COM identifiers (CLSIDs) components की पहचान करते हैं; ProgIDs कुछ components को पठनीय नाम देते हैं। हर registration का उस command या DLL तक अनुसरण करें जिसे वह चुनता है।

### App Paths Hijack {#app_paths}

**Check ID:** `app_paths` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** Machine-wide `App Paths` registrations के नीचे default values पढ़ता है, जो shell-आधारित launches के लिए application lookup को redirect कर सकती हैं।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>` default value।

**जाँच कैसे करें:** App Paths registration installed एप्लिकेशन के लिए आम है और हर process launch का सामान्य पुनर्लेखन नहीं है। Registered नाम और image की तुलना अपेक्षित software से करें, फिर प्रासंगिक shell/process साक्ष्य की जाँच करें।

### COM Object Hijack {#com_treat_as}

**Check ID:** `com_treat_as` · **Catalogue में ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei क्या जाँचता है:** समर्थित native/32-bit class stores में COM `TreatAs` redirections और उन per-user InprocServer32/LocalServer32 registrations को खोजता है जो किसी machine CLSID को shadow करते हैं। TreatAs targets को उसी उपयोगकर्ता की classes और machine classes से resolve किया जाता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` और machine `Classes\Wow6432Node` view।
- NTUSER.DAT में per-user `Software\Classes\CLSID` और UsrClass.dat में `CLSID`, उनके implement किए गए `Wow6432Node` views सहित: `TreatAs`, `InprocServer32` और `LocalServer32` default values।

**जाँच कैसे करें:** Per-user overrides के लिए administrator अधिकारों की आवश्यकता न हो सकती है, लेकिन वैध software भी इन्हें register कर सकता है। चुनी गई class और DLL/executable का अनुसरण करें, architecture और activation context का आकलन करें, और वास्तविक COM activation की स्वतंत्र रूप से पुष्टि करें।

### Disk Cleanup Handler Hijack {#disk_cleanup_handler}

**Check ID:** `disk_cleanup_handler` · **Catalogue में ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei क्या जाँचता है:** Disk Cleanup VolumeCaches handler CLSIDs पढ़ता है और resolve होने योग्य machine COM InprocServer32 registrations का उनकी libraries तक अनुसरण करता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` default CLSID और `Classes\CLSID\<CLSID>\InprocServer32`।

**जाँच कैसे करें:** चलने के लिए handler को प्रासंगिक cleanup workflow द्वारा invoke किया जाना चाहिए। यह विशेष check unresolved handlers नहीं देता, इसलिए यदि capture अधूरा है तो उनके कच्चे registrations की जाँच के लिए registry parser का उपयोग करें।

### Explorer AppKey Override {#explorer_app_key}

**Check ID:** `explorer_app_key` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** तात्कालिक Explorer `AppKey` children के नीचे `ShellExecute` और `Association` values पढ़ता है, जो application keys द्वारा चुने गए commands/actions को कवर करती हैं।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` और `Association`।

**जाँच कैसे करें:** Keyboard utilities और OEM tools इन values को परिभाषित कर सकते हैं। कोई association नाम फ़ाइल path होने के बजाय एक और registry pivot की माँग कर सकता है। उस user action की जाँच करें जो इसे trigger करेगी।

### Browser Helper Objects {#explorer_bho}

**Check ID:** `explorer_bho` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Browser Helper Object CLSID registrations पढ़ता है और उपलब्ध होने पर machine InprocServer32 libraries को resolve करता है; unresolved class IDs दिखाई देते रहते हैं।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` और machine `Classes\CLSID\<CLSID>\InprocServer32` default value।

**जाँच कैसे करें:** यह संगत browser/Explorer व्यवहार से जुड़ा है और legacy environments के लिए विशेष रूप से प्रासंगिक है। स्वीकृत browser/security extensions दिखाई दे सकते हैं। Registered BHO इस बात का प्रमाण नहीं कि किसी वर्तमान browser ने उसे load किया।

### Explorer Desktop CLSID Hijack {#explorer_clsid_hijack}

**Check ID:** `explorer_clsid_hijack` · **Catalogue में ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei क्या जाँचता है:** Machine/user class stores से तीन सूचीबद्ध विशेष Explorer namespace CLSIDs पर configure किए गए shell-open commands पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**जाँच कैसे करें:** Desktop, My Computer और My Documents के command handlers की तुलना मेल खाते baseline से करें। यह चुने गए CLSIDs और paths की जाँच करता है, हर संभव Explorer namespace hijack की नहीं।

### Explorer Context Menu Handlers {#explorer_context_menu}

**Check ID:** `explorer_context_menu` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Directory, Directory Background, Folder, Drive, AllFilesystemObjects और `*` के context-menu handlers सूचीबद्ध करता है, और संभव होने पर class IDs का एकत्र किए गए DLL registrations तक अनुसरण करता है।

**जाँचा गया साक्ष्य:**

- Machine `HKLM\SOFTWARE\Classes` और user UsrClass.dat class stores: `<subject>\shellex\ContextMenuHandlers\<handler>` default value और संगत COM library registrations।

**जाँच कैसे करें:** Archive tools, security products और cloud clients के right-click extensions आम हैं। वास्तविक handler और trigger करने वाली interaction की जाँच करें। Unresolved GUIDs उपयोगी सुराग बने रहते हैं, इस बात का प्रमाण नहीं कि DLL मौजूद नहीं थी।

### File Association Hijacking {#file_association_hijack}

**Check ID:** `file_association_hijack` · **Catalogue में ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei क्या जाँचता है:** `.txt`, `.pdf`, `.doc`, `.docx`, `.html`, `.htm`, `.js`, `.vbs`, `.hta`, `.exe`, `.bat`, `.cmd` और `.ps1` के लिए सीधे extension और resolve किए गए ProgID `shell\open\command` registrations की जाँच करता है।

**जाँचा गया साक्ष्य:**

- Machine `HKLM\SOFTWARE\Classes` और user UsrClass.dat class stores: चुने गए `<extension>` defaults और `<extension-or-ProgID>\shell\open\command` defaults।

**जाँच कैसे करें:** अपेक्षित एप्लिकेशन से तुलना करें और command arguments की जाँच करें। यह extensions का एक परिभाषित समूह है, सभी file types या पूर्ण प्रभावी UserChoice resolution नहीं। सहेजा गया command यह सिद्ध नहीं करता कि कोई document खोला गया।

### Protocol Handler Hijacking {#protocol_handler_hijack}

**Check ID:** `protocol_handler_hijack` · **Catalogue में ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei क्या जाँचता है:** http, https, mailto, ms-msdt और ms-officecmd के URL open commands, साथ ही खोजे गए `URL Protocol` registrations और लागू user overrides पढ़ता है।

**जाँचा गया साक्ष्य:**

- Machine `HKLM\SOFTWARE\Classes` और user UsrClass.dat class stores: `URL Protocol` markers और `<scheme>\shell\open\command` default values।

**जाँच कैसे करें:** Browsers और application installers वैध रूप से schemes register करते हैं। Command और arguments का अनुसरण करें, scheme के अपेक्षित उद्देश्य का आकलन करें, और URL opening/process साक्ष्य से correlate करें।

### Recycle Bin COM Extension Handler {#recycle_bin_com_extension}

**Check ID:** `recycle_bin_com_extension` · **Catalogue में ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei क्या जाँचता है:** सूचीबद्ध Recycle Bin CLSID open/empty/explore command overrides और तात्कालिक context-menu/drag-drop handler registrations पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**जाँच कैसे करें:** स्वीकृत shell extensions से तुलना करें। कुछ रिपोर्ट की गई values सीधे resolve की गई library के बजाय COM identifiers होती हैं; code निष्पादन का दावा करने से पहले संदर्भित class और user action की जाँच करें।

### Search Protocol Handler Hijack {#search_protocol_handler}

**Check ID:** `search_protocol_handler` · **Catalogue में ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei क्या जाँचता है:** Machine/user `search-ms\shell\open\command` default handler पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**जाँच कैसे करें:** Handler में बदलाव यह बदल सकता है कि search-ms URL खुलने पर क्या होता है, लेकिन इसकी उपस्थिति exploit delivery का प्रमाण नहीं है। Windows डिफ़ॉल्ट से तुलना करें और उस URL/process गतिविधि की जाँच करें जो इसका उपयोग करेगी।

### SharedTaskScheduler {#shared_task_scheduler}

**Check ID:** `shared_task_scheduler` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** Non-empty Explorer `SharedTaskScheduler` values को registered shell components के रूप में सूचीबद्ध करता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | सभी non-empty values |

**जाँच कैसे करें:** Values किसी executable path के बजाय COM classes का संदर्भ दे सकती हैं। उनके server registrations का अनुसरण करें और operating-system व्यवहार, extension के उद्गम तथा प्रासंगिक shell loading साक्ष्य की जाँच करें।

### ShellExecuteHooks {#shell_execute_hooks}

**Check ID:** `shell_execute_hooks` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** Non-empty Explorer `ShellExecuteHooks` values सूचीबद्ध करता है, जो legacy shell-execution hook registrations को कवर करती हैं।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | सभी non-empty values |

**जाँच कैसे करें:** किसी भी संदर्भित CLSID को resolve करें और सत्यापित करें कि Windows build और policy इस hook की अनुमति देते हैं या नहीं। Legacy registration अप्रयुक्त हो सकता है; value स्वयं देखा गया shell निष्पादन नहीं है।

### Start Menu Library and CLSID Folder COM Persistence {#start_menu_com}

**Check ID:** `start_menu_com` · **Catalogue में ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei क्या जाँचता है:** `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu` के नीचे ऐसी `.library-ms` library XML फ़ाइलें खोजता है
जिनका `simpleLocation` URL किसी COM CLSID का संदर्भ देता है, और `<name>.{CLSID}` नाम वाली directories भी।
इन objects के साथ Explorer की interaction COM code को सक्रिय कर सकती है। यह check एकत्र की गई ख़ाली directories को भी शामिल करता है; यह
symbolic links का अनुसरण नहीं करता। Start Menu के बाहर की सामान्य library फ़ाइलें और XML comments में GUIDs
इस check द्वारा रिपोर्ट नहीं किए जाते।

**जाँचा गया साक्ष्य:**

- Per-user Start Menu `.library-ms` फ़ाइलें और CLSID नाम वाली directory entries।
- मेल खाते उपयोगकर्ता के एकत्र किए गए class registrations और machine SOFTWARE, architecture के अनुसार अलग-अलग।
- Scheduled tasks वाले उसी resolver के माध्यम से managed CodeBase/AppID/service configuration।

**जाँच कैसे करें:** `Path` और `SourceFile` activation-trigger आर्टिफ़ैक्ट की पहचान करते हैं। `Value`
में candidate server command होता है, या resolution उपलब्ध न होने पर CLSID। `Description`
trigger को CLSID और एकत्र किए गए registry स्रोत से जोड़ता है। `ResolvedPath`/`Sha256` केवल
resolve किए गए, एकत्र किए गए payload का वर्णन करते हैं। केवल executable नाम वाला AppID association unresolved रहता है।
कोई trigger वैध हो सकता है; अस्तित्व sign-in, activation, निष्पादन या हमले को सिद्ध नहीं करता।
Library URL, class registration, payload, user context और स्वतंत्र निष्पादन logs की जाँच करें।

**संग्रह संबंधी सावधानियाँ:** केवल फ़ाइलों वाले collections ख़ाली directories छोड़ सकते हैं। मानक user Start Menu
paths की जाँच की जाती है; redirect किए गए Start Menus और अन्य Explorer namespace स्थान
इस check द्वारा कवर नहीं होते। लापता hives या unresolved registrations candidate को बनाए रखते हुए आंशिक coverage देते हैं।
Task architecture, elevation और policy प्रभावी COM lookup व्यवहार को बदल सकते हैं।

**अधिक विवरण:** [मूल प्रस्ताव और शोधकर्ता का श्रेय](https://github.com/last-byte/PersistenceSniper/pull/32),
[COM per-user/machine merged views](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root),
[MITRE COM hijacking](https://attack.mitre.org/techniques/T1546/015/)।

### TypeLib COM Hijacking {#typelib_hijack}

**Check ID:** `typelib_hijack` · **Catalogue में ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei क्या जाँचता है:** Per-user UsrClass.dat TypeLib GUID/version/locale win32/win64 registrations पढ़ता है, paths और script-moniker values बनाए रखता है। अंत में आने वाले numeric resource indices केवल target-file resolution के लिए हटाए जाते हैं।

**जाँचा गया साक्ष्य:**

- हर एकत्र की गई UsrClass.dat में `TypeLib\<GUID>\<version>\<locale>\win32` और `win64` default values।

**जाँच कैसे करें:** Type library किसी वैध DLL, वैकल्पिक फ़ाइल या script moniker की ओर इंगित कर सकती है। यह check इन registrations की सूची बनाता है; हर ग़ैर-डिफ़ॉल्ट path को हमला मानने के बजाय प्रभावित एप्लिकेशन का आकलन करें और type-library loading की पुष्टि करें।

## Office एप्लिकेशन और add-ins { #office-applications-and-add-ins }

Office वैध add-ins, templates और application-विशिष्ट extensions का समर्थन करता है। कोई संदिग्ध registration किसी विशेष Office version, एप्लिकेशन, loading setting या user interaction पर निर्भर हो सकता है। केवल configuration macro निष्पादन स्थापित नहीं करता।

### Office Add-in Registration {#office_addins}

**Check ID:** `office_addins` · **Catalogue में ATT&CK:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei क्या जाँचता है:** Unversioned और 14.0/15.0/16.0 Office roots में Word, Excel, PowerPoint, Outlook और Access add-ins सूचीबद्ध करता है, जिनमें implement किया गया machine Click-to-Run store और user roots शामिल हैं। यह स्पष्ट Manifest/FileName/Path values बनाए रखता है या योग्य COM-आधारित entries को resolve करता है, LoadBehavior context के साथ।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` और implement किया गया machine Click-to-Run समकक्ष।
- NTUSER.DAT में `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>`; SOFTWARE, NTUSER.DAT और UsrClass.dat से मेल खाती machine/user COM classes।

**जाँच कैसे करें:** Add-ins आम business software हैं। LoadBehavior, एप्लिकेशन/version, manifests, वास्तविक payloads और अपेक्षित vendor की समीक्षा करें। यह check गारंटी नहीं देता कि हर installed add-in format या registration view एकत्र या load किया गया है।

### Office AI Add-in Hijack {#office_ai_hijack}

**Check ID:** `office_ai_hijack` · **Catalogue में ATT&CK:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei क्या जाँचता है:** Office 16.0 Click-to-Run Common AI registration स्थान की values सूचीबद्ध करता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: सभी values।

**जाँच कैसे करें:** असामान्य values और paths की तुलना installed Office build से करें। किसी value की उपस्थिति यह नहीं दिखाती कि किस Office feature ने उसका उपयोग किया, और न ही यह सिद्ध करती है कि उसने executable code चुना।

### Office DLL Override {#office_dll_override}

**Check ID:** `office_dll_override` · **Catalogue में ATT&CK:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei क्या जाँचता है:** Unversioned और 14.0/15.0/16.0 settings के लिए Word/PowerPoint roots के अंतर्गत `WwlibtDll` और `PPCoreTDLL` पढ़ता है, जिनमें implement किए गए Click-to-Run और user roots शामिल हैं।

**जाँचा गया साक्ष्य:**

- Machine/user `Microsoft\Office\[<version>\]Word` और `PowerPoint` settings, साथ ही implement किया गया machine Click-to-Run Office tree: `WwlibtDll` और `PPCoreTDLL`।

**जाँच कैसे करें:** तय करें कि कौन-सा एप्लिकेशन/build इस override को पहचानता है, library और उसके उद्गम की जाँच करें, और Office launch या image load से correlate करें। केवल configure किया गया override निष्पादन का प्रमाण नहीं है।

### Office Default Templates {#office_templates}

**Check ID:** `office_templates` · **Catalogue में ATT&CK:** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**Juntei क्या जाँचता है:** एकत्र की गई Normal.dotm/NormalEmail.dotm फ़ाइलों और मान्यता प्राप्त Word Startup तथा Excel XLSTART फ़ोल्डरों में सीधे मौजूद फ़ाइलों की सूची बनाता है, जिनमें Office installation startup फ़ोल्डर शामिल हैं।

**जाँचा गया साक्ष्य:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` और `NormalEmail.dotm`।
- Roaming AppData के अंतर्गत user `Microsoft\Word\STARTUP` और `Microsoft\Excel\XLSTART` में सीधे मौजूद फ़ाइलें।
- Program Files और Program Files (x86) के अंतर्गत मान्यता प्राप्त Office `<version>\STARTUP` / `XLSTART` और `root\<version>\STARTUP` / `XLSTART` फ़ोल्डरों में सीधे मौजूद फ़ाइलें।

**जाँच कैसे करें:** कोई template वैध हो सकता है या उसमें कोई executable सामग्री न हो। इस check में Juntei macros को extract या निष्पादित नहीं करता। Document की आंतरिक संरचना, macro policy और एप्लिकेशन गतिविधि की जाँच करें; machine installation path SYSTEM process token का संकेत नहीं देता।

### Office Test DLL {#office_test_dll}

**Check ID:** `office_test_dll` · **Catalogue में ATT&CK:** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**Juntei क्या जाँचता है:** सूचीबद्ध machine/user और 32-bit machine views से Office Test Special Perf key की values सूचीबद्ध करता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | सभी non-empty values |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | सभी non-empty values |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | सभी non-empty values |

**जाँच कैसे करें:** अप्रत्याशित overrides को ऐसा सुराग मानें जिसके लिए Office-version और payload सत्यापन आवश्यक है। यह check configuration की रिपोर्ट करता है, यह परीक्षण नहीं करता कि कोई विशेष Office installation उस value को load करता है या नहीं।

### Outlook Home Page Attack {#outlook_home_page}

**Check ID:** `outlook_home_page` · **Catalogue में ATT&CK:** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**Juntei क्या जाँचता है:** Office versions 14.0, 15.0 और 16.0 के लिए user Outlook WebView children के नीचे `URL` पढ़ता है।

**जाँचा गया साक्ष्य:**

- NTUSER.DAT में `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL`।

**जाँच कैसे करें:** Folder home page वैध हो सकता है, और व्यवहार Outlook version तथा security updates के अनुसार भिन्न होता है। URL/सामग्री और प्रभावी policy की जाँच करें, फिर Outlook/web/process साक्ष्य से correlate करें।

### VBA Monitor DLL Hijack {#vba_monitors}

**Check ID:** `vba_monitors` · **Catalogue में ATT&CK:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei क्या जाँचता है:** Machine और per-user class stores में दो implement किए गए VBA-monitor-संबंधी CLSIDs के लिए InprocServer32 values पढ़ता है।

**जाँचा गया साक्ष्य:**

- Machine SOFTWARE में `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` और `{0002E157-0000-0000-C000-000000000046}`, साथ ही समकक्ष user UsrClass.dat registrations।

**जाँच कैसे करें:** चुनी गई libraries की तुलना installed Office/VBA environment से करें। यह एक लक्षित CLSID check है, हर VBA extension की सूची या इस बात का प्रमाण नहीं कि कोई macro चला।

## Debuggers, crash handlers और compatibility shims { #debuggers-crash-handlers-and-compatibility-shims }

कोई debugger या error handler तब चल सकता है जब कोई एप्लिकेशन शुरू होता है, crash होता है, hang होता है या बंद होता है। Application compatibility databases भी startup को प्रभावित कर सकते हैं। रिपोर्ट किए गए command के साथ-साथ पूर्वापेक्षित settings और trigger करने वाले प्रोग्राम की जाँच करें।

### AeDebug Debugger Hijack {#ae_debug}

**Check ID:** `ae_debug` · **Catalogue में ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei क्या जाँचता है:** Just-in-time crash debugging configuration द्वारा उपयोग किए जाने वाले native और 32-bit AeDebug `Debugger` commands पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**जाँच कैसे करें:** Development/debugging software इसे वैध रूप से set कर सकता है। पूरे debugger command और crash-trigger पूर्वापेक्षाओं की जाँच करें, फिर application-error और process events से correlate करें।

### AeDebug Protected Process Debugger {#ae_debug_protected}

**Check ID:** `ae_debug_protected` · **Catalogue में ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei क्या जाँचता है:** Protected-process debugging surface के लिए native और 32-bit AeDebugProtected `Debugger` commands पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**जाँच कैसे करें:** तय करें कि operating-system build और protection policy configure किए गए command का उपयोग कर सकते हैं या नहीं। सहेजा गया debugger इस बात का प्रमाण नहीं कि कोई protected process crash हुई या command चला।

### Image File Execution Options Debugger {#ifeo_debugger}

**Check ID:** `ifeo_debugger` · **Catalogue में ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei क्या जाँचता है:** Nested filter keys सहित native/32-bit Image File Execution Options registrations में `Debugger` values खोजता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`, nested filter keys और machine `Wow6432Node` view सहित।

**जाँच कैसे करें:** नामित image के launch होने पर कोई debugger चुना जा सकता है। Image/filter scope और debugger arguments की जाँच करें; developer tooling भी IFEO का उपयोग कर सकता है। निष्पादन स्थापित करने के लिए target launch का पता लगाएँ।

### IFEO Delegated NTDLL {#ifeo_delegated_ntdll}

**Check ID:** `ifeo_delegated_ntdll` · **Catalogue में ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei क्या जाँचता है:** Nested keys सहित native/32-bit Image File Execution Options registrations में `VerifierDlls` overrides खोजता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`, nested keys और machine `Wow6432Node` view सहित।

**जाँच कैसे करें:** इस check की शर्त configure की गई VerifierDlls value है; यह सिद्ध नहीं करता कि सभी verifier/delegation पूर्वापेक्षाएँ enabled हैं। काम करने वाले injection path का अनुमान लगाने से पहले लागू flags, Windows version और payload की समीक्षा करें।

### Silent Process Exit Monitor {#ifeo_silent_process_exit}

**Check ID:** `ifeo_silent_process_exit` · **Catalogue में ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei क्या जाँचता है:** Nested keys सहित native/32-bit SilentProcessExit registrations में `MonitorProcess` खोजता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`, nested keys और machine `Wow6432Node` view सहित।

**जाँच कैसे करें:** Silent-exit monitoring के लिए प्रासंगिक reporting/IFEO configuration भी आवश्यक है। उन settings और monitor की गई image के exit events की जाँच करें; वैध diagnostic tooling इस monitor की व्याख्या कर सकता है।

### Application Shimming {#installed_sdb}

**Check ID:** `installed_sdb` · **Catalogue में ATT&CK:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei क्या जाँचता है:** Installed application compatibility databases के लिए तात्कालिक InstalledSDB registrations से `DatabasePath` पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**जाँच कैसे करें:** Compatibility fixes वैध हो सकते हैं। SDB की सामग्री, लक्षित executables और installation इतिहास की जाँच करें। Database registration उसके shims की पहचान नहीं करता और न ही यह सिद्ध करता है कि कोई मेल खाता एप्लिकेशन चला।

### LSM Debugger Hijack {#lsm_debugger}

**Check ID:** `lsm_debugger` · **Catalogue में ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei क्या जाँचता है:** Native और 32-bit machine views में विशेष रूप से `lsm.exe` के लिए SilentProcessExit के अंतर्गत `MonitorProcess` पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**जाँच कैसे करें:** यह व्यापक SilentProcessExit inventory से overlap करता है। Configuration को निष्पादित persistence मानने से पहले पूर्वापेक्षित flags और प्रासंगिक LSM exit की पुष्टि करें; अलग-अलग findings का आकलन करते समय एक ही entry को एक बार गिनें।

### MiniDump Auxiliary DLLs {#minidump_auxiliary_dlls}

**Check ID:** `minidump_auxiliary_dlls` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** MiniDumpAuxiliaryDlls में नामित, non-empty values सूचीबद्ध करता है, value के नाम को trigger-module context और value data को चुने गए target के रूप में सुरक्षित रखता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: trigger-module value names और target data।

**जाँच कैसे करें:** Trigger module और auxiliary DLL दोनों की जाँच करें। Crash/minidump tooling किसी entry की व्याख्या कर सकता है, और registration यह नहीं दिखाता कि किसी dump operation ने library load की।

### Application Shimming {#shim_custom}

**Check ID:** `shim_custom` · **Catalogue में ATT&CK:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei क्या जाँचता है:** तात्कालिक AppCompatFlags Custom application keys के नीचे `.sdb` नाम वाली values पढ़ता है, और application-से-database association को सुरक्षित रखता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: `.sdb` से समाप्त होने वाले value names।

**जाँच कैसे करें:** इन associations को InstalledSDB और वास्तविक compatibility database के साथ मिलाकर देखें। केवल filename association database के व्यवहार को प्रकट नहीं करता या किसी मेल खाते application launch को प्रदर्शित नहीं करता।

### WER Debugger Hijack {#wer_debugger}

**Check ID:** `wer_debugger` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** Native/32-bit Windows Error Reporting `Debugger` values पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**जाँच कैसे करें:** वैध diagnostic software कोई debugger configure कर सकता है। पूरे command और प्रासंगिक crash events की जाँच करें; registration इस बात का प्रमाण नहीं कि Windows Error Reporting ने इसे शुरू किया।

### WER Hangs Debugger Hijack {#wer_hangs}

**Check ID:** `wer_hangs` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** Native/32-bit Windows Error Reporting Hangs `Debugger` values पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**जाँच कैसे करें:** जाँचें कि कोई hang-handling workflow इस setting को invoke करेगा या नहीं, और target प्रोग्राम की जाँच करें। Registry write को debugger निष्पादन मानने के बजाय application hang और process records से correlate करें।

### WER ReflectDebugger Hijack {#wer_reflect_debugger}

**Check ID:** `wer_reflect_debugger` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** Native/32-bit Windows Error Reporting `ReflectDebugger` values पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**जाँच कैसे करें:** स्वीकृत diagnostic configuration से तुलना करें और Windows-version/trigger प्रयोज्यता स्थापित करें। निष्पादन का श्रेय देने से पहले command arguments की जाँच करें और crash/process पुष्टि खोजें।

### WER Runtime Exception Modules {#wer_runtime_exception}

**Check ID:** `wer_runtime_exception` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** Native/32-bit RuntimeExceptionHelperModules के अंतर्गत value names सूचीबद्ध करता है; यहाँ library path value के नाम में होता है, ज़रूरी नहीं कि उसके data में।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` और machine `Wow6432Node` view: value names।

**जाँच कैसे करें:** नामित module और application/runtime context की जाँच करें। Data में छोटी numeric value payload path नहीं है, और केवल registration यह सिद्ध नहीं करता कि helper load हुआ।

## .NET runtime और profiler settings { #net-runtime-and-profiler-settings }

Managed runtimes application code से पहले profilers, startup assemblies या debugging libraries load कर सकते हैं। इन settings का उपयोग developers और monitoring products भी करते हैं। Runtime, process architecture, enable flags, और यह जाँचें कि कोई नई शुरू हुई process इस setting को inherit कर सकती है या नहीं।

### AppDomainManager Injection {#appdomain_manager}

**Check ID:** `appdomain_manager` · **Catalogue में ATT&CK:** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**Juntei क्या जाँचता है:** Machine/user `APPDOMAIN_MANAGER_ASM` और `APPDOMAIN_MANAGER_TYPE` environment variables पढ़ता है, जो custom .NET AppDomain manager के लिए assembly और type के चयन को कवर करते हैं।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**जाँच कैसे करें:** एप्लिकेशन जानबूझकर custom managers का उपयोग कर सकते हैं। Assembly और type को एक साथ resolve करें और runtime compatibility तथा process inheritance की जाँच करें; न तो अकेला variable और न ही उसकी key का timestamp यह सिद्ध करता है कि किसी managed एप्लिकेशन ने इसे load किया।

### .NET CLR Profiler Hijack {#cor_profiler}

**Check ID:** `cor_profiler` · **Catalogue में ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei क्या जाँचता है:** Machine/user COR profiling environment settings और अलग-अलग सेवाओं के Environment multi-strings में मेल खाती नामित entries पढ़ता है। यह enable flags, profiler identifiers और architecture-विशिष्ट paths बनाए रखता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` और हर एकत्र की गई user NTUSER.DAT `Environment`।
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: multi-string में नामित COR settings।

**जाँच कैसे करें:** Profilers वैध monitoring का समर्थन करते हैं। Enable flag, CLSID और DLL path configuration के अलग-अलग हिस्से हैं; हर row को निष्पादित profiler मानने के बजाय प्रभावी enablement और प्रासंगिक .NET process की पुष्टि करें।

### .NET Core CLR Profiler Hijack {#coreclr_profiler}

**Check ID:** `coreclr_profiler` · **Catalogue में ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei क्या जाँचता है:** संगत machine/user CORECLR profiling environment settings और मेल खाती service Environment entries पढ़ता है, जिनमें architecture-विशिष्ट paths शामिल हैं।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` और हर एकत्र की गई user NTUSER.DAT `Environment`।
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: multi-string में नामित CORECLR settings।

**जाँच कैसे करें:** .NET runtime, architecture, enable flag और library को सत्यापित करें। स्वीकृत observability products इन settings को configure कर सकते हैं, और कोई मौजूदा process नए बदले गए environment को inherit नहीं कर सकती।

### .NET DbgManagedDebugger Hijack {#dotnet_dbg_managed_debugger}

**Check ID:** `dotnet_dbg_managed_debugger` · **Catalogue में ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei क्या जाँचता है:** सूचीबद्ध machine/user `.NETFramework` `DbgManagedDebugger` settings पढ़ता है, जिनमें स्पष्ट machine 32-bit path शामिल है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**जाँच कैसे करें:** तय करें कि लागू .NET runtime/debugging policy इस command का उपयोग करती है या नहीं। Developer tooling इसकी व्याख्या कर सकता है; निष्पादन स्थापित करने के लिए managed crashes और process creation से correlate करें।

### .NET Framework COR_PROFILER Registry Key {#dotnet_framework_profiler}

**Check ID:** `dotnet_framework_profiler` · **Catalogue में ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei क्या जाँचता है:** सूचीबद्ध `.NETFramework` registry views से `COR_PROFILER`, `COR_PROFILER_PATH` और `COR_ENABLE_PROFILING` पढ़ता है। यह environment-आधारित profiler checks का पूरक है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**जाँच कैसे करें:** Values का एक साथ मूल्यांकन करें और तय करें कि runtime उस registry स्थान का पालन करता है या नहीं। Configure किया गया profiling tool वैध हो सकता है; किसी अकेले enable flag को पहचाने गए DLL load के रूप में न समझें।

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**Check ID:** `dotnet_startup_hooks` · **Catalogue में ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei क्या जाँचता है:** Machine/user `DOTNET_STARTUP_HOOKS` environment values पढ़ता है, जो समर्थित .NET runtimes के लिए startup-hook assemblies चुन सकती हैं।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` और हर user NTUSER.DAT `Environment\DOTNET_STARTUP_HOOKS`।

**जाँच कैसे करें:** कई assemblies के लिए पूरी value की जाँच करें और runtime समर्थन सत्यापित करें। यह check service-विशिष्ट startup-hook environments को सूचीबद्ध नहीं करता और न ही यह सिद्ध करता है कि किसी process ने value inherit की और उसका उपयोग किया।

### Known Managed Debugging DLLs {#known_managed_debugging_dlls}

**Check ID:** `known_managed_debugging_dlls` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** `.NETFramework` `KnownManagedDebuggingDlls` registrations को संभावित managed-debugger library overrides के रूप में पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**जाँच कैसे करें:** Installed development/runtime tooling से तुलना करें और trigger करने वाले debugger को तय करें। Registry entry debugger session या library load को सिद्ध नहीं करती।

## DLL और executable lookup overrides { #dll-and-executable-lookup-overrides }

ये checks स्पष्ट library स्थानों या उन settings की जाँच करते हैं जो प्रभावित करती हैं कि कोई component code कैसे खोजता है। Path override या कमज़ोर search policy एक जाँच सुराग है; निष्पादन के लिए फिर भी एक payload और एक संगत trigger करने वाला component आवश्यक है।

### AppCert DLLs {#appcert_dlls}

**Check ID:** `appcert_dlls` · **Catalogue में ATT&CK:** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**Juntei क्या जाँचता है:** Non-empty Session Manager AppCertDlls registrations सूचीबद्ध करता है, जो process-creation APIs से जुड़ा एक library extension surface है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | सभी non-empty values |

**जाँच कैसे करें:** हर library और उसके installation उद्गम की जाँच करें। प्रयोज्यता और runtime loading अलग-अलग तय करें; timeline में सामान्य process creation अपने आप में इस बात का प्रमाण नहीं कि configure की गई DLL load हुई।

### AppInit DLLs {#appinit_dlls}

**Check ID:** `appinit_dlls` · **Catalogue में ATT&CK:** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**Juntei क्या जाँचता है:** Native/32-bit `AppInit_DLLs` पढ़ता है, उसकी library सूची को विभाजित करता है, और हर candidate में उपलब्ध `LoadAppInit_DLLs` और `RequireSignedAppInit_DLLs` context जोड़ता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs`, `LoadAppInit_DLLs` और `RequireSignedAppInit_DLLs`; साथ ही machine `Wow6432Node` view।

**जाँच कैसे करें:** शून्य LoadAppInit_DLLs flag को स्पष्ट रूप से निष्क्रिय चिह्नित किया जाता है। Windows version, Secure Boot और signing आवश्यकताएँ भी loading को प्रभावित करती हैं। यह दावा करने से पहले कि कोई AppInit DLL निष्पादित हुई, प्रभावी settings की समीक्षा करें।

**अधिक विवरण:** [Microsoft दस्तावेज़ीकरण](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls)।

### AutodialDLL Override {#autodial_dll}

**Check ID:** `autodial_dll` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** WinSock2 Parameters का `AutodialDLL` override पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**जाँच कैसे करें:** इस setting का उपयोग करने वाले networking feature और स्रोत Windows version की जाँच करें। वास्तविक DLL और network/process गतिविधि की समीक्षा करें; केवल override सक्रिय या दुर्भावनापूर्ण load स्थापित नहीं करता।

### CHM Helper DLL {#chm_helper_dll}

**Check ID:** `chm_helper_dll` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** HtmlHelp Author के अंतर्गत `Location` पढ़ता है, जो help से संबंधित एक library/location override surface है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**जाँच कैसे करें:** तय करें कि प्रासंगिक help component इस सिस्टम पर value का उपयोग करता है या नहीं, और चुनी गई फ़ाइल की जाँच करें। Registry override इस बात का प्रमाण नहीं कि कोई CHM document खोला गया।

### NL6 ContentIndex DLL Override {#content_index_dll}

**Check ID:** `content_index_dll` · **Catalogue में ATT&CK:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei क्या जाँचता है:** सक्रिय control set के तात्कालिक ContentIndex Language registrations से `DLLOverridePath` पढ़ता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`।

**जाँच कैसे करें:** Language/search components Windows build के अनुसार भिन्न हो सकते हैं। Library और मेल खाते installed component की जाँच करें; loading की पुष्टि के लिए वास्तविक indexing process का अनुसरण करें।

### Crypto ExpoOffload DLL {#crypto_expo_offload}

**Check ID:** `crypto_expo_offload` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** Cryptography Offload `ExpoOffload` value को cryptographic library override candidate के रूप में पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**जाँच कैसे करें:** Feature/build प्रयोज्यता की पुष्टि करें और चुनी गई library के उद्गम की जाँच करें। किसी असामान्य value की उपस्थिति को credential चोरी या निष्पादित code का प्रमाण न मानें।

### Direct3D Software Rasterizer DLL {#direct3d_dll}

**Check ID:** `direct3d_dll` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** Direct3D Drivers का `SoftwareRasterizer` library override पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**जाँच कैसे करें:** Graphics और compatibility software rasterizer configuration को प्रभावित कर सकते हैं। DLL और किसी संगत Direct3D एप्लिकेशन की गतिविधि की जाँच करें; path सहेजना यह सिद्ध नहीं करता कि वह load हुई।

### DLL Search Order Weakening {#dll_search_mode}

**Check ID:** `dll_search_mode` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** `SafeDllSearchMode` या `CWDIllegalInDllSearch` की रिपोर्ट केवल तभी करता है जब सक्रिय Session Manager value शून्य के रूप में पार्स होती है। ये search-policy स्थितियाँ हैं, चुने गए payloads नहीं।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` और `CWDIllegalInDllSearch`।

**जाँच कैसे करें:** कमज़ोर lookup policy किसी अलग search-order हमले को आसान बना सकती है। यह किसी रखी गई DLL की पहचान नहीं करती और न ही निष्पादन सिद्ध करती है। ग़ैर-शून्य या अनुपस्थित values इस check द्वारा नहीं दी जातीं; प्रभावी डिफ़ॉल्ट और policy का मूल्यांकन अलग से करें।

**अधिक विवरण:** [Microsoft दस्तावेज़ीकरण](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security)।

### KnownDLLs Exclusion {#exclude_from_known_dlls}

**Check ID:** `exclude_from_known_dlls` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** Session Manager से non-empty `ExcludeFromKnownDlls` entries को protected/known DLL lookup व्यवहार में बदलाव के रूप में पढ़ता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`।

**जाँच कैसे करें:** Exclusion एक lookup स्थिति की पहचान करता है, स्वयं replacement library की नहीं। प्रभावित एप्लिकेशन और संभावित DLL search स्थान खोजें, फिर ग़ैर-मानक image load की पुष्टि करें।

**अधिक विवरण:** [Microsoft दस्तावेज़ीकरण](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order)।

### hhctrl.ocx DLL Override {#hhctrl_ocx_dll}

**Check ID:** `hhctrl_ocx_dll` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** Implement किए गए HTML Help control CLSID के लिए native/32-bit machine और समर्थित user class-store InprocServer32 overrides पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**जाँच कैसे करें:** तय करें कि कौन-सा एप्लिकेशन और architecture इस control को सक्रिय करते हैं। Registered DLL और help-संबंधी गतिविधि की जाँच करें; COM registration यह सिद्ध नहीं करता कि HTML Help ने सामग्री render की।

### Known DLLs {#known_dlls}

**Check ID:** `known_dlls` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** सक्रिय Session Manager KnownDLLs key में नामित values सूचीबद्ध करता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: नामित values।

**जाँच कैसे करें:** कई entries अपेक्षित Windows libraries हैं, और कुछ values payload path के बजाय सहायक configuration हैं। पूरी key की तुलना किसी भरोसेमंद मेल खाते build से करें; यह check हर वैकल्पिक architecture-विशिष्ट KnownDLLs tree को scan नहीं करता।

### LSA Extensions DLL {#lsa_extensions}

**Check ID:** `lsa_extensions` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** LsaExtensionConfig LsaSrv configuration से `Extensions` पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**जाँच कैसे करें:** Security software authentication components का विस्तार कर सकता है। वास्तविक libraries और स्रोत Windows version के साथ उनकी संगतता की जाँच करें, फिर LSASS image-load या अन्य स्वतंत्र उपयोग साक्ष्य खोजें।

### MAPI32 DLL Path Override {#mapi32_dll_path}

**Check ID:** `mapi32_dll_path` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** Native और 32-bit machine views में तात्कालिक Mail client registrations के नीचे `DLLPathEx` और `DLLPath` पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**जाँच कैसे करें:** Mail clients वैध MAPI providers register करते हैं। तय करें कि कौन-सा client/provider चुना गया था और वास्तविक library की जाँच करें; installed provider यह सिद्ध नहीं करता कि incident के दौरान किसी एप्लिकेशन ने MAPI का उपयोग किया।

### MSDTC XA DLL {#msdtc_xa_dll}

**Check ID:** `msdtc_xa_dll` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** MSDTC MTxOCI configuration से `OracleXaLib` और `OracleOciLib` पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**जाँच कैसे करें:** Database integration के लिए वैध रूप से इन libraries की आवश्यकता हो सकती है। Transaction-coordinator configuration और installed database software को सत्यापित करें, और यह मानने के बजाय कि configure किया गया path चला, service/module उपयोग से correlate करें।

### NLDP DLL Override {#nldp_dll}

**Check ID:** `nldp_dll` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** NlsData `DllOverridePath` configuration value पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**जाँच कैसे करें:** Target library, Windows build और उस natural-language component की समीक्षा करें जो इस override का उपयोग कर सकता है। Value यह नहीं दिखाती कि indexing या language processing हुई।

### RDP TestDVCPlugin DLL {#rdp_test_dvc_plugin}

**Check ID:** `rdp_test_dvc_plugin` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** Machine Terminal Server Client की `TestDVCPlugin` library setting पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**जाँच कैसे करें:** किसी अपरिचित test plugin और version-विशिष्ट व्यवहार की जाँच करें। निष्पादन स्थापित करने के लिए एक संगत client session और library load आवश्यक हैं; केवल registry value इनमें से कुछ भी नहीं दिखाती।

### Search Indexer DLL Override {#search_indexer_dll}

**Check ID:** `search_indexer_dll` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** SOFTWARE से Windows Search का `DllPath` override पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**जाँच कैसे करें:** मेल खाते Search component/version को तय करें और target फ़ाइल की जाँच करें। निष्क्रिय Windows Search सेवा या अप्रयुक्त setting किसी देखे गए दुर्भावनापूर्ण library load से अलग है।

### Winsock AutoProxy DLL {#winsock_auto_proxy}

**Check ID:** `winsock_auto_proxy` · **Catalogue में ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei क्या जाँचता है:** तात्कालिक NameSpace_Catalog5 Catalog_Entries और Catalog_Entries64 registrations के नीचे `LibraryPath` पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**जाँच कैसे करें:** Networking providers वैध हो सकते हैं। दोनों architectures, provider के उद्गम और प्रासंगिक network application/image-load साक्ष्य की समीक्षा करें। यह लक्षित namespace-provider lookup हर संभव Winsock extension नहीं है।

## Accounts और sign-in दृश्यता { #accounts-and-sign-in-visibility }

Local account records और sign-in दृश्यता settings पहचान-आधारित persistence को प्रकट कर सकती हैं। ये checks एकत्र किए गए SAM और SOFTWARE hives की जाँच करते हैं; ये Active Directory accounts, domain group membership या पिछले logons की सूची नहीं हैं।

### Administrators Group Membership {#admin_group_membership}

**Check ID:** `admin_group_membership` · **Catalogue में ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei क्या जाँचता है:** SAM से local Administrators alias की member सूची decode करता है। यह उन local-machine SIDs की रिपोर्ट करता है जो किसी hidden account को दर्शाते हैं या एकत्र किए गए SAM में किसी account/group को नहीं दर्शाते।

**जाँचा गया साक्ष्य:**

- SAM hive में `SAM\Domains\Builtin\Aliases\00000220\C` और `SAM\Domains\Account` के नीचे machine SID/user/group records।
- Hidden-name correlation के लिए `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`।

**जाँच कैसे करें:** यह एक लक्षित विसंगति check है, सभी administrators की सूची नहीं। बाहरी/domain SIDs का मूल्यांकन लापता local users के रूप में नहीं किया जाता। दुर्भावनापूर्ण membership का निष्कर्ष निकालने से पहले हटाए गए accounts, capture के समय या अधूरे hives की जाँच आवश्यक है।

### Hidden Local Account {#hidden_account}

**Check ID:** `hidden_account` · **Catalogue में ATT&CK:** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**Juntei क्या जाँचता है:** Winlogon SpecialAccounts UserList में numeric शून्य values की रिपोर्ट करता है, जिससे सामान्य sign-in screen से छिपाए गए नामों की पहचान होती है।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: account-name values।

**जाँच कैसे करें:** Service या maintenance accounts को जानबूझकर छिपाया जा सकता है, और value यह स्थापित नहीं करती कि नामित account मौजूद है या administrator है। SAM account records और logon गतिविधि के साथ जोड़कर देखें।

### RID Hijacking {#rid_hijacking}

**Check ID:** `rid_hijacking` · **Catalogue में ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei क्या जाँचता है:** किसी SAM account subkey के hexadecimal RID की तुलना उसकी F value में offset 0x30 पर संग्रहीत RID से करता है और mismatches की रिपोर्ट करता है।

**जाँचा गया साक्ष्य:**

- SAM hive में `SAM\Domains\Account\Users\<hexadecimal RID>\F`।

**जाँच कैसे करें:** Mismatch एक गंभीर account-record विसंगति है, लेकिन इसके लिए corruption/capture विश्लेषण भी आवश्यक हो सकता है। स्रोत bytes, transaction-log स्थिति और account/logon records की जाँच करें; यह सिद्ध नहीं करता कि किस actor ने record संशोधित किया।

### RID Suborner (Hidden Admin Account) {#rid_suborner}

**Check ID:** `rid_suborner` · **Catalogue में ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei क्या जाँचता है:** उन SAM accounts की रिपोर्ट करता है जिनका subkey RID 500 नहीं है लेकिन F-value RID 500 है; यह administrator-पहचान प्रतिस्थापन का एक अधिक विशिष्ट पैटर्न है।

**जाँचा गया साक्ष्य:**

- SAM hive में `SAM\Domains\Account\Users\<hexadecimal RID>\F`।

**जाँच कैसे करें:** यह सामान्य RID mismatch check से overlap करता है। Catalogue शीर्षक के बावजूद इसके लिए स्वतंत्र रूप से किसी hidden sign-in setting की आवश्यकता नहीं है; account दृश्यता, privileges और वास्तविक account उपयोग की पुष्टि अलग से करें।

## Authentication घटक और credential सुरक्षा { #authentication-components-and-credential-protection }

Local Security Authority (LSA) LSASS और संबंधित components के माध्यम से security कार्यों को संभालता है। Authentication packages, password filters और network providers वैध security software हो सकते हैं या स्थायी credential access का मार्ग। दो checks executable autostarts के बजाय protection settings की सूची बनाते हैं। Host की भूमिका, Windows version और प्रभावी policy के आधार पर उनकी व्याख्या करें।

### AMSI Provider DLL {#amsi_providers}

**Check ID:** `amsi_providers` · **Catalogue में ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei क्या जाँचता है:** Machine AMSI provider CLSIDs सूचीबद्ध करता है और उपलब्ध होने पर उनकी native/32-bit InprocServer32 libraries resolve करता है। Unresolved registrations class ID बनाए रखते हैं।

**जाँचा गया साक्ष्य:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` और resolve होने योग्य `Classes\CLSID` / `Classes\Wow6432Node\CLSID` InprocServer32 registrations।

**जाँच कैसे करें:** Antivirus और security products वैध रूप से AMSI providers register करते हैं। Provider/library के उद्गम और वास्तविक loading की समीक्षा करें; registration यह सिद्ध नहीं करता कि AMSI को bypass किया गया या provider दुर्भावनापूर्ण था।

### Authentication Packages {#authentication_packages}

**Check ID:** `authentication_packages` · **Catalogue में ATT&CK:** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**Juntei क्या जाँचता है:** सक्रिय control set से अलग-अलग non-empty LSA `Authentication Packages` entries पढ़ता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**जाँच कैसे करें:** स्रोत Windows build और installed authentication software से तुलना करें। ग़ैर-डिफ़ॉल्ट package वैध हो सकता है। Interception का निष्कर्ष निकालने से पहले DLL को resolve करें और loading/credential गतिविधि की पुष्टि करें।

### DSRM Admin Logon Behavior {#dsrm_backdoor}

**Check ID:** `dsrm_backdoor` · **Catalogue में ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei क्या जाँचता है:** Implement किए गए ControlSet001, ControlSet002 या CurrentControlSet स्थानों में numeric `DsrmAdminLogonBehavior=2` की रिपोर्ट करता है। यह setting DSRM account के उसके सामान्य recovery context से बाहर उपयोग की अनुमति दे सकती है।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior`, साथ ही मौजूद होने पर ControlSet002 और CurrentControlSet।

**जाँच कैसे करें:** मुख्य रूप से domain controllers के लिए प्रासंगिक। SYSTEM Select का उपयोग करके तय करें कि कौन-सा control set सक्रिय है; किसी ऐतिहासिक control set में match ज़रूरी नहीं कि प्रभावी हो। Value यह सिद्ध नहीं करती कि हमलावर को DSRM password पता था या उसने log on किया।

### Credential Guard Configuration {#lsa_cfg_flags}

**Check ID:** `lsa_cfg_flags` · **Catalogue में ATT&CK:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei क्या जाँचता है:** Configure की गई LSA `LsaCfgFlags` value पढ़ता है, जिसमें शून्य के साथ-साथ सामान्य enabled values भी शामिल हैं। यह executable autostart के बजाय Credential Guard configuration की सूची बनाता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**जाँच कैसे करें:** Value की व्याख्या Windows version, firmware/security पूर्वापेक्षाओं और प्रभावी policy के साथ करें। केवल संग्रहीत शून्य या अनुपस्थित value हमलावर द्वारा किए गए बदलाव, प्रभावी protection स्थिति या credential चोरी को स्थापित नहीं करती।

### LSA Password Filter {#lsa_password_filter}

**Check ID:** `lsa_password_filter` · **Catalogue में ATT&CK:** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**Juntei क्या जाँचता है:** LSA `Notification Packages` पढ़ता है। केवल नाम वाली libraries को एकत्र की गई Windows System32 directory के विरुद्ध resolve किया जाता है, और जहाँ उपयुक्त हो `.dll` जोड़ा जाता है।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**जाँच कैसे करें:** Password policy products आमतौर पर filters install करते हैं। Library और loading साक्ष्य की जाँच करें, और configure किए गए filters को इस प्रमाण से अलग करें कि कोई password change या credential interception हुआ।

### LSASS PPL Protection Status {#lsa_run_as_ppl}

**Check ID:** `lsa_run_as_ppl` · **Catalogue में ATT&CK:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei क्या जाँचता है:** Configure की गई LSA `RunAsPPL` value पढ़ता है, जिसमें ग़ैर-शून्य values भी शामिल हैं। यह एक protection-setting सूची है, यह परीक्षण नहीं कि LSASS बिना सुरक्षा के चला।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**जाँच कैसे करें:** प्रभावी LSA protection Windows version, policy, firmware locking और restart स्थिति पर निर्भर करती है। अनुपस्थित value के लिए कोई row न होना मापा गया protection परिणाम नहीं है। प्रभावी protection का आकलन करने के लिए स्वतंत्र system/runtime साक्ष्य का उपयोग करें।

### Network Provider DLL {#network_provider_dll}

**Check ID:** `network_provider_dll` · **Catalogue में ATT&CK:** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**Juntei क्या जाँचता है:** सक्रिय SYSTEM control set में सेवाओं के नीचे `NetworkProvider\ProviderPath` पढ़ता है।

**जाँचा गया साक्ष्य:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`।

**जाँच कैसे करें:** Network logon providers वैध हो सकते हैं। Provider क्रम/प्रयोज्यता, installed network/security software और वास्तविक library loading की जाँच करें; केवल ProviderPath value credential interception सिद्ध नहीं करती।

### Security Packages {#security_packages}

**Check ID:** `security_packages` · **Catalogue में ATT&CK:** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**Juntei क्या जाँचता है:** अलग-अलग non-empty LSA `Security Packages` entries पढ़ता है, जो configure किए गए security support providers को कवर करती हैं।

**जाँचा गया साक्ष्य:**

| Registry key | जाँची गई value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**जाँच कैसे करें:** अपेक्षित Windows/security-product baseline से तुलना करें और हर library की जाँच करें। Configuration यह सिद्ध नहीं करता कि कोई provider load हुआ, उसने credentials capture किए, या उसे incident के दौरान जोड़ा गया।

## Candidate से निष्कर्ष तक काम करें { #work-from-a-candidate-to-a-conclusion }

1. `SourceFile` द्वारा पहचाने गए मूल स्रोत को खोलें। पूरी value, XML action,
   script या पुनर्प्राप्त database record, और वह सटीक user/host जिससे वह संबंधित है, की पुष्टि करें।
2. तय करें कि तंत्र काम कर सकता था या नहीं: enable flags, runtime या Windows version,
   service/task स्थिति, आवश्यक trigger, उपयोगकर्ता, architecture और सक्रिय control set की जाँच करें।
3. चुने गए प्रोग्राम, DLL, script या assembly की जाँच करें। एकत्र किए गए bytes का hash लें और arguments
   तथा सामग्री की समीक्षा करें। कोई पहचानने योग्य Windows launcher या publisher string वैधता तय नहीं करती।
4. स्वतंत्र साक्ष्य का उपयोग करके स्थापित करें कि वास्तव में क्या हुआ। उदाहरणों में process creation,
   Sysmon image loads, Task Scheduler runtime records, service गतिविधि, Prefetch और संबंधित फ़ाइलें शामिल हैं।
   कोई configuration-change event यह सिद्ध कर सकता है कि setting बदली, बिना यह सिद्ध किए कि उसका payload चला।
5. मूल-स्रोत timestamps और दर्ज किए गए correlation उद्गम का उपयोग करके timeline बनाएँ।
   Key writes, installation बदलाव, file/container writes और निष्पादन को अलग-अलग तथ्यों के रूप में रखें।
6. Findings के साथ-साथ सीमाएँ भी दर्ज करें: लापता hives, लापता target फ़ाइलें, अज्ञात task principals,
   असत्यापित signer metadata, ऐतिहासिक carve किए गए records, या ऐसा trigger जिसकी पुष्टि नहीं की जा सकती।

Command syntax, profiles, हर output field और coverage states के लिए
[Persistence Hunter command मार्गदर्शिका](../commands/persistence-hunter.md) पर लौटें।
[आर्टिफ़ैक्ट मार्गदर्शिकाएँ](../artifacts/index.md) अंतर्निहित स्रोतों की व्याख्या करती हैं, और
[आभार](acknowledgements.md) पृष्ठ upstream persistence tools की पहचान करता है।
