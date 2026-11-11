# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## ShimCache nedir { #what-shimcache-is }

Windows'un Uygulama Uyumluluğu alt sistemi, uyumluluk/shim davranışını değerlendirirken yürütülebilir dosya yollarını
ve meta verilerini önbelleğe alır. Önbellek, dosyalar yeniden adlandırılsa, kaldırılsa veya çıkarılabilir bir birimle
birlikte bağlantısı kesilse bile varlığını sürdürür. Düzeni ve mevcut alanları Windows XP'den Windows 11'e kadar farklılık gösterir.

ShimCache, geçmişteki yolların keşfi için ve biçimin güvenilir bir yürütme bayrağına sahip olduğu durumlarda
yürütmeye ek destek için değerlidir. Önbelleğe alınan zaman normalde **dosyanın son değiştirilme zamanıdır**;
Windows'un dosyayı gördüğü veya çalıştırdığı zaman değildir.

<span class="evidence-strength">En iyi sonuç:</span> Windows, yolu uyumluluk işlemesi için
gözlemlemiştir. Bir girdi tek başına varlık/gözlem kanıtıdır; `Executed` değerini yalnızca desteklendiği durumlarda kullanın.

## Konum ve toplama { #location-and-collection }

Baytlar `SYSTEM` kayıt defteri kovanında, genellikle şu konumda bulunur:

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

`SYSTEM`, `SYSTEM.LOG1` ve `SYSTEM.LOG2` dosyalarını toplayın. Juntei tüm denetim kümelerini okur ve mümkün olduğunda
`Select\Current` değerini belirler.

!!! warning "En yeni canlı girdiler kovanda olmayabilir"
    Windows, ShimCache güncellemelerini bellekte tutup kapatma/yeniden başlatma sırasında diske yazabilir. Çalışan bir sistemden
    kopyalanan bir kovan en yeni gözlemleri içermeyebilir. Önbelleğin kayıt defteri son yazılma
    zamanı, ne zaman kaydedildiğini açıklamaya yardımcı olur, ancak girdi başına bir yürütme zamanı değildir.

## Analist için uyarılar { #analyst-cautions }

- Bir önbellek girdisi tek başına **yürütmenin kanıtı değildir**.
- `Executed` değeri `Yes`, `No` veya `NA` olabilir; mevcut anlamlar biçime/sürüme bağlıdır.
- `LastModifiedTimeUTC` gözleme veya yürütmeye değil, başvurulan dosyaya aittir.
- Sıra faydalıdır, ancak evrensel olarak kesin bir zaman çizelgesi değildir; denetim kümeleri/slack arasında yinelenen kayıtlar görülür.
- `FromSlack` girdileri etkin önbelleğin dışından kurtarılmıştır ve ek özen gerektirir.
- Kullanılabilir günlükleri olmayan kirli (dirty) bir `SYSTEM` kovanında güncel kayıtlar eksik olabilir. Kurtarma özetini okuyun.
- Bulgular, kötü amaçlı yazılım hükümlerini değil, şüpheli adlandırma/yol kalıplarını gösterir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `ControlSet` | Önbelleği tutan sayısal denetim kümesi. Birini seçmeden önce `IsCurrent` değerini karşılaştırın. |
| `CacheEntryPosition` | Kodu çözülen önbellek içindeki konum/sıra. Göreli sıralama için faydalıdır, bir zaman değildir. |
| `Path` | Önbelleğe alınan yürütülebilir dosya/paket yolu. Birincil av (hunting) pivot noktası. |
| `LastModifiedTimeUTC` | Girdide saklanan dosya değiştirilme zamanı; yürütme/gözlem zamanı değildir. |
| `Executed` | Desteklenen biçim bayraklarına göre `Yes`, `No` veya `NA`. Yalnızca sürüm bağlamıyla birlikte kullanın. |
| `Duplicate` | Aynı girdi daha önceki bir denetim kümesinde veya slack'te zaten görülmüştür; çift saymayı önlemeye yardımcı olur. |
| `SourceFile` | Girdi kovanı/ham değer. Ana bilgisayarlar arasında vazgeçilmezdir. |
| `FromSlack` | Etkin listeden değil, önbellek değerinin slack alanından kurtarılan girdi. Daha düşük güvenilirliktedir ve muhtemelen daha eskidir. |
| `CacheFormat` | Kodu çözülen Windows/önbellek düzeni. Alanların kullanılabilirliğini ve anlamlarını belirler. |
| `CacheLastWrittenUTC` | Kayıt defteri anahtarının/önbelleğin kaydedilme zamanı. Ortak bağlamdır, her girdinin çalışma zamanı değildir. |
| `InsertFlags`, `ShimFlags` | Ham uyumluluk bayrakları; `Executed`, desteklendiği durumlarda Juntei'nin yorumladığı sonuçtur. |
| `DataSize` | Bu girdinin verisinin boyutu; yapısal doğrulama için faydalıdır. |
| `Machine` | Yeni biçimlerin kaydettiği mimari (`x64`, `x86`, `ARM64`). |
| `FileSize`, `LastUpdateTimeUTC` | Mevcut olduğunda Windows XP'ye özgü önbelleğe alınmış meta veriler. |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | Desteklenen girdilerde paketlenmiş uygulama kimliği. Yayımcı kimliği bağlamdır, imza kanıtı değildir. |
| `Findings` | Yeri değiştirilmiş sistem ikili dosyası veya şüpheli hazırlık (staging) yolu gibi Juntei anomali kodları. Yalnızca ipuçlarıdır. |
| `RegistryPath`, `RegistryValueName` | Tam kovan anahtarı/değer kaynağı. Doğrulama için koruyun. |
| `CacheValueSize` | Tek bir girdinin `DataSize` değeri değil, kayıt defteri değerinin tamamının boyutu. |
| `IsCurrent` | Bu girdinin denetim kümesinin `SYSTEM\Select\Current` değerine eşit olup olmadığı; güvenli biçimde bilinmiyorsa boş. |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | Bağlam ve kurtarma analizi için ham `SYSTEM\Select` denetim kümesi numaraları. |

JSON/JSONL ayrıca girdi verilerini, etiketleri, paketlenmiş uygulama alt kayıtlarını, yapılandırılmış bulguları,
önbellek sorunlarını ve kurtarma ayrıntılarını korur.

## İnceleme stratejisi { #investigation-strategy }

1. Geçerli denetim kümesini tercih edin, ancak diğer kümeleri ve slack'i geçmişe dair kanıt olarak saklayın.
2. Yollarda ve bulgularda arama yapın; `LastModifiedTimeUTC` değerini yürütme zamanı olarak kullanmayın.
3. `Executed == Yes` olduğunda Prefetch, SRUM, olay günlükleri veya diğer çalışma zamanı izleriyle doğrulayın.
4. Oluşturma, silme ve meta veri geçmişini belirlemek için yolları `$MFT`/USN ile eşleştirin.
5. İşlem günlüklerinin yeniden oynatılıp oynatılmadığını ve canlı önbelleğin diske yazılmamış olabileceğini raporlayın.

## Daha fazla ayrıntı { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Maxim Suhanov'un Windows kayıt defteri biçim belirtimi](https://github.com/msuhanov/regf)
