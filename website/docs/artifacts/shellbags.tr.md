# ShellBags { #shellbags }

## ShellBags nedir { #what-shellbags-are }

Explorer, kullanıcının açtığı her klasörün nasıl görüntülendiğini hatırlar: görünümünü, simge boyutunu,
sıralama düzenini ve pencere konumunu. Bir klasörün ayarlarını yeniden bulabilmek için klasörün kimliğini
kullanıcının kayıt defterinde bir kabuk öğeleri (shell item) ağacı olarak tutar; bunlar bir bağlantının
hedef kimliği listesinin tuttuğu öğelerle aynıdır: bir sürücü, ardından açılan klasöre kadar her klasör.
Bu ağaçlar ShellBags'tir.

Explorer'da ve açma ve kaydetme iletişim kutularında açılan klasörleri kapsarlar; çıkarılabilir
sürücülerdeki, ağ paylaşımlarındaki, taşınabilir aygıtlardaki (telefonlar, kameralar), ZIP arşivlerindeki
ve FTP sunucularındaki klasörler ile Denetim Masası sayfaları da buna dahildir. Klasör, sürücü veya ağ
paylaşımı ortadan kalktıktan sonra da kalırlar.

<span class="evidence-strength">En iyi sonuç:</span> bu kullanıcının Explorer'ı bu klasörü (veya
altındaki bir klasörü) göstermiştir. Bir ShellBag klasörde ne yapıldığını göstermez ve her ziyareti
tarihlendirmez: aşağıdaki zamanlara bakın.

## Konum { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

Bir `BagMRU` anahtarı, her biri bir kabuk öğesi olan numaralandırılmış değerler ve bunların sırasını (en son
açılan ilk sırada) veren bir `MRUListEx` değeri tutar. Her değerin, altındaki öğeler için aynı adlı bir alt
anahtarı vardır; böylece bir yol, `BagMRU` anahtarından bir değere kadar inen öğelerden oluşur. Bir alt
anahtarın `NodeSlot` değeri, klasörün görünüm ayarlarının bulunduğu `Bags\<N>` anahtarının numarasıdır.

Her profil için `UsrClass.dat` ve `NTUSER.DAT` dosyalarını işlem günlükleriyle (`.LOG1`, `.LOG2`) birlikte
ve bir `NTUSER.DAT` kullanıcısının SID'sini veren sistemin `SOFTWARE` kovanını toplayın.

## Hangi zaman neyi söyler { #which-times-say-what }

- **Kabuk öğesinin zamanları** (`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`), Explorer öğeyi
  yazdığı andaki klasörün kendi dosya sistemi zamanlarıdır (FAT hassasiyeti: 2 saniye). Ziyareti değil
  klasörü tarihlendirirler ve sonraki ziyaretlerde güncellenmezler.
  FAT saatlerinin saklanmış bir UTC ofseti yoktur. Eski tarz `UTC` başlıkları ve gösterilen `Z` soneki
  görüntüleme kurallarıdır; kaynak saat diliminin kanıtı değildir. `auto-parse --timezone` bu alanları
  korur; bunları kaynak sistem bağlamıyla yorumlayın.
- **Bir anahtarın son yazılma zamanı**, `MRUListEx` değeri yeniden yazıldığında, yani öğelerinden biri
  açılıp öne taşındığında değişir. Dolayısıyla bir öğenin değerinin bulunduğu anahtarın zamanı
  (`ParentKeyLastWrittenUTC`), öğenin en son açıldığı zamandır; **ancak yalnızca öğe o anahtardaki en son
  öğe olduğunda** (`IsMostRecentInParent` true, `MruPosition` 0). Diğer öğeler için bu, bir kardeş öğenin
  açıldığı daha sonraki bir zamandır.
- Bir öğenin kendi anahtarı (`KeyLastWrittenUTC`), en son alt öğesinin açılışını aynı şekilde tarihlendirir.

## Analist için uyarılar { #analyst-cautions }

- **Yokluk pek bir şey kanıtlamaz.** Explorer, bir klasörün görünümü kaydedildiğinde ShellBags yazar; yalnızca
  kısa süre görülen veya başka bir yolla (komut satırı, betik, program) açılan bir klasörün hiç kaydı
  olmayabilir. Kullanıcı anahtarları silebilir.
- **Bir NTFS referansı**, öğe yazıldığı andaki klasöre aittir (`MftEntry`, `MftSequence`). Aynı klasörün hâlâ
  var olup olmadığını veya girdiyi başka bir dosyanın yeniden kullanıp kullanmadığını görmek için `$MFT`
  içinde arayın.
- **Çıkarılabilir sürücüler** sürücü harfleriyle adlandırılır. Harf, sürücünün o sırada sahip olduğu harftir;
  aygıt için USB ve bağlanmış aygıt geçmişiyle eşleştirin.
- **Adlar öğeden gelir.** Daha sonra yeniden adlandırılan bir klasör burada eski adını korur. Kök ve bilinen
  klasörler, adli bilişim araçlarının yaygın olarak adlandırdığı şekilde adlandırılır (`This PC`,
  `Quick Access`, `Downloads`); `Guid` bunların tanımlayıcısını verir.
- **Arama sonuçları, kitaplıklar ve Denetim Masası sayfaları** diskteki klasörler değil, görünümlerdir
  (`property_view`, `control_panel_item`, `control_panel_category`).
- **Hiçbir ayrıştırıcının tanımadığı bir öğe**, baytlarını `ItemType` `unknown` ve sınıf baytı `ItemClass`
  içinde olacak şekilde `DataRaw` içinde tutar. Hasarlı veya olağandışı öğeler `Problems` içinde raporlanır
  ve baytları da korunur.
- **Kirli bir kovan**, günlükleri olmadan en son öğelerinden yoksun olabilir. Juntei, `--no-logs`
  verilmedikçe kovanın yanındaki günlükleri yeniden oynatır; `TransactionLogStatus` bir kaydın onlardan gelip
  gelmediğini belirtir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

Kabuk öğelerinin kodu, `parse-file lnk` komutunun kabuk öğesi ayrıştırıcısı olan ys-lnk-parser ile çözülür.
Özet, en son açılan klasörleri (en yeniden başlayarak), üst anahtarlarının son yazılma zamanına göre,
kullanıcılarıyla birlikte listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | Ana bilgisayar, profil klasörünün adı, kullanıcının SID'si (bir `UsrClass.dat` dosyasının kök anahtarı onu adlandırır; `NTUSER.DAT` için sistemin `ProfileList` kaydı), kovan ve türü. |
| `BagKey` | Kovanın kökü altındaki öğenin anahtarı (`…\BagMRU\4\3\0`): değeri, bir üstündeki anahtardadır. |
| `MRU-Position`, `IsMostRecentInParent` | Üst anahtarının `MRUListEx` değerindeki yeri (0: en son açılan); liste onu içermiyorsa boştur. |
| `NodeSlot` | `Bags\<N>` anahtarının numarası (klasörün görünüm ayarları). |
| `Path` | Ağacının tepesinden ona kadar olan öğeler, `\` ile birleştirilmiş olarak. `[unknown]`, değeri eksik olan bir öğeyi temsil eder. |
| `ItemType`, `ItemClass` | Öğenin ne olduğu (`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`) ve sınıf baytı. |
| `ItemName`, `GUID` | Adı (bir klasörün uzun adı) ve bir kök klasörün, Denetim Masası öğesinin veya temsilci klasörün sınıf tanımlayıcısı. |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **Kabuk öğesinden:** öğe yazıldığı andaki klasörün zamanları (FAT, 2 saniye hassasiyet). |
| `MFT-Entry`, `MFT-Sequence` | **Kabuk öğesinden:** klasörün NTFS dosya referansı. |
| `ExtensionBlocks` | Öğenin kaç uzantı bloğu olduğu. |
| `KeyLastWrittenUTC` | **Kayıt defterinden:** öğenin kendi anahtarının son yazılma zamanı: en son alt öğesinin açıldığı zaman. |
| `ParentKeyLastWrittenUTC` | **Kayıt defterinden:** değerinin bulunduğu anahtarın son yazılma zamanı: `IsMostRecentInParent` true ise, öğenin en son açıldığı zaman. |
| `DataRaw` | Tam olarak anlaşılamayan veya sorunları olan bir öğe için değerin baytları (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Anahtarının `MRUListEx` değerinin adlandırdığı ancak anahtarda bulunmayan bir değer, listenin adlandırmadığı bir değer, değeri olmayan bir alt anahtar, ikili olmayan bir değer, yanlış türde bir `NodeSlot`, hasarlı bir kabuk öğesi listesi. |

## İnceleme stratejisi { #investigation-strategy }

1. Çıkarılabilir sürücüleri (Removable Drives `Guid` değerine sahip `volume`), ağ konumlarını, FTP
   klasörlerini ve taşınabilir aygıtları ve bunların altındaki klasörleri listeleyin.
2. İlgi çekici klasörleri arayın: kullanıcının yazabildiği hazırlık konumları (`Temp`, `ProgramData`,
   `Public`), araç ve arşiv klasörleri, diğer kullanıcıların profilleri ve artık var olmayan klasörler
   (`$MFT` içindeki NTFS referansını kontrol edin).
3. Ziyaretleri yalnızca `IsMostRecentInParent` değeri true olan öğelerin `ParentKeyLastWrittenUTC` değeriyle
   tarihlendirin; geri kalanları dosya açılışlarını tarihlendiren LNK dosyaları, Jump Lists ve RecentDocs ile
   ilişkilendirin.
4. Silinmiş `BagMRU` anahtarları ve değerleri için `parse-registry read --recover-deleted` komutunu kullanın.

## Daha fazla ayrıntı { #further-details }

- [libyal libfwsi: Windows Shell Item format](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

`ProfileList` üzerinden SID araması, benzersiz ve sözdizimsel olarak geçerli tek bir SID gerektirir.
Belirsiz profil klasörü eşlemeleri ve `S-1-...bak` gibi yedek anahtarlar `UserSid` değerini bilinmeyen
olarak bırakır.
