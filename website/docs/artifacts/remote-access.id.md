# Akses jarak jauh (sisi pengguna) { #remote-access-user-side }

## Apa itu jejak akses jarak jauh { #what-it-is }

`NTUSER.DAT` milik pengguna menyimpan jejak koneksi jarak jauh yang dibuat pengguna dari komputer tersebut.
Ini adalah sisi sumber dari pergerakan lateral:

- **Remote Desktop (`mstsc`):**
  - komputer yang diketik ke dalam klien, yang terbaru terlebih dahulu;
  - satu key per komputer tempat pengguna melakukan sign-in, dengan nama akun yang digunakan dan thumbprint
    sertifikat server yang diterima pengguna;
  - komputer yang diizinkan pengguna untuk menggunakan perangkat lokal.
- **PuTTY:**
  - sesi yang disimpan: host, port, akun, protokol, berkas key, proxy, dan port forwarding;
  - sesi yang terakhir dimulai;
  - setiap key server SSH yang diterima pengguna.
- **WinSCP:** sesi yang disimpan dan key server SSH yang diterima, dalam bentuk yang sama dengan PuTTY.
- **Drive yang dipetakan:** setiap drive yang dipetakan untuk dipulihkan saat sign-in, dan path yang diketik ke dalam
  *Map Network Drive*.
- **Share:** share yang di-mount oleh Explorer (`MountPoints2`).

Jejak ini menunjukkan ke mana sebuah akun pergi selanjutnya dan dengan kredensial apa, bahkan setelah log milik komputer
jarak jauh itu sendiri sudah tidak ada.

<span class="evidence-strength">Kesimpulan terbaik:</span> profil pengguna ini terhubung ke, atau menyimpan
koneksi ke, target ini. Sesi yang disimpan atau nama yang diketik saja tidak membuktikan bahwa
koneksi berhasil. Host key yang diterima atau key `Servers` dengan `UsernameHint` lebih kuat:
klien berhasil menjangkau server.

## Lokasi { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Terminal Server Client\Default            MRU0 … MRU9
Software\Microsoft\Terminal Server Client\Servers\<host>     UsernameHint, CertHash
Software\Microsoft\Terminal Server Client\LocalDevices       <host> = devices
Software\SimonTatham\PuTTY\Sessions\<session>
Software\SimonTatham\PuTTY\Jumplist                          Recent sessions
Software\SimonTatham\PuTTY\SshHostKeys                       <type>@<port>:<host> = key
Software\Martin Prikryl\WinSCP 2\Sessions\<session>
Software\Martin Prikryl\WinSCP 2\SshHostKeys
Network\<letter>                                             RemotePath, UserName, ProviderName
Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU
Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##server#share
```

Kumpulkan `NTUSER.DAT` beserta log transaksinya untuk setiap profil. Kumpulkan juga `SOFTWARE` milik sistem:
hive tersebut memberikan SID setiap pengguna.

## Peringatan bagi analis { #analyst-cautions }

- **Waktu key adalah waktu daftar.** `Default`, `SshHostKeys`, dan `Map Network Drive MRU` menyimpan daftar,
  dan last write key-nya adalah saat daftar tersebut terakhir berubah. Waktu itu milik item pertama
  (`MruPosition` 0). Untuk item lainnya, waktu itu hanyalah batas atas. Key `Servers\<host>`, sesi,
  `Network\<letter>`, atau share memiliki waktunya sendiri.
- **Disimpan bukan berarti digunakan.** Sesi PuTTY atau WinSCP dapat disimpan tanpa pernah terhubung. PuTTY
  menulis ulang `Recent sessions` ketika sesi yang disimpan dimulai, dan menambahkan host key ketika sebuah server
  pertama kali dijangkau.
- **Kata sandi:**
  - WinSCP dapat menyimpan kata sandi sesi, tunnel, dan proxy.
  - PuTTY hanya menyimpan kata sandi proxy, dalam teks biasa.
  - `PasswordStored` dan `StoredPasswords` di `Details` menunjukkan nilai mana yang diatur. Nilai-nilai tersebut tidak pernah
    didekode atau dituliskan.
- **WinSCP menghilangkan nilai default.** Sesi tanpa `FSProtocol` adalah SFTP, dan sesi tanpa
  `PortNumber` menggunakan port default protokol tersebut. Dalam hal itu, `Protocol` dan `Port` tetap kosong alih-alih
  ditebak.
- **Fingerprint host key** adalah `SHA256:…` milik OpenSSH, sebagaimana ditampilkan oleh `ssh-keygen -l`, `ssh`, dan PuTTY 0.75
  dan yang lebih baru. Juntei menghitungnya dari key yang disimpan PuTTY dan WinSCP. Bandingkan dengan
  key milik server itu sendiri, berkas `known_hosts`, dan log untuk mengidentifikasi server di balik sebuah nama atau alamat
  IP. Key SSH-1 (`rsa`) tidak memilikinya.
- **Drive yang dipetakan:** `RemoteUsername` hanya diatur ketika drive dipetakan dengan akun
  lain; Windows menulis `REG_DWORD` 0 ketika drive dipetakan dengan akun milik pengguna itu sendiri.
- **Share:** `MountPoints2` juga memiliki key untuk volume dan huruf drive. Hanya network share
  (`##server#share`) yang dilaporkan di sini; `\\server@SSL@443\DavWWWRoot` adalah share WebDAV.
- **Penghapusan:** membersihkan daftar, atau menggunakan klien portabel, tidak meninggalkan jejak di sini. Bandingkan dengan
  Jump Lists, Prefetch, berkas `.rdp`, cache bitmap RDP, dan log milik target.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry remote-access -d evidence/ -o remote.csv
juntei parse-registry remote-access -f NTUSER.DAT -t jsonl \
  | jq 'select(.Client == "rdp" or .HostKeyFingerprint != null)'
```

Ringkasan menghitung koneksi berdasarkan klien. Ringkasan mencantumkan koneksi yang key-nya terakhir ditulis,
yang terbaru terlebih dahulu, serta sesi yang menyimpan kata sandi.

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, nama folder profil, SID pengguna dari `ProfileList` milik sistem (dengan hive `SOFTWARE` milik sistem), dan hive. |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive`, atau `network_share`. |
| `Kind` | Asal record: `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point`. |
| `Target`, `Port` | Komputer (nama host atau alamat IP) dan port, atau share (`\\server\share`). Port hanya diberikan bila Windows atau klien menuliskannya. |
| `RemoteUsername` | Akun yang digunakan pada target: `UsernameHint`, `UserName` milik sesi (atau `user@` dari `HostName` milik PuTTY), `UserName` milik drive yang dipetakan. |
| `Protocol` | `Protocol` milik PuTTY (`ssh`, `telnet`, `raw`, …), `FSProtocol` milik WinSCP (`sftp`, `scp`, `ftp`, `webdav`, `s3`), provider milik drive yang dipetakan (`Microsoft Windows Network`). |
| `MRU-Position` | Posisi dalam daftarnya (0 adalah yang terbaru): `MRU<n>`, `MRUList`, `Recent sessions` milik PuTTY. |
| `SessionName` | Nama sesi yang disimpan, dengan `%20` dan escape lainnya telah didekode. |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | Tipe host key SSH yang diterima (`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …), fingerprint OpenSSH-nya, dan angka-angkanya sebagaimana disimpan. |
| `DriveLetter` | Huruf drive yang dipetakan. |
| `PasswordStored` | Apakah sesi menyimpan kata sandi; tidak pernah didekode. |
| `Details` | Pengaturan lain, sebagai `Name=Value; …` (CSV) atau objek (JSON): `CertHash` sertifikat RDP, `LocalDevices`, berkas key sesi, proxy, tunnel SSH, port forwarding, perintah dan direktori jarak jauh, `StoredPasswords`, `Label` milik share, `ConnectionType` milik drive. |
| `KeyPath`, `ValueName` | Key, dan nilai untuk record yang berupa satu nilai. |
| `KeyLastWrittenUTC` | Last write dari key (lihat peringatan untuk daftar). |
| `DataRaw` | Byte nilai (hex), bila bukan string atau tidak dapat didekode. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Nilai dengan tipe yang salah (beserta byte-nya), host key yang tidak dapat dibaca, huruf `MRUList` yang tidak ada, dan nilai yang tidak dicantumkannya. |

## Strategi investigasi { #investigation-strategy }

1. Daftarkan target setiap pengguna berdasarkan klien, dan periksa terhadap sistem yang seharusnya
   dijangkau pengguna tersebut. Cari share administratif (`C$`, `ADMIN$`) dan alamat IP.
2. Catat akun yang digunakan (`RemoteUsername`): administrator domain dan akun lokal di mesin
   lain.
3. Cocokkan `HostKeyFingerprint` dengan key milik server, dan `CertHash` dengan sertifikat server
   RDP.
4. Korelasikan dengan peristiwa logon pada target (4624 tipe 10 dan 3, 1149), Jump Lists dan berkas LNK
   (`mstsc.exe`, `putty.exe`, berkas yang dibuka dari share), Prefetch, dan cache bitmap RDP.
5. Gunakan `parse-registry read --recover-deleted` untuk entri yang dihapus, dan salinan lama dari
   `NTUSER.DAT` profil tersebut (Volume Shadow Copies, cadangan) untuk daftar yang telah dibersihkan.

## Detail lebih lanjut { #further-details }

- [Microsoft: Menghapus entri dari kotak Computer di Remote Desktop Connection](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [FAQ PuTTY: Di mana PuTTY menyimpan datanya?](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: Penyimpanan konfigurasi](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
