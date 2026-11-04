# बूट सेक्टर (`$Boot`) { #boot-sector-boot }

## `$Boot` क्या है { #what-boot-is }

`$Boot` हर NTFS volume की MFT entry 7 है: volume के पहले 8 KiB। इसका पहला sector,
**volume boot record**, BIOS parameter block रखता है जो file system का वर्णन करता है:

- sector और cluster आकार, volume में sectors की संख्या, और **hidden sectors**
  (disk पर volume से पहले के sectors, यानी जहाँ उसका partition शुरू होता है);
- `$MFT` और `$MFTMirr` कहाँ शुरू होते हैं, और FILE तथा INDX records के आकार;
- 64-bit **volume serial number**, जो volume format होने पर set होता है।

पहले sector का बाकी हिस्सा और उसके बाद के 15 sectors **boot code** रखते हैं जो
Windows को load करता है।

Serial number के निचले 32 bits वह हैं जिसे Windows `XXXX-XXXX` (`vol`, `dir`) के रूप में दिखाता है और जो
[LNK शॉर्टकट](lnk.md), [Jump Lists](jump-lists.md) और [Prefetch](prefetch.md) उस volume के बारे में दर्ज करते हैं
जिस पर कोई फ़ाइल थी। `$Boot` उन records को किसी एकत्र किए गए host के volume से जोड़ता है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> किसी LNK, Jump List या Prefetch का volume serial
number जो किसी `$Boot` के `VolumeSerialNumber` के बराबर हो, उस volume की ओर संकेत करता है, या किसी अन्य volume की ओर
जिसे वही serial number दिया गया था या जिसके साथ वह copy हुआ था। Serial numbers बदले जा सकते हैं और
hosts के बीच अद्वितीय नहीं होते।

## स्थान और संग्रह { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

इसे raw-aware forensic टूल से एकत्र करें, या volume image का पहला sector पढ़ें: दोनों
एक ही bytes हैं। Juntei NTFS boot sector से शुरू होने वाली कोई भी फ़ाइल पढ़ता है, इसलिए volume image
input के रूप में काम करती है। Disk image इसके बजाय partition table से शुरू होती है; Juntei को partition का volume दें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- Volume format होने पर serial number बदल जाता है। ऐसी LNK फ़ाइल जिसका serial number किसी वर्तमान
  volume से मेल नहीं खाता, removable media, किसी network share के server, या उसके बाद format किए गए volume की ओर संकेत कर सकती है।
- एक ही Windows द्वारा format किए गए volumes पर boot code एक जैसा होता है: DFIR Artifact Museum की
  Windows 10, 11 और Server 2012 R2 `$Boot` फ़ाइलों पर यह समान है। Volume boot record
  bootkit इसे बदलता है, या hidden sectors field को, जिसका उपयोग boot code `$Boot` का बाकी हिस्सा खोजने के लिए करता है।
  कोई निष्कर्ष निकालने से पहले `BootCodeSHA256`, `JumpInstruction` और `HiddenSectors` की तुलना अन्य hosts से और partition
  table से करें: कोई boot manager या encryption software भी boot
  code बदल सकता है।
- BitLocker-encrypted volume की image BitLocker के अपने boot sector (OEM ID
  `-FVE-FS-`) से शुरू होती है। `-f` के साथ देने पर Juntei इसे report करता है, और `Problems` बताता है कि यह NTFS नहीं है; NTFS boot
  sector encrypted data में है। Unlock किए गए volume से एकत्र किया गया `$Boot` NTFS वाला होता है।
- `Problems` पहचानी गई structural असंगतियों और truncation को सूचीबद्ध करता है, न कि Windows mountability का पूर्ण परीक्षण प्रदान करता है। इसका उपयोग carved sectors और copies के आकलन के लिए करें, अपने-आप में attack finding के रूप में नहीं। Boot-code hash में शुरुआती jump, BIOS parameter block और end signature शामिल नहीं हैं; उन fields की भी जाँच करें।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

Summary हर boot sector का volume serial number (जैसा Windows दिखाता है, और जैसा LNK तथा
Prefetch fields दिखाते हैं), आकार, cluster आकार और `$MFT` cluster सूचीबद्ध करता है। जब कई hosts parse किए जाते हैं, तो यह
उन्हें boot code hash के अनुसार समूहित करता है, ताकि जिस volume का boot code दूसरों से अलग हो वह अलग दिखे।

## CSV आउटपुट fields { #csv-output-fields }

| Field | व्याख्या |
|---|---|
| `SourcePath` | Input फ़ाइल। |
| `VolumeSerialNumber` | 32-bit serial number, जैसा LNK, Jump List और Prefetch आउटपुट इसे दिखाते हैं (`180B1E67`)। Windows इसे `180B-1E67` के रूप में दिखाता है। |
| `VolumeSerialNumber64` | पूरा 64-bit serial number। |
| `OEM-ID` | NTFS volume पर `NTFS`। |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | Volume की geometry। 128 से अधिक sectors per cluster दो की घात के रूप में दर्ज होते हैं, जिन्हें यहाँ decode किया जाता है। मान्य न होने पर खाली। |
| `TotalSectors`, `VolumeSize` | Volume में sectors और bytes में उनका आकार, उसके अंत में boot sector की copy के बिना। |
| `HiddenSectors` | Disk पर volume से पहले के sectors: जहाँ उसका partition शुरू होता है। |
| `MFT-Cluster`, `MFT-Offset` | `$MFT` कहाँ शुरू होता है, clusters में और volume की शुरुआत से bytes में। |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | `$MFTMirr` कहाँ शुरू होता है। |
| `FileRecordSize`, `IndexRecordSize` | प्रति FILE record bytes (आमतौर पर 1,024) और प्रति INDX record (आमतौर पर 4,096)। |
| `MediaDescriptor` | Hard disk के लिए `F8`। |
| `JumpInstruction` | पहले तीन bytes (`EB 52 90`): BIOS parameter block के ऊपर से boot code तक jump। |
| `EndSignature` | मान्य boot sector के अंत में `55 AA`। |
| `BootCodeSHA256` | Boot code का SHA-256: पहले sector के bytes 0x54 से 0x1FD और 8 KiB का बाकी हिस्सा। |
| `Problems` | वह जिसे Windows mount नहीं करेगा: `NTFS` के अलावा कोई OEM ID, `55 AA` का न होना, अमान्य आकार, ऐसे FAT fields set होना जिन्हें NTFS शून्य रखता है, volume के अंत से आगे `$MFT`; और 8 KiB से छोटी फ़ाइल। |

JSON/JSONL में वही fields और नाम होते हैं, संख्याएँ संख्याओं के रूप में, अज्ञात values के लिए `null` और
`Problems` एक list के रूप में।

## जाँच की रणनीति { #investigation-strategy }

1. हर host के `VolumeSerialNumber` को उसकी LNK फ़ाइलों, Jump
   Lists और Prefetch फ़ाइलों के volume serial numbers के साथ रखें: जो किसी एकत्र किए गए volume से मेल नहीं खाते वे अन्य volumes हैं (USB
   drives, shares, बाद में format किए गए volumes)।
2. अन्य serial numbers को [USB डिवाइस](usb.md) आउटपुट में खोजें, जो mount किए गए removable volumes के volume
   serial numbers सूचीबद्ध करता है।
3. Hosts के बीच `BootCodeSHA256` और `HiddenSectors` की तुलना करें। अलग boot code की जाँच
   disassembler और disk की partition table से करें, और boot से संबंधित event log entries खोजें।
4. Volume image में `$MFT` खोजने के लिए `MFT-Offset` और `ClusterSize` का उपयोग करें।

## अधिक जानकारी { #further-details }

- [Microsoft: NTFS overview](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: NTFS format documentation](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (volume header)
- [The Sleuth Kit: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (इसका `boot` crate)
