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

## इमेज का नेटिव विश्लेषण { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` **Linux, macOS और Windows** पर VHDX को नेटिव रूप से पढ़ता है। OS माउंट, FUSE या प्रशासक अधिकारों की आवश्यकता नहीं है।

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

पार्सरों को फ़ाइल पथ चाहिए, इसलिए Juntei NTFS रीडर द्वारा उपलब्ध कराई गई फ़ाइलों को निजी अस्थायी डायरेक्टरी में कॉपी करता है। **इन सभी फ़ाइलों के लिए पर्याप्त अस्थायी डिस्क स्थान रखें।** मूल इमेज केवल पढ़ने के लिए खुलती है और सामान्य समापन पर अस्थायी प्रतियाँ हटा दी जाती हैं।

पढ़ने की त्रुटियाँ और असुरक्षित नाम `ImageErrors` तथा `manifest.csv` की असफल पंक्तियों में दर्ज होते हैं। पढ़े जा सकने वाले परिणाम सुरक्षित रहते हैं; अधूरे इनपुट पर एग्ज़िट कोड 1 मिलता है।

अन्य प्रोग्रामों को फ़ाइलें उपलब्ध कराने वाले अलग `mount` कमांड के लिए अभी भी Linux, FUSE 3 और `/dev/fuse` तक पहुँच आवश्यक है।

[English: implementation details and limitations](https://yamato-security.github.io/juntei/commands/auto-parse/#vhdx-disk-images).
