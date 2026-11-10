# डिस्क इमेज माउंट करना { #mounting-disk-images }

`juntei mount` चलते समय VHDX / E01 / Ex01 disk image की files को एक directory में read-only रूप में उपलब्ध करता है। Hayabusa और Juntei जैसे tools बिना files extract किए या root अधिकारों के उनकी जाँच कर सकते हैं। यह FUSE का उपयोग करता है और Linux पर चलता है। Reader और mounting library का रखरखाव [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) में किया जाता है।

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
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## इमेज का नेटिव विश्लेषण { #parsing-every-image-with-auto-parse }

`--vhdx` के साथ `auto-parse`, [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) द्वारा **Linux, macOS और Windows** पर VHDX को नेटिव रूप से पढ़ता है। OS माउंट, FUSE या प्रशासक अधिकार आवश्यक नहीं हैं।

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

पार्सरों को पथ चाहिए; Juntei उपलब्ध NTFS फ़ाइलों को सीमित बफ़र से निजी अस्थायी स्थान में कॉपी करता है। **इन सभी फ़ाइलों के लिए पर्याप्त स्थान रखें**, सिर्फ़ परिणाम देने वाले आर्टिफ़ैक्ट के लिए नहीं। समर्थित होने पर शून्य क्षेत्रों को स्पार्स रूप में लिखा जाता है। नाम, संरचना, हार्ड लिंक और संशोधन समय सुरक्षित रहते हैं। LNK और Jump List के स्रोत समय मूल NTFS मेटाडेटा स्नैपशॉट से आते हैं, अस्थायी फ़ाइलों से नहीं। पार्सर त्रुटियों सहित सामान्य समापन प्रतियाँ हटाता है; जबरन समाप्ति उन्हें छोड़ सकती है। स्रोत केवल पढ़ने योग्य रहता है; लंबित VHDX लॉग सिर्फ़ मेमोरी में लागू होते हैं।

होस्ट खोज, CSV/JSON/JSONL और `-j` सामान्य पाइपलाइन इस्तेमाल करते हैं। `manifest.json` का `Image` मूल इमेज बताता है; `EvidenceRoot` अंदर का सापेक्ष पथ बताता है। पढ़ने की त्रुटियाँ और असुरक्षित नाम `ImageErrors` तथा `manifest.csv` की असफल पंक्तियों में दर्ज होते हैं। पढ़े जा सकने वाले परिणाम सुरक्षित रहते हैं; अधूरा इनपुट कोड 1 देता है। खतरनाक या अस्पष्ट नाम बदले नहीं जाते। नामित NTFS स्ट्रीम अतिरिक्त फ़ाइलों की तरह निकाली नहीं जातीं; सामान्य फ़ाइलों के रूप में एकत्र आर्टिफ़ैक्ट उपलब्ध रहते हैं।

अलग [`mount` कमांड](mount.md) अन्य प्रोग्रामों को फ़ाइलसिस्टम देता है और Linux FUSE की आवश्यकता होती है। `auto-parse --vhdx` इसे उपयोग नहीं करता।

स्टेजिंग डिफ़ॉल्ट रूप से आउटपुट फ़ाइल सिस्टम पर एक निजी डायरेक्टरी का उपयोग करती है। दूसरे डिस्क की मौजूदा डायरेक्टरी चुनने के लिए `--temp-dir` उपयोग करें। मूल मेटाडेटा आउटपुट फ़ाइल सिस्टम पर अलग रखा जाता है, ताकि स्टेजिंग डिस्क भरने पर भी पढ़ने योग्य फ़ाइलें पार्स हो सकें। कॉपी और परिणाम दोनों के लिए स्थान रखें; आउटपुट वॉल्यूम भरने पर परिणाम नहीं सहेजे जा सकते। Ctrl+C पार्सर प्रक्रियाएँ रोकता है, निजी कार्यक्षेत्र साफ़ करता है और कोड 130 से निकलता है। SIGKILL या बिजली जाने से कॉपी रह सकती हैं। इमेज खोजते समय अपठनीय उपडायरेक्टरियाँ छोड़ी जाती हैं, लेकिन अपठनीय इनपुट रूट त्रुटि बना रहता है।

स्टेजिंग डायरेक्टरी आउटपुट डायरेक्टरी के बाहर होनी चाहिए। Linux और macOS पर SIGTERM और SIGHUP भी पार्सर प्रक्रियाओं को रोकते हैं और कार्यक्षेत्र साफ़ करते हैं।

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## E01 और Ex01 डिस्क इमेज { #e01-and-ex01-disk-images }

`--e01` भौतिक EWF1 (`.E01`) और EWF2 (`.Ex01`) इमेज को **Linux, macOS और Windows** पर मूल रूप से पढ़ता है। FUSE, OS माउंट या व्यवस्थापक अधिकार आवश्यक नहीं हैं। यह [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) और ऊपर बताए गए NTFS पार्सिंग प्रवाह का उपयोग करता है।

विभाजित इमेज का पहला सेगमेंट दें। `.E02` सहित सभी संबंधित फ़ाइलें अपने मूल नामों के साथ उसी डायरेक्टरी में रखें। पूरे डिस्क (MBR/GPT) और एकल NTFS वॉल्यूम समर्थित हैं। मिश्रित डायरेक्टरी के लिए `--vhdx --e01` उपयोग करें।

अस्थायी स्थान और मूल मेटाडेटा की आवश्यकताएँ समान हैं। E01 इमेज में आम तौर पर पूरी डिस्क होती है, और पार्सिंग शुरू होने से पहले सभी इमेज स्टेज की जाती हैं, इसलिए उनके सभी NTFS वॉल्यूम के कुल उपयोग किए गए स्थान के लगभग बराबर खाली स्थान रखें; 200 GB उपयोग वाली डिस्क के लिए लगभग 200 GB चाहिए। इमेज केवल पढ़ी जाती हैं। संग्रह की कमियाँ और खोलते या पढ़ते समय मिली क्षति `ImageErrors` तथा `manifest.csv` में दर्ज होती हैं; पढ़ने योग्य परिणाम सुरक्षित रहते हैं और निकास कोड 1 होता है। केवल पढ़े गए डेटा की जाँच होती है, सभी चंक या पूरे मीडिया के हैश की नहीं। तार्किक `.L01` और `.Lx01` फ़ाइलें अस्वीकार की जाती हैं; पासवर्ड विकल्प नहीं है।

अलग `mount` कमांड को अभी भी Linux FUSE चाहिए; मूल स्वचालित पार्सिंग को नहीं।

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## OS को दिखने वाले माउंट की आवश्यकताएँ { #os-visible-mount-requirements }

`juntei mount` के लिए Linux FUSE 3 (`fusermount3`) और `/dev/fuse` पहुँच चाहिए। Windows और macOS पर ऊपर का नेटिव विश्लेषण इस्तेमाल करें।
