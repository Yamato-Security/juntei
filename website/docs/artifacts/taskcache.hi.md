# टास्क शेड्यूलर कैश { #task-scheduler-cache }

## TaskCache क्या है { #what-taskcache-is }

Windows Task Scheduler `SOFTWARE` के अंतर्गत एक registry cache रखता है जो task paths को GUIDs,
triggers/lists, principals, actions, runtime metadata और security descriptors से जोड़ता है। Juntei
execution और COM-handler actions के साथ legacy email/message actions, task timestamps, और सामान्य
Task Scheduler interface से छिपाए गए tasks के संकेत decode करता है।

Tasks एक आम persistence और remote-execution तंत्र हैं। Registry cache तब भी मूल्यवान होता है
जब कोई हमलावर task की XML फ़ाइल को हटा देता है या उससे छेड़छाड़ करता है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> TaskCache ने task/action को configure किया या
सुरक्षित रखा। किसी विशेष निष्पादन को सिद्ध करने के लिए runtime fields और स्वतंत्र logs की आवश्यकता होती है।

## स्थान { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

`SOFTWARE` को transaction logs के साथ और पूरा `System32\Tasks` tree एकत्र करें। Task Scheduler
Operational और Security event logs निर्माण/update/runtime के साक्ष्य जोड़ते हैं।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- छेड़छाड़, deletion, अधूरे registration या क्षति के बाद registry cache और task XML में असहमति हो सकती है।
- Juntei का `Hidden` संरचनात्मक रूप से छिपाने (प्रासंगिक स्थितियों में लापता Tree linkage/security descriptor)
  की पहचान करता है, न कि केवल सामान्य “Hidden” UI flag के साथ configure किए गए task की।
- Windows 7 task actions को cache की `Actions` value के बजाय XML में संग्रहीत करता है; वहाँ खाली action
  अपने-आप संदिग्ध नहीं है।
- `LastRun` और `LastResult` उपयोगी हैं, लेकिन पूर्ण, छेड़छाड़-रोधी निष्पादन इतिहास नहीं।
- `Context` configure किए गए principal/context का नाम देता है, उस उपयोगकर्ता का नहीं जिसने task register किया।
- COM handler CLSIDs के लिए अलग registry/file resolution आवश्यक है।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Host`, `SourcePath` | Host संदर्भ और स्रोत `SOFTWARE` hive। |
| `TaskName` | पूरा scheduler path। XML और event logs में प्राथमिक pivot। |
| `TaskGUID` | Tree और Tasks cache entries को जोड़ने वाला GUID। मज़बूत correlation key। |
| `TaskIndex` | Scheduler द्वारा उपयोग किया गया Tree/index metadata; संदर्भात्मक, निष्पादन count नहीं। |
| `TaskLists` | Cache सूचियाँ (`Boot`, `Logon`, `Plain`, `Maintenance`) जो triggering/handling को वर्गीकृत करती हैं। |
| `Hidden` | संरचनात्मक hidden-task संकेतक। छेड़छाड़ का निष्कर्ष निकालने से पहले `InTree`, `InTasks` और descriptor स्थिति की समीक्षा करें। |
| `InTree`, `InTasks` | क्या cache के दोनों पक्ष मौजूद हैं। लापता linkage deletion/छेड़छाड़/अधूरे registration का संकेत दे सकता है। |
| `HasSecurityDescriptor` | क्या Tree entry में `SD` है; इसकी अनुपस्थिति प्रासंगिक versions पर hidden-task व्यवहार में योगदान कर सकती है। |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | Cache के हर पक्ष के लिए key-स्तर के बदलाव; सटीक निर्माण/run समय नहीं। |
| `ActionIndex` | Task के भीतर zero-based action। एक task कई rows उत्पन्न कर सकता है। |
| `ActionType` | `exec`, `com_handler`, `send_email`, या `show_message`। |
| `Executable`, `Args`, `WorkingDirectory` | प्रभावी execution action। Quoting, variables, relative paths और LOLBins की व्याख्या करें। |
| `COM-ClassID`, `ActionData` | COM handler CLSID और data, या legacy email/message payload। CLSID को अलग से resolve करें। |
| `Context` | Actions के साथ encode किया गया principal context (`LocalSystem`, `Author` आदि); configure की गई पहचान, register करने वाला नहीं। |
| `Author`, `URI`, `Description` | Task metadata। आसानी से हमलावर-नियंत्रित और विश्वास का साक्ष्य नहीं। |
| `LastRegisteredUTC` | जहाँ मौजूद हो, DynamicInfo registration/update का समय। |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | Cache किया गया runtime/result metadata। Operational logs और process साक्ष्य से correlate करें। |
| `ActionsRaw` | Decoding अधूरी होने या सत्यापन आवश्यक होने पर मूल action bytes। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का संदर्भ। |
| `Problems` | Path/GUID mismatch, ग़लत value type, truncated action संरचना, या लापता हिस्से। |

## जाँच की रणनीति { #investigation-strategy }

1. पहले hidden/असंगत tasks और non-Microsoft paths की समीक्षा करें।
2. Arguments, working directory, environment expansion और COM resolution सहित प्रभावी commands का पुनर्निर्माण करें।
3. Registry record की तुलना task XML से करें और संदर्भित executable का hash/timestamp लें।
4. `TaskName`/GUID/समय को Task Scheduler events, Security 4698/4702, process logs, Prefetch और MFT/USN से correlate करें।
5. जब हमलावर ने task हटा दिया हो, तो हटाए गए/recover किए गए registry data में खोजें।

## अधिक जानकारी { #further-details }

- [Microsoft: Task Scheduler schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: Task Scheduler event log](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
