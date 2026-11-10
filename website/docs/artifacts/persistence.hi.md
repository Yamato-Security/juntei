# रेजिस्ट्री ऑटोस्टार्ट { #registry-autostarts }

## ये क्या हैं { #what-they-are }

Windows कई registry स्थानों से प्रोग्राम शुरू करता है और DLLs load करता है: boot पर, logon पर, जब कोई
प्रोग्राम शुरू या बंद होता है, जब `cmd.exe` खुलता है, जब कोई COM class बनाई जाती है, या जब spooler,
LSA, netsh या time service अपने plug-ins load करती है। हमलावर इन स्थानों का उपयोग persistence,
privilege escalation और DLL injection के लिए करते हैं, और वैध software भी इनका उपयोग करता है। Juntei इन्हें
`SOFTWARE`, `SYSTEM`, `NTUSER.DAT` और `UsrClass.dat` से पढ़कर एक table में रखता है। यह प्रति
प्रोग्राम, DLL या package एक record लिखता है, और हर एक को उसकी MITRE ATT&CK technique से tag करता है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> hive ने इस प्रोग्राम या DLL को
शुरू या load होने के लिए configure किया था। यह सिद्ध नहीं करता कि प्रोग्राम चला। इसे चलाने के लिए वे शर्तें
आवश्यक हैं जिनका तंत्र नाम लेता है (logon, debug किए गए प्रोग्राम का शुरू होना, class का बनाया जाना), और
निष्पादन के अन्य साक्ष्य भी।

## स्थान { #locations }

| Hive | क्या पढ़ा जाता है |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce` (+`Setup`), `RunOnceEx`, `RunServices(Once)` और `Policies\Explorer\Run`, उनकी `Wow6432Node` और Terminal Server install-mode copies के साथ; `Explorer\StartupApproved\Run`, `Run32` और `StartupFolder`; `Winlogon` (`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … और `Notify`); `LoadAppInit_DLLs` के साथ `Windows` `AppInit_DLLs`; `Image File Execution Options\*` `Debugger` (`UseFilter` के साथ) और `VerifierDlls`; `SilentProcessExit\*` `MonitorProcess`; `Active Setup\Installed Components\*` `StubPath`; `NetSh`; `Command Processor` `AutoRun` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, `Environment` .NET profiler variables); `Control\Lsa` `Authentication`, `Security` और `Notification Packages` (+`OSConfig`); `SecurityProviders`; `Control\Print` monitors, providers और print processors; `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | उपयोगकर्ता की `Run` keys और `StartupApproved`; `Windows NT\CurrentVersion\Windows` `Load` और `Run`; `Winlogon` `Shell`; `Environment` (`UserInitMprLogonScript`, .NET profiler); `Control Panel\Desktop` `SCRNSAVE.EXE`; `PackagedAppXDebug`; `Command Processor` `AutoRun`; `Software\Classes` (`UsrClass.dat` के रूप में) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` और `LocalServer32` (और `Wow6432Node` वाले); प्रोग्राम types (`exefile`, `batfile`, `scrfile`, …) का और auto-elevating प्रोग्राम जो शुरू करते हैं (`ms-settings`, `mscfile`, `Folder`, …) उसका `shell\open\command`; `ActivatableClasses\…\DebugInformation` |

हर hive को उसके transaction logs के साथ एकत्र करें। User hives `Users\<name>\NTUSER.DAT` और
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat` में होते हैं।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **Defaults भी report किए जाते हैं।** एक साफ़ सिस्टम पर अधिकांश records Windows defaults होते हैं:
  - `Userinit` `userinit.exe` है;
  - `Shell` `explorer.exe` है;
  - `KnownDLLs`, LSA packages, print monitors, netsh helpers और Active Setup components सभी में मानक entries होती हैं।

  वह खोजें जो उसी version के साफ़ build से भिन्न है।
- **`StartupApproved` केवल Task Manager की Startup list को कवर करता है।** यह बताता है कि कोई `Run`
  entry या Startup folder फ़ाइल वहाँ disabled है या नहीं, और कब से। जिस entry का कोई मेल खाता `Run`
  value नहीं है, उसे "only in StartupApproved" details के साथ report किया जाता है। यह ऐसा autostart दिखा सकता है जो
  हटा दिया गया था।
- **`OverridesMachineRegistration` के लिए सिस्टम का `SOFTWARE` hive आवश्यक है।** किसी उपयोगकर्ता की COM class या
  command की जाँच इसके विरुद्ध की जाती है, और यह उसी तरह खोजा जाता है जैसे host का `SYSTEM` hive। कोई
  override COM hijacking pattern (T1546.015) है। OneDrive और अन्य per-user installs भी
  अपनी classes register करते हैं जो कुछ भी override नहीं करतीं।
- **`ImagePath` और `Args` को command line से केवल वहीं अलग किया जाता है जहाँ यह पहचाना जा सके।**
  - Quote किया गया प्रोग्राम उसके closing quote पर विभाजित होता है।
  - बिना फ़ोल्डर वाले पहले शब्द को प्रोग्राम माना जाता है (`cmd /c …`)।
  - अन्यथा विभाजन उस सबसे छोटे आरंभिक भाग पर होता है जो किसी प्रोग्राम के extension पर समाप्त होता है।

  Spaces वाला unquoted path कोई अलग प्रोग्राम (`C:\Program.exe`) चला सकता है, यदि वह मौजूद हो।
- **`VerifierDlls` और `MonitorProcess` के लिए `GlobalFlag` `Enabled` तय करता है।** `VerifierDlls` केवल
  `0x100` के साथ load होता है। `MonitorProcess` केवल `0x200` और `SilentProcessExit\<program>`
  `ReportingMode` के `0x1` के साथ चलता है। `2` (dump) या `4` (notification) mode कुछ भी नहीं चलाता, और
  `Details` mode के flags का नाम बताता है।
- **`KeyLastWrittenUTC` key का last write है**, value का नहीं। कई values इसे साझा करती हैं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

Summary तंत्र के अनुसार records गिनता है। यह उन `StartupApproved\Run` और `Run32`
entries को भी सूचीबद्ध करता है जिनका `Run` value नहीं बचा (Startup folder फ़ाइलों के पास खोने के लिए कोई value नहीं होता), और वे user COM
classes और commands जो machine वालों को override करते हैं।

हर `parse-registry` command की तरह, यह ऐसे `-o` को अस्वीकार करता है जो उन फ़ाइलों में से एक हो जिन्हें यह पढ़ता है, `-C`
के साथ भी। इसमें hive, उसके transaction logs, और सिस्टम के SYSTEM और SOFTWARE hives शामिल हैं,
path, symbolic link या hard link द्वारा। जब यह निश्चित नहीं कर पाता, तो यह नहीं लिखता।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host context, स्रोत hive और उसका type। |
| `User`, `UserSID` | User hives के लिए: profile फ़ोल्डर, और SID। SID `UsrClass.dat` की root key के नाम से आता है, अन्यथा सिस्टम के `ProfileList` से। |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | MITRE ATT&CK technique, जहाँ कोई उपयुक्त हो (`T1547.001`, `T1546.012`, …)। |
| `KeyPath`, `ValueName` | Hive के root के अंतर्गत key (नाम जैसे लिखे गए) और value (खाली: default value)। |
| `Data` | Value का text, या किसी list का एक item। पढ़ा न जा सकने पर, या जब केवल `StartupApproved` entry का नाम लेता है, तब खाली। |
| `ImagePath`, `Args` | प्रोग्राम या DLL, और arguments, जहाँ उन्हें अलग पहचाना जा सके (ऊपर देखें)। |
| `Enabled`, `DisabledAtUTC` | जो `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag` (`ReportingMode` के साथ), `UseFilter`, `IsInstalled`, time provider का `Enabled` या `ScreenSaveActive` बताता है। जब कुछ नहीं बताता तब खाली। |
| `Details` | अतिरिक्त: pending rename क्या करता है, COM class का नाम, `ReportingMode`, `FilterFullPath`, Active Setup का `ShellComponent`, … |
| `OverridesMachineRegistration` | किसी उपयोगकर्ता की COM class या command के लिए: क्या machine वही register करती है। सिस्टम के `SOFTWARE` hive के बिना खाली। |
| `WOW64` | `Wow6432Node` के अंतर्गत: 32-bit प्रोग्रामों के लिए। |
| `KeyLastWrittenUTC` | Key का last write। |
| `DataRaw` | Value के bytes (hex), जब वे text न हों या मान्य UTF-16 न हों। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का context। |
| `Problems` | गलत value types, ऐसा data जिसे पढ़ा नहीं जा सकता, गलत आकार की `StartupApproved` entries। |

## जाँच की रणनीति { #investigation-strategy }

1. Summary से शुरू करें:
   - COM classes और commands जो machine वालों को override करते हैं;
   - `Run` value के बिना `StartupApproved` entries;
   - वे तंत्र जिनमें शायद ही कभी entries होती हैं (`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`)।
2. LSA packages, `BootExecute`, `KnownDLLs`, print monitors और netsh helpers की तुलना उसी Windows
   version के साफ़ build से करें।
3. `ImagePath` को file system ($MFT, hashes और signatures) के विरुद्ध resolve करें, और user
   profile, `ProgramData` और `Temp` paths जाँचें।
4. `KeyLastWrittenUTC` को Security और Sysmon registry events, process creation,
   Prefetch और Amcache से correlate करें।
5. इन keys से हटाए गए autostarts खोजने के लिए `parse-registry read --recover-deleted` का उपयोग करें।

## अधिक जानकारी { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: Run and RunOnce registry keys](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
