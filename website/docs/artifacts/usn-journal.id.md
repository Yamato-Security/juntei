# Jurnal USN (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## Apa itu Jurnal USN { #what-the-usn-journal-is }

NTFS mencatat perubahan berkas dan direktori di Update Sequence Number (USN) Change Journal. Sebuah
record dapat mendeskripsikan pembuatan, perluasan/penimpaan data, perubahan atribut atau keamanan, rename,
penutupan, dan penghapusan. Record menyimpan referensi berkas, referensi induk, nama, alasan, atribut,
timestamp, dan USN yang terus meningkat secara monoton.

Jurnal ini sangat baik untuk merekonstruksi rentetan operasi berkas yang dilakukan penyerang, terutama ketika
berkasnya telah dihapus. Jurnal ini mencatat **bahwa metadata/konten berubah**, bukan byte yang berubah.

<span class="evidence-strength">Kesimpulan terbaik:</span> NTFS mencatat alasan perubahan yang dinyatakan untuk
berkas yang dirujuk pada timestamp tersebut. Alasan sering kali terakumulasi hingga record `CLOSE`.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` adalah named stream dari metadata NTFS. Kumpulkan dengan alat forensik yang mendukung akses raw. Kumpulkan juga
`$MFT` dari volume yang sama, karena record USN menyimpan referensi berkas induk, bukan path lengkap.

## Peringatan bagi analis { #analyst-cautions }

- Jurnal bersifat sirkular dan ukurannya terbatas. Record lama akan ditimpa.
- `FILE_DELETE` menyatakan bahwa NTFS memproses penghapusan; konten mungkin masih dapat dipulihkan di tempat lain.
- Alasan seperti `DATA_OVERWRITE` tidak mengungkapkan byte, proses, atau pengguna yang bertanggung jawab.
- Beberapa alasan dapat digabungkan dalam satu record; `CLOSE` sering menandai akhir dari perubahan yang terakumulasi.
- Path direkonstruksi. Direktori induk mungkin telah di-rename, dihapus, atau digunakan ulang.
- Sequence number pada referensi berkas itu penting. Entry number saja dapat terhubung ke generasi yang salah.
- Carving dapat memulihkan record yang valid tanpa konteks jurnal aslinya; laporkan perbedaan tersebut.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## Field keluaran CSV { #csv-output-fields }

| Field | Interpretasi |
|---|---|
| `Name` | Nama berkas/direktori yang disimpan dalam record USN ini. Ini bukan path lengkap. |
| `Extension` | Ekstensi yang diurai dari `Name` untuk kemudahan; jangan gunakan sebagai tipe konten yang tepercaya. |
| `EntryNumber`, `SequenceNumber` | Referensi NTFS milik berkas. Gunakan keduanya untuk mengidentifikasi generasi record. |
| `ParentEntryNumber`, `ParentSequenceNumber` | Referensi NTFS direktori induk pada saat perubahan. |
| `ParentPath` | Path induk yang direkonstruksi Juntei dengan memperhitungkan waktu. Penanda unknown berarti silsilahnya tidak dapat di-resolve. |
| `UpdateSequenceNumber` | Nilai pengurutan USN. Berguna untuk pengurutan yang stabil dan pivot ke Amcache/MFT. |
| `UpdateTimestamp` | Timestamp yang disimpan dalam record V2/V3. Perlakukan timestamp dari konteks carving dengan validasi yang sama seperti record lainnya. |
| `UpdateReasons` | Satu atau lebih flag perubahan bernama, digabungkan dengan `|`. Interpretasikan kombinasi dan `CLOSE` secara bersamaan. |
| `FileAttributes` | Atribut NTFS yang dicatat pada saat perubahan; bit yang tidak dikenal tetap dalam heksadesimal. |
| `OffsetToData` | Offset byte di dalam input. Penting untuk validasi dan record hasil carving. |
| `SourceFile` | Jurnal/image input; membedakan host dan volume. |
| `SourceInfo` | Flag sumber yang mendeskripsikan bagaimana perubahan masuk ke jurnal. Biasanya kontekstual, bukan atribusi. |
| `SecurityID` | Indeks pengenal keamanan NTFS untuk record tersebut, bukan SID pengguna. |
| `Version` | Versi record USN. Juntei menulis V2/V3 ke CSV; JSON mempertahankan extent range-tracking V4. |

JSON/JSONL merepresentasikan referensi V3 128-bit dengan benar dan mempertahankan record range-tracking V4 beserta
extent-nya. Gunakan format ini ketika field tingkat rendah yang spesifik versi menjadi penting.

## Strategi investigasi { #investigation-strategy }

1. Filter berdasarkan waktu, nama, ekstensi, path induk, atau referensi berkas.
2. Urutkan berdasarkan `UpdateSequenceNumber` dan juga waktu untuk melihat urutan jurnal yang tepat.
3. Pasangkan `RENAME_OLD_NAME` dengan `RENAME_NEW_NAME` untuk pemindahan/rename.
4. Gunakan `$MFT` untuk memvalidasi kondisi record, path, ukuran, dan timestamp.
5. Korelasikan atribusi proses dari log peristiwa, EDR, Prefetch, layanan/tugas, atau Defender.

## Detail lebih lanjut { #further-details }

- [Microsoft: Change Journal](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Flag alasan Change Journal](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [Crate usn dari ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
