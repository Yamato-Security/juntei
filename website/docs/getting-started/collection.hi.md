# साक्ष्य संग्रह { #collecting-evidence }

Juntei Windows से पहले ही एकत्र की गई files को parse करता है। भरोसेमंद acquisition tool का उपयोग करें और जहाँ संभव हो मूल metadata सुरक्षित रखें। VHDX disk images को extract किए बिना parse किया जा सकता है: [डिस्क इमेज माउंट करना](../commands/mount.md) देखें।

## सामान्य स्थान { #common-locations }

| आर्टिफ़ैक्ट | सामान्य स्थान |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | NTFS volume metadata file `C:\$MFT` |
| `$LogFile` | NTFS volume metadata file `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| USN Journal | NTFS metadata stream `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| User Access Logging (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| वेब ब्राउज़र | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\` और अन्य browsers के profile folders |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` और अन्य उपयोगकर्ता/system स्थान |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` और `CustomDestinations` |
| ShimCache | `SYSTEM` hive: `...\Session Manager\AppCompatCache\AppCompatCache` |
| Defender quarantine | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| Machine registry | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| User registry | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## Files जिन्हें साथ रखना आवश्यक है { #files-that-must-travel-together }

- Registry hives को साथ की `.LOG`, `.LOG1` और `.LOG2` files के साथ संग्रहित करें। Hive dirty होने पर Juntei
  registry commands और ShimCache के लिए logs को replay कर सकता है।
- SRUM का `SRUDB.dat` सुसंगत रूप से संग्रहित करें। Juntei वर्तमान में dirty ESE database की रिपोर्ट करता है, लेकिन
  `SRU*.log` को replay नहीं करता; केवल उन logs में मौजूद records उपलब्ध नहीं होते।
- जहाँ संभव हो, `$MFT` को `$Extend/$J` के साथ संग्रहित करें। USN record parent file references संग्रहीत करता है,
  पूर्ण path नहीं; MFT path resolution को बेहतर बनाता है।
- `$LogFile` को उसी volume के `$MFT` के साथ, समय में यथासंभव निकट, संग्रहित करें। `$LogFile` तेज़ी से
  wrap होता है; बाद में संग्रह करने पर पहले के redo/undo records पूरी तरह खो सकते हैं।
- Account और Wi-Fi profile नामों को resolve करने के लिए SRUM को मेल खाने वाले `SOFTWARE` hive के साथ संग्रहित करें।
- पूरी User Access Logging `Sum` directory संग्रहित करें: `SystemIdentity.mdb` roles के नाम देता है और वार्षिक
  `{GUID}.mdb` databases को सूचीबद्ध करता है। SRUM की तरह, Juntei UAL के `Svc*.log` transaction logs को replay
  नहीं करता।
- Browser databases को उनके बगल वाली `-wal` files (`places.sqlite-wal`, `cookies.sqlite-wal`) के साथ
  संग्रहित करें: सबसे नए बदलाव, और वे पुराने page versions जिनसे deleted entries recover की जाती हैं, केवल
  तब तक वहाँ रहते हैं जब तक browser checkpoint नहीं करता।
- पूरा Defender `Quarantine` tree संग्रहित करें ताकि Entries, ResourceData और Resources का मिलान किया जा सके।

## Live system के लिए सावधानियाँ { #live-system-cautions }

Copy के दौरान खुली files बदल सकती हैं। Registry hives और ESE databases आंतरिक रूप से dirty हो सकते हैं।
ShimCache अपनी सबसे नई in-memory entries को shutdown तक flush नहीं कर सकता। Volume snapshots या raw access का उपयोग करने वाला
forensic collection tool सामान्य file copy से बेहतर है।

## सुरक्षा { #safety }

साक्ष्य में malware, exploit samples, दुर्भावनापूर्ण LNK files और spreadsheet-formula text हो सकते हैं।
एक पृथक system पर विश्लेषण करें, recover किए गए streams को execute न करें, और `--extract` का उपयोग केवल तभी करें जब आप
खतरनाक content को संभालने का इरादा रखते हों। अछूते source को अलग से सुरक्षित रखें।
