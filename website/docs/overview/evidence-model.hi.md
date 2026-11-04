# साक्ष्य और व्याख्या { #evidence-and-interpretation }

डिजिटल फ़ॉरेंसिक्स केवल fields पढ़ना नहीं है। यह तय करना है कि कोई field किस अवलोकन का समर्थन करता है,
वह समर्थन कितना मज़बूत है, और कौन-सी निर्दोष गतिविधि वही अवलोकन उत्पन्न कर सकती है।

## साक्ष्य की चार उपयोगी श्रेणियाँ { #four-useful-evidence-categories }

| श्रेणी | अर्थ | उदाहरण |
|---|---|---|
| उपस्थिति (Presence) | कोई path या फ़ाइल Windows को ज्ञात थी | Amcache file record, ShimCache entry, `$MFT` record |
| निष्पादन (Execution) | Windows ने किसी प्रोग्राम के launch या निष्पादन की स्थिति दर्ज की | Prefetch run time; समर्थित ShimCache execution flag |
| उपयोगकर्ता interaction | किसी उपयोगकर्ता या एप्लिकेशन ने किसी item का संदर्भ दिया/उसे खोला | RecentDocs, Jump List, Recent फ़ोल्डर की LNK |
| सिस्टम गतिविधि | Windows ने resource उपयोग या configuration दर्ज किया | SRUM counters, service configuration, TaskCache action |

कोई आर्टिफ़ैक्ट एक से अधिक श्रेणियों का समर्थन कर सकता है, लेकिन उसे कभी स्वचालित रूप से ऊँची श्रेणी में न रखें। उदाहरण के लिए,
संदिग्ध path वाली Amcache entry **उपस्थिति** का समर्थन करती है। मेल खाता Prefetch run time अधिक
मज़बूत **निष्पादन** साक्ष्य जोड़ता है। USN create record और `$MFT` path फ़ाइल को disk पर स्थापित कर सकते हैं, और
Defender क्वारंटीन समझा सकता है कि वह क्यों ग़ायब हुई।

## Timestamps कथन हैं, सार्वभौमिक सत्य नहीं { #timestamps-are-statements-not-universal-truth }

पूछें कि हर timestamp किस कारण बना:

- `$STANDARD_INFORMATION` और `$FILE_NAME` times NTFS metadata का वर्णन करते हैं और एक-दूसरे से भिन्न हो सकते हैं।
- Prefetch run time एक दर्ज launch का वर्णन करता है, लेकिन केवल हाल के run times ही बचे रहते हैं।
- ShimCache time आमतौर पर cache की गई फ़ाइल का last-modified time होता है—निष्पादन का समय नहीं।
- Registry key का last-write time पूरी key पर लागू होता है, किसी एक value पर नहीं।
- RecentDocs key का समय सबसे हाल के item को देता है; पुराने items का क्रम होता है, पर आमतौर पर खोलने का समय नहीं।
- SRUM timestamps अक्सर सटीक network operation के बजाय एक aggregation interval की पहचान करते हैं।

विश्लेषण के दौरान UTC का उपयोग करें, उपलब्ध हो तो मूल value बनाए रखें, और हर conversion का दस्तावेज़ीकरण करें।

## सकारात्मक और नकारात्मक साक्ष्य { #positive-and-negative-evidence }

किसी भली-भाँति समझे गए record की उपस्थिति मज़बूत सकारात्मक साक्ष्य हो सकती है। अनुपस्थिति आमतौर पर
कमज़ोर होती है, क्योंकि आर्टिफ़ैक्ट roll over होते हैं, caches की सीमा होती है, संग्रह अधूरा हो सकता है, settings भिन्न होती हैं,
और cleanup tools मौजूद हैं। “ऐसा नहीं हुआ” के बजाय “एकत्र किए गए आर्टिफ़ैक्ट में नहीं देखा गया” कहें।

## पुष्टि की सीढ़ी { #corroboration-ladder }

1. स्रोत फ़ाइल और संग्रह path को सत्यापित करें।
2. पुष्टि करें कि parser ने कोई प्रासंगिक क्षति या लागू न किए गए transaction logs रिपोर्ट नहीं किए।
3. Field की व्याख्या आर्टिफ़ैक्ट पृष्ठ के अनुसार करें।
4. वही कार्रवाई दर्शाने वाला कोई स्वतंत्र आर्टिफ़ैक्ट खोजें।
5. उपलब्ध हो तो event logs, network telemetry, memory या फ़ाइल की सामग्री जाँचें।
6. सटीक Juntei version, command, filters और स्रोत hash दर्ज करें।

!!! tip "निष्कर्ष साक्ष्य की मज़बूती के अनुरूप लिखें"
    “हमलावर ने ठीक 10:02:11Z पर malware निष्पादित किया” के बजाय “Prefetch `tool.exe` के 10:02:11Z पर चलने को दर्ज करता है”
    लिखना बेहतर है। दूसरा कथन पहचान और इरादा जोड़ता है, जिन्हें अकेले Prefetch
    स्थापित नहीं कर सकता।
