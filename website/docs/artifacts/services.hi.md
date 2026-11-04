# Windows सेवाएँ { #windows-services }

## सेवा कॉन्फ़िगरेशन क्या दर्ज करता है { #what-service-configuration-records }

Windows सेवाएँ और kernel drivers हर `SYSTEM` control set के अंतर्गत configure किए जाते हैं। Records में
executable/driver या service DLL, start mode, service type, account, load-order group, delayed
start, error व्यवहार, और विफलता के बाद Windows द्वारा की जाने वाली actions का नाम होता है।

हमलावर persistence, privilege escalation, driver loading और lateral movement के लिए सेवाएँ बनाते या
संशोधित करते हैं। वैध software भी ठीक यही तंत्र उपयोग करता है, इसलिए paths, accounts, signing,
निर्माण का context और runtime साक्ष्य मायने रखते हैं।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> पार्स किए गए control set ने सेवा/driver को
दिखाए अनुसार configure किया था। केवल configuration यह सिद्ध नहीं करता कि सेवा सफलतापूर्वक शुरू हुई।

## स्थान { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

`SYSTEM` को उसके transaction logs के साथ एकत्र करें। System 7045 और Security 4697 जैसे Event IDs
installation दर्ज कर सकते हैं; service-control और process events वास्तविक start/निष्पादन का समर्थन कर सकते हैं।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- `ImagePath` में environment variables, quotes, arguments, device paths, या spaces वाला unquoted
  path हो सकता है। Normalize करने से पहले इसे बिल्कुल वैसा ही सुरक्षित रखें।
- `svchost.exe` द्वारा host की गई सेवाओं के लिए `ServiceDll` अत्यंत महत्वपूर्ण है; केवल `ImagePath` में शायद सिर्फ़ host का नाम हो।
- `StartType=automatic` यह सिद्ध नहीं करता कि सेवा शुरू हुई या enabled बनी रही।
- `ObjectName` configure किया गया account है, ज़रूरी नहीं कि वह उपयोगकर्ता जिसने सेवा install की।
- Drivers और user-mode सेवाएँ अलग-अलग `ServiceType` flags और path conventions का उपयोग करती हैं।
- Failure actions विफलता के बाद service account के रूप में कोई command चला सकती हैं; यह एक persistence
  surface है जो आसानी से छूट जाता है।
- अन्य control sets पिछला configuration सुरक्षित रख सकते हैं; `IsCurrentControlSet` का उपयोग करें।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Host`, `SourcePath` | सिस्टम की पहचान और input hive। सभी collections में सुरक्षित रखें। |
| `ControlSet`, `IsCurrentControlSet` | स्रोत control set और क्या `Select\Current` उसका नाम देता है। सक्रिय स्थिति के दावों के लिए current को प्राथमिकता दें। |
| `ServiceName` | Registry key/आंतरिक सेवा नाम। Events और Service Control Manager commands में pivot करने के लिए मज़बूत आधार। |
| `DisplayName`, `Description` | उपयोगकर्ता को दिखने वाला text; resource references या हमलावर-नियंत्रित हो सकता है। |
| `ImagePath` | Configure किया गया executable/driver command। Quoting, arguments, environment variables और device paths को पार्स करें। |
| `ServiceDLL` | Host service द्वारा load की गई DLL। Shared-process सेवाओं के लिए अक्सर यही वास्तविक payload होती है। |
| `StartType`, `StartValue` | पठनीय/कच्चा start mode: boot (0), system (1), automatic (2), manual (3), disabled (4)। |
| `DelayedAutostart` | Automatic सेवा प्रारंभिक boot चरण के बाद शुरू होती है; फिर भी persistence में सक्षम। |
| `ServiceType`, `TypeValue` | Decode किए गए/कच्चे flags: kernel driver, file-system driver, own/share process, user service आदि। |
| `ErrorControl` | Boot/start error handling स्तर। मुख्यतः operational context। |
| `ObjectName` | Configure किया गया service account या driver object नाम। Installer की पहचान नहीं। |
| `Group` | Load-order group, विशेष रूप से boot/system drivers के लिए महत्वपूर्ण। |
| `FailureResetPeriod` | Failure count reset होने से पहले के सेकंड; `4294967295` का अर्थ है कभी नहीं। |
| `FailureCommand` | `run_command` failure action के लिए configure किया गया command। उच्च-मूल्य वाला persistence फ़ील्ड। |
| `FailureActions` | पूरी decode की गई action सूची (`restart`, `reboot`, `run_command`, `none`) और delays। |
| `FailureActionType`, `FailureActionDelayMs` | Filtering के लिए flatten की गई पहली action; पूरी सूची भी जाँचें। |
| `FailureActionsRaw` | जब decoding अधूरी/क्षतिग्रस्त हो तो मूल bytes। |
| `KeyLastWrittenUTC` | Service key में अंतिम बदलाव, ज़रूरी नहीं कि installation/start का समय। |
| `TransactionLogStatus`, `LogsApplied` | Registry-replay context। |
| `Problems` | Type mismatch, malformed failure structure, या लापता/अपठनीय fields। |

## जाँच की रणनीति { #investigation-strategy }

1. ग़ैर-मानक paths, writable directories, असामान्य accounts, नए key times और failure commands की समीक्षा करें।
2. Environment variables को प्रभावित host के configuration से expand करें, विश्लेषक की मशीन से नहीं।
3. यदि एकत्र किया गया हो तो वास्तविक executable या DLL का hash/signature जाँचें।
4. सेवा के नाम/path को Events 7045/4697, process creation, Prefetch, `$MFT` और USN से correlate करें।
5. बदले गए/हटाए गए persistence के लिए current और non-current control sets की तुलना करें।

## अधिक जानकारी { #further-details }

- [Microsoft: device और driver सेवाओं के लिए Registry tree](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / service types और states](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: ChangeServiceConfig2 failure actions](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
