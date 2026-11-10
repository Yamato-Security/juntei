# Glosarium DFIR { #dfir-glossary }

| Istilah | Arti dalam bahasa sederhana |
|---|---|
| Allocated | Record/objek sedang digunakan menurut artefaknya. |
| Alternate Data Stream (ADS) | Data NTFS bernama yang melekat pada sebuah berkas selain konten normalnya yang tanpa nama, seperti `Zone.Identifier`. |
| Artefak | Data persisten yang dihasilkan oleh sistem operasi/aplikasi yang dapat menjawab pertanyaan forensik. |
| Carving | Menemukan record berdasarkan struktur/signature tanpa bergantung pada wadah/indeks normalnya. |
| Control set | Sebuah versi konfigurasi SYSTEM (`ControlSet001`, dll.); `Select\Current` menyebutkan yang aktif. |
| Koroborasi | Bukti independen yang mendukung kesimpulan yang sama. |
| Database/hive dirty | Berkas primer belum sepenuhnya tersinkronisasi saat dikumpulkan; log transaksi dapat menyimpan kondisi yang diperlukan. |
| Droid | Pengenal distributed-link-tracking menyerupai GUID yang digunakan untuk melacak berkas/volume saat dipindahkan. |
| ESE / JET Blue | Format database Extensible Storage Engine milik Microsoft, digunakan oleh SRUM dan data Windows lainnya. |
| FILETIME | Jumlah interval 100 nanodetik sejak 1601-01-01 UTC; sebuah angka bermakna sebagai waktu hanya jika skemanya menyatakan demikian. |
| Referensi berkas | Entry number NTFS ditambah sequence number yang mengidentifikasi satu generasi record MFT. |
| Hive | Berkas biner yang berisi pohon Windows Registry. |
| Record INDX | Blok (biasanya 4 KiB) dari indeks direktori NTFS (`$I30`) yang mencantumkan berkas-berkas di direktori; slack-nya dapat menyimpan kemungkinan bukti nama berkas yang dihapus atau di-rename. |
| LNK | Windows Shell Link/pintasan yang berisi target dan metadata yang ekstensif. |
| MRU | Pengurutan Most Recently Used; biasanya berupa urutan relatif, bukan waktu untuk setiap item. |
| MFT | NTFS Master File Table yang berisi record dan atribut berkas/direktori. |
| Hash Prefetch | Pengenal Windows yang digunakan dalam penamaan `.pf`; bukan hash kriptografis atas konten. |
| Waktu penulisan terakhir key registry | Waktu key berubah; Windows biasanya tidak menyatakan nilai mana yang berubah. |
| Data residen | Byte berkas/stream yang disimpan langsung di atribut MFT, bukan di data run eksternal. |
| RID | Relative Identifier, komponen akun/grup terakhir di bawah otoritas/domain SID Windows. |
| SDDL | Security Descriptor Definition Language: deskriptor keamanan yang ditulis sebagai teks, seperti `O:S-1-5-18D:(A;;0x001f01ff;;;S-1-5-32-544)`. |
| Deskriptor keamanan | Pemilik, grup primer, DACL (siapa boleh melakukan apa), dan SACL (audit dan label integritas) sebuah berkas. NTFS menyimpan setiap deskriptor yang berbeda satu kali di `$Secure:$SDS`, di bawah sebuah security ID. |
| SID | Security Identifier Windows untuk akun, layanan, grup, atau security principal. |
| Slack | Byte yang tidak digunakan di dalam/setelah struktur yang teralokasi yang dapat menyimpan data lama. |
| SRUM | Database System Resource Usage Monitor yang berisi penggunaan sumber daya aplikasi/pengguna. |
| Timestomping | Modifikasi timestamp yang disengaja untuk menyesatkan investigasi; heuristik bukanlah bukti. |
| Log transaksi | Perubahan tertunda/yang telah di-commit yang digunakan untuk memulihkan registry atau database ke kondisi yang konsisten. |
| USN | Update Sequence Number yang digunakan oleh record change journal NTFS dan metadata terkait. |
| UTC | Coordinated Universal Time; basis waktu bersama yang disarankan untuk timeline lintas sistem. |
| Volume serial number | Angka yang diberikan NTFS kepada sebuah volume saat diformat (`$Boot`); 32 bit rendahnya adalah yang dicatat oleh berkas LNK, Jump List, dan Prefetch mengenai volume sebuah berkas. |
