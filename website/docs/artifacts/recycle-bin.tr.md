# Geri Dönüşüm Kutusu { #recycle-bin }

## Geri Dönüşüm Kutusu neleri kaydeder { #what-the-recycle-bin-records }

Bir kullanıcı Dosya Gezgini'nde bir dosyayı veya klasörü sildiğinde Windows bu öğeyi bulunduğu birimin Geri Dönüşüm
Kutusu'na taşır ve yanına küçük bir indeks kaydı yazar: öğenin önceki yolu, boyutu ve silindiği zaman. Her kullanıcının
her birimde, kullanıcının SID'siyle adlandırılmış bir Geri Dönüşüm Kutusu klasörü vardır; bu nedenle klasör, öğeyi
hangi hesabın sildiğini de gösterir.

| Windows sürümü | İndeks | Silinen içerik |
|---|---|---|
| Vista ve sonrası | Öğe başına bir `$I…` dosyası (Vista ile 8.1 arasında biçim sürümü 1, Windows 10 ve sonrasında sürüm 2) | `$R…`; `$R` sonrasındaki ad aynıdır: dosya ya da içeriğiyle birlikte klasör |
| NT 4.0, 2000, XP, Server 2003 | Her Geri Dönüşüm Kutusu klasörü için bir `INFO2` dosyası: bir ANSI ve bir Unicode yol içeren 800 baytlık kayıtlar | `D<drive letter><index><extension>`, örneğin `Dc1.txt` |
| 95, 98, ME | Bir `INFO2` dosyası (Windows 95'te `INFO`): yalnızca ANSI yol içeren 280 baytlık kayıtlar | Aynı `D…` adları |

<span class="evidence-strength">En iyi sonuç:</span> kaydın bulunduğu Geri Dönüşüm Kutusu klasörünün ait olduğu hesap,
özgün yoldaki öğeyi kaydedilen zamanda Geri Dönüşüm Kutusu'na taşımıştır. Shift+Delete ile, komut satırından veya çoğu
programdan yapılan silme işlemleri Geri Dönüşüm Kutusu'nu atlar ve hiçbir kayıt bırakmaz.

## Konumlar ve toplama { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

Her birimin Geri Dönüşüm Kutusu'ndaki `$I` dosyalarını veya INFO2 dosyasını toplayın. `$R` ve `D…` dosyalarını da
toplamak, silinen öğelerden hangilerinin hâlâ orada olduğunu gösterir; bunlar olmadan `RecycledFileFound` her öğe için
`false` olur. Silinen içerik başlı başına bir kanıttır ve kötü amaçlı yazılım olabilir: onu buna göre ele alın.

## Analist için uyarılar { #analyst-cautions }

- `DeletedTime`, öğenin Geri Dönüşüm Kutusu'na taşındığı zamandır; sistem saatine göre alınır ve UTC FILETIME olarak
  saklanır. Geri Dönüşüm Kutusu'nun boşaltıldığı zaman değildir. Öğenin kendi oluşturma ve değiştirme zamanları `$MFT`
  ve `$R` dosyasında bulunur.
- SID, öğeyi silen hesabı gösterir; dosyanın sahibi olmak zorunda değildir. SOFTWARE kovanının `ProfileList` verisinde
  profili olmayan bir SID'nin (silinmiş bir hesap veya hiç oturum açmamış bir etki alanı hesabı) `User` değeri yoktur.
- Geri Dönüşüm Kutusu boşaltıldığında ya da bir öğe geri yüklendiğinde veya Geri Dönüşüm Kutusu'ndan silindiğinde,
  Windows Vista ve sonrası öğenin `$I` ve `$R` dosyalarını siler. Bunların adları USN Günlüğü'nde, `$MFT` içinde ve
  `$I30` slack alanında kalabilir; Juntei bunlardan silinmiş `$I` kayıtlarını kurtarmaz. INFO2 bu tür kayıtları tutar ve
  ANSI yollarının ilk baytını sıfırlar: `RemovedFromBin` `true` olur ve Juntei bu karakteri sürücü numarasından geri
  yazar.
- Bir `$I` dosyasındaki `FileSize`, öğenin boyutudur; klasörlerde içeriğinin toplam boyutudur. INFO2'de ise öğenin
  diskte kapladığı alandır ve tam küme (cluster) olarak verilir (küçük bir dosya için 4.096 bayt).
- INFO2'deki ANSI yol çoğunlukla 8.3 biçimindedir ve sistemin kod sayfasını gerektirir: Japonca Windows için
  `--code-page 932`, Çince Windows için 936 veya 950 kullanın. Unicode yol, varsa, `OriginalPath` alanındadır.
- Bazı Windows Vista sistemleri, boyut alanı bir bayt eksik olan 543 baytlık `$I` dosyaları yazmıştır. Juntei bunların
  zamanını ve yolunu okur, `FileSize` alanını boş bırakır ve bir sorun bildirir.
- `RecycledFileFound` uç noktayı değil toplamayı tanımlar: `false`, toplayıcının `$R` dosyalarını almadığı ya da öğenin
  artık Geri Dönüşüm Kutusu'nda olmadığı anlamına gelebilir.
- Windows kurulumu olmayan birimlerdeki Geri Dönüşüm Kutusu klasörleri, kullanıcı adlarını toplamadaki tek Windows
  kurulumundan alır (tam olarak bir kurulum varsa).

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

`-d` ile Juntei, adı `$I` ve ardından altı harf veya rakamdan oluşan dosyaları (uzantıları ne olursa olsun), `INFO2`
dosyalarını ve bir `RECYCLED` klasöründeki `INFO` dosyalarını okur. `-f` ile dosyayı adı ne olursa olsun okur.

## CSV çıktı alanları { #csv-output-fields }

| Alan | Anlamı |
|---|---|
| `SourceFile` | `$I` veya INFO2 dosyası. |
| `DeletedTime` | Öğenin Geri Dönüşüm Kutusu'na taşındığı zaman (UTC). |
| `OriginalPath` | Öğenin bulunduğu yer. INFO2'de, kayıtta varsa Unicode yol, yoksa ANSI yol. |
| `FileSize` | `$I`: öğenin bayt cinsinden boyutu (klasörlerde toplam boyut). INFO2: diskte kapladığı alan, tam küme olarak. Okunamadığında boştur. |
| `User`, `SID` | Geri Dönüşüm Kutusu klasörünün SID'si ve bu SID'ye ait `ProfileList` profiline adını veren kullanıcı. |
| `RecycledFile`, `RecycledFileFound` | İndeksin yanındaki silinen içeriğin yolu (`$R…`, `D<drive><index><extension>`) ve bu içeriğin toplamada bulunup bulunmadığı. |
| `Format`, `FormatVersion` | `$I` 1 (Vista ile 8.1 arası) veya 2 (Windows 10 ve sonrası); `INFO2` 0 (95), 2 (NT 4.0), 4 (98) veya 5 (ME, 2000 ile Server 2003 arası). |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2: kaydın numarası (`D…` dosya adındaki numara), sürücü ve kaydın INFO2 içinde başladığı konum. UNC yolunun sürücüsü yoktur. |
| `RemovedFromBin` | INFO2: öğe geri yüklendiğinde veya Geri Dönüşüm Kutusu'ndan silindiğinde `true`. |
| `ShortPath` | INFO2: `OriginalPath` değerinden farklı olduğunda ANSI yol (çoğunlukla 8.3 adları). |
| `Problems` | Hasar: kesik veya fazla uzun bir dosya, geçersiz bir sürücü numarası, geçerli UTF-16 olmayan bir yol, geçerli bir zaman olmayan bir silinme zamanı. |

JSON ve JSONL aynı alanlara sahiptir; sayılar ve boole değerleri kendi türleriyle, bir kayıtta bulunmayan değerler
`null` olarak ve `Problems` bir liste olarak verilir.

## Otomatik ayrıştırma { #automatic-parsing }

`auto-parse`, `FileExistence/RecycleBin.csv` dosyasını (`-t json` veya `-t jsonl` ile `RecycleBin.json` ya da
`RecycleBin.jsonl`) `FileExistence/RecycleBin-logs.txt` ile birlikte yazar. `--code-page` seçeneği INFO2 için
geçerlidir. Disk imajlarından `$I` dosyaları ve INFO2 kopyalanır; `$R` ve `D…` dosyaları boş yer tutuculardır, bu
nedenle `RecycledFileFound` yine doldurulur. Birleştirilmiş `-j` zaman çizelgesi her satırı `Artifact` tanımlayıcısı
`recycle-bin` ile birlikte içerir; `-Z`, `DeletedTime` değerini dönüştürür ve
[`super-timeline`](../commands/super-timeline.md) her öğe için bir `Recycle Bin deletion (moved to the bin)`
olayı oluşturur.

## İnceleme stratejisi { #investigation-strategy }

1. Her kullanıcı için `DeletedTime` değerine göre sıralayın ve incelediğiniz etkinliğin zamanlarıyla karşılaştırın.
2. Öğenin silinmeden önce kullanılıp kullanılmadığını görmek için `OriginalPath` değerini LNK, Jump Lists, ShellBags
   ve RecentDocs içinde arayın.
3. USN Günlüğü'nde silme işlemi, yaklaşık `DeletedTime` zamanında `$Recycle.Bin` içine yapılan bir yeniden adlandırma
   olarak görünür; öğenin kendi zamanlarını `$MFT` verir.
4. `RecycledFileFound` `true` olduğunda `$R` veya `D…` içeriğinin karmasını alın ve inceleyin.
5. Geri Dönüşüm Kutusu boşaltıldıysa USN Günlüğü'nde ve `$MFT` içinde `$I` ve `$R` adlarını arayın.

## Daha fazla ayrıntı { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
