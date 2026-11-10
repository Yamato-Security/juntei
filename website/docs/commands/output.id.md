# Metode keluaran umum { #common-output-patterns }

## Satu artefak atau satu hasil pengumpulan { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d` mencari secara rekursif berkas-berkas yang sesuai untuk perintah tersebut. Kolom source-path membedakan
host dan artefak dalam keluaran gabungan.

## Menyimpan CSV { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Beberapa parser memerlukan direktori karena menghasilkan beberapa tabel CSV:

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## Streaming JSONL { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

Pesan dan progres dikirim ke standard error; standard output tetap dapat diurai.

## Menimpa secara sengaja { #overwrite-intentionally }

Secara default, Juntei menolak menimpa keluaran yang sudah ada. Konfirmasikan target, pertahankan hasil
sebelumnya yang penting, lalu tambahkan `-C` / `--clobber`:

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## Mode senyap dan tanpa warna { #quiet-and-no-color-modes }

Gunakan `-q` untuk otomatisasi dan `-K` bila kode warna terminal tidak sesuai:

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## Huruf besar/kecil dan path { #case-and-paths }

Beri tanda kutip pada path Windows yang mengandung spasi, `$`, metakarakter shell, atau backslash sesuai kebutuhan
shell Anda. Pada shell mirip Bash, tanda kutip tunggal mencegah `$MFT` dan `$J` menjadi variabel:

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
