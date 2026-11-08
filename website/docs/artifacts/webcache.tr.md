# Internet Explorer ve Eski Edge { #internet-explorer-and-legacy-edge }

## WebCache veritabanı nedir { #what-the-webcache-database-is }

`WebCacheV01.dat`, her kullanıcının profilinde bulunan Windows İnternet işlevlerinin (WinINet)
veritabanıdır; bir ESE veritabanıdır. Internet Explorer 10 ve 11 geçmişini, indirmelerini, önbelleğini
ve çerezlerini bu veritabanında tutar; eski Edge, yani Edge Chromium tabanlı olmadan önceki Windows 10
Edge'i de öyle. Her kayıt türü bir kapsayıcıdır (container), yani kendine ait bir tablodur:

- **Geçmiş** (`History` ve bir gün ya da bir hafta için `MSHist01…`): ziyaret edilen URL'ler; her biri
  kaydedildiği hesap, en son ne zaman ziyaret edildiği ve kaç kez ziyaret edildiğiyle birlikte. Dosya
  Gezgini de açtığı yerel dosyaları buraya `file:///` URL'leri olarak kaydeder.
- **İndirmeler** (`iedownload`): bir dosyanın geldiği URL, dosyanın kaydedildiği ve indirme sırasında
  yazıldığı yer, boyutu, indirmenin başladığı zaman, yönlendiren sayfa ve MIME türü.
- **Çerezler**: Windows 10 ve sonrasında çerezlerin kendileri (`CookieEntryEx` tabloları), ana
  bilgisayarları, adları, değerleri ve zamanlarıyla birlikte; daha önceki sürümlerde ise onları tutan
  çerez dosyaları.
- **Önbellek** (`Content`): sayfaları göstermek için indirilen dosyalar; her biri URL'si, kopyanın
  diskteki yeri ve HTTP yanıt üst bilgileriyle birlikte. WinINet kullanan diğer programlar da
  indirdiklerini burada önbelleğe alır.
- Web depolama (`DOMStore`), uyumluluk listeleri (`iecompat`) ve diğerleri.

Veritabanı tarayıcılardan uzun yaşar: Internet Explorer kullanımdan kaldırılmıştır ve Chromium tabanlı
Edge kendi SQLite veritabanlarını tutar (bkz. [Web Tarayıcıları](browser.md)), ancak veritabanı hâlâ her
profilde, daha önce kaydedilenlerle ve Dosya Gezgini ile diğer programların hâlâ kaydettikleriyle
birlikte bulunur.

<span class="evidence-strength">En iyi sonuç:</span> kullanıcının profili, kaydedilen zamanda URL'yi,
indirmeyi veya önbelleğe alınan dosyayı kaydetmiştir; veritabanı klavyenin başında kimin oturduğunu ya
da önbelleğe alınan bir dosyanın açıldığını söylemez.

## Konum ve toplama { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

Kapsayıcıların adını verdiği dosyalar (önbelleğe alınan dosyalar, çerez dosyaları) kendi
klasörlerindedir: `%LOCALAPPDATA%\Microsoft\Windows\INetCache\`, `INetCookies\`, `History\` ve
uygulamalar için `%LOCALAPPDATA%\Packages\` altında.

`WebCache` klasörünün tamamını toplayın. Windows, kullanıcı oturum açtığı sürece veritabanını açık
tutar; bu nedenle çalışan bir sistemden alınan kopya genellikle kirlidir: en yeni değişiklikler yalnızca
`V01*.log` işlem günlüklerinde bulunur.

!!! warning "İşlem günlükleri"
    Juntei ESE işlem günlüklerini yeniden oynatmaz. Düzgün kapatılmamış bir veritabanı özette kirli
    bırakılmış olarak sayılır ve bir sorun olarak raporlanır: hâlâ yalnızca günlüklerinde bulunan
    değişiklikler eksik kalır. Bunları dahil etmek için günlükleri önce klasörün bir kopyasında yeniden
    oynatın: Windows'un `esentutl /r V01 /d` komutunu o kopyanın içinde çalıştırın.

## Analist için uyarılar { #analyst-cautions }

- Zamanlar UTC'dir ve 100 nanosaniye hassasiyetindedir; tek istisna, bir günün veya bir haftanın geçmiş
  kapsayıcısındaki `ModifiedTime` değeridir: bu değer sistemin yerel saatidir, fark (offset) belirtilmeden
  yazılır ve `-Z` onu olduğu gibi bırakır. Onu, kaynak sistemin o tarihteki saat
  dilimiyle dönüştürün. `AccessedTime` ile arasındaki fark çoğu zaman o saat diliminin farkıdır, ancak her
  zaman değil: ikisi farklı anlar olabilir.
- Bir geçmiş girdisi bir ziyaret değil, bir URL'dir: `AccessedTime` URL'nin son ziyareti, `AccessCount`
  ise kaç ziyaret olduğudur. `History` kapsayıcısı URL başına bir girdi tutar; bir günün veya bir
  haftanın kapsayıcısı o döneme ait girdileri tutar ve adı, dönemin ilk gününü ve son gününden sonraki
  günü verir (`MSHist012026110120261102`: 1 Kasım 2026).
- Geçmişteki `file:///` URL'leri web sayfaları değil, Dosya Gezgini'nden ve diğer programlardan açılan
  yerel dosyalardır.
- `Account`, bir girdinin kaydedildiği hesaptır (`Visited: alice@…`); `User` ise veritabanının yolunun
  içinde bulunduğu profil klasörüdür. Bu ikisi farklı olabilir: bir profil klasörü her zaman hesabının
  adını taşımaz ve başka bir yere kopyalanmış bir veritabanının yolunda profil klasörü bulunmaz.
- `Package`, bir girdinin kime ait olduğunu söyler: eski Edge'e (`microsoft.microsoftedge_8wekyb3d8bbwe`),
  Gelişmiş Korumalı Mod'daki Internet Explorer'a (`windows_ie_ac_001`) veya başka bir uygulamaya. Boşsa
  girdi kullanıcının kendisine aittir: Internet Explorer, Dosya Gezgini ve diğer programlar.
- Microsoft indirme kaydını belgelemez. Juntei, kaydın başlangıç zamanını ve boyutunu yalnızca başlığı
  kaydın içine sığdığında, yollarını ve diğer dizelerini ise yalnızca son ikisi bir URL ve bir Windows
  yolu olduğunda okur. Aksi halde satır, girdinin kendi URL'sini ve başlangıç zamanı okunamadığında
  girdinin erişim zamanını bir sorunla birlikte içerir. `Publisher`, tarayıcının kaydettiği addır;
  dosyanın imzasının denetlendiği anlamına gelmez.
- Bir önbellek girdisi, dosyanın indirildiğini ve önbelleğe alındığını söyler. Diskteki kopya silinmiş,
  değiştirilmiş veya hiç açılmamış olabilir.
- Silinmiş girdiler henüz kurtarılmamaktadır: ESE silinmiş bir kaydı, sayfalarını temizleyene kadar bu
  sayfalarda tutabilir, ancak Juntei yalnızca canlı kayıtları okur.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d`, `WebCacheV01.dat` dosyasını adına göre arar; `-f`, adı ne olursa olsun bir veritabanını okur.

## Çıktı tabloları { #output-tables }

CSV beş dosya oluşturur; satırı olmayan dosya yalnızca başlık satırını içerir. JSON ve JSONL satırları,
`SourceFile` alanından sonra `Table` alanını (`History`, `Downloads`, `Cookies`, `Entries` veya
`Containers`) taşır.

| CSV dosyası | İçerik |
|---|---|
| `WebCache-History.csv` | Geçmiş kapsayıcılarının girdileri. |
| `WebCache-Downloads.csv` | İndirmeler. |
| `WebCache-Cookies.csv` | Çerezler (Windows 10 ve sonrası). |
| `WebCache-Entries.csv` | Diğer tüm kapsayıcıların girdileri: önbelleğe alınan dosyalar, çerez dosyaları, web depolama, uyumluluk listeleri ve geri kalanlar. |
| `WebCache-Containers.csv` | Kapsayıcılar. |

## Alanlar { #fields }

Her tablo şu alanlarla başlar:

| Alan | Yorumlama |
|---|---|
| `SourceFile` | Satırın okunduğu veritabanı. |
| `User` | Yolun içinde bulunduğu profilin sahibi olan Windows kullanıcısı (`Users` sonrasındaki klasör). |
| `Package` | Satırdaki verilerin ait olduğu uygulama: kapsayıcısının (veya çerez tablosunun) dizininde `Packages` altındaki klasör. Kullanıcının kendisine ait olanlarda boştur. |

### Geçmiş { #history }

| Alan | Yorumlama |
|---|---|
| `Container` | Kapsayıcı: `History` ya da ardından döneminin ilk günü ve son gününden sonraki gün gelen `MSHist01`. |
| `AccessedTime` | URL'nin son ziyareti (UTC); bir günün veya bir haftanın kapsayıcısında o dönemin son ziyareti. |
| `ModifiedTime` | Girdinin en son değiştirildiği zaman (UTC); bir günün veya bir haftanın kapsayıcısında, fark belirtilmeden sistemin yerel saati. |
| `CreationTime` | Girdinin oluşturulduğu zaman (UTC). |
| `ExpiryTime`, `SyncTime` | Girdinin geçmişten çıkmasının öngörüldüğü zaman ve en son eşitlendiği zaman (UTC). |
| `Account` | Girdinin kaydedildiği hesap. |
| `URL` | `Visited: account@` öneki olmadan sayfa veya dosya. |
| `AccessCount`, `SyncCount` | URL'nin kaç kez ziyaret edildiği ve girdinin kaç kez eşitlendiği. |
| `EntryType`, `Flags` | Saklandığı haliyle girdinin türü ve bayrakları. |
| `EntryID`, `ContainerID` | Girdinin kapsayıcısındaki kimliği ve kapsayıcının kimliği. |

### İndirmeler { #downloads }

| Alan | Yorumlama |
|---|---|
| `StartTime` | İndirmenin başladığı zaman (UTC). |
| `URL` | Dosyanın geldiği yer. |
| `TargetPath`, `CurrentPath` | Dosyanın kaydedildiği yer ve indirme sırasında yazıldığı yer: eski Edge'de bir `.partial` dosyası, Internet Explorer'da önbellekteki kopya. |
| `TotalBytes` | Dosyanın boyutu. |
| `Referrer` | Yönlendiren sayfa. |
| `MimeType` | Sunulduğu haliyle dosyanın türü (Internet Explorer). |
| `Publisher` | Eski Edge'in dosyanın yayımcısı için kaydettiği ad. |
| `AccessedTime` | Girdinin kendi erişim zamanı (UTC); çoğunlukla başlangıç zamanıdır. |
| `DownloadID`, `ContainerID` | Girdinin kapsayıcısındaki kimliği ve kapsayıcının kimliği. |

### Çerezler { #cookies }

| Alan | Yorumlama |
|---|---|
| `LastModified`, `Expires` | Çerezin ayarlandığı ve sona erdiği zaman (UTC). |
| `Host`, `Name`, `Path` | Çerezin ait olduğu site, çerezin adı ve yolu. |
| `Value` | Saklandığı haliyle değer. |
| `Flags` | Saklandığı haliyle çerezin bayrakları. |
| `Directory` | Veritabanının çerez tablosu için verdiği dizin: çerezlerin kime ait olduğu (`…\INetCookies\ESE\`, eski Edge'in `…\MicrosoftEdge\Cookies\ESE\` dizini, bir uygulamanın dizini). |
| `CookieTable`, `EntryID` | Tablo (`CookieEntryEx_<n>`) ve çerezin bu tablodaki kimliği. |

### Girdiler { #entries }

| Alan | Yorumlama |
|---|---|
| `Container` | Kapsayıcı: `Content` (önbelleğe alınan dosyalar), `Cookies` (çerez dosyaları), `DOMStore` (web depolama), `iecompat`, …. |
| `AccessedTime` | Girdinin en son kullanıldığı zaman (UTC). |
| `ModifiedTime` | Kaynağın, sunucunun bildirdiği haliyle son değiştirilme zamanı (UTC). |
| `CreationTime` | Girdinin oluşturulduğu zaman (UTC). |
| `ExpiryTime`, `SyncTime` | Girdinin sona erdiği ve sunucuyla en son denetlendiği zaman (UTC). |
| `URL` | Saklandığı haliyle URL: önbelleğe alınan bir dosya için bir URL, bir çerez dosyası için `Cookie:alice@example.com/`, `DOMStore:…`, `iecompat:…`. |
| `Account` | Bir çerez dosyasının hesabı. |
| `Filename`, `FilePath` | Girdinin dosyası ve bulunduğu yer: kapsayıcının dizini ve girdinin adını verdiği alt klasör. |
| `FileSize` | Dosyanın boyutu. |
| `AccessCount`, `SyncCount` | Girdinin kaç kez kullanıldığı ve sunucuyla kaç kez denetlendiği. |
| `EntryType`, `Flags` | Saklandığı haliyle girdinin türü ve bayrakları. |
| `RedirectURL` | URL'nin yönlendirildiği yer. |
| `ResponseHeaders`, `RequestHeaders` | HTTP yanıt üst bilgileri (`HTTP/1.1 200 OK`, `Content-Type`, `Server`, …) ve istek üst bilgileri; satırlarıyla birlikte, saklandıkları haliyle. |
| `EntryID`, `ContainerID` | Girdinin kapsayıcısındaki kimliği ve kapsayıcının kimliği. |

### Kapsayıcılar { #containers }

| Alan | Yorumlama |
|---|---|
| `ContainerID`, `Name` | Kapsayıcının kimliği ve adı. |
| `Directory` | Dosyalarının diskte bulunduğu yer. |
| `SecureDirectories` | Dosyalarının bulunduğu `Directory` alt klasörleri (CSV'de aralarında virgül bulunur). |
| `PartitionID` | Kullanıcının kendi kapsayıcıları için `M`, Internet Explorer'ın Korumalı Mod'una ait olanlar (`Low` klasörleri) için `L`, bir uygulamanınkiler için uygulama kapsayıcısının SID'si. |
| `LastAccessTime`, `LastScavengeTime` | Kapsayıcının en son kullanıldığı ve en son temizlendiği zaman (UTC). |

## Otomatik ayrıştırma { #automatic-parsing }

`auto-parse`, tabloları `BrowserActivity` içine `WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv` ve `WebCache-Containers.csv` olarak (`-t json` veya
`-t jsonl` ile tek bir `WebCache.json` ya da `WebCache.jsonl` dosyası olarak) ve
`BrowserActivity/WebCache-logs.txt` ile birlikte yazar. Birleştirilmiş `-j` zaman çizelgesi her satırı
`Comp` ve `Artifact` tanımlayıcısı `webcache` ile birlikte içerir. `-Z`, bir günün veya bir haftanın geçmiş
kapsayıcısındaki yerel `ModifiedTime` dışında her zamanı dönüştürür: adı `Time` ile biten alanları,
`LastModified` ve `Expires` değerlerini.

## Daha fazla ayrıntı { #further-details }

- [Microsoft — About WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE database file format](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — MSIE WebCache parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
