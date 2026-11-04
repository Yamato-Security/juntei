# पहली जाँच { #first-investigation }

यह workflow मानता है कि आपको `evidence/` में एक offline triage संग्रह मिला है।

## 1. स्रोत को सुरक्षित रखें { #1-preserve-the-source }

सत्यापित copy से काम करें। Hashes, संग्रह का समय, time zone, host, volume और संग्रह
टूल दर्ज करें। Juntei साक्ष्य पढ़ता है, लेकिन resident-stream या क्वारंटीन extraction जैसे options
नई आउटपुट फ़ाइलें बनाते हैं; उन्हें एक अलग विश्लेषण directory में भेजें।

## 2. आपके पास क्या है उसकी सूची बनाएँ { #2-inventory-what-you-have }

[साक्ष्य संग्रह](collection.md) में दिए गए paths खोजें। Missing आर्टिफ़ैक्ट सामान्य हैं। केवल इसलिए
यह न मानें कि संग्रह में transaction logs हैं क्योंकि उसमें registry hive है।

## 3. पहले summaries चलाएँ { #3-run-summaries-first }

जब कोई आउटपुट फ़ाइल नहीं माँगी जाती, तो कई commands संक्षिप्त summary print करते हैं:

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

Records पर भरोसा करने से पहले क्षति, dirty-database और लागू न किए गए logs की warnings पढ़ें।

कई computers वाले संग्रह में हर समर्थित आर्टिफ़ैक्ट को एक ही operation में parse करने के लिए, इसका उपयोग करें:

```bash
juntei auto-parse -d evidence/ -o analysis/
```

Host discovery, parallelism, आउटपुट layout और SIEM ingestion के लिए consolidated JSONL के बारे में
[संग्रहीत डेटा का स्वचालित विश्लेषण](../commands/auto-parse.md) देखें।

## 4. Structured आउटपुट सहेजें { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

Scripting और nested values के लिए JSONL को, और spreadsheet या timeline viewer के लिए CSV को प्राथमिकता दें।

## 5. किसी भरोसेमंद identifier से pivot करें { #5-pivot-from-a-reliable-identifier }

फ़ाइल नाम, पूरा path, SHA-1/SHA-256, user SID, MFT entry/sequence, task GUID,
service नाम, या timestamp window से शुरू करें। Case को केवल उन fields के लिए normalize करें जिन्हें Windows case-insensitive मानता है।

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. Correlate करें { #6-correlate }

[कई आर्टिफ़ैक्ट पर आधारित जाँच](../resources/workflows.md) का उपयोग करें। एक व्यावहारिक क्रम है:

1. निष्पादन और हाल के run times के लिए Prefetch।
2. Creation, paths, renames और deletion के लिए `$MFT` और USN Journal।
3. ऐतिहासिक उपस्थिति और metadata के लिए Amcache और ShimCache।
4. User/application context के लिए LNK, Jump Lists, RecentDocs और SRUM।
5. Persistence के लिए registry services और TaskCache।
6. Detection, removal, origin और hashes के लिए Defender क्वारंटीन।

## 7. Reproducibility का दस्तावेज़ीकरण करें { #7-document-reproducibility }

स्रोत path, स्रोत hash, Juntei version, सटीक command, आउटपुट hash, system time zone,
और बाद की कोई भी filtering दर्ज करें। उत्पन्न साक्ष्य आउटपुट को कभी चुपचाप edit न करें।
