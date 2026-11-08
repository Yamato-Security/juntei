# Pengumpulan bukti { #collecting-evidence }

Juntei mengurai berkas yang sudah dikumpulkan dari Windows. Gunakan alat akuisisi tepercaya dan pertahankan metadata asli jika memungkinkan. Image disk VHDX dapat diurai tanpa mengekstraknya: lihat [Memasang Image Disk](../commands/mount.md).

## Lokasi umum { #common-locations }

| Artefak | Lokasi umum |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | Berkas metadata volume NTFS `C:\$MFT` |
| `$LogFile` | Berkas metadata volume NTFS `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| Jurnal USN | Stream metadata NTFS `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| Pencatatan Akses Pengguna (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| Browser web | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\`, dan folder profil browser lainnya |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` dan lokasi pengguna/sistem lainnya |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` dan `CustomDestinations` |
| ShimCache | Hive `SYSTEM`: `...\Session Manager\AppCompatCache\AppCompatCache` |
| Karantina Defender | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| Registry mesin | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| Registry pengguna | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## Berkas yang harus dikumpulkan bersama { #files-that-must-travel-together }

- Kumpulkan hive registry beserta berkas `.LOG`, `.LOG1`, dan `.LOG2` yang berdampingan. Juntei dapat memutar ulang log
  untuk perintah registry dan ShimCache ketika hive dalam kondisi dirty.
- Kumpulkan `SRUDB.dat` milik SRUM secara konsisten. Saat ini Juntei melaporkan database ESE yang dirty tetapi
  tidak memutar ulang `SRU*.log`; record yang hanya ada di log tersebut tidak tersedia.
- Kumpulkan `$MFT` bersama `$Extend/$J` jika memungkinkan. Record USN menyimpan referensi berkas induk,
  bukan path lengkap; MFT meningkatkan resolusi path.
- Kumpulkan `$LogFile` bersama `$MFT` dari volume yang sama dengan waktu yang sedekat mungkin. `$LogFile` berputar
  dengan cepat; pengumpulan yang lebih lambat dapat kehilangan record redo/undo sebelumnya sepenuhnya.
- Kumpulkan SRUM bersama hive `SOFTWARE` yang bersesuaian untuk me-resolve nama akun dan nama profil Wi-Fi.
- Kumpulkan seluruh direktori `Sum` milik Pencatatan Akses Pengguna: `SystemIdentity.mdb` menamai role-role dan
  database `{GUID}.mdb` tahunan. Juntei juga tidak memutar ulang log transaksi `Svc*.log` miliknya.
- Kumpulkan database browser beserta berkas `-wal` di sampingnya (`places.sqlite-wal`,
  `cookies.sqlite-wal`): perubahan terbaru, serta versi halaman lama yang menjadi sumber pemulihan entri
  terhapus, hanya ada di sana sampai browser melakukan checkpoint.
- Kumpulkan seluruh pohon `Quarantine` Defender agar Entries, ResourceData, dan Resources dapat dicocokkan.

## Peringatan untuk sistem live { #live-system-cautions }

Berkas yang sedang terbuka dapat berubah selama penyalinan. Hive registry dan database ESE dapat berada dalam kondisi dirty secara internal.
ShimCache mungkin tidak menuliskan entri terbarunya yang ada di memori hingga shutdown. Alat pengumpulan forensik
yang menggunakan snapshot volume atau akses raw lebih disarankan daripada penyalinan berkas biasa.

## Keamanan { #safety }

Bukti dapat berisi malware, sampel exploit, berkas LNK berbahaya, dan teks formula spreadsheet.
Lakukan analisis di sistem yang terisolasi, jangan mengeksekusi stream yang dipulihkan, dan gunakan `--extract` hanya jika Anda
memang bermaksud menangani konten berbahaya. Simpan sumber yang belum tersentuh secara terpisah.
