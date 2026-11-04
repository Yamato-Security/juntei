# الوصول عن بُعد (جهة المستخدم) { #remote-access-user-side }

## ما هو { #what-it-is }

يحتفظ ملف `NTUSER.DAT` الخاص بالمستخدم بآثار الاتصالات البعيدة التي أجراها المستخدم من الحاسوب. وهذه
هي جهة المصدر في الحركة الجانبية:

- **سطح المكتب البعيد (`mstsc`):**
  - الحواسيب التي كُتبت في العميل، والأحدث أولًا؛
  - مفتاح واحد لكل حاسوب سجّل المستخدم الدخول إليه، مع اسم الحساب المستخدم وبصمة (thumbprint)
    شهادة الخادم التي قبلها المستخدم؛
  - الحواسيب التي سمح لها المستخدم باستخدام الأجهزة المحلية.
- **PuTTY:**
  - الجلسات المحفوظة: المضيف، والمنفذ، والحساب، والبروتوكول، وملف المفتاح، والوكيل، وإعادة توجيه المنافذ؛
  - الجلسات التي بدأت أخيرًا؛
  - كل مفتاح خادم SSH قبله المستخدم.
- **WinSCP:** الجلسات المحفوظة ومفاتيح خوادم SSH المقبولة، بالصيغة نفسها المستخدمة في PuTTY.
- **محركات الأقراص المعيّنة:** كل محرك أقراص عُيّن ليُستعاد عند تسجيل الدخول، والمسارات التي كُتبت في
  *Map Network Drive*.
- **المشاركات:** المشاركات التي ركّبها Explorer ‏(`MountPoints2`).

وهي تُظهر إلى أين انتقل الحساب بعد ذلك وبأي بيانات اعتماد، حتى بعد زوال السجلات الخاصة بالحاسوب البعيد
نفسه.

<span class="evidence-strength">أفضل استنتاج:</span> اتصل ملف تعريف هذا المستخدم بهذا الهدف، أو حفظ
اتصالًا به. ولا تثبت الجلسة المحفوظة أو الاسم المكتوب وحدهما أن الاتصال نجح. أما مفتاح المضيف المقبول
أو مفتاح `Servers` الذي يحتوي على `UsernameHint` فأقوى دلالة: فقد وصل العميل إلى الخادم.

## الموقع { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Terminal Server Client\Default            MRU0 … MRU9
Software\Microsoft\Terminal Server Client\Servers\<host>     UsernameHint, CertHash
Software\Microsoft\Terminal Server Client\LocalDevices       <host> = devices
Software\SimonTatham\PuTTY\Sessions\<session>
Software\SimonTatham\PuTTY\Jumplist                          Recent sessions
Software\SimonTatham\PuTTY\SshHostKeys                       <type>@<port>:<host> = key
Software\Martin Prikryl\WinSCP 2\Sessions\<session>
Software\Martin Prikryl\WinSCP 2\SshHostKeys
Network\<letter>                                             RemotePath, UserName, ProviderName
Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU
Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##server#share
```

اجمع `NTUSER.DAT` مع سجلات المعاملات الخاصة به لكل ملف تعريف. واجمع خلية `SOFTWARE` الخاصة بالنظام
أيضًا: فهي توفر معرّف SID لكل مستخدم.

## تحذيرات للمحلل { #analyst-cautions }

- **أوقات المفاتيح هي أوقات القوائم.** تحتوي `Default` و`SshHostKeys` و`Map Network Drive MRU` على
  قوائم، وآخر كتابة لمفتاحها هي وقت آخر تغيير للقائمة. ويعود ذلك الوقت إلى العنصر الأول
  (`MruPosition` 0). أما بالنسبة للعناصر الأخرى فهو مجرد حد أعلى. ولكل من مفتاح `Servers\<host>` أو
  الجلسة أو `Network\<letter>` أو المشاركة وقته الخاص.
- **المحفوظ ليس مستخدمًا بالضرورة.** يمكن حفظ جلسة PuTTY أو WinSCP دون الاتصال أبدًا. يعيد PuTTY كتابة
  `Recent sessions` عند بدء جلسة محفوظة، ويضيف مفتاح مضيف عند الوصول إلى خادم لأول مرة.
- **كلمات المرور:**
  - يمكن لـ WinSCP تخزين كلمات مرور الجلسات والأنفاق والوكلاء.
  - لا يخزن PuTTY سوى كلمة مرور الوكيل، بنص صريح.
  - يبيّن `PasswordStored` و`StoredPasswords` في `Details` القيم المضبوطة. ولا يُفك ترميزها أو تُكتب
    في الإخراج أبدًا.
- **يُسقط WinSCP القيم الافتراضية.** الجلسة التي لا تحتوي على `FSProtocol` هي SFTP، والجلسة التي لا
  تحتوي على `PortNumber` تستخدم المنفذ الافتراضي للبروتوكول. ويبقى `Protocol` و`Port` حينئذٍ فارغين
  بدلًا من التخمين.
- **بصمات مفاتيح المضيف** هي بصمات `SHA256:…` الخاصة بـ OpenSSH، كما يعرضها `ssh-keygen -l` و`ssh`
  وPuTTY 0.75 والإصدارات الأحدث. يحسبها Juntei من المفتاح الذي يحتفظ به PuTTY وWinSCP. قارنها بمفتاح
  الخادم نفسه وملفات `known_hosts` والسجلات لتحديد الخادم الكامن وراء اسم أو عنوان IP. ولا يملك مفتاح
  SSH-1 ‏(`rsa`) بصمة.
- **محركات الأقراص المعيّنة:** لا يُضبط `RemoteUsername` إلا عندما يكون محرك الأقراص قد عُيّن بحساب
  آخر؛ ويكتب Windows قيمة `REG_DWORD` بمقدار 0 عندما يستخدم حساب المستخدم نفسه.
- **المشاركات:** يحتوي `MountPoints2` أيضًا على مفاتيح لوحدات التخزين وأحرف محركات الأقراص. ولا يُبلَغ
  هنا إلا عن مشاركات الشبكة (`##server#share`)؛ و`\\server@SSL@443\DavWWWRoot` مشاركة WebDAV.
- **الإزالة:** لا يترك مسح القوائم، أو استخدام عميل محمول، أي أثر هنا. قارن مع Jump Lists وPrefetch
  وملفات `.rdp` وذاكرة التخزين المؤقت للصور النقطية في RDP ‏(RDP bitmap cache) وسجلات الهدف.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry remote-access -d evidence/ -o remote.csv
juntei parse-registry remote-access -f NTUSER.DAT -t jsonl \
  | jq 'select(.Client == "rdp" or .HostKeyFingerprint != null)'
```

يحصي الملخص الاتصالات حسب العميل. ويسرد الاتصالات التي كُتبت مفاتيحها أخيرًا، الأحدث أولًا، والجلسات
التي تحتفظ بكلمة مرور.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `User`، `UserSID`، `SourcePath` | المضيف، واسم مجلد ملف التعريف، ومعرّف SID الخاص بالمستخدم من `ProfileList` في النظام (مع خلية `SOFTWARE` الخاصة بالنظام)، والخلية. |
| `Client` | `rdp` أو `putty` أو `winscp` أو `mapped_drive` أو `network_share`. |
| `Kind` | مصدر السجل: `rdp_mru`، `rdp_server`، `rdp_local_devices`، `putty_session`، `putty_host_key`، `winscp_session`، `winscp_host_key`، `network_drive`، `map_network_drive_mru`، `mount_point`. |
| `Target`، `Port` | الحاسوب (اسم المضيف أو عنوان IP) والمنفذ، أو المشاركة (`\\server\share`). ولا يُعطى المنفذ إلا حيث كتبه Windows أو العميل. |
| `RemoteUsername` | الحساب المستخدم على الهدف: `UsernameHint`، أو `UserName` الخاص بالجلسة (أو جزء `user@` من `HostName` في PuTTY)، أو `UserName` الخاص بمحرك أقراص معيّن. |
| `Protocol` | `Protocol` في PuTTY ‏(`ssh`، `telnet`، `raw`، …)، أو `FSProtocol` في WinSCP ‏(`sftp`، `scp`، `ftp`، `webdav`، `s3`)، أو موفر محرك الأقراص المعيّن (`Microsoft Windows Network`). |
| `MRU-Position` | الموضع في قائمته (0 هو الأحدث): `MRU<n>`، `MRUList`، `Recent sessions` في PuTTY. |
| `SessionName` | اسم الجلسة المحفوظة، مع فك ترميز `%20` وغيرها من رموز الهروب. |
| `HostKeyType`، `HostKeyFingerprint`، `HostKey` | نوع مفتاح مضيف SSH المقبول (`rsa2`، `ssh-ed25519`، `ecdsa-sha2-nistp256`، …)، وبصمته بصيغة OpenSSH، وأرقامه كما خُزّنت. |
| `DriveLetter` | حرف محرك الأقراص المعيّن. |
| `PasswordStored` | ما إذا كانت الجلسة تحتفظ بكلمة مرور؛ ولا يُفك ترميزها أبدًا. |
| `Details` | إعدادات أخرى، بصيغة `Name=Value; …` ‏(CSV) أو بوصفها كائنًا (JSON): `CertHash` لشهادة RDP، و`LocalDevices`، وملف مفتاح الجلسة، والوكيل، ونفق SSH، وإعادة توجيه المنافذ، والأمر البعيد والمجلدات، و`StoredPasswords`، و`Label` للمشاركة، و`ConnectionType` لمحرك الأقراص. |
| `KeyPath`، `ValueName` | المفتاح، والقيمة للسجل الذي يمثّل قيمة واحدة. |
| `KeyLastWrittenUTC` | آخر كتابة للمفتاح (انظر التحذيرات المتعلقة بالقوائم). |
| `DataRaw` | بايتات القيمة (بالنظام الست عشري)، عندما لا تكون سلسلة نصية أو يتعذر فك ترميزها. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | قيم من النوع الخاطئ (مع بايتاتها)، ومفاتيح مضيف تتعذر قراءتها، وأحرف في `MRUList` غير موجودة، وقيم لا يسردها. |

## استراتيجية التحقيق { #investigation-strategy }

1. اسرد أهداف كل مستخدم حسب العميل، وقارنها بالأنظمة التي يُفترض أن يصل إليها ذلك المستخدم. وابحث عن
   المشاركات الإدارية (`C$`، `ADMIN$`) وعناوين IP.
2. دوّن الحسابات المستخدمة (`RemoteUsername`): مسؤولو المجال والحسابات المحلية على أجهزة أخرى.
3. طابق `HostKeyFingerprint` مع مفاتيح الخوادم، و`CertHash` مع شهادات خوادم RDP.
4. اربط النتائج مع أحداث تسجيل الدخول على الأهداف (4624 من النوع 10 و3، و1149)، وJump Lists وملفات
   LNK ‏(`mstsc.exe`، `putty.exe`، والملفات المفتوحة من المشاركات)، وPrefetch، وذاكرة التخزين المؤقت
   للصور النقطية في RDP.
5. استخدم `parse-registry read --recover-deleted` للإدخالات المحذوفة، والنسخ الأقدم من `NTUSER.DAT`
   الخاص بملف التعريف (Volume Shadow Copies، والنسخ الاحتياطية) للقوائم الممسوحة.

## مزيد من التفاصيل { #further-details }

- [Microsoft: إزالة الإدخالات من مربع Computer في Remote Desktop Connection](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [الأسئلة الشائعة لـ PuTTY: أين يخزن PuTTY بياناته؟](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: تخزين التكوين](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
