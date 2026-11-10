# شكر وتقدير { #acknowledgements }

يوجد Juntei لأن مجتمعات DFIR والمصادر المفتوحة نشرت محللات، وأبحاثًا حول التنسيقات، ومخرجات مرجعية،
ومواصفات، وأدلة اختبار. وقد أسهمت المشاريع أدناه في التنفيذ أو التحقق أو تسمية الحقول أو تحديد
المخرجات المتوقعة للمحلل. ولا يعني إدراج مشروع ما أن شيفرته مضمّنة في Juntei؛ راجع كل مشروع لمعرفة ترخيصه.

## أسس المحللات التي تصونها Yamato Security { #parser-foundations-maintained-by-yamato-security }

| مجال Juntei | المحلل المُصان | المشروع الأصلي/الأساس |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (حزمة `mft`) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| دفتر يومية USN | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (حزمة `usn`) | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (حزمة `logfile`) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| فهارس المجلدات (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (حزمة `mft`، الوحدة `i30`) | أبحاث المساحة الراكدة (slack) في الفهارس المذكورة أدناه |
| NTFS `$Boot`، `$Secure:$SDS` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (الحزمتان `boot` و`secure`) | مواصفة MS-DTYP من Microsoft وأبحاث NTFS المذكورة أدناه |
| SRUM، UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) و[ese](https://github.com/Sootmark/ese) |
| متصفحات الويب / SQLite | [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) | [Sootmark browser](https://github.com/Sootmark/browser) و[sqlite](https://github.com/Sootmark/sqlite) و[plist](https://github.com/Sootmark/plist) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | أعمال Sootmark shell حول الملفات المركبة وJump Lists |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| صور أقراص VHDX ‏(`mount`, `auto-parse --vhdx`) | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) و[ntfs-core](https://github.com/SecurityRonin/ntfs-forensic)، ويُقدَّم ذلك عبر [fuser](https://github.com/cberner/fuser) | مواصفة MS-VHDX من Microsoft |
| E01 / Ex01 (`mount`, `auto-parse --e01`) | [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) | [ewf-image](https://github.com/ebrig/ewf-image) (Apache-2.0); [libewf](https://github.com/libyal/libewf) (EWF1/EWF2) |
| سجل Windows | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | أبحاث تنسيق سجل Windows واستعادته المذكورة أدناه |

## الأدوات المرجعية والتحقق المستقل { #reference-tools-and-independent-validation }

- قدّم مشروع [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) من Hexastrike
  فهرس فحوصات الاستمرارية، وقائمة السماح الافتراضية، ولقطة LOLBAS، والسلوك المرجعي للأمر
  `persistence-hunter` المكتوب أصلًا بلغة Rust. ويُحتفظ بترخيص MIT الخاص به وإشعار حقوق النشر في
  `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`. وقد ألهم مشروع
  [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) الأصلي مشروعَ Python؛
  راجع [شكر وتقدير Persistence Hunter](persistence-hunter/acknowledgements.md) للاطلاع على سلسلة النسب الكاملة.

- أسهمت **مواصفة regf** وأداة **yarp** من Maxim Suhanov في بنى سجل Windows، وإعادة تشغيل سجلات
  المعاملات، واستعادة السجلات المحذوفة؛ وكانت أداته **dfir_ntfs** المرجع لسجلات INDX واستخراج
  البيانات من المساحة الراكدة (slack) في الفهارس.
- تُعد أداة **INDXRipper** من Harel Segev مرجعًا إضافيًا لاستخراج أسماء الملفات من المساحة الراكدة في الفهارس.
- أسهمت **dfir_ntfs**، وأداة **LogFileParser** من Joakim Schicht، وأداة **ntfs-linker** من Stroz Friedberg
  في التحقق من `$LogFile` وتفسيره؛ وقدّمت مكتبة **go-ntfs** من Velocidex أدلة اختبار NTFS عامة.
- توفر مشاريع Joachim Metz / libyal ‏**libregf** و**libfsntfs** و**libscca** و**libfwsi**
  و**libesedb** أبحاثًا تفصيلية حول التنسيقات ومراجع تحليل مستقلة.
- قدّمت **dissect.esedb** و**dissect.ntfs** و**dissect.target** من Fox-IT تفسيرات مستقلة لـ SRUM وUAL
  و`$Secure:$SDS` وSAM وDefender استُخدمت أثناء التحقق.
- كانت أداة `fsstat` من **The Sleuth Kit** المرجعَ لقطاعات الإقلاع في NTFS لوحدات التخزين المنشأة باستخدام
  `mkntfs`.
- حدّدت بعض أدوات Eric Zimmerman أسماء الحقول وتوقعات الفاحصين المستخدمة في المقارنات.
- قدّمت **regipy** و**python-registry** و**Notatin** و**artemis** تطبيقات لسجل Windows،
  أو اختبارات، أو تفسيرات لعينات.
- قدّمت **LnkParse3**، وأداتا **lnk_parser** و**jumplist_parser** من AbdulRhmanAlfaifi، وأداة JLECmd
  سلوكًا مستقلًا لـ LNK/Jump Lists وحالات اختبار عامة.
- أسهم مشروع **prefetch-forensic** من Security Ronin ومشروع **forensicnomicon** في ملاحظات Juntei
  عالية القيمة حول Prefetch/Amcache/ShimCache.
- توفر **defender-dump** وdissect.target تطبيقين منفصلين لفهم بنى حجر Defender
  وسلوك الاستخراج الآمن.
- قدّمت **Plaso** و**Chainsaw** و**Velociraptor** مقارنات للمحللات، أو أدلة اختبار، أو سلوكًا
  مرجعيًا موجهًا نحو الجمع عبر عدة آثار رقمية.
- كانت أداة **Hindsight** من Obsidian Forensics وتعريفات **SQLiteHunter** من Velocidex مرجعين لسجل
  التصفح والتنزيلات وملفات تعريف الارتباط وسجل النماذج.

## المواصفات والأبحاث { #specifications-and-research }

نشكر Microsoft على نشر المواصفات المفتوحة (Open Specifications) الخاصة بـ Shell Links، والملفات الثنائية المركبة،
وبروتوكولات NTFS/SAM، وبنى سجل Windows/الخدمات/المهام، ووثائق واجهات API. كما نشكر
مؤلفي الأبحاث الجنائية حول AppCompatCache وAmcache وSRUM وPrefetch وسجل Windows وNTFS، التي جعلت
هذه الآثار الرقمية مفهومة للممارسين.

## أدلة الاختبار { #test-evidence }

تتيح الأدلة العامة من
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) الخاص بـ Andrew Rathbun، وNIST CFReDS، وPlaso،
وRegipy، وArtemis، ومستودعات اختبار المحللات، وغيرها من المجموعات المرخصة على النحو الصحيح، إجراءَ اختبارات
الانحدار على بيانات Windows حقيقية. ويحتفظ Juntei بملاحظات الإسناد/الترخيص بجانب ملفات
الاختبار المضمّنة.

## المساهمون { #contributors }

شكرًا لكل مساهم في Juntei وفي محللات Yamato Security، ولكل مراجع ومُبلّغ عن مشكلة ومستخدم
قدّم حالات حدّية. ويُدرج المساهمون الحاليون حسب GitHub في
[صفحة مساهمي Juntei](https://github.com/Yamato-Security/juntei/graphs/contributors).

## الترخيص { #license }

يُصدر Juntei بموجب
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5)، وحقوق النشر لشركة Shirofune Security Co., Ltd. وقد نُشرت شيفرته المصدرية بموجب
AGPLv3 قبل الإصدار 1.0.0. وتوجد تراخيص الحزم وغيرها من الشيفرات والبيانات التابعة لجهات خارجية التي
بُني بها في
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
