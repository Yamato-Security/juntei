# Panduan artefak { #artifact-guide }

Windows menyimpan jejak yang saling tumpang tindih di sistem berkas, registry, database kompatibilitas,
riwayat antarmuka pengguna, pencatatan penggunaan sumber daya, dan produk keamanan. Juntei mengurai sumber-sumber ini,
tetapi investigator tetap harus menginterpretasikan makna dari setiap sumber.

## Memilih artefak berdasarkan pertanyaan { #choose-an-artifact-by-question }

| Pertanyaan investigasi | Mulai di sini | Dukung dengan |
|---|---|---|
| Apakah executable ini berjalan? | [Prefetch](prefetch.md) | [ShimCache](shimcache.md), [SRUM](srum.md), log peristiwa |
| Apakah berkas ini pernah ada? | [Amcache](amcache.md), [$MFT](mft.md) | [Jurnal USN](usn-journal.md), [$I30](i30.md), ShimCache |
| Kapan sebuah berkas dibuat, diubah namanya, atau dihapus? | [Jurnal USN](usn-journal.md) | [$MFT](mft.md), [$I30](i30.md), karantina Defender |
| Nama atau timestamp apa yang ada sebelum perubahan metadata? | [$LogFile](logfile.md) | [$MFT](mft.md), [Jurnal USN](usn-journal.md), [$I30](i30.md) |
| Siapa pemilik sebuah berkas, dan siapa yang boleh mengubahnya? | [$Secure:$SDS](secure-sds.md) dengan [$MFT](mft.md) | [SAM](sam.md), log peristiwa Security |
| Di volume mana target sebuah pintasan atau berkas Prefetch berada? | [$Boot](boot.md) | [LNK](lnk.md), [Prefetch](prefetch.md), [perangkat USB](usb.md) |
| Apa yang dibuka oleh pengguna? | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| Apakah sebuah program mentransfer data dalam jumlah besar? | [SRUM](srum.md) | Telemetri firewall, proxy, DNS, paket, atau EDR |
| Situs web mana yang dikunjungi pengguna, dan apa yang diunduhnya? | [Browser web](browser.md), [Internet Explorer dan Edge lama](webcache.md) | Log proxy dan DNS, `$MFT` dan Jurnal USN, Defender, SRUM |
| Klien dan akun mana yang menggunakan server ini, dan pada hari apa saja? | [Pencatatan Akses Pengguna (UAL)](ual.md) | Log peristiwa Security, log SMB dan RDP, log firewall |
| Apakah persistensi dikonfigurasi? | [Layanan](services.md), [TaskCache](taskcache.md), [Registry](registry.md) | Log peristiwa, Prefetch, `$MFT` |
| Apa yang dihapus oleh Defender? | [Karantina Defender](defender-quarantine.md) | Log peristiwa, `$MFT`, Jurnal USN |
| Apakah akun lokal ditambahkan atau diubah? | [SAM](sam.md) | Log peristiwa Security dan riwayat transaksi registry |

## Baca peringatan sebelum field { #read-warnings-before-fields }

Setiap halaman artefak mengikuti urutan yang sama:

1. **Apa itu artefak tersebut** dan pertanyaan apa yang dapat dijawabnya.
2. **Di mana menemukannya** dan bagaimana pengumpulan memengaruhinya.
3. **Apa yang dibuktikannya—dan apa yang tidak dibuktikannya.**
4. **Cara mengurainya** dengan Juntei.
5. **Cara membaca setiap field keluaran.**
6. **Cara mengorelasikannya** dengan bukti independen.
7. **Detail lebih lanjut** untuk riset tingkat rendah.

!!! danger "Jangan menyamakan temuan parser dengan keberbahayaan"
    Temuan Juntei mengidentifikasi pola yang layak ditinjau, seperti nama binary sistem di luar
    `System32`. Installer, administrator, lapisan kompatibilitas, dan pembaruan perangkat lunak dapat menghasilkan
    record yang tidak biasa. Validasi dengan konteks dan bukti independen.
