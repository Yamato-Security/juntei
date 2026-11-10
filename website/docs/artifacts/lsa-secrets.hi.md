# LSA सीक्रेट { #lsa-secrets }

## ये क्या हैं { #what-they-are }

Local Security Authority (LSA) `SECURITY` hive में secrets रखता है। इनमें
autologon password, उन accounts के passwords जिनके रूप में services चलती हैं, computer account का domain
password, DPAPI system keys, और वह key शामिल है जिससे cached domain logons encrypt किए जाते हैं। यह
cached domain logons भी रखता है, ताकि domain उपयोगकर्ता domain controller के बिना sign in कर सकें।

`parse-registry lsa-secrets` सूचीबद्ध करता है कि सिस्टम कौन-से secrets रखता है और वे कब सेट किए गए, और यह
कितने cached logons रखता और उपयोग करता है। **यह केवल नाम और समय पढ़ता है: कुछ भी decrypt नहीं किया जाता, और
किसी secret के bytes न पढ़े जाते हैं न लिखे जाते हैं।** Juntei साक्ष्य पढ़ता है; यह credentials extract नहीं करता।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> सिस्टम में इस नाम का एक secret था, जो
इस समय सेट किया गया। किसी service का secret (`_SC_<service>`) का अर्थ है कि LSA उस account का password संग्रहीत करता है
जिसके रूप में वह service चलती है।

## स्थान { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| नाम | प्रकार | यह क्या है |
|---|---|---|
| `DefaultPassword` | `autologon` | वह password जिससे Winlogon अपने-आप sign in करता है। |
| `_SC_<service>` | `service_account` | उस account का password जिसके रूप में `<service>` चलती है। |
| `$MACHINE.ACC` | `machine_account` | Computer account का उसके domain में password। |
| `DPAPI_SYSTEM` | `dpapi_system` | सिस्टम की DPAPI keys। |
| `NL$KM` | `cached_logon_key` | वह key जिससे cached logons encrypt किए जाते हैं। |
| `L$_SQSA_<SID>` | `security_questions` | किसी local account के security questions (Windows 10 और बाद के)। |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | Dial-up और VPN credentials। |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | अन्य private data, उस prefix के अनुसार जो `LsaStorePrivateData` उसे देता है। |

`SECURITY` को उसके transaction logs के साथ, और सिस्टम का `SYSTEM` hive एकत्र करें, जो host का
नाम बताता है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **नाम और समय, values नहीं।** किसी value में data है या नहीं, यह उसके आकार से पता चलता है। उसका content
  पढ़ा नहीं जाता।
- **`DefaultPassword` का अर्थ autologon नहीं है।** यह secret अधिकांश सिस्टमों पर मौजूद होता है, उन पर भी
  जो कभी अपने-आप sign in नहीं करते। `SOFTWARE` का
  `Microsoft\Windows NT\CurrentVersion\Winlogon` (`AutoAdminLogon`, `DefaultUserName`) बताता है
  कि autologon configure है या नहीं। हाल का `CurrentSetUTC` एक ऐसा बदलाव है जिसकी व्याख्या की जानी चाहिए।
- **Service secrets:** `_SC_<service>` तब दिखाई देता है जब कोई service password वाले account के रूप में चलने के लिए
  सेट की जाती है। इसकी तुलना `parse-registry services` (`ObjectName`) से करें, और ऐसी services खोजें
  जो हाल ही में install की गईं या अज्ञात हैं। Service के चले जाने के बाद भी secret बना रह सकता है।
- **समय:** `CurrentSetUTC` और `OldSetUTC` वे समय हैं जब Windows ने वर्तमान और पिछली
  values सेट कीं। `$MACHINE.ACC` password default रूप से हर 30 दिन में बदलता है, इसलिए उसके समय बदलते रहते हैं।
  `KeyLastWrittenUTC` और `CurrVal`/`OldVal` key के समय इस बारे में हैं कि secret आख़िरी बार कब
  बदला।
- **Cached logons:** कोई slot तब उपयोग में होता है जब उसका record किसी उपयोगकर्ता का नाम देता है। Record का header
  encrypted नहीं होता; केवल उसकी user name length पढ़ी जाती है। `CachedLogonsKept` `NL$Control` का count है,
  जो `CachedLogonsCount` (default रूप से 10) का अनुसरण करता है। उपयोग में slots का अर्थ है कि domain उपयोगकर्ताओं ने
  interactive रूप से sign in किया, और उनके credentials hive से offline crack किए जा सकते हैं। उपयोगकर्ताओं के नाम
  encrypted होते हैं और decode नहीं किए जाते। केवल पूर्ण header और सुसंगत user-name length वाले सत्यापित binary records को ही उपयोग में गिना जाता है। यदि `Problems` क्षतिग्रस्त slots report करता है, तो in-use count केवल सत्यापित slots को कवर करता है; क्षतिग्रस्त slots को खाली न समझें।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

हर hive के लिए, summary हर secret को उसके प्रकार और सेट किए जाने के समय के साथ सूचीबद्ध करती है। फिर यह
उन services के नाम बताती है जिनके account passwords LSA संग्रहीत करता है, और उपयोग में cached logon slots गिनती है।

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Host`, `SourcePath` | Host (सिस्टम के `SYSTEM` hive से), और `SECURITY` hive। |
| `Secret`, `Kind` | Secret का नाम, और यह क्या है (ऊपर की table देखें); cached logons के record के लिए `Cache` और `cached_logons`। |
| `ServiceName`, `UserSID` | वह service जिसके लिए `_SC_` secret है, और उपयोगकर्ता का SID जो नाम से मिलता है (`L$_SQSA_`, `RasDialParams`)। |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` और `OupdTime`: वर्तमान और पिछली values कब सेट की गईं। |
| `HasCurrentValue`, `HasOldValue` | क्या `CurrVal` और `OldVal` में data है। |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | `CurrVal`, `OldVal` और secret keys के last writes। |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | Cached logons का record: slots, उपयोग में slots, और `NL$Control` का count। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का संदर्भ। |
| `Problems` | विकृत या सीमा से बाहर के समय (केवल समय metadata दिखाया जाता है), क्षतिग्रस्त cache headers, ग़लत cache value types, और ऐसी keys जो पढ़ी नहीं जा सकतीं। |

## जाँच की रणनीति { #investigation-strategy }

1. अज्ञात या हाल ही में install की गई services के `_SC_` secrets खोजें, और उन्हें
   `parse-registry services` और event 7045 (एक service install की गई) से मिलाएँ।
2. `DefaultPassword` के समय की तुलना `Winlogon` की autologon settings से करें।
3. Domain सदस्यों पर, उपयोग में cached logon slots, और `Cache` आख़िरी बार कब लिखा गया, यह नोट करें।
4. Secrets के समय की तुलना timeline से करें: intrusion की अवधि में सेट किया गया secret
   किसी नए service account, autologon या machine account बदलाव को चिह्नित कर सकता है।

## अधिक जानकारी { #further-details }

- [Microsoft: Private data object](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: Cached domain logon information](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: Cached credentials](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
