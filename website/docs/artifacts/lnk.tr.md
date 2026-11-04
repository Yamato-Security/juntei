# LNK Kısayolları { #lnk-shortcuts }

## LNK dosyaları nedir { #what-lnk-files-are }

Windows Shell Link (`.lnk`) dosyaları bir hedefi; başlatma bağımsız değişkenleri, çalışma dizini, simge,
pencere durumu, hedef meta verileri, birim/ağ bilgileri, kabuk ad alanı öğeleri ve isteğe bağlı
dağıtılmış bağlantı izleme (distributed link tracking) tanımlayıcılarıyla birlikte tanımlar. Windows ve uygulamalar bu dosyaları Recent Items,
Desktop, Start Menu ve diğer konumlarda oluşturur; saldırganlar da kötü amaçlı kısayolları doğrudan teslim eder.

Recent klasöründeki bir LNK, çıkarılabilir sürücü veya ağ paylaşımı da dahil olmak üzere artık var olmayan bir
hedefle kullanıcı etkileşimini destekleyebilir. Rastgele bir LNK ise hiç tıklanmadan da var olabilir.

<span class="evidence-strength">En iyi sonuç:</span> kısayol, kaydedilen hedef bağlamıyla
oluşturulmuş/güncellenmiştir. Kullanımı destekleyip desteklemediğini konumu ve çevresindeki artefaktlar belirler.

## Konumlar ve toplama { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

Yalnızca `Recent` klasörünü değil, tüm kullanıcı profilini ve toplanan arşiv/e-posta içeriğini de arayın.
Konum yorumu değiştirdiği için kaynak dosyanın kendi zamanlarını ve yolunu koruyun.

## Analist için uyarılar { #analyst-cautions }

- Hedef zamanları, boyut, MFT referansı ve birim verileri LNK içinde saklanan anlık görüntülerdir; bunlar
  kısayol dosyasının kendi meta verileri değildir ve güncelliğini yitirmiş olabilir.
- `MachineID`, MAC ve izleyici oluşturma zamanı dağıtılmış bağlantı izleme bağlamını açıklar; bu genellikle
  bağlantıyı oluşturan/güncelleyen makinedir, kısayolu bulduğunuz makine olmak zorunda değildir.
- MAC benzeri bir UUID düğümü sanallaştırmadan, rastgele üretimden veya ağ dışı tanımlayıcılardan gelebilir.
- `Args`, zararsız bir yürütülebilir dosyayı (`powershell.exe`, `rundll32.exe`, `mshta.exe`) tehlikeli hâle getirebilir.
- ANSI dizeleri kaynak sistemin kod sayfasını gerektirir; birçok Japonca bağlantı için `--code-page 932` kullanın.
- Bir kısayol özel olarak hazırlanmış veya kopyalanmış olabilir. Prefetch,
  Jump List, RecentDocs, olay günlüğü veya başka bir destek olmadan alanlarını yürütmeyle eşdeğer saymayın.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## CSV çıktı alanları { #csv-output-fields }

| Alan grubu | Alanlar ve yorumlama |
|---|---|
| Kaynak | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: kısayol yolu ve barındıran dosya sistemindeki zamanlar. Bunlar hedef zamanlarından ayrıdır. |
| Hedef zamanları | `TargetCreated`, `TargetModified`, `TargetAccessed`: LNK içinde kaydedilen anlık görüntüler; geçmişe dair faydalı ipuçlarıdır, hedefin güncel meta verileri değildir. |
| Hedef temel bilgileri | `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`: kaydedilen hedef/başlatma bağlamı. Bağımsız değişkenler anlamı değiştirebilir. |
| Birim | `DriveType`, `VolumeSerialNumber`, `VolumeLabel`: sabit/çıkarılabilir/ağ bağlamını belirler ve diğer kısayollarla ilişkilendirmeyi sağlar. |
| Yollar | `LocalPath`, `NetworkPath`, `CommonPath`, `TargetIDAbsolutePath`: birbirini tamamlayan yol gösterimleri. Aralarındaki tutarsızlıkları ve kabuk ad alanı yollarını inceleyin. |
| Başlatma davranışı | `Args`, `ShowWindow`, `HotKey`, `ShimLayer`, `DarwinId`: kabuğun hedefi nasıl başlattığı veya yapılandırdığı. `Args` kritik önemdedir. |
| NTFS kimliği | `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`: bağlantı oluşturulduğu andaki hedef referansı; her ikisini de `$MFT` ile eşleştirin. |
| İzleme | `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `VolumeDroid`, `FileDroid`, `VolumeDroidBirth`, `FileDroidBirth`: dağıtılmış bağlantı izleme. Tek başına atıf için değil, ilişkilendirme için kullanın. |
| Ek bloklar | `ExtraBlocksPresent`: mevcut isteğe bağlı yapıların listesi; diğer hangi alanların desteklendiğini açıklar. |
| Sunum | `Name`, `IconLocation`, `IconIndex`, `IconEnvironment`, `EnvironmentVariables`: görüntüleme ve ortam değişkenleri genişletilmiş hedef verileri. Saldırganlar yanıltıcı adlar/simgeler kullanabilir. |
| Kabuk klasörleri | `KnownFolder`, `SpecialFolder`: çözümlenmiş Windows kabuk klasörü tanımlayıcıları. |
| Ağ | `NetworkDevice`, `NetworkProvider`: ağ paylaşımı/aygıtı ve sağlayıcı bağlamı. |
| Kodlama | `ConsoleCodePage`: mevcutsa kaydedilmiş konsol kod sayfası; ayrıştırıcının `--code-page` seçeneğinden ayrıdır. |
| Sondaki veriler | `AppendedBytes`: mantıksal sonun ardından gelen baytlar. Beklenmeyen veriler kaçakçılığa (smuggling) veya bozuk biçimli dosyalara işaret edebilir. |
| Doğrulama | `Problems`: hasar, kesilme, tutarsız bloklar veya kod çözme sınırlamaları. İncelemeye dahil edin. |

JSON/JSONL ayrıca her hedef kimliği öğesini (tür, ad, GUID, FAT zamanları ve MFT
referansı), tüm ek veri bloklarını, özellik deposu özelliklerini, konsol ayarlarını ve ham yapısal
ilişkileri korur. Şüpheli bir kısayol için yalnızca düz CSV satırına güvenmek yerine bunu kullanın.
İç içe `TargetItems` zamanları, saklanmış bir ofset içermeyen FAT kabuk öğesi saatlerinden gelir.
Bunların eski tarz `Z` gösterimi UTC olduklarını kanıtlamaz; `auto-parse --timezone` bunları olduğu gibi korur.
LNK başlığındaki hedef FILETIME değerleri ve izleyici zamanları dönüştürülebilir mutlak zaman damgaları olarak kalır.

## İnceleme stratejisi { #investigation-strategy }

1. Hedef alanlarını yorumlamadan önce kaynak konumunu ve kaynak zamanlarını inceleyin.
2. Hedef yolu, `Args`, `WorkingDirectory` ve ortam bloklarından gerçek başlatma komutunu yeniden oluşturun.
3. Hedef MFT referansını, birim seri numarasını, ağ yolunu, droid'leri, MachineID'yi ve MAC'i kanıtlar arasında pivot noktası olarak kullanın.
4. Yürütme için Prefetch ve olay günlüklerini; kullanıcı etkileşimi için Jump Lists/RecentDocs'u kontrol edin.
5. Kötü amaçlı ekler için sona eklenmiş verileri koruyun ve LNK'nin kendisinin karmasını alıp analiz edin.

## Daha fazla ayrıntı { #further-details }

- [Microsoft Open Specifications: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Eric Zimmerman's LECmd](https://github.com/EricZimmerman/LECmd)
