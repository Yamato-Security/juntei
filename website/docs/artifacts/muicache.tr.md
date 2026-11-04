# MuiCache { #muicache }

## MuiCache nedir { #what-muicache-is }

Bir kullanıcı bir program başlattığında Windows, kabuğun göstermesi gereken ad için programın sürüm kaynaklarını
okur (`FileDescription`; örneğin `cmd.exe` için "Windows Command Processor") ve bunu kullanıcı başına
`MuiCache` içinde önbelleğe alır. Windows 8 ve sonrası şirketi de (`CompanyName`) önbelleğe alır.
Önbellek, programın tam yoluyla anahtarlanır:

- **Windows 8 ve sonrası:** program başına iki değer, `<path>.FriendlyAppName` ve
  `<path>.AppCompany`.
- **Windows Vista ve 7:** program başına bir değer, kolay anlaşılır adı tutan `<path>`.
- **Windows XP:** aynı biçim, `NTUSER.DAT` içinde.

MuiCache, çıkarılabilir sürücülerdeki, ağ paylaşımlarındaki ve artık var olmayan klasörlerdeki programlar da
dahil olmak üzere bir kullanıcının kabuk oturumu için program yürütme ipuçları sağlayabilir; bunları dosyanın
kendisinin verdiği ad ve şirketle birlikte sunar.

<span class="evidence-strength">En iyi sonuç:</span> bu kabuk profili bir dosyanın yolunu,
açıklamasını ve şirketini önbelleğe almıştır. Bunu diğer yürütme artefaktlarıyla ilişkilendirin. Profil,
işlemin hangi hesap olarak çalıştığını belirlemez. MuiCache program başına bir zaman veya çalıştırma sayısı içermez.

## Konum { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (önbelleğe alınmış dizelerin dili) bir program değildir ve dışarıda bırakılır. `@<file>,-<number>`
(`@shell32.dll,-22019`) biçiminde adlandırılan değerler, kabuğun kendi menüleri ve dosya türleri için önbelleğe aldığı
kaynak dizeleridir, program değildir; bunlar dışarıda bırakılır ve özette sayılır. Windows Vista
ve sonrasında `NTUSER.DAT` içinde MuiCache bulunmaz.

Her profil için `UsrClass.dat` (ve `NTUSER.DAT`) kovanlarını işlem günlükleriyle birlikte, ana bilgisayar ve kullanıcı adları
için de sistemin `SYSTEM` ve `SOFTWARE` kovanlarını toplayın.

## Analist için uyarılar { #analyst-cautions }

- **Profil sahibi, işlemin hesabı olmak zorunda değildir.** Başka bir hesapla başlatılan bir program,
  bu kullanıcının oturumunda bir pencere gösterebilir ve bu kullanıcının önbelleğinde meta veri bırakabilir.
  `User` ve `UserSid`, işlemin güvenlik bağlamını değil, kovanın sahibini belirtir.
- **Program başına zaman yoktur.** `KeyLastWrittenUTC` anahtarın son yazılma zamanıdır: belirli bir programın
  çalıştığı zaman değil, programlarından herhangi birindeki en son değişikliktir. Zamanları Prefetch, BAM, UserAssist ve olay günlükleri
  verir.
- **Yalnızca kullanıcının elle başlattığı programlar değildir.** Kabuğun kendisinin yüklediği bir DLL veya program
  (`Open with`, bir kabuk uzantısı, bir yükleyicinin alt işlemi için) görünebilir; örneğin `explorerframe.dll` veya
  `shell32.dll`.
- **Ad dosyadan gelir.** `FriendlyAppName` ve `AppCompany`, dosya çalıştığında dosyanın sürüm kaynaklarının
  söylediği değerlerdir. Kötü amaçlı yazılımlar herhangi bir açıklama ve şirket iddia edebilir; yeniden adlandırılmış bir
  araç özgün açıklamasını korur ve bu onu ele verebilir (kolay anlaşılır adı bir uzaktan erişim aracı olan `svchost.exe`
  adlı bir dosya).
- **Yol başına bir girdi.** Aynı programın yeniden çalıştırılması girdi eklemez; başka bir yoldaki kopyası ise ekler.
  Bir programın girdisi, dosya silindikten sonra da kalabilir.
- **Tek bir yol için iki biçim.** Bir yolun her iki biçimde de kolay anlaşılır adı olduğunda (yükseltilmiş bir
  profil), her biri kendi kaydını alır; böylece hiçbir değer kaybolmaz.
- **Hasarlı değerler.** Dize olmayan, geçerli UTF-16 olmayan veya bütün olarak okunamayan bir değer
  baytlarını (`FriendlyAppNameRaw`, `AppCompanyRaw`) korur ve `Problems` alanında raporlanır.
  Geçersiz UTF-16, görüntülenen metinde yedek karakterler de üretebilir; özgün baytlar için ham
  sütunu kullanın.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Özet; Windows ve Program Files klasörleri dışındaki programları (kullanıcı klasörleri, diğer sürücüler,
ağ paylaşımları) kullanıcılarıyla birlikte ve dışarıda bırakılan kaynak dizelerinin sayısını listeler.
Başka adlara sahip Program Files klasörleri (başka dillerdeki Windows sürümlerinde) de listelenir.
Göreli yollar ve bir ortam değişkeni klasörünün dışına çıkan yollar listelenmeye devam eder. Sıradan
sürücü yolları, inceleyicinin dosya sistemine erişilmeden `.` ve `..` bileşenleri çözümlendikten sonra
kontrol edilir.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Ana bilgisayar, profil klasörünün adı, kullanıcının SID'i (bir `UsrClass.dat` kovanının kök anahtarı bunu adlandırır; aksi hâlde sistemin `ProfileList` verisi) ve kovan. |
| `HiveType` | `UsrClass.dat` veya `NTUSER.DAT`. |
| `Path` | Programın yolu: `.FriendlyAppName` veya `.AppCompany` olmadan değerin adı. |
| `FriendlyAppName` | `<path>.FriendlyAppName` veya `<path>` değerinden programın açıklaması. |
| `AppCompany` | `<path>.AppCompany` değerinden programın şirketi (Windows 8 ve sonrası). |
| `KeyPath`, `KeyLastWrittenUTC` | MuiCache anahtarı ve son yazılma zamanı: programlarından herhangi birindeki en son değişiklik. |
| `FriendlyAppNameRaw`, `AppCompanyRaw` | Kodu düzgün çözülen metin olmadığında değerin baytları (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı: günlükler programın değerlerinden birini yazdığında `replayed`. |
| `Problems` | Dize olmayan, geçerli UTF-16 olmayan veya okunamayan değerler. |

## İnceleme stratejisi { #investigation-strategy }

1. Kullanıcının yazabildiği yerlerdeki (`\Users\`, `Downloads`, `Temp`, `ProgramData`), diğer sürücülerdeki ve
   ağ paylaşımlarındaki programlara ve yönetim, uzaktan erişim ve arşivleme araçlarına bakın.
2. Dosya adını kolay anlaşılır adı ve şirketiyle karşılaştırın: bir uyuşmazlık yeniden adlandırılmış bir
   aracı veya kılık değiştirmiş bir dosyayı ortaya çıkarabilir.
3. Prefetch, BAM ve UserAssist (ne zaman çalıştığı), Amcache ve ShimCache (dosyanın
   karması ve varlığı) ve `$MFT` (dosyanın hâlâ orada olup olmadığı) ile ilişkilendirin.
4. Silinmiş MuiCache değerleri için `UsrClass.dat` üzerinde `parse-registry read --recover-deleted` kullanın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: VERSIONINFO kaynağı](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage: MuiCache deneyleri, GUI oturumları ve işlem hesabı sınırlamaları](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
