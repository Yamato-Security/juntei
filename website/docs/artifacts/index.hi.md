# आर्टिफ़ैक्ट मार्गदर्शिका { #artifact-guide }

Windows file system, registry, compatibility databases,
user-interface history, resource accounting और सुरक्षा उत्पादों में एक-दूसरे से जुड़े (overlapping) निशान सुरक्षित रखता है। Juntei इन स्रोतों को parse करता है,
लेकिन जाँचकर्ता को फिर भी यह व्याख्या करनी होती है कि हर स्रोत का अर्थ क्या है।

## प्रश्न के आधार पर आर्टिफ़ैक्ट चुनें { #choose-an-artifact-by-question }

| जाँच का प्रश्न | यहाँ से शुरू करें | इनसे पुष्टि करें |
|---|---|---|
| कौन-सी PowerShell कमांड लाइनें सहेजी गई थीं? | [PowerShell इतिहास](powershell-history.md) | PowerShell इवेंट लॉग, प्रोसेस टेलीमेट्री, Prefetch |
| क्या यह executable चला? | [Prefetch](prefetch.md) | [ShimCache](shimcache.md), [SRUM](srum.md), event logs |
| क्या यह फ़ाइल कभी मौजूद थी? | [Amcache](amcache.md), [$MFT](mft.md) | [USN जर्नल](usn-journal.md), [$I30](i30.md), ShimCache |
| कोई फ़ाइल कब बनाई गई, rename की गई या हटाई गई? | [USN जर्नल](usn-journal.md) | [$MFT](mft.md), [$I30](i30.md), Defender क्वारंटीन |
| उपयोगकर्ता ने रीसायकल बिन में क्या delete किया, और कब? | [रीसायकल बिन](recycle-bin.md) | [USN जर्नल](usn-journal.md), [$MFT](mft.md), [LNK](lnk.md), [Jump Lists](jump-lists.md) |
| Metadata बदलाव से पहले कौन-सा नाम या timestamp मौजूद था? | [$LogFile](logfile.md) | [$MFT](mft.md), [USN जर्नल](usn-journal.md), [$I30](i30.md) |
| किसी फ़ाइल का owner कौन है, और उसे कौन बदल सकता है? | [$MFT](mft.md) के साथ [$Secure:$SDS](secure-sds.md) | [SAM](sam.md), Security event logs |
| किसी शॉर्टकट या Prefetch फ़ाइल का target किस volume पर था? | [$Boot](boot.md) | [LNK](lnk.md), [Prefetch](prefetch.md), [USB डिवाइस](usb.md) |
| उपयोगकर्ता ने क्या खोला? | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| क्या किसी प्रोग्राम ने बड़ी मात्रा में data transfer किया? | [SRUM](srum.md) | Firewall, proxy, DNS, packet या EDR telemetry |
| उपयोगकर्ता ने किन websites पर visit किया, और क्या download किया? | [वेब ब्राउज़र](browser.md), [Internet Explorer और लेगेसी Edge](webcache.md) | Proxy और DNS logs, `$MFT` और USN Journal, Defender, SRUM |
| उपयोगकर्ता ने किसके साथ e-mail का आदान-प्रदान किया, और उनके साथ क्या attach था? | [Outlook](outlook.md) | Mail server और gateway logs, `$MFT` और USN Journal, Defender, LNK और Jump Lists |
| किन clients और accounts ने इस server का उपयोग किया, और किन दिनों में? | [उपयोगकर्ता एक्सेस लॉगिंग (UAL)](ual.md) | Security event logs, SMB और RDP logs, firewall logs |
| क्या persistence configure किया गया था? | [सेवाएँ](services.md), [TaskCache](taskcache.md), [रेजिस्ट्री](registry.md) | Event logs, Prefetch, `$MFT` |
| Defender ने क्या हटाया? | [Defender क्वारंटीन](defender-quarantine.md) | Event logs, `$MFT`, USN Journal |
| क्या local accounts जोड़े या बदले गए? | [SAM](sam.md) | Security event logs और registry transaction history |

## Fields से पहले चेतावनियाँ पढ़ें { #read-warnings-before-fields }

हर आर्टिफ़ैक्ट पेज एक ही क्रम का पालन करता है:

1. **यह क्या है** और यह किन प्रश्नों का उत्तर दे सकता है।
2. **इसे कहाँ पाएँ** और संग्रह इसे कैसे प्रभावित करता है।
3. **यह क्या सिद्ध करता है—और क्या सिद्ध नहीं करता।**
4. Juntei से **इसे कैसे parse करें**।
5. **हर आउटपुट field को कैसे पढ़ें।**
6. स्वतंत्र साक्ष्य से **इसे कैसे correlate करें**।
7. Low-level शोध के लिए **अधिक जानकारी**।

!!! danger "Parser finding को दुर्भावनापूर्णता के बराबर न मानें"
    Juntei findings समीक्षा योग्य patterns की पहचान करते हैं, जैसे `System32` के बाहर
    किसी system-binary का नाम। Installers, administrators, compatibility layers और software updates
    असामान्य records बना सकते हैं। Context और स्वतंत्र साक्ष्य से सत्यापित करें।
