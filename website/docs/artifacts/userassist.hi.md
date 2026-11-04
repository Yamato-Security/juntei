# UserAssist { #userassist }

## UserAssist क्या है { #what-userassist-is }

Windows Explorer उन प्रोग्रामों और शॉर्टकट को गिनता है जिन्हें उपयोगकर्ता Explorer shell से शुरू करता है:
Start menu, taskbar, desktop, और फ़ोल्डरों में double-click। यह गिनती
उपयोगकर्ता के `NTUSER.DAT` में रखता है। हर item में होता है:

- एक run count;
- Windows 7 और बाद के versions पर, कितनी बार और कितनी देर (milliseconds में) उस पर focus रहा;
- यह अंतिम बार कब चला।

Value नाम ROT13-encoded होते हैं। किसी प्रोग्राम का नाम आमतौर पर किसी known folder के GUID से शुरू होता है
(`{1AC14E77-…}` `System32` है)।

UserAssist दिखाता है कि किसी विशेष उपयोगकर्ता ने interactively क्या शुरू किया, जिसे निष्पादन के अधिकांश अन्य
साक्ष्य किसी उपयोगकर्ता से नहीं जोड़ते।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> उपयोगकर्ता ने यह प्रोग्राम या शॉर्टकट
Explorer के माध्यम से इतनी बार शुरू किया, अंतिम बार इस समय। किसी अन्य तरीके से (command
line, सेवा, scheduled task, कोई अन्य प्रोग्राम) शुरू किए गए प्रोग्राम नहीं गिने जाते।

## स्थान { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | क्या रखता है |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | प्रोग्राम (`executable`), Windows 7 और बाद के |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | शॉर्टकट (`shortcut`), Windows 7 और बाद के |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | प्रोग्राम और शॉर्टकट (`active_desktop`), Windows 2000 से Vista |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | Internet toolbar (`internet_toolbar`), Windows 2000 से Vista |

अन्य GUIDs `other` के रूप में report किए जाते हैं। हर profile के लिए `NTUSER.DAT` को उसके transaction logs के साथ एकत्र करें,
और सिस्टम का `SOFTWARE` hive भी, जो उपयोगकर्ता का SID देता है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **नई profiles पहले से भरी (seeded) होती हैं।** Windows 7 और बाद के versions पर, नई profile default items के साथ शुरू होती है
  (Getting Started, Calculator, Paint, Sticky Notes, …)। उनके run counts 14 से नीचे गिनते हैं,
  और वे सभी एक ही last-run समय साझा करते हैं, लगभग profile बनने के समय। ये उपयोगकर्ता की
  गतिविधि नहीं हैं।
- **Version 3 की गिनती 5 से शुरू होती है** (Windows 2000 से Vista)। संग्रहीत count 5 से शुरू होता है, इसलिए उसमें से 5
  घटाया जाता है। 5 से कम संग्रहीत count `Problems` में report किया जाता है।
- **Paths known folder के default path का उपयोग करते हैं** (`%windir%\system32`, `%ProgramFiles%`,
  `%APPDATA%\…`)। कहीं और redirect किया गया फ़ोल्डर फिर भी अपना default दिखाता है। App IDs
  (`Microsoft.Windows.Explorer`, `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) और Control Panel
  items (`::{…}`) का कोई path नहीं होता।
- **Tracking बंद की जा सकती है** `Explorer\Advanced` `Start_TrackProgs` = 0,
  `Policies\Explorer` `NoInstrumentation` = 1, या `UserAssist\Settings` `NoLog` = 1 से। Summary
  उन hives को सूचीबद्ध करता है जहाँ यह बंद है। किसी सक्रिय profile पर खाली या missing UserAssist
  anti-forensics का संकेत हो सकता है।
- **`NoEncrypt` = 1** से Windows नाम बिना ROT13 के लिखता है। ऐसे नाम गलत decode होते हैं, इसलिए
  उस hive का हर item `Problems` में यह बताता है (और जब records stdout पर जाते हैं तो stderr पर भी)।
- **Last-run समय और counts प्रति item रखे जाते हैं,** लेकिन key का last-write समय
  (`KeyLastWrittenUTC`) केवल किसी भी item के सबसे हाल के update का समय है।
- **Run counts 0 हो सकते हैं** उन items के लिए जिन पर केवल focus रहा। Focus counts और times केवल
  version 5 (Windows 7 और बाद के) में होते हैं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Summary सबसे अंत में चलाए गए items (सबसे नए पहले) को उनके उपयोगकर्ताओं के साथ सूचीबद्ध करता है, और वे hives जहाँ
UserAssist बंद है।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, profile फ़ोल्डर का नाम, सिस्टम के `ProfileList` से उपयोगकर्ता का SID (सिस्टम के `SOFTWARE` hive के साथ), और hive। |
| `GUID`, `Kind` | `{GUID}` key और वह क्या रखती है (`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`)। |
| `NameRot13`, `Name` | Value का नाम जैसा संग्रहीत है, और decode किया हुआ। |
| `Path` | Decode किया गया नाम path के रूप में, known folder का GUID उसके default path से बदला हुआ; app IDs और shell items के लिए खाली। |
| `RunCount` | इसे Explorer से कितनी बार शुरू किया गया (version 3: संग्रहीत count में से 5 घटाकर)। |
| `FocusCount`, `FocusTimeMs` | इस पर कितनी बार focus रहा, और कितनी देर milliseconds में (version 5)। |
| `LastRunUTC` | यह अंतिम बार कब शुरू किया गया। खाली जब Windows ने 0 संग्रहीत किया, या जब संग्रहीत समय 9999 से आगे हो (तब `Problems` उसे देता है और `DataRaw` bytes रखता है)। |
| `Format`, `SessionID` | Data का layout (5: 72 bytes, 3: 16 bytes) और वह session number जिसमें यह अंतिम बार लिखा गया। |
| `KeyLastWrittenUTC` | `Count` key का last write: उसके किसी भी item का सबसे हाल का update। |
| `DataRaw` | Value के bytes (hex), जब वे decode नहीं होते। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का context। |
| `Problems` | अज्ञात आकार का data, ऐसा आकार जो key के `Version` से मेल नहीं खाता, 5 से कम version 3 count, 9999 से आगे का last run, `NoEncrypt`। |

## जाँच की रणनीति { #investigation-strategy }

1. `LastRunUTC` के अनुसार sort करें और seeded items को अलग रखें: वे profile के
   बनने के आसपास का एक ही समय साझा करते हैं।
2. उपयोगकर्ता द्वारा लिखने योग्य स्थानों (`%USERPROFILE%`, `Downloads`, `Temp`,
   `ProgramData`), removable drives और network paths में प्रोग्राम खोजें, और administration तथा remote access
   टूल भी।
3. Prefetch (run times के साथ निष्पादन), Amcache और ShimCache (उपस्थिति),
   Jump Lists और LNK फ़ाइलों (उनसे क्या खोला गया), और प्रोग्राम फ़ाइलों के लिए `$MFT` से correlate करें।
4. जिन hives में tracking बंद है, और जिन profiles का UserAssist खाली है, उनमें साफ़ किए जाने की जाँच करें।
5. हटाई गई UserAssist values के लिए `parse-registry read --recover-deleted` का उपयोग करें।

## अधिक जानकारी { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: User Assist keys](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
