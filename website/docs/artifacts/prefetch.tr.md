# Prefetch { #prefetch }

## Prefetch nedir { #what-prefetch-is }

Windows Prefetch dosyaları, uygulama başlatmayı optimize etmek için kullanılan bilgileri kaydeder.
Programa özgü bir `.pf` dosyası genellikle yürütülebilir dosya adını ve karmasını, çalıştırma sayısını, son
çalıştırma zamanlarını, birim bilgilerini, dizinleri ve başlatma sırasında başvurulan dosyaları saklar.

Desteklenen Windows sürümlerinde geçerli bir uygulama Prefetch kaydı, genellikle yürütülebilir dosyanın
çalıştığına dair güçlü bir kanıttır. Olay günlüğü dışındaki en faydalı yürütme artefaktlarından biridir.

<span class="evidence-strength">En iyi sonuç:</span> adı geçen yürütülebilir dosya, işletim sistemi
davranışına ve ayrıştırıcı geçerliliğine bağlı olarak, kaydedilen son çalıştırma zaman(lar)ında
başlatılmıştır.

## Konum ve toplama { #location-and-collection }

```text
C:\Windows\Prefetch\*.pf
```

Juntei, sıkıştırılmış Windows 8+ dosyaları dahil olmak üzere 17 ile 31 arasındaki biçim sürümlerini
(Windows XP'den Windows 11'e kadar) destekler. Yürütülebilir dosya adı karmalarının ve komşu kayıtların
karşılaştırılabilmesi için dizinin tamamını toplayın.

## Analist için uyarılar { #analyst-cautions }

- Windows eski sürümlerde yalnızca bir geçmiş çalıştırma zamanı, yeni sürümlerde ise en fazla sekiz tane
  saklar. `RunCount`, mevcut zaman damgalarının sayısını aşabilir.
- Prefetch bazı sistemlerde/rollerde devre dışı bırakılmış, temizlenmiş, hariç tutulmuş veya kullanılamıyor
  olabilir. Yokluğu, bir programın çalışmadığının kanıtı değildir.
- Yüklenen dosya listeleri başlatma bağımlılıklarını ve erişimleri tanımlar; listelenen her dosyanın kötü
  amaçlı olduğunu, bilerek açıldığını veya tamamen okunduğunu kanıtlamaz.
- Prefetch başlığındaki yürütülebilir dosya adının uzunluğu sınırlıdır. Juntei, aynı adlı yüklenen
  yolları SCCA yol karmasıyla ayırt eder. Birden fazla aday çözümlenemezse başlık adını korur ve
  PathAmbiguous değerini ayarlar; yüklenen dosyaların listesi kullanılabilir olmaya devam eder.
  Karması eşleşmeyen tek bir aday korunur ve PathValidated değeri false olur. Barındırma süreçleri
  komut satırı verilerini karmaya katabilir; bu nedenle doğrulanmamış bir yol kendiliğinden yanlış
  değildir.
- Bir `.pf` kaynak dosyasının dosya sistemi zaman damgaları, içine gömülü çalıştırma zamanlarından ayrıdır.
- `Findings` tehdit avı gözlemleridir. Yükleyiciler ve meşru yönetim işlemleri sıklıkla hazırlık yollarını
  kullanır.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.ExecutableName? | test("powershell|rundll32"; "i"))'
```

## Standart CSV alanları { #standard-csv-fields }

| Alan | Yorumlama |
|---|---|
| `SourceFile` | Girdi `.pf` dosyası; ana bilgisayarları ve yinelenen yürütülebilir dosya adlarını ayırt etmek için koruyun. |
| `ExecutableName` | Başlıktaki yürütülebilir dosya adı. Çoğu tehdit avında büyük/küçük harf duyarsızdır; özgün yazımı koruyun. |
| `ExecutablePath` | Mümkün olduğunda Juntei'nin çözümlediği yüklenme yolu. `Files` ve diğer artefaktlarla doğrulayın. |
| `PathValidated` | Seçilen yol SCCA yol karmasıyla eşleşir. Bu, dosyanın içeriğini veya güvenilirliğini değil yol atfını doğrular. |
| `PathAmbiguous` | Birden fazla yüklenen yol adayı çözümlenememiştir; `ExecutablePath` başlık adını korur. `Files` ve diğer artefaktları inceleyin. |
| `PrefetchHash` | Windows'un `.pf` dosya adında kullandığı karma; kriptografik bir dosya içeriği karması değildir. |
| `FormatVersion` | Prefetch yapı sürümü; mevcut alanları ve zaman damgası sayısını belirler. |
| `RunCount` | Windows'un kaydettiği kümülatif sayaç. Sıklık için faydalıdır, kesin geçmiş için değil. |
| `LastRun` | Saklanan en son yürütme zamanı. |
| `PreviousRun1` … `PreviousRun7` | Saklanan önceki yürütme zamanları, en yeniden eskiye. Biçimde/geçmişte bulunmadıkları yerde boştur. |
| `VolumeCount` | Kaydedilen kaynak birimlerin sayısı. |
| `VolumeDevicePaths` | Başlatma sırasında kullanılan birimlerin NT aygıt yolları. Seri numaralarıyla ilişkilendirin. |
| `VolumeSerialNumbers` | Birim tanımlayıcıları; çıkarılabilir veya yeniden biçimlendirilmiş sürücü kanıtlarını bağlamak için faydalıdır. |
| `VolumeCreated` | Kaydedilen birim oluşturma zamanları; yürütülebilir dosyanın oluşturulma zamanı değildir. |
| `DirectoryCount` | Kaydedilen dizin dizelerinin sayısı. |
| `FileCount` | Kaydedilen yüklenen dosya referanslarının sayısı. |
| `Findings` | Yeri değiştirilmiş sistem ikili dosyaları veya şüpheli yürütme yolları gibi Juntei gözlemleri. Elle gözden geçirin. |
| `Files` | Mevcut NTFS referanslarıyla birlikte yüklenen/başvurulan dosyalar. Mükemmel bir DLL/yol pivot noktasıdır; dosyaya yönelik niyetin kanıtı değildir. |
| `Dir` | Başlatmayla ilişkili dizin dizeleri. Tek tek yüklenen dosya yolları eksik olduğunda faydalıdır. |

## Zaman çizelgesi CSV alanları (`-T`) { #timeline-csv-fields-t }

| Alan | Yorumlama |
|---|---|
| `RunTime` | Saklanan bir Prefetch yürütme zamanı. |
| `ExecutableName`, `ExecutablePath` | Program kimliği/yol bağlamı. |
| `PathValidated` | Seçilen yol SCCA yol karmasıyla eşleşir. Bu, dosyanın içeriğini veya güvenilirliğini değil yol atfını doğrular. |
| `PathAmbiguous` | Birden fazla yüklenen yol adayı çözümlenememiştir; `ExecutablePath` başlık adını korur. `Files` ve diğer artefaktları inceleyin. |
| `PrefetchHash` | Aynı yürütülebilir dosya adını paylaşan varyantları/yolları ayırır. |
| `RunCount` | Prefetch dosyasındaki sayaç; her zaman çizelgesi satırında tekrarlanır. |
| `SourceFile` | Doğrulama için kaynak `.pf` dosyası. |

JSON/JSONL; birimleri, dizinleri, yüklenen dosyaları ve NTFS dosya referanslarını virgülle birleştirilmiş
CSV hücreleri yerine yapılandırılmış diziler olarak korur.

## İnceleme stratejisi { #investigation-strategy }

1. Saklanan çalıştırmaları bir zaman çizelgesine eklemek için `-T` kullanın.
2. Yürütülebilir dosya adı ile Prefetch karmasına göre gruplayın; aynı adın farklı yolları/karmaları olabilir.
3. `ExecutablePath`, hazırlık yollarını, yeri değiştirilmiş sistem ikili dosyası bulgularını ve yüklenen
   DLL'leri gözden geçirin.
4. Hedef MFT referanslarını ve dosya yollarını `$MFT`/USN kayıtlarıyla eşleştirin.
5. Çalıştırma zamanlarını olay günlükleri, SRUM, ağ telemetrisi, hizmetler, görevler ve kullanıcı
   artefaktlarıyla ilişkilendirin.

## Daha fazla ayrıntı { #further-details }

- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [Security Ronin prefetch-forensic](https://github.com/SecurityRonin/prefetch-forensic)
- [Eric Zimmerman'ın PECmd aracı](https://github.com/EricZimmerman/PECmd)
