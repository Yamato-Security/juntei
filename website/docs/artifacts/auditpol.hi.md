# ऑडिट नीति { #audit-policy }

## यह क्या है { #what-it-is }

ऑडिट नीति बताती है कि Windows कौन-से security events को Security event log में दर्ज करता है। Windows Vista
और बाद के versions पर यह प्रति subcategory set की जाती है (Windows 10 और 11 पर 59), और Windows XP तथा
Server 2003 पर प्रति category। हर एक successes, failures, दोनों, या कुछ भी दर्ज नहीं करता।

LSA लागू नीति को `SECURITY` hive में रखता है, चाहे वह `auditpol`, local
security policy या Group Policy से आई हो।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> जब hive अंतिम बार लिखा गया था, तब सिस्टम
इन events को दर्ज करने (या न करने) के लिए set था। जब किसी subcategory का ऑडिट नहीं होता, तो किसी event की
अनुपस्थिति इस बारे में कुछ नहीं बताती कि गतिविधि हुई या नहीं।

## स्थान { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista और बाद के versions एक 12-byte header रखते हैं, फिर category दर category हर subcategory
के लिए एक setting, फिर यह कि हर category में कितनी subcategories हैं। Versions केवल इन गिनतियों में भिन्न होते हैं:
हर version किसी category के अंत में subcategories जोड़ता है (पढ़े गए Windows 8.x और Server 2012 R2 hives
पर 56, Windows 10, 11 और Server 2022 पर 59)। Windows XP और Server 2003 यह रखते हैं कि auditing
चालू है या नहीं (एक byte, फिर padding जिसे Windows साफ़ नहीं करता) और नौ category settings। उस layout को
NIST के CFReDS Hacking Case की सार्वजनिक Windows XP image पर जाँचा गया था, जो किसी चीज़ का ऑडिट नहीं करती;
इसकी category settings के values Microsoft के documentation का अनुसरण करते हैं।

`SECURITY` को उसके transaction logs के साथ एकत्र करें, और सिस्टम का `SYSTEM` hive भी, जो
host का नाम बताता है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **अज्ञात data अज्ञात ही रहता है।** असमर्थित revisions को diagnostics के साथ सुरक्षित रखा जाता है। अज्ञात
  setting flags `Success` और `Failure` को खाली छोड़ते हैं और audited-setting गिनती से बाहर रखे जाते हैं।
  क्षतिग्रस्त count table आरक्षित word को किसी अन्य audit setting में नहीं बदल सकती।
  Captures में name/GUID catalogue से परे अतिरिक्त slots हो सकते हैं। उनकी settings और
  संख्यात्मक `SubcategoryId` खाली names/GUIDs के साथ रखे जाते हैं; unmapped slot अपने-आप में
  corruption का साक्ष्य नहीं है।


- **क्रम Windows का अपना है।** किसी category के भीतर settings subcategories के message IDs
  के क्रम में होती हैं, उनके GUIDs के क्रम में नहीं। Special Logon (`%%12548`, Logon/Logoff का index
  4) IPsec Quick Mode से पहले आता है। जो टूल इन्हें GUID क्रम में पढ़ते हैं, वे Windows 10 के
  default को Special Logon "No Auditing" और IPsec Quick Mode "Success" के रूप में report करते हैं। `SubcategoryId` वही
  ID है जो event 4719 (System audit policy was changed) देता है, इसलिए दोनों का मिलान किया जा सकता है।
- **लागू नीति, उसका स्रोत नहीं।** Group Policy और local policy एक ही value लिखते हैं।
  Group Policy का `audit.csv` (SYSVOL और `C:\Windows\System32\GroupPolicy` में) दिखाता है कि क्या
  लागू किया जाना था (किसी policy का `Machine\Microsoft\Windows NT\Audit\audit.csv`)।
- **समय के साथ बदलाव:** `KeyLastWrittenUTC` लगभग यह बताता है कि नीति अंतिम बार कब बदली। Event 4719
  हर बदलाव दर्ज करता है (जब तक Security log उसे रखता है)। हमलावर auditing बंद कर सकता है,
  और फिर उसे वापस चालू कर सकता है।
- **Subcategories बनाम categories।** Windows Vista और बाद के versions पर Windows subcategories का ही
  उपयोग करता है (जब तक `SCENoApplyLegacyAuditPolicy` security option बंद न हो और कोई legacy
  category policy उन्हें override न करे)।
- **Windows XP और 2003:** auditing बंद होने पर (`AuditingEnabled` False), किसी भी category का ऑडिट नहीं होता,
  चाहे उसकी setting कुछ भी हो।
- **Events:** `Events` हर subcategory के मुख्य events सूचीबद्ध करता है, जो advanced audit policy settings के
  Microsoft documentation से लिए गए हैं। ये वे events हैं जिन्हें जाँच में आमतौर पर खोजा जाता है, न कि
  subcategory द्वारा दर्ज किया जाने वाला हर event।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

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
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

हर hive के लिए summary उसका layout, कितनी settings का ऑडिट होता है और नीति अंतिम बार कब
लिखी गई थी, बताता है। यह सूचीबद्ध करता है कि success और failure, केवल success और केवल failure के लिए क्या
ऑडिट होता है, फिर वे subcategories जिनका ऑडिट नहीं होता, उनके मुख्य events के साथ: वे events
captured नीति में disabled हैं; इससे पहले की गतिविधि के दौरान की नीति स्थापित नहीं होती।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `SourcePath` | Host (सिस्टम के `SYSTEM` hive से), और `SECURITY` hive। |
| `Layout` | `subcategories` (Windows Vista और बाद के) या `categories` (XP और 2003); value decode न होने पर खाली। |
| `Category`, `Subcategory` | जैसे `auditpol` उन्हें नाम देता है। |
| `SubcategoryGUID`, `SubcategoryID` | Subcategory का GUID, और उसका message ID जैसा event 4719 उसे देता है (`%%12548`)। |
| `Setting` | `none`, `success`, `failure` या `success_and_failure`। |
| `Success`, `Failure` | क्या successes और failures दर्ज होते हैं। |
| `AuditingEnabled` | Windows XP और 2003: क्या auditing चालू है भी या नहीं। |
| `Events` | Subcategory के मुख्य event IDs। |
| `KeyLastWrittenUTC` | `PolAdtEv` का last write: लगभग जब नीति अंतिम बार बदली। |
| `DataRaw` | Value के bytes (hex), जब वह पूरा decode नहीं होता। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का context। |
| `Problems` | अज्ञात layout या आकार, ऐसी गिनतियाँ जो मेल नहीं खातीं, और 3 से अधिक values। |

## जाँच की रणनीति { #investigation-strategy }

1. यह निष्कर्ष निकालने से पहले कि कुछ नहीं हुआ, जाँचें कि उसकी subcategory का ऑडिट हो रहा था या नहीं:
   Process Creation (4688), Logon (4624, 4625), Special Logon (4672), Other Object Access Events
   (scheduled tasks, 4698), Security System Extension (service installs, 4697), User Account और
   Security Group Management (4720, 4732), File Share (5140), Kerberos (4768, 4769)।
2. नीति की तुलना Security log से करें: उन subcategories के events जिनका अब ऑडिट नहीं होता, का
   अर्थ है कि नीति बदली थी। Event 4719 और हाल के `KeyLastWrittenUTC` को खोजें।
3. Domain-wide नीति को स्थानीय बदलाव से अलग करने के लिए Group Policy के `audit.csv` से तुलना करें।
4. `PolAdtEv` के पुराने versions के लिए `parse-registry read --recover-deleted` और transaction logs का
   उपयोग करें।

## अधिक जानकारी { #further-details }

- [Microsoft: Advanced security audit policy settings](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: Auditing constants (subcategory GUIDs)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
