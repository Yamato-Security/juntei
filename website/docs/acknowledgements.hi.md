# आभार { #acknowledgements }

Juntei इसलिए अस्तित्व में है क्योंकि DFIR और open-source समुदायों ने parser, फ़ॉर्मैट शोध,
संदर्भ आउटपुट, specification और परीक्षण साक्ष्य प्रकाशित किए। नीचे दिए गए projects ने implementation,
सत्यापन, field naming या विश्लेषक के अपेक्षित आउटपुट को दिशा दी। सूची में होने का अर्थ यह नहीं है कि
उनका code Juntei में शामिल है; प्रत्येक project की license के लिए उसी project को देखें।

## Yamato Security द्वारा अनुरक्षित parser आधार { #parser-foundations-maintained-by-yamato-security }

| Juntei क्षेत्र | अनुरक्षित parser | Upstream/आधार |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`mft` crate) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| USN जर्नल | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`usn` crate) | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`logfile` crate) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| डायरेक्टरी इंडेक्स (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`mft` crate, `i30` module) | नीचे श्रेय दिया गया index-slack शोध |
| NTFS `$Boot`, `$Secure:$SDS` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`boot` और `secure` crates) | Microsoft का MS-DTYP और नीचे श्रेय दिया गया NTFS शोध |
| SRUM, UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) और [ese](https://github.com/Sootmark/ese) |
| वेब ब्राउज़र / SQLite | [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) | [Sootmark browser](https://github.com/Sootmark/browser), [sqlite](https://github.com/Sootmark/sqlite) और [plist](https://github.com/Sootmark/plist) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Sootmark shell का compound-file और Jump List कार्य |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| VHDX डिस्क इमेज (`mount`, `auto-parse --vhdx`) | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) और [ntfs-core](https://github.com/SecurityRonin/ntfs-forensic), जिन्हें [fuser](https://github.com/cberner/fuser) के ज़रिए serve किया जाता है | Microsoft का MS-VHDX |
| E01 / Ex01 (`mount`, `auto-parse --e01`) | [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) | [ewf-image](https://github.com/ebrig/ewf-image) (Apache-2.0); [libewf](https://github.com/libyal/libewf) (EWF1/EWF2) |
| रेजिस्ट्री | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | नीचे श्रेय दिया गया registry-format और recovery शोध |

## संदर्भ tools और स्वतंत्र सत्यापन { #reference-tools-and-independent-validation }

- Hexastrike के [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) ने native Rust
  `persistence-hunter` command के लिए persistence check catalogue, default allowlist, LOLBAS snapshot और
  संदर्भ व्यवहार प्रदान किया। इसकी MIT license और copyright notice
  `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt` में सुरक्षित रखी गई हैं। मूल
  [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) ने इस Python project को प्रेरित किया;
  पूरी वंश-परंपरा के लिए [पर्सिस्टेंस आभार](persistence-hunter/acknowledgements.md) देखें।

- Maxim Suhanov के **regf specification** और **yarp** ने registry संरचनाओं, transaction-log
  replay और हटाए गए record की recovery को दिशा दी; उनका **dfir_ntfs** INDX records और
  index-slack carving के लिए संदर्भ था।
- Harel Segev का **INDXRipper** index slack से फ़ाइल नाम carve करने के लिए एक अतिरिक्त संदर्भ है।
- **dfir_ntfs**, Joakim Schicht के **LogFileParser** और Stroz Friedberg के **ntfs-linker** ने
  `$LogFile` के सत्यापन और व्याख्या को दिशा दी; Velocidex के **go-ntfs** ने सार्वजनिक NTFS परीक्षण साक्ष्य प्रदान किए।
- Joachim Metz / libyal projects **libregf**, **libfsntfs**, **libscca**, **libfwsi** और
  **libesedb** विस्तृत फ़ॉर्मैट शोध और स्वतंत्र parsing संदर्भ प्रदान करते हैं।
- Fox-IT के **dissect.esedb**, **dissect.ntfs** और **dissect.target** ने सत्यापन के दौरान उपयोग की गई
  SRUM, UAL, `$Secure:$SDS`, SAM और Defender की स्वतंत्र व्याख्याएँ प्रदान कीं।
- **The Sleuth Kit** का `fsstat` उन volumes के NTFS boot sectors के लिए संदर्भ था जो
  `mkntfs` से बनाए गए थे।
- Eric Zimmerman के कुछ tools ने तुलना में उपयोग किए गए field नाम और परीक्षक की अपेक्षाएँ स्थापित कीं।
- **regipy**, **python-registry**, **Notatin** और **artemis** ने registry implementations,
  परीक्षण या नमूना व्याख्याएँ प्रदान कीं।
- **LnkParse3**, AbdulRhmanAlfaifi के **lnk_parser** और **jumplist_parser**, तथा JLECmd ने
  स्वतंत्र LNK/Jump List व्यवहार और सार्वजनिक test cases प्रदान किए।
- Security Ronin के **prefetch-forensic** और **forensicnomicon** project ने Juntei के
  उच्च-मूल्य वाले Prefetch/Amcache/ShimCache observations को दिशा दी।
- **defender-dump** और dissect.target, Defender quarantine संरचनाओं और सुरक्षित extraction व्यवहार को
  समझने के लिए अलग implementations प्रदान करते हैं।
- **Plaso**, **Chainsaw** और **Velociraptor** ने कई आर्टिफ़ैक्ट में parser तुलना, परीक्षण साक्ष्य या
  संग्रह-केंद्रित संदर्भ व्यवहार प्रदान किया।
- Obsidian Forensics का **Hindsight** और Velocidex की **SQLiteHunter** definitions browser history,
  downloads, cookies और form history के लिए संदर्भ रहीं।
- Abel Cheung का **rifiuti2**, libyal के **dtformats** में दोनों formats के documentation के साथ,
  रीसायकल बिन की `$I` files और INFO2 के लिए संदर्भ रहा।

## Specification और शोध { #specifications-and-research }

हम Shell Links, Compound Binary Files, NTFS/SAM protocols, registry/service/task संरचनाओं और API
documentation के लिए Open Specifications प्रकाशित करने हेतु Microsoft का धन्यवाद करते हैं। हम
AppCompatCache, Amcache, SRUM, Prefetch, registry और NTFS forensic शोध के उन लेखकों का भी धन्यवाद करते हैं
जिन्होंने इन आर्टिफ़ैक्ट को practitioners के लिए समझने योग्य बनाया।

## परीक्षण साक्ष्य { #test-evidence }

Andrew Rathbun के
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum), NIST CFReDS, Plaso,
rifiuti2, Regipy, Artemis, parser test repositories और अन्य उचित रूप से licensed संग्रहों के सार्वजनिक साक्ष्य
वास्तविक Windows डेटा के विरुद्ध regression testing को संभव बनाते हैं। Juntei शामिल की गई test files के
साथ attribution/license notes रखता है।

## योगदानकर्ता { #contributors }

Juntei और Yamato Security parser के प्रत्येक योगदानकर्ता, reviewer, issue reporter और उस उपयोगकर्ता
का धन्यवाद जिसने edge cases उपलब्ध कराए। वर्तमान योगदानकर्ताओं की सूची GitHub पर
[Juntei contributors page](https://github.com/Yamato-Security/juntei/graphs/contributors) पर उपलब्ध है।

## License { #license }

Juntei को
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) के अंतर्गत जारी किया गया है और इसका copyright Shirofune Security Co., Ltd. के पास है। संस्करण
1.0.0 से पहले इसका source code AGPLv3 के अंतर्गत प्रकाशित किया गया था। जिन crates तथा अन्य third-party
code और डेटा से यह बनाया गया है, उनकी licenses
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES) में हैं।
