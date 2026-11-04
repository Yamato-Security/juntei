# USB डिवाइस { #usb-devices }

## USB history क्या है { #what-the-usb-history-is }

Windows अपने द्वारा install किए गए हर device को दर्ज करता है। USB storage, अन्य USB devices और portable
devices (phones, cameras, media players) के लिए, `SYSTEM` hive प्रति device एक key रखता है जिसमें उसका नाम,
serial number और वे times होते हैं जब Windows ने उसे install किया, अंतिम बार उसे आते देखा और अंतिम बार उसे हटाते देखा।
`MountedDevices` उनके volumes को दिए गए volume GUIDs और drive letters दर्ज करता है।
`SOFTWARE` hive वे नाम रखता है जो Explorer ने portable devices और volumes को दिए, और volume serial numbers का ReadyBoost
record भी। हर उपयोगकर्ता का `NTUSER.DAT` उन volumes को दर्ज करता है जिन्हें उस उपयोगकर्ता ने
mount किया था।

`parse-registry usb` इन्हें प्रति device और volume एक record में जोड़ता है: वह कौन-सा device था, उसे कब
install और connect किया गया, उसके पास कौन-सा drive letter और volume था, और किन उपयोगकर्ताओं ने वह
volume mount किया था।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> इस vendor, product और
serial number वाला device इस सिस्टम से connect किया गया था, पहली और अंतिम बार इन times पर; उसके volume का यह
drive letter था; इन उपयोगकर्ताओं ने उसे mount किया था। Registry records यह नहीं दिखाते कि कौन-सी फ़ाइलें copy की गईं।

## स्थान { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| Device property | Field | अर्थ (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: उसका driver अंतिम बार कब install हुआ |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: device पहली बार कब install हुआ |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: यह अंतिम बार कब connect हुआ (Windows 8 और बाद के) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: यह अंतिम बार कब हटाया गया (Windows 8 और बाद के) |

Windows 8 और बाद के versions हर समय को `0064` … `0067` key की default value के रूप में रखते हैं। Windows 7
इसे `00000064\00000000\Data` में रखता है। Juntei दोनों पढ़ता है।

कई DFIR संदर्भ `0064` को first install और `0065` को install के रूप में label करते हैं। Juntei ऊपर दी गई
Microsoft की `devpkey.h` परिभाषाओं का पालन करता है, और columns का नाम इस पर रखता है कि हर समय क्या दर्ज करता है।

`-f` या `-d` को `SYSTEM` hive दें। सिस्टम का `SOFTWARE` hive और उसके उपयोगकर्ताओं के `NTUSER.DAT`
hives उसके पास पाए जाते हैं, जैसे कोई संग्रह उन्हें रखता है (`Windows\System32\config\SOFTWARE`,
`Users\<user>\NTUSER.DAT`), या उसी directory में और एक level नीचे, जब hives एक साथ export किए गए हों।
इन सभी को उनके transaction logs के साथ एकत्र करें।

## Records कैसे जोड़े जाते हैं { #how-the-records-are-joined }

- **Devices:** प्रति device key एक record: `usbstor`, `usb` और `wpdbusenum` (`DeviceClass`)। कोई
  USB storage device अपने vendor और product IDs उसी container ID (अन्यथा उसी serial number) वाले
  USB device से लेता है।
- **Volumes:** किसी USB storage device के volumes `MountedDevices` की वे `\??\Volume{GUID}` values हैं
  जिनका data ऐसा path है जो device का नाम लेता है। दो volumes वाले device के दो
  records होते हैं। उसके drive letters समान data वाली `\DosDevices\X:` values हैं।
- **नाम और serial numbers:** volume नाम `Windows Portable Devices` से और उन
  portable devices से आते हैं जो उसी device का नाम लेते हैं। Volume labels और serial numbers उन
  `EMDMgmt` keys से आते हैं जो उसका नाम लेती हैं।
- **उपयोगकर्ता:** वे उपयोगकर्ता जिनके `MountPoints2` में volume का GUID है, उस key के last-write समय के साथ।
- **Device key के बिना volumes** (`volume`): ऐसा `MountedDevices` volume जिसका नाम कोई device key नहीं लेती (किसी
  internal disk का, या हटाए गए device का जिसकी `Enum` key मिट चुकी है), ऐसा drive letter जिसके साथ कोई volume GUID
  value data साझा नहीं करती, और ऐसा volume जो केवल उपयोगकर्ताओं के `MountPoints2` में है।
- **ऐसे devices जिनका नाम केवल `SOFTWARE` लेता है** (`portable_device`, `readyboost`)।

`SourceKeys` हर वह key और value सूचीबद्ध करता है जिनसे कोई record जोड़ा गया, उसके hive के साथ।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **Drive letters फिर से उपयोग होते हैं।** `MountedDevices` हर letter दिए गए अंतिम device को रखता है।
  Letter केवल उस device पर दिखाया जाता है जिसका data अभी भी उस letter का data है।
- **Serial numbers गढ़े जा सकते हैं।** जब किसी instance का दूसरा character `&` होता है, तो Windows ने उसे
  गढ़ा है क्योंकि device कोई serial number report नहीं करता; तब `Serial` खाली होता है। ऐसा
  instance किसी अन्य computer पर अलग हो सकता है।
- **USB storage instances logical unit पर समाप्त होते हैं** (`&0`)। `Serial` इसे छोड़ देता है; `Instance`
  इसे रखता है।
- **Times Windows version के अनुसार भिन्न होते हैं।** Windows 7 में केवल install times होते हैं। `LastArrivalUTC` और
  `LastRemovalUTC` केवल अंतिम connection हैं, हर connection नहीं। Event logs
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`) और
  `setupapi.dev.log` अधिक जानकारी देते हैं।
- **Control sets:** default रूप से वह control set पढ़ा जाता है जिसे Windows ने अंतिम बार उपयोग किया। `--all-control-sets`
  हर control set पढ़ता है; कोई पुराना control set अभी भी ऐसा device सूचीबद्ध कर सकता है जो वर्तमान में अब नहीं है।
- **किसी key का last-write समय connection का समय नहीं है।** Windows कई कारणों से device keys फिर से
  लिखता है।
- **हटाए गए devices निशान छोड़ते हैं।** Vendor और product वाला लेकिन device key के बिना volume record
  ऐसा device है जिसकी `Enum` key हटा दी गई थी (uninstall या cleaning टूल द्वारा) लेकिन जिसका volume
  `MountedDevices` में अभी भी है।
- **ReadyBoost serial numbers** volume का serial number हैं (`VolumeSerials`, जैसा `dir` उसे
  दिखाता है), जिसे LNK फ़ाइलें और Jump Lists भी दर्ज करती हैं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

Summary सबसे अंत में आए USB storage devices (सबसे नए पहले) को उनके drive
letters के साथ सूचीबद्ध करता है, और बताता है कि device key के बिना कितने volumes USB storage devices के हैं।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host` | Computer का नाम, `SYSTEM` से। |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum`, या device key के बिना, `volume`, `portable_device` या `readyboost`। |
| `ControlSet` | Device key का control set। |
| `Vendor`, `Product`, `Revision` | USB storage: device के नाम (`Disk&Ven_…&Prod_…&Rev_…`) से, उस device के लिए भी जिसका नाम कोई volume या `SOFTWARE` key लेती है। |
| `Vid`, `PID` | USB vendor और product IDs। |
| `Serial`, `Instance` | Serial number, और लिखे अनुसार instance key का नाम। |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | Device key की values, जैसी लिखी गईं (`DeviceDesc` अक्सर `@<inf>,%…%;<text>` होता है)। किसी portable device का `FriendlyName` आमतौर पर उसके volume का label या drive होता है। |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | Device property times (ऊपर देखें)। |
| `DriveLetters` | वे drive letters जिनका `MountedDevices` data volume का data है। |
| `VolumeGUID`, `VolumeData` | Volume का GUID, और उसका `MountedDevices` data बताता है कि वह किस पर है: device path, MBR disk signature और partition offset, या GPT partition। |
| `VolumeNames`, `VolumeSerials` | Explorer द्वारा device या उसके volume को दिए गए नाम, ReadyBoost labels, और volume serial numbers (`1234-ABCD`)। |
| `User` | वे उपयोगकर्ता जिनके `MountPoints2` में volume है, उस key के last write के साथ (JSON: `User`, `MountPoints2LastWritten`, `SourcePath`)। |
| `FoundIn`, `SourceKeys` | यह कहाँ मिला (`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`), और हर वह key और value जिससे इसे जोड़ा गया। |
| `KeyPath`, `KeyLastWrittenUTC` | Device की instance key और उसका last write। |
| `SourcePath`, `SoftwarePath` | `SYSTEM` hive, और `SOFTWARE` hive जब उसने कुछ दिया हो। |
| `TransactionLogStatus`, `LogsApplied` | उन hives का registry recovery context जिनसे record जोड़ा गया। |
| `Problems` | ऐसी keys जिन्हें पढ़ा नहीं जा सका, और ऐसी device properties जो FILETIMEs नहीं हैं (उनके bytes के साथ)। |

## जाँच की रणनीति { #investigation-strategy }

1. USB storage devices को `LastArrivalUTC` (अन्यथा `DriverLastInstalledUTC`) के अनुसार sort करें, और
   computer के अपने readers और hubs को अलग रखें।
2. हर device का vendor, product, serial number, volume GUID, drive letter और volume serial
   number नोट करें, और उन्हें LNK फ़ाइलों, Jump Lists, ShellBags और `$MFT` में खोजें (उस
   drive letter या volume पर के paths)।
3. `User` में देखें कि volume किसने mount किया था, और कब (`MountPoints2` key का last write)।
4. ऐसे `volume` records खोजें जो किसी USB storage device का नाम लेते हैं: ऐसे devices जिनकी keys हटा दी गईं।
5. हर connection के लिए event logs से correlate करें, केवल अंतिम के लिए नहीं।
6. हटाई गई device keys के लिए `parse-registry read --recover-deleted` का उपयोग करें।

## अधिक जानकारी { #further-details }

- [Microsoft: Device properties (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: Supporting mount manager requests](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (Wine के headers के अनुसार)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): device property numbers
- [libyal winreg-kb: USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md),
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) और
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

User joins हर hive की स्रोत पहचान का उपयोग करते हैं, इसलिए एक ही profile नाम वाले export किए गए hives अलग रहते हैं। `SourceKeys` योगदान देने वाले user-hive paths को रखता है। Companion-user discovery symbolic links को छोड़ देता है, जो recursive hive discovery के अनुरूप है। VID/PID correlation के लिए मेल खाने वाला उपयोग योग्य container ID आवश्यक है; उसके बिना, serial matching से अस्पष्टता-रहित IDs मिलने चाहिए। परस्पर विरोधी candidates एक problem के साथ अज्ञात रहते हैं।
