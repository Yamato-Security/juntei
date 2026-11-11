# Windows रेजिस्ट्री हाइव { #windows-registry-hives }

## रेजिस्ट्री हाइव क्या हैं { #what-registry-hives-are }

Windows Registry system और उपयोगकर्ता configuration को binary hive files में संग्रहीत करती है। Keys एक tree बनाती हैं;
किसी key के अंतर्गत values में typed डेटा होता है। Registry आर्टिफ़ैक्ट persistence, installed software,
services, accounts, network configuration, उपयोगकर्ता गतिविधि, सुरक्षा settings और Windows state के लगभग हर भाग
का वर्णन करते हैं।

Juntei का generic reader किसी भी primary hive को decode करता है, transaction logs को replay करता है, क्षति की रिपोर्ट करता है, और
deleted/free-cell, cell-slack, remnant, pre-replay तथा केवल transaction-log में मौजूद records को recover कर सकता है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> hive में key/value state मौजूद थी या recover किए जा सकने योग्य रूप में
सुरक्षित थी। Key का last-write time दिखाता है कि key बदली, यह नहीं कि कौन-सी value बदली या क्यों।

## सामान्य hives और स्थान { #common-hives-and-locations }

| Hive | सामान्य path | साक्ष्य के उदाहरण |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | Services, drivers, control sets, host name, mounted devices, ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | Installed software, TaskCache, profiles, Windows configuration |
| `SAM` | `C:\Windows\System32\config\SAM` | Local accounts और account metadata |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | Local security policy/secrets metadata (access पर निर्भर) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | Default system profile configuration |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | उपयोगकर्ता preferences, Run keys, RecentDocs, application गतिविधि |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | Shell classes, COM/उपयोगकर्ता shell गतिविधि |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | Application inventory (semantic आउटपुट के लिए `parse-registry amcache` का उपयोग करें) |

साथ की `.LOG`, `.LOG1` और `.LOG2` files संग्रहित करें। Windows चलते समय copy किया गया hive
dirty हो सकता है; transaction logs में committed state होती है जो अकेली primary file अभी तक नहीं दर्शाती।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- Key का last-write timestamp **key** पर लागू होता है, किसी एक value पर नहीं।
- Registry values का आमतौर पर कोई स्वतंत्र timestamp नहीं होता।
- `CurrentControlSet` एक विश्लेषक/Windows alias है जो `SYSTEM\Select` के माध्यम से resolve होता है; यह
  वास्तव में संग्रहीत key नहीं है।
- Deleted records आंशिक, पुराने, कई recovery sources में duplicated, या overwritten हो सकते हैं।
- केवल transaction-log में मौजूद records ऐतिहासिक/मध्यवर्ती state दिखाते हैं। `recovered_from` को स्पष्ट करें।
- `absent_value` परिणाम का अर्थ है कि query की गई key मौजूद है लेकिन parse की गई state में उसमें वह value नहीं है;
  यह सिद्ध नहीं करता कि value कभी मौजूद नहीं थी।
- Strings, paths और command lines configuration हैं, इस बात का प्रमाण नहीं कि configure की गई क्रिया चली।
- लागू logs के बिना dirty hive पुरानी state को छोड़ सकता है या उजागर कर सकता है।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host` | उपलब्ध system संदर्भ से प्राप्त host name; जब Juntei इसे सुरक्षित रूप से पहचान नहीं सकता तो खाली। |
| `HiveType` | पहचानी गई hive श्रेणी (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT` आदि)। Paths की व्याख्या के लिए उपयोग करें। |
| `SourcePath` | सटीक input hive। जब directory mode कई hosts/उपयोगकर्ताओं को मिलाता है तो आवश्यक। |
| `RecordType` | `key`, `value`, `default_value` या `absent_value`। अनुपस्थिति केवल query/state पर लागू होती है। |
| `KeyPath` | Hive root के नीचे का path। `[unknown]` resolve न हुई deleted ancestry को चिह्नित करता है। |
| `KeyLastWrittenUTC` | पूरी key का अंतिम संशोधन। यह नहीं बता सकता कि कौन-सी value बदली। |
| `ValueName` | Named value; default/unnamed value के लिए खाली। |
| `ValueType` | Registry type (`REG_SZ`, `REG_DWORD`, `REG_BINARY` आदि)। Type व्याख्या को प्रभावित करता है। |
| `ValueData` | Type के अनुसार render की गई value। Multi-strings JSON में lists बनी रहती हैं। |
| `DataSize` | Value द्वारा दर्ज size। Raw bytes और `Problems` से तुलना करें। |
| `ValueDataRaw` | Decoding विफल होने पर या `--raw-data` का उपयोग होने पर सटीक bytes। सत्यापन के लिए सुरक्षित रखें। |
| `ValueU64` | DWORD/QWORD का पूर्ण संख्यात्मक प्रतिनिधित्व, JavaScript precision loss के बिना। |
| `ValueFiletimeUTC` | वैकल्पिक QWORD-as-FILETIME rendering। केवल तभी अर्थपूर्ण जब key का schema FILETIME परिभाषित करता हो। |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only` या `transaction_log_recovered`। विश्वसनीयता को नियंत्रित करता है। |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay`, या `SYSTEM.LOG1#1234` जैसी log entry। |
| `TransactionLogStatus` | इस record/hive संदर्भ के लिए `clean`, `replayed`, `unchanged` या `not_applied`। |
| `LogsApplied` | Juntei द्वारा replay की गई transaction-log files के नाम। इन्हें रिपोर्ट में दर्ज करें। |
| `Problems` | क्षति, अनुपस्थित cells, type mismatch, resolve न हुई lists/parents, या अन्य सीमाएँ। |

## Recovery status की व्याख्या { #recovery-status-interpretation }

| Status | अर्थ |
|---|---|
| `deleted_complete` | Deleted key/value को पूर्ण ज्ञात path/डेटा के साथ recover किया गया। फिर भी ऐतिहासिक है, सक्रिय state नहीं। |
| `deleted_partial` | कुछ path, value list, value या डेटा overwritten/अपठनीय था। पुनः उपयोग हुए cells से रिक्त स्थान न भरें। |
| `residual_value_only` | Value पर्याप्त key संदर्भ के बिना बची रही; `KeyPath` अनुपस्थित हो सकता है। |
| `transaction_log_recovered` | State केवल pre-replay/log संस्करणों में मौजूद है, वर्तमान allocated/free records में नहीं। |

## जाँच की रणनीति { #investigation-strategy }

1. ज्ञात होने पर किसी सटीक semantic key से शुरू करें; खोज के लिए regex/data searching का उपयोग करें।
2. Content से पहले `TransactionLogStatus`, `AllocationStatus`, `RecoveredFrom` और `Problems` जाँचें।
3. डेटा की व्याख्या key के documented schema के अनुसार करें; `REG_BINARY` स्वयं अपना वर्णन नहीं करता।
4. Configuration को runtime साक्ष्य (event logs, Prefetch, SRUM, services/tasks) से correlate करें।
5. नए या विवादित fields के लिए raw डेटा सुरक्षित रखें।

## अधिक जानकारी { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Maxim Suhanov का registry format specification](https://github.com/msuhanov/regf)
- [yarp registry parser](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Microsoft: Registry value types](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
