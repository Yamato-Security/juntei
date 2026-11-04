# NTFS ट्रांज़ैक्शन लॉग (`$LogFile`) { #ntfs-transaction-log-logfile }

## यह क्या है और क्यों महत्वपूर्ण है { #what-it-is-and-why-it-matters }

NTFS वह file system है जिसका उपयोग Windows आमतौर पर अपने system volume के लिए करता है। Crash से उबरने के लिए यह
file-system metadata में जो बदलाव करने वाला होता है, उन्हें `$LogFile` नामक एक छिपी हुई volume फ़ाइल में दर्ज
करता है। हर update में **redo** bytes (नई स्थिति) और **undo** bytes (किसी बदलाव को वापस लेने के लिए आवश्यक पिछली
स्थिति) हो सकते हैं। यह Windows Event Logs, registry `.LOG1`
फ़ाइलों और [USN जर्नल](usn-journal.md) से अलग है।

जाँचकर्ता के लिए पिछली स्थिति विशेष रूप से उपयोगी है। एक undo record rename से पहले का filename,
हटाई गई directory entry, या metadata update से पहले के timestamps सुरक्षित रख सकता है।
कुछ records में वे bytes भी होते हैं जो NTFS ने USN Journal में लिखे, जिनमें ऐसे records भी शामिल हैं जो अब
एकत्र किए गए `$J` में नहीं हैं। हालाँकि, कई transactions सामान्य operating-system गतिविधि से संबंधित होते हैं और
किसी हमले का वर्णन नहीं करते।

**उदाहरण:** एक हटाई गई index entry में `report.txt` का उल्लेख है; बाद में जोड़ी गई entry में
`report.txt.encrypted` का। पूरे file reference, parent, transaction chain और अन्य
artifacts का मिलान rename का समर्थन कर सकता है। केवल ये दो नाम ransomware, उसके operator,
या encryption के सटीक समय की पहचान नहीं करते।

## स्थान और संग्रह { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

हर NTFS volume का अपना log होता है। इसे ऐसे tool से प्राप्त करें जो NTFS metadata पढ़ता हो या इसे
forensic image या snapshot से export करता हो; साधारण Windows file copy शायद इस तक पहुँच न पाए। उसी
volume के `$MFT` और `$Extend\$UsnJrnl:$J` को समय में जितना संभव हो उतना पास एकत्र करें। मूल
फ़ाइलें और hashes सुरक्षित रखें। बाद में किए गए acquisition में गतिविधि का कोई अलग हिस्सा हो सकता है।

`$LogFile` एक **circular log** है: नए records पुराने records को overwrite करते हैं। Retention log के आकार
और workload पर निर्भर करता है और केवल कुछ मिनट या घंटे भी हो सकता है। किसी filename या operation का अनुपस्थित होना
इस बात का साक्ष्य नहीं है कि वह कभी हुआ ही नहीं।

## एक record क्या सिद्ध करता है, और सावधानियाँ { #what-a-record-proves-and-cautions }

- एक update वह metadata दर्ज करता है जिसे NTFS ने recovery के लिए log किया। अकेली redo entry यह सिद्ध नहीं करती कि
  कोई पूरी उच्च-स्तरीय action पूरी हुई या किसी विशेष व्यक्ति ने उसे शुरू किया। संबंधित
  records, transaction के पूरा होने, परिणामी file system और स्वतंत्र साक्ष्य को correlate करें।
- **कोई सामान्य transaction timestamp नहीं होता।** अंतर्निहित timestamps फ़ाइल metadata का वर्णन करते हैं,
  जो कभी-कभी पुराने, copy किए गए या हमलावर द्वारा संशोधित होते हैं। `CreatedOn` को incident timeline पर
  rename या timestamp बदलाव के समय के रूप में न रखें। USN records एक अलग timestamp दे सकते हैं।
- LSNs (log sequence numbers) records को **एक volume के log के भीतर** क्रमबद्ध करते हैं। ये clock समय नहीं हैं
  और इनकी तुलना hosts या volumes के बीच नहीं की जानी चाहिए। पुराने circular-buffer चक्रों के records
  नई गतिविधि के साथ बचे रह सकते हैं।
- Transaction-table slot का पुनः उपयोग होता है। केवल `TransactionId` के बजाय `TransactionFirstLsn`, `TransactionId`, client
  पहचान और स्रोत के आधार पर समूह बनाएँ। Chains रखे गए data के बाहर की ओर इशारा कर सकती हैं।
- File references में एक entry number **और एक sequence number** होता है। अन्यथा MFT entry का पुनः उपयोग
  असंबंधित फ़ाइलों को एक ही फ़ाइल जैसा दिखा सकता है।
- `$MFT` द्वारा दिए गए paths **एकत्र की गई MFT स्थिति** का वर्णन करते हैं, पुनर्निर्मित ऐतिहासिक
  path का नहीं। Juntei sequence numbers का मिलान करता है और अज्ञात घटकों को स्पष्ट छोड़ता है। Transaction के बाद
  parent का नाम बदला गया हो सकता है, और hard links एक से अधिक वर्तमान नाम दे सकते हैं।
- `TargetInferred` का अर्थ है कि open-attribute mapping log के किसी अलग हिस्से से आई।
  उस mapping को अतिरिक्त सावधानी से लें। Unresolved index entries केवल उनके
  layout के आधार पर decode की जा सकती हैं; उनके नामों को raw bytes और अन्य स्रोतों से सत्यापित करें।
- Torn page को छोड़ दिया जाता है; आसपास के पठनीय records उपलब्ध रहते हैं। `Truncated` ऐसे
  record को चिह्नित करता है जिसका continuation लापता है। Output को पूर्ण मानने से पहले parser diagnostics की
  समीक्षा करें। Zero-length input एक error है; बिना लिखा गया all-`FF` log वैध रूप से कोई rows नहीं रखता।
- Parser ऐतिहासिक MFT को पुनर्निर्मित करने के लिए transactions को **replay नहीं** करता। यह मनमाने
  non-resident file contents recover नहीं करता और न ही एकत्र किए गए किसी data को execute करता है। DAX CRC-protected
  pages समर्थित नहीं हैं। वास्तविक साक्ष्य पर सत्यापन मुख्यतः 4 KiB log pages को कवर करता है।

## Juntei से parse करें { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

सभी options के लिए [command reference](../commands/filesystem.md#parse-ntfs-logfile) देखें। Directory
discovery `$LogFile`, `LogFile`, उनके `.bin` रूप, और `C_$LogFile` या
`C_$LogFile.bin` जैसे नाम स्वीकार करता है, case को अनदेखा करते हुए। `-f` से दी गई एक फ़ाइल का कोई भी नाम हो सकता है।

स्वचालित MFT pairing export के volume prefix का सम्मान करता है: `C_$LogFile` `C_$MFT` का उपयोग करता है, और
`D_$LogFile.bin` पहले `D_$MFT.bin`, फिर `D_$MFT` आज़माता है। Prefix वाले logs कभी भी fallback के रूप में साधारण `$MFT`
का उपयोग नहीं करते। बिना prefix वाला `LogFile` `MFT` या `$MFT` भी स्वीकार करता है। जब आपके export में कोई अन्य naming scheme हो,
तो मेल खाते MFT को स्पष्ट रूप से चुनने के लिए `-f` के साथ `-m` का उपयोग करें।

Output प्रति LSN एक record है, जिसमें update, checkpoint और unknown client records शामिल हैं। JSON
pretty-printed objects की stream है; JSONL में प्रति line एक object होता है। CSV में वही field names होते हैं;
arrays और संरचित fields CSV cell के अंदर compact JSON होते हैं। अज्ञात scalars CSV में खाली और
JSON में `null` होते हैं। खाली arrays का अर्थ है कि कोई item decode नहीं हुआ, यह प्रमाण नहीं कि कोई मौजूद नहीं था।

एक समय में एक पूरा log और उसके पार्स किए गए records memory में रखे जाते हैं। MFT paths resolve करने से
memory और काम बढ़ता है। `--raw-data` output का आकार बहुत बढ़ा सकता है। `auto-parse` में, कम memory वाली मशीन पर
समवर्ती parser processes सीमित करने के लिए `-T` का उपयोग करें।

यदि साथ वाला MFT पढ़ा नहीं जा सकता, तो Juntei चेतावनी देता है और log को paths के बिना export करता है; तब `MftSource`
null रहता है। यह enrichment विफलता पठनीय प्राथमिक साक्ष्य को नहीं हटाती।
लापता या अस्पष्ट MFT मिलान भी एक चेतावनी और summary में `Logs without a matching MFT`
count उत्पन्न करते हैं; इन्हें parse विफलताओं के रूप में नहीं गिना जाता।

## आउटपुट fields { #output-fields }

### Record की पहचान, integrity और provenance { #record-identity-integrity-and-provenance }

| फ़ील्ड | अर्थ और व्याख्या |
|---|---|
| `SourceFile`, `FileOffset` | Input path और वह byte offset जहाँ इस record का header पढ़ा गया; bytes सत्यापित करने के लिए इनका उपयोग करें। |
| `LSN` | Record का log sequence number। बड़े values integers हैं, timestamps नहीं। |
| `PreviousLSN`, `UndoNextLSN` | पिछले और rollback records के links। शून्य का अर्थ है कोई link नहीं; nonzero links रखे गए data के बाहर की ओर इशारा कर सकते हैं। |
| `TransactionID`, `TransactionFirstLSN` | Transaction-table slot और इस transaction से जुड़ा सबसे पहला LSN; केवल slot अद्वितीय नहीं है। |
| `RecordType` | `ClientRecord` (update), `ClientRestart` (checkpoint), या `Other`। |
| `ClientIndex`, `ClientSequence` | Log client की पहचान; सामान्यतः NTFS। ये कोई user account या process ID नहीं हैं। |
| `Flags`, `SpansPages` | कच्चे record flags और क्या client data दूसरे page पर जारी रहता है। Pages में फैलना सामान्य है। |
| `Truncated` | True, यदि parser को इस LSN के लिए लापता continuation मिला। यह पूरी फ़ाइल की सारी क्षति का सारांश नहीं देता। |
| `ClusterSize`, `ClusterSizeSource` | Target offsets के लिए उपयोग किए गए प्रति cluster bytes; स्रोत `Caller`, `Checkpoint`, या `Inferred` है। ज्ञात हो तो वास्तविक value `--cluster-size` से दें। |
| `LogVersion` | नवीनतम पठनीय restart page से version, आमतौर पर `1.1` या `2.0`; यदि कोई पठनीय restart page नहीं बचा तो null। |
| `MFT-Source` | वर्तमान path enrichment के लिए उपयोग किया गया MFT, यदि मौजूद हो। यह उसी volume का होना चाहिए। |

`auto-parse` JSON/JSONL में, अन्य parsers की तरह `Comp` और `Artifact` जोड़े जाते हैं। Task की
`.txt` log में diagnostics होते हैं; manifests बताते हैं कि task सफल हुआ, विफल हुआ, या छोड़ दिया गया।
बचाई जा सकने वाली क्षति के साथ task सफल हो सकता है, जबकि उसकी log में `Parse problems` count nonzero हो।

### Operations और target { #operations-and-target }

| फ़ील्ड | अर्थ और व्याख्या |
|---|---|
| `RedoOperation`, `UndoOperation` | NTFS operation नाम, या अज्ञात code के लिए hexadecimal। Redo इच्छित update का वर्णन करता है; undo उसके व्युत्क्रम का। |
| `TargetEntryNumber`, `TargetSequenceNumber` | File record पर operation के लिए: प्रभावित MFT record number, बिना कोई sequence number अनुमानित किए। अन्यथा: resolve किए गए open attribute का स्वामी file reference। Inline directory-index update directory को target करता है, न कि उसकी entry द्वारा वर्णित child को। |
| `TargetAttributeType`, `TargetAttributeName` | Operation द्वारा उपयोग किए गए open attribute का numeric type और नाम। File-record operations के लिए यह सामान्यतः `$MFT` का `$DATA` होता है। सामान्य types: 16 (`$STANDARD_INFORMATION`), 48 (`$FILE_NAME`), 128 (`$DATA`), 144 (`$INDEX_ROOT`), 160 (`$INDEX_ALLOCATION`), 176 (`$BITMAP`)। |
| `TargetPath` | जब target का पूरा reference ज्ञात हो तो वर्तमान MFT path, या अज्ञात घटक। उन file-record operations के लिए खाली जिनकी ऐतिहासिक generation अज्ञात है; केवल record number को उसके वर्तमान occupant में resolve नहीं किया जाना चाहिए। |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | वह फ़ाइल जो उस open attribute की स्वामी है जिसके माध्यम से operation लिखता है, उसके वर्तमान MFT path के साथ। File-record operations के लिए ये `$MFT` संदर्भ को प्रभावित record से अलग सुरक्षित रखते हैं। |
| `TargetAttribute` | Log की open-attribute table में offset; यह attribute के type या MFT identifier से अलग है। |
| `TargetInferred` | True, यदि target mapping log के किसी अन्य हिस्से से आई है। यह flag false होने पर भी null target unresolved ही रहता है। |
| `TargetVCN`, `Lcns` | Target में virtual cluster और physical cluster numbers; समय निर्धारण के बजाय low-level सत्यापन के लिए उपयोगी। |
| `RecordOffset`, `AttributeOffset` | Attribute/entry के और उसके भीतर update के offsets। |
| `ClusterBlockOffset`, `TargetBlockSize` | 512 bytes की इकाइयों में offset और target का आकार। |
| `OffsetInTarget` | प्रभावित file record या non-resident stream में गणना किया गया byte offset। Cluster और MFT record के आकार पर निर्भर करता है। |
| `RedoLength`, `UndoLength` | घोषित data lengths। Zeroing operation इतने payload bytes लिए बिना भी कोई length घोषित कर सकता है। |
| `RedoBytesAvailable`, `UndoBytesAvailable` | Decoding के लिए वास्तव में उपलब्ध payload bytes। घोषित lengths और operation के अर्थ से तुलना करें। |

### सुविधाजनक फ़ाइल metadata columns { #convenient-file-metadata-columns }

ये columns redo पक्ष से एक पसंदीदा non-DOS filename दिखाते हैं, और उसके न होने पर undo पक्ष से।
यदि किसी भी पक्ष में filename नहीं है, तो timestamp-update fields पहले redo timestamps को प्राथमिकता देते हैं, फिर undo को।
Directory-entry updates भी अपने समय, आकार और flags सीधे दिखाते हैं, undo की तुलना में redo को
प्राथमिकता देते हुए। Decode किए गए `$DATA` attributes filename मौजूद न होने पर भी अपना आकार दिखाते हैं। Decode किए गए
file record का unnamed `$DATA` आकार उसके संभवतः पुराने filename आकार पर प्राथमिकता लेता है।
ये browsing की सुविधा हैं; पूर्ण पहले/बाद की तुलना के लिए अलग पक्ष-विशिष्ट fields का उपयोग करें।

| फ़ील्ड | अर्थ और व्याख्या |
|---|---|
| `MetadataSide` | `Redo` या `Undo`, जो उस पक्ष की पहचान करता है जो सुविधाजनक filename/timestamp columns देता है। |
| `EntryNumber`, `SequenceNumber` | Decode किए गए filename/file-record/attribute metadata या आंशिक `$STANDARD_INFORMATION` timestamp update द्वारा वर्णित फ़ाइल, ज्ञात होने पर sequence के साथ। ऐसे directory-entry update के लिए खाली जो child की पहचान नहीं करता, और कच्चे size/run-list updates के लिए भी। प्रभावित record के लिए `TargetEntryNumber` का उपयोग करें। |
| `Filename` | पसंदीदा decode किया गया नाम; सभी नाम, DOS aliases और hard links सहित, पक्ष-विशिष्ट arrays में रहते हैं। |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | Parent reference और वर्तमान MFT path; ऐतिहासिक path का पुनर्निर्माण नहीं। |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | UTC metadata समय, 100 ns तक। ये transaction timestamps नहीं हैं। लापता/अमान्य समय खाली/null रहते हैं। |
| `FileSize`, `FileAttributes` | चुने गए metadata से आकार और numeric Windows flags; directory-entry values पुराने हो सकते हैं। `$DATA` attributes और decode किए गए file records अधिक प्रत्यक्ष आकार दे सकते हैं। |
| `AttrType` | Decode किए गए attribute record का numeric type; decode किए गए directory-entry metadata update के लिए 48। Operation द्वारा target किए गए open attribute से अलग। |

### पूर्ण redo/undo metadata और payloads { #complete-redoundo-metadata-and-payloads }

| फ़ील्ड | अर्थ और व्याख्या |
|---|---|
| `RedoFileNames`, `UndoFileNames` | उस पक्ष पर decode किए गए हर `$FILE_NAME` के arrays, जो जोड़े गए/नए बनाम हटाए गए/पुराने नामों को सुरक्षित रखते हैं। |
| `RedoTimes`, `UndoTimes` | हर पक्ष द्वारा लिखे गए `$STANDARD_INFORMATION` timestamps; आंशिक updates अपरिवर्तित timestamps का अनुमान लगाने के बजाय उन्हें अनुपस्थित छोड़ते हैं। |
| `RedoDecoded`, `UndoDecoded` | संरचित file record, attribute, index entry या directory-entry जानकारी, जिसकी पहचान `Kind` से होती है। Null का अर्थ असमर्थित operation या अपर्याप्त bytes हो सकता है। |
| `JournalWriteOffset`, `JournalWriteLength` | `$DATA:$J` में लिखने वाले redo data का offset और byte count। यह अन्यथा खो चुके USN records के संभावित स्रोत की पहचान करता है; यह पुनर्निर्मित journal export नहीं करता। |
| `Checkpoint` | Checkpoint version, start LSN, cluster आकार, और open-attribute, attribute-name, dirty-page तथा transaction-table dumps का LSN/length। |
| `OtherRecordType` | अपरिचित या decode न हो सकने वाले body के लिए कच्चा client record type। |
| `RedoData`, `UndoData`, `OtherData` | Hex payloads केवल `--raw-data` के साथ; अन्यथा null/खाली। असमर्थित या क्षतिग्रस्त संरचनाओं के सत्यापन के लिए इन्हें रखें। |

हर filename object में file और parent references (`Raw`, `EntryNumber`,
`SequenceNumber`), `Name`, `NameEscaped`, `Namespace`, वर्तमान `ParentPath`, चार timestamps,
`Size`, `AllocatedSize`, `Attr` और `ReparseValue` शामिल होते हैं। `NameEscaped` escapes के साथ दर्शाए गए अमान्य UTF-16 की
पहचान करता है। Reparse value एक reparse tag या extended-attribute आकार हो सकता है।

Timestamp objects `Utc` और integer `Filetime` दोनों सुरक्षित रखते हैं; कोई sentinel या अमान्य value
अपना मूल integer बनाए रखते हुए null UTC प्रतिनिधित्व रख सकता है। `RedoTimes` और `UndoTimes`
में `Created`, `Modified`, `Changed` और `Accessed` होते हैं; लापता सदस्यों के values null होते हैं।

Decode किए गए `FileRecord` objects में number, sequence, hard-link count, flags, base reference,
क्या केवल header बचा है, और मौजूद attributes शामिल होते हैं। हर decode किया गया attribute
type, नाम, resident स्थिति, flags, ID, आकार और allocated आकार सुरक्षित रखता है। `$STANDARD_INFORMATION`
चार समय, attributes, owner/security IDs, quota और USN जोड़ता है; इन IDs को अलग से resolve करना पड़ता है और
ये फ़ाइल बदलने वाले उपयोगकर्ता का प्रमाण नहीं हैं। Index entries में flags, key length और एक
वैकल्पिक filename शामिल होते हैं। Directory-entry updates चार समय, आकार, attributes और reparse
value सुरक्षित रखते हैं, भले ही नाम स्वयं log न किया गया हो।

## जाँच का workflow { #investigation-workflow }

1. लापता restart pages, torn pages और truncated records के लिए diagnostics की समीक्षा करें।
2. किसी संदिग्ध filename या extension के लिए `RedoFileNames` और `UndoFileNames` दोनों में खोजें।
3. Entry **और sequence** numbers का मिलान करें, फिर transaction की LSN chain और operations की जाँच करें।
4. बदलावों के लिए redo और undo timestamps की तुलना करें। सामान्य copying, extraction और restoration भी
   timestamps बदल सकते हैं; कोई अंतर एक सुराग है, स्वचालित timestomping निर्णय नहीं।
5. [USN जर्नल](usn-journal.md), [$MFT](mft.md) और [$I30](i30.md) से correlate करें। निष्पादन या attribution स्थापित
   करने के लिए event logs, [Prefetch](prefetch.md) और user-activity artifacts का उपयोग करें।

उदाहरण के लिए, किसी भी पक्ष पर encrypted filename रखने वाले सभी records खोजें:

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

यह filename खोज है, पूर्ण ransomware detector नहीं। Journal payloads की जाँच के लिए,
`--raw-data` के साथ export करें और non-null `JournalWriteOffset` वाले records चुनें; `RedoData`
hex उस offset पर लिखे गए bytes को दर्शाता है। उन अंतर्निहित USN records को पार्स और deduplicate करना
एक अलग चरण है। Juntei अभी उन्हें `parse-ntfs usn-journal` output के साथ merge नहीं करता।

## अधिक जानकारी { #further-details }

- [ys-ntfs-parser का logfile crate](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile): समर्थित संरचनाएँ, सत्यापन corpus और ज्ञात सीमाएँ।
- [Sootmark ntfslog](https://github.com/Sootmark/ntfslog): upstream Rust implementation।
- [libfsntfs NTFS format documentation](https://github.com/libyal/libfsntfs/tree/main/documentation): log pages, fixups और metadata संरचनाएँ।
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs): स्वतंत्र NTFS parsing शोध और implementation।
- [Joakim Schicht द्वारा LogFileParser](https://github.com/jschicht/LogFileParser): विस्तृत transaction-log विश्लेषण।
- [Stroz Friedberg द्वारा ntfs-linker](https://github.com/strozfriedberg/ntfs-linker): NTFS स्रोतों को फ़ाइल इतिहास में correlate करना।
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum): वास्तविक Windows NTFS samples।
- [Velocidex go-ntfs](https://github.com/Velocidex/go-ntfs): NTFS implementation और सार्वजनिक test volume।
