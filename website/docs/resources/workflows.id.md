# Investigasi lintas artefak { #cross-artifact-workflows }

Tidak ada artefak yang boleh menanggung kesimpulan penting sendirian. Alur kerja ini menunjukkan cara mengubah sebuah petunjuk
menjadi rangkaian bukti yang dapat dipertanggungjawabkan.

## Eksekusi executable yang mencurigakan { #suspicious-executable-execution }

1. **Prefetch:** tetapkan waktu eksekusi yang tersimpan, konteks path/hash, jumlah eksekusi, dan berkas yang dimuat.
2. **`$MFT`:** temukan path, ukuran, timestamp, ADS, status terhapus, dan referensi MFT.
3. **Jurnal USN:** rekonstruksi pembuatan, penulisan, penggantian nama, dan penghapusan.
4. **Amcache:** tambahkan SHA-1, metadata PE, versi, dan path historis.
5. **ShimCache:** tambahkan path/urutan cache kompatibilitas dan flag eksekusi yang didukung.
6. **SRUM:** cari penggunaan sumber daya/jaringan oleh executable/pengguna yang sama.
7. **Defender:** periksa waktu karantina, path asli, hash, dan asal unduhan.
8. **Log peristiwa/EDR:** identifikasi parent proses, baris perintah, akun, jaringan, dan konteks keamanan.

Kesimpulan harus membedakan keberadaan berkas, eksekusi, dan atribusi kepada seseorang atau intrusi.

## Aktivitas berkas ransomware { #ransomware-file-activity }

1. Identifikasi ekstensi terenkripsi dan path korban di `$MFT`.
2. Gunakan reason dan urutan USN untuk merekonstruksi penulisan/penggantian nama/penghapusan massal.
3. Gunakan Prefetch untuk mengidentifikasi kandidat program enkripsi dan waktu eksekusinya.
4. Gunakan SRUM untuk lonjakan CPU/disk/jaringan oleh proses kandidat.
5. Periksa layanan dan TaskCache untuk penyebaran/persistensi.
6. Periksa karantina Defender dan log peristiwa untuk deteksi/remediasi.

MFT/USN dapat membuktikan dampak terhadap sistem berkas bahkan ketika tidak ada artefak eksekusi yang tersisa. Keduanya umumnya
tidak mengidentifikasi proses yang bertanggung jawab tanpa telemetri proses atau korelasi.

## Akses awal melalui dokumen / pintasan berbahaya { #malicious-document-shortcut-initial-access }

1. RecentDocs dan Jump Lists mengidentifikasi interaksi pengguna/aplikasi dan konteks MRU.
2. Uraikan LNK yang sesuai di folder Recent untuk target, volume/share, referensi MFT, dan argumen.
3. Gunakan Zone.Identifier dari ADS resident di `$MFT` atau konten Defender untuk URL/zona asal.
4. Gunakan Prefetch/log peristiwa untuk aplikasi dokumen dan proses child-nya.
5. Gunakan USN/MFT untuk mengikuti pembuatan dan penghapusan payload yang diekstrak.

## Persistensi layanan { #service-persistence }

1. Dekode nama layanan, `ImagePath`, `ServiceDll`, akun, tipe start, dan tindakan kegagalan.
2. Selesaikan variabel lingkungan dan perilaku path dengan/tanpa tanda kutip.
3. Temukan executable/DLL di `$MFT` dan riwayat USN.
4. Periksa Prefetch dan log peristiwa untuk start layanan/proses yang sebenarnya.
5. Bandingkan control set yang tidak aktif dan record registry yang dipulihkan untuk konfigurasi sebelumnya.

## Persistensi tugas terjadwal { #scheduled-task-persistence }

1. Dekode tindakan TaskCache, principal/konteks, metadata registrasi/runtime, dan status tersembunyi.
2. Bandingkan dengan XML di `C:\Windows\System32\Tasks`.
3. Korelasikan nama/GUID tugas dengan peristiwa Security dan Task Scheduler Operational.
4. Temukan dan hitung hash executable/skrip; periksa Prefetch dan riwayat sistem berkas.
5. Pulihkan record TaskCache yang dihapus jika tugas tersebut telah dihapus.

## Dugaan eksfiltrasi { #suspected-exfiltration }

1. Gunakan SRUM untuk mengidentifikasi byte terkirim yang tidak biasa menurut aplikasi/pengguna/profil dan periode pengukuran.
2. Konfirmasikan identitas executable dengan Prefetch, Amcache, dan `$MFT`.
3. Identifikasi berkas yang diakses/dibuka melalui Jump Lists, LNK, RecentDocs, dan data peristiwa/EDR.
4. Gunakan telemetri proxy, firewall, DNS, paket, atau EDR untuk tujuan/protokol.

Volume SRUM mendukung hipotesis eksfiltrasi tetapi tidak mengidentifikasi konten yang ditransfer atau membuktikan
niat jahat dengan sendirinya.

## Kunci korelasi yang layak dipertahankan { #correlation-keys-worth-preserving }

| Kunci | Artefak |
|---|---|
| Path lengkap / nama berkas | Hampir setiap artefak |
| SHA-1 / SHA-256 | Amcache, Defender, berkas yang dipulihkan |
| Entri MFT + sequence | `$MFT`, Prefetch, LNK, Jump Lists, USN |
| USN | Amcache, `$MFT`, Jurnal USN |
| Nomor seri volume / droid | Prefetch, LNK, Jump Lists, object ID MFT |
| SID / nama pengguna | SRUM, SAM, registry, log peristiwa |
| GUID tugas / nama tugas | TaskCache, XML tugas, log peristiwa |
| Nama layanan | Layanan, log peristiwa, data proses/driver |
| EntryId / ScanId Defender | Karantina, riwayat Defender, log peristiwa |
