# Amcache { #amcache }

## Amcache nedir { #what-amcache-is }

Amcache, bir kayıt defteri kovanında saklanan Windows uygulama uyumluluğu envanteridir. Modern sistemler
yürütülebilir dosyaları `InventoryApplicationFile` altında, Tak ve Çalıştır (Plug-and-Play) aygıtlarını ise
`InventoryDevicePnp` altında kaydeder; eski Windows sürümleri eski tarz bir `Root\File` düzeni kullanır.

Yürütülebilir dosya incelemelerinde Amcache değerlidir; çünkü özgün dosya ortadan kalktıktan sonra bile tam yolu,
SHA-1 değerini, dosya boyutunu, PE meta verilerini, sürüm bilgilerini, yayımcı metnini, bağlama (link) tarihini ve
kayıt defteri zaman damgalarını koruyabilir. Aygıt kayıtları, bağlanan donanımı, sürücüleri,
kurulum tarihlerini ve tanımlayıcıları belirleyebilir.

<span class="evidence-strength">En iyi sonuç:</span> Windows, kaydedilen dosyayı veya aygıtı
envantere almıştır. Bir yürütülebilir dosya kaydı **varlığın kanıtıdır, yürütmenin kanıtı değildir**.

## Konum ve toplama { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

Kovanı etkilenen kurulumdan toplayın. `parse-registry amcache` şu anda birincil kovanı ayrıştırır,
ancak yanındaki Amcache işlem günlüklerini **yeniden oynatmaz**; bu nedenle yalnızca bu günlüklerde
bulunan kayıtlar, canlı veya tutarsız bir toplamada eksik olabilir. Bu sınırlamayı raporda belirtin.

## Analist için uyarılar { #analyst-cautions }

- Bir Amcache yürütülebilir dosya girdisi, bir kullanıcının dosyayı başlattığını kanıtlamaz.
- `Publisher`, Windows tarafından kaydedilen PE/sürüm kaynağı meta verisidir. Dosyanın toplama anında
  geçerli bir Authenticode imzasına sahip olduğunu kanıtlamaz.
- Karma değerleri bulunmayabilir ve eski/modern şemalar aynı değerleri sunmaz.
- `LinkDate` PE başlığından gelir; sahte olabilir, kopyalanmış olabilir veya yeniden üretilebilir derlemelerle üretilmiş olabilir.
- Bir anahtarın son yazılma zamanı kayıt defteri kaydının güncellenmesini açıklar; bu, indirme, oluşturma,
  ilk yürütme veya son yürütme olmak zorunda değildir.
- Yollar çıkarılabilir veya artık mevcut olmayan birimlere başvurabilir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

CSV çıktısı özet, bulgular, dosyalar ve aygıtlar tablolarını içerir. `auto-parse` ile bunlar,
her bilgisayarın `FileExistence/` dizininin hemen altında bulunan `Registry-Amcache-Summary.csv`,
`Registry-Amcache-Findings.csv`, `Registry-Amcache-Files.csv` ve `Registry-Amcache-Devices.csv`
dosyalarıdır; `Registry-Amcache-logs.txt` de tabloların yanında yer alır. Birden fazla kovan olduğunda uzantıdan
önce `-Source1`, `-Source2` vb. eklenir; manifestlerdeki `OutputFiles` tam yolları listeler. Bağımsız CSV
dışa aktarımları kendi tablo adlarını ve istenen çıktı konumunu korur. JSONL kayıtları
`summary`, `finding`, `file` ve `device` değerlerine sahip `RecordType` kullanır.

## Dosya çıktı alanları { #file-output-fields }

| Alan | Analistin nasıl kullanması gerektiği |
|---|---|
| `KeyName`, `SourceKeyPath` | Tam kaynak kayıt defteri kaydını ve şema dalını belirler. Yeniden üretilebilirlik için koruyun. |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | Kayıt defteri kaydının güncellenme zamanı; okunabilir ve ham biçimde. Bunu yürütme zamanı olarak etiketlemeyin. |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | Eski şemalarda üst birim anahtarının zamanı; bağlamsaldır, bir dosya eylemi zamanı değildir. |
| `FileID` | Amcache'in kaydettiği dosya tanımlayıcısı. Olduğu gibi korunur; her zaman bir SHA-1 olduğunu varsaymayın. |
| `SHA1`, `SHA256` | Defender, tehdit istihbaratı, EDR ve dosya depolarına güçlü pivot noktaları. Boş (null) değer kaydedilmediği anlamına gelir. |
| `Name`, `OriginalFileName`, `Description` | Kaydedilen PE/sürüm adları. Kılık değiştirmeyi (masquerading) tespit etmek için gerçek yolla karşılaştırın; meta veriler saldırganın denetimindedir. |
| `FullPath` | En kullanışlı konum pivot noktası. Büyük/küçük harfi dikkatle normalleştirin, ancak özgün kanıt dizesini koruyun. |
| `LongPathHash` | Windows'un yolla ilgili tanımlayıcısı; kayıtları eşleştirmek için kullanışlıdır, bir içerik karması değildir. |
| `ProgramID` | Mevcut olduğunda dosyayı bir uygulama envanteri kaydına bağlar. |
| `Publisher` | Kaydedilen yayımcı metni. İmza doğrulaması olarak değil, meta veri olarak değerlendirin. |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | Sürüm kaynağı bağlamı. Farklılıklar veya akla yatkın olmayan değerler kılık değiştirme analizini destekleyebilir. |
| `BinaryType` | Kaydedilen mimari/tür bağlamı; dosya kurtarıldığında PE analiziyle karşılaştırın. |
| `LinkDate` | Kaydedildiği hâliyle PE derleme/bağlama zaman damgası. Yalnızca şüpheyle ve doğrulayan kanıtlarla birlikte kullanışlıdır. |
| `Size` | Kaydedilen boyut; `$MFT`, Defender içeriği veya kurtarılan bir dosyayla karşılaştırın. |
| `Language` | PE dil tanımlayıcısı; zayıf bir bağlamsal ipucudur, atıf için kullanılamaz. |
| `IsPE-File`, `IsOS-Component` | Windows'un kaydettiği sınıflandırma bayrakları. Yanlış (false) değer zararsızlığı kanıtlamaz. |
| `USN` | Amcache'in kaydettiği NTFS güncelleme sıra değeri; hâlâ mevcut USN kayıtlarına pivot yapmaya yardımcı olabilir. |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | Paketlenmiş uygulamaları/bileşenleri dosyayla ilişkilendirir. |
| `RegistryValues` | Türü, kodu çözülmüş değeri ve ham onaltılık verisiyle birlikte tüm özgün kayıt defteri değerleri. Türlendirilmiş bir alan bulunmadığında veya gelecekteki/eski alanlar için kullanın. |

## Aygıt çıktı alanları { #device-output-fields }

| Alan grubu | Alanlar ve yorumlama |
|---|---|
| Kaynak | `KeyName`, `SourceKeyPath`, `KeyLastWrittenUTC`, `KeyLastWrittenFiletime`: tam aygıt kaydı ve kayıt defteri güncellenme zamanı. |
| Kimlik | `Description`, `BusDescription`, `Manufacturer`, `Model`: kullanıcıya yönelik aygıt açıklamaları; üretici dizeleri genel olabilir. |
| Donanım tanımlayıcıları | `HWID`, `CompID`, `Class`, `ClassGUID`, `ContainerID`, `DeviceInterfaceClasses`: aygıt ailesi ve örneği için en güçlü pivot noktaları. |
| Durum | `DeviceState`, `ProblemCode`, `ParentID`, `MatchingID`, `Enumerator`: Windows numaralandırma ve ilişki bağlamı. |
| Sürücü | `DriverID`, `DriverName`, `DriverPackageStrongName`, `DriverVerDate`, `DriverVerVersion`, `Provider`, `Service`, `INF`: sürücü paketi ve hizmet ilişkileri; yüklü dosyalar ve hizmetlerle ilişkilendirin. |
| Kurulum | `InstallDate`, `FirstInstallDate`, `InstallState`, `ExtendedInfs`: kaydedilen kurulum yaşam döngüsü. Tarihin bulunmaması aygıtın hiç bağlanmadığını kanıtlamaz. |
| Filtreler ve yığın | `LowerClassFilters`, `UpperClassFilters`, `LowerFilters`, `UpperFilters`, `StackID`, `LocationPaths`: ileri düzey sürücü yığını bağlamı; beklenmeyen filtre sürücüleri incelenmelidir. |
| Ham veri koruma | `RegistryValues`: doğrulama ve desteklenmeyen alanlar için eksiksiz türlendirilmiş/ham kaynak değerleri. |

CSV'deki dizi ve ham değer sütunları bir `JSON` son eki kullanır (örneğin `HWID-JSON` ve
`RegistryValuesJSON`); bu hücreler JSON içerir, JSON/JSONL ise yerel dizileri korur.

## İnceleme stratejisi { #investigation-strategy }

1. `FullPath`, `SHA1`, `Name` ve `USN` üzerinden pivot yapın.
2. Yürütmeyi desteklemek için eşleşen Prefetch kayıtlarını arayın.
3. Dosya oluşturma, yeniden adlandırma, silme ve boyutu belirlemek için `$MFT` ve USN Journal'ı kullanın.
4. Yayımcı/sürüm meta verilerini bağımsız olarak kurtarılmış bir PE ve imza doğrulamasıyla karşılaştırın.
5. Tespit veya kaldırma için Defender karantinasını ve olay günlüklerini kontrol edin.

## Daha fazla ayrıntı { #further-details }

- [Juntei'nin ys-amcache-parser çıktı şeması](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [Eric Zimmerman'ın AmcacheParser aracı](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [Regipy Amcache eklentisi](https://github.com/mkorman90/regipy)
