# SRUM { #srum }

## SRUM nedir { #what-srum-is }

Sistem Kaynak Kullanımı İzleyicisi (System Resource Usage Monitor, SRUM), uygulama ve hizmet kaynak
kullanımını bir ESE veritabanında saklar. Windows sürümüne bağlı olarak ağ baytlarını, arabirim/profil
tanımlayıcılarını, bağlantıları, CPU süresini, ön plan/arka plan etkinliğini, disk etkinliğini, enerji
kullanımını, anında iletme bildirimlerini ve uygulama zaman çizelgesi verilerini kaydeder. Girdiler
`SruDbIdMapTable` aracılığıyla uygulama kimlikleri ve kullanıcı SID'leriyle ilişkilendirilir.

SRUM, özellikle bir yürütülebilir dosya silindiğinde yararlıdır: uygulama kimliği, kullanıcısı ve
kaynak kullanım satırları dosyadan daha uzun yaşayabilir. Ağ toplamları bir veri sızdırma hipotezini
destekleyebilir; ancak SRUM bir paket yakalama değildir ve genellikle her uzak uç noktayı veya
aktarılan her dosyayı tanımlamaz.

<span class="evidence-strength">En iyi sonuç:</span> Windows, kaydedilen kaynak kullanımını tablonun
ölçüm dönemi boyunca uygulamaya/hesaba atfetti.

## Konum ve toplama { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM, Windows 8 ve sonraki sürümlerde bulunur. `SRUDB.dat` dosyasını tutarlı bir şekilde, tercihen bir
anlık görüntüden veya bir adli bilişim toplayıcısıyla kopyalayın. Ayrıca `SOFTWARE` kovanını ve `sru`
dizinindeki ESE işlem/günlük dosyalarını da toplayın.

Eşleşen SOFTWARE kovanı ESE veritabanından ayrıdır: kovan dirty durumundaysa Juntei, yanındaki Registry işlem günlüklerini (`SOFTWARE.LOG`, `.LOG1`, `.LOG2`) bellekte otomatik olarak yeniden oynatır. Bu dosyaları da toplayın. Kurtarma ayrıntıları ve uyarılar gösterilir; günlükler yoksa hesap veya Wi-Fi adları eksik ya da eski olabilir. Bu işlem SRUM veritabanı günlüklerini yeniden oynatmaz.

!!! warning "Kirli veritabanları"
    Juntei, kirli bir `SRUDB.dat` dosyasını toplandığı haliyle okur ve bunu raporlar. Şu anda
    `SRU*.log` dosyalarını yeniden oynatmaz; bu nedenle yalnızca bu günlüklerde bulunan değişiklikler
    eksik kalır. Kirli bir veritabanını sessizce eksiksizmiş gibi tanımlamayın.

## Analist için uyarılar { #analyst-cautions }

- Birçok satır, kesin eylem zamanını değil, genellikle saat başı sınırları civarında yapılan dönemsel
  toplamayı temsil eder.
- Bayt toplamları tek başına içeriği, amacı, protokolü veya hedefi tanımlamaz.
- Bir uygulama kimliği bir yol, hizmet, paketlenmiş uygulama veya nesne olabilir; `AppType` değerini
  inceleyin.
- Kullanıcı ve Wi-Fi adları, eşleşen `SOFTWARE` kovanından yapılan zenginleştirmelerdir. Adların
  eksik olması, temeldeki kimliğin bulunmadığı anlamına gelmez.
- `null`, sıfır, bulunmayan sütunlar, desteklenmeyen türler ve hasarlı değerler farklı durumlardır.
- Sağlayıcı şemaları Windows sürümüne göre değişir. Bilinmeyen tabloları ve özgün sütun adlarını
  koruyun.
- Arka plan Windows hizmetleri meşru olarak büyük kaynak toplamları üretir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## Çıktı tabloları { #output-tables }

CSV, `SruDbIdMapTable` için `SRUM-IdentityMap.csv` ve her sağlayıcı için bir tablo oluşturur. Bilinen çıktı adları şunlardır:

| Sağlayıcı/tablo | İnceleme amacı |
|---|---|
| `NetworkUsages` | Uygulama, kullanıcı, arabirim ve profil başına gönderilen/alınan baytlar. |
| `NetworkConnections` | Bağlantı sayısı/süresi türünde etkinlik ve bağlantı başlangıç zamanları. |
| `AppResourceUseInfo` | Ön plan/arka plan CPU, disk, ağ ve ilgili uygulama kullanımı. |
| `AppTimelineProvider` | Bitiş zamanı ve kullanım durumu alanlarıyla uygulama zaman çizelgesi etkinliği. |
| `EnergyUsage` | Kısa ve uzun vadeli enerji/sağlayıcı ölçümleri; `IsLt` bunları birbirinden ayırır. |
| `PushNotifications` | Anında iletme bildirimi etkinliği. |
| `VfuProvider` | `vfuprov` sağlayıcısı; ayrıntılı amacı iyi belgelenmemiştir, bu nedenle alanlarını dikkatle yorumlayın. |
| `EnergyEstimation`, `TaggedEnergy` | Sürüme özgü enerji ve kullanım sağlayıcıları. |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Windows Server sağlayıcı tabloları. |
| Bilinmeyen tablo adı | Juntei bilinmeyen sağlayıcıları GUID/tablo adlarıyla korur ve her katalog sütununu çıktıya yazar. |

SRUM katalog tabanlı olduğundan ve Windows sağlayıcı sütunları eklediğinden, sağlayıcıya özgü
alanların sabit ve sonlu bir listesi kasıtlı olarak yoktur. Juntei, **veritabanında bulunan her
sütunu Juntei'nin PascalCase alan etiketleriyle** çıktıya yazar. Aşağıdaki alanlar kararlı/ortak
çıktı sözleşmesidir.

## Kategorilere ayrılmış auto-parse çıktısı { #categorized-auto-parse-output }

`auto-parse`, her SRUM tablosunu doğrudan ilgili inceleme kategorisine yerleştirir. CSV, JSON ve JSONL
aynı `SRUM-` dosya adı önekini kullanır; kapsayan bir `SRUM/` dizini yoktur.

| Kategori | CSV dosyaları |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`, `SRUM-EnergyEstimation.csv`, `SRUM-TaggedEnergy.csv`, `SRUM-SizeInBytes.csv`, `SRUM-TotalUsed.csv`, `SRUM-IdentityMap.csv`, `SRUM-VfuProvider.csv` ve bilinmeyen sağlayıcı tabloları |

Yalnızca yakalamada bulunan tablolar dışa aktarılır. Ağ sayaçları ve bildirim kayıtları ana inceleme
kullanımlarına göre yerleştirilir; bu, her satırın bir uzak bağlantıyı tanımladığı iddiası değildir.
Kimlik eşlemesi birkaç kategoriyi destekler. VFU ve bilinmeyen sağlayıcılar `SystemInformation`
içinde kalır, çünkü yalnızca varlıkları yürütmeyi kanıtlamaz.

`SystemInformation/SRUM-logs.txt`, ayrıştırıcı tanılamalarını ve özgün çıktıdan nihai çıktıya
eşlemeyi içerir. Her iki bildirim dosyası (manifest) da gerçek tablo yollarını `OutputFiles` içinde
listeler; SRUM görevinin `Output` değeri bilgisayar dizinidir ve `Category`, ortak günlük/meta veri
konumunu belirtir.

Birden çok veritabanı olduğunda, dosya adlarına uzantıdan önce `-Source1`, `-Source2` vb. eklenir.
Günlük, özgün yakalama takma adlarını korur ve JSON/JSONL satırları `SourceFile`, `Table`, `Provider`,
`Comp` ve `Artifact` köken bilgilerini tutar. Hiçbir tablo birden çok kategoriye kopyalanmaz.
Birleştirilmiş `-j` SIEM zaman çizelgesi, korunan her SRUM kaydını bir kez içerir.

Bağımsız `parse-ese srum` komutu istenen çıktı konumunu korur. CSV tablo dosyaları da `SRUM-` önekini
taşır; açıkça belirtilen JSON/JSONL çıktı dosya adları analistin seçtiği gibi kalır.

## Ortak çıktı alanları { #common-output-fields }

| Alan | Yorumlama |
|---|---|
| `SourceFile` | Girdi veritabanı; JSON/JSONL'de bulunur ve ana bilgisayarlar arasında yararlıdır. |
| `Table` | Özgün ESE tablo adı/GUID'i. Sürümleri veya bilinmeyen sağlayıcıları karşılaştırırken koruyun. |
| `Provider` | Kolay anlaşılır sağlayıcı/çıktı adı veya bilinmiyorsa özgün tablo adı. |
| `ID` | Satırın `AutoIncId` değeri; bir işlem kimliği değil, bir veritabanı satır tanımlayıcısıdır. |
| `Timestamp` | SRUM'un değerleri kaydettiği/topladığı zaman, milisaniyeye yuvarlanmış. Genellikle kesin olay zamanı değildir. |
| `EXE-Info` | `AppID` değerinden çözümlenmiş uygulama/hizmet/paket kimliği. Ana program pivotu. |
| `EXE-InfoDescription` | Bileşik SRUM uygulama adlarından çıkarılan açıklama. Yalnızca bağlam içindir. |
| `EXE-Timestamp` | Varsa bileşik bir uygulama kimliğine gömülü zaman; satırın kullanım zamanı değildir. |
| `AppType` | `NormalApp`, `Service`, `ModernApp` veya `Object`; `EXE-Info` değerinin nasıl yorumlanacağını belirler. |
| `SID` | `UserID` değerinden çözümlenmiş hesap SID'i. Güçlü bir kullanıcı/hizmet kimliği pivotu. |
| `User` | `SOFTWARE` kovanından zenginleştirilmiş kolay anlaşılır profil adı; `SID` geçerli kalırken null olabilir. |
| `UserID`, `AppID` | `SruDbIdMapTable` içine ham sayısal başvurular; çözümlenmemiş veya yinelenen adlar için koruyun. |
| `IsLt` | Uzun vadeli EnergyUsage tablosu için `true`, kısa vadeli için `false`. |
| `InterfaceLUID` | Sağlayıcı satırlarından ham ağ arabirimi LUID'i. |
| `InterfaceType` | Kodu çözülmüş arabirim sınıfı (örneğin `IF_TYPE_IEEE80211`); benzersiz bir bağdaştırıcı değil, bir sınıftır. |
| `L2ProfileID` | Sayısal ağ profili başvurusu. |
| `ProfileName` | `SOFTWARE` kovanından zenginleştirilmiş Wi-Fi/profil adı; potansiyel olarak hassastır ve güncel bağlantının kanıtı değildir. |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | UTC'ye dönüştürülmüş bilinen sağlayıcı FILETIME sütunları. Tam anlamları evrensel bir yürütme zamanı değil, sağlayıcıya özgüdür. |
| Sağlayıcıya özgü sütunlar | `BytesSent`, `BytesReceived` (veritabanı sütunu `BytesRecvd`), CPU/disk sayaçları, süreler, sayımlar, bayraklar veya tahminler gibi dışa aktarılan sağlayıcı sütunları. Birimleri sağlayıcıya göre yorumlayın ve satırları aynı şema içinde karşılaştırın. |

## Tanılama alanları (`--include-value-state`) { #diagnostic-fields-include-value-state }

| Alan | Yorumlama |
|---|---|
| `ValueState` | Veritabanı alanından `parsed`, `null`, `absent`, `unsupported` veya `damaged` durumuna eşleme. Bu, kanıt yokluğunu ayrıştırıcı hatasından ayırır. |
| `Raw` | Baytları olan değerler için onaltılık özgün baytlar. Doğrulama ve gelecekte kod çözme için kullanın. |

`SRUM-IdentityMap.csv` (`SruDbIdMapTable` tablosundan) `IdIndex`, `IdType` ve `IdBlob` alanlarını içerir: sağlayıcı satırlarının kullandığı ham indeks, kimlik kategorisi
ve kodu çözülmüş program/SID/nesne metni.

## İnceleme stratejisi { #investigation-strategy }

1. Programı `EXE-Info`, `AppType` ve `AppID` ile belirleyin; `Id` değerini PID ile karıştırmayın.
2. Ağ satırlarını program, kullanıcı, profil, arabirim ve zaman aralığına göre gruplayın.
3. Gönderilen/alınan baytları normal temel değerlerle ve diğer ana bilgisayarlarla karşılaştırın.
4. Şüpheli aralıkları Prefetch, olay günlükleri, DNS/proxy/güvenlik duvarı verileri ve dosya
   erişimiyle ilişkilendirin.
5. Kirli veritabanı durumunu ve eksik işlem günlüğü yeniden oynatmasını bir toplama sınırlaması olarak
   raporlayın.

## Daha fazla ayrıntı { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [Eric Zimmerman'ın SrumECmd aracı](https://github.com/EricZimmerman/Srum)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
