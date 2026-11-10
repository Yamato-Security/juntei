# NTFS İşlem Günlüğü (`$LogFile`) { #ntfs-transaction-log-logfile }

## Nedir ve neden önemlidir { #what-it-is-and-why-it-matters }

NTFS, Windows'un sistem birimi için genellikle kullandığı dosya sistemidir. Bir çökmeden sonra kurtarma yapabilmek için
dosya sistemi meta verilerinde yapmayı planladığı değişiklikleri `$LogFile` adlı gizli bir birim dosyasına
kaydeder. Her güncelleme **redo** baytları (yeni durum) ve **undo** baytları (bir değişikliği geri almak için gereken
önceki durum) içerebilir. Bu; Windows Olay Günlüklerinden, kayıt defteri `.LOG1`
dosyalarından ve [USN Günlüğü](usn-journal.md)'nden ayrıdır.

Bir inceleme yapan için önceki durum özellikle faydalıdır. Bir undo kaydı, yeniden adlandırma öncesindeki bir dosya adını,
silinmiş bir dizin girdisini veya bir meta veri güncellemesinden önceki zaman damgalarını koruyabilir.
Bazı kayıtlar ayrıca NTFS'in USN Günlüğü'ne yazdığı baytları da içerir; bunlara toplanan `$J` dosyasında artık
bulunmayan kayıtlar da dahildir. Ancak işlemlerin çoğu rutin işletim sistemi etkinliğiyle ilgilidir ve
bir saldırıyı tanımlamaz.

**Örnek:** kaldırılmış bir indeks girdisi `report.txt` adını içerir; daha sonra eklenen bir girdi ise
`report.txt.encrypted` adını içerir. Tam dosya referansının, üst dizinin, işlem zincirinin ve diğer
artefaktların eşleştirilmesi bir yeniden adlandırmayı destekleyebilir. Bu iki ad tek başına fidye yazılımını,
onu kullanan kişiyi veya şifrelemenin tam zamanını belirlemez.

## Konum ve toplama { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

Her NTFS biriminin kendi günlüğü vardır. Bunu, NTFS meta verilerini okuyan veya adli bir imajdan ya da anlık görüntüden
dışa aktaran bir araçla edinin; sıradan bir Windows dosya kopyalama işlemi buna erişemeyebilir. Aynı birimin
`$MFT` ve `$Extend\$UsnJrnl:$J` dosyalarını zaman olarak mümkün olduğunca yakın toplayın. Özgün
dosyaları ve karmaları koruyun. Daha sonra yapılan bir edinim farklı bir etkinlik aralığı içerebilir.

`$LogFile` **döngüsel bir günlüktür**: yeni kayıtlar eskilerinin üzerine yazar. Saklama süresi günlük boyutuna
ve iş yüküne bağlıdır ve yalnızca dakikalar veya saatler olabilir. Bir dosya adının veya işlemin bulunmaması,
bunun hiç gerçekleşmediğinin kanıtı değildir.

## Bir kaydın kanıtladıkları ve uyarılar { #what-a-record-proves-and-cautions }

- Bir güncelleme, NTFS'in kurtarma için günlüğe kaydettiği meta verileri kaydeder. Tek bir redo girdisi, daha üst
  düzey bir eylemin tamamının tamamlandığını veya belirli bir kişinin bunu başlattığını kanıtlamaz. İlgili
  kayıtları, işlemin tamamlanmasını, ortaya çıkan dosya sistemini ve bağımsız kanıtları ilişkilendirin.
- **Genel bir işlem zaman damgası yoktur.** Gömülü zaman damgaları dosya meta verilerini açıklar; bunlar
  bazen eskimiş, kopyalanmış veya saldırgan tarafından değiştirilmiş olabilir. `CreatedOn` değerini bir olay
  zaman çizelgesine yeniden adlandırma veya zaman damgası değişikliği zamanı olarak koymayın. USN kayıtları ayrı bir zaman damgası sağlayabilir.
- LSN'ler (log sequence number, günlük sıra numarası) kayıtları **tek bir birimin günlüğü içinde** sıralar. Bunlar saat
  zamanı değildir ve ana bilgisayarlar veya birimler arasında karşılaştırılmamalıdır. Döngüsel tamponun daha eski
  geçişlerine ait kayıtlar, daha yeni etkinlikle birlikte varlığını sürdürebilir.
- İşlem tablosu yuvaları yeniden kullanılır. Yalnızca `TransactionId` yerine `TransactionFirstLsn`, `TransactionId`, istemci
  kimliği ve kaynak ile gruplayın. Zincirler, saklanan verilerin dışına işaret edebilir.
- Dosya referansları bir girdi numarası **ve bir sıra numarası** içerir. Aksi takdirde bir MFT girdisinin yeniden
  kullanılması, ilgisiz dosyaların aynı dosya gibi görünmesine neden olabilir.
- `$MFT` tarafından sağlanan yollar, yeniden oluşturulmuş geçmiş bir yolu değil, **toplanan MFT durumunu** açıklar.
  Juntei sıra numaralarını eşleştirir ve bilinmeyen bileşenleri açıkça belirtir. Bir üst dizin işlemden sonra
  yeniden adlandırılmış olabilir ve sabit bağlantılar (hard link) birden fazla güncel ad sunabilir.
- `TargetInferred`, bir açık öznitelik eşlemesinin günlüğün farklı bir aralığından geldiği anlamına gelir.
  Bu eşlemeyi ek özenle ele alın. Çözümlenemeyen indeks girdilerinin kodu yalnızca düzenlerine bakılarak
  çözülmüş olabilir; adlarını ham baytlar ve diğer kaynaklarla doğrulayın.
- Yırtık (torn) bir sayfa atlanır; çevresindeki okunabilir kayıtlar kullanılabilir kalır. `Truncated`, devamı
  eksik olan bir kaydı işaretler. Çıktıyı eksiksiz kabul etmeden önce ayrıştırıcı tanılamalarını inceleyin.
  Sıfır uzunluklu bir girdi bir hatadır; hiç yazılmamış, tamamı `FF` olan bir günlükte satır bulunmaması olağandır.
- Ayrıştırıcı, geçmişteki bir MFT'yi yeniden oluşturmak için işlemleri **yeniden oynatmaz**. Rastgele yerleşik
  olmayan (non-resident) dosya içeriklerini kurtarmaz veya toplanan hiçbir veriyi yürütmez. DAX CRC korumalı
  sayfalar desteklenmez. Gerçek kanıtlarla yapılan doğrulama esas olarak 4 KiB'lık günlük sayfalarını kapsar.

## Juntei ile ayrıştırma { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

Tüm seçenekler için [komut başvurusuna](../commands/filesystem.md#parse-ntfs-logfile) bakın. Dizin
taraması; büyük/küçük harf ayrımı yapmadan `$LogFile`, `LogFile`, bunların `.bin` biçimlerini ve `C_$LogFile` veya
`C_$LogFile.bin` gibi adları kabul eder. `-f` ile verilen tek bir dosya herhangi bir ada sahip olabilir.

Otomatik MFT eşleştirmesi, bir dışa aktarımın birim önekine uyar: `C_$LogFile`, `C_$MFT` dosyasını kullanır;
`D_$LogFile.bin` önce `D_$MFT.bin`, ardından `D_$MFT` dosyasını dener. Önekli günlükler asla yedek olarak düz bir `$MFT`
kullanmaz. Öneksiz `LogFile` ayrıca `MFT` veya `$MFT` dosyasını da kabul eder. Dışa aktarımınız başka bir adlandırma
düzeni kullanıyorsa eşleşen bir MFT'yi açıkça seçmek için `-f` ile birlikte `-m` kullanın.

Çıktı; güncelleme, denetim noktası (checkpoint) ve bilinmeyen istemci kayıtları dahil olmak üzere LSN başına bir kayıttır. JSON,
okunaklı biçimlendirilmiş nesnelerden oluşan bir akıştır; JSONL her satırda bir nesne içerir. CSV aynı alan adlarına sahiptir;
diziler ve yapılandırılmış alanlar bir CSV hücresi içinde sıkıştırılmış JSON olarak yer alır. Bilinmeyen skaler değerler
CSV'de boş, JSON'da `null` olur. Boş diziler hiçbir öğenin kodunun çözülmediği anlamına gelir; hiçbir öğe bulunmadığının kanıtı değildir.

Bir seferde bir günlüğün tamamı ve ayrıştırılmış kayıtları bellekte tutulur. MFT yollarının çözümlenmesi
bellek kullanımını ve iş yükünü artırır. `--raw-data` çıktı boyutunu büyük ölçüde artırabilir. `auto-parse` içinde, belleği kısıtlı bir
makinede eşzamanlı ayrıştırıcı işlemlerini sınırlamak için `-T` kullanın.

Yanındaki bir MFT okunamazsa Juntei uyarı verir ve günlüğü yollar olmadan dışa aktarır; bu durumda `MftSource`
null kalır. Bu zenginleştirme hatası okunabilir birincil kanıtları atmaz.
Eksik veya belirsiz MFT eşleşmeleri de bir uyarı ve özette bir `Logs without a matching MFT`
sayısı üretir; bunlar ayrıştırma hatası olarak sayılmaz.

## Çıktı alanları { #output-fields }

### Kayıt kimliği, bütünlük ve köken { #record-identity-integrity-and-provenance }

| Alan | Anlamı ve yorumlanması |
|---|---|
| `SourceFile`, `FileOffset` | Girdi yolu ve bu kaydın başlığının okunduğu bayt ofseti; baytları doğrulamak için kullanın. |
| `LSN` | Kaydın günlük sıra numarası. Büyük değerler tamsayıdır, zaman damgası değildir. |
| `PreviousLSN`, `UndoNextLSN` | Önceki kayda ve geri alma kayıtlarına bağlantılar. Sıfır, bağlantı olmadığı anlamına gelir; sıfır olmayan bağlantılar saklanan verilerin dışına işaret edebilir. |
| `TransactionID`, `TransactionFirstLSN` | İşlem tablosu yuvası ve bu işlemle bağlantılı en eski LSN; yuva tek başına benzersiz değildir. |
| `RecordType` | `ClientRecord` (güncelleme), `ClientRestart` (denetim noktası) veya `Other`. |
| `ClientIndex`, `ClientSequence` | Günlük istemcisinin kimliği; normalde NTFS'tir. Bunlar bir kullanıcı hesabı veya işlem kimliği değildir. |
| `Flags`, `SpansPages` | Ham kayıt bayrakları ve istemci verisinin başka bir sayfada devam edip etmediği. Sayfalara yayılması normaldir. |
| `Truncated` | Ayrıştırıcı bu LSN için eksik bir devam bulduysa True. Dosyanın tamamındaki tüm hasarı özetlemez. |
| `ClusterSize`, `ClusterSizeSource` | Hedef ofsetleri için kullanılan küme başına bayt sayısı; kaynak `Caller`, `Checkpoint` veya `Inferred` olur. Biliniyorsa gerçek değeri `--cluster-size` ile sağlayın. |
| `LogVersion` | Okunabilir en son yeniden başlatma (restart) sayfasındaki sürüm; genellikle `1.1` veya `2.0`. Okunabilir bir yeniden başlatma sayfası kalmamışsa null. |
| `MFT-Source` | Mevcutsa güncel yol zenginleştirmesi için kullanılan MFT. Aynı birime ait olmalıdır. |

`auto-parse` JSON/JSONL çıktısında, diğer ayrıştırıcılarda olduğu gibi `Comp` ve `Artifact` eklenir. Görevin
`.txt` günlüğü tanılamaları içerir; manifestler görevin başarılı olup olmadığını, başarısız olup olmadığını veya atlanıp atlanmadığını belirtir.
Kurtarılabilir hasar, günlüğünde sıfırdan farklı bir `Parse problems` sayısı bulunan başarılı bir görev üretebilir.

### İşlemler ve hedef { #operations-and-target }

| Alan | Anlamı ve yorumlanması |
|---|---|
| `RedoOperation`, `UndoOperation` | NTFS işlem adları veya bilinmeyen bir kod için onaltılık değer. Redo amaçlanan güncellemeyi, undo ise bunun tersini açıklar. |
| `TargetEntryNumber`, `TargetSequenceNumber` | Bir dosya kaydı üzerindeki işlem için: etkilenen MFT kayıt numarası; sıra numarası çıkarımı yapılmaz. Diğer durumlarda: çözümlenen açık özniteliğin sahibi olan dosya referansı. Satır içi bir dizin indeksi güncellemesi, girdisinin tanımladığı alt öğeyi değil, dizini hedefler. |
| `TargetAttributeType`, `TargetAttributeName` | İşlemin kullandığı açık özniteliğin sayısal türü ve adı. Dosya kaydı işlemleri için bu normalde `$MFT` dosyasının `$DATA` özniteliğidir. Yaygın türler: 16 (`$STANDARD_INFORMATION`), 48 (`$FILE_NAME`), 128 (`$DATA`), 144 (`$INDEX_ROOT`), 160 (`$INDEX_ALLOCATION`), 176 (`$BITMAP`). |
| `TargetPath` | Hedefin tam referansı biliniyorsa güncel MFT yolu veya bilinmeyen bileşenler. Geçmişteki kuşağı (generation) bilinmeyen dosya kaydı işlemleri için boştur; yalnızca bir kayıt numarası, o kaydı şu anda kullanan dosyaya çözümlenmemelidir. |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | İşlemin aracılığıyla yazdığı açık özniteliğin sahibi olan dosya ve güncel MFT yolu. Dosya kaydı işlemleri için bunlar `$MFT` bağlamını etkilenen kayıttan ayrı olarak korur. |
| `TargetAttribute` | Günlüğün açık öznitelik tablosundaki ofset; bu, özniteliğin türünden veya MFT tanımlayıcısından farklıdır. |
| `TargetInferred` | Hedef eşlemesi günlüğün başka bir aralığından geliyorsa True. Bu bayrak false olsa bile null bir hedef çözümlenmemiş olarak kalır. |
| `TargetVCN`, `Lcns` | Hedefteki sanal küme ve fiziksel küme numaraları; zamanlama için değil, düşük düzeyli doğrulama için faydalıdır. |
| `RecordOffset`, `AttributeOffset` | Özniteliğin/girdinin ve bunun içindeki güncellemenin ofsetleri. |
| `ClusterBlockOffset`, `TargetBlockSize` | 512 baytlık birimler cinsinden ofset ve hedef boyutu. |
| `OffsetInTarget` | Etkilenen dosya kaydında veya yerleşik olmayan akışta hesaplanan bayt ofseti. Küme ve MFT kayıt boyutlarına bağlıdır. |
| `RedoLength`, `UndoLength` | Bildirilen veri uzunlukları. Sıfırlama işlemi, o kadar yük baytı taşımadan bir uzunluk bildirebilir. |
| `RedoBytesAvailable`, `UndoBytesAvailable` | Kod çözme için gerçekten kullanılabilir olan yük baytları. Bildirilen uzunluklar ve işlem anlamıyla karşılaştırın. |

### Kullanışlı dosya meta verisi sütunları { #convenient-file-metadata-columns }

Bu sütunlar redo tarafından tercih edilen DOS olmayan bir dosya adını gösterir; yoksa undo tarafına geri dönülür.
Hiçbir taraf dosya adı içermiyorsa, zaman damgası güncelleme alanları önce redo zaman damgalarını, ardından undo zaman damgalarını tercih eder.
Dizin girdisi güncellemeleri de zamanlarını, boyutlarını ve bayraklarını doğrudan sunar ve redo'yu undo'ya
tercih eder. Kodu çözülen `$DATA` öznitelikleri, dosya adı bulunmasa bile boyutlarını sunar. Kodu çözülen bir
dosya kaydının adsız `$DATA` boyutu, eskimiş olabilecek dosya adı boyutuna göre önceliklidir.
Bunlar göz atmayı kolaylaştıran yardımcılardır; eksiksiz bir önce/sonra karşılaştırması için ayrı taraf alanlarını kullanın.

| Alan | Anlamı ve yorumlanması |
|---|---|
| `MetadataSide` | Kullanışlı dosya adı/zaman damgası sütunlarını sağlayan tarafı belirten `Redo` veya `Undo`. |
| `EntryNumber`, `SequenceNumber` | Kodu çözülen dosya adı/dosya kaydı/öznitelik meta verilerinin veya kısmi bir `$STANDARD_INFORMATION` zaman damgası güncellemesinin tanımladığı dosya; biliniyorsa sıra numarasıyla birlikte. Alt öğeyi tanımlamayan bir dizin girdisi güncellemesi ile ham boyut/run-list güncellemeleri için boştur. Etkilenen kayıt için `TargetEntryNumber` kullanın. |
| `Filename` | Tercih edilen, kodu çözülmüş ad; DOS takma adları ve sabit bağlantılar dahil tüm adlar taraf dizilerinde kalır. |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | Üst dizin referansı ve güncel MFT yolu; geçmiş yolun yeniden oluşturulması değildir. |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | 100 ns hassasiyetle UTC meta veri zamanları. Bunlar işlem zaman damgaları değildir. Eksik/geçersiz zamanlar boş/null kalır. |
| `FileSize`, `FileAttributes` | Seçilen meta verilerden boyut ve sayısal Windows bayrakları; dizin girdisi değerleri eskimiş olabilir. `$DATA` öznitelikleri ve kodu çözülen dosya kayıtları daha doğrudan bir boyut sağlayabilir. |
| `AttrType` | Kodu çözülen bir öznitelik kaydının sayısal türü; kodu çözülen bir dizin girdisi meta veri güncellemesi için 48. İşlemin hedeflediği açık öznitelikten ayrıdır. |

### Eksiksiz redo/undo meta verileri ve yükler { #complete-redoundo-metadata-and-payloads }

| Alan | Anlamı ve yorumlanması |
|---|---|
| `RedoFileNames`, `UndoFileNames` | O taraftaki kodu çözülen her `$FILE_NAME` özniteliğinin dizileri; eklenen/yeni adları kaldırılan/eski adlardan ayrı olarak korur. |
| `RedoTimes`, `UndoTimes` | Her tarafın yazdığı `$STANDARD_INFORMATION` zaman damgaları; kısmi güncellemeler, değişmeyen zaman damgalarını tahmin etmek yerine bulunmuyor olarak bırakır. |
| `RedoDecoded`, `UndoDecoded` | `Kind` ile tanımlanan yapılandırılmış dosya kaydı, öznitelik, indeks girdisi veya dizin girdisi bilgileri. Null, desteklenmeyen bir işlem veya yetersiz bayt anlamına gelebilir. |
| `JournalWriteOffset`, `JournalWriteLength` | `$DATA:$J` akışına yazan redo verisinin ofseti ve bayt sayısı. Bu, aksi hâlde kaybolmuş olacak USN kayıtlarının olası bir kaynağını belirtir; yeniden oluşturulmuş bir günlüğü dışa aktarmaz. |
| `Checkpoint` | Denetim noktası sürümü, başlangıç LSN'i, küme boyutu ve açık öznitelik, öznitelik adı, kirli sayfa (dirty page) ve işlem tablosu dökümlerinin LSN/uzunluk değerleri. |
| `OtherRecordType` | Tanınmayan veya kodu çözülemeyen bir gövde için ham istemci kayıt türü. |
| `RedoData`, `UndoData`, `OtherData` | Yalnızca `--raw-data` ile onaltılık yükler; aksi hâlde null/boş. Desteklenmeyen veya hasarlı yapıların doğrulanması için bunları saklayın. |

Her dosya adı nesnesi dosya ve üst dizin referanslarını (`Raw`, `EntryNumber`,
`SequenceNumber`), `Name`, `NameEscaped`, `Namespace`, güncel `ParentPath`, dört zaman damgası,
`Size`, `AllocatedSize`, `Attr` ve `ReparseValue` değerlerini içerir. `NameEscaped`, kaçış dizileriyle
gösterilen geçersiz UTF-16'yı belirtir. Bir reparse değeri bir reparse etiketi veya genişletilmiş öznitelik boyutu olabilir.

Zaman damgası nesneleri hem `Utc` değerini hem de tamsayı `Filetime` değerini korur; bir gözcü (sentinel) veya geçersiz değer,
özgün tamsayısını korurken null bir UTC gösterimine sahip olabilir. `RedoTimes` ve `UndoTimes`
`Created`, `Modified`, `Changed` ve `Accessed` içerir; eksik üyelerin değerleri null olur.

Kodu çözülen `FileRecord` nesneleri numarayı, sıra numarasını, sabit bağlantı sayısını, bayrakları, temel referansı,
yalnızca başlığın kalıp kalmadığını ve mevcut öznitelikleri içerir. Kodu çözülen her öznitelik
türü, adı, yerleşik olma durumunu, bayrakları, kimliği, boyutu ve ayrılmış boyutu korur. `$STANDARD_INFORMATION`
dört zamanı, öznitelikleri, sahip/güvenlik kimliklerini, kotayı ve USN'i ekler; bu kimlikler ayrıca çözümlenmelidir ve
dosyayı değiştiren kullanıcının kanıtı değildir. İndeks girdileri bayrakları, anahtar uzunluğunu ve isteğe bağlı
bir dosya adını içerir. Dizin girdisi güncellemeleri, adın kendisi günlüğe kaydedilmemiş olsa bile dört zamanı, boyutları,
öznitelikleri ve reparse değerini korur.

## İnceleme iş akışı { #investigation-workflow }

1. Eksik yeniden başlatma sayfaları, yırtık sayfalar ve kesilmiş kayıtlar için tanılamaları inceleyin.
2. Şüpheli bir dosya adı veya uzantı için hem `RedoFileNames` hem de `UndoFileNames` içinde arama yapın.
3. Girdi **ve sıra** numaralarını eşleştirin, ardından işlemin LSN zincirini ve işlemlerini inceleyin.
4. Değişiklikler için redo ve undo zaman damgalarını karşılaştırın. Rutin kopyalama, arşivden çıkarma ve geri yükleme
   de zaman damgalarını değiştirebilir; bir fark bir ipucudur, otomatik olarak zaman damgası oynaması (timestomping) hükmü değildir.
5. [USN Günlüğü](usn-journal.md), [$MFT](mft.md) ve [$I30](i30.md) ile ilişkilendirin. Yürütmeyi veya atfı
   belirlemek için olay günlüklerini, [Prefetch](prefetch.md) ve kullanıcı etkinliği artefaktlarını kullanın.

Örneğin, herhangi bir tarafında şifrelenmiş bir dosya adı taşıyan tüm kayıtları bulun:

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

Bu bir dosya adı aramasıdır, eksiksiz bir fidye yazılımı tespit aracı değildir. Günlük yüklerini incelemek için
`--raw-data` ile dışa aktarın ve `JournalWriteOffset` değeri null olmayan kayıtları seçin; `RedoData`
onaltılık verisi o ofsete yazılan baytları temsil eder. Bu gömülü USN kayıtlarının ayrıştırılması ve yinelenenlerin
ayıklanması ayrı bir adımdır. Juntei bunları henüz `parse-ntfs usn-journal` çıktısıyla birleştirmez.

## Daha fazla ayrıntı { #further-details }

- [ys-ntfs-parser'ın logfile crate'i](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile): desteklenen yapılar, doğrulama derlemi ve bilinen sınırlamalar.
- [Sootmark ntfslog](https://github.com/Sootmark/ntfslog): upstream Rust uygulaması.
- [libfsntfs NTFS biçim belgeleri](https://github.com/libyal/libfsntfs/tree/main/documentation): günlük sayfaları, düzeltmeler (fixup) ve meta veri yapıları.
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs): bağımsız NTFS ayrıştırma araştırması ve uygulaması.
- [Joakim Schicht'in LogFileParser aracı](https://github.com/jschicht/LogFileParser): ayrıntılı işlem günlüğü analizi.
- [Stroz Friedberg'in ntfs-linker aracı](https://github.com/strozfriedberg/ntfs-linker): NTFS kaynaklarını dosya geçmişi için ilişkilendirme.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum): gerçek Windows NTFS örnekleri.
- [Velocidex go-ntfs](https://github.com/Velocidex/go-ntfs): NTFS uygulaması ve herkese açık test birimi.
