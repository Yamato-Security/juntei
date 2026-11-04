# Persistence Hunter { #persistence-hunter }

Kalıcılık (persistence), kodun yeniden çalışmasını sağlamanın bir yoludur: Windows başladığında, bir kullanıcı oturum açtığında, bir uygulama
açıldığında, zamanlanmış bir saat geldiğinde veya başka bir olay gerçekleştiğinde. Saldırganlar, meşru yazılımların güncellemeler, aygıt desteği, güvenlik ve
yönetim için kullandığı ayarların aynısını kötüye kullanabilir.

`juntei persistence-hunter`, toplanmış Windows kanıtlarını çevrimdışı olarak bu mekanizmaları ve ilgili hesap/koruma
anormalliklerini kapsayan **131 kontrol** için inceler. Kayıt defteri kovanlarını, görev dosyalarını,
betikleri, uygulama ayarlarını ve seçili veritabanı kayıtlarını okur.

Bu sayfa uygulanan her kontrolü açıklar. Seçeneklerin, çıktı alanlarının, profillerin ve
zaman damgası işlemenin tamamı için [komut kılavuzunu](../commands/persistence-hunter.md) açın.
[Teşekkürler](acknowledgements.md) sayfası Python PyrsistenceSniper projesine ve özgün
PowerShell PersistenceSniper aracına bağlantı verir.

## Bir incelemeye başlayın { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

Eksiksiz aday envanteri için `--min-severity info` kullanın. Elle çalıştırılan komutun varsayılanı
`medium` değeridir; profili, eksik yük (payload) veya imzalayan kanıtı tam bir karşılaştırmayı engellese bile kısmi izin listesi eşleşmelerini `LOW` olarak
sınıflandırabilir. Ne bir izin listesi eşleşmesi ne de tanıdık olmayan bir girdi, bir şeyin kötü amaçlı olup olmadığını
kesinleştirir. `auto-parse` zaten tüm önem düzeylerini korur ve bulguları
her bilgisayarın `persistence/` kategorisi altına yazar.

Bir mekanizmaya odaklanmak için aşağıdan ilgili **Kontrol kimliği** değerini kopyalayın:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

Komut varsayılan olarak paylaşılan bir çalışan havuzu kullanır. Dört çalışan seçmek için `-T 4`, tek iş parçacıklı
tarama için `-T 1` ekleyin. Girdi birkaç bilgisayar içerebilir; kanıtları birbirinden ayrı kalır.

## Bulguları kanıt olarak okuyun { #read-findings-as-evidence }

| Soru | Neyi incelemeli |
|---|---|
| Ne, nerede yapılandırılmış? | `CheckId`, `Path` ve bağımsız değişkenler dahil eksiksiz `Value`. `SourceFile` üzerinden toplanmış kovana, dosyaya veya veritabanına gidin. |
| Hedef toplamada mevcut mu? | `ResolvedPath`, `ResolutionStatus` ve `Exists`. Toplanmamış, belirsiz veya desteklenmeyen başvurular bilinmeyen olarak kabul edilir; Juntei analistin makinesindeki bir dosyayı onların yerine koymaz. |
| Yapılandırma etkili mi? | Gerçek tetikleyici, etkinleştirme bayrakları, seçili denetim kümesi, Windows/uygulama sürümü, kullanıcı ve mimari. Bazı kontroller etkin olmayan yapılandırmayı bilinçli olarak korur. |
| Bu bir saldırı mı? | `Severity` ve `PolicyReason` bir hüküm değil, profil eşleşmesini açıklar. Beklenen yazılımla, tam komutla, dosya karmalarıyla, bağımsız yürütme kanıtlarıyla ve çevredeki zaman çizelgesiyle karşılaştırın. |
| Bunu kim çalıştırabilir? | `AccessGained` bir kapsam/bağlam ipucudur. Makine genelindeki yapılandırma çoğunlukla `SYSTEM` der, ancak bu, başlatılan işlemin SYSTEM olarak çalıştığını kanıtlamaz. Görev sorumluları (principal) ve BITS sahibi SID'leri, mevcut olduğunda daha belirli bir bağlam sağlar. |
| Ne zaman değişti? | `KeyLastWrite` kayıt defteri anahtarının bütününü tarihlendirir. `LastChange`, `ChangeSource` ve `ChangeEvidence`, mevcut olduğunda ayrı MFT/EVTX ilişkilendirmesini açıklar. Bu alanların hiçbiri yükün yürütülmesini otomatik olarak tarihlendirmez. |
| Kurtarılan durum ne kadar güvenilir? | Yeniden oynatılmış, kazınmış, hasarlı, eksik veya sınırlı kanıtlar için `RecordStatus`, `Coverage.csv` ve uyarıları okuyun. Kurtarılmış geçmiş bir kayıt artık etkin olmayabilir. |
| Adı geçen bir yayımcı güvenilir mi? | `Signer` ve `SignerStatus` açıklayıcı PE/sertifika/katalog meta verileridir. Juntei, Authenticode imzalarını veya sertifika güvenini doğrulamaz. |

Bulgu olmaması, kalıcılık olmadığını kanıtlamaz. `complete` durumundaki bir kapsam satırı,
ilgili toplanmış kaynağın okunduğu anlamına gelir; edinimin her ilgili dosyayı,
her kullanıcıyı veya olası her Windows kalıcılık mekanizmasını içerdiği anlamına gelmez. Diğer kapsam durumları eksik,
kısmi, devre dışı bırakılmış veya başarısız kontrolleri açıklar. Bu avcı (hunter) bir canlı bellek, ağ veya bulut kimliği taraması değildir.

## Katalog ve konum gösterimi { #catalogue-and-location-notation }

Katalog PyrsistenceSniper'a ve buna ek olarak yerel `start_menu_com` kontrolüne dayanır;
aşağıdaki açıklamalar Juntei'nin kanıt sınırlamaları da dahil olmak üzere
kendi yerel davranışını belgeler. **116 kayıt defteri tabanlı
kontrol ve 15 dosya/depo kontrolü** vardır. Bazı dosya kontrolleri kayıt defteri verilerini de birleştirir; bu nedenle bunlar
birbirini dışlayan edinim gereksinimleri değil, uygulama sayılarıdır. Mekanizmalar
aşağıda inceleme amacına göre gruplandırılmış ve her grup içinde Kontrol kimliğine göre alfabetik olarak sıralanmıştır.

- `HKLM\SYSTEM`, `HKLM\SOFTWARE` ve `HKLM\SAM`, normalde
  `Windows\System32\config` konumundan, mevcutsa işlem günlükleriyle birlikte toplanan makine kovanlarını ifade eder.
- `HKU\<user>`, ilgili toplanmış kullanıcı kovanı anlamına gelir. Kullanıcı ayarlarının çoğu NTUSER.DAT dosyasından gelir;
  sınıf kayıtları belirtildiği gibi `AppData\Local\Microsoft\Windows\UsrClass.dat` dosyasından gelebilir.
- `<ControlSet>`, SYSTEM içindeki seçili etkin denetim kümesi anlamına gelir. DSRM kontrolünün ek
  geçmiş konumları aşağıda açıkça belirtilmiştir. Bunları incelerken `SYSTEM\Select` değerini kontrol edin.
- `<name>` değişken bir yol bileşenidir. Kayıt defteri hedef tablolarında `<child>`, doğrudan
  alt kayıtlar anlamına gelir; sınırsız özyinelemeli bir dolaşma anlamına gelmez. Açıkça özyinelemeli olan kontroller
  dolaşmalarını ayrıca açıklar. Office yollarındaki köşeli parantezler isteğe bağlı bir sürümü belirtir.
- Tablolar, kontrol tarafından seçilen gerçek makine ve kullanıcı görünümlerini listeler. 32 bit görünüm,
  uygulandığı yerlerde listelenir; her kontrolün olası her mimariyi/görünümü incelediğini varsaymayın.
- Genel değer kontrolleri, uygun olduğunda çoklu dize öğeleri ve sayısal sıfır da dahil olmak üzere boş olmayan yapılandırılmış girdileri
  çıktı olarak verir. Kendi başlarına kötü amaçlı bir yük, etkin bir
  tetikleyici veya başarılı bir yürütme gerektirmezler. Daha seçici koşullar her kontrol için ayrıca belirtilir.

ATT&CK kimlikleri, katalogdan devralınan eşlemeler ve daha fazla çalışma için bağlantılardır. Bir saldırganın
o tekniği gerçekleştirdiğine dair bir bulguyu değil, olası kötüye kullanımı açıklarlar; koruma ayarı ve tanılama
kontrolleri, bağımsız bir kalıcılık mekanizması yerine bunu mümkün kılan koşulları açıklayabilir.

| Mekanizma grubu | Kontroller |
|---|---:|
| [Önyükleme, oturum açma ve başlangıç](#boot-sign-in-and-startup) | 32 |
| [Hizmetler ve sistem uzantıları](#services-and-system-extensions) | 13 |
| [Zamanlanmış görevler](#scheduled-tasks) | 3 |
| [WMI, BITS ve diğer olay tetikleyicileri](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorer, COM ve dosya veya URL işleyicileri](#explorer-com-and-file-or-url-handlers) | 15 |
| [Office uygulamaları ve eklentileri](#office-applications-and-add-ins) | 7 |
| [Hata ayıklayıcılar, çökme işleyicileri ve uyumluluk shim'leri](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [.NET çalışma zamanı ve profil oluşturucu ayarları](#net-runtime-and-profiler-settings) | 7 |
| [DLL ve yürütülebilir dosya arama geçersiz kılmaları](#dll-and-executable-lookup-overrides) | 18 |
| [Hesaplar ve oturum açma görünürlüğü](#accounts-and-sign-in-visibility) | 4 |
| [Kimlik doğrulama bileşenleri ve kimlik bilgisi koruması](#authentication-components-and-credential-protection) | 8 |
| **Toplam** | **131** |

## Önyükleme, oturum açma ve başlangıç { #boot-sign-in-and-startup }

Bu ayarlar, önyükleme, oturum açma veya masaüstü başlatma sırasında bir komutun veya kitaplığın çalışmasını sağlar. Windows sürümünü ve ilgili kullanıcıyı inceleyin: bir ayar eskimiş, devre dışı bırakılmış veya bir saldırgan yerine bir kiosk için tasarlanmış olabilir.

### Active Setup {#active_setup}

**Kontrol kimliği:** `active_setup` · **Katalog ATT&CK:** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**Juntei neyi kontrol eder:** Yerel ve 32 bit görünümlerdeki makine genelindeki Active Setup bileşen kayıtlarından `StubPath` değerini okur. Windows bu komutları bir kullanıcının profilini hazırlarken kullanabilir; kontrol, `U` ve `/UserInstall` sabit yer tutucu değerlerini atlar.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` ve karşılık gelen `Wow6432Node` ağacı.

**Nasıl incelenir:** Her bileşeni yüklü ürünüyle karşılaştırın ve komut bağımsız değişkenlerini inceleyin. Makine genelindeki bir kayıt, oturum açan bir kullanıcının oturumunda yürütmeye yol açabilir; çıktıdaki `SYSTEM` etiketi yapılandırma kapsamını açıklar, ortaya çıkan işlem belirtecinin (token) kanıtı değildir.

### Boot Execute {#boot_execute}

**Kontrol kimliği:** `boot_execute` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Session Manager `BootExecute` çoklu dizesini okur ve boş olmayan her girdiyi ayrı ayrı korur. Erken bir önyükleme aşaması için yapılandırılmış komutları kaydeder.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**Nasıl incelenir:** `autocheck autochk *` bilinen bir varsayılandır ve disk araçları meşru girdiler ekleyebilir. Varsayılan olmayan her girdinin çalıştığını veya kötü amaçlı olduğunu varsaymadan eklenmiş komutları ve dosyalarını inceleyin.

### Boot Verification Program {#boot_verification_program}

**Kontrol kimliği:** `boot_verification_program` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Önyükleme doğrulamasıyla ilişkili bir yürütülebilir dosya konumu olan `BootVerificationProgram` anahtarının `ImagePath` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**Nasıl incelenir:** Bu eski mekanizmanın toplanmış Windows sürümüne ve önyükleme yapılandırmasına uygulanabilir olup olmadığını kontrol edin. Hedef dosyayı çözümleyin ve amacını beklenen önyükleme yönetimi yazılımıyla karşılaştırın.

### Explorer Load Değeri {#explorer_load}

**Kontrol kimliği:** `explorer_load` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Makine ve kullanıcı kovanlarından Windows oturum açma başlatma yapılandırmasıyla ilişkili `Load` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**Nasıl incelenir:** Tanıdık olmayan komutları, kullanıcının yazabildiği hedefleri ve içerik indiren veya kodunu çözen bağımsız değişkenleri inceleyin. Uygulanabilirliği ve yürütmeyi ayrıca doğrulayın; boş bir sonuç diğer oturum açma mekanizmalarını kapsamaz.

### Font Drivers {#font_drivers}

**Kontrol kimliği:** `font_drivers` · **Katalog ATT&CK:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei neyi kontrol eder:** Eski `Font Drivers` kayıt defteri anahtarındaki boş olmayan değerleri olası kitaplık kayıtları olarak listeler.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Bunu olağandışı, eski bir yapılandırma yüzeyi olarak ele alın. Yüklenebileceği sonucuna varmadan önce Windows sürüm desteğini ve başvurulan kitaplığı kontrol edin ve gerçek kullanımı işlem veya görüntü yükleme (image load) kanıtlarıyla doğrulayın.

### Grup İlkesi Uzantı DLL'leri {#gp_extension_dlls}

**Kontrol kimliği:** `gp_extension_dlls` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Winlogon altındaki her doğrudan Grup İlkesi uzantı kaydından `DllName` değerini okur. Bu DLL'ler ilke işlemeyi genişletir.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**Nasıl incelenir:** Yönetim ve güvenlik ürünleri uzantı yükleyebilir. Uzantı GUID'ini ve DLL'i beklenen ürünle eşleştirin ve yürütme olduğunu öne sürmeden önce ilke işleme ve görüntü yükleme kanıtlarını inceleyin.

### Grup İlkesi Betikleri {#gp_scripts}

**Kontrol kimliği:** `gp_scripts` · **Katalog ATT&CK:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei neyi kontrol eder:** Kullanıcı başına yerel ilke depoları da dahil olmak üzere, toplanmış yerel Grup İlkesi `scripts.ini` ve `psscripts.ini` kayıt dosyalarından komut/parametre çiftlerini ayrıştırır. Başlangıç/kapanış ve oturum açma/kapatma bölümleri yapılandırılmış komutlarını ve bağımsız değişkenlerini korur; makine bölümleri kullanıcı bölümlerinden ayrılır.

**İncelenen kanıtlar:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` ve eşdeğer `GroupPolicy\User\Scripts` dosyaları.
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`.

**Nasıl incelenir:** Yalnızca uygulanmış yerel ilke kayıt konumları incelenir; bu eksiksiz bir SYSVOL/GPO denetimi değildir. Bir ilke dizini altında başka bir yerde duran bir betik dosyası yeterli değildir. Kaydını ilke işleme ve yürütme kanıtlarıyla ilişkilendirin.

### Oturum Açma Betikleri (UserInitMprLogonScript) {#logon_scripts}

**Kontrol kimliği:** `logon_scripts` · **Katalog ATT&CK:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei neyi kontrol eder:** Kullanıcının oturum açması için bir betik seçebilen `UserInitMprLogonScript` ortam değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**Nasıl incelenir:** Onaylı oturum açma betikleriyle karşılaştırın ve asıl betiği inceleyin. Değer, sonraki bir oturum açmayı veya yürütmeyi kanıtlamaz; etki alanı tarafından sağlanan betikler ek ilke kanıtı gerektirir.

### Platform Execute {#platform_execute}

**Kontrol kimliği:** `platform_execute` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Session Manager'daki boş olmayan her `PlatformExecute` girdisini okur. Platformla ilgili önyükleme başlatması için seçilmiş programları belirler.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**Nasıl incelenir:** Donanım veya hazırlama (provisioning) iş akışları bir girdiyi açıklayabilir. Bu Windows derlemesindeki önyükleme yolunun bunu kullanıp kullanmadığını belirleyin ve değer adından bir saldırı çıkarmak yerine başvurulan görüntüyü inceleyin.

### RDP İstemci Uzantısı DLL'i {#rdp_clx_dll}

**Kontrol kimliği:** `rdp_clx_dll` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Terminal Server'ın varsayılan kullanıcı yapılandırmasındaki `ClxDllPath` değerini olası bir RDP ile ilgili uzantı konumu olarak okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**Nasıl incelenir:** Windows/RDP sürümünü ve yüklü uzantıları kontrol edin. Yapılandırılmış bir DLL, görüntü yükleme analizi için bir ipucudur; kayıt defteri değeri tek başına bir RDP bağlantısının veya DLL yüklemesinin gerçekleştiğini göstermez.

### RDP Sanal Kanal DLL'i {#rdp_virtual_channel}

**Kontrol kimliği:** `rdp_virtual_channel` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Makine genelindeki Terminal Server Client `Default\AddIns` kayıtlarının altındaki değerleri listeler. İkili değerlerin kodu mümkün olduğunda UTF-16LE olarak çözülür ve kullanılabilir uzantı yolları korunur.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: tüm değerler.

**Nasıl incelenir:** RDP istemcileri ve aygıt yeniden yönlendirme ürünleri meşru olarak eklenti kaydeder. Asıl DLL'i veya komutu ve ilgili istemci oturumunu inceleyin; bir eklentinin altındaki her değer yürütülebilir bir yol değildir.

### RDP WDS Başlangıç Programları {#rdp_wds_startup}

**Kontrol kimliği:** `rdp_wds_startup` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** RDP Windows Station sürücüsü yapılandırmasından `StartupPrograms` değerini okur. Uzak oturum başlangıcı için yapılandırılmış programları belirler.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**Nasıl incelenir:** `rdpclip` bilinen bir girdidir. Eklemeleri ve ana bilgisayarın RDP yapılandırmasını inceleyin; yapılandırılmış programın gerçekten başlayıp başlamadığını belirlemek için oturum/işlem kayıtlarını kullanın.

### Kayıt Defteri Run Anahtarları {#run_keys}

**Kontrol kimliği:** `run_keys` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Makine/kullanıcı kovanlarındaki Run, RunOnce, RunEx, RunOnceEx ve ilke Explorer Run değerlerini, ayrıca listelenen makine 32 bit anahtarlarını listeler. RunEx/RunOnceEx bölümleri sınır ve döngü kontrolleriyle dolaşılır.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Boş olmayan tüm değerler |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Boş olmayan tüm değerler |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Boş olmayan tüm değerler |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Boş olmayan tüm değerler |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Boş olmayan tüm değerler |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Boş olmayan tüm değerler |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Boş olmayan tüm değerler |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Boş olmayan tüm değerler |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Boş olmayan tüm değerler |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Boş olmayan tüm değerler |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | Boş olmayan tüm değerler |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | Boş olmayan tüm değerler |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Yazılım güncelleyicileri ve sistem tepsisi uygulamaları burada sıkça görülür. Yürütülebilir dosyayı, komut bağımsız değişkenlerini ve kullanıcı bağlamını karşılaştırın. Bir RunOnce girdisi kullanıldıktan sonra kaybolabilir; toplanmış durum, önceki oturum açmaların eksiksiz bir geçmişi değildir.

### RunServices (Eski) {#run_services}

**Kontrol kimliği:** `run_services` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Eski, makine genelindeki `RunServices` anahtarındaki değerleri listeler.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Bu, geçmişe ait bir yapılandırma yüzeyidir; modern bir Windows derlemesinin bunu yürüteceğinin garantisi değildir. Hedefi ve işletim sistemine uygulanabilirliği inceleyin, ardından bağımsız yürütme kanıtı arayın.

### RunServicesOnce (Eski) {#run_services_once}

**Kontrol kimliği:** `run_services_once` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Eski, makine genelindeki `RunServicesOnce` anahtarındaki değerleri listeler.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Boş olmayan değerleri yapılandırma ipuçları olarak ele alın. Kaynak Windows sürümünün mekanizmayı destekleyip desteklemediğini ve dağıtım veya uyumluluk yazılımının girdiyi açıklayıp açıklamadığını kontrol edin.

### S0 Initial Command {#s0_initial_command}

**Kontrol kimliği:** `s0_initial_command` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Session Manager'dan `S0InitialCommand` değerini olası bir Session 0 başlatma komutu olarak okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**Nasıl incelenir:** Toplanmış Windows derlemesine ve önyükleme durumuna uygulanabilirliği belirleyin. Yapılandırılmış bir komut, önyükleme yürütmesinin zaman damgalı bir kaydı değildir; dosyasını inceleyin ve önyükleme/işlem telemetrisiyle ilişkilendirin.

### Session Manager Execute {#session_manager_execute}

**Kontrol kimliği:** `session_manager_execute` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Session Manager'dan boş olmayan `Execute` girdilerini okuyarak oturum başlatması için seçilmiş programları ortaya çıkarır.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**Nasıl incelenir:** Beklenen görüntü temel çizgisiyle (baseline) karşılaştırın ve olağandışı yerel yürütülebilir dosyaları veya yolları inceleyin. Yürütme olduğunu öne sürmeden önce değişiklikten sonra ilgili bir önyükleme/oturumun gerçekleşip gerçekleşmediğini belirleyin.

### Session Manager SubSystems {#session_manager_subsystems}

**Kontrol kimliği:** `session_manager_subsystems` · **Katalog ATT&CK:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei neyi kontrol eder:** Session Manager'ın `SubSystems` anahtarındaki, Windows alt sistemi komutunu yapılandıran `Windows` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**Nasıl incelenir:** Normal komut uzundur ve sürüme bağlıdır. Değerin tamamını ve bağımsız değişkenlerini aynı derlemenin güvenilir bir temel çizgisiyle karşılaştırın; kritik bir alt sistem ayarını değiştirmek başlangıcı da bozabilir.

### Setup Execute {#setup_execute}

**Kontrol kimliği:** `setup_execute` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Session Manager'daki `SetupExecute` girdilerini okuyarak başlangıç için yapılandırılmış kurulumla ilgili komutları kapsar.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**Nasıl incelenir:** Kurulum, yükseltme ve kurtarma etkinlikleri meşru girdiler oluşturabilir. Yapılandırmayı kalıcılığa bağlamadan önce kurulum günlükleri ve başvurulan görüntüyle karşılaştırın.

### Shell Folders Startup Yeniden Yönlendirmesi {#shell_folders_startup}

**Kontrol kimliği:** `shell_folders_startup` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Hem Shell Folders hem de User Shell Folders içindeki `Startup` ve `Common Startup` değerlerini varsayılan Startup konumlarıyla karşılaştırır. Varsayılan olmayan yeniden yönlendirmeler bildirilir ve toplanmış klasör içerikleri incelenir.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` ve `User Shell Folders`: `Common Startup`.
- NTUSER.DAT içindeki eşdeğer `Software\Microsoft\Windows\CurrentVersion\Explorer` ayarları: `Startup`, ayrıca yeniden yönlendirilmiş konumlardaki toplanmış dosyalar.

**Nasıl incelenir:** Yeniden yönlendirilmiş bir Startup klasörü otomatik başlatmaları gizleyebilir, ancak yeniden yönlendirme meşru olabilir. Yolun toplanmış kanıtlar içinde çözümlenmesi gerekir; toplanmamış bir hedef bilinmeyendir, klasörün boş olduğunun kanıtı değildir.

### Shell Launcher Geçersiz Kılması {#shell_launcher}

**Kontrol kimliği:** `shell_launcher` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** SOFTWARE kovanından ilke `Shell` değerini ve `system.ini` boot Shell eşlemesini okur. Bunlar alternatif kabuk seçimi yüzeyleridir.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**Nasıl incelenir:** Kiosklar ve uygulama cihazları Explorer'ı bilinçli olarak değiştirebilir. Seçilen kabuğu makinenin rolüyle karşılaştırın ve ayarın ilgili oturum için etkili olduğunu doğrulayın.

### Startup Klasörü {#startup_folder}

**Kontrol kimliği:** `startup_folder` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** İlk kullanılabilir Startup klasörü yeniden yönlendirmesini izleyerek varsayılan kullanıcı başına ve ortak Startup klasörlerindeki doğrudan dosyaları inceler. Toplanmış bir `.lnk` ayrıştırılabildiğinde kısayol hedefleri kurtarılır.

**İncelenen kanıtlar:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` ve `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`.
- Makine/kullanıcı Shell Folders ve User Shell Folders ayarlarındaki ilgili `Common Startup` / `Startup` yeniden yönlendirmeleri.

**Nasıl incelenir:** Startup bağlantıları ve dosyaları oturum açıldığında meşru uygulamaları başlatabilir. Dosya adlarının yanı sıra hedef bağımsız değişkenlerini de inceleyin. Ortak klasör paylaşılan bir yapılandırma konumudur; başlatılan programın bir SYSTEM belirteci kullandığını kanıtlamaz.

### Terminal Services Initial Program {#ts_initial_program}

**Kontrol kimliği:** `ts_initial_program` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Terminal Services ilkesi `InitialProgram` değerini ve sınırlı iç içe RunOnceEx bölümleri dahil olmak üzere makinenin Terminal Server kurulumu Run/RunOnce/RunOnceEx yapılandırmasını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | Boş olmayan tüm değerler |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | Boş olmayan tüm değerler |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Uzak uygulama dağıtımları bu ayarları bilinçli olarak kullanabilir. Etkilenen oturumları belirleyin ve seçilen komutları işlem kanıtlarına kadar izleyin; kayıt tek başına uzak bir oturum açmayı göstermez.

### Winlogon AppSetup {#winlogon_appsetup}

**Kontrol kimliği:** `winlogon_appsetup` · **Katalog ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei neyi kontrol eder:** Winlogon'dan `AppSetup` değerini okur ve virgülle ayrılmış girdileri ayrı adaylara böler.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**Nasıl incelenir:** Her bileşeni beklenen Windows kurulum/oturum açma yazılımıyla karşılaştırın. Bir liste girdisi ayrı olarak gözlemlenmiş bir yürütme değildir ve önemi Windows sürümüne ve dağıtım durumuna bağlı olabilir.

### Winlogon GinaDLL {#winlogon_gina_dll}

**Kontrol kimliği:** `winlogon_gina_dll` · **Katalog ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei neyi kontrol eder:** Eski Winlogon `GinaDLL` değiştirme ayarını okur. GINA, daha eski bir Windows oturum açma uzantısı modeliydi.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**Nasıl incelenir:** Bu öncelikle Vista öncesi sistemlerle ilgilidir. Daha yeni bir sistemde kalmış bir anahtar, etkili bir oturum açma uzantısının kanıtı değildir; kaynak sürümü ve başvurulan DLL'leri inceleyin.

**Daha fazla ayrıntı:** [Microsoft belgeleri](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina).

### Winlogon MPNotify {#winlogon_mpnotify}

**Kontrol kimliği:** `winlogon_mpnotify` · **Katalog ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei neyi kontrol eder:** Oturum açmayla ilgili bir uzantı yüzeyi olan Winlogon'un `mpnotify` komut/kitaplık ayarını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**Nasıl incelenir:** Hedefi ve bağımsız değişkenleri eşleşen bir Windows temel çizgisiyle karşılaştırın. Kodun çalıştığını öne sürmeden önce sürüme özgü davranışı kontrol edin ve sonraki bir oturum açmayla ilişkilendirin.

### Winlogon Notify Paketleri {#winlogon_notify_packages}

**Kontrol kimliği:** `winlogon_notify_packages` · **Katalog ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei neyi kontrol eder:** Her doğrudan Winlogon `Notify` paket kaydından `DllName` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**Nasıl incelenir:** Bildirim paketleri eski bir uzantı mekanizmasıdır. Toplanmış Windows sürümünün bunları yükleyip yüklemediğini belirleyin ve onaylı kimlik doğrulama yazılımını açıklanamayan kitaplıklardan ayırt edin.

### Winlogon Shell {#winlogon_shell}

**Kontrol kimliği:** `winlogon_shell` · **Katalog ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei neyi kontrol eder:** Masaüstü kabuğu seçimini ortaya çıkarmak için makine genelindeki ve kullanıcı başına Winlogon `Shell` değerlerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**Nasıl incelenir:** Explorer yaygındır, ancak kiosklar başka bir kabuk kullanabilir. Eklenmiş komutları veya olağandışı yolları inceleyin ve oturum açıldığında hangi kullanıcının etkili yapılandırmasının uygulandığını belirleyin.

### Winlogon System {#winlogon_system}

**Kontrol kimliği:** `winlogon_system` · **Katalog ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei neyi kontrol eder:** Winlogon'un `System` değerini okur ve virgülle ayrılmış girdileri tek tek adaylara ayırır.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**Nasıl incelenir:** Bunu sürüme duyarlı bir oturum açma yapılandırması ipucu olarak ele alın. Listeyi bir çalıştırma geçmişi olarak görmek yerine başvurulan her bileşeni inceleyin ve yürütme kanıtlarıyla doğrulayın.

### Winlogon Taskman {#winlogon_taskman}

**Kontrol kimliği:** `winlogon_taskman` · **Katalog ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei neyi kontrol eder:** Alternatif bir görev yönetimi komutu için makine/kullanıcı Winlogon `Taskman` değerlerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**Nasıl incelenir:** Hangi Windows davranışının bu ayarı kullanacağını ve hedefin beklenen bir hedef olup olmadığını belirleyin. Yapılandırılmış bir değiştirme, bir kullanıcının o özelliği çağırdığının kanıtı değildir.

### Winlogon Userinit {#winlogon_userinit}

**Kontrol kimliği:** `winlogon_userinit` · **Katalog ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei neyi kontrol eder:** Makine/kullanıcı Winlogon `Userinit` değerlerini okur; virgülle ayrılmış programları bölerken sondaki boş öğeleri yok sayar.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**Nasıl incelenir:** Normal bir girdi çoğunlukla `userinit.exe` dosyasını gösterir. Ek girdileri ve değiştirilmiş yolları inceleyin. İlgili kullanıcıyı ve sonraki bir oturum açmayı doğrulayın; `KeyLastWrite` özellikle bu değere değil, anahtarın bütününe aittir.

### Winlogon VMApplet {#winlogon_vmapplet}

**Kontrol kimliği:** `winlogon_vmapplet` · **Katalog ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei neyi kontrol eder:** Daha eski bir sistem uygulamacığı (applet) başlatma yüzeyini kapsayan Winlogon `VMApplet` komut ayarını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**Nasıl incelenir:** İşletim sistemine uygulanabilirliği ve beklenen uygulamacığı kontrol edin. Beklenmeyen bir komut incelenmeyi hak eder, ancak tetikleyici arayüzün açıldığını kanıtlamaz.

## Hizmetler ve sistem uzantıları { #services-and-system-extensions }

Windows hizmetleri ve genişletilebilir sistem bileşenleri, yürütülebilir dosyaları veya DLL'leri yüklemek için birçok meşru yol sağlar. Yapılandırılmış hedefi beklenen ürünle, hizmet hesabıyla, başlatma ayarlarıyla ve kaynağa dayalı kurulum geçmişiyle karşılaştırın.

### DiagTrack DLL {#diagtrack_dll}

**Kontrol kimliği:** `diagtrack_dll` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** DiagTrack hizmetinin `ImagePath` değerini okur. Katalogdaki DLL odaklı adına rağmen bu kontrol, yapılandırılmış hizmet görüntüsü komutunu inceler.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**Nasıl incelenir:** Hizmet temel çizgisiyle ve genel hizmet kontrolleriyle karşılaştırın. Aynı yapılandırma birden fazla kontrol sonucu üretebilir; aday sayıları farklı saldırıların sayısı değildir.

### DiagTrack Listener DLL {#diagtrack_listener_dll}

**Kontrol kimliği:** `diagtrack_listener_dll` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** DiagTrack-Listener autologger yapılandırmasından `FileName` değerini okur. Bu bir ETW autologger ayarıdır; bir DLL'in yüklendiğinin kanıtı değildir.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**Nasıl incelenir:** Ham değeri ve Windows günlük kaydı yapılandırmasını inceleyin. Katalog bunu bir DLL tekniği olarak etiketler, ancak bu arama yüklenebilir bir DLL'i doğrulamaz veya bu değer aracılığıyla yürütmeyi kanıtlamaz.

### Netsh Yardımcı DLL'i {#netsh_helper}

**Kontrol kimliği:** `netsh_helper` · **Katalog ATT&CK:** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**Juntei neyi kontrol eder:** Makinenin `NetSh` yardımcı kayıt anahtarı altındaki boş olmayan değerleri listeler. Yardımcılar `netsh.exe` aracını DLL tabanlı komutlarla genişletebilir.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Ağ ürünleri yardımcı yükleyebilir. Her değeri beklenen bir ürünle eşleştirin ve çözümlenen kitaplığı inceleyin, ardından ilgili netsh yürütmesini veya görüntü yüklemelerini arayın.

### Yazdırma İzleyicileri (Print Monitors) {#print_monitors}

**Kontrol kimliği:** `print_monitors` · **Katalog ATT&CK:** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**Juntei neyi kontrol eder:** Etkin denetim kümesindeki her doğrudan Print Monitor kaydının altındaki `Driver` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**Nasıl incelenir:** Yazdırma yazılımları ve bağlantı noktası izleyicileri biriktiriciyi (spooler) meşru olarak genişletir. DLL'i ve ürün meta verilerini yazıcı envanteriyle karşılaştırın ve yüklemeyi biriktirici/görüntü yükleme kanıtlarıyla doğrulayın.

### Yazdırma İşlemcileri (Print Processors) {#print_processors}

**Kontrol kimliği:** `print_processors` · **Katalog ATT&CK:** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**Juntei neyi kontrol eder:** x64 ve x86 yazdırma işlemcisi kayıt ağaçları altındaki `Driver` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**Nasıl incelenir:** Üçüncü taraf yazıcı paketleri, bilinen Windows varsayılanlarının ötesinde işlemciler ekleyebilir. Kötü niyet atfetmeden önce her iki mimariyi, asıl DLL'i ve yüklü yazıcı yazılımını inceleyin.

### Service Control Manager Uzantısı {#scm_extension}

**Kontrol kimliği:** `scm_extension` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Bir hizmet yöneticisi uzantı yüzeyi olan `ServiceControlManagerExtension` içindeki boş olmayan değerleri listeler.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Adı geçen kitaplıkları ve bu Windows derlemesinin ayarı kullanıp kullanmadığını inceleyin. Önyükleme/hizmet etkinliğiyle ilişkilendirin; kontrol, yüklenmiş bir uzantıyı kanıtlamak yerine kayıtların envanterini çıkarır.

### DNS Server Level Plugin DLL {#server_level_plugin_dll}

**Kontrol kimliği:** `server_level_plugin_dll` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Etkin SYSTEM denetim kümesindeki DNS hizmetinin `ServerLevelPluginDll` ayarını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**Nasıl incelenir:** Bu, bazı etki alanı denetleyicileri de dahil olmak üzere DNS Server rolünü çalıştıran sistemlerle ilgilidir. DLL'i, hizmet hesabını ve hizmetin yeniden başlatılması/yüklenmesi kanıtlarını inceleyin; etkin olmayan veya bulunmayan bir rol sonucu değiştirir.

### Hizmet Hata Komutu {#service_failure_command}

**Kontrol kimliği:** `service_failure_command` · **Katalog ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei neyi kontrol eder:** Etkin denetim kümesindeki her hizmetin `FailureCommand` değerini okuyarak yapılandırılmış bir kurtarma komutunu ortaya çıkarır.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`.

**Nasıl incelenir:** Komut yalnızca hizmetin kurtarma eylemleri onu seçtiğinde ve ilgili hata gerçekleştiğinde çalışır. Kurtarma yapılandırmasını ve hata/işlem kayıtlarını inceleyin; değer tek başına etkin bir kurtarma eylemini kanıtlamaz.

### SNMP Uzantı Aracısı DLL'i {#snmp_extension_agent}

**Kontrol kimliği:** `snmp_extension_agent` · **Katalog ATT&CK:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei neyi kontrol eder:** SNMP hizmetinin `ExtensionAgents` kayıt anahtarı altındaki değerleri listeler. Değerler doğrudan bir DLL yolu yerine başka bir kayıt defteri konumunu belirtebilir.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Dolaylı bir kaydı kitaplığına kadar elle izleyin ve SNMP'nin yüklü olduğunu ve kullanıldığını doğrulayın. Bu çıktıdaki çözümlenmemiş bir dize, DLL'in eksik veya kötü amaçlı olduğunun kanıtı değildir.

### Zaman Sağlayıcıları (Time Providers) {#time_providers}

**Kontrol kimliği:** `time_providers` · **Katalog ATT&CK:** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**Juntei neyi kontrol eder:** Doğrudan Windows Time `TimeProviders` kayıtlarından `DllName` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**Nasıl incelenir:** Sağlayıcı etkinleştirme ayarlarını ve Windows Time hizmetinin durumunu ayrıca inceleyin; DLL yolu kontrolü etkin bir sağlayıcı gerektirmez. Zaman eşitleme ürünleri meşru yedekler sağlayabilir.

### Windows Hizmeti (ServiceDll) {#windows_service_dll}

**Kontrol kimliği:** `windows_service_dll` · **Katalog ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei neyi kontrol eder:** Etkin SYSTEM denetim kümesindeki hizmetlerin altındaki `Parameters\ServiceDll` değerini okur ve mevcut olduğunda hizmete özgü zaman damgası kanıtlarını ekler.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`.

**Nasıl incelenir:** `svchost.exe` gibi paylaşılan bir barındırıcı, yük DLL'i değildir. Kitaplığı, hizmet hesabını, başlatma durumunu ve eşleşen hizmet kurulumu/görüntü yükleme kanıtlarını inceleyin; bir kayıt, hizmetin çalıştığını kanıtlamaz.

**Daha fazla ayrıntı:** [Microsoft belgeleri](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Windows Hizmeti (ImagePath) {#windows_service_image_path}

**Kontrol kimliği:** `windows_service_image_path` · **Katalog ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei neyi kontrol eder:** Etkin denetim kümesindeki hizmet `ImagePath` komutlarını okur. ImagePath içermeyen 1, 2 veya 8 türündeki sürücüler, örtük bir `System32\drivers\<service>.sys` hedefini korur.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath`, `Type` ve destekleyici hizmet meta verileri.

**Nasıl incelenir:** Hizmet hesabını ve başlatma ayarlarını çıktıdaki makine kapsamı etiketinden ayrı olarak inceleyin. Devre dışı bırakılmış veya isteğe bağlı başlatılan kayıtlar da envanterin parçasıdır. Zaman damgaları için belirsiz paylaşılan yürütülebilir dosya yolları yerine hizmet adı kanıtı tercih edilir.

**Daha fazla ayrıntı:** [Microsoft belgeleri](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Windows Update Hizmeti Başlangıç DLL'i {#wu_service_startup_dll}

**Kontrol kimliği:** `wu_service_startup_dll` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Etkin denetim kümesinden Windows Update hizmetinin `Parameters\ServiceDll` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**Nasıl incelenir:** Kitaplığı aynı Windows derlemesi ve yüklü bakım (servicing) durumuyla karşılaştırın. Bu, genel ServiceDll envanteriyle örtüşür ve ikinci, ayrı bir ihlali temsil etmez.

## Zamanlanmış görevler { #scheduled-tasks }

Görev Zamanlayıcı yapılandırmayı hem XML dosyalarında hem de SOFTWARE kovanında saklar. Bu kaynakları birlikte okuyun. Bir görev hiç çalışmadan kayıtlı olabilir ve eksik bir toplama, gizli veya hayalet bir göreve benzeyebilir.

### Hayalet Zamanlanmış Görev {#ghost_task}

**Kontrol kimliği:** `ghost_task` · **Katalog ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei neyi kontrol eder:** SOFTWARE TaskCache `Tree` ve `Tasks` kayıtlarını karşılaştırır ve toplamada görev dosyaları mevcut olduğunda bunlara karşılık gelen görev XML dosyalarını arar.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` ve `TaskCache\Tasks`.
- Dosya sistemi karşılaştırması için toplanmış `Windows\System32\Tasks\<task path>` dosyaları.

**Nasıl incelenir:** Eksik karşılıklar silinmiş/eskimiş görevleri, yerleşik kayıtları veya eksik bir toplamayı yansıtabilir. Toplanmış XML olmaması, dosya sistemi karşılaştırmasının yapılamadığı anlamına gelir. Bir görevi gizli veya kötü amaçlı olarak nitelendirmeden önce kapsamı ve tam tutarsızlığı inceleyin.

### Gizli Zamanlanmış Görev {#hidden_scheduled_task}

**Kontrol kimliği:** `hidden_scheduled_task` · **Katalog ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei neyi kontrol eder:** TaskCache Tree kayıtlarında eksik bir `SD` değeri veya boş bir ikili güvenlik tanımlayıcısı olup olmadığını kontrol eder; bu koşullar bir görevi olağan listelemeden gizlemekle ilişkilidir.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: TaskCache karşılık meta verileriyle birlikte `SD` varlığı/verisi.

**Nasıl incelenir:** Eksik/boş SD belirli bir yapılandırma anormalliğidir; görevin yürütüldüğünün veya kötü amaçlı olduğunun kanıtı değildir. XML `Hidden` ayarı ve rastgele ACL sorunları farklı koşullardır ve burada kullanılan koşul bunlar değildir.

**Daha fazla ayrıntı:** [Microsoft belgeleri](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/).

### Zamanlanmış Görev (XML Dosyaları) {#scheduled_task_files}

**Kontrol kimliği:** `scheduled_task_files` · **Katalog ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei neyi kontrol eder:** Toplanmış görev XML'ini Exec komutları/bağımsız değişkenleri ve COM işleyicisi sınıf kimlikleri/verileri için ayrıştırır. Sorumluyu `Actions Context`/`Principal id` ile seçer, kullanıcı ve grup kimliğini ayrı ayrı korur ve SYSTEM, USER veya UNKNOWN olarak bildirir. Çözümlenmemiş COM eylemleri görünür kalır.

**İncelenen kanıtlar:**

- Exec, ComHandler ve mevcut sorumlu alanları dahil olmak üzere `Windows\System32\Tasks\<task path>` XML dosyaları.
- Eşleşen kullanıcının toplanmış `UsrClass.dat` dosyası ve NTUSER `Software\Classes`, ayrıca yerel ve 32 bit görünümlerde makine SOFTWARE `Classes\CLSID`.
- SID sorumluları için toplanmış SOFTWARE `ProfileList`. Adlandırılmış sorumlular için önce benzersiz eşleşen bir `user.DOMAIN` klasörü, ardından toplanmış yerel ana bilgisayar sonekleri ve son olarak düz `user` klasörü tercih edilir. Eşleşmeler açıkça sezgisel (heuristic) olarak etiketlenir; gerçek belirsizlik çözümsüz kalır ve bulguda ve kapsamda not edilir. Geçici (volatile) kayıt defteri anahtarları gerekmez.
- Yönetilen `CodeBase` (sürüm alt anahtarları dahil), AppID ilişkilendirmeleri ve bir AppID `LocalService` için SYSTEM hizmeti `ImagePath`.

COM sunucusu adayları kayıt defteri kaynaklarını `Description` içinde içerir. Yalnızca yürütülebilir dosya adı içeren
AppID ilişkilendirmelerine tahmin edilmiş bir dizin atanmaz, bunlar çözümsüz kalır. Yerel
`file:` URL'lerinin kodu çözülür; uzak URL'ler ağ erişimi olmadan korunur. Kullanıcı sınıfı geçersiz kılmaları
o profille sınırlıdır ve eksik bir kullanıcı CLSID'si, gölgelediği makine kaydına geri düşmez.
Yalnızca gruba ait görevler, kapsam hata özetine olağan bildirimler eklemeden, açıklamalarında bireysel kullanıcı
sınırlamasıyla birlikte makine adaylarını korur.

**Nasıl incelenir:** Kaynak XML'deki ve olay günlüklerindeki tetikleyicileri, etkin durumu, sorumluyu ve çalıştırma geçmişini inceleyin. Oluşturulan bir görev hiç çalışmamış olabilir. Çözümlenmemiş bir COM işleyicisi yine de bir adaydır ve eksik/yalnızca gruba ait sorumlu verilerinin SYSTEM anlamına geldiği varsayılmamalıdır.

## WMI, BITS ve diğer olay tetikleyicileri { #wmi-bits-and-other-event-triggers }

Windows Management Instrumentation (WMI) sistem yönetimini destekler. Background Intelligent Transfer Service (BITS) arka plan aktarımlarını destekler. Her ikisi de olayla tetiklenen komutları saklayabilir. Bazı mekanizmalar bir yönetim olayını, aktarım bildirimini, uygulama başlatmasını veya kullanıcı eylemini bekler. Saklanan bir tetikleyici faydalı bir yapılandırma kanıtıdır, ancak varlığı tetikleyicinin devreye girdiğini kanıtlamaz.

### Erişilebilirlik Özellikleri Arka Kapısı {#accessibility_tools}

**Kontrol kimliği:** `accessibility_tools` · **Katalog ATT&CK:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei neyi kontrol eder:** Toplanmış bir başvuru ikili dosyasından kullanılabilir Microsoft katalog meta verilerini zorunlu tuttuktan sonra, toplanmış yedi System32 erişilebilirlik ikili dosyasını PE OriginalFilename ve sertifika/katalog meta verilerini kullanarak karşılaştırır.

**İncelenen kanıtlar:**

- `Windows\System32\sethc.exe`, `osk.exe`, `narrator.exe`, `magnify.exe`, `utilman.exe`, `atbroker.exe` ve `DisplaySwitch.exe`.
- Toplanmış `Windows\System32\winlogon.exe`, `cmd.exe` veya `notepad.exe` başvuru meta verileri ve ilgili toplanmış kataloglar.

**Nasıl incelenir:** Meta veriler yeniden adlandırılmış bir değiştirmeye işaret edebilir, ancak Juntei imzaları veya sertifika güvenini doğrulamaz. Gerekli kataloglar olmadan yapılan seyrek bir toplama, imzasız görünen her dosyayı arka kapı olarak değerlendirmek yerine eksik karşılaştırma kapsamı üretir.

### Erişilebilirlik Özellikleri {#assistive_technology}

**Kontrol kimliği:** `assistive_technology` · **Katalog ATT&CK:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei neyi kontrol eder:** Yerel/32 bit SOFTWARE görünümlerinde kayıtlı erişilebilirlik `StartExe` ve `StartParams` komutlarını ve kullanıcı ile DEFAULT kovanlarından etkin erişilebilirlik `Configuration` girdilerini okur. Sayısal/dize olmayan StartExe yer tutucuları atlanır.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` ve `StartParams`; ayrıca makinenin `Wow6432Node` görünümü.
- NTUSER.DAT içindeki `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` ve DEFAULT kovanındaki eşdeğer ayar.

**Nasıl incelenir:** Erişilebilirlik yazılımları meşrudur. Etkin bir özellik adını gerçek yürütülebilir dosya kaydından ayırt edin, ardından hedefi ve oturum açma/masaüstü yürütme kanıtlarını inceleyin. Bu, erişilebilirlik yürütülebilir dosyasının kendisinin değiştirilmesinden ayrı bir durumdur.

**Daha fazla ayrıntı:** [Teknik başvurusu (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/).

### BITS İş Bildirimi Komutu {#bits_notify_command}

**Kontrol kimliği:** `bits_notify_command` · **Katalog ATT&CK:** [T1197](https://attack.mitre.org/techniques/T1197/)

**Juntei neyi kontrol eder:** Toplanmış `qmgr.db`, `qmgr0.dat` veya `qmgr1.dat` dosyalarından sayılmış dize (counted-string) BITS iş kayıtlarını kurtarır; boş olmayan bildirim programlarını, parametreleri, iş kimliklerini ve sahip SID'lerini korur.

**İncelenen kanıtlar:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`, `qmgr0.dat` ve `qmgr1.dat`.

**Nasıl incelenir:** Bunlar kazınmış kayıtlardır: ayrılmış/geçerli iş durumu doğrulanmaz ve eskimiş/silinmiş işler görünebilir. Veritabanının MFT yazma zamanı zayıf bir kapsayıcı kanıtıdır; bildirimin yürütme zamanı değildir. Sahip SID'leri SYSTEM, yerel hizmet, ağ hizmeti ve kullanıcıları birbirinden ayırır.

### Command Processor AutoRun {#cmd_autorun}

**Kontrol kimliği:** `cmd_autorun` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Komut istemi başlangıcını etkileyebilen bir ayar olan makine/kullanıcı Command Processor `AutoRun` komutlarını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**Nasıl incelenir:** Geliştirme araçları ve kabuk özelleştirmeleri bir değeri açıklayabilir. `cmd.exe /d` AutoRun'ı atlar; bu nedenle yapılandırma tek başına cmd.exe her göründüğünde bunun çalıştığını kanıtlamaz. Komut içeriğini ve tetikleyen işlemin bağımsız değişkenlerini inceleyin.

**Daha fazla ayrıntı:** [Microsoft belgeleri](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd).

### ErrorHandler.cmd Kalıcılığı {#error_handler_cmd}

**Kontrol kimliği:** `error_handler_cmd` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Toplanmış `Windows\System32\ErrorHandler.cmd` betiğinin varlığını bir hata işleyicisi kalıcılık adayı olarak bildirir.

**İncelenen kanıtlar:**

- `Windows\System32\ErrorHandler.cmd`.

**Nasıl incelenir:** Varlık tek başına ilgili Windows kurtarma/hata yolunun onu çağırdığını kanıtlamaz. Betiği okuyun, kökenini inceleyin ve kurulum/hata/işlem kanıtlarıyla ilişkilendirin.

### Power Automate Desktop Akışları {#power_automate}

**Kontrol kimliği:** `power_automate` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Her kullanıcının Power Automate Desktop `flows` ve `scripts` depolarının altındaki toplanmış akış/betik dizinlerini bulur ve keşfedilen her akışı bir kez bildirir.

**İncelenen kanıtlar:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` ve `scripts\<flow>\...`.

**Nasıl incelenir:** Bu, her zamanlanmış tetikleyiciyi veya yürütülmüş bir akışı değil, saklanan otomasyonu belirler. Bunu otomatik veya kötü amaçlı kalıcılık olarak değerlendirmeden önce akış içeriğini ve harici başlatma yapılandırmasını inceleyin.

### PowerShell Profili {#powershell_profiles}

**Kontrol kimliği:** `powershell_profiles` · **Katalog ATT&CK:** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**Juntei neyi kontrol eder:** Kullanıcı Documents klasörleri, keşfedilen OneDrive Documents klasörleri ve toplanmış Documents yeniden yönlendirmeleri dahil olmak üzere, tanınan Windows PowerShell ve PowerShell kurulum/profil dizinlerinde profile.ps1 tarzı dosyaları bulur.

**İncelenen kanıtlar:**

- Doğrudan `Windows\System32\WindowsPowerShell\v1.0` ve `Windows\SysWOW64\WindowsPowerShell\v1.0` içinde bulunan, `profile.ps1` ile biten dosyalar.
- Toplanmış Documents yeniden yönlendirmeleri ve keşfedilen OneDrive Documents klasörleri dahil olmak üzere kullanıcı başına `Documents\WindowsPowerShell` ve `Documents\PowerShell`.
- `Program Files\PowerShell` veya `Program Files (x86)\PowerShell` altındaki sürüm dizinlerinde doğrudan bulunan, `profile.ps1` ile biten dosyalar.

**Nasıl incelenir:** Profiller zararsız özelleştirmeler içerebilir. Betiği okuyun ve çağrılan yükleri izleyin. PowerShell'i `-NoProfile` ile başlatmak bunları atlar; dosya varlığı tek başına profil yürütmesini veya kötü amaçlı içeriği kanıtlamaz.

### Ekran Koruyucu Ele Geçirme {#screensaver}

**Kontrol kimliği:** `screensaver` · **Katalog ATT&CK:** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**Juntei neyi kontrol eder:** Her kullanıcının Desktop `SCRNSAVE.EXE` değerini okuyarak yapılandırılmış ekran koruyucu görüntüsünü belirler.

**İncelenen kanıtlar:**

- NTUSER.DAT içindeki `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE`.

**Nasıl incelenir:** Onaylı ekran koruyucularla karşılaştırın ve hedefi inceleyin. Diğer masaüstü ayarları, oturum durumu ve zaman aşımı tetiklenip tetiklenemeyeceğini belirler; kaydedilmiş bir yol, bir ekran koruyucunun başladığını kanıtlamaz.

### Telemetry Controller Komutu {#telemetry_controller}

**Kontrol kimliği:** `telemetry_controller` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** TelemetryController ve onun doğrudan alt kayıtlarından `Binary`, `Command` ve `MaintenanceCommand` değerlerini okur.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` ve doğrudan alt öğeleri: `Binary`, `Command`, `MaintenanceCommand`.

**Nasıl incelenir:** Seçilen yürütülebilir dosyayı ve bağımsız değişkenleri aynı Windows derlemesi ve bakım yazılımıyla karşılaştırın. Bir kayıt defteri komutu, bir bakım tetikleyicisinin devreye girdiğini göstermez.

### Windows Terminal Özel Profilleri {#windows_terminal}

**Kontrol kimliği:** `windows_terminal` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Paketlenmiş, önizleme ve paketlenmemiş Windows Terminal JSON/JSONC ayarlarını ayrıştırır ve boş olmayan profil/varsayılan `commandline` ayarlarını bildirir.

**İncelenen kanıtlar:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` ve WindowsTerminalPreview eşdeğeri.
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: eski profiles dizileri, `profiles.list` ve `profiles.defaults` commandline alanları.

**Nasıl incelenir:** Sıradan kabuklar doğal olarak burada görünür. Seçili/varsayılan profili belirleyin ve olağandışı komutları veya başlatma bağımsız değişkenlerini inceleyin. Yalnızca bir profili kaydetmek onu başlatmaz.

### WMI Olay Aboneliği {#wmi_event_subscription}

**Kontrol kimliği:** `wmi_event_subscription` · **Katalog ATT&CK:** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**Juntei neyi kontrol eder:** Toplanmış WMI OBJECTS.DATA deposundan tanınan olay tüketicisi örneklerini ve filtreden tüketiciye bağlama başvurularını kazır. Tanınan sınıflar arasında komut satırı, etkin betik, günlük dosyası, olay günlüğü ve SMTP tüketicileri bulunur.

**İncelenen kanıtlar:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` ve `Repository\FS\OBJECTS.DATA`.

**Nasıl incelenir:** Ham bir depo eşleşmesi eskimiş, bağlanmamış veya geçmişe ait olabilir; çıktı bu niteliği korur. Kazıma, eksiksiz bir etkin abonelik grafiğini yeniden oluşturmaz veya tetikleyicinin devreye girdiğini kanıtlamaz. Deponun dosya yazma zamanı bir abonelik oluşturma/yürütme zamanı değildir.

## Explorer, COM ve dosya veya URL işleyicileri { #explorer-com-and-file-or-url-handlers }

Component Object Model (COM), uygulamaların kayıtlı yazılım bileşenlerini etkinleştirmesini sağlar. Windows kabuğu dosyaları, URL'leri, klasörleri ve özel nesneleri açmak için kayıtlı işleyicileri kullanır. COM tanımlayıcıları (CLSID'ler) bileşenleri tanımlar; ProgID'ler bazı bileşenlere okunabilir adlar verir. Her kaydı seçtiği komuta veya DLL'e kadar izleyin.

### App Paths Ele Geçirme {#app_paths}

**Kontrol kimliği:** `app_paths` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Kabuk tabanlı başlatmalar için uygulama aramasını yeniden yönlendirebilen, makine genelindeki `App Paths` kayıtlarının altındaki varsayılan değerleri okur.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>` varsayılan değeri.

**Nasıl incelenir:** Bir App Paths kaydı yüklü uygulamalar için yaygındır ve her işlem başlatmasının genel olarak yeniden yazılması anlamına gelmez. Kayıtlı adı ve görüntüyü beklenen yazılımla karşılaştırın, ardından ilgili kabuk/işlem kanıtlarını inceleyin.

### COM Nesnesi Ele Geçirme {#com_treat_as}

**Kontrol kimliği:** `com_treat_as` · **Katalog ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei neyi kontrol eder:** Desteklenen yerel/32 bit sınıf depolarında COM `TreatAs` yeniden yönlendirmelerini ve bir makine CLSID'sini gölgeleyen kullanıcı başına InprocServer32/LocalServer32 kayıtlarını bulur. TreatAs hedefleri aynı kullanıcının sınıfları ve makine sınıfları kullanılarak çözümlenir.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` ve makinenin `Classes\Wow6432Node` görünümü.
- Uygulanmış `Wow6432Node` görünümleri dahil olmak üzere NTUSER.DAT içindeki kullanıcı başına `Software\Classes\CLSID` ve UsrClass.dat içindeki `CLSID`: `TreatAs`, `InprocServer32` ve `LocalServer32` varsayılan değerleri.

**Nasıl incelenir:** Kullanıcı başına geçersiz kılmalar yönetici hakları gerektirmeyebilir, ancak meşru yazılımlar da bunları kaydedebilir. Seçilen sınıfı ve DLL'i/yürütülebilir dosyayı izleyin, mimariyi ve etkinleştirme bağlamını değerlendirin ve gerçek COM etkinleştirmesini bağımsız olarak doğrulayın.

### Disk Temizleme İşleyicisi Ele Geçirme {#disk_cleanup_handler}

**Kontrol kimliği:** `disk_cleanup_handler` · **Katalog ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei neyi kontrol eder:** Disk Temizleme VolumeCaches işleyici CLSID'lerini okur ve çözümlenebilen makine COM InprocServer32 kayıtlarını kitaplıklarına kadar izler.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` varsayılan CLSID'si ve `Classes\CLSID\<CLSID>\InprocServer32`.

**Nasıl incelenir:** İşleyicinin çalışması için ilgili temizleme iş akışı tarafından çağrılması gerekir. Çözümlenmemiş işleyiciler bu kontrol tarafından çıktı olarak verilmez; bu nedenle ele geçirme eksikse ham kayıtlarını incelemek için kayıt defteri ayrıştırıcısını kullanın.

### Explorer AppKey Geçersiz Kılması {#explorer_app_key}

**Kontrol kimliği:** `explorer_app_key` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Doğrudan Explorer `AppKey` alt öğelerinin altındaki `ShellExecute` ve `Association` değerlerini okuyarak uygulama tuşlarıyla seçilen komutları/eylemleri kapsar.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` ve `Association`.

**Nasıl incelenir:** Klavye yardımcı programları ve OEM araçları bu değerleri tanımlayabilir. Bir ilişkilendirme adı bir dosya yolu olmak yerine başka bir kayıt defteri pivotu gerektirebilir. Bunu tetikleyecek kullanıcı eylemini kontrol edin.

### Browser Helper Objects {#explorer_bho}

**Kontrol kimliği:** `explorer_bho` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Browser Helper Object CLSID kayıtlarını okur ve mevcut olduğunda makine InprocServer32 kitaplıklarını çözümler; çözümlenmemiş sınıf kimlikleri görünür kalır.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` ve makinenin `Classes\CLSID\<CLSID>\InprocServer32` varsayılan değeri.

**Nasıl incelenir:** Bu, uyumlu tarayıcı/Explorer davranışına bağlıdır ve özellikle eski ortamlarla ilgilidir. Onaylı tarayıcı/güvenlik uzantıları görünebilir. Kayıtlı bir BHO, güncel bir tarayıcının onu yüklediğinin kanıtı değildir.

### Explorer Masaüstü CLSID Ele Geçirme {#explorer_clsid_hijack}

**Kontrol kimliği:** `explorer_clsid_hijack` · **Katalog ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei neyi kontrol eder:** Makine/kullanıcı sınıf depolarından, listelenen üç özel Explorer ad alanı CLSID'si üzerinde yapılandırılmış kabuk açma (shell-open) komutlarını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**Nasıl incelenir:** Desktop, My Computer ve My Documents için komut işleyicilerini eşleşen bir temel çizgiyle karşılaştırın. Bu, olası her Explorer ad alanı ele geçirmesini değil, seçili CLSID'leri ve yolları kontrol eder.

### Explorer Bağlam Menüsü İşleyicileri {#explorer_context_menu}

**Kontrol kimliği:** `explorer_context_menu` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Directory, Directory Background, Folder, Drive, AllFilesystemObjects ve `*` için bağlam menüsü işleyicilerini listeler; mümkün olduğunda sınıf kimliklerini toplanmış DLL kayıtlarına kadar izler.

**İncelenen kanıtlar:**

- Makine `HKLM\SOFTWARE\Classes` ve kullanıcı UsrClass.dat sınıf depoları: `<subject>\shellex\ContextMenuHandlers\<handler>` varsayılan değeri ve karşılık gelen COM kitaplığı kayıtları.

**Nasıl incelenir:** Arşiv araçlarından, güvenlik ürünlerinden ve bulut istemcilerinden gelen sağ tıklama uzantıları yaygındır. Asıl işleyiciyi ve tetikleyen etkileşimi inceleyin. Çözümlenmemiş GUID'ler, DLL'in var olmadığının kanıtı değil, faydalı ipuçları olarak kalır.

### Dosya İlişkilendirmesi Ele Geçirme {#file_association_hijack}

**Kontrol kimliği:** `file_association_hijack` · **Katalog ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei neyi kontrol eder:** `.txt`, `.pdf`, `.doc`, `.docx`, `.html`, `.htm`, `.js`, `.vbs`, `.hta`, `.exe`, `.bat`, `.cmd` ve `.ps1` için doğrudan uzantı ve çözümlenmiş ProgID `shell\open\command` kayıtlarını inceler.

**İncelenen kanıtlar:**

- Makine `HKLM\SOFTWARE\Classes` ve kullanıcı UsrClass.dat sınıf depoları: seçili `<extension>` varsayılanları ve `<extension-or-ProgID>\shell\open\command` varsayılanları.

**Nasıl incelenir:** Beklenen uygulamayla karşılaştırın ve komut bağımsız değişkenlerini inceleyin. Bu, tüm dosya türleri veya tam etkili UserChoice çözümlemesi değil, tanımlı bir uzantı kümesidir. Kaydedilmiş bir komut, bir belgenin açıldığını kanıtlamaz.

### Protokol İşleyicisi Ele Geçirme {#protocol_handler_hijack}

**Kontrol kimliği:** `protocol_handler_hijack` · **Katalog ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei neyi kontrol eder:** http, https, mailto, ms-msdt ve ms-officecmd için URL açma komutlarını, ayrıca keşfedilen `URL Protocol` kayıtlarını ve ilgili kullanıcı geçersiz kılmalarını okur.

**İncelenen kanıtlar:**

- Makine `HKLM\SOFTWARE\Classes` ve kullanıcı UsrClass.dat sınıf depoları: `URL Protocol` işaretçileri ve `<scheme>\shell\open\command` varsayılan değerleri.

**Nasıl incelenir:** Tarayıcılar ve uygulama yükleyicileri şemaları meşru olarak kaydeder. Komutu ve bağımsız değişkenleri izleyin, şemanın beklenen amacını değerlendirin ve URL açma/işlem kanıtlarıyla ilişkilendirin.

### Geri Dönüşüm Kutusu COM Uzantı İşleyicisi {#recycle_bin_com_extension}

**Kontrol kimliği:** `recycle_bin_com_extension` · **Katalog ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei neyi kontrol eder:** Listelenen Geri Dönüşüm Kutusu CLSID'si open/empty/explore komut geçersiz kılmalarını ve doğrudan bağlam menüsü/sürükle-bırak işleyici kayıtlarını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**Nasıl incelenir:** Onaylı kabuk uzantılarıyla karşılaştırın. Bildirilen bazı değerler doğrudan çözümlenmiş bir kitaplık yerine COM tanımlayıcılarıdır; kod yürütüldüğünü öne sürmeden önce başvurulan sınıfı ve kullanıcı eylemini inceleyin.

### Arama Protokolü İşleyicisi Ele Geçirme {#search_protocol_handler}

**Kontrol kimliği:** `search_protocol_handler` · **Katalog ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei neyi kontrol eder:** Makine/kullanıcı `search-ms\shell\open\command` varsayılan işleyicisini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**Nasıl incelenir:** Bir işleyici değişikliği, bir search-ms URL'si açıldığında olanları değiştirebilir, ancak varlığı istismar tesliminin kanıtı değildir. Windows varsayılanlarıyla karşılaştırın ve onu kullanacak URL/işlem etkinliğini inceleyin.

### SharedTaskScheduler {#shared_task_scheduler}

**Kontrol kimliği:** `shared_task_scheduler` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Boş olmayan Explorer `SharedTaskScheduler` değerlerini kayıtlı kabuk bileşenleri olarak listeler.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Değerler yürütülebilir bir yol yerine COM sınıflarına başvurabilir. Sunucu kayıtlarını izleyin ve işletim sistemi davranışını, uzantının kökenini ve ilgili kabuk yükleme kanıtlarını kontrol edin.

### ShellExecuteHooks {#shell_execute_hooks}

**Kontrol kimliği:** `shell_execute_hooks` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Boş olmayan Explorer `ShellExecuteHooks` değerlerini listeleyerek eski kabuk yürütme kancası kayıtlarını kapsar.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Başvurulan her CLSID'yi çözümleyin ve Windows derlemesinin ve ilkenin kancaya izin verip vermediğini doğrulayın. Eski bir kayıt kullanılmıyor olabilir; değerin kendisi gözlemlenmiş bir kabuk yürütmesi değildir.

### Başlat Menüsü Kitaplığı ve CLSID Klasörü COM Kalıcılığı {#start_menu_com}

**Kontrol kimliği:** `start_menu_com` · **Katalog ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei neyi kontrol eder:** `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu` altında, `simpleLocation` URL'si
bir COM CLSID'sine başvuran `.library-ms` kitaplık XML dosyalarını ve `<name>.{CLSID}` adlı dizinleri
bulur. Explorer'ın bu nesnelerle etkileşimi
COM kodunu etkinleştirebilir. Kontrol, toplanmış boş dizinleri de içerir; sembolik bağlantıları
izlemez. Başlat Menüsü dışındaki sıradan kitaplık dosyaları ve XML açıklamalarındaki GUID'ler bu kontrol
tarafından bildirilmez.

**İncelenen kanıtlar:**

- Kullanıcı başına Başlat Menüsü `.library-ms` dosyaları ve CLSID adlı dizin girdileri.
- Eşleşen kullanıcının toplanmış sınıf kayıtları ve makine SOFTWARE kovanı, mimariye göre ayrı ayrı.
- Zamanlanmış görevlerle aynı çözümleyici aracılığıyla yönetilen CodeBase/AppID/hizmet yapılandırması.

**Nasıl incelenir:** `Path` ve `SourceFile`, etkinleştirme tetikleyicisi artefaktını belirler. `Value`,
aday sunucu komutunu veya çözümleme yapılamıyorsa CLSID'yi içerir. `Description`,
tetikleyiciyi CLSID'ye ve toplanmış kayıt defteri kaynağına bağlar. `ResolvedPath`/`Sha256` yalnızca
çözümlenmiş, toplanmış bir yükü açıklar. Yalnızca yürütülebilir dosya adı içeren bir AppID ilişkilendirmesi çözümsüz kalır.
Bir tetikleyici meşru olabilir; varlığı bir oturum açmayı, etkinleştirmeyi, yürütmeyi veya saldırıyı kanıtlamaz.
Kitaplık URL'sini, sınıf kaydını, yükü, kullanıcı bağlamını ve bağımsız yürütme günlüklerini inceleyin.

**Toplama ile ilgili uyarılar:** Yalnızca dosya içeren toplamalar boş dizinleri atlayabilir. Standart kullanıcı Başlat Menüsü
yolları incelenir; yeniden yönlendirilmiş Başlat Menüleri ve diğer Explorer ad alanı konumları bu kontrol
kapsamında değildir. Eksik kovanlar veya çözümlenmemiş kayıtlar, adayı koruyarak kısmi kapsam üretir.
Görev mimarisi, yükseltme ve ilke etkili COM arama davranışını değiştirebilir.

**Daha fazla ayrıntı:** [Özgün öneri ve araştırmacı katkısı](https://github.com/last-byte/PersistenceSniper/pull/32),
[COM kullanıcı başına/makine birleştirilmiş görünümleri](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root),
[MITRE COM ele geçirme](https://attack.mitre.org/techniques/T1546/015/).

### TypeLib COM Ele Geçirme {#typelib_hijack}

**Kontrol kimliği:** `typelib_hijack` · **Katalog ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei neyi kontrol eder:** Kullanıcı başına UsrClass.dat TypeLib GUID/sürüm/yerel ayar win32/win64 kayıtlarını okur; yolları ve betik bilinen ad (script moniker) değerlerini korur. Sondaki sayısal kaynak dizinleri yalnızca hedef dosya çözümlemesi için kaldırılır.

**İncelenen kanıtlar:**

- Toplanmış her UsrClass.dat içindeki `TypeLib\<GUID>\<version>\<locale>\win32` ve `win64` varsayılan değerleri.

**Nasıl incelenir:** Bir tür kitaplığı meşru bir DLL'e, alternatif bir dosyaya veya bir betik bilinen adına işaret edebilir. Kontrol bu kayıtların envanterini çıkarır; varsayılan olmayan her yolun bir saldırı olduğunu varsaymak yerine etkilenen uygulamayı değerlendirin ve tür kitaplığı yüklemesini doğrulayın.

## Office uygulamaları ve eklentileri { #office-applications-and-add-ins }

Office meşru eklentileri, şablonları ve uygulamaya özgü uzantıları destekler. Şüpheli bir kayıt belirli bir Office sürümüne, uygulamaya, yükleme ayarına veya kullanıcı etkileşimine bağlı olabilir. Yapılandırma tek başına makro yürütmesini kanıtlamaz.

### Office Eklentisi Kaydı {#office_addins}

**Kontrol kimliği:** `office_addins` · **Katalog ATT&CK:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei neyi kontrol eder:** Uygulanmış makine Click-to-Run deposu ve kullanıcı kökleri dahil olmak üzere, sürümsüz ve 14.0/15.0/16.0 Office köklerindeki Word, Excel, PowerPoint, Outlook ve Access eklentilerini listeler. Açık Manifest/FileName/Path değerlerini korur veya uygun COM destekli girdileri LoadBehavior bağlamıyla birlikte çözümler.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` ve uygulanmış makine Click-to-Run eşdeğeri.
- NTUSER.DAT içindeki `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>`; SOFTWARE, NTUSER.DAT ve UsrClass.dat'tan eşleşen makine/kullanıcı COM sınıfları.

**Nasıl incelenir:** Eklentiler yaygın iş yazılımlarıdır. LoadBehavior'ı, uygulamayı/sürümü, bildirimleri (manifest), asıl yükleri ve beklenen satıcıyı inceleyin. Kontrol, yüklü her eklenti biçiminin veya kayıt görünümünün toplandığını ya da yüklendiğini garanti etmez.

### Office AI Eklentisi Ele Geçirme {#office_ai_hijack}

**Kontrol kimliği:** `office_ai_hijack` · **Katalog ATT&CK:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei neyi kontrol eder:** Office 16.0 Click-to-Run Common AI kayıt konumundaki değerleri listeler.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: tüm değerler.

**Nasıl incelenir:** Olağandışı değerleri ve yolları yüklü Office derlemesiyle karşılaştırın. Bir değerin varlığı, onu hangi Office özelliğinin kullandığını göstermez veya yürütülebilir kod seçtiğini kanıtlamaz.

### Office DLL Geçersiz Kılması {#office_dll_override}

**Kontrol kimliği:** `office_dll_override` · **Katalog ATT&CK:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei neyi kontrol eder:** Uygulanmış Click-to-Run ve kullanıcı kökleri dahil olmak üzere, sürümsüz ve 14.0/15.0/16.0 ayarları için Word/PowerPoint kökleri altındaki `WwlibtDll` ve `PPCoreTDLL` değerlerini okur.

**İncelenen kanıtlar:**

- Makine/kullanıcı `Microsoft\Office\[<version>\]Word` ve `PowerPoint` ayarları, ayrıca uygulanmış makine Click-to-Run Office ağacı: `WwlibtDll` ve `PPCoreTDLL`.

**Nasıl incelenir:** Geçersiz kılmayı hangi uygulamanın/derlemenin tanıdığını belirleyin, kitaplığı ve kökenini inceleyin ve bir Office başlatması veya görüntü yüklemesiyle ilişkilendirin. Yapılandırılmış bir geçersiz kılma tek başına yürütme kanıtı değildir.

### Office Varsayılan Şablonları {#office_templates}

**Kontrol kimliği:** `office_templates` · **Katalog ATT&CK:** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**Juntei neyi kontrol eder:** Toplanmış Normal.dotm/NormalEmail.dotm dosyalarının ve Office kurulum başlangıç klasörleri dahil olmak üzere tanınan Word Startup ve Excel XLSTART klasörlerinde doğrudan bulunan dosyaların envanterini çıkarır.

**İncelenen kanıtlar:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` ve `NormalEmail.dotm`.
- Roaming AppData altındaki kullanıcı `Microsoft\Word\STARTUP` ve `Microsoft\Excel\XLSTART` klasörlerinde doğrudan bulunan dosyalar.
- Program Files ve Program Files (x86) altındaki tanınan Office `<version>\STARTUP` / `XLSTART` ve `root\<version>\STARTUP` / `XLSTART` klasörlerinde doğrudan bulunan dosyalar.

**Nasıl incelenir:** Bir şablon meşru olabilir veya hiç yürütülebilir içerik barındırmayabilir. Juntei bu kontrolde makroları çıkarmaz veya yürütmez. Belgenin iç yapısını, makro ilkesini ve uygulama etkinliğini inceleyin; bir makine kurulum yolu, bir SYSTEM işlem belirteci anlamına gelmez.

### Office Test DLL {#office_test_dll}

**Kontrol kimliği:** `office_test_dll` · **Katalog ATT&CK:** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**Juntei neyi kontrol eder:** Listelenen makine/kullanıcı ve 32 bit makine görünümlerinden Office Test Special Perf anahtarındaki değerleri listeler.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | Boş olmayan tüm değerler |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | Boş olmayan tüm değerler |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Beklenmeyen geçersiz kılmaları Office sürümü ve yük doğrulaması gerektiren bir ipucu olarak ele alın. Kontrol, belirli bir Office kurulumunun değeri yükleyip yüklemediğini test etmek yerine yapılandırmayı bildirir.

### Outlook Giriş Sayfası Saldırısı {#outlook_home_page}

**Kontrol kimliği:** `outlook_home_page` · **Katalog ATT&CK:** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**Juntei neyi kontrol eder:** Office 14.0, 15.0 ve 16.0 sürümleri için kullanıcı Outlook WebView alt öğelerinin altındaki `URL` değerini okur.

**İncelenen kanıtlar:**

- NTUSER.DAT içindeki `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL`.

**Nasıl incelenir:** Bir klasör giriş sayfası meşru olabilir ve davranış Outlook sürümüne ve güvenlik güncelleştirmelerine göre değişir. URL'yi/içeriği ve etkili ilkeyi inceleyin, ardından Outlook/web/işlem kanıtlarıyla ilişkilendirin.

### VBA Monitor DLL Ele Geçirme {#vba_monitors}

**Kontrol kimliği:** `vba_monitors` · **Katalog ATT&CK:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei neyi kontrol eder:** Makine ve kullanıcı başına sınıf depolarında, uygulanmış iki VBA izleyicisiyle ilgili CLSID için InprocServer32 değerlerini okur.

**İncelenen kanıtlar:**

- Makine SOFTWARE kovanındaki `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` ve `{0002E157-0000-0000-C000-000000000046}`, ayrıca eşdeğer kullanıcı UsrClass.dat kayıtları.

**Nasıl incelenir:** Seçilen kitaplıkları yüklü Office/VBA ortamıyla karşılaştırın. Bu, her VBA uzantısının envanteri veya bir makronun çalıştığının kanıtı değil, hedefe yönelik bir CLSID kontrolüdür.

## Hata ayıklayıcılar, çökme işleyicileri ve uyumluluk shim'leri { #debuggers-crash-handlers-and-compatibility-shims }

Bir hata ayıklayıcı veya hata işleyicisi, bir uygulama başladığında, çöktüğünde, yanıt vermez hâle geldiğinde veya kapandığında çalışabilir. Uygulama uyumluluk veritabanları da başlangıcı etkileyebilir. Bildirilen komutla birlikte ön koşul ayarlarını ve tetikleyen programı inceleyin.

### AeDebug Hata Ayıklayıcı Ele Geçirme {#ae_debug}

**Kontrol kimliği:** `ae_debug` · **Katalog ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei neyi kontrol eder:** Tam zamanında (just-in-time) çökme hata ayıklama yapılandırmasının kullandığı yerel ve 32 bit AeDebug `Debugger` komutlarını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**Nasıl incelenir:** Geliştirme/hata ayıklama yazılımları bunu meşru olarak ayarlayabilir. Hata ayıklayıcı komutunun tamamını ve çökme tetikleyicisi ön koşullarını inceleyin, ardından uygulama hatası ve işlem olaylarıyla ilişkilendirin.

### AeDebug Korumalı İşlem Hata Ayıklayıcısı {#ae_debug_protected}

**Kontrol kimliği:** `ae_debug_protected` · **Katalog ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei neyi kontrol eder:** Korumalı işlem hata ayıklama yüzeyi için yerel ve 32 bit AeDebugProtected `Debugger` komutlarını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**Nasıl incelenir:** İşletim sistemi derlemesinin ve koruma ilkesinin yapılandırılmış komutu kullanıp kullanamayacağını belirleyin. Kaydedilmiş bir hata ayıklayıcı, korumalı bir işlemin çöktüğünün veya komutun çalıştığının kanıtı değildir.

### Image File Execution Options Hata Ayıklayıcısı {#ifeo_debugger}

**Kontrol kimliği:** `ifeo_debugger` · **Katalog ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei neyi kontrol eder:** İç içe filtre anahtarları dahil olmak üzere yerel/32 bit Image File Execution Options kayıtlarında `Debugger` değerlerini arar.

**İncelenen kanıtlar:**

- İç içe filtre anahtarları ve makinenin `Wow6432Node` görünümü dahil olmak üzere `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`.

**Nasıl incelenir:** Adı geçen görüntü başlatıldığında bir hata ayıklayıcı seçilebilir. Görüntü/filtre kapsamını ve hata ayıklayıcı bağımsız değişkenlerini inceleyin; geliştirici araçları da IFEO kullanabilir. Yürütmeyi kanıtlamak için hedefin başlatılmasını izleyin.

### IFEO Delegated NTDLL {#ifeo_delegated_ntdll}

**Kontrol kimliği:** `ifeo_delegated_ntdll` · **Katalog ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei neyi kontrol eder:** İç içe anahtarlar dahil olmak üzere yerel/32 bit Image File Execution Options kayıtlarında `VerifierDlls` geçersiz kılmalarını arar.

**İncelenen kanıtlar:**

- İç içe anahtarlar ve makinenin `Wow6432Node` görünümü dahil olmak üzere `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`.

**Nasıl incelenir:** Kontrolün koşulu yapılandırılmış VerifierDlls değeridir; tüm doğrulayıcı/yetki devri ön koşullarının etkin olduğunu kanıtlamaz. Çalışan bir enjeksiyon yolu çıkarmadan önce ilgili bayrakları, Windows sürümünü ve yükü inceleyin.

### Sessiz İşlem Çıkışı İzleyicisi {#ifeo_silent_process_exit}

**Kontrol kimliği:** `ifeo_silent_process_exit` · **Katalog ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei neyi kontrol eder:** İç içe anahtarlar dahil olmak üzere yerel/32 bit SilentProcessExit kayıtlarında `MonitorProcess` değerini arar.

**İncelenen kanıtlar:**

- İç içe anahtarlar ve makinenin `Wow6432Node` görünümü dahil olmak üzere `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`.

**Nasıl incelenir:** Sessiz çıkış izlemesi ayrıca ilgili raporlama/IFEO yapılandırmasını da gerektirir. Bu ayarları ve izlenen görüntünün çıkış olaylarını inceleyin; meşru tanılama araçları izleyiciyi açıklayabilir.

### Uygulama Shim'leme {#installed_sdb}

**Kontrol kimliği:** `installed_sdb` · **Katalog ATT&CK:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei neyi kontrol eder:** Yüklü uygulama uyumluluk veritabanları için doğrudan InstalledSDB kayıtlarından `DatabasePath` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**Nasıl incelenir:** Uyumluluk düzeltmeleri meşru olabilir. SDB içeriğini, hedeflenen yürütülebilir dosyaları ve kurulum geçmişini inceleyin. Bir veritabanı kaydı, shim'lerini tanımlamaz veya eşleşen bir uygulamanın çalıştığını kanıtlamaz.

### LSM Hata Ayıklayıcı Ele Geçirme {#lsm_debugger}

**Kontrol kimliği:** `lsm_debugger` · **Katalog ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei neyi kontrol eder:** Yerel ve 32 bit makine görünümlerinde özellikle `lsm.exe` için SilentProcessExit altındaki `MonitorProcess` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**Nasıl incelenir:** Bu, daha geniş SilentProcessExit envanteriyle örtüşür. Yapılandırmayı yürütülmüş kalıcılık olarak değerlendirmeden önce ön koşul bayraklarını ve ilgili bir LSM çıkışını doğrulayın; ayrı bulguları değerlendirirken aynı girdiyi bir kez sayın.

### MiniDump Yardımcı DLL'leri {#minidump_auxiliary_dlls}

**Kontrol kimliği:** `minidump_auxiliary_dlls` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** MiniDumpAuxiliaryDlls içindeki adlandırılmış, boş olmayan değerleri listeler; değer adını tetikleyici modül bağlamı, değer verisini ise seçilen hedef olarak korur.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: tetikleyici modül değer adları ve hedef verileri.

**Nasıl incelenir:** Hem tetikleyici modülü hem de yardımcı DLL'i inceleyin. Çökme/minidump araçları bir girdiyi açıklayabilir ve bir kayıt, bir döküm işleminin kitaplığı yüklediğini göstermez.

### Uygulama Shim'leme {#shim_custom}

**Kontrol kimliği:** `shim_custom` · **Katalog ATT&CK:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei neyi kontrol eder:** Doğrudan AppCompatFlags Custom uygulama anahtarlarının altındaki `.sdb` adlı değerleri okur ve uygulama ile veritabanı arasındaki ilişkilendirmeyi korur.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: `.sdb` ile biten değer adları.

**Nasıl incelenir:** Bu ilişkilendirmeleri InstalledSDB ve asıl uyumluluk veritabanıyla eşleştirin. Bir dosya adı ilişkilendirmesi tek başına veritabanının davranışını ortaya koymaz veya eşleşen bir uygulama başlatmasını göstermez.

### WER Hata Ayıklayıcı Ele Geçirme {#wer_debugger}

**Kontrol kimliği:** `wer_debugger` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Yerel/32 bit Windows Error Reporting `Debugger` değerlerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**Nasıl incelenir:** Meşru tanılama yazılımları bir hata ayıklayıcı yapılandırabilir. Komutun tamamını ve ilgili çökme olaylarını inceleyin; kayıt, Windows Error Reporting'in onu başlattığının kanıtı değildir.

### WER Hangs Hata Ayıklayıcı Ele Geçirme {#wer_hangs}

**Kontrol kimliği:** `wer_hangs` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Yerel/32 bit Windows Error Reporting Hangs `Debugger` değerlerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**Nasıl incelenir:** Bir yanıt vermeme (hang) işleme iş akışının bu ayarı çağırıp çağırmayacağını kontrol edin ve hedef programı inceleyin. Bir kayıt defteri yazmasını hata ayıklayıcı yürütmesi olarak değerlendirmek yerine uygulama yanıt vermeme ve işlem kayıtlarıyla ilişkilendirin.

### WER ReflectDebugger Ele Geçirme {#wer_reflect_debugger}

**Kontrol kimliği:** `wer_reflect_debugger` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Yerel/32 bit Windows Error Reporting `ReflectDebugger` değerlerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**Nasıl incelenir:** Onaylı tanılama yapılandırmasıyla karşılaştırın ve Windows sürümü/tetikleyici uygulanabilirliğini belirleyin. Yürütme atfetmeden önce komut bağımsız değişkenlerini inceleyin ve çökme/işlem doğrulaması arayın.

### WER Çalışma Zamanı Özel Durum Modülleri {#wer_runtime_exception}

**Kontrol kimliği:** `wer_runtime_exception` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Yerel/32 bit RuntimeExceptionHelperModules altındaki değer adlarını listeler; burada kitaplık yolu, verisinde olmak zorunda değildir, değer adında taşınır.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` ve makinenin `Wow6432Node` görünümü: değer adları.

**Nasıl incelenir:** Adı geçen modülü ve uygulama/çalışma zamanı bağlamını inceleyin. Verideki küçük bir sayısal değer yük yolu değildir ve kayıt tek başına yardımcının yüklendiğini kanıtlamaz.

## .NET çalışma zamanı ve profil oluşturucu ayarları { #net-runtime-and-profiler-settings }

Yönetilen çalışma zamanları, uygulama kodundan önce profil oluşturucuları, başlangıç derlemelerini (assembly) veya hata ayıklama kitaplıklarını yükleyebilir. Bu ayarlar geliştiriciler ve izleme ürünleri tarafından da kullanılır. Çalışma zamanını, işlem mimarisini, etkinleştirme bayraklarını ve yeni başlatılan bir işlemin ayarı devralıp devralamayacağını kontrol edin.

### AppDomainManager Enjeksiyonu {#appdomain_manager}

**Kontrol kimliği:** `appdomain_manager` · **Katalog ATT&CK:** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**Juntei neyi kontrol eder:** Özel bir .NET AppDomain yöneticisi için derleme ve tür seçimini kapsayan makine/kullanıcı `APPDOMAIN_MANAGER_ASM` ve `APPDOMAIN_MANAGER_TYPE` ortam değişkenlerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**Nasıl incelenir:** Uygulamalar özel yöneticileri bilinçli olarak kullanabilir. Derlemeyi ve türü birlikte çözümleyin ve çalışma zamanı uyumluluğunu ve işlem devralmasını kontrol edin; ne tek bir değişken ne de anahtarının zaman damgası, yönetilen bir uygulamanın onu yüklediğini kanıtlar.

### .NET CLR Profil Oluşturucu Ele Geçirme {#cor_profiler}

**Kontrol kimliği:** `cor_profiler` · **Katalog ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei neyi kontrol eder:** Makine/kullanıcı COR profil oluşturma ortam ayarlarını ve tek tek hizmetlerin Environment çoklu dizelerindeki eşleşen adlandırılmış girdileri okur. Etkinleştirme bayraklarını, profil oluşturucu tanımlayıcılarını ve mimariye özgü yolları korur.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` ve toplanmış her kullanıcı NTUSER.DAT `Environment` anahtarı.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: çoklu dizedeki adlandırılmış COR ayarları.

**Nasıl incelenir:** Profil oluşturucular meşru izlemeyi destekler. Bir etkinleştirme bayrağı, CLSID ve DLL yolu farklı yapılandırma parçalarıdır; her satırı yürütülmüş bir profil oluşturucu olarak değerlendirmek yerine etkili etkinleştirmeyi ve ilgili .NET işlemini doğrulayın.

### .NET Core CLR Profil Oluşturucu Ele Geçirme {#coreclr_profiler}

**Kontrol kimliği:** `coreclr_profiler` · **Katalog ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei neyi kontrol eder:** Mimariye özgü yollar dahil olmak üzere karşılık gelen makine/kullanıcı CORECLR profil oluşturma ortam ayarlarını ve eşleşen hizmet Environment girdilerini okur.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` ve toplanmış her kullanıcı NTUSER.DAT `Environment` anahtarı.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: çoklu dizedeki adlandırılmış CORECLR ayarları.

**Nasıl incelenir:** .NET çalışma zamanını, mimariyi, etkinleştirme bayrağını ve kitaplığı doğrulayın. Onaylı gözlemlenebilirlik ürünleri bu ayarları yapılandırabilir ve mevcut bir işlem yeni değiştirilmiş bir ortamı devralmayabilir.

### .NET DbgManagedDebugger Ele Geçirme {#dotnet_dbg_managed_debugger}

**Kontrol kimliği:** `dotnet_dbg_managed_debugger` · **Katalog ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei neyi kontrol eder:** Açık makine 32 bit yolu dahil olmak üzere listelenen makine/kullanıcı `.NETFramework` `DbgManagedDebugger` ayarlarını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**Nasıl incelenir:** İlgili .NET çalışma zamanı/hata ayıklama ilkesinin bu komutu kullanıp kullanmadığını belirleyin. Geliştirici araçları bunu açıklayabilir; yürütmeyi kanıtlamak için yönetilen çökmeler ve işlem oluşturmayla ilişkilendirin.

### .NET Framework COR_PROFILER Kayıt Defteri Anahtarı {#dotnet_framework_profiler}

**Kontrol kimliği:** `dotnet_framework_profiler` · **Katalog ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei neyi kontrol eder:** Listelenen `.NETFramework` kayıt defteri görünümlerinden `COR_PROFILER`, `COR_PROFILER_PATH` ve `COR_ENABLE_PROFILING` değerlerini okur. Bu, ortam tabanlı profil oluşturucu kontrollerini tamamlar.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**Nasıl incelenir:** Değerleri birlikte değerlendirin ve çalışma zamanının bu kayıt defteri konumunu dikkate alıp almadığını belirleyin. Yapılandırılmış bir profil oluşturma aracı meşru olabilir; tek başına bir etkinleştirme bayrağını, tanımlanmış bir DLL yüklemesi olarak yorumlamayın.

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**Kontrol kimliği:** `dotnet_startup_hooks` · **Katalog ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei neyi kontrol eder:** Desteklenen .NET çalışma zamanları için başlangıç kancası derlemelerini seçebilen makine/kullanıcı `DOTNET_STARTUP_HOOKS` ortam değerlerini okur.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` ve her kullanıcının NTUSER.DAT `Environment\DOTNET_STARTUP_HOOKS` değeri.

**Nasıl incelenir:** Birden fazla derleme için değerin tamamını inceleyin ve çalışma zamanı desteğini doğrulayın. Bu kontrol, hizmete özgü başlangıç kancası ortamlarını listelemez veya herhangi bir işlemin değeri devralıp kullandığını kanıtlamaz.

### Known Managed Debugging DLLs {#known_managed_debugging_dlls}

**Kontrol kimliği:** `known_managed_debugging_dlls` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** `.NETFramework` `KnownManagedDebuggingDlls` kayıtlarını olası yönetilen hata ayıklayıcı kitaplık geçersiz kılmaları olarak okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**Nasıl incelenir:** Yüklü geliştirme/çalışma zamanı araçlarıyla karşılaştırın ve tetikleyen hata ayıklayıcıyı belirleyin. Kayıt defteri girdisi bir hata ayıklayıcı oturumunu veya kitaplık yüklemesini kanıtlamaz.

## DLL ve yürütülebilir dosya arama geçersiz kılmaları { #dll-and-executable-lookup-overrides }

Bu kontroller, açık kitaplık konumlarını veya bir bileşenin kodu nasıl bulduğunu etkileyen ayarları inceler. Bir yol geçersiz kılması veya zayıflatılmış bir arama ilkesi bir inceleme ipucudur; yürütme için yine de bir yük ve uyumlu bir tetikleyici bileşen gerekir.

### AppCert DLL'leri {#appcert_dlls}

**Kontrol kimliği:** `appcert_dlls` · **Katalog ATT&CK:** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**Juntei neyi kontrol eder:** İşlem oluşturma API'leriyle ilişkili bir kitaplık uzantı yüzeyi olan boş olmayan Session Manager AppCertDlls kayıtlarını listeler.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | Boş olmayan tüm değerler |

**Nasıl incelenir:** Her kitaplığı ve kurulum kökenini inceleyin. Uygulanabilirliği ve çalışma zamanı yüklemesini ayrı ayrı belirleyin; bir zaman çizelgesindeki normal işlem oluşturma, tek başına yapılandırılmış DLL'in yüklendiğinin kanıtı değildir.

### AppInit DLL'leri {#appinit_dlls}

**Kontrol kimliği:** `appinit_dlls` · **Katalog ATT&CK:** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**Juntei neyi kontrol eder:** Yerel/32 bit `AppInit_DLLs` değerini okur, kitaplık listesini böler ve her adaya mevcut `LoadAppInit_DLLs` ve `RequireSignedAppInit_DLLs` bağlamını ekler.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs`, `LoadAppInit_DLLs` ve `RequireSignedAppInit_DLLs`; ayrıca makinenin `Wow6432Node` görünümü.

**Nasıl incelenir:** Sıfır değerli bir LoadAppInit_DLLs bayrağı açıkça etkin değil olarak işaretlenir. Windows sürümü, Secure Boot ve imzalama gereksinimleri de yüklemeyi etkiler. Bir AppInit DLL'inin yürütüldüğünü öne sürmeden önce etkili ayarları inceleyin.

**Daha fazla ayrıntı:** [Microsoft belgeleri](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls).

### AutodialDLL Geçersiz Kılması {#autodial_dll}

**Kontrol kimliği:** `autodial_dll` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** WinSock2 Parameters `AutodialDLL` geçersiz kılmasını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**Nasıl incelenir:** Bu ayarı kullanan ağ özelliğini ve kaynak Windows sürümünü kontrol edin. Asıl DLL'i ve ağ/işlem etkinliğini inceleyin; bir geçersiz kılma tek başına etkin veya kötü amaçlı bir yüklemeyi kanıtlamaz.

### CHM Yardımcı DLL'i {#chm_helper_dll}

**Kontrol kimliği:** `chm_helper_dll` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Yardımla ilgili bir kitaplık/konum geçersiz kılma yüzeyi olan HtmlHelp Author altındaki `Location` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**Nasıl incelenir:** İlgili yardım bileşeninin bu sistemde değeri kullanıp kullanmadığını belirleyin ve seçilen dosyayı inceleyin. Bir kayıt defteri geçersiz kılması, bir CHM belgesinin açıldığının kanıtı değildir.

### NL6 ContentIndex DLL Geçersiz Kılması {#content_index_dll}

**Kontrol kimliği:** `content_index_dll` · **Katalog ATT&CK:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei neyi kontrol eder:** Etkin denetim kümesindeki doğrudan ContentIndex Language kayıtlarından `DLLOverridePath` değerini okur.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`.

**Nasıl incelenir:** Dil/arama bileşenleri Windows derlemesine göre farklılık gösterebilir. Kitaplığı ve eşleşen yüklü bileşeni inceleyin; yüklemeyi doğrulamak için asıl dizin oluşturma işlemini izleyin.

### Crypto ExpoOffload DLL {#crypto_expo_offload}

**Kontrol kimliği:** `crypto_expo_offload` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Cryptography Offload `ExpoOffload` değerini bir şifreleme kitaplığı geçersiz kılma adayı olarak okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**Nasıl incelenir:** Özellik/derleme uygulanabilirliğini doğrulayın ve seçilen kitaplığın kökenini inceleyin. Nadir görülen bir değerin varlığını kimlik bilgisi hırsızlığının veya yürütülmüş kodun kanıtı olarak değerlendirmeyin.

### Direct3D Yazılım Rasterleştirici DLL'i {#direct3d_dll}

**Kontrol kimliği:** `direct3d_dll` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Direct3D Drivers `SoftwareRasterizer` kitaplık geçersiz kılmasını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**Nasıl incelenir:** Grafik ve uyumluluk yazılımları rasterleştirici yapılandırmasını etkileyebilir. DLL'i ve uyumlu bir Direct3D uygulamasının etkinliğini inceleyin; yolun kaydedilmesi yüklendiğini kanıtlamaz.

### DLL Arama Sırasının Zayıflatılması {#dll_search_mode}

**Kontrol kimliği:** `dll_search_mode` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** `SafeDllSearchMode` veya `CWDIllegalInDllSearch` değerlerini yalnızca etkin Session Manager değeri sıfır olarak ayrıştırıldığında bildirir. Bunlar seçilmiş yükler değil, arama ilkesi koşullarıdır.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` ve `CWDIllegalInDllSearch`.

**Nasıl incelenir:** Zayıflatılmış bir arama ilkesi, ayrı bir arama sırası saldırısını kolaylaştırabilir. Yerleştirilmiş bir DLL'i tanımlamaz veya yürütmeyi kanıtlamaz. Sıfır olmayan veya bulunmayan değerler bu kontrol tarafından çıktı olarak verilmez; etkili varsayılanları ve ilkeyi ayrıca değerlendirin.

**Daha fazla ayrıntı:** [Microsoft belgeleri](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security).

### KnownDLLs Hariç Tutması {#exclude_from_known_dlls}

**Kontrol kimliği:** `exclude_from_known_dlls` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Session Manager'daki boş olmayan `ExcludeFromKnownDlls` girdilerini korumalı/bilinen DLL arama davranışındaki değişiklikler olarak okur.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`.

**Nasıl incelenir:** Bir hariç tutma, değiştirme kitaplığının kendisini değil, bir arama koşulunu belirler. Etkilenen uygulamayı ve olası DLL arama konumlarını bulun, ardından standart dışı bir görüntü yüklemesini doğrulayın.

**Daha fazla ayrıntı:** [Microsoft belgeleri](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order).

### hhctrl.ocx DLL Geçersiz Kılması {#hhctrl_ocx_dll}

**Kontrol kimliği:** `hhctrl_ocx_dll` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Uygulanmış HTML Help denetimi CLSID'si için yerel/32 bit makine ve desteklenen kullanıcı sınıf deposu InprocServer32 geçersiz kılmalarını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**Nasıl incelenir:** Denetimi hangi uygulamanın ve mimarinin etkinleştirdiğini belirleyin. Kayıtlı DLL'i ve yardımla ilgili etkinliği inceleyin; bir COM kaydı, HTML Help'in içerik işlediğini kanıtlamaz.

### Known DLLs {#known_dlls}

**Kontrol kimliği:** `known_dlls` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Etkin Session Manager KnownDLLs anahtarındaki adlandırılmış değerleri listeler.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: adlandırılmış değerler.

**Nasıl incelenir:** Girdilerin çoğu beklenen Windows kitaplıklarıdır ve bazı değerler bir yük yolu değil, destekleyici yapılandırmadır. Anahtarın tamamını güvenilir, eşleşen bir derlemeyle karşılaştırın; bu kontrol mimariye özgü her alternatif KnownDLLs ağacını taramaz.

### LSA Uzantıları DLL'i {#lsa_extensions}

**Kontrol kimliği:** `lsa_extensions` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** LsaExtensionConfig LsaSrv yapılandırmasından `Extensions` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**Nasıl incelenir:** Güvenlik yazılımları kimlik doğrulama bileşenlerini genişletebilir. Asıl kitaplıkları ve kaynak Windows sürümüyle uyumluluğunu inceleyin, ardından LSASS görüntü yükleme veya başka bağımsız kullanım kanıtı arayın.

### MAPI32 DLL Yolu Geçersiz Kılması {#mapi32_dll_path}

**Kontrol kimliği:** `mapi32_dll_path` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Yerel ve 32 bit makine görünümlerinde doğrudan Mail istemcisi kayıtlarının altındaki `DLLPathEx` ve `DLLPath` değerlerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**Nasıl incelenir:** Posta istemcileri meşru MAPI sağlayıcıları kaydeder. Hangi istemcinin/sağlayıcının seçildiğini belirleyin ve asıl kitaplığı inceleyin; yüklü bir sağlayıcı, bir uygulamanın olay sırasında MAPI kullandığını kanıtlamaz.

### MSDTC XA DLL {#msdtc_xa_dll}

**Kontrol kimliği:** `msdtc_xa_dll` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** MSDTC MTxOCI yapılandırmasından `OracleXaLib` ve `OracleOciLib` değerlerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**Nasıl incelenir:** Veritabanı tümleştirmesi bu kitaplıkları meşru olarak gerektirebilir. İşlem koordinatörü yapılandırmasını ve yüklü veritabanı yazılımını doğrulayın; yapılandırılmış bir yolun çalıştığını varsaymak yerine hizmet/modül kullanımıyla ilişkilendirin.

### NLDP DLL Geçersiz Kılması {#nldp_dll}

**Kontrol kimliği:** `nldp_dll` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** NlsData `DllOverridePath` yapılandırma değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**Nasıl incelenir:** Hedef kitaplığı, Windows derlemesini ve geçersiz kılmayı kullanabilecek doğal dil bileşenini inceleyin. Değer, dizin oluşturmanın veya dil işlemenin gerçekleştiğini göstermez.

### RDP TestDVCPlugin DLL {#rdp_test_dvc_plugin}

**Kontrol kimliği:** `rdp_test_dvc_plugin` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Makinenin Terminal Server Client `TestDVCPlugin` kitaplık ayarını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**Nasıl incelenir:** Tanıdık olmayan bir test eklentisini ve sürüme özgü davranışı inceleyin. Yürütmeyi kanıtlamak için uyumlu bir istemci oturumu ve kitaplık yüklemesi gerekir; kayıt defteri değeri tek başına bunların hiçbirini göstermez.

### Search Indexer DLL Geçersiz Kılması {#search_indexer_dll}

**Kontrol kimliği:** `search_indexer_dll` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** SOFTWARE kovanından Windows Search `DllPath` geçersiz kılmasını okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**Nasıl incelenir:** Eşleşen Search bileşenini/sürümünü belirleyin ve hedef dosyayı inceleyin. Etkin olmayan bir Windows Search hizmeti veya kullanılmayan bir ayar, gözlemlenmiş kötü amaçlı bir kitaplık yüklemesinden farklıdır.

### Winsock AutoProxy DLL {#winsock_auto_proxy}

**Kontrol kimliği:** `winsock_auto_proxy` · **Katalog ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei neyi kontrol eder:** Doğrudan NameSpace_Catalog5 Catalog_Entries ve Catalog_Entries64 kayıtlarının altındaki `LibraryPath` değerini okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**Nasıl incelenir:** Ağ sağlayıcıları meşru olabilir. Her iki mimariyi, sağlayıcı kökenini ve ilgili ağ uygulaması/görüntü yükleme kanıtlarını inceleyin. Bu hedefe yönelik ad alanı sağlayıcısı araması, olası her Winsock uzantısını kapsamaz.

## Hesaplar ve oturum açma görünürlüğü { #accounts-and-sign-in-visibility }

Yerel hesap kayıtları ve oturum açma görünürlüğü ayarları kimlik tabanlı kalıcılığı ortaya çıkarabilir. Bu kontroller toplanmış SAM ve SOFTWARE kovanlarını inceler; Active Directory hesaplarının, etki alanı grup üyeliklerinin veya geçmiş oturum açmaların envanteri değildir.

### Administrators Grubu Üyeliği {#admin_group_membership}

**Kontrol kimliği:** `admin_group_membership` · **Katalog ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei neyi kontrol eder:** SAM'den yerel Administrators diğer ad (alias) üye listesinin kodunu çözer. Toplanmış SAM'de gizli bir hesabı adlandıran veya hiçbir hesabı/grubu adlandırmayan yerel makine SID'lerini bildirir.

**İncelenen kanıtlar:**

- SAM kovanındaki `SAM\Domains\Builtin\Aliases\00000220\C` ve `SAM\Domains\Account` altındaki makine SID'si/kullanıcı/grup kayıtları.
- Gizli ad ilişkilendirmesi için `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`.

**Nasıl incelenir:** Bu, tüm yöneticilerin listesi değil, hedefe yönelik bir anormallik kontrolüdür. Harici/etki alanı SID'leri eksik yerel kullanıcılar olarak değerlendirilmez. Silinmiş hesaplar, ele geçirme zamanlaması veya eksik kovanlar, kötü amaçlı bir üyelik sonucuna varmadan önce inceleme gerektirir.

### Gizli Yerel Hesap {#hidden_account}

**Kontrol kimliği:** `hidden_account` · **Katalog ATT&CK:** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**Juntei neyi kontrol eder:** Winlogon SpecialAccounts UserList içindeki sayısal sıfır değerlerini bildirerek olağan oturum açma ekranından gizlenen adları belirler.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: hesap adı değerleri.

**Nasıl incelenir:** Hizmet veya bakım hesapları bilinçli olarak gizlenebilir ve değer, adı geçen hesabın var olduğunu veya bir yönetici olduğunu kanıtlamaz. SAM hesap kayıtları ve oturum açma etkinliğiyle birleştirin.

### RID Ele Geçirme {#rid_hijacking}

**Kontrol kimliği:** `rid_hijacking` · **Katalog ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei neyi kontrol eder:** Bir SAM hesap alt anahtarının onaltılık RID'sini, F değerindeki 0x30 ofsetinde saklanan RID ile karşılaştırır ve uyuşmazlıkları bildirir.

**İncelenen kanıtlar:**

- SAM kovanındaki `SAM\Domains\Account\Users\<hexadecimal RID>\F`.

**Nasıl incelenir:** Bir uyuşmazlık ciddi bir hesap kaydı anormalliğidir, ancak bozulma/ele geçirme analizi de gerektirebilir. Kaynak baytları, işlem günlüğü durumunu ve hesap/oturum açma kayıtlarını inceleyin; bu, kaydı hangi aktörün değiştirdiğini kanıtlamaz.

### RID Suborner (Gizli Yönetici Hesabı) {#rid_suborner}

**Kontrol kimliği:** `rid_suborner` · **Katalog ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei neyi kontrol eder:** Alt anahtar RID'si 500 olmayan, ancak F değeri RID'si 500 olan SAM hesaplarını bildirir; bu, daha belirli bir yönetici kimliği ikame örüntüsüdür.

**İncelenen kanıtlar:**

- SAM kovanındaki `SAM\Domains\Account\Users\<hexadecimal RID>\F`.

**Nasıl incelenir:** Bu, genel RID uyuşmazlığı kontrolüyle örtüşür. Katalog başlığına rağmen bağımsız olarak gizli bir oturum açma ayarı gerektirmez; hesap görünürlüğünü, ayrıcalıkları ve gerçek hesap kullanımını ayrıca doğrulayın.

## Kimlik doğrulama bileşenleri ve kimlik bilgisi koruması { #authentication-components-and-credential-protection }

Local Security Authority (LSA), güvenlik işlevlerini LSASS ve ilgili bileşenler aracılığıyla yerine getirir. Kimlik doğrulama paketleri, parola filtreleri ve ağ sağlayıcıları meşru güvenlik yazılımları olabileceği gibi kalıcı kimlik bilgisi erişimi için bir yol da olabilir. İki kontrol, yürütülebilir otomatik başlatmalar yerine koruma ayarlarının envanterini çıkarır. Bunları ana bilgisayar rolü, Windows sürümü ve etkili ilkeyi kullanarak yorumlayın.

### AMSI Sağlayıcı DLL'i {#amsi_providers}

**Kontrol kimliği:** `amsi_providers` · **Katalog ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei neyi kontrol eder:** Makine AMSI sağlayıcı CLSID'lerini listeler ve mevcut olduğunda yerel/32 bit InprocServer32 kitaplıklarını çözümler. Çözümlenmemiş kayıtlar sınıf kimliğini korur.

**İncelenen kanıtlar:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` ve çözümlenebilen `Classes\CLSID` / `Classes\Wow6432Node\CLSID` InprocServer32 kayıtları.

**Nasıl incelenir:** Virüsten koruma ve güvenlik ürünleri AMSI sağlayıcılarını meşru olarak kaydeder. Sağlayıcının/kitaplığın kökenini ve gerçek yüklemeyi inceleyin; bir kayıt, AMSI'nin atlatıldığını veya sağlayıcının kötü amaçlı olduğunu kanıtlamaz.

### Kimlik Doğrulama Paketleri {#authentication_packages}

**Kontrol kimliği:** `authentication_packages` · **Katalog ATT&CK:** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**Juntei neyi kontrol eder:** Etkin denetim kümesinden boş olmayan LSA `Authentication Packages` girdilerini tek tek okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**Nasıl incelenir:** Kaynak Windows derlemesi ve yüklü kimlik doğrulama yazılımıyla karşılaştırın. Varsayılan olmayan bir paket meşru olabilir. Araya girme sonucuna varmadan önce DLL'i çözümleyin ve yükleme/kimlik bilgisi etkinliğini doğrulayın.

### DSRM Yönetici Oturum Açma Davranışı {#dsrm_backdoor}

**Kontrol kimliği:** `dsrm_backdoor` · **Katalog ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei neyi kontrol eder:** Uygulanmış ControlSet001, ControlSet002 veya CurrentControlSet konumlarındaki sayısal `DsrmAdminLogonBehavior=2` değerini bildirir. Bu ayar, DSRM hesabının olağan kurtarma bağlamı dışında kullanılmasına izin verebilir.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior`, ayrıca mevcut olduğunda ControlSet002 ve CurrentControlSet.

**Nasıl incelenir:** Öncelikle etki alanı denetleyicileriyle ilgilidir. SYSTEM Select kullanarak hangi denetim kümesinin etkin olduğunu belirleyin; geçmişe ait bir denetim kümesindeki eşleşme etkili olmak zorunda değildir. Değer, bir saldırganın DSRM parolasını bildiğini veya oturum açtığını kanıtlamaz.

### Credential Guard Yapılandırması {#lsa_cfg_flags}

**Kontrol kimliği:** `lsa_cfg_flags` · **Katalog ATT&CK:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei neyi kontrol eder:** Sıfırın yanı sıra olağan etkin değerler de dahil olmak üzere yapılandırılmış LSA `LsaCfgFlags` değerini okur. Bu, yürütülebilir bir otomatik başlatma yerine Credential Guard yapılandırmasının envanterini çıkarır.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**Nasıl incelenir:** Değeri Windows sürümü, ürün yazılımı/güvenlik ön koşulları ve etkili ilkeyle birlikte yorumlayın. Saklanmış bir sıfır veya bulunmayan bir değer tek başına bir saldırganın yaptığı bir değişikliği, etkili koruma durumunu veya kimlik bilgisi hırsızlığını kanıtlamaz.

### LSA Parola Filtresi {#lsa_password_filter}

**Kontrol kimliği:** `lsa_password_filter` · **Katalog ATT&CK:** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**Juntei neyi kontrol eder:** LSA `Notification Packages` değerini okur. Yalın kitaplık adları, uygun olduğunda `.dll` eklenerek toplanmış Windows System32 dizinine göre çözümlenir.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**Nasıl incelenir:** Parola ilkesi ürünleri sıklıkla filtre yükler. Kitaplığı ve yükleme kanıtlarını inceleyin ve yapılandırılmış filtreleri, bir parola değişikliğinin veya kimlik bilgisi araya girmesinin gerçekleştiğinin kanıtından ayırt edin.

### LSASS PPL Koruma Durumu {#lsa_run_as_ppl}

**Kontrol kimliği:** `lsa_run_as_ppl` · **Katalog ATT&CK:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei neyi kontrol eder:** Sıfır olmayan değerler dahil olmak üzere yapılandırılmış LSA `RunAsPPL` değerini okur. Bu, LSASS'in korumasız çalıştığına dair bir test değil, bir koruma ayarı envanteridir.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**Nasıl incelenir:** Etkili LSA koruması Windows sürümüne, ilkeye, ürün yazılımı kilitlemesine ve yeniden başlatma durumuna bağlıdır. Bulunmayan bir değer için satır olmaması, ölçülmüş bir koruma sonucu değildir. Etkili korumayı değerlendirmek için bağımsız sistem/çalışma zamanı kanıtlarını kullanın.

### Ağ Sağlayıcısı DLL'i {#network_provider_dll}

**Kontrol kimliği:** `network_provider_dll` · **Katalog ATT&CK:** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**Juntei neyi kontrol eder:** Etkin SYSTEM denetim kümesindeki hizmetlerin altındaki `NetworkProvider\ProviderPath` değerini okur.

**İncelenen kanıtlar:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`.

**Nasıl incelenir:** Ağ oturum açma sağlayıcıları meşru olabilir. Sağlayıcı sırasını/uygulanabilirliğini, yüklü ağ/güvenlik yazılımlarını ve gerçek kitaplık yüklemesini kontrol edin; bir ProviderPath değeri tek başına kimlik bilgisi araya girmesini kanıtlamaz.

### Güvenlik Paketleri {#security_packages}

**Kontrol kimliği:** `security_packages` · **Katalog ATT&CK:** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**Juntei neyi kontrol eder:** Yapılandırılmış güvenlik destek sağlayıcılarını kapsayan boş olmayan LSA `Security Packages` girdilerini tek tek okur.

**İncelenen kanıtlar:**

| Kayıt defteri anahtarı | İncelenen değer |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**Nasıl incelenir:** Beklenen Windows/güvenlik ürünü temel çizgisiyle karşılaştırın ve her kitaplığı inceleyin. Yapılandırma, bir sağlayıcının yüklendiğini, kimlik bilgilerini ele geçirdiğini veya olay sırasında eklendiğini kanıtlamaz.

## Bir adaydan bir sonuca ulaşmak { #work-from-a-candidate-to-a-conclusion }

1. `SourceFile` ile belirtilen özgün kaynağı açın. Değerin tamamını, XML eylemini,
   betiği veya kurtarılmış veritabanı kaydını ve ait olduğu kullanıcıyı/ana bilgisayarı tam olarak doğrulayın.
2. Mekanizmanın çalışıp çalışamayacağını belirleyin: etkinleştirme bayraklarını, çalışma zamanını veya Windows sürümünü,
   hizmet/görev durumunu, gerekli tetikleyiciyi, kullanıcıyı, mimariyi ve etkin denetim kümesini kontrol edin.
3. Seçilen programı, DLL'i, betiği veya derlemeyi inceleyin. Toplanmış baytların karmasını alın ve bağımsız değişkenleri
   ve içeriği inceleyin. Tanınabilir bir Windows başlatıcısı veya yayımcı dizesi meşruiyeti kesinleştirmez.
4. Gerçekte ne olduğunu bağımsız kanıtlarla belirleyin. Örnekler arasında işlem oluşturma,
   Sysmon görüntü yüklemeleri, Görev Zamanlayıcı çalışma zamanı kayıtları, hizmet etkinliği, Prefetch ve ilgili dosyalar bulunur.
   Bir yapılandırma değişikliği olayı, yükünün çalıştığını kanıtlamadan ayarın değiştiğini kanıtlayabilir.
5. Özgün kaynak zaman damgalarını ve kaydedilen ilişkilendirme kökenini kullanarak bir zaman çizelgesi oluşturun.
   Anahtar yazmalarını, kurulum değişikliklerini, dosya/kapsayıcı yazmalarını ve yürütmeleri ayrı olgular olarak tutun.
6. Bulguların yanı sıra sınırlamaları da kaydedin: eksik kovanlar, eksik hedef dosyalar, bilinmeyen görev sorumluları,
   doğrulanmamış imzalayan meta verileri, geçmişe ait kazınmış kayıtlar veya doğrulanamayan bir tetikleyici.

Komut sözdizimi, profiller, her çıktı alanı ve kapsam durumları için
[Persistence Hunter komut kılavuzuna](../commands/persistence-hunter.md) dönün.
[Artefakt kılavuzları](../artifacts/index.md) temel kaynakları açıklar ve
[Teşekkürler](acknowledgements.md) sayfası üst akıştaki (upstream) kalıcılık araçlarını belirtir.
