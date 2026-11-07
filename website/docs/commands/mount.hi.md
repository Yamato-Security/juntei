# डिस्क इमेज माउंट करना { #mounting-disk-images }

`juntei mount` किसी VHDX disk image के अंदर की files को, जब तक यह चलता रहता है, एक directory पर read-only
रूप में पढ़ने योग्य बनाता है। इसके बाद KAPE के `--vhdx` संग्रह और Hyper-V disks को किसी भी tool से scan
किया जा सकता है, जैसे Hayabusa या Juntei के अपने commands से, कुछ भी extract किए बिना, root के बिना, और
operating system में VHDX या NTFS support के बिना। यह FUSE का उपयोग करता है: Linux, या macFUSE के साथ macOS।

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

- Mount point पर NTFS volume की files: KAPE संग्रह के लिए, संग्रहीत files वाला `C` फ़ोल्डर, KAPE का
  `LongFileNames` फ़ोल्डर, और volume की अपनी metadata files, जैसे `$MFT`। कई NTFS volumes वाली image उन्हें
  `Volume1`, `Volume2` आदि के रूप में दिखाती है।
- हर file read-only होती है और उसका owner वही होता है जो mount point का owner है। File times NTFS के
  `$STANDARD_INFORMATION` times होते हैं।
- नाम वैसे ही सूचीबद्ध होते हैं जैसे NTFS उन्हें संग्रहीत करता है, और उन्हें exact match से खोजा जाता है।
  DOS 8.3 नाम और alternate data streams नहीं दिखाए जाते।
- Compressed files decompress की जाती हैं (हर file अधिकतम 1 GiB तक)। EFS-encrypted files पढ़ी नहीं जा सकतीं।

## साक्ष्य नहीं बदला जाता { #the-evidence-is-not-changed }

Image read-only खोली जाती है और mount writes को अस्वीकार करता है। जो VHDX ठीक से बंद नहीं हुई थी, उसे
उसके pending log को memory में लागू करके पढ़ा जाता है; image में कुछ भी वापस नहीं लिखा जाता। KAPE अपनी
VHDX files DiscUtils से लिखता है, जो NTFS boot sector को उसके jump instruction और `55 AA` signature के बिना
छोड़ देता है; Juntei इन volumes को जैसे हैं वैसे ही पढ़ता है।

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
- KAPE, Volume Shadow Copies से copy की गई files को `VSS32` जैसे फ़ोल्डरों में रखता है। इनमें से हर फ़ोल्डर
  खोजा जाता है और एक अतिरिक्त system (`HOST1_2`, …) के रूप में parse किया जाता है।
- जो image mount नहीं हो पाती, उसकी सूचना दी जाती है; बाकी images फिर भी parse की जाती हैं, और run exit
  status 1 के साथ समाप्त होता है।

## आवश्यकताएँ { #requirements }

- Linux: FUSE 3 (`fusermount3`, `fuse3` package) और `/dev/fuse` तक access, जो सामान्य users के पास आमतौर पर
  होता है। Root की आवश्यकता नहीं है।
- macOS: macFUSE।
- Windows में FUSE नहीं है। इसके बजाय image को elevated PowerShell में read-only attach करें, और tools को उसका
  drive letter दें:

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
