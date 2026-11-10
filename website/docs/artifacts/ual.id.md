# Pencatatan Akses Pengguna (UAL) { #user-access-logging-ual }

## Apa itu UAL { #what-ual-is }

Pencatatan Akses Pengguna (User Access Logging, UAL) adalah fitur Windows Server, yang terinstal dan aktif secara default sejak
Windows Server 2012, yang mengagregasi permintaan yang dibuat klien ke role server dalam database
ESE. Untuk setiap role, alamat klien, dan akun, UAL menyimpan kapan klien pertama dan terakhir kali
terlihat dalam tahun tersebut, jumlah total permintaan, dan hitungan untuk setiap hari dalam tahun itu. Role yang dicatat antara lain
File Server (SMB), Active Directory Domain Services, Active Directory Certificate Services, DHCP
Server, Print and Document Services, Remote Access, Web Server (IIS), dan Windows Server Update
Services.

UAL sangat berharga untuk investigasi pergerakan lateral: baris kliennya menunjukkan alamat dan
akun mana yang menggunakan sebuah server, dan pada hari apa saja, sering kali lama setelah log peristiwa sudah tertimpa.

<span class="evidence-strength">Kesimpulan terbaik:</span> sebuah klien di alamat yang tercatat,
yang terautentikasi sebagai akun yang tercatat, menggunakan role tersebut pada setiap hari yang memiliki hitungan.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` menyimpan aktivitas terbaru. `SystemIdentity.mdb` mendeskripsikan server, menamai
role-role, dan mencantumkan berkas `{GUID}.mdb` yang menyimpan setiap tahun (`CHAINED_DATABASES`). Layanan UAL
selalu membuka database tersebut: kumpulkan seluruh direktori `Sum` dengan collector forensik atau dari
snapshot volume.

!!! warning "Database dirty"
    `Current.mdb` sering kali dirty ketika dikumpulkan dari server yang sedang berjalan. Juntei membacanya apa adanya saat dikumpulkan
    dan melaporkannya, tetapi tidak memutar ulang `Svc*.log`: aktivitas yang hanya ada di log transaksi
    tidak akan muncul.

## Peringatan bagi analis { #analyst-cautions }

- UAL menghitung permintaan per hari; UAL tidak mencatat setiap permintaan satu per satu. Satu-satunya waktu pada baris klien adalah
  `InsertDate` dan `LastAccess`.
- Hari-hari pada `DailyAccesses` adalah hari UTC.
- `Address` adalah alamat yang dilihat server: gateway NAT, proxy, atau VPN dapat berada di antara server dan
  klien yang sebenarnya. `::1` dan `127.0.0.1` adalah server itu sendiri.
- Urai setiap database di direktori `Sum`: klien yang sama dapat memiliki baris di `Current.mdb` dan di
  `{GUID}.mdb` untuk tahunnya.
- Tidak adanya baris tidak membuktikan bahwa tidak ada yang terhubung: UAL dapat dimatikan (`Disable-Ual`, atau dengan
  menghentikan User Access Logging Service), dan database yang dirty kehilangan perubahan terbarunya.
- Tanpa `SystemIdentity.mdb`, `RoleName` berasal dari nama yang diberikan Windows Server untuk role-rolenya.
- `LastAccess` bernilai nol atau tidak valid tidak menentukan tahun. Juntei kemudian memakai `InsertDate` yang valid, lalu metadata basis data tahunan; tanpa tahun yang valid, `DayOfYear` dipertahankan.
- Total ringkasan menjumlahkan semua baris sumber. `Current.mdb`, basis data tahunan, dan snapshot dapat tumpang tindih; ini bukan jumlah koneksi atau logon yang telah dideduplikasi.

## Mengurai dengan Juntei { #parse-with-juntei }

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

## Tabel keluaran { #output-tables }

CSV membuat satu berkas per tabel, yang hanya berisi header-nya jika tabel tersebut kosong. Baris JSON dan JSONL
membawa `SourceFile` (database-nya) dan `Table` (nama tabel di dalam database).

| Berkas CSV | Tabel | Isi |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | Klien setiap role: alamat, akun, akses pertama dan terakhir dalam tahun tersebut, jumlah permintaan total dan harian. |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | Kapan setiap role pertama dan terakhir kali digunakan. |
| `UAL-DNS.csv` | `DNS` | Pasangan alamat dan nama host, beserta kapan masing-masing terakhir terlihat. |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Mesin virtual Hyper-V: pengenal, nomor seri, waktu pembuatan, dan waktu terakhir aktif. |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | Server: nama, domain, versi Windows, tipe produk, perangkat keras, zona waktu, dan boot terakhir; baris baru dapat menunjukkan penggantian nama, bergabung ke domain, atau promosi menjadi domain controller. |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | Berkas `{GUID}.mdb` untuk setiap tahun. |
| `UAL-Roles.csv` | `ROLE_IDS` | GUID role beserta namanya. |
| `UAL-<table>.csv` | Tabel lain | Setiap kolom dari tabel yang tidak dikenal Juntei. |

## Field { #fields }

| Field | Interpretasi |
|---|---|
| `SourceFile` | Database tempat baris tersebut dibaca. |
| `RoleGUID`, `RoleName` | Role, dan namanya dari `SystemIdentity.mdb` yang berada di samping database (nama bawaan jika tidak ada). |
| `TenantID` | GUID tenant klien, jika role melaporkannya. |
| `TotalAccesses` | Permintaan klien dalam tahun database; jumlah dari `DailyAccesses`. |
| `InsertDate` | Kapan baris dibuat: permintaan pertama klien dalam tahun tersebut di database ini (UTC). |
| `LastAccess` | Permintaan terakhir klien (UTC). |
| `Address` | Alamat IPv4 atau IPv6 klien. |
| `AuthenticatedUserName` | Akun yang digunakan klien untuk autentikasi (`DOMAIN\user`; akun komputer diakhiri dengan `$`). |
| `ClientName` | Nama klien, jika role melaporkannya. |
| `DailyAccesses` | Hari-hari yang memiliki permintaan beserta jumlahnya, dari kolom `Day1` … `Day366`, diberi tanggal dalam tahun `LastAccess`: `2023-01-12: 5, 2023-01-13: 2` di CSV, objek dengan `Date` dan `Accesses` di JSON. Hari yang tahunnya tidak diketahui memiliki `DayOfYear` sebagai pengganti `Date`. |
| `FirstSeen`, `LastSeen` | Penggunaan pertama dan terakhir sebuah role (`ROLE_ACCESS`), terakhir kali sebuah pasangan DNS terlihat (`DNS`), dalam UTC. |
| `CreationTime`, `LastSeenActive` | Kapan sebuah mesin virtual atau baris `SYSTEM_IDENTITY` dibuat, dan kapan sebuah mesin virtual terakhir aktif, dalam UTC. |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server`. |
| `OS-CurrentTimeZone` | Selisih waktu server dari UTC dalam menit (`-420` berarti UTC-07:00). |
| `OS-LastBootUpTime` | Boot terakhir server dalam UTC (teks WMI sebagaimana tersimpan jika tidak dapat dibaca). |
| Kolom lain | Setiap kolom lain dari tabel dengan label field PascalCase milik Juntei, misalnya `SystemDNS-HostName`, `SystemDomainName`, `OS-BuildNumber`, dan `SystemSerialNumber`. |

## Penguraian otomatis { #automatic-parsing }

`auto-parse` menulis tabel-tabel ke `LateralMovement` sebagai `UAL-Clients.csv`, `UAL-RoleAccess.csv`, dan
seterusnya (satu berkas `UAL.json` atau `UAL.jsonl` dengan `-t json` atau `-t jsonl`), beserta
`LateralMovement/UAL-logs.txt`. Timeline gabungan `-j` menyertakan setiap baris, dengan `Comp` dan
pengenal `Artifact` `ual`. `-Z` mengonversi `InsertDate`, `LastAccess`, `FirstSeen`, `LastSeen`,
`CreationTime`, `LastSeenActive`, dan `OS-LastBootUpTime`.

## Detail lebih lanjut { #further-details }

- [Microsoft — User Access Logging](https://learn.microsoft.com/en-us/windows-server/administration/user-access-logging/get-started-with-user-access-logging)
- [Plaso — User Access Logging parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/user_access_logging.py)
- [Velociraptor — Windows.Forensics.UserAccessLogs](https://docs.velociraptor.app/artifact_references/pages/windows.forensics.useraccesslogs/)
- [CrowdStrike — How to Leverage User Access Logging for Forensic Investigations](https://www.crowdstrike.com/en-us/blog/user-access-logging-ual-overview/)
