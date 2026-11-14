# Ana Dosya Tablosu (`$MFT`) { #master-file-table-mft }

## MFT nedir { #what-the-mft-is }

NTFS, her dosyayı ve dizini Ana Dosya Tablosu'ndaki (Master File Table) bir kayıtla temsil eder. Kayıtlar;
adlar, üst dizin referansları, zaman damgaları, boyutlar, bayraklar, güvenlik tanımlayıcıları, yeniden
ayrıştırma (reparse) bilgileri ve veri akışı meta verileri gibi öznitelikler içerir. Küçük dosya içerikleri
ve alternatif veri akışları **yerleşik** (resident) olabilir; yani baytları MFT kaydının kendi içinde saklanır.

MFT temel bir artefakttır, çünkü tüm birimdeki ayrılmış ve silinmiş kayıtları kapsar. Sıradan dizin
gezintisi artık göstermediğinde bile bir dosyanın var olduğunu, nereye bağlandığını, silinip silinmediğini
ve meta verilerinin nasıl değiştiğini gösterebilir.

<span class="evidence-strength">En iyi sonuç:</span> görüntülenen meta verilere sahip bir NTFS dosya
kaydı toplama anında mevcuttu veya ayrılmamış bir kayıtta varlığını sürdürmüştür. MFT tek başına yürütmeyi
veya kullanıcı niyetini kanıtlamaz.

## Konum ve toplama { #location-and-collection }

```text
C:\$MFT
```

`$MFT` korumalı NTFS meta verisidir. Ham NTFS erişimi sağlayabilen bir adli edinim veya triyaj aracıyla
toplayın; sıradan bir kullanıcı modu kopyası normalde onu okuyamaz. Mümkün olduğunda `$J`, `$LogFile` ve
birim meta verileriyle birlikte toplayın.

## Analist için uyarılar { #analyst-cautions }

- Silinmiş bir kaydın üzerine kısmen yazılmış olabilir veya kayıt numarası yeniden kullanılmış olabilir.
- `SN`, aynı girdi numarasının nesillerini birbirinden ayırır. Girdiyi **ve** sıra numarasını
  eşleştirin.
- `$STANDARD_INFORMATION` (`0x10`) ve `$FILE_NAME` (`0x30`) zaman damgalarının güncellenme davranışları
  farklıdır. Hiçbiri manipülasyona karşı bağışık değildir.
- `SI<FN`, `uSecZeros` ve `Copied` ipuçlarıdır; otomatik zaman damgası manipülasyonu (timestomping)
  hükümleri değildir.
- Sabit bağlantılar (hard link), tek bir dosya kaydı için birden fazla `$FILE_NAME` özniteliği ve yol üretir.
- NTFS birden fazla ad (DOS 8.3 dahil), alternatif veri akışları ve uzantı kayıtları saklayabilir.
- Erişim zamanı güncellemeleri devre dışı bırakılmış, geciktirilmiş veya birleştirilmiş olabilir.
- Tam yol, üst kayıtlardan yeniden oluşturulur. `[Orphaned]`, `[Unknown]` veya `PathUnknown`, eksik/yeniden
  kullanılmış/hasarlı soy zincirine işaret eder; gerçek bir dizin adı değildir.
- Bir FILE kaydının **slack** alanı (kullanımdaki boyutun ötesindeki baytları), daha önce tuttuğu adları
  saklayabilir: indeksi bir indeks ayırmasına (index allocation) taşınmış küçük bir dizinin indeks girdileri
  (bunların çoğu orada hâlâ kullanımdadır), daha önceki bir ad veya yeniden kullanılmış bir kaydın önceki
  sahibi. Kurtarılan adların `$STANDARD_INFORMATION` özniteliği yoktur ve bir adın girdi başlığının üzerine
  yazılmış olabilir. Bir kaydın kullanımdaki boyutu veya öznitelikleri slack alanının nerede başladığını
  belirtmediğinde, ondan hiçbir şey kurtarılmaz ve satırının `ParseErrors` alanı `slack not read` ifadesini
  ve nedenini belirtir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams
  -R, --reconstruct-filesystem <DIR>  Recreate a tree; recovered content gets .dontrun
      --reconstruction-i30 <FILE>      Supplement the tree with an I30 file (repeatable)

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

`-e` ile ayıklanan yerleşik akışları asla çalıştırmayın; Juntei onlara boşuna `.dontrun` adları vermez.
`-i` seçeneğinin yerleşik veri sütunları aynı baytlardır: bunları güvenilmeyen içerik olarak ele alın.

`-D` tek bir girdiyi yazdırır: başlığını, her özniteliği (veri dizileriyle birlikte) ve slack alanındaki
adları `-t json -x` çıktısının yazdığı biçimde, ardından kaydın baytlarını. `-L` bir dizindeki kullanımda
olan ve silinmiş her adı; girdi ve sıra numarası, türü, boyutu ve değiştirilme zamanıyla birlikte listeler.
Dizin, girdiyle (`30`, `0x1E`, `30-1`) veya yolla (`\Windows\System32`, `C:/Windows`; büyük/küçük harf
duyarsız) verilir.

Örnek, gezilebilir dosya yollarını yeniden oluşturur. Yerleşik olmayan içerik boş dosya olur. Ayrıntılar: [Dosya sistemi yeniden oluşturma](../commands/auto-parse.md#filesystem-reconstruction).

```bash
juntei parse-ntfs mft -f '$MFT' -R reconstructed
```

## CSV çıktı alanları { #csv-output-fields }

| Alan | Yorumlama |
|---|---|
| `EntryNum` (Entry Number) | MFT kayıt numarası. `SN` ile birleştirin; kayıt numaraları yeniden kullanılır. |
| `SN` (Sequence Number) | NTFS referanslarının kullandığı nesil sayacı. Bir uyuşmazlık, güncelliğini yitirmiş/silinmiş ilişkileri ortaya çıkarabilir. |
| `InUse` | Toplama anındaki ayrılma bayrağı. `false`, kaydın boş/silinmiş olduğu anlamına gelir; tüm baytların sağlam olduğu anlamına gelmez. |
| `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number) | `$FILE_NAME` özniteliğinden gelen üst dizin referansı. Her iki parçayı da doğrulayın. |
| `ParentPath` | Yeniden oluşturulan üst yol. Bilinmeyen/sahipsiz işaretleri, çözümlenemeyen soy zincirini gösterir. |
| `File` (Filename) | Geçerli satırın `$FILE_NAME` özniteliğinden veya akışından gelen ad. Bir kaydın birkaç satırı olabilir. |
| `Ext` (Extension) | Addan ayrıştırılan, kolaylık amaçlı uzantı; güvenilir bir dosya türü tespiti değildir. |
| `Size` (File Size) | Satır/akışla ilişkili mantıksal boyut. Seyrek/sıkıştırılmış ayırma farklı olabilir. |
| `RefCnt` (Reference Count) | NTFS sabit bağlantı referans sayısı; varlığını sürdüren adların sayısıyla dikkatle karşılaştırın. |
| `ReparseTarget` | Bağlantı noktaları (junction)/sembolik bağlantılar gibi desteklenen yeniden ayrıştırma noktaları için kodu çözülmüş hedef. Yollar yönlendirme yapabilir. |
| `Dir` (Directory) | Kayıt dizin bayrağına sahiptir. |
| `HasADS` (Has Alternate Data Streams) | Kayıt adlandırılmış alternatif veri akışları içerir. Bu onu kötü amaçlı yapmaz. |
| `IsADS` (Is Alternate Data Stream) | Bu satır, adsız dosya verisi yerine adlandırılmış bir alternatif veri akışını temsil eder. |
| `SI<FN` (Standard Information Before File Name) | SI oluşturma zamanı, farklı bir FN oluşturma zamanından öncedir. Kopyalamayı/zaman damgası manipülasyonunu ve meşru nedenleri araştırın. |
| `uSecZeros` (Microsecond Zeros) | SI oluşturma/değiştirme zamanı tam olarak bir milisaniye sınırına denk gelir. Yalnızca zayıf bir zaman damgası manipülasyonu buluşsal yöntemidir. |
| `Copied` | Değiştirilme zamanı oluşturulma zamanından öncedir. Çoğunlukla dosya kopyalamayla tutarlıdır; saldırgan eyleminin kanıtı değildir. |
| `SI-Flags` (Standard Information Flags) | `$STANDARD_INFORMATION` dosya öznitelikleri/bayrakları. Gizli/sistem bayrakları faydalı pivot noktaları olabilir. |
| `Type` (Name Type) | `$FILE_NAME` ad alanı (Win32, DOS, POSIX veya birleşik). Adları iki kez saymamak için kullanın. |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | SI zamanları. Yaygın zaman çizelgesi alanlarıdır, ancak kullanıcı modu araçları bunların birçoğunu değiştirebilir. |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | FN zamanları. Farklı biçimde güncellenir ve sıradan API'lerle değiştirilmeleri genellikle daha zordur; yine de değiştirilemez değildir. |
| `USN` (Update Sequence Number) | Kaydın SI özniteliğiyle ilişkili son USN. Bu USN varlığını sürdürüyorsa `$J` içine pivot yapın. |
| `LogfileSN` (Logfile Sequence Number) | FILE kaydı başlığındaki NTFS `$LogFile` sıra numarası; düşük seviyeli işlem analizinde faydalıdır. |
| `SID` (Security ID) | NTFS `$Secure` yapısına bir indeks; kendi başına bir Windows SID'si değildir. Dosyanın sahibi ve ACL'leri için [`parse-ntfs secure-sds`](secure-sds.md) çıktısının `SecurityID` alanıyla birleştirin. |
| `ObjectID-FileDroid` | Mevcutsa nesne kimliği / dağıtılmış bağlantı izleme tanımlayıcısı; LNK/Jump List droid'leriyle ilişkilendirin. |
| `LoggedUtilStream` | Desteklendiği yerde kodu çözülmüş `$LOGGED_UTILITY_STREAM` bağlamı. Alışılmadık değerleri gözden geçirin. |
| `ZoneID-Contents` | ZoneId ve indirme URL'leri gibi yerleşik `Zone.Identifier` içeriği. Bulunmayabilir, kaldırılmış olabilir veya yerleşik olmayabilir. |
| `SourceFile` | Girdi `$MFT` dosyası; ana bilgisayarları veya birimleri birleştirirken vazgeçilmezdir. |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | `-i` ile: satırın yerleşik `$DATA` içeriği (dosyanın veya alternatif veri akışının); Base64 olarak, bayt olarak (`41-42`) ve yazdırılabilir ASCII ise metin olarak. Veri yerleşik değilse veya `-E`/`-M` ile filtrelenmişse boştur. |
| `ParseErrors` | Bu satırı etkileyen hasar veya desteklenmeyen ilişkiler. Filtreleme sırasında bu sütunu asla atmayın. |
| `FromSlack` | `-x` ile: bir FILE kaydının slack alanından kurtarılan bir ad için `True`. `EntryNum` ve `SN` değerleri adın işaret ettiği dosyadır (girdi başlığının üzerine yazılmışsa boştur), kurtarılan zaman damgaları `(FN)` sütunlarındadır ve `InUse` değeri `False` olur. |
| `SlackRecord`, `SlackOffset` | `-x` ile: slack alanı adı tutan FILE kaydı; içine yazılmış kayıt numarasıyla değil `$MFT` içindeki yeriyle belirtilir (adları temel kaydıyla birlikte listelense bile uzantı kaydının kendisi) ve adın `$MFT` içindeki bayt ofseti. Bir slack satırının `ParseErrors` değeri, `fixup values do not match` gibi o kaydın sorunlarıdır: kurtarılan baytlar bundan etkilenmiş olabilir. |
| `CopyOfEntryInUse` | `-x` ile: kullanımdaki bir dosya aynı dizinde hâlâ bu ada sahiptir (referans varlığını sürdürmüşse aynı dosyadır). İlginç olanlar, kullanımdaki hiçbir dosyanın sahip olmadığı adlardır: silinmiş, yeniden adlandırılmış veya yeniden kullanılmış bir kaydın önceki sahibine ait. |

Zaman damgası sütunları MACB sırasını (Modified, Accessed, Changed, Birth/oluşturma) izler:
`DataModified` SI/FN, `Accessed` SI/FN, `MetadataChanged` SI/FN, ardından `Created` SI/FN.

`(SI)`, `$STANDARD_INFORMATION` anlamına gelir; `(FN)`, `$FILE_NAME` anlamına gelir. Bu adlar, eksiksiz bir
kayıt için olağan kaynağı tanımlar. SI özniteliği olmayan normal bir dosya adı satırında mevcut yedek
davranış, FN oluşturma zamanını `Created (FN)` içine, FN değiştirme/kayıt değişikliği/erişim değerlerini ise
sırasıyla `DataModified (SI)`, `MetadataChanged (SI)` ve `Accessed (SI)` içine yerleştirir. Kurtarılan slack
satırları dört zamanın tümünü `(FN)` sütunlarına yerleştirir ve SI sütunlarını boş bırakır. Normal satırdaki
yedek değerler bağımsız SI kanıtı **değildir**. Zaman damgasının kaynağını belirlerken özgün JSON
özniteliklerini ve slack kökenini kullanın.

`-t ys-ntfs-parser-csv` (eski adıyla `ys-mft-parser-csv`, hâlâ kabul edilir), bakımı yapılan ayrıştırıcının
girdi odaklı şemasını kullanır. Bu okunabilir sütun adları, `auto-parse` komutunun `MFT.csv` dosyası da
dahil olmak üzere varsayılan `-t csv` çıktısı için geçerlidir. JSON/JSONL her özniteliği (onaltılık olarak
yerleşik `$DATA` dahil) ve `FullPath` alanını korur; düz inceleme CSV'si düşük seviyeli yapıyı içermediğinde
bunu kullanın. `-x` ile her JSON girdisinde ayrıca `SlackFileNames` bulunur.

## İnceleme stratejisi { #investigation-strategy }

1. Bir zaman penceresine göre filtreleyin, ancak silinmiş kayıtları ve her iki zaman damgası kümesini koruyun.
2. Tam yolları, şüpheli çift uzantıları, ADS'leri, hazırlık dizinlerini ve bilinen karmaları/adları arayın.
   `-x` ile, kullanımdaki hiçbir dosyanın sahip olmadığı slack adlarını (`CopyOfEntryInUse` `False`)
   dahil edin; `-i -E ps1,bat,vbs,js` ile küçük betikleri doğrudan CSV'den okuyun.
3. Girdi/sıra numarası ve `USN` üzerinden USN Journal'a pivot yapın.
4. Prefetch/LNK/Jump List MFT referanslarını hedef kayıtla karşılaştırın.
5. Silmeyi Defender karantinası veya USN `FILE_DELETE`/yeniden adlandırma etkinliğiyle ilişkilendirin.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [ys-ntfs-parser'ın mft crate'i](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [Omer Ben-Amram'ın mft ayrıştırıcısı](https://github.com/omerbenamram/mft)
- [libfsntfs biçim belgeleri](https://github.com/libyal/libfsntfs/tree/main/documentation)
