# Kullanıcı Erişim Günlüğü (UAL) { #user-access-logging-ual }

## UAL nedir { #what-ual-is }

Kullanıcı Erişim Günlüğü (User Access Logging, UAL), Windows Server 2012'den bu yana varsayılan olarak
yüklü ve etkin gelen bir Windows Server özelliğidir; istemcilerin sunucunun rollerine yaptığı istekleri
ESE veritabanlarında toplar. Her rol, istemci adresi ve hesap için istemcinin o yıl ilk ve son görüldüğü
zamanı, toplam istek sayısını ve yılın her günü için bir sayıyı saklar. Roller arasında File Server (SMB),
Active Directory Domain Services, Active Directory Certificate Services, DHCP Server, Print and Document
Services, Remote Access, Web Server (IIS) ve Windows Server Update Services bulunur.

UAL, yanal hareket incelemeleri için değerlidir: istemci satırları, bir sunucuyu hangi adreslerin ve
hesapların hangi günlerde kullandığını gösterir ve bu kayıtlar çoğu zaman olay günlüklerinin üzerine
yazılmasından çok sonra bile durur.

<span class="evidence-strength">En iyi sonuç:</span> kaydedilen adresteki bir istemci, kaydedilen hesap
olarak kimlik doğrulaması yapmış ve rolü sayı bulunan her gün kullanmıştır.

## Konum ve toplama { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` en son etkinliği içerir. `SystemIdentity.mdb` sunucuyu tanımlar, rolleri adlandırır ve her
yılın verilerini tutan `{GUID}.mdb` dosyasını listeler (`CHAINED_DATABASES`). UAL hizmeti veritabanlarını açık tutar:
`Sum` dizininin tamamını bir adli bilişim toplayıcısıyla veya bir birim anlık görüntüsünden toplayın.

!!! warning "Kirli veritabanları"
    Çalışan bir sunucudan toplanan `Current.mdb` dosyası çoğu zaman kirlidir. Juntei onu toplandığı haliyle
    okur ve bunu raporlar, ancak `Svc*.log` dosyalarını yeniden oynatmaz: yalnızca işlem günlüklerinde
    bulunan etkinlik eksik kalır.

## Analist için uyarılar { #analyst-cautions }

- UAL istekleri gün başına sayar; her isteği tek tek kaydetmez. Bir istemci satırındaki zamanlar
  yalnızca `InsertDate` ve `LastAccess` değerleridir.
- `DailyAccesses` içindeki günler UTC günleridir.
- `Address`, sunucunun gördüğü adrestir: sunucu ile gerçek istemci arasında bir NAT ağ geçidi, proxy veya
  VPN bulunabilir. `::1` ve `127.0.0.1` sunucunun kendisidir.
- `Sum` dizinindeki her veritabanını ayrıştırın: aynı istemcinin hem `Current.mdb` içinde hem de kendi
  yılına ait `{GUID}.mdb` içinde satırları olabilir.
- Satırların bulunmaması hiçbir şeyin bağlanmadığını kanıtlamaz: UAL kapatılabilir (`Disable-Ual` ile veya
  User Access Logging Service durdurularak) ve kirli bir veritabanında en yeni değişiklikler eksiktir.
- `SystemIdentity.mdb` olmadığında `RoleName`, Windows Server'ın rollerine verdiği adlardan gelir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Current.mdb, {GUID}.mdb and SystemIdentity.mdb
  -f, --file <FILE>      Parse one UAL database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl \
  | jq 'select(.Table == "CLIENTS") | {Address, AuthenticatedUserName, RoleName, LastAccess}'
```

## Çıktı tabloları { #output-tables }

CSV her tablo için bir dosya oluşturur; tablo boşsa dosya yalnızca başlık satırını içerir. JSON ve JSONL
satırları `SourceFile` (veritabanı) ve `Table` (tablonun veritabanındaki adı) alanlarını taşır.

| CSV dosyası | Tablo | İçerik |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | Her rolün istemcileri: adres, hesap, yılın ilk ve son erişimi, toplam ve günlük istek sayıları. |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | Her rolün ilk ve son kullanıldığı zaman. |
| `UAL-DNS.csv` | `DNS` | Adres ve ana bilgisayar adı çiftleri ile her birinin son görüldüğü zaman. |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Hyper-V sanal makineleri: tanımlayıcılar, seri numarası, oluşturulma ve son etkin olma zamanları. |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | Sunucunun kendisi: ad, etki alanı, Windows sürümü, ürün türü, donanım, saat dilimi ve son önyükleme; yeni bir satır bir yeniden adlandırmayı, bir etki alanına katılmayı veya etki alanı denetleyicisine yükseltmeyi gösterebilir. |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | Her yılın `{GUID}.mdb` dosyası. |
| `UAL-Roles.csv` | `ROLE_IDS` | Rol GUID'leri ve adları. |
| `UAL-<table>.csv` | Başka bir tablo | Juntei'nin tanımadığı bir tablonun her sütunu. |

## Alanlar { #fields }

| Alan | Yorumlama |
|---|---|
| `SourceFile` | Satırın okunduğu veritabanı. |
| `RoleGUID`, `RoleName` | Rol ve veritabanının yanındaki `SystemIdentity.mdb` dosyasından alınan adı (bu dosya yoksa yerleşik adlar). |
| `TenantID` | Rol bildiriyorsa, istemcinin kiracısının GUID'i. |
| `TotalAccesses` | İstemcinin veritabanının yılı içindeki istekleri; `DailyAccesses` değerlerinin toplamı. |
| `InsertDate` | Satırın oluşturulduğu zaman: istemcinin bu veritabanındaki yılın ilk isteği (UTC). |
| `LastAccess` | İstemcinin en son isteği (UTC). |
| `Address` | İstemcinin IPv4 veya IPv6 adresi. |
| `AuthenticatedUserName` | İstemcinin kimlik doğrulaması yaptığı hesap (`DOMAIN\user`; bilgisayar hesapları `$` ile biter). |
| `ClientName` | Rol bildiriyorsa, istemcinin adı. |
| `DailyAccesses` | İstek olan günler ve sayıları; `Day1` … `Day366` sütunlarından alınır ve `LastAccess` yılına göre tarihlendirilir: CSV'de `2023-01-12: 5, 2023-01-13: 2`, JSON'da `Date` ve `Accesses` içeren nesneler. Yılı bilinmeyen bir günde `Date` yerine `DayOfYear` bulunur. |
| `FirstSeen`, `LastSeen` | Bir rolün ilk ve son kullanımı (`ROLE_ACCESS`), bir DNS çiftinin son görülmesi (`DNS`); UTC olarak. |
| `CreationTime`, `LastSeenActive` | Bir sanal makinenin veya bir `SYSTEM_IDENTITY` satırının oluşturulduğu zaman ve bir sanal makinenin son etkin olduğu zaman; UTC olarak. |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server`. |
| `OS-CurrentTimeZone` | Sunucunun UTC'ye göre dakika cinsinden farkı (`-420`, UTC-07:00 demektir). |
| `OS-LastBootUpTime` | Sunucunun UTC olarak son önyüklemesi (okunamadığında WMI metni saklandığı haliyle). |
| Diğer sütunlar | Tablonun diğer tüm sütunları, Juntei'nin PascalCase alan etiketleriyle; örneğin `SystemDNS-HostName`, `SystemDomainName`, `OS-BuildNumber` ve `SystemSerialNumber`. |

## Otomatik ayrıştırma { #automatic-parsing }

`auto-parse`, tabloları `LateralMovement` içine `UAL-Clients.csv`, `UAL-RoleAccess.csv` vb. olarak
(`-t json` veya `-t jsonl` ile tek bir `UAL.json` ya da `UAL.jsonl` dosyası olarak) ve
`LateralMovement/UAL-logs.txt` ile birlikte yazar. Birleştirilmiş `-j` zaman çizelgesi her satırı
`Comp` ve `Artifact` tanımlayıcısı `ual` ile birlikte içerir. `-Z`; `InsertDate`, `LastAccess`,
`FirstSeen`, `LastSeen`, `CreationTime`, `LastSeenActive` ve `OS-LastBootUpTime` değerlerini dönüştürür.
