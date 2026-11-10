# Birleşik zaman çizelgesi { #super-timeline }

`super-timeline`, ayrıştırılmış yapıtları ortak bir şemaya sahip olaylara dönüştürür. Çıktı **varsayılan olarak sıralanmamıştır**: çalışanın tamamlama sırasından bağımsız olarak keşfedilen giriş dosyası sırasını ve her ayrıştırıcının projeksiyon sırasını takip eder. Kronolojik sıralama ve kopya birleştirme için `--sort`'i ekleyin. Bir Prefetch kaydı birden fazla yürütme zamanı içerebilir; bir MFT kaydı sekiz veya daha fazla dosya zaman damgası içerebilir. Her biri ayrı ayrı etiketlenmiş bir olay haline gelir. Bu, her zaman damgasının gerçekte ne anlama geldiğini kaybetmeden, farklı kaynaklardan elde edilen kanıtları karşılaştırmayı kolaylaştırır.

Özgün kanıt dizini yerine [`auto-parse`](auto-parse.md) sonuçlarını kullanın:

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

Ayrıştırılmış dosyaları, manifestleri ve özgün kanıtları saklayın. Bu analiz görünümü onların yerini almaz ve yerel Plaso deposu oluşturmaz.

## Kullanım { #usage }

```bash
# Native CSV (default)
juntei super-timeline -d parsed/ -o SuperTimeline.csv

# Chronological CSV with duplicate consolidation
juntei super-timeline -d parsed/ -o SortedTimeline.csv --sort

# Native JSON Lines
juntei super-timeline -d parsed/ -o SuperTimeline.jsonl -t jsonl

# Combine native records and an existing Hayabusa JSONL timeline
juntei super-timeline -f NativeRecords.jsonl --hayabusa Hayabusa.jsonl \
  -o SuperTimeline.jsonl -t jsonl

# Timesketch import profile; also available with CSV
juntei super-timeline -d parsed/ -o Timesketch.jsonl -t jsonl --profile timesketch

# One standalone CSV needs an artifact identifier and, if unavailable, a computer name
juntei super-timeline -f MFT.csv --input-artifact mft --computer HOST01 \
  -o Host01Timeline.csv

# Reduce the time range, memory used by sorting, or number of workers
juntei super-timeline -d parsed/ -o IncidentTimeline.jsonl -t jsonl \
  --timeline-start "2026-10-20T06:00:00Z" --timeline-end "2026-10-20T14:00:00Z" \
  --sort --memory-limit 64 -T 4 --temp-dir scratch/
```

`--hayabusa FILE` tek başına veya tekrarlanarak, Hayabusa çalıştırmadan mevcut olay/uyarı zamanlarını ve kural bağlamını aktarır. `Details` değeri metin olan bilgi olayları desteklenir. `--artifact mft,prefetch` artefaktları sınırlar. Zaman sınırları dahildir; açık UTC ofseti veya `Z` gerekir.

### Girdi seçimi { #choosing-an-input }

| Girdi | Tanımlama |
|---|---|
| `-d DIR` | `manifest.json` başarılı birincil görevleri, artefakt kimliklerini, bilgisayarları ve çıktı dosyalarını verir. Çift aktarımı önlemek için ikincil `auto-parse -j` görevleri dışlanır. |
| `-f FILE` | Yerel JSON/JSONL, gömülü `Artifact` ve `Comp` alanlarını kullanır. Etiketsiz dışa aktarımlar `--input-artifact` gerektirir; bağımsız CSV için daima zorunludur. |
| `--computer NAME` | Kaydın bilgisayar adı yoksa yedek ad; bilinmeyen bilgisayar açıkça `Unknown` olarak gösterilir. |
| `--hayabusa FILE` | Hayabusa zamanları, kural metaverileri, bilgisayar ve varsa `EvtxFile` içeren JSONL. |

Güncel manifestler sonuç dizininin taşınmasını destekler; eski mutlak yollar yeni dizinde çözülür. Sembolik bağlantı hedefleri dahil girdiler bu dizinde kalmalıdır. Eski Amcache, SRUM ve Jump Lists dizinleri desteklenir; kategori dizinleri açık `OutputFiles` gerektirir.

Çıktıya göreli yollar ve düzleştirilmiş çok tablolu çıktılar desteklenir. Otomatik ayrıştırmanın planladığı tüm artefaktlar; tarayıcı, WebCache, Outlook ve UAL dahil tanınır.

Manifestteki VHDX/E01 görüntülerinde kaynak referansı görüntüyü ve iç yolu içerir; görüntü adı bağlamda korunur. Sonuçları okumak için geçici hazırlama dizini gerekmez.

Kayıttan ham bir kaynak veya net bir bildirim komutu alınır. `-d` ile ham kaynak referansları kanıt dizinine göredir ve `Details.ParsedFile` ve özet `InputFiles` ayrıştırılan dizine göredir. Özet çıktı/rapor adları, zaman çizelgesinin çıktı dizinine göredir. SRUM CSV tabloları, yalnızca ana bilgisayarın benzersiz şekilde tanımlanabilen toplanmış bir SRUDB.dat dosyasına sahip olması durumunda bir veritabanına atanır. Aksi halde `SourceFile` ayrıştırılmış dosyayı işaret eder ve `Details.RawSourceAvailable`, `false`'tir.

Bağımsız bir girişe dosya adıyla başvurulur. Aynı dosya adına sahip girişler farklı `./Inputs/<ID>/<filename>` referanslarını kullanır. Bilinen kanıt köklerinin dışındaki mutlak ham kaynak yolları, `./Sources/<ID>/<filename>` tanımlayıcılarını kullanır. Bu şeffaf olmayan referanslar, analistin mutlak dizin adlarını dışa aktarmadan kaynak ayrımlarını korur; Orijinal kaynak için `ParsedFile` ve `InputRecord`'i kullanarak yerel kayda bakın. Kanıta kaydedilen Windows yürütülebilir yolları gibi yollar değişmeden kalır.

## Ortak olay alanları { #common-event-fields }

| Alan | Anlam |
|---|---|
| `Timestamp` | Tutarlı sıralama için dokuz ondalık basamaklı UTC RFC 3339. Sıfır eklemek hassasiyeti artırmaz. |
| `Comp` | Girdi kaydının bilgisayarı. |
| `Artifact` | `mft`, `prefetch`, `srum` veya `hayabusa` gibi kararlı artefakt kimliği. |
| `TimestampType` | `MFT SI Data modified` gibi zaman damgasının anlamı. |
| `User` | Varsa kayıtlı hesap veya SID; boş olabilir. |
| `Path` | İlgili dosya, kayıt defteri anahtarı veya artefakta özgü nesne. |
| `Message` | Zaman çizelgesi görüntüleyicisi için kısa açıklama. |
| `SourceFile` | Kanıta dayalı ham kaynak referansı veya kökü bilinmediğinde göreceli kaynak tanımlayıcı; aksi takdirde yukarıda açıklanan ayrıştırılmış dosya geri dönüşü. |
| `SourceField` | Zamanı sağlayan tam alan veya iç içe konum. |
| `RecordID` | Artefakta özgü kayıt kimliği; mutlaka EVTX kayıt ID değeri değildir. |
| `Details` | Özgün zaman, hassasiyet, zaman temeli/rolü, dosya, kayıt/konum ve kurtarma/kural metaverilerinin yapılandırılmış bağlamı. |

CSV içinde `Details`, tırnaklı JSON hücresidir; JSONL içinde nesnedir. Timesketch profili `Timestamp`, `TimestampType`, `Message` alanlarını `datetime`, `timestamp_desc`, `message` olarak adlandırır; diğer kaynak alanlarını korur. Bunlar [Timesketch'in zorunlu üç alanıdır](https://timesketch.org/guides/user/import-from-json-csv/).

`Details` tüm ikili veriyi ve yerel alanları değil, seçilen bağlamı içerir. Uzun metin ve yapılar sınırlandırılıp `ProjectionOmitted` ile işaretlenir. Tam kayıt için `Details.ParsedFile`, `InputRecord`, `Location` kullanılır. CSV numarası başlıktan sonraki CSV kayıtlarını sayar, hücre içindeki fiziksel satırları değil. JSONL için satır; JSON için üst belge numarası ve iç konumdur.

## Zamanların anlamı { #what-the-times-mean }

Zaman damgası kayıtlı nesneye ilişkin kanıttır; tek başına yürütme veya saldırı kanıtı değildir. Adaptörler şu ayrımları korur:

| Artefaktlar | Zaman çizelgesindeki yorum |
|---|---|
| MFT | Ayrı SI ve FN değiştirme, erişim, meta veri değiştirme ve oluşturma süreleri; FN adlı öznitelikler farklı kalır. `--sort` ile dosya adı satırlarındaki tekrarlanan SI alanları birleştirilir. ADS satırlarının paylaşılan temel dosya süreleri her iki modda da hariç tutulur. Kurtarılan FILE kaydı bolluk adları, kurtarılan FN olaylarını ayrı tutar. |
| I30 | Kaydedilmiş FN/indeks metaverileri ve varsa boşluk/kurtarma bağlamı; dizin indeksinin yazılma zamanı değildir. |
| USN Günlüğü | Günlük zamanı ve değişiklik nedenleri; bir kayıt birden fazla değişikliği tam başlangıç saati olmadan açıklayabilir. |
| LogFile | Saklı redo/undo zaman görüntüleri ve dosya adları ayrı tutulur. LSN ve dosya ofsetleri kimliktir, tarihe çevrilmez. |
| Prefetch | Yürütme süreleri kaydedildi. `--sort` ile bunlar, eşleşen Prefetch zaman çizelgesi dışa aktarımıyla birleştirilir. Başvurulan birim oluşturma meta verileri ayrı olarak etiketlenir. |
| Tarayıcı SQLite veritabanları | Ziyaretler, kurtarılan son ziyaretler, indirme başlangıç/bitişi, çerez zamanları ve form ilk/son kullanım ya da silme izleri. Bitiş başarı/yürütme kanıtı değildir; çerez son kullanımı yapılandırılmış süredir. Kurtarma durumu, güven ve köken korunur. |
| Internet Explorer / eski Edge WebCache | Erişim, oluşturma/değiştirme, son kullanma, eşitleme, indirme, çerez ve kapsayıcı bakım zamanları. Önbellek bakımı kullanıcının sayfayı açtığını otomatik olarak kanıtlamaz. |
| Outlook PST, OST, MSG ve OFT | İleti gönderimi/teslimi, klasör/öğe/ek oluşturma-değiştirme, takvim ve görev zamanları, hatırlatmalar ve açıkça zaman türündeki özellikler. Planlanan saat toplantı, hatırlatma veya yürütme kanıtı değildir; teslim, okuma kanıtı değildir. |
| Kullanıcı Erişim Günlüğü (UAL) | İlk/son erişim, veritabanı ekleme/oluşturma, ilk/son görülme ve sistem açılışı. Rol, hesap, kaynak adresi ve günlük sayaçlar bağlamdır. Günlük sayılar gece yarısı oturumlarına dönüştürülmez; zamansız kimlik ve DNS satırları ek raporda kalır. |
| Amcache | Envanter anahtarı yazımı, üst birim bağlamı, kurulum ve PE bağlama metaverileri. Envanter veya LinkDate tek başına yürütme kanıtı değildir. |
| ShimCache | Dosya değişikliği, önbellek güncellemesi ve anahtar son yazımı otomatik olarak yürütme zamanı değildir. |
| SRUM | Sağlayıcı zamanı/aralığı, bağlantı başlangıcı, açık başlangıç/bitiş ve uygulama kimlikleri. Bayt, süre ve sayaçlar tarih değildir. |
| LNK | Hedef MAC, izleme kimliği ve shell öğesi zamanları saklı hedef metaverisidir; kısayolun açılma zamanı olmayabilir. |
| Jump Lists | Son kullanım, izleme kimliği ve gömülü LNK hedef zamanları. Toplama sistemindeki kapsayıcı dosya zamanları varsayılan olarak dışlanır. |
| Defender karantinası | Kayıt zamanı ve kaynak oluşturma/erişim/yazma zamanları. Çıkarma saati uç nokta etkinliği değildir. |
| BAM, UserAssist, FeatureUsage | Kayıtlı etkinlik ve ayrı anahtar yazma bağlamı. Zamansız sayaç olay üretmez. |
| RecentDocs, MRU, Aç/Kaydet MRU, ShellBags, Office | Kullanım/güven/shell zamanları ve anahtar bağlamı. Liste anahtarının zamanı her öğenin son kullanımı değildir. |
| SAM ve LSA sırları | Hesap giriş/parola/son kullanma, sır güncelleme ve anahtar metaverileri kimlik doğrulama olay günlüklerinin yerini almaz. |
| Hizmetler, TaskCache, kayıt defteri kalıcılığı, BCD | Anahtar/yapılandırma ve açık görev zamanları. Hizmet anahtarı yazımı çalıştırılma kanıtı değildir. |
| Ağlar ve USB | Ağ profili/kira, aygıt kurma/geliş/çıkarma ve anahtar bağlamı. Yerel USB kullanıcı MountPoints2 zamanları ilgili kullanıcı kovanını korur. |
| Sistem bilgisi, kurulu programlar, AppCompat, uygulama geçmişi | Kurulum, kapanma, envanter/etkinlik ve anahtar metaverileri. Yerel profil/sürüm dizilerinin iç zamanları korunur. |
| Denetim ilkesi, güvenlik ayarları, MuiCache, genel kayıt defteri | Varsa anahtar son yazımı; zamansız değer ve sayaçlar yerel girdide kalır. |
| Persistence Hunter | Destekleyici anahtar yazımı ve ilişkili değişim. Bulgu tek başına zamanlanmış kurulum veya ihlal hükmü değildir. |
| Hayabusa | Kural, seviye, kanal, olay ID ve bağlamla olay/uyarı zamanı. Farklı kurallar ve korelasyon satırları ayrı kalır. |

Varsayılan dışa aktarma, tekrarlanan meta veriler de dahil olmak üzere tüm zaman damgası tahminlerini korur. Genel sıralama veya yinelenen konsolidasyon gerçekleştirmez. `--sort` ile kopya projeksiyonlar birleştirilir; bu, aynı girdilerin sıralanmamış bir şekilde dışa aktarılmasından daha az satır üretebilir. Farklı anlamlara veya kanıt kaynaklarına sahip birden fazla eşit zaman damgası her iki modda da korunur. Yinelenen kimlik, bilgisayarı, kaynağı, yapıyı, kaydı, zaman anlamını ve kurtarma durumunu içerir. Birleştirilmiş satırlar, `DuplicateCount`'i ve sınırlı alternatif yolları/kaynak alanlarını kaydeder. Farklı kaynaklar hiçbir zaman yalnızca yolları ve zaman damgaları birbirine benzediği için birleştirilmez. Ham kaynak çözümlenemediğinde, farklı yakalamaların çökmesini önlemek için giriş-kayıt kimliği korunur.

### Hassasiyet, yerel saat ve varsayımlar { #precision-local-times-and-assumptions }

Bilinen UTC alanları ve açık ofsetler UTC'ye çevrilir. 100 ns FILETIME dahil özgün hassasiyet saklanır; önceden yuvarlanan hassasiyet geri getirilemez.

Bazı shell/FAT alanları `Z` ile yazılsa bile yerel saattir. Değer ve köken korunur ama eksik saat dilimi veya önceki dönüşüm doğrulanamaz. Kesin makineler arası ilişki için özgün artefaktı ve sistem ayarlarını inceleyin.

Artefaktın saat dilimi tanımlamadığı zamanlar sessizce UTC sayılmaz, tanılanır. Kanıt uygunsa işaretli sabit ofset verin:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Yalnızca tarihler de bu seçeneği gerektirir; gece yarısı **gün hassasiyetli** varsayımdır, gözlenen olay değildir. Saat dilimi adları ve yaz saati kabul edilmez. Eksik, sıfır, bozuk veya aralık dışı değerler yapay epoch olayı üretmez; geçerli açık 1970 tarihi korunur.

İki kategori varsayılan olarak dışlanır:

- `--include-collection-times`: toplama/analiz sistemindeki LNK/Jump List kaynak dosya zamanları, `collection_filesystem` olarak uç nokta etkinliğinden ayrılır.
- `--include-interpreted-times`: okuyucunun FILETIME olarak yorumladığı genel kayıt defteri QWORD değerleri. Çoğu sayaç olduğundan `interpreted_value` etiketi taşır.

## Ek raporlar { #companion-reports }

`SuperTimeline.csv` için komut ayrıca şunu da yazar:

- `SuperTimeline.csv-Untimed.jsonl`: kullanılabilir eşlenmiş olay zamanı olmayan yerel kayıt konumları.
- `SuperTimeline.csv-Diagnostics.jsonl`: geçersiz/belirsiz zamanlar, bilerek atlanan ortak metaveriler ve özgün kaydı gerektiren sınırlamalar.
- `SuperTimeline.csv-Summary.json`: dosyalar, artefakt sayıları, filtrelenen/aday/birleştirilen olaylar, çıktı, işçiler, sıralama bütçesi/parçaları/geçişleri ve süre.

Tamamlayıcı adlar, çıktı dosya adının tamamını içerir. Örneğin, `SuperTimeline.jsonl`'in kendi `SuperTimeline.jsonl-Summary.json`'i ve karşılık gelen zamanlanmamış/tanılama raporları vardır. Aynı köke sahip CSV ve JSONL dışa aktarmaları bir arada bulunabilir; `-C` yalnızca mevcut bir sonuç kümesini değiştirmek için gereklidir.

`InvalidTimestamps` yerine `UnresolvedTimeZones` gerekçeli saat dilimi varsayım artışını bekleyen geçerli tarihler. Her ikisi de tanılamada izlenebilir olmaya devam ediyor; hiçbir zaman damgası icat edilmedi.

Geçerli süreleri istenen aralığın dışında olan kayıtlar, süresiz değil, filtrelenmiş olarak sayılır. Aday etkinliği sayımları yinelenen konsolidasyondan önce gelir. `Sorted` ve `DuplicateConsolidation`, `--sort`'in etkin olup olmadığını kaydeder. Bu olmadan, `DuplicateEvents`, `InitialSortChunks` ve `MergePasses` sıfırdır ve sıralama arabelleği ayrılmadığından `SortMemoryBudgetMiB`, `null` olur. `Complete: true`, seçilen tüm ayrıştırılmış girişlerin başarıyla işlendiği anlamına gelir; bu, her eserin toplandığı, her alanın zaman damgalı olduğu veya her saldırının tespit edilebildiği anlamına gelmez.

CSV, sistem profilleri, USB kullanıcı bağlama noktaları ve shell yapılarını düzleştirebilir; yerel JSON/JSONL daha zengindir. CSV ile kaybolan veri geri üretilemez. Desteklenmeyen artefaktlar ve bozuk girdiler kısmi başarı değil hata verir.

## Performans ve güvenli çıktı { #performance-and-output-safety }

Girişler paralel olarak aktarılır ve işlenir. Varsayılan olarak her çalışan biçimlendirilmiş bir olay parçası yazar; parçalar, olay yapılarını sıralamadan veya yeniden okumadan, giriş dosyası sırasına göre birleştirilir. CSV, birden fazla dosya işlendiğinde bile tek bir başlığa sahiptir. Çıktı sırası çalışan sayılarına göre belirleyicidir; kronoloji anlamına gelmez.

`--sort` ile arabellekler özel bir geçici dizine dökülür ve ardından sınırlı birleştirme geçişleri kronolojik çıktı üretir. Sıralama, 1970'ten önceki tarihler de dahil olmak üzere tam tamsayı saniye ve nanosaniyeleri kullanır. Eşit zamanlı sıralama, çalışan sayıları ve bütçeleri sıralama açısından belirleyicidir. Tekrarlanan geçişler dahil, ara ve son birleştirmeler sırasında işlenen her olay için bir ilerleme çubuğu ilerler. Bir terminalde görünür ve çıktı yeniden yönlendirildiğinde gizlenir.

`--memory-limit` yalnızca `--sort` için geçerlidir: bu, toplam **sıralama arabellek bütçesidir**, bir süreç RSS sınırı değildir. Giriş kayıtları, JSON nesne/dize yükü, CSV arabellekleri ve birleştirme kayıtları ek bellek gerektirir. Küçük bütçeler için işçi sayısı azaltılır. `--max-record-size`, ayrıştırılmış bir giriş kaydını sınırlar (varsayılan 16 MiB); güzel bir JSON dizisi/raporu, bir üst düzey giriş belgesi olarak sayılır. Daha büyük belgeler için daha yüksek bir limit veya tercihen JSONL gerekebilir. CSV ayrıştırıcının ileri okuması da bu sınıra katkıda bulunur.

Geçici disk alanı, nihai sonuçlara ek olarak biçimlendirilmiş olay parçalarını (varsayılan) veya genişletilmiş sıralama kayıtlarını ve birleştirme geçişlerini (`--sort`) barındırmalıdır. Her iki mod da geçici çıktı aşamalandırmasını ve başarısız girişlerin yayınlanmasına karşı korumayı korur. Bir kayıt birçok olaya neden olabilir, dolayısıyla zaman çizelgesi girdiden çok daha büyük olabilir. Uygun olduğunda zaman aralığını/yapıları filtreleyin ve yeterli boş alana sahip geçici bir birim seçin.

Ctrl+C birlikte iptal eder, özel sıralama/hazırlama dizinlerini kaldırır ve `130` koduyla çıkar. Linux ve macOS'te bu aynı zamanda SIGTERM ve SIGHUP için de geçerlidir. Yayın başlamadan önce raporların okunması, normalleştirilmesi, birleştirilmesi ve yazılması sırasında iptal kontrolü yapılır. Yakalanamayan sonlandırma veya güç kaybı hala geçici dosyalar bırakabilir.

Mevcut çıktı için `-C` gerekir. Girdiler, manifestler ve korunan kanıtlar ezilemez. Girdi/sıralama hatası eski çıktıyı korur. Dosyalar hedef yanında hazırlanıp ayrı değiştirilir, çizelge en sondadır. Dört dosya **tek işlem değildir**; son G/Ç hatasında bazı ek raporlar değişmiş olabilir. Eski tam çıktıyı korumak için yeni temel ad kullanın.

## Ayrıntılar { #further-details }

- [Timesketch CSV/JSONL aktarımı](https://timesketch.org/guides/user/import-from-json-csv/)
- [Plaso zaman genişletme ve eski CSV](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Plaso çıktı ve biçimlendirme](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Kanıt ve yorumlama](../overview/evidence-model.md)
- [Çıktı biçimleri](../getting-started/output-formats.md)

Güncel Juntei, `Comp`, `Filename`, `Args`, `Attr` gibi kısa adlar kullanır. MFT CSV ayrıca `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN`, `HasADS` alanlarını kabul edip sıra nesillerini ve DOS/ADS ayrımını korur. Eski adlar desteklenir; `SourceField` özgün yazımı tutar. Hayabusa kendi `Computer` alanını korur. Önyükleme sektörü/güvenlik tanımlayıcısı olay zamanı taşımadığından zamansız rapordadır.

## Yeni çok tablolu ayrıştırıcılar { #newer-multi-table-parsers }

Yerel JSON/JSONL tablo adını içerir. Bağımsız CSV için tarayıcı, WebCache ve Outlook sütun şemasını kullanır; CSV dosyasını yeniden adlandırmak veya kaynak eki yorumlamayı değiştirmez. UAL bilinen zaman alanlarını kullanır, sayaçları veya tarih benzeri metni tarih saymaz.

Outlook özellikleri yalnızca bildirilen zaman/çoklu zaman türünde yansıtılır. Tarihli metin metin kalır. Gömülü iletiler kimlik/üst referanslarını korur; çıkarma zamanı ekin kayıtlı zamanı yerine geçmez. Zamansız satırlar yerel veri ve ek raporda izlenebilir.

Aynı CSV/JSONL şemaları geçerlidir; yeni artefakt yüzlerce CSV sütunu eklemez. Özel bağlam `Details` içindedir.
