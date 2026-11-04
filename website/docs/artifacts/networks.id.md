# Jaringan { #networks }

## Apa itu { #what-they-are }

Windows mengingat jaringan-jaringan yang pernah dimasuki sistem dan bagaimana jaringannya dikonfigurasi.

- **`SOFTWARE`:**
  - **Profil jaringan** (`NetworkList\Profiles`): satu untuk setiap jaringan yang pernah terhubung dengan sistem,
    berisi namanya (SSID untuk jaringan Wi-Fi), kategori (publik, privat, domain), jenis koneksi
    (kabel, nirkabel, VPN, mobile broadband), serta kapan sistem pertama dan terakhir kali terhubung ke jaringan itu.
  - **Signature** (`NetworkList\Signatures`): apa yang mengidentifikasi setiap jaringan, seperti
    alamat MAC gateway dan DNS suffix.
  - **Kartu jaringan** (`NetworkCards`): adaptor-adaptornya.
- **`SYSTEM`** (per control set):
  - **Antarmuka TCP/IP:** alamat, server dan lease DHCP, server DNS, domain.
  - **Share** yang ditawarkan sistem (`LanmanServer\Shares`).
  - **Port proxy** (`PortProxy`): tunnel yang dibuat oleh `netsh interface portproxy`, yang
    digunakan penyerang untuk melakukan pivot dan menjangkau RDP melalui port lain.
  - **Persistent route.**
  - **Zona waktu**, yang mengubah waktu lokal pada profil menjadi UTC.

<span class="evidence-strength">Kesimpulan terbaik:</span> sistem terhubung ke jaringan dengan
nama dan gateway ini, pertama dan terakhir kali pada waktu-waktu ini. Sistem memiliki alamat ini dari server DHCP ini,
dan menawarkan share serta tunnel ini saat hive disimpan. Nama jaringan saja tidak
mengidentifikasi di mana jaringan itu berada: banyak jaringan bernama "Network" atau menggunakan SSID yang sama.

## Lokasi { #location }

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

Kumpulkan kedua hive beserta log transaksinya. Hive SYSTEM diperlukan untuk mengonversi waktu
profil ke UTC.

## Peringatan bagi analis { #analyst-cautions }

- **Waktu profil bersifat lokal.** `DateCreated` dan `DateLastConnected` ditulis dalam waktu
  lokal sistem.
  - `FirstConnectedLocal` dan `LastConnectedLocal` mempertahankannya apa adanya.
  - `FirstConnectedUTC` dan `LastConnectedUTC` mengonversinya dengan zona waktu dari hive
    SYSTEM milik sistem (`TimeZone`) dan aturan daylight saving-nya.
  - Waktu dari sebelum zona waktu diubah, atau dari tahun dengan aturan daylight saving
    yang berbeda (tanggal AS sebelum 2007), dapat meleset sebesar selisihnya.
  - Tanpa hive SYSTEM yang dapat dibaca dan current control set yang valid, hanya waktu lokal yang diberikan. Celah DST dan jam berulang yang ambigu tidak memiliki satu timestamp UTC yang terbukti: UTC dibiarkan kosong dan `Problems` menjelaskan keterbatasannya. Offset atau tanggal yang tidak valid juga tetap tidak dikonversi.
- **Hanya pertama dan terakhir.** Setiap profil memiliki satu koneksi pertama dan satu koneksi terakhir, bukan setiap
  koneksi. Waktu penulisan terakhir key profil biasanya sesuai dengan koneksi terakhir.
- **Nama berulang.** Windows menamai jaringan kabel yang tidak dikelola "Network", "Network 2", dan seterusnya.
  SSID Wi-Fi tidak unik. Gunakan `GatewayMac` dan `DnsSuffix` untuk membedakan jaringan, dan
  korelasikan dengan log DHCP dan inventaris aset. MAC gateway belum tentu merupakan BSSID access point Wi-Fi dan dengan sendirinya tidak menetapkan lokasi fisik.
- **Share, proxy, dan route** merupakan konfigurasi: sebuah share atau port proxy dapat dibuat dan
  tidak pernah digunakan. Setiap nilai tidak memiliki waktunya sendiri; waktu penulisan terakhir key adalah saat salah satunya
  berubah.
- **Antarmuka:** key antarmuka di bawah GUID tanpa alamat merupakan adaptor yang belum
  dikonfigurasi. Dengan DHCP, alamatnya adalah alamat dari lease terakhir. Alamat statis yang tersisa dari sebelum DHCP
  tidak ditampilkan sebagai sedang digunakan.
- **Control set:** secara default, control set yang terakhir digunakan Windows yang dibaca. `--all-control-sets`
  membaca semuanya, yang dapat menampilkan share atau proxy yang telah dihapus dari control set saat ini.

## Mengurai dengan Juntei { #parse-with-juntei }

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

Ringkasan mencantumkan jaringan yang terakhir terhubung (terbaru lebih dulu), serta setiap share, port proxy, dan
persistent route.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath` | Host, dan hive. |
| `ControlSet`, `IsCurrentControlSet` | Untuk record SYSTEM: control set, dan apakah control set itu yang terakhir digunakan Windows. |
| `Source` | `network_profile`, `network_signature` (signature yang tidak dimiliki profil mana pun), `network_card`, `interface`, `share`, `port_proxy`, `route`. |
| `ProfileName` | Nama profil (SSID untuk jaringan Wi-Fi); deskripsi signature. |
| `Category`, `ConnectionType` | `public`, `private`, `domain`; `wired`, `wireless`, `vpn`, `mobile_broadband` (selain itu, angkanya). |
| `FirstConnectedUTC`, `LastConnectedUTC` | `DateCreated` dan `DateLastConnected` yang dikonversi ke UTC dengan `TimeZone`. |
| `FirstConnectedLocal`, `LastConnectedLocal` | Nilai yang sama, apa adanya (waktu lokal, hingga milidetik). |
| `TimeZone` | Zona waktu yang digunakan untuk konversi. |
| `GatewayMAC`, `DNS-Suffix` | Alamat MAC gateway dan DNS suffix dari signature profil (`|` di antara beberapa nilai). |
| `InterfaceGUID` | GUID antarmuka; `ServiceName` kartu jaringan. |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | Alamat yang digunakan antarmuka, server DHCP, dan lease-nya. |
| `ShareName`, `SharePath` | Sebuah share dan folder yang ditawarkannya. |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | Sebuah port proxy: alamat yang didengarkannya dan tujuan penerusannya. |
| `Route` | Persistent route sebagaimana tersimpan: `destination,netmask,gateway,metric`. |
| `Details` | Nilai lain, sebagai `Name=Value; …` (CSV) atau objek (JSON): deskripsi profil, `Managed`, nama pertama, dan key signature; adaptor antarmuka (dari `NetworkCards`), mask, gateway, server DNS, dan domain; tipe share, keterangan, dan field lainnya; jenis proxy; bagian-bagian route. |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | Key, nilai untuk share, proxy, atau route, dan waktu penulisan terakhir key. |
| `DataRaw` | Byte yang tidak dapat didekode: waktu yang bukan `SYSTEMTIME` valid, share yang bukan list string. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Nilai dengan tipe atau ukuran yang salah, tanggal yang mustahil, dan nama yang tidak sesuai dengan bentuknya. |

## Strategi investigasi { #investigation-strategy }

1. Urutkan profil berdasarkan `LastConnectedUTC` dan periksa jaringan di sekitar waktu insiden: jaringan
   Wi-Fi yang tidak dikenal, mobile hotspot, VPN.
2. Gunakan `GatewayMac` dan `DnsSuffix` untuk membedakan jaringan dengan nama yang sama, dan untuk mencocokkan
   jaringan di berbagai host.
3. Periksa setiap port proxy dan share. Port proxy ke port 3389 atau 445 milik host internal, atau share
   dari folder staging (`C:\ProgramData`, `C:\Users\Public`), merupakan petunjuk. Korelasikan dengan log
   peristiwa (pembuatan proses `netsh`, peristiwa share 5142/5144) dan waktu key.
4. Gunakan alamat dan waktu lease antarmuka untuk menempatkan host di jaringan, dan cocokkan
   dengan log firewall, DHCP, dan proxy.
5. Gunakan `--all-control-sets` dan `parse-registry read --recover-deleted` untuk share,
   proxy, dan profil yang telah dihapus.

## Detail lebih lanjut { #further-details }

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
