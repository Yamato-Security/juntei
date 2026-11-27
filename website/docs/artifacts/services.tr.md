# Windows Hizmetleri { #windows-services }

## Hizmet yapılandırması neleri kaydeder { #what-service-configuration-records }

Windows hizmetleri ve çekirdek sürücüleri her `SYSTEM` denetim kümesi (control set) altında yapılandırılır. Kayıtlar
yürütülebilir dosyayı/sürücüyü veya hizmet DLL'ini, başlatma modunu, hizmet türünü, hesabı, yükleme sırası grubunu, gecikmeli
başlatmayı, hata davranışını ve Windows'un bir hatanın ardından gerçekleştirdiği eylemleri adlandırır.

Saldırganlar kalıcılık, ayrıcalık yükseltme, sürücü yükleme ve yanal hareket için hizmet oluşturur veya
değiştirir. Meşru yazılımlar da tamamen aynı mekanizmayı kullanır; bu nedenle yollar, hesaplar, imzalama,
oluşturma bağlamı ve çalışma zamanı kanıtları önemlidir.

<span class="evidence-strength">En iyi sonuç:</span> ayrıştırılan denetim kümesi, hizmeti/sürücüyü
gösterildiği şekilde yapılandırmıştır. Yapılandırma tek başına hizmetin başarıyla başladığını kanıtlamaz.

## Konum { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

`SYSTEM` kovanını işlem günlükleriyle birlikte toplayın. System 7045 ve Security 4697 gibi olay kimlikleri
kurulumu kaydedebilir; hizmet denetimi ve işlem olayları gerçek başlatmayı/yürütmeyi destekleyebilir.

## Analist için uyarılar { #analyst-cautions }

- `ImagePath` ortam değişkenleri, tırnak işaretleri, bağımsız değişkenler, aygıt yolları veya boşluk içeren
  tırnaksız bir yol içerebilir. Normalleştirmeden önce olduğu gibi koruyun.
- `ServiceDll`, `svchost.exe` tarafından barındırılan hizmetler için kritiktir; `ImagePath` tek başına yalnızca barındırıcıyı adlandırıyor olabilir.
- `StartType=automatic`, hizmetin başladığını veya etkin kaldığını kanıtlamaz.
- `ObjectName` yapılandırılmış hesaptır; hizmeti yükleyen kullanıcı olmak zorunda değildir.
- Sürücüler ve kullanıcı modu hizmetleri farklı `ServiceType` bayrakları ve yol kuralları kullanır.
- Hata eylemleri, bir hatanın ardından hizmet hesabı olarak bir komut yürütebilir; bu, gözden kaçması kolay
  bir kalıcılık yüzeyidir.
- Diğer denetim kümeleri önceki yapılandırmayı koruyor olabilir; `IsCurrentControlSet` kullanın.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## Çıktı alanları { #output-fields }

CSV çıktısında `DependOnService`, `DependOnGroup` ve `RequiredPrivileges` kayıt defterindeki sırayı koruyan JSON dizileridir. Boş dizi (`[]`), değerin mevcut ve listenin boş olduğunu gösterir. Boş hücre, değerin bulunmadığını veya okunamadığını gösterir. Okuma hataları için `Problems` alanına bakın.

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath` | Sistem kimliği ve girdi kovanı. Toplamalar arasında koruyun. |
| `ControlSet`, `IsCurrentControlSet` | Kaynak denetim kümesi ve `Select\Current` değerinin onu adlandırıp adlandırmadığı. Etkin durumla ilgili iddialar için geçerli olanı tercih edin. |
| `ServiceName` | Kayıt defteri anahtarı/dahili hizmet adı. Olaylara ve Service Control Manager komutlarına güçlü bir pivot noktası. |
| `DisplayName`, `Description` | Kullanıcıya yönelik metin; kaynak başvuruları olabilir veya saldırgan tarafından kontrol ediliyor olabilir. |
| `ImagePath` | Yapılandırıldığı hâliyle yürütülebilir dosya/sürücü komutu. Tırnaklamayı, bağımsız değişkenleri, ortam değişkenlerini ve aygıt yollarını ayrıştırın. |
| `ServiceDLL` | Barındırıcı bir hizmetin yüklediği DLL. Paylaşılan işlem hizmetlerinde çoğunlukla asıl yüktür (payload). |
| `StartType`, `StartValue` | Okunabilir/ham başlatma modu: boot (0), system (1), automatic (2), manual (3), disabled (4). |
| `DelayedAutostart` | Otomatik hizmet, ilk önyükleme aşamasından sonra başlar; yine de kalıcılık sağlayabilir. |
| `ServiceType`, `TypeValue` | Çözümlenmiş/ham bayraklar: çekirdek sürücüsü, dosya sistemi sürücüsü, kendi/paylaşılan işlem, kullanıcı hizmeti vb. |
| `ErrorControl` | Önyükleme/başlatma hatası işleme düzeyi. Öncelikle operasyonel bağlam sağlar. |
| `ObjectName` | Yapılandırılmış hizmet hesabı veya sürücü nesnesi adı. Yükleyicinin kimliği değildir. |
| `Group` | Yükleme sırası grubu; özellikle boot/system sürücüleri için önemlidir. |
| `DependOnService`, `DependOnGroup` | Hizmetin başlatma bağımlılıkları. Adı belirtilen hizmetin önce başlatılması gerekir. Bir yükleme sırası grubu için Windows, grup üyelerini başlatmayı dener ve en az bir üyenin çalışıyor olmasını gerektirir. |
| `RequiredPrivileges` | Hizmet için yapılandırılmış ayrıcalıklar. Bu ayar yoksa Hizmet Denetim Yöneticisi işlem belirtecinin varsayılan ayrıcalıklarını kullanır. Paylaşılan işlem, hizmetlerinin gerektirdiği ayrıcalıkların birleşimini kullanır. Belirteçte `SeChangeNotifyPrivilege` zaten varsa listede yer almasa bile Windows bu ayrıcalığı korur. |
| `ServiceSidType` | `none`, `unrestricted` veya `restricted`: belirtecin hizmetin kendi SID'sini (`NT SERVICE\<name>`) içerip içermediği; ACL'ler bu SID'ye erişim verebilir. `restricted` ayrıca belirteci yazma kısıtlamalı yapar. |
| `UserServiceFlags` | Kullanıcı başına hizmet bayrakları, saklandığı gibi (sayı); kullanıcı başına hizmet şablonlarında ve örneklerinde bulunur. |
| `FailureResetPeriod` | Hata sayacının sıfırlanmasından önceki saniye; `4294967295` hiçbir zaman anlamına gelir. |
| `FailureCommand` | Bir `run_command` hata eylemi için yapılandırılmış komut. Yüksek değerli bir kalıcılık alanı. |
| `RebootMessage` | Bir `reboot` hata eyleminden önce kullanıcılara yayınlanan ileti; metin veya kaynak başvurusu. |
| `FailureActionsOnNonCrashFailures` | `True`: hata eylemleri yalnızca işlem durduğunu bildirmeden sona erdiğinde değil, hizmet sıfır olmayan bir çıkış koduyla durduğunda da çalışır. Boş: bu anahtarda saklanmıyor (kullanıcı başına hizmet örneği şablonunun ayarını alabilir). |
| `FailureActions` | Saklanan tüm eylemler sırayla (`restart`, `reboot`, `run_command`, `own_restart`, `none`) ve gecikmeleri. Sonuncusu sonraki her hatada uygulanır; Kurtarma sekmesi yalnızca ilk, ikinci ve sonraki hataları gösterir. |
| `FailureActionType`, `FailureActionDelayMs` | Filtreleme için düzleştirilmiş ilk eylem; tam listeyi de inceleyin. |
| `FailureActionsRaw` | Kod çözme başarısız olduğunda veya bildirilen eylemlerden sonra bayt bulunduğunda özgün baytlar. |
| `KeyLastWrittenUTC` | Hizmet anahtarındaki son değişiklik; kurulum/başlatma zamanı olmak zorunda değildir. |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri yeniden oynatma bağlamı. |
| `Problems` | Tür uyuşmazlığı, bozuk biçimli hata yapısı veya eksik/okunamayan alanlar. |

## İnceleme stratejisi { #investigation-strategy }

1. Standart dışı yolları, yazılabilir dizinleri, olağandışı hesapları, yeni anahtar zamanlarını ve hata komutlarını inceleyin.
2. Ortam değişkenlerini analistin makinesine göre değil, etkilenen ana bilgisayarın yapılandırmasına göre genişletin.
3. Toplandıysa gerçek yürütülebilir dosyanın veya DLL'in karmasını/imzasını kontrol edin.
4. Hizmet adını/yolunu 7045/4697 olayları, işlem oluşturma, Prefetch, `$MFT` ve USN ile ilişkilendirin.
5. Değiştirilmiş/silinmiş kalıcılık için geçerli ve geçerli olmayan denetim kümelerini karşılaştırın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Registry tree for device and driver services](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / service types and states](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: ChangeServiceConfig2 failure actions](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [Microsoft: SERVICE_REQUIRED_PRIVILEGES_INFO](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_required_privileges_infow#remarks)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
