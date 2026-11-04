# Sistem Bilgileri { #system-information }

## Nedir { #what-it-is }

Bir sistemin dört makine kovanı, bir analistin zaman çizelgesini okumadan önce ihtiyaç duyduğu
bilgileri verir:

- adları (NetBIOS, DNS ana bilgisayar adı, DNS etki alanı) ve dahil olduğu etki alanı veya çalışma
  grubu;
- Windows sürümü, derlemesi ve güncelleme revizyonu ile Windows'un ne zaman yüklendiği veya en son
  yükseltildiği;
- sistemin tuttuğu yerel saatleri anlamanızı sağlayan saat dilimi;
- en son ne zaman düzgün biçimde kapatıldığı;
- işlemci mimarisi ve kod sayfaları;
- en son oturum açan kullanıcı ve oturum açmış her kullanıcının profilleri;
- SID'leri: bilgisayarınki (yerel hesaplar bu SID artı bir RID'dir) ve etki alanınınki;
- yükseltildiği önceki Windows sürümleri.

`parse-registry system-info`, bunların tümünü içeren sistem başına bir kayıt yazar.

<span class="evidence-strength">En iyi sonuç:</span> kovanlar toplandığında (veya en son
yazıldığında) mevcut olan yapılandırma. Anahtar zamanları, bunun parçalarının en son ne zaman
değiştiğini söyler.

## Konum { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

Dört kovanı işlem günlükleriyle birlikte toplayın. Bunlardan herhangi biri tek başına, içerdiği
bilgileri verir.

## Analist için uyarılar { #analyst-cautions }

- **Kayıt defteri zamanları UTC'dir; saat dilimi geri kalan her şey içindir.** Anahtar zamanları,
  FILETIME'lar ve çoğu olay zamanı UTC'dir. Metin günlükleri, bazı uygulama günlükleri ve adlara veya
  dizelere yazılmış tarihler (`Source OS (Updated on …)` gibi) yerel olabilir. `Bias`, yerel standart
  saatten UTC'ye olan dakika farkıdır (`UTC = local + Bias`). `UtcOffset` ve `DaylightUtcOffset` iki
  ofseti verir; `TimeZoneDescription` ise yaz saatinin ne zaman başlayıp bittiğini söyler.
- **Saat dilimi değişebilir.** `TimeZoneLastWrittenUTC`, anahtarın son yazılma zamanıdır. Windows,
  `ActiveTimeBias` değerini (o anda geçerli olan ofset) güncellediğinde anahtarı kendiliğinden de
  yeniden yazar; bu nedenle yakın tarihli bir zaman, birinin saat dilimini değiştirdiğini kanıtlamaz.
  Olay günlükleriyle karşılaştırın: Security 4616 (sistem saati değişti) ve System günlüğündeki
  `Microsoft-Windows-Kernel-General` olayları.
- **`StandardStart` ve `DaylightStart`, `SYSTEMTIME` değil, çekirdeğin `TIME_FIELDS` yapısıdır.**
  Haftanın günü, üçüncü alanları değil (`SOFTWARE` içindeki dilimin `TZI` değerinde olduğu gibi), son
  alanlarıdır. ABD dilimlerinde her iki düzen de aynı okunur, çünkü haftanın günü Pazar'dır (0).
  Juntei bunları `TIME_FIELDS` olarak çözer: herkese açık bir Paskalya Adası sistemi (ilk Cumartesi
  saat 22:00) farkı gösterir. 0 değerindeki ay, yaz saati uygulaması olmadığı anlamına gelir.
- **Windows 11 "Windows 10" der.** `ProductName`, Windows 11'de `Windows 10 …` olarak kalır. 22000 ve
  sonraki derlemeler Windows 11'dir ve özet bunu belirtir. `CurrentVersion`, Windows 8.1, 10 ve 11'de
  `6.3` değerindedir; `Version`, varsa `CurrentMajorVersionNumber` (10) değerini kullanır.
- **`InstallDate` özellik yükseltmeleriyle değişir.** Yeni bir sürüme yerinde yükseltme (örneğin
  1809'dan 21H2'ye) onu sıfırlar. Her yükseltmenin yerini aldığı sürüm, kendi `InstallDate` değeriyle
  birlikte `PreviousVersions` (`Setup\Source OS`) içindedir. Bunların en eski `InstallDate` değeri,
  özgün kuruluma en yakın olanıdır. Bir `Source OS` anahtarının adındaki tarih, saat dilimini
  belirtmez.
- **`ShutdownTime` son düzgün kapatmadır.** Bir çökme veya güç kaybı onu güncellemez. Olay
  günlükleriyle (6006, 6008, 1074) ve kovanlara yapılan son yazmaların zamanlarıyla karşılaştırın.
- **`LastLoggedOnUser`, her oturum açma değil, oturum açma ekranının son kullanıcısıdır.** Uzak
  Masaüstü, `runas`, hizmetler ve zamanlanmış görevlerin tümü bunu ayarlamaz.
  `LogonUiLastWrittenUTC`, yaklaşık olarak en son ne zaman ayarlandığını gösterir.
- **Profiller:** `ProfileList`, sistemde profili olmuş her kullanıcıyı (ve hizmeti) listeler; silinmiş
  hesaplar da profilleri kaldırılana kadar buna dahildir. Windows 10 ve sonrası
  `LocalProfileLoadTime` ve `LocalProfileUnloadTime` değerlerini yazar: yaklaşık son oturum açma ve
  oturum kapatma.
- **SID'ler:** yerel hesaplar `MachineSid-<RID>`, etki alanı hesapları `DomainSid-<RID>` biçimindedir.
  `MachineSid`, `SAM` kovanından gelir (yoksa `SECURITY` kovanının `PolAcDmS` değerinden). İkisinin
  eşleşmesi gerekir; eşleşmediklerinde, farklı sistemlerin kovanlarında olacağı gibi, `Problems` bunu
  belirtir. Bir çalışma grubunda `DomainSid` boştur.
- **Bir dizin, bir sistem.** `-d` ile bir dizindeki kovanlar tek bir sisteme aittir. `RegBack`
  kopyaları ve aynı türden ikinci bir kovan (`SYSTEM.old`) kendi kayıtlarını oluşturur. `-f` ile
  verilen kovanlar tek bir sisteme aittir ve diğerleri onların yanında aranır.
- **Denetim kümeleri:** yalnızca Windows'un en son kullandığı denetim kümesi okunur.
  `parse-registry read` diğerlerini gösterir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

Özet, her sistem için şunları verir: Windows sürümü, ne zaman yüklendiği, sözcüklerle saat dilimi,
anahtar en son yazıldığında geçerli olan ofset, son kapatma, etki alanı veya çalışma grubu, en son
oturum açan kullanıcı, SID'i, kaç profili olduğu ve yükseltildiği sürümler. `auto-parse` da bulduğu
sistemleri listelerken her sistemin saat dilimini yazdırır.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | Ana bilgisayar ve okunan kovanlar (`SourcePath`, `SYSTEM` kovanıdır; yoksa okunan ilk kovan). |
| `CompName`, `Hostname`, `Domain` | NetBIOS adı (`SYSTEM`, yoksa `SECURITY` kovanının `PolAcDmN` değeri), DNS ana bilgisayar adı ve DNS etki alanı (`Tcpip\Parameters`). |
| `PrimaryDomain`, `DNS-Domain` | Bilgisayarın katıldığı etki alanı veya çalışma grubu (`PolPrDmN`) ve etki alanının DNS adı (`PolDnDDN`). |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: yazıldığı şekliyle ürün adı, `EditionID`, `DisplayVersion` (yoksa `ReleaseId`), ana ve alt sürüm (yoksa `CurrentVersion`), derleme, güncelleme derleme revizyonu, `CSDVersion` ve `Client`, `Server` veya `Server Core`. |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (saniye hassasiyetinde) ve `InstallTime` (100 ns hassasiyetinde, Windows 10 ve sonrası): kurulum veya son özellik yükseltmesi. |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | Yazıldığı şekliyle. |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (yoksa `StandardName`) ve ofsetleri ve yaz saati tarihleriyle birlikte sözcüklerle saat dilimi. |
| `UTC-Offset`, `DaylightUTC-Offset` | Yerel standart saatin UTC'den farkı ve yaz saatinin farkı (yaz saati uygulaması yoksa boş). |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`, `ActiveTimeBias` (anahtar yazıldığında geçerli olan), `StandardBias`, `DaylightBias`. |
| `StandardName`, `DaylightName` | Yazıldığı şekliyle; çoğu zaman bir kaynak başvurusu (`@tzres.dll,-112`). |
| `StandardStart`, `DaylightStart` | Standart saatin ve yaz saatinin ne zaman başladığı, sözcüklerle (`first Sunday of November, 02:00`). |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | Dinamik yaz saati uygulaması kapatılmış; donanım saati UTC tutuyor. |
| `TimeZoneLastWrittenUTC` | Saat dilimi anahtarının son yazılma zamanı. |
| `LastShutdownUTC` | `ShutdownTime`: son düzgün kapatma. |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | Okunan denetim kümesi, `PROCESSOR_ARCHITECTURE`, `ACP` ve `OEMCP`. |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | Oturum açma ekranının son kullanıcısı ve anahtarının son yazılma zamanı. |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | Bilgisayarın SID'i, etki alanının SID'i (`PolPrDmS`) ve bilgisayar hesabının etki alanındaki SID'i (`PolMachineAccountS`). |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | Bilgisayarın yaklaşık olarak en son ne zaman adlandırıldığı, bir etki alanına veya çalışma grubuna en son ne zaman katıldığı ve en son ne zaman güncellendiği (`CurrentVersion` anahtarı). |
| `PreviousVersions` | Her `Source OS` anahtarı: sürüm, kurulum tarihi ve anahtar adındaki tarih (JSON: her biri bir nesne). |
| `Profiles` | Her profil: SID, klasör, anahtar zamanı, son yükleme ve kaldırma (JSON: her biri bir nesne). |
| `TransactionLogStatus`, `LogsApplied` | Dört kovanın tümü için kayıt defteri kurtarma bağlamı. |
| `Problems` | Yanlış tür veya boyuttaki değerler (baytları korunur) ve bilgisayar SID'leri farklı olan bir `SAM` ve `SECURITY`. |

## İnceleme stratejisi { #investigation-strategy }

1. Önce saat dilimini okuyun: `UtcOffset` değerini, yaz saati tarihlerini ve saat diliminin yakın
   zamanda değiştirilip değiştirilmediğini not edin. Yerel saatli günlükleri bununla dönüştürün.
2. Sürümü, derlemeyi ve kurulum tarihini not edin. Yükseltmeler için `PreviousVersions` değerini
   denetleyin ve kurulum tarihini en eski dosya sistemi ve olay günlüğü zamanlarıyla karşılaştırın.
3. `LastShutdownUTC` değerini not edin: ondan sonra yeni bir kapatma zamanı olmadan gerçekleşen
   etkinlik bir çökmeye, güç kaybına veya canlı toplamaya işaret eder.
4. Başka yerlerdeki SID'lerde (BAM, `ProfileList`, olay günlükleri) yerel hesapları etki alanı
   hesaplarından ayırmak için `MachineSid` ve `DomainSid` değerlerini, son oturum açmalar için ise
   profillerin yükleme ve kaldırma zamanlarını kullanın.
5. Yeniden adlandırmalar ve etki alanına katılmalar için bilgisayar ve etki alanı adlarını olay
   günlükleriyle ve `CompNameLastWrittenUTC` ve `PrimaryDomainLastWrittenUTC` ile ilişkilendirin.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: Güvenlik tanımlayıcıları](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: Saat dilimleri](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: Windows ürün bilgileri](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
