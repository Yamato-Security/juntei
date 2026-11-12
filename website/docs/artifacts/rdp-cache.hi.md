# RDP बिटमैप कैश { #rdp-bitmap-cache }

## RDP बिटमैप कैश क्या दर्ज करता है { #what-the-rdp-bitmap-cache-records }

जब Remote Desktop client (`mstsc.exe`) persistent bitmap caching चालू रहते हुए connect करता है, जो कि
default है, तो वह remote screen के टुकड़े disk पर सहेज लेता है, ताकि बाद के sessions को उन्हें दोबारा प्राप्त
न करना पड़े। हर टुकड़ा अधिकतम 64 × 64 pixels का एक tile होता है। सभी tiles मिलकर उस सामग्री के छोटे-छोटे
अंश हैं जो उपयोगकर्ता ने remote computers पर देखी: windows, text, फ़ाइलों के नाम, commands और desktops।

| Client | Cache फ़ाइलें | Tiles |
|---|---|---|
| Windows 7 और बाद के संस्करण | `Cache0000.bin`, `Cache0001.bin`, …, हर एक अधिकतम 100 MB | 32-bit, एक के बाद एक |
| Windows XP, Server 2003 और पहले के संस्करण | `bcache2.bmc` (8 bits per pixel), `bcache22.bmc` (16), `bcache23.bmc` (24) और `bcache24.bmc` (32), हर एक अधिकतम 10 MB | हर 64 × 64 slot में एक; कुछ compressed |

Windows 10 और बाद के संस्करण `.bin` फ़ाइलों के पास एक खाली `bcache24.bmc` भी छोड़ते हैं।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> जिस उपयोगकर्ता की profile में cache है, उसने Remote
Desktop client के ज़रिए किसी ऐसे computer से connect किया जिसने cached सामग्री दिखाई। Cache यह नहीं बताता कि वह
कौन-सा computer था, या कब।

## स्थान और संग्रह { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

`HKCU\Software\Microsoft\Terminal Server Client` का `BitmapPersistCacheLocation` value cache को किसी दूसरी
जगह ले जा सकता है। हर profile का पूरा `Cache` फ़ोल्डर एकत्र करें, volume shadow copies से भी: किसी cache
फ़ाइल की पुरानी copy में ऐसे tiles हो सकते हैं जो मौजूदा फ़ाइल में अब नहीं हैं।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- Tiles का कोई समय नहीं होता, और cache उन computers के नाम नहीं बताता जिनसे वे आए। उन्हें दूसरे साक्ष्यों
  से मिले RDP connections से जोड़ें: client की `Terminal Server Client` registry keys
  (`parse-registry remote-access`), `Default.rdp` और दूसरी `.rdp` फ़ाइलें, `mstsc.exe` की Jump Lists,
  `$MFT` में cache फ़ाइलों के times, और remote computers के logon events।
- एक cache फ़ाइल में कई sessions और कई remote computers के tiles मिले होते हैं। फ़ाइल भरने पर client पुराने
  tiles बदल देता है, इसलिए कोई tile कई दिन या कई महीने पुराना हो सकता है।
- Tiles screens के टुकड़े हैं, पूरी screens नहीं। जो tiles screen पर एक-दूसरे के पास थे, वे अक्सर फ़ाइल में
  भी एक-दूसरे के पास होते हैं, पर हमेशा नहीं। Screens को
  [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher) से, या collages को देखकर स्वयं, फिर से
  जोड़ें।
- अधिकांश tiles 64 × 64 pixels के होते हैं। Screen के दाएँ और निचले किनारों वाले tiles resolution के अनुसार
  संकरे या कम ऊँचे होते हैं: उदाहरण के लिए 58 या 62 pixels चौड़े, या 28 या 35 pixels ऊँचे। Juntei हर tile को
  उसके अपने आकार में पढ़ता है। जो tools हर tile को 64 pixels चौड़ा मानकर पढ़ते हैं, वे संकरे tiles को तिरछा
  दिखाते हैं।
- अधिकांश एक-रंग वाले tiles (`SingleColor`) background होते हैं। सामग्री खोजने के लिए उन्हें filter करके हटा
  दें।
- `.bmc` फ़ाइलों में 8-bit tiles के रंग session के palette से आते हैं, जिसे cache नहीं रखता। Juntei Windows का
  default palette उपयोग करता है, इसलिए उनके रंग गलत हो सकते हैं। Compressed tiles Remote Desktop protocol के
  interleaved RLE (MS-RDPBCGR 3.1.9) का उपयोग करते हैं, जिसे Juntei specification की परिभाषा के अनुसार
  decompress करता है।
- जो `.bmc` tile अपने 64 × 64 slot से छोटा होता है, उसके बाद किसी पहले वाले tile का अंतिम हिस्सा बचा रहता है।
  Juntei बची हुई rows को एक अलग image (`RemnantImage`) के रूप में सहेजता है। वे किसी पुराने tile की हैं, slot
  वाले tile की नहीं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```

```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```

`-d` के साथ, Juntei वे फ़ाइलें पढ़ता है जिनके नाम में `Cache` के बाद अंक और फिर `.bin` हो, या `bcache` के
बाद अंक और फिर `.bmc` हो। जो `Cache????.bin` फ़ाइल `RDP8bmp` signature से शुरू नहीं होती, वह किसी दूसरे
program की है: उसे summary में गिना जाता है और छोड़ दिया जाता है। `-f` के साथ, Juntei फ़ाइल को पढ़ता है,
उसका नाम चाहे जो हो।

### इमेज { #images }

`--extract` हर cache फ़ाइल के लिए एक फ़ोल्डर बनाता है, `<DIR>/<user>/<cache file>/`। जब दो cache फ़ाइलों के
लिए एक ही फ़ोल्डर बनता, जैसे कोई cache फ़ाइल और volume shadow copy से ली गई उसकी copy, तो दूसरे फ़ोल्डर के नाम में
एक संख्या जोड़ी जाती है (`Cache0001.bin_2`)। फ़ोल्डर में ये होते हैं:

- `<cache file>_<index>.bmp`: हर tile, उसके अपने आकार में। नाम वही हैं जो bmc-tools उपयोग करता है, इसलिए
  RdpCacheStitcher उस फ़ोल्डर को खोल सकता है।
- `<cache file>_old_<index>.bmp`: केवल `.bmc` के लिए, किसी छोटे tile के बाद बचा किसी पहले वाले tile का
  अंतिम हिस्सा।
- `<cache file>_collage.png`: फ़ाइल के क्रम में हर tile, एक row में 64 (`--collage-width`), हर एक 64 × 64
  cell में उसके ऊपरी बाएँ कोने से। Index *i* वाला tile row *i* / 64 और column *i* % 64 में होता है, 0 से
  गिनकर। Cells के खाली हिस्से magenta से भरे जाते हैं: संकरे tile के बगल में, कम ऊँचे tile के नीचे, और जहाँ
  कोई tile पढ़ा नहीं जा सका।

`--collage-only` केवल collages सहेजता है। पहले से मौजूद image `-C` के बिना नहीं बदली जाती, और उसके tile का
`Problems` यह बताता है। Cache फ़ाइलें स्वयं कभी नहीं बदली जातीं, `-C` के साथ भी नहीं।

## CSV आउटपुट fields { #csv-output-fields }

| Field | अर्थ |
|---|---|
| `SourceFile` | Cache फ़ाइल। |
| `User` | वह उपयोगकर्ता जिसके profile फ़ोल्डर में cache फ़ाइल है, उसके path से। |
| `TileIndex` | फ़ाइल में tile की संख्या, 0 से गिनी गई, जैसी उसकी image के नाम में है। |
| `Width`, `Height` | Tile का आकार pixels में। |
| `SingleColor` | `true`, जब tile के हर pixel का रंग एक जैसा हो। |
| `TileImage` | `--extract` के साथ: tile की BMP फ़ाइल। |
| `Collage` | `--extract` के साथ: cache फ़ाइल का collage। |
| `RemnantImage` | `--extract` के साथ `.bmc`: slot में बचा किसी पहले वाले tile का अंतिम हिस्सा। |
| `Key` | Remote Desktop protocol में tile की 64-bit key, hexadecimal में। |
| `Format`, `FormatVersion` | `bin` और उसके signature के बाद का version (Windows 10 और बाद के संस्करणों में 6), या `bmc`। |
| `BitsPerPixel` | `.bin` के लिए 32; `.bmc` के लिए 8, 16, 24 या 32। |
| `Compressed`, `Flags` | `.bmc`: tile compressed था या नहीं, और उसके header के flags। |
| `Offset` | फ़ाइल में tile का header कहाँ से शुरू होता है। |
| `Problems` | क्षति: ऐसी फ़ाइल जो किसी tile के बीच में खत्म हो जाती है, ऐसा tile header जो किसी भी tile का नहीं हो सकता, ऐसा data जो tile के आकार में नहीं बैठता, ऐसा compressed data जिसे decompress नहीं किया जा सकता, ऐसी image जिसे सहेजा नहीं जा सका। |

JSON और JSONL में यही fields होते हैं: numbers और booleans typed होते हैं, जो values किसी tile में नहीं हैं
उनके लिए `null` होता है, और `Problems` एक list होती है। किसी एक tile से बाहर की क्षति, जैसे अंतिम tile के
बाद का data, run के messages में बताई जाती है।

## स्वचालित विश्लेषण { #automatic-parsing }

`auto-parse` `LateralMovement/RdpCache.csv` लिखता है (`-t json` या `-t jsonl` के साथ `RdpCache.json` या
`RdpCache.jsonl`), साथ में `LateralMovement/RdpCache-logs.txt` भी, और हर cache फ़ाइल का collage
`LateralMovement/RdpCache/` फ़ोल्डर में सहेजता है। हर tile भी सहेजने के लिए `--extract-rdp-tiles` जोड़ें; केवल
एक उपयोगकर्ता के लिए ही ये दसियों हज़ार फ़ाइलें हो सकती हैं। Disk images से cache फ़ाइलें कॉपी की जाती हैं।
संयुक्त `-j` timeline में हर row `Artifact` identifier `rdp-cache` के साथ शामिल होती है। Tiles का कोई समय नहीं
होता, इसलिए [`super-timeline`](../commands/super-timeline.md) उनसे कोई event नहीं बनाता।

## जाँच की रणनीति { #investigation-strategy }

1. Collages खोलें और text खोजें: computer और user के नाम, file paths, commands, console windows, browser
   के address bars और dialog boxes।
2. `--extract` से सहेजे गए tiles के फ़ोल्डर से RdpCacheStitcher द्वारा screens फिर से बनाएँ।
3. Client की registry (`parse-registry remote-access`), `.rdp` फ़ाइलों और `mstsc.exe` की Jump Lists से वे
   computers खोजें जिनसे उपयोगकर्ता ने connect किया। उन computers पर मेल खाते RDP logons और उपयोगकर्ता एक्सेस
   लॉगिंग (UAL) records खोजें।
4. Tiles कब लिखे गए, यह सीमित करने के लिए cache फ़ाइलों के `$MFT` times, और volume shadow copies में मौजूद उनकी
   copies के times का उपयोग करें।

## अधिक जानकारी { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: RDP bitmap cache (फ़्रेंच में)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — RDP cache plugin](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — persistent bitmap cache](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
