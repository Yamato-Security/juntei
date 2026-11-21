# Microsoft Defender Tespit Geçmişi { #microsoft-defender-detection-history }

## Geçmişin kaydettikleri { #what-the-history-records }

Microsoft Defender her tespit için bir tespit geçmişi dosyası yazar. Her dosya şunları kaydeder:

- tehdidin tanımlayıcısı, adı, önem derecesi ve kategorisi;
- Defender'ın ne yaptığı ve bunun başarılı olup olmadığı;
- tehdidin çalışıyor olup olmadığı;
- ilgili kullanıcı ve süreç ile tehdidi neyin tespit ettiği;
- varsa karantina girdisi;
- tehdidin bulunduğu her kaynak: dosyalar, arşiv üyeleri, indirmeler, süreçler, kayıt defteri değerleri,
  hizmetler, komut satırları ve davranışlar.

Defender bu dosyaları hiç karantinaya almadığı tehditler için de yazar: yalnızca tespit ettikleri, izin
verilenler ve kaldıramadıkları. Windows Güvenliği'nin koruma geçmişi ve WMI'nin
`MSFT_MpThreatDetection` sınıfı (`Get-MpThreatDetection`) bu kayıtları gösterir. Orange
Cyberdefense bir dosyadaki değerleri değiştirdiğinde WMI değiştirilmiş değerleri raporlamıştır.

<span class="evidence-strength">En iyi sonuç:</span> Defender, adı geçen tehdidin listelenen
kaynaklarda tespit edildiğini, kaydedilen durum ve zamanlarla birlikte kaydetmiştir. Tehdit adı ürünün
kararıdır. Bir tespit kaydı tek başına tehdidin çalıştığını veya tamamen kaldırıldığını göstermez.

## Konum ve toplama { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

Her dosya kendi `DetectionID` değerinin adını taşır. `DetectionHistory` klasörünün tamamını toplayın.
Dizin girdisi ve `auto-parse`, bir `DetectionHistory` klasörünün içinde herhangi bir derinlikte bulunan
ve bir tespit geçmişi dosyası gibi başlayan her dosyayı okur. Böyle bir klasörün dışında tutulan bir
kopya için `-f` kullanın. Defender, tarama geçmişindeki öğeleri yapılandırılabilir bir gün sayısından
sonra kaldırır (`Set-MpPreference` içindeki `ScanPurgeItemsAfterDelay`); bu nedenle eski tespitler
silinmiş olabilir.

Karantina ayrı bir artefakttır: bkz. [Defender Karantinası](defender-quarantine.md). Bir tespitin
`QuarantineID` değeri, karantina çıktısındaki `EntryID` ile aynı yazımı (büyük harf, süslü parantezsiz)
kullanır; bu nedenle ikisi birleştirilebilir.

## Analist için uyarılar { #analyst-cautions }

- Tespit, bir düzeltme girişimi ve başarılı kaldırma ayrı durumlardır. `ThreatStatus` değerini
  (`Detected`, `Quarantined`, `Removed`, `Allowed`, `RemoveFailed`, …), son durum değişikliği başarılı
  olduğunda 0 olan `ThreatStatusErrorCode` ile birlikte okuyun.
- `ExecutionStatus`, Defender'ın tespit anındaki görüşüdür (`Blocked`, `Allowed`, `Executing`,
  `NotExecuting`, `Unknown`). Tehdidin çalıştığının da, hiç çalışmadığının da kanıtı değildir.
  Prefetch, Amcache, olay günlükleri ve süreç telemetrisiyle doğrulayın.
- `ThreatName`, o andaki imza adıdır. Hatalı pozitifler ve sonradan yeniden sınıflandırma mümkündür.
  EICAR gibi test dosyaları gerçek tehditler gibi raporlanır.
- `User`, Defender'ın kaydettiği `DomainUser` değeridir (`UserSource` değeri `DomainUser` olur). Boş,
  bir hizmet hesabı veya dosyayı oluşturan kullanıcıdan farklı bir kullanıcı olabilir.
- Defender tek bir tehdit için birkaç dosya yazabilir ve Juntei bunların her birini tutar. Kopyalanmış
  veya geri yüklenmiş bir dosyada olduğu gibi, iki dosya aynı `DetectionID` değerini bile taşıyabilir.
  `DetectionID` ve `SourceFile` alanlarına göre birlikte gruplayın.
- Anlamı bilinmeyen değerler yorumlanmadan saklanır: bölümleri ve konumları `UnknownValues` içindedir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

Varsayılan dosya biçimi CSV'dir. `-t csv`, `-t json` veya `-t jsonl` verilip `-o` verilmezse kayıtlar
stdout'a yazılır. İki seçenek de verilmezse yalnızca özet gösterilir. Var olan çıktının üzerine yazmak
için `-C` ekleyin. Çıktı, `-C` ile bile hiçbir zaman girdi dosyalarından birinin üzerine yazılmaz.

Juntei her tespitin her kaynağı için bir kayıt yazar. Kaynağı olmayan bir tespit, kaynak alanları boş
olan tek bir kayıt alır. Bir tespitin kayıtları onun `DetectionID` değerini paylaşır ve saklandıkları
sıraya göre `ResourceIndex` ile numaralandırılır.

Hasar çalışmayı durdurmaz. Kısa kesilmiş bir dosya, hasardan önce okunan değerleri korur ve `Problems`
neyin eksik olduğunu belirtir. Bir `DetectionHistory` klasöründe bulunan ama tespit geçmişi dosyası
olmayan bir dosya raporlanır ve atlanır. Diğer dosyalar yine de okunur.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Record` | `resource` veya kaynağı olmayan bir tespit için `detection`. |
| `InitialDetectionTime` | Defender'ın tehdidi ilk tespit ettiği zaman (UTC). Dosyada 0 varsa boştur; zaman olmayan bir değer `UnknownValues` içinde saklanır. |
| `LastThreatStatusChangeTime` | Tehdidin durumunun son değiştiği zaman (UTC). |
| `RemediationTime` | Defender'ın tehdidi düzelttiği zaman (UTC). Bir girişimin zamanıdır, kaldırmanın kanıtı değildir; `ThreatStatus` alanını kontrol edin. |
| `ThreatName`, `ThreatID` | Defender'ın imza adı ve sayısal tanımlayıcısı (`Get-MpThreatCatalog`). |
| `Severity`, `SeverityID` | Saklanan sayıyla birlikte `Low`, `Moderate`, `High`, `Severe` veya `Unknown`. |
| `Category`, `CategoryID` | Saklanan sayıyla birlikte `TROJAN`, `TOOL` veya `BEHAVIOR` gibi tehdit kategorisi. |
| `ThreatStatus`, `ThreatStatusID` | Defender'ın tehdit için belirlediği durum. Belgelenmiş bir adı olmayan sayı, boş bir `ThreatStatus` ile saklanır. |
| `ThreatStatusErrorCode` | Son durum değişikliğinin hata kodu; 0 başarılı olduğu anlamına gelir. |
| `ExecutionStatus`, `ExecutionStatusID` | Defender'ın, tespit edildiği anda tehdidi çalışırken görüp görmediği. |
| `DetectionSource`, `DetectionSourceTypeID` | Tehdidi neyin tespit ettiği: `Real-time`, `IOAV` (indirmeler ve ekler), `User`, `System`, `AMSI` ve diğerleri. |
| `User`, `UserSource` | Defender'ın kaydettiği `DomainUser` değeri ve dayanağı (`DomainUser`). Dosyada kullanıcı adı yoksa boştur. |
| `Process` | İlgili süreç (`ProcessName`); `Unknown` olabilir. |
| `ResourceIndex`, `ResourceCount` | Bu kaynağın konumu ve tespitteki kaynak sayısı. |
| `ResourceType` | Defender'ın adlandırdığı şekliyle `file`, `containerfile`, `webfile`, `process`, `regkeyvalue`, `service`, `CmdLine`, `behavior`, `amsi` ve diğerleri. |
| `Path` | Kaynak: bir yol, bir arşiv üyesi (`archive.zip->member`), bir kayıt defteri değeri, bir komut satırı veya bir süreç. Bir `webfile` ayrıca URL'sini ve indiren süreci (`pid:…,ProcessStart:…`) belirtir. Yazımı birebir koruyun. |
| `ResourceFlags` | Onaltılık kaynak bayrakları; anlamları belgelenmemiştir. |
| `TrackingStartTime` | Kaynağın `ThreatTrackingStartTime` değeri; plaso'nun bir tespiti tarihlendirdiği gibi FILETIME (UTC) olarak okunan bir tamsayı. |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | Kaynağın izleme verilerindeki karmalar ve boyut. Dosyayı Defender'ın taradığı andaki hâliyle tanımlar. |
| `TrackingID` | Kaynağın `ThreatTrackingId` değeri. |
| `OtherTracking` | Diğer tüm izleme değerleri, saklanma sırasıyla `Key=Value` biçiminde; örneğin davranış tespitleri için `ThreatTrackingScanSource` veya `ThreatTrackingBMContextRich`. |
| `QuarantineID` | Karantina girdisi (`Quarantine\Entries\{GUID}`); dosyada sıfır GUID varsa boştur. |
| `DetectionID` | Tespitin GUID'i; aynı zamanda dosyanın adı. Uyuşmazlık `Problems` içinde raporlanır. |
| `MagicVersion` | Başlıktaki `Magic.Version:1.2`; bazı Windows Server 2016 dosyaları gibi bu değer olmadan yazılmış dosyalarda boştur. |
| `UnknownValues` | Anlamı bilinmeyen değerler, `position=value` biçiminde (örneğin `6.13=NT AUTHORITY\SYSTEM`); durumdan sonraki sayımlı liste `2.list` olarak verilir. |
| `SourceFile` | Okunan dosya; girdi dizinine göre göreli yolu veya `-f` ile yalnızca adı. |
| `Problems` | Hasar, eksik değerler, beklenmeyen türler veya eşleşmeyen bir dosya adı. Gizlemeyin. |

JSON ve JSONL aynı alanları türlü değerlerle tutar: sayılar sayı olarak kalır ve eksik değerler `null`
olur. `OtherTracking` yerine `Tracking` alanını içerirler; bu alan her izleme değerini Defender'ın
anahtarı, türü ve değeriyle verir. `UnknownValues`, türleriyle birlikte konumlardan oluşan bir liste
olur. `CountedList` sayımlı listeyi tutar. Saklanan bir FILETIME, ham tik sayısını okunduğu zamanın
yanında korur.

## Zaman çizelgesi davranışı { #timeline-behavior }

`auto-parse`, kayıtları `SecurityProducts/DefenderHistory.csv` (veya `.json`/`.jsonl`) olarak kaydeder
ve birleştirilmiş `-j` zaman çizelgesine ekler. `super-timeline`, her kaynağın kaydındaki
`InitialDetectionTime`, `LastThreatStatusChangeTime` ve `RemediationTime` alanlarından olaylar
oluşturur; durum, hata kodu, yürütme durumu ve tespit kaynağı olayın ayrıntılarında yer alır.
`TrackingStartTime` bir tamsayı olarak saklandığından yalnızca `--include-interpreted-times` ile olaya
dönüşür. Dosyanın kendi zamanları hiçbir zaman kullanılmaz. Kullanılabilir zamanı olmayan bir kayıt
ayrıştırılmış çıktıda kalır ve zamanı olmayan kayıt olarak sayılır.

## İnceleme stratejisi { #investigation-strategy }

1. `ThreatName`, `Path`, `SHA256`, `InitialDetectionTime` ve `ThreatStatus` alanlarından başlayın.
2. Bir tehdidin kaldırıldığı sonucuna varmadan önce `ThreatStatusErrorCode` ve `Problems` alanlarını
   kontrol edin.
3. Karantinaya alınan baytları kurtarmak için `QuarantineID` değerini karantinadaki `EntryID` ile
   birleştirin.
4. Karmaları ve yolları `$MFT`, USN Journal, Prefetch, Amcache, tarayıcı geçmişi ve olay günlüklerine
   (Defender'ın operasyonel günlüğü: 1116, 1117, 1118 ve 1119 numaralı olaylar) pivot olarak kullanın.
5. `webfile` kaynaklarında ilk erişimi araştırmak için URL'yi ve süreç başlangıcını kullanın; bunları
   tarayıcı, proxy ve DNS kanıtlarıyla doğrulayın.

## Test edilen sürümler { #tested-versions }

Testler, sonuçlarını başka araçların bağımsız olarak okuduğu herkese açık dosyalar kullanır:

- Orange Cyberdefense'in Windows 10 örneği (Defender 4.18.24090.11); WMI'nin raporladığı değerlerle
  karşılaştırılmıştır.
- plaso'nun iki Windows test dosyası; plaso'nun `windefender_history` ayrıştırıcısıyla
  karşılaştırılmıştır.
- DFIRArtifactMuseum'un APT simülatörü sanal makinesinden 14 dosya.
- Bazı Windows Server 2016 dosyaları gibi, başlıktaki sürüm olmadan yazılmış bir kopya.
- Kısa kesilmiş, boş veya başka türden dosyalar.

## Daha fazla ayrıntı { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [Orange Cyberdefense wddh-parser](https://github.com/cert-orangecyberdefense/wddh-parser)
- [plaso Defender geçmişi ayrıştırıcısı](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: MSFT_MpThreatDetection sınıfı](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: Microsoft Defender Antivirus olay kimlikleri](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
