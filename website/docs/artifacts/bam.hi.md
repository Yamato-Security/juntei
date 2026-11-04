# BAM / DAM { #bam-and-dam }

## BAM और DAM क्या हैं { #what-bam-and-dam-are }

Background Activity Moderator (BAM) और Desktop Activity Moderator (DAM) Windows
सेवाएँ हैं (Windows 10 1709 और बाद के versions) जो यह नियंत्रित करती हैं कि ऐप्लिकेशन background में क्या कर सकते हैं।
ये `SYSTEM` hive में, प्रति उपयोगकर्ता, हर प्रोग्राम के अंतिम बार चलने का समय रखती हैं:

- प्रति उपयोगकर्ता एक key, जिसका नाम उपयोगकर्ता के SID पर होता है;
- प्रति प्रोग्राम एक value: उसका नाम उसके device पर प्रोग्राम का path होता है
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) या किसी packaged app का ID
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`);
- Value के data के पहले 8 bytes वह समय हैं जब यह अंतिम बार चला (FILETIME)।

BAM उपयोगकर्ता के अपने hive के बिना किसी प्रोग्राम को किसी उपयोगकर्ता और समय से जोड़ता है, और यह removable और
network devices से चलाए गए प्रोग्राम भी सूचीबद्ध करता है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> इस प्रोग्राम का एक process इस
उपयोगकर्ता के session में चला; यह अंतिम बार इस समय चला। BAM runs की गिनती नहीं करता, और प्रोग्राम
के हटाए जाने के बाद भी entry बनी रह सकती है।

## स्थान { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

हर उपयोगकर्ता की key में `Version` और `SequenceNumber` values भी होती हैं; ये प्रोग्राम नहीं हैं और
छोड़ दी जाती हैं। 1709 या 1803 से upgrade किए गए सिस्टम में दोनों layouts हो सकते हैं।

`SYSTEM` को उसके transaction logs के साथ एकत्र करें, और सिस्टम के `SAM` और `SOFTWARE` भी: ये
उपयोगकर्ताओं के नाम बताते हैं।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **प्रति प्रोग्राम और उपयोगकर्ता एक समय।** केवल सबसे हाल का run रखा जाता है। Prefetch, Amcache और
  event logs पहले के runs देते हैं।
- **हाल की गतिविधि।** बताया गया है कि जो प्रोग्राम कुछ समय से नहीं चले, उनकी entries Windows शुरू होने पर
  हटा दी जाती हैं, इसलिए BAM ज़्यादातर हाल की गतिविधि दिखाता है: इसमें न दिखने वाला प्रोग्राम
  बहुत पहले चला हो सकता है।
- **Device paths, drive letters नहीं।** `\Device\HarddiskVolume3` वह तीसरा volume है जिसे Windows ने
  क्रमांकित किया, `C:` नहीं। `DevicePath` और `Path` device को उस पर के path से अलग करते हैं; drive letter लिखने से पहले
  volume का मिलान `MountedDevices`, `$MFT` या अन्य आर्टिफ़ैक्ट से करें।
- **हर प्रोग्राम नहीं।** BAM और DAM उपयोगकर्ताओं के sessions में प्रोग्राम track करते हैं; सेवाएँ ज़्यादातर
  नहीं दिखतीं। `S-1-5-18` entries आमतौर पर setup और sign-in screen के processes होती हैं
  (`FirstLogonAnim.exe`), और `S-1-5-90-0-<n>` वाली Desktop Window Manager (`DWM-<n>`) की।
- **Control sets।** Default रूप से वह control set पढ़ा जाता है जिसे Windows ने अंतिम बार उपयोग किया। `--all-control-sets`
  हर एक को पढ़ता है: कोई पुराना control set ऐसी entry रख सकता है जो वर्तमान में अब नहीं है।
- **उपयोगकर्ता नाम।** Local account का नाम `SAM` से आता है (computer का SID और
  account का RID); domain account का नाम `ProfileList` में उसके profile फ़ोल्डर से। `UserSource`
  बताता है कि कौन-सा (`sam`, `profile_list`, `well_known`)। इनके बिना केवल SID दिया जाता है। ऐसा RID
  जो अब `SAM` में नहीं है (हटाया गया account) बिना नाम के रहता है।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Summary हर उपयोगकर्ता को उसके SID, नाम और entries की संख्या के साथ सूचीबद्ध करता है, और अंतिम बार चलाए गए
प्रोग्राम (सबसे नए पहले)।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `SourcePath` | Host, और `SYSTEM` hive। |
| `ControlSet`, `IsCurrentControlSet` | वह control set जिसमें entry है, और क्या यह वही है जिसे Windows ने अंतिम बार उपयोग किया। |
| `Moderator` | `bam` या `dam`। |
| `UserSID`, `User`, `UserSource` | वह SID जिस पर key का नाम है, उपयोगकर्ता का नाम, और वह कहाँ से आता है (`sam`, `profile_list`, `well_known`)। |
| `Program` | Value का नाम जैसा Windows ने लिखा: device path या packaged app का ID। |
| `DevicePath`, `Path` | Device path के लिए, device (`\Device\HarddiskVolume3`) और उस पर का path (`\Windows\System32\cmd.exe`); packaged app के लिए खाली। |
| `LastRunUTC` | प्रोग्राम अंतिम बार कब चला (100 ns तक)। |
| `KeyPath`, `KeyLastWrittenUTC` | उपयोगकर्ता की key और उसका last write: उसकी किसी भी entry का सबसे हाल का update। |
| `DataRaw` | Value के bytes (hex), जब उनमें समय न हो: `REG_BINARY` नहीं, या 8 bytes से कम। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का context। |
| `Problems` | ऐसा data जिसमें समय नहीं है, और ऐसी values जिन्हें पढ़ा नहीं जा सकता। |

## जाँच की रणनीति { #investigation-strategy }

1. `LastRunUTC` के अनुसार sort करें और उपयोगकर्ता द्वारा लिखने योग्य स्थानों (`\Users\`, `Downloads`,
   `Temp`, `ProgramData`) में, system volume के अलावा अन्य devices पर, और administration तथा
   remote access टूल वाले प्रोग्राम देखें।
2. जाँचें कि उन्हें किस उपयोगकर्ता ने चलाया, और ऐसे accounts जो किसी वर्तमान profile से मेल नहीं खाते।
3. Prefetch (run counts और पहले के runs), UserAssist (Explorer से शुरुआत), Amcache
   और ShimCache (उपस्थिति), `$MFT` (प्रोग्राम की फ़ाइल) और event logs से correlate करें।
4. हटाई गई BAM values के लिए `--all-control-sets` और `parse-registry read --recover-deleted` का उपयोग करें।

## अधिक जानकारी { #further-details }

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: Security identifiers (well-known SIDs)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

सीमा से बाहर के last-run FILETIMEs `LastRunUTC` को खाली छोड़ते हैं और मूल value को `DataRaw` में सुरक्षित रखते हैं, साथ में `Problems` में स्पष्टीकरण होता है। Recent-activity summary हर host की पहचान करता है और अलग-अलग hive/उपयोगकर्ता अवलोकनों को अलग रखता है।
