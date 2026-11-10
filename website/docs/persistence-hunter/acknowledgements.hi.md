# Persistence Hunter आभार { #persistence-hunter-acknowledgements }

Juntei का `persistence-hunter` command community द्वारा persistence hunting की दो पीढ़ियों पर आधारित है।
उन लेखकों और योगदानकर्ताओं का धन्यवाद जिन्होंने अपनी techniques, implementations और शोध
investigators के लिए उपलब्ध कराए।

## PyrsistenceSniper: Python project { #pyrsistencesniper-the-python-project }

[Hexastrike](https://github.com/Hexastrike) का [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper)
इस feature के लिए प्रत्यक्ष संदर्भ है। Juntei इसके
130-check catalogue को, default allowlist, LOLBAS snapshot और
classification व्यवहार के साथ, native Rust में port करता है। संदर्भ संस्करण
[commit `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f) पर pinned है।

Python project MIT licensed है। Juntei इसकी copyright और license notice को
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt) में सुरक्षित रखता है।
Juntei का implementation Python interpreter के बिना offline चलता है। इसकी collection handling,
coverage reporting और अपूर्ण साक्ष्य की सतर्क व्याख्या
[command guide](../commands/persistence-hunter.md) में documented है।

## PersistenceSniper: मूल PowerShell tool { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) मूल PowerShell
persistence hunting project है जिसने PyrsistenceSniper को प्रेरित किया। PyrsistenceSniper
Federico Lagrasta को श्रेय देता है; मूल repository Riccardo Ancarani और अन्य
योगदानकर्ताओं को भी श्रेय देती है। Windows persistence
mechanisms की जाँच को आसान बनाने के लिए हम मूल project और उसके योगदानकर्ताओं का धन्यवाद करते हैं।

Juntei का प्रत्यक्ष porting संदर्भ Python project है। मूल PowerShell tool को
उस वंश-परंपरा के लिए श्रेय दिया गया है। Juntei मूल project के
[scheduled-task COM resolution PR #31](https://github.com/last-byte/PersistenceSniper/pull/31)
और [Start Menu library/CLSID-folder PR #32](https://github.com/last-byte/PersistenceSniper/pull/32) में
[hongson11698](https://github.com/hongson11698) द्वारा प्रस्तावित mechanisms को भी स्वतंत्र रूप से implement करता है।
ये Python संदर्भ catalogue में जोड़ते हैं; इसका यह अर्थ नहीं है कि किसी भी upstream project की हर live-system feature
Juntei के offline command द्वारा implement की गई है।

## सहायक शोध और tools { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/) catalogue में मौजूद technique identifiers प्रदान करता है।
  Technique mapping किसी mechanism के संभावित उपयोग का वर्णन करती है; मेल खाने वाली registry value
  या file अपने आप में ATT&CK-पुष्ट हमला नहीं है।
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  को भी PyrsistenceSniper ने Windows autostarts की जाँच के लिए प्रेरणा के रूप में स्वीकार किया है।
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) उन वैध Windows binaries को document करता है जिनका
  दुरुपयोग किया जा सकता है। इनमें से किसी binary को पहचानना किसी command की व्याख्या में मदद करता है, लेकिन उस
  command को स्वचालित रूप से दुर्भावनापूर्ण नहीं बनाता।

पूरे Juntei में उपयोग किए गए registry, NTFS, Shell Link, event-log और अन्य parser आधारों के लिए,
[project-व्यापी आभार](../acknowledgements.md) देखें।

[पर्सिस्टेंस तंत्र मार्गदर्शिका](index.md) पर लौटें।
