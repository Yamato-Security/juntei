# डिस्क इमेज माउंट करना { #mounting-disk-images }

`juntei mount` चलते समय VHDX disk image की files को एक directory में read-only रूप में उपलब्ध करता है। Hayabusa और Juntei जैसे tools बिना files extract किए या root अधिकारों के उनकी जाँच कर सकते हैं। यह FUSE का उपयोग करता है और Linux पर चलता है। Reader और mounting library का रखरखाव [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) में किया जाता है।

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

दूसरे terminal में:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

Unmount करने के लिए पहले terminal में Ctrl+C दबाएँ, या `fusermount3 -u /mnt/host1` चलाएँ।

## Mount क्या दिखाता है { #what-the-mount-shows }

- Mount point पर NTFS volume की files दिखती हैं। एकत्र की गई image में `C` और `LongFileNames` जैसी directories तथा `$MFT` जैसे volume metadata हो सकते हैं। कई NTFS volumes `Volume1`, `Volume2` आदि के रूप में दिखते हैं।
- हर file read-only होती है और उसका owner वही होता है जो mount point का owner है। File times NTFS के
  `$STANDARD_INFORMATION` times होते हैं।
- नाम वैसे ही सूचीबद्ध होते हैं जैसे NTFS उन्हें संग्रहीत करता है, और उन्हें exact match से खोजा जाता है।
  DOS 8.3 नाम और alternate data streams नहीं दिखाए जाते।
- Compressed files decompress की जाती हैं (हर file अधिकतम 1 GiB तक)। EFS-encrypted files पढ़ी नहीं जा सकतीं।

## साक्ष्य नहीं बदला जाता { #the-evidence-is-not-changed }

Image read-only खोली जाती है और mount लिखने से मना करता है। ठीक से बंद न की गई VHDX का pending log memory में लागू होता है और image में वापस नहीं लिखा जाता। कुछ DiscUtils writers NTFS boot sector की jump instruction और `55 AA` signature को शून्य छोड़ते हैं; Juntei ऐसे volumes को सीधे पढ़ता है।

## उपयोग { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## auto-parse से हर image को parse करना { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` evidence directory की हर VHDX disk image (या `-d` के साथ दी गई एकमात्र image) के अंदर के
Windows systems को, अलग से mount किए बिना, parse करता है:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- हर image को एक नई temporary directory में, image के नाम वाले फ़ोल्डर (`host1-triage`) में read-only
  mount किया जाता है, और वहाँ मिले systems को disk पर रखे संग्रह की तरह ही parse किया जाता है। अंत में, और
  Ctrl+C दबाने पर, images unmount कर दी जाती हैं।
- Terminal images और हर system की image सूचीबद्ध करता है:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`।
- Manifest में, `EvidenceDirectory` दी गई directory होती है और हर system का `Image` उसकी image का नाम बताता
  है। `EvidenceRoot` और parser commands mount फ़ोल्डर (`./host1-triage/C`) का उपयोग करते हैं।
- यदि image में Volume Shadow Copies की files `VSS32` जैसे folders में हैं, तो हर folder को दूसरे system (`HOST1_2`, …) के रूप में खोजकर parse किया जाता है।
- जो image mount नहीं हो पाती, उसकी सूचना दी जाती है; बाकी images फिर भी parse की जाती हैं, और run exit
  status 1 के साथ समाप्त होता है।

यदि कोई NTFS वॉल्यूम नहीं खुलता, तो Juntei समस्या बताता है और पढ़ने योग्य वॉल्यूम को बनाए रखता है। आंशिक रूप से पढ़ी जा सकने वाली इमेज के लिए `auto-parse --vhdx` निकास कोड 1 लौटाता है; `mount` भी अनमाउंट करने के बाद 1 लौटाता है।

## आवश्यकताएँ { #requirements }

- Linux: FUSE 3 (`fusermount3`, `fuse3` package) और `/dev/fuse` तक access, जो सामान्य users के पास आमतौर पर
  होता है। Root की आवश्यकता नहीं है।
- Windows और macOS समर्थित नहीं हैं। Windows पर, इसके बजाय image को elevated PowerShell में read-only attach
  करें, और tools को उसका drive letter दें:

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
