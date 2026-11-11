# लेख और संदर्भ { #articles-and-references }

यह पृष्ठ उन पाठकों के लिए उच्च-गुणवत्ता वाले शुरुआती स्रोत एकत्र करता है जो आर्टिफ़ैक्ट
मार्गदर्शिकाओं से आगे जाना चाहते हैं। हर आर्टिफ़ैक्ट के **अधिक विवरण** अनुभागों में सबसे लक्षित low-level links हैं।

## शुरुआती लोगों के लिए DFIR परिचय { #beginner-dfir-orientation }

- [SANS Windows Forensic Analysis poster](https://www.sans.org/posters/windows-forensic-analysis/) — सामान्य Windows आर्टिफ़ैक्ट और उनके स्थानों का एक दृश्य मानचित्र।
- [SANS Hunt Evil poster](https://www.sans.org/posters/hunt-evil/) — व्यावहारिक hunting pivots और Windows जाँच के लिए स्मरण-बिंदु।
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — सीखने और parser सत्यापन के लिए सार्वजनिक आर्टिफ़ैक्ट नमूने।
- [Velociraptor artifact reference](https://docs.velociraptor.app/artifact_references/) — कई Windows आर्टिफ़ैक्ट के लिए संग्रह-केंद्रित विवरण और VQL implementations।
- [Plaso दस्तावेज़ीकरण](https://plaso.readthedocs.io/) — कई आर्टिफ़ैक्ट पर आधारित विश्लेषण के लिए timeline अवधारणाएँ और parsers।

## File-system आर्टिफ़ैक्ट { #file-system-artifacts }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) और [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — security descriptors, ACEs और conditional expressions।
- [libfsntfs format दस्तावेज़ीकरण](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Omer Ben-Amram का mft parser](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — INDX record और index-slack carving के साथ NTFS parser।
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — फ़ाइल नामों के लिए `$I30` slack को carve करता है।
- [LogFileParser](https://github.com/jschicht/LogFileParser) — विस्तृत NTFS transaction-log शोध और parsing।
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — `$LogFile`, `$MFT` और USN इतिहास को correlate करता है।

## निष्पादन और inventory आर्टिफ़ैक्ट { #execution-and-inventory-artifacts }

- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — Prefetch format/library पर कार्य।
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)

## उपयोगकर्ता-गतिविधि आर्टिफ़ैक्ट { #user-activity-artifacts }

- [Microsoft Open Specifications: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## रेजिस्ट्री और पर्सिस्टेंस { #registry-and-persistence }

- [Microsoft: Registry value types](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Maxim Suhanov का Windows registry format specification](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) और [libregf](https://github.com/libyal/libregf) — low-level hive parsing और recovery के संदर्भ।
- [Microsoft: Services registry tree](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: Task Scheduler schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender और malware का प्रबंधन { #defender-and-malware-handling }

- [Microsoft: Restore quarantined files in Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [dissect.target Defender plugin](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — एक अलग शोध/extraction implementation।
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/) — प्रबंधन और विश्लेषण संसाधन।

## Juntei parser projects { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`, `$I30`, USN Journal, `$LogFile`, `$Boot`, `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "बाहरी सामग्री बदलती रहती है"
    दावों को प्रभावित Windows version और अपने एकत्र किए गए data के विरुद्ध सत्यापित करें। कोई उपयोगी लेख
    किसी सार्वभौमिक नियम के बजाय किसी विशिष्ट operating-system पीढ़ी, parser version या प्रयोग का
    वर्णन कर सकता है।
