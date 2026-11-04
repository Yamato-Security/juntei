# कई आर्टिफ़ैक्ट पर आधारित जाँच { #cross-artifact-workflows }

किसी भी आर्टिफ़ैक्ट को अकेले कोई महत्वपूर्ण निष्कर्ष नहीं ढोना चाहिए। ये workflows दिखाते हैं कि किसी सुराग को
साक्ष्य की एक बचाव-योग्य (defensible) श्रृंखला में कैसे बदला जाए।

## संदिग्ध executable का निष्पादन { #suspicious-executable-execution }

1. **Prefetch:** रखा गया run time, path/hash context, run count और loaded files स्थापित करें।
2. **`$MFT`:** path, size, timestamps, ADS, हटाई गई स्थिति और MFT reference खोजें।
3. **USN Journal:** creation, writes, rename और deletion का पुनर्निर्माण करें।
4. **Amcache:** SHA-1, PE metadata, version और ऐतिहासिक path जोड़ें।
5. **ShimCache:** compatibility-cache path/क्रम और समर्थित execution flag जोड़ें।
6. **SRUM:** उसी executable/उपयोगकर्ता द्वारा resource/network उपयोग खोजें।
7. **Defender:** क्वारंटीन समय, मूल path, hash और download origin जाँचें।
8. **Event logs/EDR:** process parent, command line, account, network और security context की पहचान करें।

निष्कर्ष में फ़ाइल की उपस्थिति, निष्पादन, और किसी व्यक्ति या intrusion से attribution के बीच अंतर होना चाहिए।

## Ransomware फ़ाइल गतिविधि { #ransomware-file-activity }

1. `$MFT` में encrypted extensions और पीड़ित paths की पहचान करें।
2. बड़े पैमाने पर writes/renames/deletes के पुनर्निर्माण के लिए USN reasons और क्रम का उपयोग करें।
3. संभावित encryptors और run times की पहचान के लिए Prefetch का उपयोग करें।
4. संभावित process द्वारा CPU/disk/network spikes के लिए SRUM का उपयोग करें।
5. Deployment/persistence के लिए services और TaskCache जाँचें।
6. Detection/remediation के लिए Defender क्वारंटीन और event logs जाँचें।

MFT/USN file-system प्रभाव सिद्ध कर सकते हैं, तब भी जब कोई निष्पादन आर्टिफ़ैक्ट नहीं बचा हो। Process telemetry या
correlation के बिना ये आमतौर पर ज़िम्मेदार process की पहचान नहीं करते।

## दुर्भावनापूर्ण document / शॉर्टकट से initial access { #malicious-document-shortcut-initial-access }

1. RecentDocs और Jump Lists उपयोगकर्ता/application interaction और MRU context की पहचान करते हैं।
2. Target, volume/share, MFT reference और arguments के लिए मेल खाती Recent-folder LNK को parse करें।
3. Origin URLs/zones के लिए `$MFT` resident ADS या Defender सामग्री से Zone.Identifier का उपयोग करें।
4. Document application और child processes के लिए Prefetch/event logs का उपयोग करें।
5. Extract किए गए payload के creation और deletion का अनुसरण करने के लिए USN/MFT का उपयोग करें।

## सेवा के माध्यम से persistence { #service-persistence }

1. Service नाम, `ImagePath`, `ServiceDll`, account, start type और failure actions decode करें।
2. Environment variables और quoted/unquoted path व्यवहार को resolve करें।
3. `$MFT` और USN history में executable/DLL का पता लगाएँ।
4. वास्तविक service/process start के लिए Prefetch और event logs जाँचें।
5. पहले के configuration के लिए non-current control sets और recover किए गए registry records की तुलना करें।

## Scheduled task के माध्यम से persistence { #scheduled-task-persistence }

1. TaskCache actions, principal/context, registration/runtime metadata और hidden स्थिति decode करें।
2. `C:\Windows\System32\Tasks` XML से तुलना करें।
3. Task नाम/GUID को Security और Task Scheduler Operational events से correlate करें।
4. Executable/script का पता लगाएँ और उसका hash निकालें; Prefetch और file-system history की जाँच करें।
5. यदि task हटाया गया था तो हटाए गए TaskCache records recover करें।

## संदिग्ध exfiltration { #suspected-exfiltration }

1. App/उपयोगकर्ता/profile द्वारा भेजे गए असामान्य bytes और measurement अवधि की पहचान के लिए SRUM का उपयोग करें।
2. Prefetch, Amcache और `$MFT` से executable की पहचान की पुष्टि करें।
3. Jump Lists, LNK, RecentDocs और event/EDR data के माध्यम से access/खोली गई फ़ाइलों की पहचान करें।
4. Destination/protocol के लिए proxy, firewall, DNS, packet या EDR telemetry का उपयोग करें।

SRUM volume exfiltration की परिकल्पना का समर्थन करता है, लेकिन अपने-आप में transfer की गई सामग्री की पहचान नहीं करता या
दुर्भावनापूर्ण मंशा सिद्ध नहीं करता।

## सुरक्षित रखने योग्य correlation keys { #correlation-keys-worth-preserving }

| Key | आर्टिफ़ैक्ट |
|---|---|
| पूरा path / फ़ाइल नाम | लगभग हर आर्टिफ़ैक्ट |
| SHA-1 / SHA-256 | Amcache, Defender, recover की गई फ़ाइलें |
| MFT entry + sequence | `$MFT`, Prefetch, LNK, Jump Lists, USN |
| USN | Amcache, `$MFT`, USN Journal |
| Volume serial / droids | Prefetch, LNK, Jump Lists, MFT object IDs |
| SID / उपयोगकर्ता नाम | SRUM, SAM, registry, event logs |
| Task GUID / task नाम | TaskCache, task XML, event logs |
| Service नाम | Services, event logs, process/driver data |
| Defender EntryId / ScanId | क्वारंटीन, Defender history, event logs |
