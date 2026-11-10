# قائمة الأوامر { #command-list }

تُجمَّع المحللات الفردية حسب نوع مصدرها. شغّل `juntei --help`
أو `juntei <SOURCE> --help` أو `juntei <SOURCE> <ARTIFACT> --help` لاستكشافها:

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

لمجموعة أدلة Windows مجمعة من عدة حواسيب، استخدم
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md). فهو يستدعي الأوامر المجمعة نفسها،
وينظّم النتائج والسجلات حسب الحاسوب وفئة التحقيق.

للحصول على مخطط زمني تم فرزه زمنيًا عبر العناصر التي تم تحليلها، استخدم [`juntei super-timeline -d <PARSED> -o SuperTimeline.csv`](super-timeline.md). يقوم بتوسيع وتسمية معاني الطابع الزمني؛ يستمر `auto-parse -j` في الاحتفاظ بالسجلات الأصلية.

## [صور الأقراص](mount.md) { #disk-images }

| الأمر | المُدخل | الدليل التفصيلي |
|---|---|---|
| `mount` | صورة قرص VHDX / E01 / Ex01 ‏ | [تحميل صور الأقراص](mount.md) |

## [أوامر ESE](user-activity.md) { #ese-commands }

| الأمر | الأثر الرقمي المُدخل | الدليل التفصيلي |
|---|---|---|
| `parse-ese srum` | ملف ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ملفات ESE `Current.mdb` و`{GUID}.mdb` و`SystemIdentity.mdb` | [تسجيل وصول المستخدمين (UAL)](../artifacts/ual.md) |
| `parse-ese webcache` | ملف ESE `WebCacheV01.dat` | [Internet Explorer وEdge القديم](../artifacts/webcache.md) |

## [أوامر الملفات](execution.md) { #file-commands }

| الأمر | الأثر الرقمي المُدخل | الدليل التفصيلي |
|---|---|---|
| `parse-file defender-quarantine` | شجرة حجر Defender أو أحد إدخالاتها | [حجر Defender](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`، `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [اختصارات LNK](../artifacts/lnk.md) |
| `parse-file outlook` | `*.pst`، `*.ost`، `*.msg`، `*.oft` | [Outlook](../artifacts/outlook.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [أوامر NTFS](filesystem.md) { #ntfs-commands }

| الأمر | الأثر الرقمي المُدخل | الدليل التفصيلي |
|---|---|---|
| `parse-ntfs boot` | `$Boot`، أو صورة وحدة تخزين NTFS | [قطاع الإقلاع ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`)، أو صورة قرص | [فهارس المجلدات ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | الملف `$LogFile` في NTFS | [سجل معاملات NTFS](../artifacts/logfile.md) |
| `parse-ntfs mft` | الملف `$MFT` في NTFS | [جدول الملفات الرئيسي](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [واصفات الأمان ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [دفتر يومية USN](../artifacts/usn-journal.md) |

## [أوامر سجل Windows](security-registry.md) { #registry-commands }

| الأمر | الأثر الرقمي المُدخل | الدليل التفصيلي |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`، `SYSTEM` | [الشبكات](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [أسرار LSA](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (و`BCD-Template`) | [بيانات تكوين الإقلاع (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`، `NTUSER.DAT` | [توافق التطبيقات](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [نهج التدقيق](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage وRecentApps والعناصر المثبتة على شريط المهام](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [سجل التطبيقات](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [قوائم العناصر المستخدمة مؤخرًا (MRU)](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (مع `SAM` و`SOFTWARE` لأسماء المستخدمين) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (`NTUSER.DAT` في Windows XP) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`، `NTUSER.DAT` | [البرامج المثبتة](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [مربعات حوار الفتح والحفظ باسم](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`، `SYSTEM`، `NTUSER.DAT`، `UsrClass.dat` | [التشغيل التلقائي عبر سجل Windows](../artifacts/persistence.md) |
| `parse-registry read` | أي خلية أساسية من خلايا سجل Windows | [خلايا سجل Windows](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (مع `SOFTWARE` لمعرّفات SID الخاصة بالمستخدمين) | [الوصول عن بُعد (جهة المستخدم)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [حسابات SAM المحلية](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`، `SYSTEM`، `NTUSER.DAT` | [إعدادات الأمان](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [خدمات Windows](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` أو قيمة AppCompatCache مُصدَّرة | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | `SYSTEM` و`SOFTWARE` و`SAM` و`SECURITY` لنظام واحد | [معلومات النظام](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [ذاكرة التخزين المؤقت لجدولة المهام](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`، مع `SOFTWARE` و`NTUSER.DAT` | [أجهزة USB](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`، `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## [أوامر SQLite](sqlite.md) { #sqlite-commands }

| الأمر | الأثر الرقمي المُدخل | الدليل التفصيلي |
|---|---|---|
| `parse-sqlite browser` | `History`، `Cookies`، `Web Data`، `places.sqlite`، `cookies.sqlite`، `formhistory.sqlite` | [متصفحات الويب](../artifacts/browser.md) |

## أنواع المصادر وتغييرات التنسيق { #source-types-and-format-changes }

تصف المجموعات التنسيقات التي يدعمها Juntei حاليًا. يقرأ `parse-registry amcache` الملف
`Amcache.hve`؛ ولا يعني ذلك دعم قواعد بيانات جرد Amcache بتنسيق SQLite. وتظل حزم (crates)
محللات الآثار الرقمية منفصلة، بحيث يمكن إضافة دعم تنسيق تخزين آخر بشكل مستقل.

استُبدلت أسماء المحللات السابقة ذات المستوى الواحد بمسارات المصدر والأثر الرقمي الموضحة أعلاه.
ضع خيارات المحلل الحالية بعد اسم الأثر الرقمي كما هي. ويحتفظ `auto-parse` والأوامر الفرعية
الحالية لسجل Windows بأسمائها.

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

يبحث `juntei persistence-hunter` عبر عدة تنسيقات مصدرية، ولذلك فهو أمر تحليل من المستوى الأعلى.
وهو يكمّل الجرد الكامل للتكوين الذي يوفره `parse-registry persistence` بما يلي: 131 فحصًا،
وتصنيف السماح/الحظر، والبيانات الوصفية للحمولة، والتغطية، والربط بين الطوابع الزمنية.

## الاستخدام على المستوى الأعلى { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX or E01 disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  parse-sqlite        Parse SQLite databases
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

تعرض صفحات المصادر طريقة استخدام الأوامر وتقدم أمثلة قصيرة. أما الأدلة الإرشادية للآثار الرقمية
فتشرح الأدلة، وحقول الإخراج، والقيود، واستراتيجية التحقيق.
