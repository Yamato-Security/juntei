# MuiCache { #muicache }

## MuiCache क्या है { #what-muicache-is }

जब कोई उपयोगकर्ता कोई program शुरू करता है, तो Windows shell में दिखाए जाने वाले नाम के लिए program के version resources
पढ़ता है (`FileDescription`, उदाहरण के लिए `cmd.exe` के लिए "Windows Command Processor")
और उसे प्रति उपयोगकर्ता `MuiCache` में cache करता है। Windows 8 और बाद के संस्करण company
(`CompanyName`) भी cache करते हैं। Cache की key program का पूरा path होता है:

- **Windows 8 और बाद के:** प्रति program दो values, `<path>.FriendlyAppName` और
  `<path>.AppCompany`।
- **Windows Vista और 7:** प्रति program एक value, `<path>`, जिसमें friendly name होता है।
- **Windows XP:** वही रूप, `NTUSER.DAT` में।

MuiCache किसी उपयोगकर्ता के shell session के लिए program-निष्पादन के सुराग दे सकता है, जिनमें removable drives,
network shares और अब मौजूद न रहे folders पर के programs भी शामिल हैं, उस नाम और company के साथ
जो फ़ाइल ने स्वयं दिए थे।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> इस shell profile ने किसी फ़ाइल का path,
विवरण और company cache किया। इसे अन्य निष्पादन artifacts से correlate करें। Profile उस account की पहचान
नहीं करता जिसके रूप में process चला। MuiCache में प्रति program कोई समय और कोई run count नहीं होता।

## स्थान { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (cache की गई strings की भाषा) कोई program नहीं है और छोड़ दिया जाता है। `@<file>,-<number>`
(`@shell32.dll,-22019`) नाम वाली values वे resource strings हैं जिन्हें shell ने अपने
menus और file types के लिए cache किया, programs नहीं; इन्हें छोड़ दिया जाता है और summary में गिना जाता है। Windows Vista
और बाद के संस्करणों पर `NTUSER.DAT` में कोई MuiCache नहीं होता।

हर profile के लिए `UsrClass.dat` (और `NTUSER.DAT`) को उनके transaction logs के साथ, और host तथा
उपयोगकर्ता नामों के लिए सिस्टम के `SYSTEM` और `SOFTWARE` hives एकत्र करें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **Profile का owner ज़रूरी नहीं कि process का account हो।** किसी अन्य account से शुरू किया गया
  program इस उपयोगकर्ता के session में window दिखा सकता है और इस उपयोगकर्ता के cache में metadata छोड़ सकता है।
  `User` और `UserSid` hive के owner की पहचान करते हैं, process के security context की नहीं।
- **प्रति program कोई समय नहीं।** `KeyLastWrittenUTC` key का last write है: उसके किसी भी program में हुआ
  सबसे हालिया बदलाव, न कि कोई विशेष program कब चला। Prefetch, BAM, UserAssist और event logs
  समय देते हैं।
- **केवल वे programs नहीं जिन्हें उपयोगकर्ता ने हाथ से शुरू किया।** Shell द्वारा स्वयं load की गई कोई DLL या program
  (`Open with`, किसी shell extension, किसी installer के child के लिए) दिखाई दे सकता है, जैसे `explorerframe.dll` या
  `shell32.dll`।
- **नाम फ़ाइल से आता है।** `FriendlyAppName` और `AppCompany` वही हैं जो फ़ाइल के
  version resources ने चलते समय बताया। Malware कोई भी विवरण और company होने का दावा कर सकता है; rename किया गया
  tool अपना मूल विवरण बनाए रखता है, जो उसे उजागर कर सकता है (`svchost.exe` नाम की फ़ाइल जिसका
  friendly name कोई remote access tool है)।
- **प्रति path एक entry.** उसी program को फिर से चलाने पर entry नहीं जुड़ती; किसी अन्य path पर
  रखी copy से जुड़ती है। फ़ाइल हटाए जाने के बाद भी program की entry बनी रह सकती है।
- **एक path के लिए दोनों रूप.** जब किसी path का friendly name दोनों रूपों में हो (upgrade किया गया
  profile), तो हर एक को अपना record मिलता है, ताकि कोई value खो न जाए।
- **क्षतिग्रस्त values.** जो value string नहीं है, वैध UTF-16 नहीं है या पूरी पढ़ी नहीं जा सकती, वह अपने
  bytes (`FriendlyAppNameRaw`, `AppCompanyRaw`) बनाए रखती है और `Problems` में report की जाती है।
  अमान्य UTF-16 दिखाए गए text में replacement characters भी उत्पन्न कर सकता है; मूल bytes के लिए raw
  column का उपयोग करें।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Summary Windows और Program Files folders के बाहर के programs (user folders,
अन्य drives, network shares) उनके उपयोगकर्ताओं के साथ, और छोड़ी गई resource strings की संख्या सूचीबद्ध करती है।
अन्य नामों वाले Program Files folders (अन्य भाषाओं के Windows पर) भी सूचीबद्ध होते हैं।
Relative paths और environment-variable folder से बाहर निकलने वाले paths सूचीबद्ध रहते हैं। साधारण
drive paths की जाँच परीक्षक के file system तक पहुँचे बिना `.` और `..` घटकों को resolve करने के बाद
की जाती है।

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, profile folder का नाम, उपयोगकर्ता का SID (`UsrClass.dat` की root key इसका नाम देती है; अन्यथा सिस्टम की `ProfileList`), और hive। |
| `HiveType` | `UsrClass.dat` या `NTUSER.DAT`। |
| `Path` | Program का path: `.FriendlyAppName` या `.AppCompany` के बिना value का नाम। |
| `FriendlyAppName` | Program का विवरण, `<path>.FriendlyAppName` या `<path>` से। |
| `AppCompany` | Program की company, `<path>.AppCompany` से (Windows 8 और बाद के)। |
| `KeyPath`, `KeyLastWrittenUTC` | MuiCache key, और उसका last write: उसके किसी भी program में हुआ सबसे हालिया बदलाव। |
| `FriendlyAppNameRaw`, `AppCompanyRaw` | Value के bytes (hex), जब वे साफ़ decode होने वाला text न हों। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का संदर्भ: `replayed` जब logs ने program की कोई value लिखी। |
| `Problems` | ऐसी values जो strings नहीं हैं, वैध UTF-16 नहीं हैं या पढ़ी नहीं जा सकतीं। |

## जाँच की रणनीति { #investigation-strategy }

1. User-writable स्थानों (`\Users\`, `Downloads`, `Temp`, `ProgramData`), अन्य drives और
   network shares पर के programs, तथा administration, remote access और archiving tools देखें।
2. File name की तुलना उसके friendly name और company से करें: mismatch rename किए गए
   tool या masquerading फ़ाइल को उजागर कर सकता है।
3. Prefetch, BAM और UserAssist (यह कब चला), Amcache और ShimCache (फ़ाइल का
   hash और उपस्थिति), और `$MFT` (क्या फ़ाइल अब भी मौजूद है) से correlate करें।
4. हटाई गई MuiCache values के लिए `UsrClass.dat` पर `parse-registry read --recover-deleted` का उपयोग करें।

## अधिक जानकारी { #further-details }

- [Microsoft: VERSIONINFO resource](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage: MuiCache प्रयोग, GUI sessions, और process-account की सीमाएँ](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
