# Windows Search Dizini { #windows-search-index }

## Dizinin kaydettikleri { #what-the-index-records }

Windows Search (`WSearch` hizmeti) kullanıcı profillerini, Başlat menüsünü, Outlook'un posta kutularını,
Internet Explorer'ın geçmişini, çevrimdışı dosyaları ve kapsamındaki diğer her şeyi tarar ve okuduklarını
bir ESE veritabanı olan `Windows.edb` içinde tutar. Bunun iki bölümü bir inceleme için önemlidir:

- dizine eklenen her öğe için bir satır içeren **özellik deposu** (`SystemIndex_PropertyStore`;
  Windows 7'de `SystemIndex_0A`): öğenin yolu, adı, türü ve boyutu, sahibi, dizin oluşturucunun dosyadan
  kopyaladığı oluşturma, değiştirme ve erişim zamanları, dizin oluşturucunun onu ne zaman okuduğu, bir
  belgenin başlığı ve yazarı, metninin başlangıcı, bilgisayar adı ve yüzlerce başka özellik;
- tarayıcının (crawler) her öğeye ilişkin kendi kaydını içeren **toplayıcı** tabloları
  (`SystemIndex_Gthr`, `SystemIndex_GthrPth`): öğenin URL yolu, kuyruğa alındığı andaki değiştirme
  zamanı, tarama numarası ve bayrakları ile bir silme sayacı.

Bir öğe; dosya, ileti veya sayfa ortadan kalktıktan sonra da tarayıcı silinmeyi fark edene kadar dizinde
kalır. Bu nedenle dizin, artık var olmayan dosyaları, dizine eklendikleri andaki meta verileri ve
metinlerinden bir alıntıyla birlikte adlandırır.

<span class="evidence-strength">En iyi sonuç:</span> dizin oluşturucu, öğeyi kaydedilen yolda,
kaydedilen meta verilerle, `GatherTime` zamanında, adı geçen bilgisayarda okumuştur. Dizin, dosyanın var
olduğunu ve neler içerdiğini gösterir. Bir kullanıcının onu açtığını göstermez.

## Konum ve toplama { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Windows 7'den Windows 11 21H2'ye kadar olan sürümler ve aynı yılların Windows Server sürümleri dizini
`Windows.edb` içinde tutar. Windows 11 22H2 ve sonrası dizini aynı klasördeki SQLite veritabanlarında
(`Windows.db`, `Windows-gather.db`, `Windows-usn.db`) tutar; Juntei bunları bulur ve adlarını verir,
desteklenmeyen daha yeni biçim olarak raporlar.

`Applications\Windows` klasörünün tamamını bir disk imajından, bir birim gölge kopyasından veya
`WSearch` hizmetini durdurduktan sonra toplayın: hizmet çalışırken veritabanı açıktır ve en yeni
değişiklikleri yalnızca Juntei'nin yeniden oynatmadığı `MSS*.log` işlem günlüklerinde bulunur. Çalışan
bir sistemden kopyalanan dizin kirli kalır; çıktı bunu belirtir ve diskte olanı okur. Klasör girdisi ve
`auto-parse`, herhangi bir derinlikteki her `Windows.edb` dosyasını okur; tek bir dosya için `-f`
kullanın.

## Analist için uyarılar { #analyst-cautions }

- Değişmez bir delil kopyasını ayrıştırın. Juntei verimli erişim için dosyayı belleğe eşler; okuma sırasında veritabanını değiştirmeyin veya kısaltmayın.

- `GatherTime`, dizin oluşturucunun öğeyi okuduğu zamandır; bir kullanıcının onu oluşturduğu,
  değiştirdiği veya açtığı zaman değildir. Dizin oluşturma arka planda, çoğu zaman bir değişiklikten
  dakikalar ila saatler sonra çalışır ve bilgisayar meşgulken veya pille çalışırken duraklar.
- `DateCreated`, `DateModified` ve `DateAccessed`, dizin oluşturucunun dosyadan aldığı kopyalardır.
  Öğe yeniden dizine eklendiğinde yenilenirler; bu nedenle dosyayı `GatherTime` anındaki hâliyle
  tanımlarlar.
- `ItemPath` görüntülenen yoldur. `ItemURL` ve `GatherPath` tarayıcının URL'leridir: dosyalar için
  `file:` (Windows 8 ve sonrasında birim `[GUID]` olarak yazılır), Internet Explorer'ın geçmişi için
  `iehistory://{SID}/`, Outlook öğeleri için `mapi://`, çevrimdışı dosyalar için `csc://{SID}/` ve
  Windows uygulamalarının içeriği için `winrt://{SID}/`.
- `User`, öğenin yolunda adı geçen profildir (`C:\Users\<name>\…`). Öğenin nerede olduğunu gösterir,
  kimin oluşturduğunu değil; `FileOwner`, dizin oluşturucunun gördüğü hâliyle dosyanın sahibidir.
- `AutoSummary`, öğenin metninin başlangıcıdır. O zamandan beri silinmiş veya şifrelenmiş bir belgeden
  alıntı yapabilir; belgenin kendisi değil, kısaltılmış bir alıntıdır.
- Kirli bir veritabanı, dal sayfalarının henüz ulaşmadığı sayfalar içerebilir. Juntei yaprak sayfaların
  kendi zincirini izler ve bu yolla kaç sayfa okuduğunu özette belirtir. Veritabanının en yeni
  değişiklikleri yine de işlem günlüklerindedir.
- Toplayıcı tablolarında bulunup özellik deposunda satırı olmayan bir öğe (`InTables` değeri `Gthr`)
  kuyruğa alınmış veya taranmıştır ve özellikleri henüz yazılmamış ya da kaldırılmıştır. Yolu, adı ve
  toplayıcı zamanları yine de oradadır.
- Windows 7, dizeleri kendine özgü sıkıştırılmış ve karartılmış bir kodlamayla saklar. Juntei bunun
  kodunu çözer; çözülemeyen bir değer onaltılık olarak kalır ve `Problems` içinde bir not bulunur.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

Varsayılan dosya biçimi CSV'dir. `-t csv`, `-t json` veya `-t jsonl` verilip `-o` verilmezse kayıtlar
stdout'a yazılır. İki seçenek de verilmezse yalnızca özet gösterilir. Var olan çıktının üzerine yazmak
için `-C` ekleyin. Çıktı, `-C` ile bile hiçbir zaman girdi dosyalarından birinin üzerine yazılmaz.

Juntei, özellik deposu ile toplayıcı tablolarını öğenin tanımlayıcısı üzerinden birleştirerek dizine
eklenen her öğe için bir kayıt yazar. Özet; veritabanlarını şemaya göre, kirli olanları, her iki tabloda,
yalnızca toplayıcıda ve yalnızca özellik deposunda bulunan öğeleri, klasörleri, silinmiş olarak
işaretlenen öğeleri, depo başına öğeleri, saklanmış zamanı olmayan öğeleri ve sorunları sayar.

Hasar çalışmayı durdurmaz. Kısa kesilmiş bir veritabanı, kalan sayfalarının tuttuğu öğeleri korur ve
özet neyin okunamadığını belirtir. `Windows.edb` adını taşıyan ama Windows Search dizini olmayan bir
dosya raporlanır ve atlanır. Diğer veritabanları yine de okunur.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `DocumentID` | Öğenin dizindeki tanımlayıcısı: özellik deposunun `WorkID` değeri (Windows 7'de `DocID`) ve toplayıcı satırının `DocumentID` değeri. |
| `InTables` | `Gthr+PropertyStore`, `Gthr` (özellikleri olmayan, kuyruğa alınmış veya taranmış bir öğe) veya `PropertyStore`. |
| `ItemPath` | `System_ItemPathDisplay`: Windows'un gösterdiği hâliyle yol. Özellik deposunda satır yoksa boştur. |
| `ItemURL` | `System_ItemUrl`: tarayıcının öğe için kullandığı URL (`file:`, `iehistory://`, `mapi://`, `csc://`, `winrt://`). |
| `GatherPath` | Toplayıcının yolu; kapsam ağacı ile satırın dosya adından birleştirilir. |
| `Name` | `System_ItemName` veya toplayıcı satırının dosya adı. |
| `ItemType`, `ItemTypeText` | `System_ItemType` (uzantı veya `Directory`) ve görünen adı. |
| `Kind` | `System_Kind`: `document`, `folder`, `link`, `program`, `email`, `picture`, …; birden fazlası `\|` ile birleştirilir. |
| `MimeType` | `System_MIMEType`. |
| `Store` | `System_Search_Store`: `file`, `mapi` (Outlook), `iehistory`, `csc` (çevrimdışı dosyalar) veya `winrt` (Windows uygulamaları). |
| `IsFolder`, `IsDeleted` | Saklandığı hâliyle `System_IsFolder` ve `System_IsDeleted`. |
| `DeletedCount` | Toplayıcı satırının silme sayacı. |
| `Size` | Bayt cinsinden `System_Size`. Bir yer tutucu barındıran klasörler için boştur. |
| `FileAttributes` | Sayı olarak `System_FileAttributes` (16 dizin, 32 arşiv, 2 gizli, …). |
| `FileOwner` | Dizin oluşturucunun okuduğu hâliyle `System_FileOwner` (`DOMAIN\user`). |
| `DateCreated`, `DateModified`, `DateAccessed` | Dizin oluşturucunun kopyaladığı hâliyle dosyanın zamanları (UTC). |
| `ItemDate` | `System_ItemDate`: öğenin kendi tarihi; örneğin bir iletinin gönderilme zamanı veya bir fotoğrafın çekilme tarihi; bir dosya için genellikle saniye hassasiyetinde değiştirme zamanı. |
| `GatherTime` | `System_Search_GatherTime`: dizin oluşturucunun öğeyi okuduğu zaman (UTC). |
| `GatherLastModified` | Toplayıcı satırının `LastModified` değeri: tarayıcı öğeyi kuyruğa aldığında öğenin değiştirme zamanı (UTC). |
| `Title`, `Author` | `System_Title` ve `System_Author`. |
| `AutoSummary` | `System_Search_AutoSummary`: öğenin metninin başlangıcı. |
| `CompName` | `System_ComputerName`: dizinin ait olduğu bilgisayar. |
| `TargetURL`, `DateVisited` | Bir Internet Explorer geçmiş öğesinin URL'si ve ziyaret zamanı (`System_Link_TargetUrl`, `System_Link_DateVisited`). |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | Bir Windows 10 etkinlik geçmişi öğesi (`System_Activity_*`, `System_ActivityHistory_*`). |
| `User`, `UserSource` | Öğenin yolunda adı geçen profil ve hangi yoldan okunduğu (`ItemPath`, `ItemURL` veya `GatherPath`). Profil dışında boştur. |
| `ScopeID`, `SDID` | Toplayıcı satırının kapsam ve güvenlik tanımlayıcısı kimlikleri. |
| `CrawlNumber` | Toplayıcı satırının `CrawlNumberCrawled` değeri. |
| `GatherFlags` | Toplayıcı satırının `TransactionFlags` değeri. |
| `Priority` | Hâlâ kuyrukta olan bir öğenin tarama önceliği. ESE, öğe tarandığında bunu NULL olarak işaretler; bu nedenle çoğu öğe için boştur. |
| `PropertyCount` | Özellik deposunun öğe için tuttuğu özellik sayısı. |
| `OtherProperties` | Diğer tüm özellikler, `; ` ile ayrılmış `Name=Value` biçiminde. |
| `SourceFile` | Okunan veritabanı; girdi dizinine göre göreli yolu veya `-f` ile yalnızca adı. |
| `Problems` | Kodu çözülemeyen değerler, zaman olmayan zamanlar, yinelenen satırlar ve hasar. Gizlemeyin. |

JSON ve JSONL aynı alanları türlü değerlerle tutar: sayılar sayı olarak kalır ve eksik değerler `null`
olur. `OtherProperties` yerine `Properties` alanını içerirler; bu alan öğenin her özelliğini adı, sayısal
tanımlayıcısı, sütun türü ve değeriyle verir ve bir zaman için okunduğu zamanın yanında ham FILETIME tik
sayısını da içerir. `Gather`, toplayıcı satırının sütunlarını tutar.

## Zaman çizelgesi davranışı { #timeline-behavior }

`auto-parse`, kayıtları `FileExistence/WindowsSearch.csv` (veya `.json`/`.jsonl`) olarak kaydeder ve
birleştirilmiş `-j` zaman çizelgesine ekler. `super-timeline`; `DateCreated`, `DateModified` ve
`DateAccessed` alanlarından (Windows Search'ün kaydettiği hâliyle dosyanın zamanları), `GatherTime`
alanından (öğenin dizine eklendiği zaman), bir Internet Explorer geçmiş öğesinin `DateVisited`
alanından ve bir etkinliğin `ActivityStartTime` ile `ActivityEndTime` alanlarından olaylar oluşturur.
`GatherLastModified`, yalnızca özellik deposunda değiştirme zamanı bulunmayan bir öğe için olaya
dönüşür. Bunların hiçbirine sahip olmayan bir öğe ayrıştırılmış çıktıda kalır ve zamanı olmayan kayıt
olarak sayılır.

## İnceleme stratejisi { #investigation-strategy }

1. `Store` alanını `file` değerine göre süzün ve vakanın dosyalarını, adlarını ve sözcüklerini `ItemPath`,
   `Name` ve `AutoSummary` içinde arayın. Dizin, o zamandan beri silinmiş dosyaları adlandırır.
2. `GatherTime` değerini dosyanın kaydedilen meta verilerle var olduğu en son zaman, `DateModified`
   değerini ise o andaki değiştirme zamanı olarak okuyun.
3. Bir dosyayı bir hesapla ilişkilendirmek için `FileOwner` ve `User` alanlarını kullanın, ardından
   `$MFT`, USN Journal ve `$Secure:$SDS` ile doğrulayın.
4. E-posta (`Store` değeri `mapi`) ve Internet Explorer geçmişi (`iehistory`) için tam kayıtlara
   ulaşmak üzere Outlook'un dosyalarına ve WebCache'e pivot yapın.
5. Dizini ana bilgisayarlar arasında karşılaştırın: `CompName` dizinin ait olduğu bilgisayarı
   adlandırır ve bir imajdan toplanan dizin o imajdaki dosyaları adlandırır.

## Test edilen sürümler { #tested-versions }

Testler, sonuçlarını başka araçların bağımsız olarak okuduğu herkese açık dizinler kullanır:

- plaso'nun Windows 7 dizini (`SystemIndex_0A`); Windows 7 dize kodlamaları dâhil olmak üzere libyal'ın
  `esedbexport` aracıyla karşılaştırılmıştır.
- DFIRArtifactMuseum'un kirli bırakılmış Windows 10 dizini; Stroz Friedberg'in SIDR aracıyla
  karşılaştırılmıştır. Ayrıca Windows 11 dizini (ESE biçim revizyonu 200); `esedbexport` ile
  karşılaştırılmıştır.
- DFIRArtifactMuseum'un Windows 11 22H2 SQLite dizini; desteklenmeyen daha yeni biçim olarak
  raporlanmıştır.
- Kısa kesilmiş, boş veya başka türden veritabanları.

## Daha fazla ayrıntı { #further-details }

- [Microsoft — Windows Search'e genel bakış](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE veritabanı dosya biçimi](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb — Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg — SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft — Extensible Storage Engine kaynak kodu](https://github.com/microsoft/Extensible-Storage-Engine)
