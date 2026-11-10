# المقالات والمراجع { #articles-and-references }

تجمع هذه الصفحة نقاط انطلاق عالية الجودة للقراء الذين يرغبون في التعمق أكثر مما تقدمه أدلة الآثار
الرقمية. وتحتوي أقسام **مزيد من التفاصيل** في صفحة كل أثر رقمي على الروابط منخفضة المستوى الأكثر تحديدًا.

## مدخل إلى DFIR للمبتدئين { #beginner-dfir-orientation }

- [ملصق SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/) — خريطة مرئية للآثار الرقمية الشائعة في Windows ومواقعها.
- [ملصق SANS Hunt Evil](https://www.sans.org/posters/hunt-evil/) — نقاط محورية عملية للبحث عن التهديدات وتذكيرات خاصة بالتحقيق في Windows.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — عينات آثار رقمية عامة للتعلم والتحقق من المحللات.
- [مرجع الآثار الرقمية في Velociraptor](https://docs.velociraptor.app/artifact_references/) — أوصاف موجهة نحو الجمع وتطبيقات VQL للعديد من آثار Windows الرقمية.
- [وثائق Plaso](https://plaso.readthedocs.io/) — مفاهيم الجداول الزمنية والمحللات للتحليل عبر عدة آثار رقمية.

## آثار نظام الملفات الرقمية { #file-system-artifacts }

- [Microsoft: جدول الملفات الرئيسي (Master File Table)](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [المواصفات المفتوحة من Microsoft: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: دفاتر يومية التغيير (Change Journals)](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: واصفات الأمان](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) و[SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [المواصفات المفتوحة من Microsoft: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — واصفات الأمان وإدخالات ACE والتعبيرات الشرطية.
- [وثائق تنسيق libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [محلل mft من Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — محلل NTFS يدعم استخلاص (carving) سجلات INDX والمساحة الفائضة (slack) في الفهارس.
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — يستخلص أسماء الملفات من المساحة الفائضة في `$I30`.
- [LogFileParser](https://github.com/jschicht/LogFileParser) — بحث مفصل في سجل معاملات NTFS وتحليله.
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — يربط بين `$LogFile` و`$MFT` وسجل USN.

## آثار التنفيذ والجرد الرقمية { #execution-and-inventory-artifacts }

- [أداة AmcacheParser من Eric Zimmerman](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — أعمال على تنسيق Prefetch ومكتبته.
- [أداة PECmd من Eric Zimmerman](https://github.com/EricZimmerman/PECmd)
- [Mandiant: الاستفادة من ذاكرة التخزين المؤقت لتوافق التطبيقات في التحقيقات الجنائية](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [أداة AppCompatCacheParser من Eric Zimmerman](https://github.com/EricZimmerman/AppCompatCacheParser)

## آثار نشاط المستخدم الرقمية { #user-activity-artifacts }

- [المواصفات المفتوحة من Microsoft: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [المواصفات المفتوحة من Microsoft: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [أداة LECmd من Eric Zimmerman](https://github.com/EricZimmerman/LECmd)
- [أداة JLECmd من Eric Zimmerman](https://github.com/EricZimmerman/JLECmd)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [أداة SrumECmd من Eric Zimmerman](https://github.com/EricZimmerman/Srum)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## سجل Windows والاستمرارية { #registry-and-persistence }

- [Microsoft: أنواع قيم سجل Windows](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [مواصفة تنسيق سجل Windows من Maxim Suhanov](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) و[libregf](https://github.com/libyal/libregf) — مراجع منخفضة المستوى لتحليل الخلايا واستردادها.
- [وثائق Registry Explorer / RECmd من Eric Zimmerman](https://ericzimmerman.github.io/)
- [Microsoft: شجرة الخدمات في سجل Windows](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: مخطط Task Scheduler](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [المواصفات المفتوحة من Microsoft: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender والتعامل مع البرامج الضارة { #defender-and-malware-handling }

- [Microsoft: استعادة الملفات المحجورة في Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [المكوّن الإضافي Defender في dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — تطبيق بحثي/استخراجي منفصل.
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/) — موارد للتعامل مع البرامج الضارة وتحليلها.

## مشاريع محللات Juntei { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`، `$I30`، USN Journal، `$LogFile`، `$Boot`، `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "المواد الخارجية تتغير"
    تحقق من صحة الادعاءات مقارنةً بإصدار Windows المعني والبيانات التي جمعتها. فقد تصف مقالة مفيدة
    جيلًا معينًا من نظام التشغيل، أو إصدارًا معينًا من المحلل، أو تجربة بعينها، بدلًا من قاعدة عامة.
