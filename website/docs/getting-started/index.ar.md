# تثبيت Juntei { #install-juntei }

## الملفات الثنائية للإصدارات { #release-binaries }

نزّل ملف zip المناسب لنظامك من
[صفحة إصدارات Juntei](https://github.com/Yamato-Security/juntei/releases). وإذا لم يُنشر بعدُ ملف
ثنائي للإصدار، فقم بالترجمة من الشيفرة المصدرية كما هو موضح أدناه. احتفظ بالأرشيف أو بإيداع (commit)
الشيفرة المصدرية وبمجموع التحقق (checksum) مع ملاحظات القضية حتى يتمكن محلل آخر من إعادة إنتاج التشغيل.

| النظام | ملف Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` أو `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` أو `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

يحتوي كل ملف zip على الملف الثنائي وحده، مسمّى بالإصدار والمنصة (على سبيل المثال
`juntei-1.0.0-win-x64.exe`): لا يحتاج Juntei إلى أي ملفات أخرى، ولا يحتاج الملف الثنائي لنظام Windows
إلى Visual C++ Redistributable. الملفات الثنائية لنظام Linux بإصدار musl مرتبطة ارتباطًا ثابتًا (statically
linked) وتعمل على أي توزيعة؛ أما إصدارات gnu فتحتاج إلى إصدار حديث من glibc. على macOS، اسمح بالملف
الثنائي الذي نزّلته من System Settings > Privacy & Security، أو أزل سمة الحجر الخاصة به باستخدام
`xattr -d com.apple.quarantine juntei-<version>-mac-*`.

```bash
juntei --version
juntei --help
```

تستخدم الأمثلة `juntei` اسمًا للملف الثنائي؛ أعد تسميته، أو اكتب اسمه الكامل.

## الترجمة من الشيفرة المصدرية { #compile-from-source }

يتطلب Juntei حاليًا إصدار Rust المعلن في `Cargo.toml` (‏Rust 1.99 وقت كتابة هذه الصفحة).

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

يستخدم `--locked` مراجعات التبعيات المسجلة بدقة في `Cargo.lock`.

## الشكل الأساسي للأوامر { #basic-command-shape }

يقبل كل محلل إما ملفًا واحدًا (`-f`) أو مجموعة مجمّعة في مجلد (`-d`):

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

اعرض الخيارات الحالية بدلًا من نسخ أمر قديم من تقرير:

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

تابع إلى [التحقيق الأول](first-investigation.md) للتعرف على سير عمل آمن.
