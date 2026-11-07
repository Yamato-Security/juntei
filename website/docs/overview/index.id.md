# Tentang Juntei { #about-juntei }

Juntei adalah parser artefak forensik Windows yang ditulis dalam Rust yang aman memori (memory-safe). Juntei melengkapi
[Hayabusa](https://github.com/Yamato-Security/hayabusa): Hayabusa menganalisis log peristiwa Windows,
sedangkan Juntei mengurai artefak persisten dari sistem berkas, registry, eksekusi, aktivitas pengguna, penggunaan sumber daya,
dan produk keamanan.

Binary yang sama berjalan di Windows, macOS, dan Linux. Analis dapat memeriksa bukti Windows yang telah dikumpulkan dan image disk dengan satu antarmuka konsisten, bukan program terpisah untuk setiap artefak.

## Tujuan desain { #design-goals }

- **Penguraian yang dapat dipertanggungjawabkan:** Mempertahankan nilai mentah dan melaporkan kerusakan alih-alih diam-diam mengarang data.
- **Berguna bagi semua orang:** Penggunaan default harus mudah bagi pemula, tetapi juga dapat disesuaikan dan memiliki fitur
  lengkap bagi para veteran.
- **Binary mandiri:** Tidak memerlukan pustaka runtime apa pun untuk dieksekusi. Langsung berfungsi begitu
  digunakan.
- **Sangat cepat:** Bertujuan menjadi parser tercepat sekaligus tidak memerlukan memori dalam jumlah
  besar.
- **Penggunaan lintas platform:** Menganalisis bukti Windows tanpa memerlukan workstation Windows.
- **Otomatisasi:** CSV untuk spreadsheet dan alat timeline; JSON/JSONL untuk `jq`, skrip, dan data lake.
- **Kegagalan yang aman:** Catatan yang cacat dilaporkan dan dilewati bila memungkinkan.
- **Asal-usul yang transparan:** Juntei mengidentifikasi parser open-source dan alat referensi di balik setiap implementasi.

## Apa yang tidak dilakukan Juntei { #what-juntei-does-not-do }

Juntei tidak melakukan akuisisi disk secara live dan tidak menggantikan preservasi bukti, dan satu-satunya image disk
yang di-mount-nya adalah image VHDX, dalam mode baca saja. Juntei
tidak secara otomatis menggabungkan setiap artefak menjadi satu narasi insiden. Juntei mengurai
bukti yang Anda berikan dan menghasilkan catatan yang sesuai untuk investigasi dan korelasi.

Juntei juga tidak akan melakukan analisis untuk Anda. Diperlukan seorang spesialis DFIR untuk melakukan analisis terhadap bukti.

## Lisensi { #license }

Juntei dirilis di bawah
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) dan hak ciptanya dimiliki oleh Shirofune Security Co., Ltd. Kode sumbernya dipublikasikan di bawah
AGPLv3 sebelum versi 1.0.0. Lisensi crate serta kode dan data pihak ketiga lainnya yang digunakan untuk
membangunnya tercantum di
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
Proyek-proyek yang menjadi dasar Juntei dan yang digunakan sebagai pembanding dalam pengujiannya tercantum di
[Ucapan terima kasih](../acknowledgements.md).
