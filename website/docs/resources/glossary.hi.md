# DFIR शब्दावली { #dfir-glossary }

| शब्द | सरल भाषा में अर्थ |
|---|---|
| Allocated | आर्टिफ़ैक्ट के अनुसार record/object वर्तमान में उपयोग में है। |
| Alternate Data Stream (ADS) | किसी file से उसके सामान्य बिना-नाम वाले content के अलावा जुड़ा named NTFS डेटा, जैसे `Zone.Identifier`। |
| Artifact (आर्टिफ़ैक्ट) | Operating system/application द्वारा उत्पन्न स्थायी डेटा जो forensic प्रश्नों का उत्तर दे सकता है। |
| Carving | Records को उनके सामान्य container/index पर निर्भर हुए बिना संरचना/signature द्वारा खोजना। |
| Control set | SYSTEM configuration का एक संस्करण (`ControlSet001` आदि); `Select\Current` सक्रिय control set का नाम बताता है। |
| Corroboration | उसी निष्कर्ष का समर्थन करने वाला स्वतंत्र साक्ष्य। |
| Dirty database/hive | संग्रह के समय primary file पूरी तरह synchronized नहीं थी; transaction logs में आवश्यक state हो सकती है। |
| Droid | GUID जैसा distributed-link-tracking identifier, जिसका उपयोग स्थानांतरण के बाद भी files/volumes को track करने के लिए होता है। |
| ESE / JET Blue | Microsoft का Extensible Storage Engine database format, जिसका उपयोग SRUM और अन्य Windows डेटा करते हैं। |
| FILETIME | 1601-01-01 UTC से 100-nanosecond अंतरालों की गिनती; कोई संख्या समय के रूप में तभी अर्थपूर्ण है जब उसका schema ऐसा कहे। |
| File reference | NTFS entry number और sequence number, जो MFT record की एक generation की पहचान करते हैं। |
| Hive | Windows Registry tree वाली binary file। |
| INDX record | NTFS directory index (`$I30`) का block (आमतौर पर 4 KiB) जो directory की files सूचीबद्ध करता है; इसका slack हटाए गए या rename किए गए file names के संभावित साक्ष्य रख सकता है। |
| LNK | Windows Shell Link/shortcut, जिसमें एक target और व्यापक metadata होता है। |
| MRU | Most Recently Used क्रम; आमतौर पर हर item का समय नहीं, बल्कि सापेक्ष क्रम। |
| MFT | NTFS Master File Table, जिसमें file/directory records और attributes होते हैं। |
| Prefetch hash | `.pf` naming में उपयोग किया जाने वाला Windows identifier; cryptographic content hash नहीं। |
| Registry key last-write | वह समय जब key बदली; Windows आमतौर पर यह नहीं बताता कि कौन-सी value बदली। |
| Resident data | File/stream bytes जो बाहरी data runs के बजाय सीधे किसी MFT attribute में संग्रहीत होते हैं। |
| RID | Relative Identifier, Windows SID authority/domain के अंतर्गत अंतिम account/group घटक। |
| SDDL | Security Descriptor Definition Language: text के रूप में लिखा गया security descriptor, जैसे `O:S-1-5-18D:(A;;0x001f01ff;;;S-1-5-32-544)`। |
| Security descriptor | किसी file का owner, primary group, DACL (कौन क्या कर सकता है) और SACL (auditing और integrity label)। NTFS प्रत्येक अलग descriptor को एक security ID के अंतर्गत `$Secure:$SDS` में एक बार रखता है। |
| SID | किसी account, service, group या security principal के लिए Windows Security Identifier। |
| Slack | किसी allocated संरचना के अंदर/बाद के अप्रयुक्त bytes जो पुराना डेटा बनाए रख सकते हैं। |
| SRUM | Application/उपयोगकर्ता resource उपयोग का System Resource Usage Monitor database। |
| Timestomping | जाँच को भ्रमित करने के लिए timestamps में जानबूझकर किया गया बदलाव; heuristics प्रमाण नहीं हैं। |
| Transaction log | Registry या database को सुसंगत state में recover करने के लिए उपयोग किए जाने वाले pending/committed बदलाव। |
| USN | Update Sequence Number, जिसका उपयोग NTFS change-journal records और संबंधित metadata करते हैं। |
| UTC | Coordinated Universal Time; cross-system timelines के लिए पसंदीदा सामान्य समय आधार। |
| Volume serial number | Format किए जाने पर NTFS द्वारा volume को दी गई संख्या (`$Boot`); इसके निचले 32 bits वही हैं जो LNK, Jump List और Prefetch files किसी file के volume के बारे में दर्ज करती हैं। |
