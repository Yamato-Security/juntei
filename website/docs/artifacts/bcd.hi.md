# बूट कॉन्फ़िगरेशन डेटा (BCD) { #boot-configuration-data-bcd }

## BCD store क्या है { #what-the-bcd-store-is }

Windows Vista और उसके बाद के संस्करण अपना boot configuration एक registry hive, यानी BCD store में रखते हैं:
BIOS systems पर `\Boot\BCD`, और UEFI systems के EFI system partition पर `\EFI\Microsoft\Boot\BCD`।
`bcdedit` और Windows installer इसे संपादित करते हैं; boot manager हर boot पर इसे पढ़ता है।

Store में objects होते हैं, जिनमें से प्रत्येक का नाम एक GUID पर आधारित होता है:

- **Windows boot manager** (`{bootmgr}`), जिसमें default entry, display order और
  menu timeout होते हैं; UEFI systems पर firmware के boot order के साथ **firmware boot manager**
  (`{fwbootmgr}`) भी;
- प्रत्येक Windows installation (recovery environment सहित) के लिए एक **Windows boot loader**, एक
  **resume application** (hibernation) और **memory tester**;
- **inherited settings** objects (`{globalsettings}`, `{bootloadersettings}`, …), जिनकी settings
  अन्य objects inherit करते हैं;
- **device** objects (recovery environment के ramdisk options)।

प्रत्येक object में elements होते हैं: एक description, एक device और path, तथा
`recoveryenabled`, `bootstatuspolicy`, `testsigning`, `nointegritychecks` और `safeboot` जैसी settings।

दो प्रकार के हमले इन्हें बदलते हैं:

- **Ransomware** recovery बंद कर देता है ताकि system की मरम्मत न हो सके:
  `bcdedit /set {default} recoveryenabled No` और
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`। कुछ families `safeboot` सेट करती हैं ताकि
  Windows safe mode में restart हो, जहाँ सुरक्षा उत्पाद नहीं चलते।
- **Rootkits और unsigned drivers** boot-time checks को कमज़ोर करने के लिए `testsigning on` या `nointegritychecks on` का उपयोग कर सकते हैं। केवल ये settings यह सिद्ध नहीं करतीं कि कोई rootkit चला था।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> संग्रह के समय, इस boot entry में
यह setting थी। Element की key का last-write time वह समय है जब setting अंतिम बार लिखी गई थी।

## स्थान { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

किसी element का type 8 hexadecimal अंकों का होता है: उसका class (`1` हर application, `2` एक प्रकार का
application, `3` devices), उसका format (`1` device, `2` string, `3` object, `4` object list, `5`
integer, `6` boolean, `7` integer list) और उसकी संख्या। Command elements को उसी नाम से दर्शाता है
जैसे Microsoft उन्हें document करता है (`BcdLibraryBoolean_AutoRecoveryEnabled`)।

संग्रह tools अक्सर store को copy नहीं करते: EFI system partition mount नहीं होता, और `\Boot`
छिपा होता है। इसे स्पष्ट रूप से संग्रहित करें, या partition को mount करें। `-d` `BCD-Template` को भी पढ़ता है
(`HiveType` बताता है कि कौन-सा है); template वह है जिससे Windows शुरुआत करता है, न कि वह store जिससे वह boot होता है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **Settings inherit होती हैं।** एक boot loader अपने
  `BcdLibraryObjectList_InheritedObjects` में दिए objects से inherit करता है (आमतौर पर `{bootloadersettings}`, जो
  `{globalsettings}` से inherit करता है)। `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks`, `SafeBoot` और
  `BootStatusPolicy` resolved stored values हैं (अपनी, अन्यथा inherited), और `Inherited` बताता है कि कौन-सी
  किसी अन्य object से आती हैं। Inheritance अधिकतम आठ स्तरों तक follow किया जाता है, और cycles सीमित रहते हैं। कोई referenced object आंशिक store में अनुपस्थित हो सकता है। खाली fields किसी प्रभावी Windows default को स्थापित नहीं करते। Firmware और Secure Boot भी किसी stored option को प्रभावी होने से रोक सकते हैं।
- **Set न होना, बंद होना नहीं है।** खाली `RecoveryEnabled` का अर्थ है कि element मौजूद नहीं है; Windows का
  default लागू होता है।
- **समय प्रत्येक element का अलग होता है।** प्रत्येक element का key time (`Elements` में `key_last_written`) वह समय है जब वह
  अंतिम बार लिखा गया था। Windows updates और upgrades के समय कुछ elements को फिर से लिखता है; अन्य
  elements के समय से तुलना करें।
- **Devices आंशिक रूप से decode होते हैं।** GPT partition उसके partition और disk के GUIDs के रूप में दिया जाता है;
  ramdisk उसके options object के रूप में। अन्य devices केवल अपने bytes (`data_raw`) रखते हैं।
- **Undocumented elements** (`0x15000065`, `0x4…` class) का कोई नाम नहीं होता और वे अपने bytes रखते हैं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

Summary प्रत्येक store की default entry और हर test signing, integrity check, recovery,
boot status policy (कोई भी `Ignore…`) और safe mode setting को सूचीबद्ध करता है, साथ में वह समय जब इसे सेट किया गया था या वह object
जिससे यह inherit हुई है।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, hive, और `BCD` या `BCD-Template`। |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | Object का GUID, उसका type (`0x10200003`) और वह क्या है (`Windows boot loader`, `inherited settings for Windows boot loader`, …)। |
| `Description`, `Path`, `Device` | Description, application का path, और उसका device (एक GPT partition और disk, या ramdisk का options object)। |
| `IsDefault`, `DisplayOrder` | क्या Windows boot manager इसे default रूप से boot करता है, और boot menu में इसका स्थान। |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | `recoveryenabled`, `testsigning`, `nointegritychecks`, जैसा store से resolve किया गया। |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`, `Network`, `DsRepair`) और `bootstatuspolicy` (`IgnoreAllFailures`, …), संख्या के साथ। |
| `Inherited` | इनमें से कौन-सी किसी अन्य object से आती हैं, और किस object से। |
| `Elements` | हर element: उसका type, नाम, value, bytes (devices, undocumented types और decode न होने वाले डेटा के लिए) और key time। |
| `KeyLastWrittenUTC` | Object की key का अंतिम write। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का संदर्भ। |
| `Problems` | बिना type वाले objects, ऐसे elements जिनका डेटा उनके format में फिट नहीं होता। |

## जाँच की रणनीति { #investigation-strategy }

1. Default entry और हर boot loader में `TestSigning`, `NoIntegrityChecks`,
   `RecoveryEnabled = false`, कोई `Ignore…` `BootStatusPolicy` और `SafeBoot` जाँचें।
2. बदलाव का समय निर्धारित करने के लिए element times का उपयोग करें, और Prefetch,
   Amcache, process creation events (4688, Sysmon 1) और System event log में `bcdedit.exe` से correlate करें।
3. ऐसी boot entries या devices खोजें जिन्हें वहाँ नहीं होना चाहिए (कोई अन्य loader path, कोई ramdisk), और
   बदले हुए default या display order को खोजें।
4. Store की तुलना `BCD-Template` से और उसी build के किसी अन्य system से करें।

## अधिक जानकारी { #further-details }

- [Microsoft: BCD boot options reference](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (object types)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
