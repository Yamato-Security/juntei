# इंस्टॉल किए गए प्रोग्राम { #installed-programs }

## Windows क्या रखता है { #what-windows-keeps }

Windows और installers install किए गए software को कई registry सूचियों में दर्ज करते हैं। हर सूची में प्रति
program, product, distribution या package एक key होती है; Juntei प्रति key एक record देता है।

| स्रोत | Hive और key | इसमें क्या होता है |
|---|---|---|
| `uninstall` | `SOFTWARE` और `NTUSER.DAT` `…\CurrentVersion\Uninstall\*`, और `SOFTWARE\Wow6432Node\…` | Settings और *Programs and Features* जो सूचीबद्ध करते हैं: नाम, version, publisher, install तिथि, folders, uninstall commands, आकार, क्या यह छिपा हुआ है |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | Windows Installer products: product का नाम और version, package का file name और वह folder जहाँ से इसे install किया गया |
| `app_path` | `SOFTWARE` और `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | वे programs जिन्हें Windows नाम से शुरू करता है (`Win+R`, `start tool`): program का path और उसका folder |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | Windows Subsystem for Linux distributions: नाम, उसकी फ़ाइलें या disk कहाँ हैं, WSL 1 या 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | Packaged (Store) apps: हर उपयोगकर्ता के लिए provisioned, Windows के अपने, staged, deprovisioned, और हर उपयोगकर्ता के |

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> यह program, product या package
सिस्टम पर, हर उपयोगकर्ता के लिए या इस उपयोगकर्ता के लिए, install (या register) किया गया था। Install तिथि वह है
जो installer ने लिखी; key का last write वह समय है जब entry आख़िरी बार बदली।

## स्थान { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

`SOFTWARE`, हर `NTUSER.DAT` और उनके transaction logs एकत्र करें। `SYSTEM` host का नाम बताता है, और
`SOFTWARE` की `ProfileList` हर उपयोगकर्ता का SID देती है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **Install तिथि और key का समय.** `InstallDate` installer द्वारा local time में लिखी गई एक तिथि (समय नहीं) है;
  कुछ installers इसे नहीं लिखते, और कुछ दूसरा format लिखते हैं (जो `InstallDateRaw` में
  लिखे अनुसार रखा जाता है)। कोई update तिथि बदले बिना entry को फिर से लिख सकता है, या program में ज़्यादा बदलाव
  हुए बिना तिथि बदल सकता है: `InstallDate` की तुलना `KeyLastWrittenUTC` से करें।
- **Uninstall किया गया software.** साफ़ तरीके से हटाया गया program अपनी keys खो देता है। हटाई गई `Uninstall` keys के लिए
  `parse-registry read --recover-deleted` का उपयोग करें, और पीछे छूटे निशानों के लिए Amcache, Prefetch और `$MFT` का।
- **Per-user registrations.** `NTUSER.DAT` entries किसी profile के लिए register किए गए software की पहचान करती हैं।
  यह scope यह स्थापित नहीं करता कि installer किसने चलाया या administrator अधिकारों का उपयोग हुआ या नहीं।
- **छिपी हुई entries.** `SystemComponent` किसी entry को *Programs and Features* से छिपाता है;
  `DisplayName` के बिना वाली entries (updates, components) भी नहीं दिखाई जातीं। ये सभी report की जाती हैं।
- **Product codes.** GUID के आकार वाली `Uninstall` key को `ProductCode` के माध्यम से packed
  `msi_product` key से मिलाया जा सकता है। केवल GUID के आकार का नाम यह सिद्ध नहीं करता कि
  entry Windows Installer ने बनाई।
- **App Paths.** ये programs का पता लगाने के लिए उपयोग किए जाने वाले registrations हैं, निष्पादन
  records नहीं। `Path` value में कई search directories हो सकती हैं और यह ज़रूरी नहीं कि
  installation directory हो।
- **Publisher.** यह installer द्वारा दिया गया registry metadata है, सत्यापित signature
  जानकारी नहीं।
- **Architecture.** `Wow6432Node` 64-bit Windows पर 32-bit programs रखता है (`Architecture` `x86`,
  `Wow64` `True`); अन्य entries सिस्टम के अपने architecture की होती हैं (खाली)। किसी package का
  architecture उसका अपना होता है (`x64`, `neutral`, …)।
- **WSL.** `Version` distribution के `Flags` से लिया गया WSL version है (`0x8`: यह WSL 2 की
  virtual machine में चलता है)। Distribution का अपना `Version` value (`Details` में) उसके file system का
  version है, जो WSL 1 distributions पर भी 2 होता है। `BasePath` में उसका `ext4.vhdx` (WSL 2) या `rootfs`
  (WSL 1) होता है: अंदर देखने के लिए इसे एकत्र करें।
- **Packages.** `<SID>` सूचियाँ हर उपयोगकर्ता के लिए register किए गए packages हैं; अधिकांश Windows के
  अपने होते हैं। `Deprovisioned` packages नए उपयोगकर्ताओं के लिए हटाए गए थे, और `EndOfLife\<SID>` वाले उस उपयोगकर्ता के लिए।
- **क्षतिग्रस्त values.** विकृत numbers, ग़लत value types और अमान्य UTF-16 `Problems` में report किए जाते हैं
  और `DataRaw` में रखे जाते हैं, जिसमें शून्य bytes वाली मौजूद value के लिए खाली hex string भी शामिल है।
  वैकल्पिक numeric field में साफ़ खाली string को अनुपस्थित माना जाता है। Package
  version में चार decimal घटक होने चाहिए, हर एक unsigned 16-bit सीमा के भीतर।
  जब `-t` records को stdout पर stream करता है, तो hive-स्तर की चेतावनियाँ stderr पर रहती हैं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

Summary सबसे अंत में install किए गए programs (`InstallDate` के अनुसार), उपयोगकर्ताओं ने अपने लिए
क्या install किया, और WSL distributions सूचीबद्ध करती है।

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, hive फ़ाइल, और `SOFTWARE` या `NTUSER.DAT`। |
| `User`, `UserSID` | User hive का उपयोगकर्ता (उसका profile folder और `ProfileList` SID), या वह उपयोगकर्ता जिसके लिए package सूचीबद्ध है। |
| `Source`, `Scope` | सूची (`uninstall`, `msi_product`, `app_path`, `wsl`, `package`), और `machine` या `user`। |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher`; किसी product का `ProductName` और `Version`; किसी App Path के program का file name; किसी distribution का नाम और WSL version; किसी package का नाम और version। |
| `InstallDate`, `InstallDateRaw` | `YYYYMMDD` के रूप में लिखे जाने पर तिथि के रूप में `InstallDate`; अन्यथा जैसा लिखा गया है। |
| `KeyLastWrittenUTC` | Entry की key आख़िरी बार कब बदली। |
| `InstallLocation`, `InstallSource` | `InstallLocation` (किसी App Path का `Path`, किसी distribution का `BasePath`, किसी package का `Path`); `InstallSource` (किसी product का अंतिम उपयोग किया गया source folder)। |
| `Program` | किसी App Path का program। |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`। |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | KiB में `EstimatedSize`; क्या यह छिपा हुआ है; क्या इसे Windows Installer ने install किया। |
| `ProductCode` | Windows Installer product code (`{GUID}`)। |
| `Architecture`, `WOW64` | `Wow6432Node` के अंतर्गत `x86`, या किसी package का architecture; क्या key `Wow6432Node` के अंतर्गत है। |
| `Details` | अतिरिक्त: `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User` (वह account जिसने Inno Setup installer चलाया); किसी product का `PackageCode`, `PackageName`; किसी distribution का `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName`, `Default`; किसी package का `List`, `ResourceId`, `PublisherId`। |
| `KeyPath` | Entry की key। |
| `DataRaw` | उन values के bytes (hex) जो वैसी नहीं हैं जैसा उनके नाम बताते हैं। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का संदर्भ। |
| `Problems` | ऐसी values जिन्हें पढ़ा नहीं जा सका या जो वैसी नहीं हैं जैसा उनके नाम बताते हैं। |

## जाँच की रणनीति { #investigation-strategy }

1. घटना के आसपास `InstallDate` और `KeyLastWrittenUTC` के अनुसार sort करें; remote access
   tools, tunnelling और file transfer clients, archivers, scripting runtimes और security tools देखें।
2. `Scope` `user` records की समीक्षा करें: administrator अधिकारों के बिना install किया गया software।
3. User-writable folders (`Downloads`, `AppData`, `Temp`, `ProgramData`) के लिए `InstallLocation`, `InstallSource` और `UninstallCommand`
   जाँचें।
4. WSL distributions खोजें, और उनका `BasePath` एकत्र करें।
5. Amcache और Prefetch (installer और चलता हुआ program), `$MFT` (install folder का
   निर्माण) और Application event log (MsiInstaller events) से correlate करें।

## अधिक जानकारी { #further-details }

- [Microsoft: Uninstall registry key](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: App registration (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: Package identity overview](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
