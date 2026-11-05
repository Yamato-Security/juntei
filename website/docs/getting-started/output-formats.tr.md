# Çıktı Biçimleri { #output-formats }

## Terminal özetleri { #terminal-summaries }

`-o` veya yapılandırılmış bir `-t` olmadan komutlar genellikle sayıları, zaman aralıklarını, hasarı ve
yüksek değerli bulguları gösterir. Özetler, büyük çıktılar oluşturmadan önce kapsamı anlamanıza
yardımcı olur.

## CSV { #csv }

CSV, varsayılan kayıt biçimidir. Elektronik tablo araçları, SQL içe aktarımı ve zaman çizelgesi
görüntüleyicileri için kullanışlıdır. Bazı ayrıştırıcılar, artefaktları farklı kayıt türleri içerdiği
için birden çok tablo oluşturur (Amcache, Jump Lists ve SRUM).

!!! warning "Kanıt sütunlarını metin olarak ele alın"
    Bir yol veya komut satırı `=`, `+`, `-` veya `@` ile başlayabilir. Elektronik tablo yazılımı bunu
    bir formül olarak yorumlayabilir. Tüm alanları metin olarak içe aktarın veya adli bilişim açısından
    güvenli bir CSV görüntüleyicisi kullanın. Juntei kanıt dizelerini değiştirmek yerine korur.

## Alan adları { #field-names }

CSV sütun başlıkları ve JSON/JSONL alan adları, Amcache ve `auto-parse` bildirim dosyaları dahil olmak
üzere PascalCase kullanır. Kısaltmalar büyük harf olarak kalır: `ProcessID`, `TimestampUTC`,
`UserSID`, `SHA256`. Bir kısaltmadan sonra gelen sözcük kısa çizgiyle ayrılır: `IP-Address`,
`MFT-Entry`, `USN-Journal`. `jq` ile kısa çizgili bir alanı tırnak içine alın: `.["IP-Address"]`.

Varsayılan MFT CSV'si `EntryNum`, `Ext`, `Dir`, `HasADS`, `Size`, `Created (SI)` ve `Created (FN)`
gibi daha kısa, okunabilir başlıklar kullanır. Tam CSV şeması için
[MFT alan başvurusuna](../artifacts/mft.md) bakın. Zaman damgası çiftleri MACB sırasını izler:
DataModified, Accessed, MetadataChanged, Created; SI, FN'den önce gelir. CSV araçlarında boşluk veya
noktalama işareti içeren başlıkları tırnak içine alın.

Kanıt değerleri özgün yazımlarını korur. `Details`/`DataRaw` altındaki kayıt defteri değer adları ve
`ZoneIdentifier` altındaki `Zone.Identifier` adları gibi kaynaktan gelen sözlük anahtarları da
değişmeden kalır. SRUM'un `ValueState` ve `Raw` sözlükleri, veri sütunlarıyla aynı etiketleri kullanır.
Bir normalleştirme çakışması, bir alanı sessizce atmak yerine raporlanır.

Bu, önceki dışa aktarma şemalarını değiştirir. Mevcut sonuçları yeniden oluşturun ve kayıtlı sorguları
güncelleyin (örneğin `.record_type`, `.RecordType` olur ve Amcache `full_path`, `FullPath` olur).
İnsan tarafından okunabilir metin raporları ve girdi/yapılandırma biçimleri mevcut kurallarını korur.

## Kısaltılmış alan adları { #abbreviated-field-names }

Aşağıdaki etiketler, otomatik ayrıştırma ve bildirim dosyaları dahil olmak üzere CSV, JSON ve JSONL
dışa aktarımlarına uygulanır. Eski sonuçları yeniden oluştururken kayıtlı `jq`, `xsv` ve elektronik
tablo filtrelerini güncelleyin. Yalnızca çıktı etiketleri değişir; kanıt değerleri ve ham kayıt
defteri değer adları özgün yazımlarını korur.

| Önceki etiket | Güncel etiket |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

MFT CSV çıktısı özel sütun adlarını kullanır: `File` (Filename), `SN` (Sequence Number), `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number), `USN` (Update Sequence Number), `LogfileSN` (Logfile Sequence Number), `SID` (Security ID), `Type` (Name Type) ve `RefCnt` (Reference Count). Bunlar `auto-parse` MFT CSV dosyaları için de geçerlidir; bkz. [MFT çıktı alanları](../artifacts/mft.md#csv-output-fields).

Yerel sistem bilgileri JSON zamanı `CompNameLastWritten` alanıdır (CSV:
`CompNameLastWrittenUTC`). `HasADS` bir Boole göstergesidir; Prefetch'teki `Dir` kaydedilmiş dizin
listesini içerir. `User` artefakta özgü türünü korur: bir hesap adı için bir dize veya USB çıktısında
ilişkili kullanıcıların listesi. Yürütmeyi yapan kimliği ima etmez. Sorgularda noktalama işaretlerini
tırnak içine alın: `."SI<FN"` ve `.["ZeroU-Secs"]`.

## JSON { #json }

JSON, sonucun tamamını tek bir belge olarak saklar. Dizileri, iç içe yapıları, null değerleri, büyük
tamsayıları ve düz bir CSV'nin her zaman temsil edemeyeceği ayrımları korur.

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines { #json-lines }

JSONL her satıra bağımsız bir JSON nesnesi yazar. Akış için uygundur ve `jq` ile doğal olarak çalışır:

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

Yapılandırılmış çıktı standart çıktıya gittiğinde Juntei, ilerleme durumunu ve iletileri standart hataya
gönderir; böylece bir boru (pipe) yalnızca veri alır.

`auto-parse -j timeline.jsonl`, kategorilere ayrılmış CSV sonuçlarını korurken çok ana bilgisayarlı
eksiksiz bir koleksiyon için ek bir JSONL veri kümesi oluşturur. Tek tek sonuçların biçimini
değiştirmek için `-t json` veya `-t jsonl` ekleyin. Her kaydın `Comp` ve `Artifact` alanları vardır;
böylece kaynağı SIEM'e aktarıldıktan sonra da açık kalır.

## UTC ve hassasiyet { #utc-and-precision }

Juntei, ayrıştırılan zamanları biçimin izin verdiği yerlerde RFC 3339 kullanarak UTC olarak biçimlendirir.
Bazı artefaktlar 100 nanosaniye hassasiyetine sahiptir; diğerleri saniyeleri, FAT zamanını, toplama
aralıklarını saklar veya hiç zaman saklamaz. Basamak sayısı doğruluk demek değildir: her zaman
artefakt sayfasında açıklanan kaynak alanı anlayın.

UTC, mevcut kesirli hassasiyeti koruyarak Windows olay günlüğü biçimini kullanır:
`2026-10-20T08:46:38.2001668Z`. `auto-parse -Z "+09:00"`, ayrıştırılmış anları tüm sonuç
biçimlerinde ve isteğe bağlı `-j` dışa aktarımında açık, sabit bir ofsetle gösterir. Varsayılan,
Juntei'yi çalıştıran bilgisayardan bağımsız olarak UTC'dir. Ham değerler ve saat dilimi bilgisi
olmayan duvar saati değerleri kaydedildiği gibi kalır;
[çıktı saat dilimleri](../commands/auto-parse.md#output-time-zone) bölümüne bakın.

## Hasar ve eksik değerler { #damage-and-missing-values }

Boş, `null`, `NA`, `Problems` ve ayrıştırıcıya özgü durum alanları anlamlıdır. Artefaktın bir değer
saklamadığı, kaydın hasarlı olduğu, biçimin bunu desteklemediği veya ayrıştırıcının değeri
çözümleyemediği anlamına gelebilirler. Tüm boş değerleri sıfırla değiştirmeyin.
