# Validasi parser { #parser-validation }

Crate parser yang dikelola Juntei diuji terhadap bukti publik dan implementasi
independen. Perbandingan ini membantu mendeteksi regresi dan perbedaan interpretasi; perbandingan ini
tidak membuat parser mana pun bebas dari kesalahan. Simpan bukti sumber, tinjau field masalah atau
kerusakan pada setiap record, dan validasi temuan yang berdampak besar dengan artefak atau implementasi lain.

Pernyataan di bawah ini mempertahankan informasi validasi yang sebelumnya disimpan di README utama
Juntei. Ikuti repositori parser yang ditautkan untuk fixture uji dan detail implementasi terkini.

## Amcache { #amcache }

Juntei menggunakan [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser), sebuah
fork berlisensi Apache-2.0 dari
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) milik Security Ronin.
[Dokumentasi keluarannya](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
mendefinisikan setiap field executable, device, finding, dan summary.

## Master File Table { #master-file-table }

Crate mft dari [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) adalah fork yang dikelola dari crate
[mft](https://github.com/omerbenamram/mft) karya Omer Ben-Amram. Pada delapan berkas `$MFT`—sampel milik
crate tersebut, NIST hacking case, dan enam berkas dari DFIR Artifact Museum—1.048.733 baris dan kolom CSV
cocok dengan parser referensi secara berurutan. Perbandingan mencakup keluaran default, `--all-times`,
`--short-names`, dan `--cutoff`; penyajian timestamp merupakan perbedaan yang disengaja.

## Log transaksi NTFS ($LogFile) { #ntfs-transaction-log-logfile }

Crate logfile dari [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) dikelola dari
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog). Validasinya mencakup enam log lengkap dari DFIR
Artifact Museum, fixture publik versi 1.1 dan 2.0, metadata redo/undo yang didekode, dan penulisan
jurnal yang dibandingkan dengan data NTFS yang diurai secara independen. Lihat repositori parser untuk cakupan
terperinci dan perbedaan yang diketahui.

Juntei menambahkan pengujian CLI yang membandingkan setiap field CSV dengan JSON dan JSONL pada volume uji
go-ntfs publik (2.129 record), memverifikasi data redo/undo yang terpisah dan panjang payload mentah, menguji log yang rusak,
menolak penimpaan bukti, dan menjaga agar path MFT saat ini tidak mengikuti nomor record yang digunakan ulang.
Kasus regresi membedakan record MFT yang terpengaruh dari atribut terbuka yang menjadi jalur penulisannya,
memeriksa metadata entri direktori/atribut data yang datar, dan mencegah pemasangan MFT lintas volume.
Pengujian `auto-parse` multivolume mencakup berkas CSV terpisah dan satu berkas JSONL dengan provenans.

Perintah ini juga diperiksa pada enam log Windows yang dikumpulkan secara privat: seluruh 708.465 record CSV
yang diekspor cocok dengan JSONL field demi field, dan ekspor `auto-parse` per host mempertahankan jumlah record
yang sama. Bukti sumbernya tidak didistribusikan bersama repositori. Pemeriksaan ini memverifikasi konsistensi
keluaran, bukan bahwa setiap transaksi yang tercatat merepresentasikan tindakan pengguna yang telah selesai.

## Prefetch { #prefetch }

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) adalah fork yang dikelola dari
[frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) milik ForensicRS. Keluaran yang didekodenya
dibandingkan dengan [libscca](https://github.com/libyal/libscca), implementasi Prefetch independen
milik libyal.

## Jurnal USN { #usn-journal }

Crate usn dari [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) adalah fork yang
dikelola dari [Sootmark usn](https://github.com/Sootmark/usn). Pada enam jurnal nyata, path induk
Juntei yang memperhitungkan waktu cocok dengan parser independen di setiap tempat parser tersebut me-resolve path. Juntei
juga me-resolve 4.321 record yang tidak ter-resolve dalam perbandingan tersebut. Carver-nya tidak menghasilkan record
palsu saat diuji terhadap 880 MB data `$MFT`.

## Indeks direktori ($I30) { #directory-indexes-i30 }

Modul `i30` dari crate mft milik [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) membaca record INDX
beserta slack-nya. Pada dua stream `$I30` dari volume yang dibuat dengan ntfs-3g (`test_files/i30/` milik
Juntei) dan pada tiga sampel `$I30` publik yang ditulis Windows, entri yang digunakan dan entri di
slack cocok dengan entri dari [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) karya Maxim Suhanov, dengan
nama dan waktu yang sama. Ketika dfir_ntfs melaporkan referensi induk yang melampaui 32 bit (byte yang menimpa
induk), Juntei membiarkan induknya kosong.

## Sektor boot ($Boot) { #boot-sector-boot }

Crate boot dari [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) adalah buatan Yamato
Security sendiri. Pada lima berkas `$Boot` publik dari DFIR Artifact Museum (Windows 10, 11, dan
Server 2012 R2), setiap field cocok dengan parser referensi. Pada volume yang dibuat dengan `mkntfs` dengan sektor 512,
1.024, dan 4.096 byte serta cluster 4 KiB hingga 64 KiB, ukuran, cluster `$MFT` dan `$MFTMirr`,
serta serial number cocok dengan `fsstat` dari The Sleuth Kit. `$Boot` Windows 11 dari Museum
memiliki volume serial number yang dicatat oleh berkas LNK dari host yang sama, dan kelima berkas memiliki
kode boot yang sama.

## Deskriptor keamanan ($Secure:$SDS) { #security-descriptors-securesds }

Crate secure dari [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) adalah buatan Yamato
Security sendiri. Pada lima stream `$SDS` dari Museum (7.808 deskriptor, 60.781 ACE),
security ID, hash, offset, pemilik, grup, control flag, dan jumlah ACE cocok dengan parser
referensi, dan tipe, flag, mask, SID, serta object type setiap ACE cocok dengan `dissect.ntfs`. Setiap
hash yang tercatat adalah milik deskriptornya, setiap salinan mirror cocok dengan entrinya, dan setiap deskriptor memiliki
SDDL, termasuk kondisi dari 3.384 callback ACE-nya.

## SRUM { #srum }

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) didasarkan pada crate
[ese](https://github.com/Sootmark/ese) dan [srum](https://github.com/Sootmark/srum) milik Sootmark. Pada
11 database nyata, setiap kolom dalam 45.255 entri yang dikeluarkan cocok dengan dekode independen atas
nilai-nilai yang diekspos oleh `dissect.esedb`.

## Pencatatan Akses Pengguna { #user-access-logging }

`parse-ese ual` membaca database UAL dengan pembaca ESE yang sama. Pada 16 database nyata (308 baris,
145 di antaranya baris klien), setiap nilai cocok dengan dekode independen atas nilai-nilai yang diekspos oleh
`dissect.esedb`. Pada setiap baris klien, hari-hari yang memiliki hitungan berlangsung dari hari UTC `InsertDate`
hingga hari UTC `LastAccess`, dan jumlah hitungannya sama dengan `TotalAccesses`.

## Browser web { #web-browsers }

[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) adalah fork yang dikelola dari crate
browser, sqlite, dan plist milik Sootmark, yang dipilih setelah dibandingkan dengan pustaka Rust lain serta dengan
Plaso, Velociraptor (SQLiteHunter), dan Hindsight pada database yang sama. `parse-sqlite browser` dibandingkan
dengan pustaka `sqlite3` yang membaca salinan setiap database beserta berkas `-wal`-nya, pada 38 database
publik: berkas uji Plaso (Chrome 8 hingga 68, Firefox 2 hingga 118), profil yang dibuat oleh Chromium 150 dan
Firefox 140, serta database uji riwayat formulir milik ys-browser-parser. Seluruh 293 kunjungan, 21 unduhan,
692 cookie, dan 41 entri formulir cocok. Entri terhapus hanya dipulihkan di tempat entri memang telah dihapus:
dua halaman dan dua kunjungan yang dihapus dari riwayat Firefox 140, beserta waktu, URL, dan judulnya, serta
entri formulir terhapus pada database uji.

## Internet Explorer dan Edge lama (WebCache) { #internet-explorer-and-legacy-edge-webcache }

`parse-ese webcache` membaca `WebCacheV01.dat` dengan
[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser). Perintah ini dibandingkan dengan
pembaca ESE milik `dissect.database` pada dua database WebCache publik milik Plaso: satu dari Internet
Explorer 11, dan satu database dirty dari Windows 10 yang berisi riwayat, unduhan, dan cookie Edge lama.
Seluruh 4.156 baris cocok, nilai demi nilai: 44 kontainer, 182 entri riwayat, 6 unduhan, 170 cookie, dan
3.754 entri lainnya. Perbandingan tersebut mendekode record unduhan secara mandiri. Record yang ditandai
terhapus oleh database dikecualikan dari keduanya.

## Outlook { #outlook }

`parse-file outlook` membaca berkas PST, OST, `.msg`, dan `.oft` dengan
[ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser), yang pembaca PST dan OST-nya
merupakan fork dari [outlook-pst-rs](https://github.com/microsoft/outlook-pst-rs) milik Microsoft dan
pembaca `.msg`-nya ditulis berdasarkan spesifikasi terbuka Microsoft.

- **PST dan OST**: 66 berkas PST dan OST publik (Apache Tika, artemis, contoh Aspose.Email, DFIR
  Artifact Museum, Enron, contoh GroupDocs, java-libpst, data uji libyal, outlook-pst-rs, dan
  pst-extractor) dibandingkan dengan libpff, properti demi properti: setiap properti dari setiap
  store, folder, pesan, penerima, dan lampiran, serta data setiap lampiran. 324.175 fakta sama dan
  tidak ada yang berbeda tanpa penjelasan. Sisanya adalah apa yang hanya dibaca oleh ys-outlook-parser
  (15.846 fakta dari pesan, penerima, dan lampiran yang gagal dibaca libpff, serta dari sebuah store
  yang tidak dapat dibukanya; 8.223 dari item tersembunyi), sel penerima yang diberikan libpff sebagai
  kosong atau nol padahal barisnya menyatakan nilai itu tidak ada (49.077), entry id dan tipe folder
  yang dihitung seperti yang dilakukan MAPI (2.004), serta objek dan pesan tertanam yang diberikan
  libpff sebagai data mentahnya (480). Dua dari berkas tersebut sama sekali tidak dapat dibaca oleh
  kedua pembaca.
- **`.msg` dan `.oft`**: 214 berkas publik, dengan 2.404 pesan, penerima, dan lampiran serta 41.002
  properti, dibandingkan dengan extract-msg: 110.114 fakta sama, dan masing-masing dari 791 fakta yang
  berbeda dapat dijelaskan: 601 adalah kesalahan extract-msg terkait waktu, 35 berasal dari berkas ANSI
  yang didekodenya dengan code page yang salah, dan sisanya adalah apa yang dihilangkan atau
  ditambahkannya.
- **Input rusak**: lebih dari 260.000 berkas `.msg` hasil mutasi dan 160.000 PST hasil mutasi, serta
  berkas yang dibuat khusus, dibaca tanpa crash, hang, atau penggunaan memori yang tak terbatas.

## Pintasan LNK { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) adalah fork yang dikelola dari crate
[shell](https://github.com/Sootmark/shell) milik Sootmark. Parser ini dibandingkan pada 1.118 pintasan
publik. Dari 30 kolom CSV yang sama dengan parser referensi, 16 nilai individual berbeda:
13 path target yang melibatkan item rusak, terpotong, placeholder, MTP, atau Control Panel; dua
list extra data block yang melibatkan blok yang tidak dikenal atau terpotong; dan satu referensi MFT yang
dibaca parser referensi dari byte yang berbeda. Juntei melaporkan struktur yang rusak alih-alih menormalisasinya
secara diam-diam.

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) menggunakan karya compound file
dan Jump List milik Sootmark, dengan tautan tertanam yang didekode oleh `ys-lnk-parser`. Pada 77 Jump Lists
publik, seluruh 13.057 sel perbandingan bersama—termasuk timestamp hingga 100 ns—cocok dengan
parser referensi. Enam berkas yang ditolak parser referensi (custom list kosong, footer yang hilang, atau berkas
yang bukan Jump List) didekode atau dilaporkan secara eksplisit oleh Juntei.

## Keranjang Sampah (Recycle Bin) { #recycle-bin }

`parse-file recycle-bin` adalah parser milik Juntei sendiri. Pada sampel publik
[rifiuti2](https://github.com/abelcheung/rifiuti2), setiap record yang dibaca rifiuti2 memiliki indeks,
waktu penghapusan (hingga detik, sebagaimana ditampilkan rifiuti2), status keluar dari Recycle Bin,
ukuran, dan path yang sama: 92 record, 67 di sebelas berkas INFO2 dari Windows 95 hingga Server 2003 dan
25 di tiga folder berisi berkas `$I` Windows Vista dan Windows 10. Path ANSI INFO2 cocok pada code page
yang digunakan pengujian rifiuti2 sendiri. Juntei juga membaca apa yang ditolak rifiuti2: folder yang
mencampur kedua versi `$I`, header `$I` tanpa path, dan record yang terpotong, masing-masing dengan
kerusakan yang dilaporkan. Berkas `$I` dan INFO2 dari data uji Plaso menghasilkan waktu, ukuran, path,
indeks, dan drive yang diharapkan pengujian Plaso. Pada image disk publik M57-Jean dari Digital Corpora
(Windows XP), yang dibaca dengan `auto-parse --e01`, record INFO2 cocok dengan rifiuti2, dan berkas di
Recycle Bin di sebelahnya, `Dc1.jpg`, memiliki nama yang diberikan Juntei.

## Cache Bitmap RDP { #rdp-bitmap-cache }

`parse-file rdp-cache` adalah parser milik Juntei sendiri. Pada kofu, salah satu dari enam host dalam koleksi
CODE BLUE 2026 CTF, setiap tile dari 1.911 tile di tiga berkas `Cache????.bin`-nya sama, piksel demi piksel,
dengan gambar yang dibuat [bmc-tools](https://github.com/ANSSI-FR/bmc-tools). Begitu pula 1.980 tile dari
`Cache0000.bin` publik milik tantangan "Phantom Connection" ApoorvCTF 2025, 105 di antaranya berukuran
64 × 48 piksel. bmc-tools membaca setiap tile selebar 64 piksel, sehingga tile yang lebih sempit di tepi kanan
layar tampak miring; Juntei membaca setiap tile sesuai lebarnya sendiri.

Tidak ditemukan cache `.bmc` publik yang berisi tile: sampel di atas, seperti klien Windows 10 dan yang lebih
baru lainnya, hanya memiliki `bcache24.bmc` yang kosong. Karena itu, pembacaan `.bmc` diuji dengan berkas yang
dibuat sesuai format: tile tanpa kompresi dari keempat kedalaman warna, tile kecil yang diikuti bagian akhir
tile sebelumnya, slot yang rusak, dan 3.000 tile 16-bit acak terkompresi RLE yang menggunakan setiap order
MS-RDPBCGR 3.1.9, tanpa ada order yang berlanjut melewati akhir baris pertama tile. Juntei dan bmc-tools
mendekode seluruh 3.000 tile tersebut menjadi piksel yang sama. Ketika sebuah order dimulai di baris pertama
dan berlanjut melewatinya, hasil keduanya berbeda: Juntei mengikuti spesifikasi dan dekoder FreeRDP, yang
mempertahankan aturan baris pertama untuk seluruh order, sedangkan bmc-tools mengganti aturan di tengah
order tersebut.

## ShimCache { #shimcache }

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) adalah fork yang dikelola
dari [shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) milik
Security Ronin, dengan dukungan hive registry dan
log transaksi. Pada 20 hive `SYSTEM` publik (24 control set dan 7.290 entri) ditambah 17
nilai AppCompatCache mandiri, setiap entri cocok dengan path, waktu, flag eksekusi, flag, dan
posisi yang dikeluarkan oleh parser referensi.

## Karantina Microsoft Defender { #microsoft-defender-quarantine }

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) adalah
fork yang dikelola dari crate
[defender](https://github.com/Sootmark/defender) milik Sootmark. Folder karantina yang ditulis dalam format
Defender oleh tooling pengujian parser didekode secara konsisten oleh pembaca karantina
`dissect.target` yang independen.

## Artefak turunan registry { #registry-derived-artifacts }

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) dibandingkan dengan
[yarp](https://github.com/msuhanov/yarp) untuk key dan nilai yang teralokasi, pemutaran ulang transaksi pada hive dirty,
pemulihan free cell, cell slack, data di luar hive bin, dan record yang dihapus atau diganti oleh pemutaran ulang.
Perbandingan yang spesifik untuk artefak mencakup:

- layanan, nilai, dan failure action dari hive `SYSTEM` Windows 7 dan Windows 11 yang dirty;
- RecentDocs yang didekode dengan yarp dan libfwsi pada sembilan hive `NTUSER.DAT` publik;
- tugas terjadwal yang didekode secara independen dengan yarp pada tujuh hive `SOFTWARE` publik yang mencakup
  Windows 7–11, Server 2012 R2, dan Server 2022; serta
- struktur akun SAM yang dibandingkan dengan `dissect.target` pada tujuh hive `SAM` publik.

Lihat [Ucapan terima kasih](../acknowledgements.md) untuk daftar lengkap alat referensi, riset
format, spesifikasi, dan sumber bukti publik.
