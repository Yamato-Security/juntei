# Ağlar { #networks }

## Bunlar nedir { #what-they-are }

Windows, bir sistemin katıldığı ağları ve ağının nasıl yapılandırıldığını hatırlar.

- **`SOFTWARE`:**
  - **Ağ profilleri** (`NetworkList\Profiles`): sistemin bağlandığı her ağ için bir tane;
    adı (bir Wi-Fi ağının SSID'si), kategorisi (genel, özel, etki alanı), bağlantı türü
    (kablolu, kablosuz, VPN, mobil geniş bant) ve sistemin ona ilk ve son bağlandığı zamanlarla birlikte.
  - **İmzalar** (`NetworkList\Signatures`): her ağı tanımlayan bilgiler; örneğin ağ geçidinin MAC
    adresi ve DNS soneki.
  - **Ağ kartları** (`NetworkCards`): bağdaştırıcılar.
- **`SYSTEM`** (denetim kümesi başına):
  - **TCP/IP arabirimleri:** adresler, DHCP sunucusu ve kira süresi, DNS sunucuları, etki alanı.
  - Sistemin sunduğu **paylaşımlar** (`LanmanServer\Shares`).
  - **Bağlantı noktası proxy'leri** (`PortProxy`): `netsh interface portproxy` tarafından kurulan ve
    saldırganların pivot yapmak ve RDP'ye başka bir bağlantı noktası üzerinden ulaşmak için
    kullandığı tüneller.
  - **Kalıcı yollar.**
  - Profillerin yerel saatlerini UTC'ye dönüştüren **saat dilimi**.

<span class="evidence-strength">En iyi sonuç:</span> sistem bu ada ve ağ geçidine sahip bir ağa ilk
ve son kez bu zamanlarda bağlandı. Kovan kaydedildiğinde bu DHCP sunucusundan bu adresi almıştı ve bu
paylaşımları ve tünelleri sunuyordu. Bir ağın adı tek başına nerede olduğunu belirlemez: birçok ağ
"Network" olarak adlandırılır veya aynı SSID'yi paylaşır.

## Konum { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\NetworkList\Profiles\{GUID}
      ProfileName, Description, Category, NameType, Managed, DateCreated, DateLastConnected
  Microsoft\Windows NT\CurrentVersion\NetworkList\Signatures\Managed|Unmanaged\<signature>
      ProfileGuid, DefaultGatewayMac, DnsSuffix, FirstNetwork
  Microsoft\Windows NT\CurrentVersion\NetworkCards\<n>          ServiceName, Description
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Services\Tcpip\Parameters\Interfaces\{GUID}                    IP, DHCP, DNS, lease times
  Services\LanmanServer\Shares                                   one value per share
  Services\PortProxy\v4tov4|v4tov6|v6tov4|v6tov6\tcp             listen = connect
  Services\Tcpip\Parameters\PersistentRoutes                     destination,mask,gateway,metric
  Control\TimeZoneInformation
```

Her iki kovanı da işlem günlükleriyle birlikte toplayın. Profil zamanlarını UTC'ye dönüştürmek için
SYSTEM kovanı gereklidir.

## Analist için uyarılar { #analyst-cautions }

- **Profil zamanları yereldir.** `DateCreated` ve `DateLastConnected`, sistemin yerel saatiyle
  yazılır.
  - `FirstConnectedLocal` ve `LastConnectedLocal` bunları bulunduğu gibi korur.
  - `FirstConnectedUTC` ve `LastConnectedUTC` bunları sistemin SYSTEM kovanındaki saat dilimi
    (`TimeZone`) ve yaz saati kuralıyla dönüştürür.
  - Saat dilimi değiştirilmeden önceki bir zaman ya da farklı bir yaz saati kuralına sahip bir yıla
    ait bir zaman (2007 öncesi ABD tarihleri), aradaki fark kadar kayık olabilir.
  - Okunabilir bir SYSTEM kovanı ve geçerli bir güncel denetim kümesi olmadan yalnızca yerel zamanlar
    verilir. Yaz saati boşlukları ve belirsiz tekrarlanan saatlerin kanıtlanmış tek bir UTC zaman
    damgası yoktur: UTC boş bırakılır ve `Problems` bu sınırlamayı açıklar. Geçersiz ofsetler veya
    tarihler de dönüştürülmeden kalır.
- **Yalnızca ilk ve son.** Her profilin her bağlantısı değil, yalnızca bir ilk ve bir son bağlantısı
  vardır. Profil anahtarının son yazılma zamanı genellikle son bağlantıyla eşleşir.
- **Adlar tekrarlanır.** Windows, yönetilmeyen kablolu ağları "Network", "Network 2" vb. olarak
  adlandırır. Wi-Fi SSID'leri benzersiz değildir. Ağları birbirinden ayırmak için `GatewayMac` ve
  `DnsSuffix` kullanın ve bunları DHCP günlükleri ve varlık envanteriyle ilişkilendirin. Bir ağ geçidi
  MAC adresi mutlaka bir Wi-Fi erişim noktası BSSID'si değildir ve tek başına fiziksel konumu
  kanıtlamaz.
- **Paylaşımlar, proxy'ler ve yollar** yapılandırmadır: bir paylaşım veya bağlantı noktası proxy'si
  kurulup hiç kullanılmamış olabilir. Her değerin kendine ait bir zamanı yoktur; anahtarın son yazılma
  zamanı, bunlardan herhangi birinin değiştiği zamandır.
- **Arabirimler:** adresi olmayan bir GUID altındaki arabirim anahtarı, yapılandırılmamış bir
  bağdaştırıcıdır. DHCP ile adresler son kiraya aittir. DHCP öncesinden kalan statik bir adres
  kullanımda olarak gösterilmez.
- **Denetim kümeleri:** varsayılan olarak Windows'un en son kullandığı denetim kümesi okunur.
  `--all-control-sets` hepsini okur; bu, güncel kümeden kaldırılmış bir paylaşımı veya proxy'yi
  gösterebilir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and SYSTEM hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry networks -d evidence/ -o networks.csv
juntei parse-registry networks -d evidence/ -t jsonl \
  | jq 'select(.Source == "port_proxy" or .Source == "share")'
```

Özet, en son bağlanılan ağları (en yeniden eskiye) ve her paylaşımı, bağlantı noktası proxy'sini ve
kalıcı yolu listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath` | Ana bilgisayar ve kovan. |
| `ControlSet`, `IsCurrentControlSet` | SYSTEM kayıtları için: denetim kümesi ve Windows'un en son kullandığı küme olup olmadığı. |
| `Source` | `network_profile`, `network_signature` (hiçbir profile ait olmayan bir imza), `network_card`, `interface`, `share`, `port_proxy`, `route`. |
| `ProfileName` | Bir profilin adı (bir Wi-Fi ağının SSID'si); bir imzanın açıklaması. |
| `Category`, `ConnectionType` | `public`, `private`, `domain`; `wired`, `wireless`, `vpn`, `mobile_broadband` (aksi hâlde sayısal değer). |
| `FirstConnectedUTC`, `LastConnectedUTC` | `TimeZone` ile UTC'ye dönüştürülmüş `DateCreated` ve `DateLastConnected`. |
| `FirstConnectedLocal`, `LastConnectedLocal` | Aynı değerler, bulunduğu gibi (yerel saat, milisaniye hassasiyetinde). |
| `TimeZone` | Dönüştürme için kullanılan saat dilimi. |
| `GatewayMAC`, `DNS-Suffix` | Profilin imzalarındaki ağ geçidinin MAC adresi ve DNS soneki (birden çok değer arasında `|`). |
| `InterfaceGUID` | Bir arabirimin GUID'i; bir ağ kartının `ServiceName` değeri. |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | Bir arabirimin kullanımdaki adresleri, DHCP sunucusu ve kirası. |
| `ShareName`, `SharePath` | Bir paylaşım ve sunduğu klasör. |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | Bir bağlantı noktası proxy'si: neyi dinlediği ve nereye yönlendirdiği. |
| `Route` | Depolandığı şekliyle kalıcı bir yol: `destination,netmask,gateway,metric`. |
| `Details` | Diğer değerler; `Name=Value; …` (CSV) veya bir nesne (JSON) olarak: bir profilin açıklaması, `Managed`, ilk ad ve imza anahtarları; bir arabirimin bağdaştırıcısı (`NetworkCards` içinden), maskeleri, ağ geçitleri, DNS sunucuları ve etki alanı; bir paylaşımın türü, açıklaması ve diğer alanları; bir proxy'nin türü; bir yolun parçaları. |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | Anahtar, bir paylaşım, proxy veya yol için değer ve anahtarın son yazılma zamanı. |
| `DataRaw` | Kodu çözülemeyen baytlar: geçerli bir `SYSTEMTIME` olmayan bir zaman, dize listesi olmayan bir paylaşım. |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Yanlış tür veya boyuttaki değerler, imkânsız tarihler ve beklenen biçimde olmayan adlar. |

## İnceleme stratejisi { #investigation-strategy }

1. Profilleri `LastConnectedUTC` değerine göre sıralayın ve olay zamanı civarındaki ağları denetleyin:
   bilinmeyen Wi-Fi ağları, mobil erişim noktaları, VPN'ler.
2. Aynı ada sahip ağları birbirinden ayırmak ve bir ağı ana bilgisayarlar arasında eşleştirmek için
   `GatewayMac` ve `DnsSuffix` kullanın.
3. Her bağlantı noktası proxy'sini ve paylaşımı denetleyin. Dahili bir ana bilgisayarın 3389 veya 445
   numaralı bağlantı noktasına yönelen bir proxy ya da bir hazırlık klasörünün (`C:\ProgramData`,
   `C:\Users\Public`) paylaşımı bir ipucudur. Olay günlükleri (`netsh` işlem oluşturma, 5142/5144
   paylaşım olayları) ve anahtar zamanlarıyla ilişkilendirin.
4. Ana bilgisayarı ağ üzerinde konumlandırmak için arabirimlerin adreslerini ve kira zamanlarını
   kullanın ve bunları güvenlik duvarı, DHCP ve proxy günlükleriyle eşleştirin.
5. Kaldırılmış paylaşımlar, proxy'ler ve profiller için `--all-control-sets` ve
   `parse-registry read --recover-deleted` kullanın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
