# Persistence Hunter { #persistence-hunter }

Kalıcılık (persistence), bir programın yeniden başlatma, oturum açma, zamanlanmış bir tetikleyici veya başka
bir olaydan sonra yeniden çalışmasını sağlar. Bir saldırgan bu amaçla normal Windows yapılandırmasını
kullanabilir. Birçok meşru program tam olarak aynı mekanizmaları kullanır; bu nedenle bir kalıcılık girdisi
bulmak analizin başlangıç noktasıdır.

`juntei persistence-hunter`,
[Hexastrike'ın PyrsistenceSniper aracının](https://github.com/hexastrike/PyrsistenceSniper) 131 kontrol
içeren yerel bir Rust uyarlamasıdır: 130 kontrollük PyrsistenceSniper kataloğu artı yerel bir Start Menu COM
kontrolü.

Her mekanizmanın açıklaması, kanıt konumları ve bir bulgunun nasıl inceleneceği için
[131 kontrollük kalıcılık kılavuzuna](../persistence-hunter/index.md) bakın.
[Kalıcılık teşekkürleri](../persistence-hunter/acknowledgements.md) sayfası, kataloğun arkasındaki Python
ve PowerShell projelerini açıklar.

## Kullanım { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## İlk tarama { #first-scan }

```bash
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

Kanıt; bağlanmış bir Windows birimi, bir Velociraptor koleksiyonu veya toplanmış birden fazla bilgisayarı
içeren bir dizin olabilir. Bilgisayarlar analiz sırasında ayrı tutulur ve her sonuç `Comp` alanını içerir.
Windows yolları çözümlenirken toplama sırasında uygulanan dosya adı kaçışlarının kodu çözülür.

Tek bir kovan için:

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

Tek bir kovan, başka bir kovan veya dosya sistemi kanıtı gerektiren kontrolleri sağlayamaz. Boş bir sonucu
anlamlı saymadan önce kapsam raporunu okuyun.

Kontrol kimliklerini ve ATT&CK eşlemelerini listeleyin:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

`--list-checks` tarafından yazdırılan kontrol kimliklerini veya `T1547.001` gibi bir ATT&CK kimliğini
kullanın. `T1547` gibi bir üst kimlik, alt tekniklerini seçer. `--technique`, `--check` için bir takma addır.
Her kontrol, bulgularda tekniğini, açıklamasını ve referans bağlantılarını içerir.

## Paralel taramalar { #parallel-scans }

Komut, varsayılan olarak mantıksal CPU sayısı kadar çalışan içeren paylaşılan bir çalışan havuzu kullanır.
Çalışan sayısını seçmek için `-T, --threads <N>`, tek iş parçacıklı bir tarama için `-T 1` ayarlayın:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

Kanıt keşfi, bağımsız kontroller, yük (payload) meta verileri ve MFT/EVTX zaman çizelgesi kaynakları paralel
çalışır. Bir ana bilgisayarın yüklenen kayıt defteri kovanları salt okunur olarak paylaşılır ve aynı yük için
önbelleğe alınan meta veriler bir kez başlatılır. Kovan belleğinin koleksiyon genelinde katlanmasını önlemek
için ana bilgisayarlar birer birer yüklenir. Bulgular, kapsam, zaman damgası seçimi ve uyarı sırası farklı iş
parçacığı sayılarında aynı kalır.

`auto-parse` zaten ayrıştırıcı işlemlerini paralel çalıştırdığı için her `persistence-hunter` alt işlemine
tek bir çalışan verir. Kendi `-T` seçeneği, eşzamanlı ayrıştırıcı işlemlerinin sayısını denetler.

## Toplanacak kanıtlar { #evidence-to-collect }

| Kaynak | Kontrol örnekleri |
|---|---|
| `SYSTEM` ve yanındaki işlem günlükleri | Hizmetler, hizmet kurtarma komutları, LSA paketleri, önyükleme ayarları, kimlik doğrulama sağlayıcıları |
| `SOFTWARE` ve yanındaki işlem günlükleri | Run anahtarları, Winlogon, IFEO, COM kayıtları, TaskCache, Office ayarları |
| Her kullanıcının `NTUSER.DAT` ve `UsrClass.dat` dosyaları | Kullanıcı otomatik başlatmaları, ortam değişkenleri, COM geçersiz kılmaları, Office ayarları |
| `SAM` ve `SOFTWARE` | RID anormallikleri, gizli hesaplar, olağandışı yerel Administrators üyeliği |
| `Windows/System32/Tasks` | Zamanlanmış görev komutları ve COM işleyicileri; TaskCache ile karşılaştırma |
| Kullanıcı başına Start Menu `.library-ms` dosyaları ve CLSID adlı klasörler | Eşleşen kullanıcı/makine sınıf kayıtlarıyla birleştirilen COM etkinleştirme tetikleyicisi adayları |
| Startup klasörleri, PowerShell profilleri, Office şablonları, Terminal ayarları | Kullanıcı veya sistem tetikleyicileriyle çalıştırılan dosyalar ve komutlar |
| WMI `OBJECTS.DATA` | Olay tüketicisi örnekleri ve filtre-tüketici bağlamaları |
| BITS `qmgr.db` veya `qmgr*.dat` | Kurtarılan bildirim komutları ve sahip SID'leri |
| `$MFT` ve ilgili EVTX dosyaları | Bağımsız zaman damgası ilişkilendirmesi |
| Başvurulan yürütülebilir dosyalar, DLL'ler ve katalog dosyaları | SHA256 karmaları ve tanımlayıcı PE/sertifika meta verileri |

Kayıt defteri işlem günlükleri mevcut olduğunda yeniden oynatılır. `--no-logs`, kovanları toplandıkları
hâliyle okur; yeniden oynatılmamış kirli bir kovan son değişiklikleri içermeyebilir ve bu durum kapsamda
raporlanır. Etkin sistem kontrolleri için geçerli denetim kümesi kullanılır. DSRM arka kapı kontrolü ayrıca,
mevcut olduklarında `ControlSet001`, `ControlSet002` ve `CurrentControlSet` kümelerini inceler; bu ayarı etkin
olarak tanımlamadan önce raporlanan yolunu `SYSTEM\\Select` ile karşılaştırın. Diğer geçmiş denetim kümeleri
tek tek kayıt defteri ayrıştırıcı komutlarıyla incelenebilir.

Kovan okumaları, bilgisayar başına 1 GiB'lık bir girdi bütçesini paylaşır. Makine kovanları kullanıcı
kovanlarından önce denenir; sığmayan girdiler veya işlem günlükleri kısmi kapsamda adlandırılır. Okunabilen
diğer kovanlar ve dosyaya dayalı kontroller devam eder. Kovan kimliğini yalnızca standart kovan konumları veya
açıkça sağlanan dışa aktarımlar belirler; bir kullanıcının geçici dizinindeki bir kopya o kullanıcının gerçek
kovanının yerini alamaz. SOFTWARE kovanında kaydedilen özgün sistem sürücüsü, toplanan sistem birimine
eşlenir; böylece özgün bir `C:` birimini farklı bir kanıt sürücüsü etiketi altında bağlamak yük meta
verilerinin atılmasına neden olmaz.

`auto-parse` da avcıyı çalıştırır ve her bilgisayar dizininin içine `persistence/PersistenceHunter.csv`
(veya `.json`/`.jsonl`) dosyasını, yanına da `PersistenceHunter-logs.txt` dosyasını kaydeder. Bulguları ayrıca
isteğe bağlı `-j <FILE>` SIEM zaman çizelgesine de girer. Otomatik ayrıştırma, izin kurallarıyla eşleşen
girdiler dahil olmak üzere eksiksiz yapılandırma envanterini korumak için `--min-severity info` kullanır.

## COM kayıtları ve etkinleştirme tetikleyicileri { #com-registrations-and-activation-triggers }

`scheduled_task_files` kontrolü, görev XML'inin `Actions Context` değerini ve eşleşen `Principal id`
değerini kullanarak her eylemin sorumlusunu (principal) seçer. Bir grup sorumlusu grup olarak korunur;
hangi bireysel kullanıcının sınıf kayıtlarının kullanılacağını belirleyemez.

COM işleyicisi adayları, yerel ve 32 bit sınıf görünümlerinde ayrı ayrı çözümlenir. Yalnızca eşleşen
kullanıcının toplanmış `UsrClass.dat`/`NTUSER.DAT` dosyaları ve makinenin `SOFTWARE` kovanı dikkate
alınır. SID sorumluları, toplanmış `ProfileList` yolları üzerinden birleştirilir. `HOST\user` ve
`DOMAIN\user` dahil olmak üzere hesap adlarında, düz bir `user` profili yerine eşleşen bir `user.DOMAIN`
profili tercih edilir. Yerel ana bilgisayar takma adları ve `.\user`, sonek eşleştirmesi için toplanmış
ana bilgisayar adlarını kullanır; tam bir sonek diğer ana bilgisayar takma adlarına göre önceliklidir. Düz
profil yalnızca hiçbir nitelikli profil eşleşmediğinde dikkate alınır. Bu tür her eşleşme `Description`
içinde açıkça bir **klasör adı buluşsal yöntemi** olarak etiketlenir; hesap sahipliğini veya etki alanı
üyeliğini kanıtlamaz. Belirsiz eşleşmeler çözümsüz kalır ve hem bulgu açıklamasında hem de kapsamda
raporlanır; nitelikli profiller arasındaki belirsizlik, düz profile geri dönüşü tetiklemez. Bu, normalde
yalnızca bellekte bulunan ve kayıt defteri kovanlarıyla birlikte kaydedilmeyen `Volatile Environment`
anahtarına bağlı değildir. Yinelenen bir kullanıcı CLSID'si, makine CLSID'sinin alt öğelerini gölgeler. Grup
sorumluları makine COM adaylarını kullanır ve bu sınırlamayı bulgu açıklamasında korur; normal bir grup
sorumlusu kendi başına kapsamı kısmi yapmaz veya hata özeti girdilerini tüketmez.

Çözümleyici `InprocServer32` ve `LocalServer32` değerlerini izler, yönetilen `CodeBase` yollarını (sürüm alt
anahtarları dahil) kullanır ve AppID `LocalService` değerini toplanan SYSTEM hizmetinin `ImagePath` değeriyle
birleştirir. AppID yürütülebilir dosya adı ilişkilendirmeleri korunur, ancak bir temel ad, bir yürütülebilir
dosyanın tam yolunu belirlemez. Yerel `file:` CodeBase URL'lerinin kodu çözülür; uzak URL'ler onlara
erişilmeden kaydedilir. Eksik kayıtlar CLSID'yi `ResolutionStatus: unresolved_com_class`, boş yol/karma
alanları ve bilinmeyen `Exists` ile korur.

`start_menu_com` kontrolü, kullanıcı başına Start Menu `.library-ms` XML URL'lerini okur ve boş dizinler
dahil CLSID adlı klasörleri toplar. Tetikleyici yolunu, CLSID'yi, aday sunucuyu ve kayıt defteri kökenini
raporlar. Yalnızca dosya içeren bir edinim boş bir dizini koruyamaz ve eksik kullanıcı kovanları kullanıcı
başına eksiksiz bir kayıt aramasını engeller. Sembolik bağlantılar izlenmez. Bunlar yapılandırma adaylarıdır:
mimari, yükseltme, UAC, ilke ve etkinleştirme bağlamı bir kaydın gerçekten kullanılıp kullanılmadığını
etkileyebilir. Yürütmeyi işlem, modül yükleme veya diğer bağımsız kanıtlarla doğrulayın.

## Sınıflandırma ve profiller { #classification-and-profiles }

Varsayılan en düşük sınıflandırma `medium` değeridir. Paketle gelen izin listesiyle eşleşen girdiler dahil
tam envanteri incelemek için:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "Eksik koleksiyonları incelerken tüm önem düzeylerini kullanın"
    Geniş bir izin kuralı, imzalayan eksik veya farklı olsa bile bir saldırganın komutuyla kısmen
    eşleşebilir. Upstream, kısmi eşleşmeleri `LOW` olarak sınıflandırır; elle çalıştırılan komutun
    varsayılan `medium` eşiği bunları gizler. Bu durum Windows dizinleri altındaki kötü amaçlı zamanlanmış
    görevleri etkileyebilir. Eksiksiz bir inceleme için `--min-severity info` kullanın ve `PolicyReason`,
    `SignerStatus`, komutu ve kaynak kanıtını birlikte gözden geçirin. `auto-parse` bu girdileri zaten korur.

| Sınıflandırma | Anlamı |
|---|---|
| `HIGH` | Yapılandırılmış bir engelleme kuralı tamamen eşleşti |
| `MEDIUM` | Hiçbir izin veya engelleme kuralı eşleşmedi |
| `LOW` | Bir izin kuralının kesin koşulları eşleşti, ancak imzalayan meta verileri eşleşmedi veya mevcut değildi |
| `INFO` | Bir izin kuralı tamamen eşleşti |

Bunlar ilke sınıflandırmalarıdır; ölçülmüş ele geçirilme olasılıkları değildir. Varsayılan bir `MEDIUM`
girdisi kötü amaçlı yazılımın kanıtı değildir. `INFO`, bir dosyanın geçerli bir imzaya sahip olduğunu veya bir
saldırganın onu kötüye kullanmadığını kanıtlamaz. Kısmi bir izin eşleşmesi varsayılan eşiğin altındadır; bu
nedenle seyrek koleksiyonları incelerken veya izin listesini ayarlarken `--min-severity low` veya `info`
ekleyin.

`--profile profile.yaml` ile bir YAML profili sağlayın. Bu, paketle gelen profilin **yerini alır**. Seçilen
dosyadaki genel ve kontrol başına kurallar birleştirilir; tamamen eşleşen herhangi bir engelleme kuralı
kazanır. Örneğin:

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

Desteklenen kural koşulları `path_matches`, `value_matches`, `hash` (64 onaltılık basamaklı SHA256),
`not_lolbin` ve `signer` (büyük/küçük harf duyarsız alt dize) koşullarıdır. Belirtilen tüm kesin koşullar
eşleşmelidir. İmzalayan meta verileri, yukarıda açıklanan esnek koşuldur. Düzenli ifadeler Rust'ın düzenli
ifade sözdizimini kullanır: ileri/geri bakışlar (lookaround) ve geri başvurular desteklenmez. Bilinmeyen
kontrol kimlikleri, geçersiz ifadeler ve bilinmeyen profil alanları, ilkeyi sessizce zayıflatmak yerine hata
üretir.

## Çıktı alanları { #output-fields }

CSV varsayılan dosya biçimidir. Okunabilir JSON nesneleri için `-t json`, satır başına bir nesne içeren
sıkıştırılmış çıktı için `-t jsonl` kullanın. `-o` olmadan, açıkça verilen bir `-t` stdout'a yalnızca veri
yazar; kapsam uyarıları stderr'e gider. Her iki seçenek de olmadan Juntei bir özet ve en fazla 30 aday yazdırır.

| Alanlar | Nasıl yorumlanır |
|---|---|
| `Comp`, `CheckID`, `Technique`, `MitreID` | Satırın hangi bilgisayarı ve kalıcılık mekanizmasını tanımladığı |
| `Path`, `Value` | Kayıt defteri konumu, dosya, görev veya kurtarılan kayıt ve yapılandırılmış içeriği |
| `Description`, `References` | Mekanizmanın neden önemli olduğu ve daha fazlasının nerede okunabileceği |
| `AccessGained` | Mekanizmanın olağan kapsamı (`USER` veya `SYSTEM`; BITS ayrıca `LOCAL_SERVICE`/`NETWORK_SERVICE` belirleyebilir); belirli bir komutun bu ayrıcalığa ulaştığının kanıtı değildir |
| `Severity`, `PolicyReason` | Profil sınıflandırması, eşleşen ilke açıklaması ve varsa kısmi eşleşme veya doğrulanmamış imzalayan nitelemesi |
| `SourceFile`, `RecordStatus` | Toplanan kaynağın tam kendisi, verildiği hâliyle girdiyle başlayan yolu (auto-parse'tan `./HOST/...`) ve ayırma/yeniden oynatma/kurtarma bağlamı; bunu özgün kanıta kadar geri izleyin |
| `KeyLastWrite` | Kayıt defteri **anahtarının** zaman damgası. Tek bir değeri tarihlendirmez |
| `LastChange`, `ChangeSource`, `ChangeEvidence` | Bağımsız olarak ilişkilendirilmiş değişiklik zamanı, kaynak kanıtı ve ilişkilendirmenin başarılı olup olmadığı |
| `ResolvedPath`, `ResolutionStatus`, `Exists` | Yük yolu ve çevrimdışı çözümleme sonucu; toplanmamış kanıt `false` değil, bilinmeyendir |
| `SHA256` | Okunabildiğinde toplanan yük baytlarının SHA256 değeri |
| `Signer`, `SignerStatus` | Tanımlayıcı gömülü sertifika/katalog meta verileri ve mevcudiyeti; sertifika güveni ve imza geçerliliği doğrulanmaz |
| `IsLolbin`, `Launcher` | Yapılandırılmış başlatıcının veya yükün paketle gelen LOLBAS listesinde olup olmadığı ve vekil bir yük için kullanılan başlatıcı |
| `IsBuiltin`, `IsInOS-Directory` | Yaygın Windows komutları ve sistem dizinleri için bağlam; hiçbir alan meşruiyeti kanıtlamaz |

Bir `rundll32.exe` komutu bir DLL yükünü adlandırabilir. Juntei, yükü çözümleyip karmasını alırken
başlatıcıyı da korur. Toplanan bilgisayarın dışındaki bir referans, eksik bir sürücü, belirsiz bir büyük/küçük
harf duyarsız eşleşme veya çözümlenmemiş bir ortam değişkeni, analistin makinesindeki bir dosyayla
değiştirilmez.

## Zaman damgaları ve sınırlamalar { #timestamps-and-limitations }

Avcı, analist sanal makinesindeki kopyalanmış dosyaların değiştirilme zamanlarını saldırı zaman damgaları
olarak kullanmaz. Mevcut olduğunda kayıt defteri yazmalarını Sysmon 13 olayıyla, görev kurulumunu Security
4698 veya Task Scheduler 106 ile, hizmet kurulumunu System 7045 ile ve dosyaya dayalı yapılandırmayı özgün
`$MFT` zaman damgalarıyla ilişkilendirir. Köken bilgisi `ChangeSource` içinde görünür. Bir olay, yapılandırılmış
yükün çalıştığını kanıtlamadan bir yapılandırma değişikliğini tarihlendirebilir. Yerel System 6011 olayı,
önceki bir bilgisayar adını kanıtlayabilir. Avcı, toplanan bilgisayara bağlı doğrulanmış yeniden adlandırma
zincirlerini izler; geçerli `Comp` değerini korurken özgün olay bilgisayarını ve yeniden adlandırma kaydı
kanıtını `ChangeSource` içine kaydeder. Kanıtlanmamış yabancı adlar, arşivlenmiş günlükler, iletilmiş
günlükler ve başka bir Windows kurulumu bu kural aracılığıyla kabul edilmez.

Toplanmış ComputerName/ActiveComputerName, TCP/IP Hostname/NV Hostname ve istemci bilgisi
hostname/FQDN değerleri, DNS/NetBIOS farklılıklarını ve bekleyen yeniden adlandırmaları kapsayan ana bilgisayar
takma adlarını sağlar. Toplanmış bir kimlik yoksa, birincil günlüklerdeki tek bir ad, açık bir
doğrulanmamış kimlik kökeni ve bir kapsam uyarısıyla tarihlendirmeyi destekleyebilir. Birden fazla
doğrulanmamış ad ana bilgisayara atfedilmez. Reddedilen Computer adları bir ilişkilendirme kapsamı
sınırlaması olarak raporlanır; kanalları yine de toplanmış olarak işaretlenir.

Tek bir bilgisayar için harici olarak toplanmış bir MFT seçmek üzere `--mft <FILE>` kullanın. Zaman damgası
ilişkilendirmesini atlamak için `--no-timeline` kullanın. `NOT_RUN`, `NOT_APPLICABLE`, `NO_ARTIFACT`, `NO_MATCH`
ve `REJECTED`, boş bir `LastChange` alanını açıklar; boş bir alan, girdinin zararsız olduğunun kanıtı değildir.

WMI ve BITS kurtarması, eksiksiz bir veritabanı işlem ayrıştırıcısı değil, sınırlandırılmış bir oymadır
(carving). Kurtarılan dizeler veya bağlamalar eski/silinmiş veriler içerebilir; bu kayıtlar doğrulanmalıdır.
WMI sınıf şemaları tek başına abonelik olarak sayılmaz. Toplanmış bir görev XML dosyası olmayan bir TaskCache
girdisi incelenecek bir tutarsızlıktır; bir triyaj koleksiyonu bu dosyayı basitçe dışarıda bırakmış olabilir.
Katalog ve PE meta verileri doğrulanmamıştır ve bir koleksiyonda bulunmayabilir. Erişilebilirlik dosyası
değiştirme kontrolleri uygun referans imza meta verilerine bağlıdır; eksik katalog kanıtı, her erişilebilirlik
yürütülebilir dosyasının işaretlenmesine yol açmamalıdır.

Sıfır bulgulu kontroller dahil her kontrol için kapsam kaydedilir. `complete`, ilgili toplanmış kaynağın
okunduğu anlamına gelir; koleksiyonun eksiksiz olduğunu veya bilgisayarda başka bir kalıcılık bulunmadığını
kanıtlamaz. `partial`, `not_collected`, `failed` ve `disabled` sınırları açıklar. `--coverage Coverage.csv`
dosyasını ve ayrıştırıcı günlüğünü bulgularınızla birlikte saklayın.

Bu uyarlama Juntei'nin CSV/JSON/JSONL biçimlerini kullanır. Upstream Python programını çağırmaz, çevrimiçi
zenginleştirme hizmetlerini sorgulamaz veya upstream HTML/XLSX arayüzünü yeniden üretmez.

## Daha fazla ayrıntı { #further-details }

- [PyrsistenceSniper uygulaması ve profilleri](https://github.com/hexastrike/PyrsistenceSniper)
- [MITRE ATT&CK kalıcılık teknikleri](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft: Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [Kayıt defteri otomatik başlatma envanteri](../artifacts/persistence.md)
- [Hizmetler](../artifacts/services.md) ve [TaskCache](../artifacts/taskcache.md)
