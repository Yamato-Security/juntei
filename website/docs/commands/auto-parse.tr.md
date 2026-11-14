# Toplanan Verilerin Otomatik Ayrıştırılması { #automatic-collection-parsing }

`auto-parse`, birden çok bilgisayardan alınmış bir Windows triyaj koleksiyonunu yapılandırılmış bir
analiz dizinine dönüştürmenin en hızlı yoludur. Ona kanıt kök dizinini ve yeni bir çıktı dizinini
verin. Toplanan sistemleri keşfeder, bilgisayar adlarını belirler ve uyumlu her Juntei ayrıştırıcısını
paralel olarak çalıştırır.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

Varsayılan çıktı CSV'dir. Bu CSV sonuçlarını korurken `jq` veya SIEM içe aktarımı için bir JSONL zaman
çizelgesi ekleyin:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "Çıktı ve kanıt ağaçlarını ayrı tutun"
    Juntei, `<EVIDENCE>` içinde bulunan veya `<EVIDENCE>` dizinini içeren bir çıktı dizinini reddeder.
    Bu, üretilen verilerin yeniden kanıt olarak keşfedilmesini ve `--clobber` seçeneğinin kaynak
    koleksiyondaki dosyalara dokunmasını önler.

## VHDX disk imajları { #vhdx-disk-images }

`--vhdx` ile `auto-parse`, [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) üzerinden **Linux, macOS ve Windows** üzerinde VHDX’i yerel olarak okur. OS bağlaması, FUSE veya yönetici yetkisi gerekmez.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Ayrıştırıcılar dosya yolu gerektirir; Juntei ayrıştırıcıların okuduğu dosyaları sınırlı tamponlarla özel bir geçici dizine kopyalar: kayıt defteri hive'ları ve işlem günlükleri, NTFS meta veri dosyaları, veritabanları, Prefetch dosyaları, kısayollar, Jump List'ler, Outlook dosyaları, Geri Dönüşüm Kutusu indeks dosyaları ve Defender karantina klasörleri. Diğer her dosya özgün değiştirme zamanını koruyan boş bir yer tutucu olur; böylece dizinler ve yollar diskteki gibi görünür. persistence-hunter bir yer tutucunun içeriğini imajdan yalnızca gerektiğinde okur, örneğin bir kalıcılık kaydının gösterdiği yürütülebilir dosya. **Sonuçlara ek olarak kopyalanan dosyalar için alan gerekir.** Destekleniyorsa sıfır alanlar seyrek yazılır. Adlar, dizin yapısı, sabit bağlantılar ve değiştirme zamanları korunur. LNK ve Jump List kaynak zamanları geçici dosyalardan değil özgün NTFS metadata anlık görüntüsünden alınır. Ayrıştırıcı hatalarıyla da normal bitiş kopyaları siler; zorla sonlandırma dosya bırakabilir. Kaynak salt okunurdur; bekleyen VHDX günlükleri yalnızca bellekte uygulanır.

Bilgisayar keşfi, CSV/JSON/JSONL ve `-j` normal işlem hattını kullanır. `manifest.json` içindeki `Image` özgün imajı, `EvidenceRoot` iç göreli yolu tanımlar. Okuma hataları ve güvensiz adlar `ImageErrors` ve `manifest.csv` içindeki başarısız satırlara yazılır. Okunabilir sonuçlar korunur; eksik girdiler 1 döndürür. Güvensiz veya belirsiz adlar yeniden adlandırılmaz. Adlandırılmış NTFS akışları ek dosya olarak çıkarılmaz; normal dosya olarak toplanmış eserler kullanılabilir.

Ayrı [`mount` komutu](mount.md) dosya sistemini başka programlara sunar ve Linux FUSE gerektirir. `auto-parse --vhdx` bunu kullanmaz.

Hazırlama alanı varsayılan olarak çıktı dosya sistemindeki özel bir dizini kullanır. Başka bir diskteki mevcut dizini seçmek için `--temp-dir` kullanın. Özgün meta veriler çıktı dosya sisteminde ayrı tutulur; böylece hazırlama diski dolsa da okunabilen dosyalar ayrıştırılabilir. Hem kopyalar hem sonuçlar için yer ayırın; çıktı dosya sistemi doluysa sonuçlar kaydedilemez. Ctrl+C ayrıştırıcı süreçlerini durdurur, özel çalışma alanını temizler ve 130 koduyla çıkar. SIGKILL veya güç kesintisi kopya bırakabilir. İmaj ararken okunamayan alt dizinler atlanır; okunamayan giriş kökü hata olmaya devam eder.

Hazırlama dizini çıktı dizininin dışında olmalıdır. Linux ve macOS üzerinde SIGTERM ve SIGHUP da ayrıştırıcı süreçlerini durdurur ve çalışma alanını temizler.

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## E01 ve Ex01 disk imajları { #e01-and-ex01-disk-images }

`--e01`, fiziksel EWF1 (`.E01`) ve EWF2 (`.Ex01`) imajlarını **Linux, macOS ve Windows** üzerinde yerel olarak okur; FUSE, işletim sistemi bağlaması veya yönetici yetkisi gerekmez. [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) ve yukarıdaki NTFS ayrıştırma hattını kullanır.

Bölünmüş imajın ilk parçasını belirtin. `.E02` dahil tüm parçaları özgün adlarıyla aynı dizinde tutun. Tam diskler (MBR/GPT) ve tek NTFS birimleri desteklenir. Karışık dizinler için `--vhdx --e01` kullanın.

Aynı geçici alan ve kaynak meta verisi gereksinimleri geçerlidir. E01 imajları genellikle diskin tamamını içerir ve tüm imajlar ayrıştırma başlamadan önce hazırlanır, ancak yalnızca ayrıştırıcıların okuduğu dosyalar kopyalanır; bu genellikle kullanılan alanın küçük bir bölümüdür. İmajlar salt okunurdur. Edinim boşlukları ve açma veya okuma sırasında bulunan hasarlar `ImageErrors` ve `manifest.csv` içinde bildirilir; okunabilen sonuçlar korunur ve çıkış kodu 1 olur. Yalnızca erişilen veriler denetlenir; tüm parçalar veya medyanın tam özeti doğrulanmaz. Mantıksal `.L01` ve `.Lx01` dosyaları reddedilir; parola seçeneği yoktur.

Ayrı `mount` komutu hâlâ Linux FUSE gerektirir; yerel otomatik ayrıştırma gerektirmez.

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## Çıktı saat dilimi { #output-time-zone }

Ayrıştırılan zaman damgaları, analistin bilgisayarının yerel saat diliminden bağımsız olarak varsayılan
olarak UTC'dir. UTC, Windows olay günlüğü biçimini kullanır: `2026-10-20T08:46:38.2001668Z`; yani
ISO 8601 tarih/saat, tarih ile saat arasında `T` ve UTC için `Z`. Mevcut kesirli basamaklar korunur.
`manifest.json` dosyasındaki `GeneratedUTC` de bu biçimi kullanır. Konsoldaki `Start time` ve
`End time`, çıktı saat diliminden bağımsız olarak analistin bilgisayarının yerel saatidir; örneğin
`2026-11-04 09:15:02`.

`-Z, --timezone` ile sabit bir görüntüleme ofseti seçin:

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

Ofset, işaretli ve iki basamaklı bir `HH:MM` değeridir (saat `00–23`, dakika `00–59`). `JST` gibi
adlar, `+9` gibi kısaltılmış değerler ve geçersiz aralıklar, örnek söz dizimiyle birlikte bir hata
üretir. UTC için seçeneği atlayın veya `--timezone "+00:00"` kullanın. Sabit bir ofset yaz saati
kurallarını uygulamaz.

Seçenek; artefakt başına CSV, JSON ve JSONL sonuçlarındaki ayrıştırılmış zaman damgalarına ve ek `-j`
JSONL dışa aktarımına uygulanır. Örneğin `2026-10-20T08:46:38.2001668Z`,
`2026-10-20T17:46:38.2001668+09:00` olur; ikisi de aynı anı tanımlar.

Alan adları, `UTC` ile biten mevcut adlar dahil olmak üzere sabit kalır. Dönüştürülen değerler seçilen
ofseti açıkça taşır. Her iki bildirim dosyasındaki `OutputTimeZone`, istenen görüntüleme ofsetini
kaydeder; `GeneratedUTC` ise UTC çalışma meta verisi olarak kalır. Ayrıştırıcı başına günlükler,
ayrıştırıcıların yerel UTC özetlerini korur.

Ham FILETIME/epoch sayıları, kayıt defteri değeri sözlükleri, kaynak saat dilimi yapılandırması, metin
meta verileri, yalnızca tarih içeren değerler ve ofseti bilinmeyen yerel saat değerleri korunur. Juntei,
bilinmeyen bir yerel saate `Z` eklemez. FAT'tan türetilen shell item saatlerinin (ShellBags/OpenSaveMRU
içindeki `Item*UTC` ve LNK/Jump List `TargetItems`) kaydedilmiş bir ofseti yoktur; eski UTC etiketleri
veya gösterilen `Z` soneki UTC olduklarını kanıtlamaz, bu nedenle ofset seçeneği bunları korur.
Serbest metin özetleri içeren CSV sütunları kaydedilmiş metinlerini korur; JSON/JSONL, iç içe
ayrıştırılmış zaman damgalarını yapılandırılmış alanlar olarak korur.

UTC, mevcut ayrıştırıcı çıktısını doğrudan kullanır. Açıkça belirtilmiş sıfır dışı bir ofset, her
ayrıştırıcı çalışanı içinde akışlı bir biçimlendirme geçişi ekler; bu, kanıtı değiştirmeden görüntülenen
değerleri değiştirir.

Bir ofset bir zaman damgasını `0000–9999` yıllarının dışına taşıyacaksa, o değer özgün açık ofsetini
korur. Dokuzdan fazla kesirli basamağa sahip değerler de yuvarlanmak yerine değiştirilmeden kalır.
Juntei, ayrıştırıcının `*-logs.txt` dosyasına bir uyarı kaydeder ve diğer zaman damgalarını dönüştürür.
Bozuk CSV/JSON veya bir çıktı hatası yine de görevi başarısız kılar ve etkilenen özgün sonuç dosyasını
değiştirmeden bırakır.

## Neleri keşfeder { #what-it-discovers }

Juntei, büyük/küçük harf duyarsız olarak `System32`, `Prefetch` veya `AppCompat` dizini içeren `Windows`
adlı dizinleri özyinelemeli olarak arar. Son ikisinin tanınması, yalnızca Prefetch veya Amcache içeren
hedefli koleksiyonları da destekler. Bu, `AppData\Roaming\Microsoft\Windows` gibi kullanıcı profili
dizinlerini dışarıda bırakır. `C:\Windows.old\Windows` gibi iç içe eski kurulumlar, yinelenen
bilgisayarlar haline gelmek yerine dıştaki toplanmış birime atanır. Varsayılan en fazla derinlik
10'dur. Bu, aşağıdaki gibi yaygın Velociraptor yollarını kapsar:

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

Sınırı yalnızca koleksiyon daha derinde iç içe geçmişse değiştirin:

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

Keşfedilen her sistem için bilgisayar adı şu sırayla seçilir:

1. En yakın üst dizindeki `client_info.json` dosyasındaki `hostname` (değeri bildirim dosyasında
   `FQDN` olarak korunur).
2. Toplanmış bir `SYSTEM` kayıt defteri kovanındaki `ComputerName`.
3. Sürücü ve Velociraptor `uploads/auto` iskeleti atlanarak ve `_F.…` gibi bir sonek kaldırılarak
   bulunan en yakın yararlı koleksiyon dizini adı.

Adlar, dizin adı olarak kullanılmaya uygun hale getirilir. Yinelenen adlar hem çıktı dizinlerinde hem
de `Comp` değerinde sayısal bir sonek alır; böylece ayrı sistemler JSON'da ayırt edilebilir kalır.
Bildirim dosyası, ana bilgisayar adını keşfedildiği gibi ayrı bir `Hostname` alanında tutar.

Sistemleri listeledikten sonra `auto-parse`, `Parser tasks` satırının hemen üstüne bir `Timezone:`
satırı yazdırır. Bu satır seçilen çıktı saat dilimini gösterir: varsayılan olarak `UTC` veya
`--timezone "+09:00"` kullanıldığında `UTC+09:00` gibi bir değer. Etiket yeşildir ve değer, diğer
özet değerleri gibi terminalin varsayılan metin rengini kullanır. `-K, --no-color` satırı düz tutar.
Toplanan sistemlerin saat dilimi yapılandırması [Sistem Bilgileri](../artifacts/system-info.md)
sonuçlarında bulunur.

## Çalıştırdığı ayrıştırıcılar { #parsers-it-runs }

Bir ayrıştırıcının başarısız olması diğerlerini durdurmaz. `auto-parse`, keşfedilen her bilgisayar
için aşağıdaki işleri çalıştırır. İşler, `parse-ntfs mft`, `parse-registry amcache`, `parse-ese srum`
ve `parse-file prefetch` gibi, el ile ayrıştırmayla aynı kaynağa göre gruplanmış komutları kullanır:

- Amcache
- `$MFT`
- `$LogFile` NTFS redo/undo kayıtları
- Prefetch kayıtları ve ayrı bir Prefetch yürütme zaman çizelgesi
- USN Journal
- `$I30` dizin indeksleri
- `$Boot` önyükleme sektörleri ve `$Secure:$SDS` güvenlik tanımlayıcıları
- SRUM
- Kullanıcı Erişim Günlüğü (UAL)
- Web tarayıcısı geçmişi, indirmeleri, çerezleri ve form geçmişi (Chromium ve Firefox tabanlı tarayıcılar)
- Internet Explorer'ın ve eski Edge'in WebCache verileri: geçmiş, indirmeler, çerezler ve önbellek girdileri
- Outlook PST, OST, MSG ve OFT dosyaları: klasörler, öğeler, alıcılar, ekler, kişiler, takvim öğeleri ve görevler
- LNK kısayolları
- Jump Lists
- Geri Dönüşüm Kutusu: silinen öğelerin özgün yolları, boyutları, silinme zamanları ve kullanıcıları
- ShimCache
- Persistence Hunter (izin/engelleme profilleriyle 131 kaynaklar arası kalıcılık denetimi)
- Defender karantina meta verileri
- Aç ve Farklı Kaydet iletişim kutusu geçmişi
- kurtarılabilir silinmiş veriler dahil tüm kayıt defteri anahtarları ve değerleri
- hizmetler, RecentDocs, TaskCache, SAM hesapları, kayıt defteri otomatik başlatma konumları,
  UserAssist, BAM, yüklü programlar, MuiCache, ShellBags ve USB aygıtları
- sistem bilgileri: adlar, Windows sürümü, saat dilimi, son kapatma, profiller ve SID'ler
- denetim ilkesi: sistemin hangi güvenlik olaylarını günlüğe kaydedecek şekilde ayarlandığı
- önyükleme yapılandırması (BCD ve BCD-Template)
- ağ profilleri, arabirimler, paylaşımlar, bağlantı noktası proxy'leri ve yollar
- LSA sırlarının adları ve zamanları ile kullanımdaki önbelleğe alınmış etki alanı oturum açmaları
  (hiçbir şeyin şifresi çözülmez)

Persistence Hunter, kısmi izin eşleşmeleri dahil her önem düzeyini korur (`--min-severity info`).
`LOW`, incelenmesi gereken bir sınıflandırmadır; eksik imza meta verileri kötü amaçlı komutlar için de
bunu üretebilir. Otomatik envanter, el ile çalıştırılan komutun varsayılan orta eşiğini kullanarak bu
girdileri atmaz.

Hunter, her otomatik ayrıştırıcı işleminin içinde tek bir çalışan kullanır. Bu, `auto-parse -T <N>`
komutunun işlem bütçesini her alt işlemde CPU boyutunda bir başka havuzla çarpmasını önler. Bağımsız
`persistence-hunter -T <N>`, hunter'ın kendi çalışan iş parçacıklarını denetler.

CSV `$MFT` çıktısı tüm zaman damgalarını ve kısa adları içerir. Kayıt defteri çıktısı ham verileri ve
`REG_QWORD` değerlerinin FILETIME yorumunu içerir. JSON ve JSONL SRUM çıktısı değer durumu meta
verilerini içerir. Defender karantina çıkarımı isteğe bağlı kalır.

`--mft-slack` ile `$MFT` çıktısı, FILE kayıtlarının slack alanında kalan dosya adlarını da [`parse-ntfs mft -x`](../artifacts/mft.md) ile aynı biçimde içerir: CSV'de `FromSlack` ile işaretlenmiş satırlar, JSON ve JSONL'de `SlackFileNames`. `--code-page`, LNK dosyalarına, Jump List'lere, ShellBags ve OpenSaveMRU'ya uygulanır.

Toplanmamış bir artefakt, başarısızlık değil, normal bir `skipped` görevidir.


## Dosya sistemi yeniden oluşturma { #filesystem-reconstruction }

Dosya sistemi yeniden oluşturma **varsayılan olarak kapalıdır**. Toplanan her MFT için gezilebilir bir ağaç eklemek üzere `-R, --reconstruct-filesystem` seçeneğini etkinleştirin. Mevcut tabloların adları ve içerikleri CSV, JSON ve JSONL kiplerinde değişmez.

Ağaçlar komşu özel çalışma alanında oluşturulur ve yalnızca tamamlanınca yayımlanır. İşlenen hata ve kesintiler alanı temizler, eski ağacı korur. Zorla sonlandırma `.juntei-reconstruction-*` dizinleri bırakabilir; bunlar yeniden denemeyi engellemez veya etkin birim olmaz.

Toplama yolundan çıkarılan sürücü `FileSystem/C` gibi bir dizin kullanır. MFT sürücü harfini saklamaz. Tanınan etiket yoksa `UnknownVolume` kullanılır; birden çok MFT ayrı kalır, örneğin `C-MFT-2`.

Ayrılmamış dosya ve dizinlere, silinmiş üst dizinler dahil, `-deleted` eklenir. Kalan adsız yerleşik DATA içeriği kurtarılır. Yerleşik olmayan veya ulaşılamayan içerik boş dosya olur; mantıksal boyut kurtarılan bayt sayısı değildir.

`-i30`, silindiği doğrulanmamış ek indeks adlarını gösterir. MFT yerleşik indeksleri ve aynı birimden toplanmış I30 dosyaları kullanılır. Slack, yeniden adlandırma ve dengeleme kopyaları içerebilir. Başka dosya nesillerinden içerik alınmaz.

`_Juntei-Unresolved`, üst başvuruları eksik, yeniden kullanılmış, döngüsel veya çok derin yolları içerir. Güvensiz adlar kaçırılır, çakışmalar ayrı saklanır ve özgün adlar envanterde kalır. Uzun ad varsa DOS takma adları atlanır; sabit bağlantılar bağımsız kopyalara dönüşür.

`_Juntei-Reconstruction.csv` yeniden oluşturmayı kaydeder: `DataStatus`, `LogicalSize`, `RecoveredBytes`, `OriginalPath`, `OutputPath`. Kaynak alanları: `EntryNum`, `SN`, `ParentEntryNum`, `ParentSN`, `Source`, `FromSlack`, `Offset`. Tahmini sürücü, silinme durumu ve dizin türü de kaydedilir. Kısmi envanter .partial.csv ile biter; son ad yalnızca başarıdan sonra oluşur.

`manifest.json` / `manifest.csv`: `ReconstructionRoot`, `ReconstructionInventory`, `ReconstructionStatus`, `ReconstructionError`.

Okunamayan veya hasarlı isteğe bağlı I30 girdileri `i30_input_error` olarak kaydedilir; geçerli MFT sonuçları korunur.

Boş olmayan kurtarılmış yerleşik içerik taşıyan dosyalar yazılmadan önce `.dontrun` alır. Özgün adlar envanterde kalır; boş dosyalar adlarını korur. İçeriği çalıştırmayın. Reparse noktaları, ACL, özgün zamanlar ve yürütme izinleri geri yüklenmez. Adlandırılmış akışları ayrıca çıkarın.

Yeniden oluşturma yüz binlerce dosya yaratabilir; boş dosyalar bile inode ve dizin metaverisi tüketir. Yalnızca ağaca ihtiyacınız olduğunda etkinleştirin:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> --reconstruct-filesystem
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -R
```

Üzerine yazma yalnızca eksik, ek nesne veya bağlantı içermeyen tam envanterli ağacı değiştirir. Kısmi ağaçlar ve `.DS_Store` gibi ekler korunur. Auto-parse yalnızca yeniden oluşturmayı atlar; dizini ve nedeni günlük ve iki manifeste kaydeder, diğer ayrıştırıcılar sürer. Yazma hataları ayrı bildirilir ve genel hata durumu döndürür; ayrıştırılmış sonuçlar korunur. Tekil komut engelli hedefleri reddetmeye devam eder.


## Çıktı düzeni { #output-layout }

Sonuçlar bilgisayara ve inceleme kategorisine göre düzenlenir. Kategori dizinleri `Accounts` ve
`FileAndFolderOpening` gibi PascalCase kullanır. Amcache ve Jump Lists tabloları, dosya adı önekleriyle
doğrudan kendi kategorilerinde yer alır. Bilgisayar dizini adları keşfedildikleri yazımı korur. Mevcut
çıktı dizinleri taşınmaz; eski adlandırmadan geçerken yeni bir çıktı dizini seçin. Her ayrıştırıcı,
artefakt toplanmamış olsa bile sonuçlarının yanına bir `<artifact>-logs.txt` dosyası yazar. Her
artefaktın farklı sütunları olduğundan CSV ayrı tablolar tutar:

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── BrowserActivity/
│   │   ├── Browser-Cookies.csv
│   │   ├── Browser-Downloads.csv
│   │   ├── Browser-FormHistory.csv
│   │   ├── Browser-History.csv
│   │   ├── Browser-logs.txt
│   │   ├── WebCache-Containers.csv
│   │   ├── WebCache-Cookies.csv
│   │   ├── WebCache-Downloads.csv
│   │   ├── WebCache-Entries.csv
│   │   ├── WebCache-History.csv
│   │   └── WebCache-logs.txt
│   ├── Communication/
│   │   ├── Outlook-Attachments.csv
│   │   ├── Outlook-Calendar.csv
│   │   ├── Outlook-Contacts.csv
│   │   ├── Outlook-Files.csv
│   │   ├── Outlook-Folders.csv
│   │   ├── Outlook-Items.csv
│   │   ├── Outlook-Recipients.csv
│   │   ├── Outlook-Tasks.csv
│   │   └── Outlook-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── RecycleBin.csv
│   │   ├── RecycleBin-logs.txt
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| Kategori | Artefaktlar |
|---|---|
| `Accounts` | SAM yerel hesapları, LSA sırlarının adları ve zamanları |
| `Applications` | Uygulama geçmişi (üçüncü taraf araçların izleri) |
| `Applications` | Yüklü programlar |
| `BrowserActivity` | Web tarayıcısı geçmişi, indirmeleri, çerezleri ve form geçmişi; kurtarılan silinmiş girdilerle birlikte; Internet Explorer'ın ve eski Edge'in WebCache verileri |
| `Communication` | Outlook PST, OST, MSG ve OFT dosyaları: klasörler, öğeler, alıcılar, ekler, kişiler, takvim öğeleri ve görevler |
| `ExternalDevices` | USB aygıtları ve birimleri |
| `FileAndFolderOpening` | Jump Lists, LNK kısayolları, Microsoft Office, MRU listeleri, Aç ve Farklı Kaydet iletişim kutuları, RecentDocs, ShellBags |
| `FileExistence` | Amcache, Geri Dönüşüm Kutusu, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, USN Journal |
| `NetworkActivity` | Ağ profilleri, arabirimler, paylaşımlar, bağlantı noktası proxy'leri ve yollar; SRUM bağlantıları, trafik ve anında iletme bildirimleri |
| `LateralMovement` | Kullanıcıların giden Uzak Masaüstü, PuTTY ve WinSCP bağlantıları, eşlenmiş sürücüler ve paylaşımlar; Kullanıcı Erişim Günlüğü (bir sunucunun rollerinin istemcileri) |
| `Persistence` | Önyükleme yapılandırması (BCD), kayıt defteri otomatik başlatmaları, hizmetler, TaskCache |
| `ProcessExecution` | Uygulama uyumluluğu, BAM ve DAM, FeatureUsage ve sabitlenmiş öğeler, MuiCache, Prefetch, Prefetch zaman çizelgesi, SRUM uygulama/CPU etkinliği, UserAssist |
| `Registry` | Kurtarılmış kayıtlar dahil genel kovan anahtarları ve değerleri |
| `SecurityProducts` | Defender karantina meta verileri ve isteğe bağlı çıkarılmış içerik; güvenlik ayarları (Defender, UAC, kimlik bilgisi koruması, RDP, güvenlik duvarı, günlük kaydı) |
| `SystemInformation` | Adlar, Windows sürümü, saat dilimi, son kapatma, profiller, SID'ler ve denetim ilkesi; SRUM enerji/kapasite, kimlik eşlemeleri ve bilinmeyen sağlayıcılar |

Kategoriler incelemeleri düzenlemeye yardımcı olur; belirli bir etkinliğin kanıtı değildir. Örneğin,
Amcache ve ShimCache girdileri tek başına yürütmeyi kanıtlamaz. SRUM kaynak kullanımını kaydeder; bir
programın çalıştığına dair sonuç çıkarmadan önce artefakt kılavuzuna başvurun.

CSV, JSON ve JSONL aynı kategorileri kullanır. Ayrıştırıcı işi başına bir `<artifact>-logs.txt`
dosyası, işin tam komutunu ve iletilerini içerir; örneğin `HOST1/FileSystem/MFT-logs.txt` ve
`HOST1/ProcessExecution/Prefetch-logs.txt`. JSON ve JSONL aynı günlük dosya adlarını kullanır.
Amcache ve Jump Lists'in her birinin, önekli tablolarının yanında ortak bir günlüğü vardır. SRUM
sağlayıcı tabloları doğrudan `NetworkActivity`, `ProcessExecution` ve `SystemInformation` içine
yerleştirilir ve ortak bir `SystemInformation/SRUM-logs.txt` dosyası bulunur. Hiçbir `logs/` alt
dizini oluşturulmaz.

Varsayılan CSV dosya adları şunlardır: `Registry-SAM.csv`, `LNK.csv`, `RecycleBin.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv` ve
`DefenderQuarantine.csv`. Kayıt defterinden türetilen tablolar `Registry-` önekini kullanır; buna
`Registry-ShimCache.csv`, `Registry-UserAssist.csv`, `Registry-BAM.csv` ve `Registry-SystemInfo.csv`
dahildir. Genel kovan anahtarları ve değerleri `Registry-Dump.csv` içindedir. Amcache tabloları
`FileExistence/Registry-Amcache-Devices.csv`, `Registry-Amcache-Files.csv`,
`Registry-Amcache-Findings.csv` ve `Registry-Amcache-Summary.csv` dosyalarıdır. Jump Lists,
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` ve
`JumpLists-CustomDestinations.csv` dosyalarını kullanır. Amcache veya Jump Lists alt dizini yoktur.
SRUM tabloları, kategorilerinin hemen altında `SRUM-` dosya adlarını kullanır; örneğin
`NetworkActivity/SRUM-NetworkConnections.csv`.
Kullanıcı Erişim Günlüğü tabloları `LateralMovement` içindedir; her tablo için bir dosya vardır, örneğin
`UAL-Clients.csv` ve `UAL-SystemIdentity.csv`, bunlarla birlikte `UAL-logs.txt` bulunur; bkz.
[Kullanıcı Erişim Günlüğü (UAL)](../artifacts/ual.md#automatic-parsing).
Web tarayıcısı tabloları `BrowserActivity` içindedir: `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` ve `Browser-FormHistory.csv`, bunlarla birlikte `Browser-logs.txt` bulunur; bkz.
[Web Tarayıcıları](../artifacts/browser.md#automatic-parsing). Internet Explorer'ın ve eski Edge'in
WebCache tabloları da oradadır: `WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv` ve `WebCache-Containers.csv`, bunlarla birlikte
`WebCache-logs.txt` bulunur; bkz. [Internet Explorer ve Eski Edge](../artifacts/webcache.md#automatic-parsing).
Outlook tabloları `Communication` içindedir: `Outlook-Files.csv`, `Outlook-Folders.csv`,
`Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`, `Outlook-Contacts.csv`,
`Outlook-Calendar.csv` ve `Outlook-Tasks.csv`, bunlarla birlikte `Outlook-logs.txt` bulunur; bkz.
[Outlook](../artifacts/outlook.md#automatic-parsing).
[SRUM yerleşiminin tam tablosuna](../artifacts/srum.md#categorized-auto-parse-output) bakın.

Günlükler, büyük harfle başlayan artefakt adıyla eşleşir; örneğin `Registry-SAM-logs.txt`,
`PrefetchTimeline-logs.txt` ve `Registry-UserAssist-logs.txt`. JSON/JSONL dosyaları aynı gövde
adlarını kullanır; örneğin `MFT.jsonl`; sorgular için kaydın `Artifact` tanımlayıcısı `mft`
değeridir. Çok tablolu çıktıların tablolarının yanında `Registry-Amcache-logs.txt` veya
`JumpLists-logs.txt` bulunur. Ortak SRUM günlüğü `SystemInformation` içindedir.


`manifest.json`; keşfedilen özgün `Hostname` değerini, benzersiz çıktı/kayıt `Comp` kimliğini, ana
bilgisayar adının kökenini, kanıt yollarını, artefakt `Category` değerini, komut satırlarını, süreyi,
çıkış kodunu, sonuç yolunu, günlük yolunu, istenen JSONL zaman çizelgesini ve herhangi bir dışa
aktarma hatasını, Defender çıkarmanın istenip istenmediğini
ve her görevin
başarılı olup olmadığını, atlanıp atlanmadığını veya başarısız olup olmadığını kaydeder.

Auto-parse hiçbir mutlak yol yazmaz; böylece sonuçlar, kanıtların, sonuçların veya Juntei'nin analiz
sisteminde nerede olduğu gösterilmeden paylaşılabilir:

- Kanıt yolları kanıt dizinine göredir. Terminal her bilgisayarı kanıt içindeki diziniyle listeler
  (`HOST1: ./HOST1_F`); ayrıştırıcılar kanıt dizininde çalışır ve onlara bu tür yollar verilir.
  Sonuçların kaynak yolu sütunları (`SourceFile`, `SourcePath`, …), günlükler ve bildirim
  dosyalarındaki `EvidenceRoot` ve `WindowsDirectories`, her dosyanın kanıt içinde nerede olduğunu
  gösterir (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`).
- Bildirim dosyalarındaki ve günlüklerdeki sonuç yolları (`Output`, `OutputFiles`, `Log`,
  `JSON-Timeline`, `Command` içindeki çıktı yolları) ve Defender'ın `ExtractedFile` değeri çıktı
  dizinine göredir (`./HOST1/ProcessExecution/Prefetch.csv`, yanındaki bir zaman çizelgesi için
  `../siem.jsonl`). `Command`, programın adıyla (`juntei`) başlar; kanıt yolları kanıt dizinine,
  çıktı yolları ise çıktı dizinine göredir.
- `EvidenceDirectory` ve `OutputDirectory`, iki dizinin adlarıdır. `ManifestVersion` 2, sonuç
  yollarını çıktı dizinine göre kaydeden bildirim dosyalarını işaretler; önceki bildirim
  dosyalarında `ManifestVersion` yoktur ve yolları `-o` seçeneğine verildiği şekilde kaydeder. Bir
  `-C` yeniden çalıştırması her ikisini de okur.
- Terminal, bir çıktı veya zaman çizelgesi yolunu göreli olduğunda verildiği gibi, mutlak olduğunda
  ise geçerli dizine göre (`../results/`) gösterir.

Paralel işlem sayısından sonra terminal, bulunan Windows sistemlerinin dosyalarının toplam boyutunu
gösterir (`Total filesize: 26.1 MiB`).

İlerleme iletisi, etkin bilgisayarı ve artefaktı ve ardından yaklaşık bir girdi boyutunu gösterir;
örneğin `HOST1 / prefetch (~2.0 MiB)`. Bir dosya grubu toplam boyutunu kullanır; her MFT, LogFile veya
USN Journal görevi kendi girdi boyutunu gösterir. Tahminler, yardımcı işlem günlüklerini ve
zenginleştirme dosyalarını içermez ve ayrıştırılmış çıktının boyutunu öngörmez.

`manifest.csv`, aynı görev dizinini, ayrıştırıcı görevi başına bir satır içeren, elektronik tabloya
uygun bir tablo olarak sunar. `Comp`, özgün `Hostname`, `FQDN`, `HostnameSource`, `EvidenceRoot`,
`Artifact`, `Category`, `OutputType`, `Purpose`, `Status`, `DurationMs`, `ExitCode`, `Output`, `OutputFiles`, `Log`,
`Error` ve `Command` alanlarını içerir. `Purpose`, kategorilere ayrılmış sonuçlar için `artifact`, CSV
ile birlikte kullanılan ek yerel JSONL geçişi için ise `json_timeline` değeridir. `OutputType`,
ayrıştırıcının gerçek çıktı biçimidir. `Command`, boşlukları ve tırnaklamayı koruyan, yürütülebilir
dosya ve bağımsız değişkenlerinden oluşan bir JSON dizisidir. SRUM için `Output` bilgisayar dizinidir,
`OutputFiles` sağlayıcı çıktılarının tam yollarını içeren bir JSON dizisidir ve `Category` ortak
günlük konumunu (`SystemInformation`) belirtir. Dosyalar tek tek kategorilere ayrılır; `OutputFiles`
JSON bildirim dosyasında da bulunur.

İnceleme gerektiren görevleri bulmak için `Status` alanını `skipped` veya `failed` olarak
filtreleyin; atlanan bir görevin `Error` değeri boş olabilir ve açıklama `Log` dosyasında yer alır.

Terminal özeti, her iki bildirim dosyası, ayrıştırıcı günlükleri ve çıkarılan dosyalar dahil olmak
üzere, çıktı dizininin altında şu anda bulunan normal dosyaların toplam sayısını ve boyutunu içerir.
Sembolik bağlantılar izlenmez. Boyutlar, `Total filesize` ile aynı biçimlendirici kullanılarak
`KiB`, `MiB` ve `GiB` gibi birimlerle otomatik olarak ölçeklenir:

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

Birden çok kaynak veritabanı toplandığında Amcache ve SRUM tabloları `-Source1`, `-Source2` vb.
kullanır. Ortak ayrıştırıcı günlüğü özgün yollardan nihai yollara eşlemeyi kaydeder ve JSON/JSONL
kayıtları kaynak kökenini korur. CSV Amcache/Jump Lists görevleri, tam tablo yollarını bildirim
dosyasının `OutputFiles` alanında listeler; `Output` değerleri inceleme kategorisi dizinidir. JSON ve
JSONL Amcache ve Jump Lists dışa aktarımları zaten `Registry-Amcache.json[l]` ve
`JumpLists.json[l]` adlı tek dosyalardır. Birleştirilmiş `-j` zaman çizelgesi aynı kayıt içeriklerini
ve `Artifact` değerlerini korur. `--clobber`, önceki adlar dahil kaydedilmiş çıktıları temizlemek için
bir önceki bildirim dosyasını kullanır ve ilgisiz kategori dosyalarını korur. Normal tek birimli bir
koleksiyon, `$MFT` ve USN Journal kayıtlarını doğrudan
`FileSystem/MFT.csv` ve `FileSystem/UsnJournal.csv` dosyalarına yazar. Toplanan ek birimler,
`FileSystem/MFT-2.csv` ve `FileSystem/MFT-2-logs.txt` gibi numaralandırılmış kardeş dosyalar kullanır.

## JSON ve SIEM içe aktarımı { #json-and-siem-ingestion }

Her JSON veya JSONL kaydı iki üst düzey alan alır:

- `Comp`: kaynak sistem.
- `Artifact`: kaydı üreten Juntei ayrıştırıcısı; örneğin `mft` veya `prefetch-timeline`.

`-j, --json-timeline <FILE>`, tek tek sonuçların biçiminden bağımsız olarak ek bir JSONL dosyası
oluşturur. Kategorilere ayrılmış tüm çıktılar ve günlükleri korunur:

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

Dosya adı verildiği gibi kullanılır; göreli yollar geçerli çalışma dizinine göredir. Dosya çıktı
kökünde, ayrılmamış başka bir çıktı alt dizininde veya çıktı dizininin dışında olabilir. Kanıtlar,
bildirim dosyaları, bilgisayar başına sonuçlar/günlükler ve geçici ayrıştırıcı dizinleri hedef olarak
kullanılamaz. Var olan bir dosya `-C` gerektirir; başarısız bir dışa aktarma onu olduğu gibi bırakır.

**CSV modunda** Juntei ek bir yerel JSONL geçişi gerçekleştirir; böylece SIEM dosyası sayısal ve boole
türlerini, iç içe verileri, ham alanları ve CSV tablolarının atlayabileceği meta verileri korur. Ek
işler aynı `-T` eşzamanlılık sınırını paylaşır, karantina içeriklerini yeniden çıkarmaz ve ayrı
`*-json-timeline-logs.txt` yan günlüklerine sahiptir. Her iki bildirim dosyası da amaçlarını ve çıktı
biçimlerini ayırt eder. Geçici JSONL dosyaları başarılı bir çalışmadan sonra kaldırılır; bir
ayrıştırıcı veya dışa aktarma başarısız olursa korunur. Ek işlem süresini ve geçici disk alanını
hesaba katın.

**JSON modunda** Juntei, kanıtı yeniden ayrıştırmadan mevcut sonuçları sıkıştırılmış JSONL'ye akış
olarak aktarır. Amcache raporları, yerel JSONL ile aynı tek tek özet, bulgu, dosya ve aygıt kayıtlarına
genişletilir. **JSONL modunda** mevcut açıklamalı kayıtlar doğrudan birleştirilir. Tüm modlar aynı SIEM
kayıt yapısını kullanır. Veri kümesi yerel zaman damgalarını ve artefakt alanlarını korur; ortak bir olay
zaman damgası uydurmaz veya farklı türdeki artefaktları genel olarak sıralamaz.

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json`, istenen `JSON-Timeline` yolunu ve varsa `JSON-TimelineError` değerini kaydeder. Görev
kayıtları ve `manifest.csv` yine korunan artefakt çıktılarını gösterir. Başarılı, yalnızca zaman
çizelgesi amaçlı CSV işleri, silinmiş geçici dosyalar yerine nihai JSONL dosyasına başvurur. Herhangi
bir ayrıştırıcı başarısız olursa çalışma başarısızlıkla sonlanır ve zaman çizelgesi yalnızca başarılı
sonuçları içerir; onu eksiksiz kabul etmeden önce bildirim dosyalarını ve kategori günlüklerini
inceleyin. Dosya sayısı/boyutu özeti çıktı dizini içindeki dosyaları kapsar; harici bir zaman
çizelgesi ayrıca `JSONL timeline: <FILE>` olarak raporlanır.

## Performans { #performance }

Ayrıştırıcı işleri ayrı işlemlerdir; bu nedenle CPU yoğun artefaktlar aynı anda çalışabilir ve bir
ayrıştırıcıdaki hasar tüm çalışmayı sonlandıramaz. Varsayılan olarak Juntei mantıksal CPU sayısını
kullanır. Kanıt yavaş bir diskte olduğunda veya sistemin belleği az olduğunda eşzamanlılığı sınırlayın:

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## Kullanım { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse VHDX images natively on Linux, macOS and Windows (read-only; temporary disk space required)
      --e01                   Parse E01/Ex01 images natively (split images from their first segment; temporary disk space required)
      --temp-dir <DIR>        Directory for private image staging (default: output filesystem)

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files
  -R, --reconstruct-filesystem        Reconstruct filesystem trees from MFT and I30

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Mümkün olduğunda yeni bir çıktı dizini kullanın. `-C`, yalnızca bu çalışma için üretilen tam hedefleri
kaldırır veya üzerine yazar; böylece artık bulunmayan bir artefakt güncelliğini yitirmiş bir sonuç
bırakamaz. İlgisiz dosyaları veya çıktı ağacının tamamını silmez. Ayrıca ayrıştırmadan önce, yeni
çalışma `-x` kullansın ya da kullanmasın, her bilgisayarın
`SecurityProducts/DefenderQuarantined/` dizinini ve önceki düz düzenden kalan ayrılmış
`defender-quarantined/` dizinini kaldırır.

Eski düz bir düzenden geçerken yeni bir çıktı dizini kullanın. Önceki CSV/JSON dosyaları otomatik olarak
taşınmaz veya silinmez.

!!! danger "Çıkarılan Defender içeriğini kötü amaçlı yazılım olarak ele alın"
    `-x, --extract-defender-quarantined`, karantinaya alınmış içeriği her bilgisayarın
    `SecurityProducts/DefenderQuarantined/` dizini altında diske yazar. Kurtarılan ikili akışlar
    `.quarantined` ile biter ve Juntei yürütme iznini kaldırır. Her ikili güvenlik tanımlayıcısı için
    ayrıca sahiplik, izinler ve SDDL içeren okunabilir bir `.security_descriptor.txt` raporu
    oluşturulur. Kurtarılan yükleri asla çalıştırmayın. Yalıtılmış bir analiz sistemi kullanın ve
    virüsten koruma veya EDR yazılımının kurtarılan içerik için uyarı vermesini veya onu kaldırmasını
    bekleyin.

Rapor alanları ve adli bilişim açısından sınırlamaları için
[kurtarılan güvenlik tanımlayıcısını okuma](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
bölümüne bakın.

Auto-parse, kullanıcıların uzak bağlantılarını da (RDP, PuTTY, WinSCP ve eşlenmiş sürücüler) okur.

## Kronolojik birleşik zaman çizelgesi oluşturma { #build-a-chronological-super-timeline }

Ayrıştırmadan sonra [`super-timeline`](super-timeline.md) ile kayıtlı zamanları etiketli olaylara dönüştürün. Bilgisayarlar ve artefaktlar arasında sıralamak için `--sort` ekleyin:

```bash
juntei super-timeline -d output/ -o SuperTimeline.csv --sort
```

`-j, --json-timeline` dışa aktarımı, şemaları koruyan yerel kayıtların birleşik JSONL dosyasıdır; zamanları genişletmez veya sıralamaz. Kanıt incelemesi için bunu ve ayrı ayrıştırıcı çıktılarını saklayın.
