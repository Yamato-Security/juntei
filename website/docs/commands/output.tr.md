# Ortak Çıktı Yöntemleri { #common-output-patterns }

## Tek bir artefakt veya bir koleksiyon { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d`, o komuta uygun dosyaları özyinelemeli olarak arar. Kaynak yolu sütunları, birleştirilmiş çıktıda ana
bilgisayarları ve artefaktları birbirinden ayırır.

## CSV kaydetme { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Bazı ayrıştırıcılar birden fazla CSV tablosu ürettikleri için bir dizine ihtiyaç duyar:

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## JSONL akışı { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

İletiler ve ilerleme bilgisi standart hataya gider; standart çıktı ayrıştırılabilir kalır.

## Bilinçli olarak üzerine yazma { #overwrite-intentionally }

Juntei varsayılan olarak mevcut çıktının üzerine yazmayı reddeder. Hedefi doğrulayın, önemli olan önceki
sonuçları koruyun, ardından `-C` / `--clobber` ekleyin:

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## Sessiz ve renksiz modlar { #quiet-and-no-color-modes }

Otomasyon için `-q`, terminal renk kodları uygun olmadığında `-K` kullanın:

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## Büyük/küçük harf ve yollar { #case-and-paths }

Boşluk, `$`, kabuk meta karakterleri veya ters eğik çizgi içeren Windows yollarını kabuğunuzun gerektirdiği
şekilde tırnak içine alın. Bash benzeri kabuklarda tek tırnak, `$MFT` ve `$J` ifadelerinin değişkene
dönüşmesini engeller:

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
