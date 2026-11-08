# Web Tarayıcıları { #web-browsers }

## Tarayıcı veritabanları nedir { #what-the-browser-databases-are }

Tarayıcılar geçmişlerini, indirmelerini, çerezlerini ve form girdilerini her kullanıcının profilindeki
SQLite veritabanlarında saklar. Chromium tabanlı tarayıcılar (Chrome, Edge, Brave, Opera, Vivaldi) ve
Firefox tabanlı tarayıcılar kendi dosyalarını ve tablolarını kullanır, ancak aynı türde etkinlikleri
kaydeder:

- **Geçmiş**: ziyaret edilen sayfalar; her ziyaret, zamanı, tarayıcının sayfaya nasıl geldiği (adres
  yazılarak, bir bağlantıyla, bir yönlendirmeyle, bir yer imiyle) ve Chromium'da sayfanın ne kadar süre
  ön planda kaldığıyla birlikte.
- **İndirmeler**: bir dosyanın geldiği URL (ve yol üzerindeki yönlendirmeler), nereye kaydedildiği,
  boyutu, indirmenin ne zaman başlayıp bittiği ve tamamlanıp tamamlanmadığı.
- **Çerezler**: onları ayarlayan siteler; ne zaman ayarlandıkları, en son ne zaman gönderildikleri ve ne
  zaman sona erdikleri.
- **Form geçmişi**: web formlarına ve arama kutularına yazılan değerler; kaç kez, ilk ve son ne zaman
  yazıldıkları.

Tarayıcı etkinliği çoğu zaman bir kullanıcının ne yaptığının en doğrudan kaydıdır: izlediği oltalama
sayfası, indirdiği araç, kullandığı web postası, yazdığı arama terimleri.

<span class="evidence-strength">En iyi sonuç:</span> tarayıcı profili, kaydedilen zamanda URL'ye bir
ziyaret kaydetmiş veya indirmeyi kaydedilen yola kaydetmiştir; veritabanı klavyenin başında kimin
oturduğunu söylemez.

## Konum ve toplama { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>`; `Default`, `Profile 1`, `Profile 2` vb. olur. Chromium, 96 sürümünde `Cookies` dosyasını
`Network` içine taşıdı; eski sürümler onu `History` dosyasının yanında tutar. Firefox 25 sürümüne kadar
indirmelerini `downloads.sqlite` içinde tutuyordu; sonraki sürümler bunları `places.sqlite` içinde tutar.

Profilin her dosyasını toplayın; her veritabanını da yanında bulunan ve adını paylaşan dosyalarla
birlikte toplayın: `places.sqlite-wal`, `cookies.sqlite-wal`, `formhistory.sqlite-wal`, `History-journal`.

!!! warning "Önceden yazma günlükleri"
    Firefox veritabanlarını önceden yazma günlüğü (write-ahead log) kipinde tutar: en yeni değişiklikler,
    tarayıcı onları veritabanına kopyalayana (denetim noktası, checkpoint) kadar yalnızca `-wal`
    dosyasındadır. Juntei her veritabanını, tarayıcının yapacağı gibi `-wal` dosyasıyla birlikte okur.
    Bu dosya olmadan son ziyaretler, çerezler ve form girdileri eksik kalır ve günlükte silinmiş
    girdiler hâlâ canlı olarak okunur.

## Silinmiş girdiler { #deleted-entries }

Bir tarayıcı geçmişi sildiğinde satırlar tablolardan çıkar, ancak kayıtları veritabanının boş alanında,
boş sayfa listesinde (freelist) veya veritabanı sayfalarının eski sürümlerinde kalabilir: `-wal`
dosyasında ya da, yeni sürümleri bir `-wal` dosyasında bulunan sayfalar için, veritabanı dosyasının
kendisinde. Juntei bu kaynaklardan silinmiş ziyaretleri, sayfaları, indirmeleri ve form girdilerini,
`Status` değeri `Deleted` olan satırlar olarak kurtarır:

- `Confidence`, kaydın tablosuyla eşleştirilmesinden ne kadar emin olunduğunu belirtir: her değeri kendi
  sütununun türünde olan bütün bir kayıt için `High`, hasarlı kayıtlar veya birden çok tabloya uyan
  kayıtlar için `Medium` ya da `Low`.
- `RecoveredFrom`, kaydın nerede bulunduğunu belirtir: sayfanın hangi alanı (bir hücre, bir boş blok
  (freeblock), ayrılmamış alan, boş bir sayfa), hangi sayfa ve onun hangi sürümü ve kaydın sayfa içindeki
  ofseti. WAL çerçeveleri 0'dan başlayarak sayılır.
- `LostColumns`, hasarlı bir kaydın artık içermediği sütunları listeler: bu sütunların alanları boştur.

Tarayıcılar SQLite'ın `secure_delete` ayarıyla siler: veritabanında serbest bırakılan alan sıfırlanır. Bu
durumda kurtarılabilecek veriler çoğunlukla eski sayfa sürümlerindedir: `-wal` dosyasının hâlâ tuttuğu
sürümlerde ve bir sonraki denetim noktasına kadar, veritabanı dosyasının, yerini bir `-wal` dosyasındaki
sürümlerin aldığı sayfaların kendi kopyalarında. `-wal` dosyasını veritabanıyla birlikte kopyalayın ve
bunu tarayıcı yeniden çalışmadan önce yapın.

## Analist için uyarılar { #analyst-cautions }

- Zamanlar UTC'dir. Chromium ve Firefox bunları mikrosaniye hassasiyetinde kaydeder; Chromium'un form
  geçmişi ise saniye hassasiyetinde.
- Bir ziyaret, kullanıcının bir şey yazdığı veya tıkladığı anlamına gelmez: yönlendirmeler, çerçeveler ve
  başka programların açtığı sayfalar da ziyarettir. `Transition`, `Typed` ve `Hidden` alanlarını okuyun.
- Chromium çerez değerlerini şifreler: bu çerezler için `Value` boştur. Juntei şifre çözmez.
- `Removed` durumundaki bir form geçmişi satırı, Firefox'un `moz_deleted_formhistory` tablosundan gelir;
  bu tabloyu Android için Firefox, Sync için doldurur. Masaüstü Firefox normalde onu boş bırakır.
- `Low` güven düzeyindeki silinmiş bir girdi, başka bir kaydın parçası olabilir: onu canlı satırlarla ve
  diğer kanıtlarla karşılaştırın.
- Silinmiş satır bulunmaması hiçbir şeyin silinmediği anlamına gelmez: sıfırlanmış alan ve bir denetim
  noktası, kurtarılacak hiçbir şey bırakmaz. Silinmiş çerezler henüz kurtarılmamaktadır.
- Veritabanı içeriği tarayıcı ailesini belirler. Uyumlu profil yolu `Browser` değerini Chrome veya Firefox olarak özelleştirir; çelişen yol aileyi değiştirmez. Aksi halde etiket `Chromium-based`, `Firefox-based` veya `Safari` olur (belirlenemiyorsa `Unknown`). `User` ve `Profile` yoldan gelir ve dosyalar kopyalanırken kaybolabilir.
- Geri alma günlükleri (`-journal`) okunmaz; Internet Explorer'ın ve eski Edge'in `WebCacheV01.dat`
  dosyası da okunmaz.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d`, dosyaları adlarına göre arar ve her birini tablolarından tanır: başka bir programa ait bir
`History` dosyası atlanır. `-f`, adı ne olursa olsun bir veritabanını okur.

## Çıktı tabloları { #output-tables }

CSV dört dosya oluşturur; satırı olmayan dosya yalnızca başlık satırını içerir. JSON ve JSONL satırları,
`SourceFile` alanından sonra `Table` alanını (`History`, `Downloads`, `Cookies` veya `FormHistory`) taşır.

| CSV dosyası | İçerik |
|---|---|
| `Browser-History.csv` | Canlı ve silinmiş ziyaretler; URL'si hiçbir canlı sayfada bulunmayan silinmiş sayfalar. |
| `Browser-Downloads.csv` | Canlı ve silinmiş indirmeler. |
| `Browser-Cookies.csv` | Çerezler. |
| `Browser-FormHistory.csv` | Canlı ve silinmiş form girdileri; Firefox'un kaldırma kayıtları. |

## Alanlar { #fields }

Her tablo şu alanlarla başlar:

| Alan | Yorumlama |
|---|---|
| `SourceFile` | Satırın okunduğu veritabanı. |
| `User` | Yolun içinde bulunduğu profilin sahibi olan Windows kullanıcısı (`Users` sonrasındaki klasör). |
| `Browser` | İçerikten belirlenen, uyumlu profil yoluyla özelleştirilen tarayıcı ailesi. |
| `Profile` | Tarayıcı profili: veritabanının klasörü. |
| `Status` | `Live`, `Deleted` (kurtarılmış) veya `Removed` (bir Firefox kaldırma kaydı). |

### Geçmiş { #history }

| Alan | Yorumlama |
|---|---|
| `RecordType` | `Visit` ya da ziyaretleri de silinmiş olabilecek silinmiş bir sayfa için `Page`. |
| `VisitTime` | Ziyaretin başladığı zaman (UTC). |
| `URL`, `Title` | Sayfa ve en son görüldüğü haliyle başlığı. |
| `Transition` | Tarayıcının sayfaya nasıl geldiği. Chromium: önce çekirdek tür, ardından niteleyicileri (`LINK`, `TYPED`, `AUTO_BOOKMARK`, `FORM_SUBMIT`, `RELOAD`, … ve bunlarla birlikte `CHAIN_START`, `SERVER_REDIRECT`, `FROM_ADDRESS_BAR`, …); Firefox: `LINK`, `TYPED`, `BOOKMARK`, `EMBED`, `REDIRECT_PERMANENT`, `REDIRECT_TEMPORARY`, `DOWNLOAD`, `FRAMED_LINK`, `RELOAD`. |
| `VisitDuration` | Sayfanın ön planda kaldığı saniye (Chromium; ölçülmediğinde 0). |
| `VisitCount` | Sayfaya yapılan toplam ziyaret sayısı. |
| `Typed`, `TypedCount` | URL'nin hiç yazılıp yazılmadığı ve kaç kez yazıldığı (Chromium). |
| `Hidden` | Geçmiş listesinde gizli: alt çerçeveler, yönlendirme kaynakları, hatalar. |
| `Frecency` | Firefox'un adres çubuğu önerileri için sayfaya verdiği sıralama puanı. |
| `VisitID`, `FromVisitID` | Ziyaretin satır kimliği ve gelinen ziyaretin satır kimliği (yönlendiren sayfa, yönlendirmenin kaynağı). Kurtarılan bir kayıt satır kimliğini kaybetmişse ziyaretin satır kimliği boştur. |
| `LastVisitTime` | Silinmiş bir sayfanın son ziyareti (UTC). |
| `PageSource` | Silinmiş bir ziyaretin URL'sinin ve başlığının nereden geldiği: `Live` (canlı bir sayfa), `Recovered` (silinmiş bir sayfa) veya `NotFound`. |
| `Confidence`, `RecoveredFrom`, `LostColumns` | Silinmiş bir girdinin nasıl kurtarıldığı; bkz. [Silinmiş girdiler](#deleted-entries). |

### İndirmeler { #downloads }

| Alan | Yorumlama |
|---|---|
| `StartTime`, `EndTime` | İndirmenin başladığı ve bittiği zaman (UTC). |
| `URL`, `RedirectChain` | Dosyanın geldiği yer ve yol üzerindeki her URL (Chromium; CSV'de aralarında ` -> ` bulunur). |
| `TargetPath`, `CurrentPath` | Dosyanın kaydedildiği yer (Firefox'ta bir `file://` URI'si) ve indirme sırasında yazıldığı yer. |
| `ReceivedBytes`, `TotalBytes` | Alınan baytlar ve dosyanın boyutu. |
| `State` | `Complete`, `InProgress`, `Cancelled`, `Interrupted`, `Paused`, `Blocked` veya `Dirty` (kötü amaçlı yazılım olarak engellendi). |
| `DangerType` | Chromium'un dosya hakkındaki kararı: 0 tehlikeli değil, 1 tehlikeli dosya türü, 2 tehlikeli URL, 3 tehlikeli içerik, 4 tehlikeli olabilir, 5 yaygın olmayan, 6 kullanıcı tarafından tutuldu, 7 tehlikeli ana bilgisayar, 8 potansiyel olarak istenmeyen, … |
| `InterruptReason` | Kesintiye uğrayan bir indirmenin neden durduğu (Chromium; kesinti yoksa 0). |
| `Referrer`, `TabURL` | Yönlendiren sayfa ve indirmeyi başlatan sekmedeki sayfa. |
| `MimeType`, `Opened` | Sunulduğu haliyle dosyanın türü ve tarayıcıdan açılıp açılmadığı (Chromium). |
| `FileDeletedByUser` | Kullanıcının indirilen dosyayı tarayıcının indirme listesi üzerinden diskten silip silmediği (Firefox). |
| `DownloadID` | İndirmenin satır kimliği. Kurtarılan bir kayıt onu kaybetmişse boştur. |

### Çerezler { #cookies }

| Alan | Yorumlama |
|---|---|
| `Created`, `LastAccessed`, `Expires` | Çerezin ayarlandığı, en son gönderildiği ve sona erdiği zaman (UTC; oturum çerezinde `Expires` boştur). |
| `Host`, `Name`, `Path` | Çerezin ait olduğu site, çerezin adı ve yolu. |
| `Value` | Saklandığı haliyle değer: Chromium şifrelediğinde boştur. |
| `Secure`, `HttpOnly`, `Persistent` | Yalnızca HTTPS üzerinden gönderilir, betiklerden gizlenir, oturumlar arasında korunur (Chromium). |
| `GoogleAnalytics` | Bir Google Analytics çerezinin kodu çözülmüş alanları (`__utma`, `__utmb`, `__utmt`, `__utmz`): ziyaret zamanları, oturumlar, kampanya kaynağı ve arama terimleri. |
| `RowID` | Çerezin satır kimliği. |

### Form geçmişi { #form-history }

| Alan | Yorumlama |
|---|---|
| `FirstUsed`, `LastUsed` | Değerin ilk ve son girildiği zaman (UTC). |
| `RemovedTime` | Bir Firefox kaldırma kaydına göre girdinin kaldırıldığı zaman (UTC). |
| `FieldName`, `Value` | Form alanı (`email`, `q`, Firefox'un arama çubuğu için `searchbar-history`) ve yazılan değer. |
| `Count` | Değerin kaç kez girildiği. |
| `GUID` | Firefox'un girdiye verdiği ve kaldırma kayıtlarının andığı tanımlayıcı. |
| `Sources` | Firefox'un girdiyi nereden kaydettiği; bunu tutan sürümlerde (CSV'de dikey çizgiyle ayrılır). |
| `RowID` | Kaydın satır kimliği; kurtarma sırasında kaybolmuşsa boş/null. Gerçekten kaydedilmiş sıfır `0` olarak kalır. |

## Otomatik ayrıştırma { #automatic-parsing }

`auto-parse`, tabloları `BrowserActivity` içine `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` ve `Browser-FormHistory.csv` olarak (`-t json` veya `-t jsonl` ile tek bir
`Browser.json` ya da `Browser.jsonl` dosyası olarak) ve `BrowserActivity/Browser-logs.txt` ile birlikte
yazar. Birleştirilmiş `-j` zaman çizelgesi her satırı `Comp` ve `Artifact` tanımlayıcısı `browser` ile
birlikte içerir. `-Z`; `VisitTime`, `LastVisitTime`, `StartTime`, `EndTime`, `Created`, `LastAccessed`,
`Expires`, `FirstUsed`, `LastUsed` ve `RemovedTime` değerlerini dönüştürür.

## Daha fazla ayrıntı { #further-details }

- [Chromium — page transition types](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — download danger types](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places database](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — write-ahead logging](https://www.sqlite.org/wal.html)
- [Plaso — browser SQLite parsers](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
