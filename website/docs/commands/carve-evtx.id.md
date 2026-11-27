# Carving Log Peristiwa { #carve-evtx }

Pulihkan chunk log peristiwa yang tersisa setelah sebuah log dikosongkan atau dihapus: dari image disk, ruang yang tidak teralokasi, pagefile, atau image memori. Setiap chunk EVTX berukuran 64 KiB membawa tabel string dan templatnya sendiri, sehingga record-nya dapat dibaca tanpa bagian log lainnya. Juntei menulis chunk yang dipulihkan sebagai berkas `.evtx` biasa, satu per channel, untuk dibuka di Hayabusa atau Event Viewer.

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
```

Tanpa `-o`, hanya ringkasan yang ditampilkan.

## Menemukan chunk { #finding-chunks }

- Signature chunk dicari di setiap offset byte, dan setiap header harus masuk akal (ukuran header, nomor record, offset). Checksum dilaporkan, tetapi tidak diwajibkan: chunk yang sebagian sudah tertimpa tetap menyimpan record yang dapat dibaca.
- Chunk dari log yang dikompresi NTFS didekompresi (LZNT1, unit 16 cluster). Pada image disk, ukuran cluster volume yang digunakan; untuk berkas lain, `--cluster-size <BYTES>` (bawaan 4096).
- Record ditelusuri berdasarkan ukurannya hingga offset record terakhir yang tercatat di header, dan penelusuran tidak pernah berhenti karena ID record yang berulang. Setelah record yang rusak, penelusuran berlanjut di record valid berikutnya. Record lama di slack chunk juga dicantumkan.

## Image disk dan volume NTFS { #disk-images }

Pada image E01 atau VHDX, atau disk atau volume mentah dengan NTFS, setiap chunk dikelompokkan berdasarkan cluster tempatnya berada:

| Lokasi | Arti |
|---|---|
| `FreeSpace` | Cluster yang tidak digunakan berkas mana pun. |
| `DeletedLog` | Cluster bebas yang masih dicantumkan oleh record `$MFT` milik log yang terhapus, atau log yang dibangun ulang dari cluster tersebut. |
| `LiveLog` | Cluster milik log peristiwa yang sedang digunakan. Tidak disertakan kecuali dengan `--keep-live`. |
| `AllocatedFile` | Cluster milik berkas lain, seperti `pagefile.sys` atau berkas penyimpanan shadow copy. |
| `AllocatedUnknown` | Cluster teralokasi yang tidak dicantumkan oleh record `$MFT` mana pun. |
| `OutsideVolumes` | Di luar semua volume NTFS pada disk. |
| `Unclassified` | Input tidak memiliki volume NTFS (pagefile atau image memori). |

Log terhapus yang record `$MFT`-nya hanya mencantumkan cluster bebas dibangun ulang secara utuh, termasuk yang terkompresi, lalu disimpan di bawah `Recovered/`. `--free-space-only` hanya mencari di cluster bebas: lebih cepat, tetapi melewatkan pagefile, shadow copy, dan berkas lainnya.

## Deduplikasi { #deduplication }

Setiap chunk ditulis satu kali. Sebuah chunk tidak disertakan jika chunk itu milik log aktif, identik dengan chunk lain, atau semua record-nya sudah ada di log aktif atau di chunk yang sudah ditulis. Chunk dengan record lebih banyak dipertimbangkan lebih dulu, sehingga salinan chunk yang lebih lama dan lebih pendek tidak menambahkan apa pun. Untuk input tanpa NTFS, berikan log aktif dengan `--live-logs <DIR>`.

## Hasil { #results }

- `Carved/<CHANNEL>.evtx`: satu log per channel, dinamai seperti cara Windows menamai log (`Microsoft-Windows-Sysmon%4Operational.evtx`). Nomor record di header chunk dinomori ulang dan checksum dihitung ulang. Record yang rusak dan semua yang ada setelahnya menjadi slack. Selain itu, setiap chunk disimpan utuh, termasuk slack-nya. Opsi `-x` Hayabusa memindai chunk yang ditandai kosong. Lihat `records.csv` dari Juntei untuk menemukan rekaman di ruang slack pada chunk yang terisi sebagian.
- `Recovered/Volume<N>/...`: log terhapus yang dibangun ulang dari record `$MFT`-nya.
- `chunks.csv` / `chunks.jsonl`: setiap chunk yang ditemukan: lokasinya, apa yang terjadi padanya (`Status`), channel, ID record, waktu, dan status checksum.
- `records.csv` / `records.jsonl`: setiap record dari chunk yang ditulis. `OnlyInCarvedData` menandai record yang tidak ada di log aktif.

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

Carving hanya menemukan chunk yang tersimpan berurutan. Chunk yang terbagi di cluster yang tidak berurutan, atau di halaman fisik memori yang tidak berurutan, tidak lengkap, dan record-record berikutnya bisa rusak atau hilang. Dekompresi `hiberfil.sys` terlebih dahulu. Kode keluar bernilai 0 jika pencarian selesai dan 1 jika terjadi kesalahan.

## Rincian lebih lanjut { #further-details }

- [Penggunaan lengkap dan referensi kolom (Inggris)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.md)
- [Penggunaan lengkap dan referensi kolom (Jepang)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.ja.md)
- [Format EVTX (dokumentasi libevtx)](https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc)
