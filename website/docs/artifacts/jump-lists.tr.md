# Jump Lists { #jump-lists }

## Jump Lists nedir { #what-jump-lists-are }

Jump Lists, görev çubuğundaki ve Başlat menüsündeki uygulama girdilerinden açılan menülerin arkasındaki
yapıdır. Otomatik Jump Lists, bir uygulamanın kullandığı öğelerin Windows tarafından yönetilen geçmişleridir.
Özel Jump Lists ise uygulama tarafından tanımlanan kategoriler ve görevlerdir. Her ikisi de hedef yollar,
zamanlar, birim/ağ bağlamı, MFT referansları ve başlatma bağımsız değişkenleri içeren LNK yapıları
barındırabilir.

Otomatik kayıtlar, bir uygulamanın açtığı dosyaları veya klasörleri, son kullanım sırasını/zamanını,
etkileşim sayısını ve sabitlenme durumunu ortaya koyabilir. Özel listeler "son kapatılan" sayfaları veya
uygulama görevlerini, kötü amaçlı komut bağımsız değişkenleri de dahil olmak üzere açığa çıkarabilir.

<span class="evidence-strength">En iyi sonuç:</span> sahip uygulama, öğeyi/görevi kendi Jump List'inde
kaydetmiş veya sunmuştur. Davranış uygulamaya göre farklılık gösterir; gerçek yürütmeyi/erişimi doğrulayın.

## Konumlar ve toplama { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

Onaltılık dosya adı öneki bir AppID'dir. Her iki dizini de toplayın ve kullanıcı profili yolunu koruyun.
Otomatik listeler, bir `DestList` akışı ve numaralandırılmış LNK akışları içeren OLE Compound File
kapsayıcılarıdır; özel listeler kendi kategori/bağlantı biçimlerini kullanır.

## Analist için uyarılar { #analyst-cautions }

- AppID açıklamaları bilinen bir eşlemeden gelir; bilinmeyen bir AppID kendi başına şüpheli değildir.
- `LastModified` / son kullanım alanları bir DestList girdisine aittir; gömülü hedef zamanları farklıdır.
- Etkileşim sayıları yalnızca desteklenen sürümlerde bulunur ve tam bir açılma sayısı değildir.
- Sabitleme, öğeleri normal güncellik süresinin ötesinde korur; temizlik veya uygulama ayarları diğerlerini
  kaldırabilir.
- Bir liste, güncelliğini yitirmiş/bağlı olmayan akışlar veya daha önceki, daha uzun bir özel listeden
  kalan baytlar içerebilir.
- Gömülü LNK bağımsız değişkenleri ve yolları saldırgan tarafından kontrol edilebilir. Uygulama
  ilişkilendirmesi bir güven göstergesi değil, bağlamdır.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

`auto-parse` ile CSV tabloları, her bilgisayarın doğrudan altında
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` ve
`FileAndFolderOpening/JumpLists-CustomDestinations.csv` olur. Paylaşılan günlük, bu kategorideki
`JumpLists-logs.txt` dosyasıdır. Manifestler tam yolları `OutputFiles` içinde listeler. Bağımsız CSV
dışa aktarımları, seçtiğiniz dizinde `AutomaticDestinations.csv` ve `CustomDestinations.csv` adlarını
korur. JSON ve JSONL otomatik dışa aktarımları tek bir `JumpLists.json` veya `JumpLists.jsonl`
dosyasıdır; kayıt yapısı değişmez.

## AutomaticDestinations CSV alanları { #automaticdestinations-csv-fields }

| Alan grubu | Alanlar ve yorumlama |
|---|---|
| Kaynak | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: Jump List kapsayıcısı ve dosya sistemi zamanları. |
| Uygulama | `AppId`, `AppIdDescription`: dosya adındaki AppID ve bilinen uygulama eşlemesi. Bilinmeyen bir karmadan kimlik çıkarmayın. |
| Kapsayıcı | `HasSps`, `DestListVersion`, `LastUsedEntryNumber`: DestList yapısı/sürümü ve başlık bağlamı. |
| Sıralama | `MRU`, `EntryNumber`: en son kullanım konumu ve onaltılık girdi/akış tanımlayıcısı. |
| Kullanım meta verileri | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition`: DestList yaşam döngüsü/kullanım/sabitleme durumu. Sürüme göre yorumlayın. |
| Ana bilgisayar/yol | `Hostname`, `MacAddress`, `Path`, `DisplayPath`: hedef konumu ve izleme bağlamı. Ana bilgisayar/MAC tek başına atıf için yeterli değildir. |
| İzleme kimlikleri | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid`: taşınmış hedefleri ve gömülü LNK izlemesini ilişkilendirin. |
| Notlar | `Notes`: bağlı olmayan akışlar, kurtarılan artıklar veya diğer ayrıştırıcı bağlamı hakkında bilgi verir. |
| Gömülü LNK | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Args`: [LNK Kısayolları](lnk.md) sayfasında açıklandığı gibi yorumlayın. |
| Ek LNK | `NetworkPath`, `Name`, `IconLocation`: gömülü kısayoldan gelen ağ ve sunum alanları. |
| Doğrulama | `Problems`: eksik akışlar, kesilme, sayı uyuşmazlığı veya bağlantı hasarı. |

## CustomDestinations CSV alanları { #customdestinations-csv-fields }

Özel liste çıktısı `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription` ve `EntryName` (kategori adı veya `Tasks`) ile başlar; ardından yukarıda listelenen tüm
gömülü LNK alanları, sonra da `Notes`, `NetworkPath`, `Name`, `IconLocation` ve `Problems` gelir.

JSON/JSONL ayrıca DestList başlığını, girdi başına puan/sağlama toplamı/özellik deposunu,
`DestListPropertyStore` alanını, bileşik dosya akışlarını, özel kategorileri, son kategoriden sonraki
baytları ve her gömülü LNK'nin iç içe alanlarını korur.

## İnceleme stratejisi { #investigation-strategy }

1. Kullanıcıyı ve kaynak profili belirleyin, ardından AppID'yi dikkatle uygulamaya eşleyin.
2. Otomatik girdileri `LastModified` ve `MRU` alanlarına göre sıralayın; sabitlenmiş öğeleri ayırın.
3. Yolları, ağ paylaşımlarını, çıkarılabilir birim seri numaralarını, MFT referanslarını ve bağımsız
   değişkenleri gözden geçirin.
4. `Notes` alanını ve bağlı olmayan/artık bağlantıları inceleyin: silinmiş veya daha önceki öğeler orada
   kalmış olabilir.
5. RecentDocs, bağımsız Recent LNK'leri, Prefetch, SRUM, `$MFT` ve olay günlükleriyle ilişkilendirin.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Compound Binary File format (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: Shell Link format (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [JumpList ayrıştırıcısı test/referans projesi](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
