# Önyükleme Sektörü (`$Boot`) { #boot-sector-boot }

## `$Boot` nedir { #what-boot-is }

`$Boot`, her NTFS biriminin 7 numaralı MFT girdisidir: birimin ilk 8 KiB'ı. İlk sektörü olan
**birim önyükleme kaydı** (volume boot record), dosya sistemini tanımlayan BIOS parametre bloğunu tutar:

- sektör ve küme (cluster) boyutları, birimdeki sektör sayısı ve **gizli sektörler** (diskte birimden
  önce gelen sektörler, yani bölümünün başladığı yer);
- `$MFT` ve `$MFTMirr` yapılarının nerede başladığı ve FILE ile INDX kayıtlarının boyutları;
- birim biçimlendirildiğinde ayarlanan 64 bitlik **birim seri numarası**.

İlk sektörün geri kalanı ve onu izleyen 15 sektör, Windows'u yükleyen **önyükleme kodunu** tutar.

Seri numarasının düşük 32 biti, Windows'un `XXXX-XXXX` olarak gösterdiği değerdir (`vol`, `dir`) ve
[LNK kısayollarının](lnk.md), [Jump Lists](jump-lists.md) ve [Prefetch](prefetch.md) dosyalarının bir
dosyanın bulunduğu birim hakkında kaydettiği değerdir. `$Boot`, bu kayıtları toplanan bir ana bilgisayarın
bir birimine bağlar.

<span class="evidence-strength">En iyi sonuç:</span> bir `$Boot` dosyasının `VolumeSerialNumber`
değerine eşit olan bir LNK, Jump List veya Prefetch birim seri numarası, o birime ya da aynı seri numarası
verilmiş veya aynı seri numarasıyla kopyalanmış başka bir birime işaret eder. Seri numaraları
değiştirilebilir ve ana bilgisayarlar arasında benzersiz değildir.

## Konum ve toplama { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

Bunu ham erişimi destekleyen bir adli bilişim aracıyla toplayın veya bir birim imajının ilk sektörünü
okuyun: ikisi aynı baytlardır. Juntei, bir NTFS önyükleme sektörüyle başlayan her dosyayı okur; bu nedenle
bir birim imajı da girdi olarak çalışır. Bir disk imajı ise bölüm tablosuyla başlar; Juntei'ye bölümün
birimini verin.

## Analist için uyarılar { #analyst-cautions }

- Seri numarası, birim biçimlendirildiğinde değişir. Seri numarası hiçbir geçerli birimle eşleşmeyen bir
  LNK dosyası çıkarılabilir bir ortama, bir ağ paylaşımının sunucusuna veya o zamandan beri biçimlendirilmiş
  bir birime işaret ediyor olabilir.
- Önyükleme kodu, aynı Windows tarafından biçimlendirilen birimlerde aynıdır: DFIR Artifact Museum'un
  Windows 10, 11 ve Server 2012 R2 `$Boot` dosyalarında birebir aynıdır. Bir birim önyükleme kaydı
  bootkit'i bunu veya önyükleme kodunun `$Boot` yapısının geri kalanını bulmak için kullandığı gizli
  sektörler alanını değiştirir. Herhangi bir sonuca varmadan önce `BootCodeSHA256`, `JumpInstruction` ve
  `HiddenSectors` değerlerini diğer ana bilgisayarlarla ve bölüm tablosuyla karşılaştırın: bir önyükleme
  yöneticisi veya şifreleme yazılımı da önyükleme kodunu değiştirebilir.
- BitLocker ile şifrelenmiş bir birimin imajı, BitLocker'ın kendi önyükleme sektörüyle başlar (OEM ID
  `-FVE-FS-`). `-f` ile verildiğinde Juntei bunu raporlar ve `Problems` bunun NTFS olmadığını belirtir;
  NTFS önyükleme sektörü şifrelenmiş verinin içindedir. Kilidi açılmış birimden toplanan bir `$Boot`
  ise NTFS önyükleme sektörüdür.
- `Problems`, eksiksiz bir Windows bağlanabilirlik testi sunmak yerine tanınan yapısal tutarsızlıkları ve
  kesilmeleri listeler. Bunu oyulmuş (carved) sektörleri ve kopyaları değerlendirmek için kullanın; tek
  başına bir saldırı bulgusu olarak değil. Önyükleme kodu karması ilk atlama talimatını, BIOS parametre
  bloğunu ve bitiş imzasını hariç tutar; bu alanları da inceleyin.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

Özet, her önyükleme sektörünün birim seri numarasını (Windows'un gösterdiği biçimde ve LNK ile Prefetch
alanlarının gösterdiği biçimde), boyutunu, küme boyutunu ve `$MFT` kümesini listeler. Birden fazla ana
bilgisayar ayrıştırıldığında bunları önyükleme kodu karmasına göre gruplar; böylece önyükleme kodu
diğerlerinden farklı olan bir birim göze çarpar.

## CSV çıktı alanları { #csv-output-fields }

| Alan | Yorumlama |
|---|---|
| `SourcePath` | Girdi dosyası. |
| `VolumeSerialNumber` | LNK, Jump List ve Prefetch çıktısının gösterdiği biçimde 32 bitlik seri numarası (`180B1E67`). Windows bunu `180B-1E67` olarak gösterir. |
| `VolumeSerialNumber64` | 64 bitlik seri numarasının tamamı. |
| `OEM-ID` | Bir NTFS biriminde `NTFS`. |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | Birimin geometrisi. 128'in üzerindeki küme başına sektör sayısı ikinin kuvveti olarak kaydedilir ve burada kodu çözülür. Geçerli değilse boştur. |
| `TotalSectors`, `VolumeSize` | Birimdeki sektörler ve bayt cinsinden boyutları; sonundaki önyükleme sektörü kopyası hariç. |
| `HiddenSectors` | Diskte birimden önce gelen sektörler: bölümünün başladığı yer. |
| `MFT-Cluster`, `MFT-Offset` | `$MFT` yapısının nerede başladığı; küme cinsinden ve birimin başından itibaren bayt cinsinden. |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | `$MFTMirr` yapısının nerede başladığı. |
| `FileRecordSize`, `IndexRecordSize` | FILE kaydı başına (genellikle 1.024) ve INDX kaydı başına (genellikle 4.096) bayt. |
| `MediaDescriptor` | Sabit disk için `F8`. |
| `JumpInstruction` | İlk üç bayt (`EB 52 90`): BIOS parametre bloğunun üzerinden önyükleme koduna atlama. |
| `EndSignature` | Geçerli bir önyükleme sektörünün sonundaki `55 AA`. |
| `BootCodeSHA256` | Önyükleme kodunun SHA-256 değeri: ilk sektörün 0x54 ile 0x1FD arasındaki baytları ve 8 KiB'ın geri kalanı. |
| `Problems` | Windows'un bağlamayacağı durumlar: `NTFS` dışında bir OEM ID, `55 AA` olmaması, geçersiz boyutlar, NTFS'nin sıfır tuttuğu FAT alanlarının ayarlanmış olması, birimin sonunu aşan `$MFT`; ayrıca 8 KiB'dan kısa bir dosya. |

JSON/JSONL aynı alanlara ve adlara sahiptir; sayılar sayı olarak, bilinmeyen değerler `null` olarak ve
`Problems` bir liste olarak verilir.

## İnceleme stratejisi { #investigation-strategy }

1. Her ana bilgisayarın `VolumeSerialNumber` değerini LNK dosyalarının, Jump Lists ve Prefetch
   dosyalarının birim seri numaralarının yanına koyun: toplanan hiçbir birimle eşleşmeyenler başka
   birimlerdir (USB sürücüler, paylaşımlar, o zamandan beri biçimlendirilmiş birimler).
2. Diğer seri numaralarını, bağlanan çıkarılabilir birimlerin birim seri numaralarını listeleyen
   [USB aygıtları](usb.md) çıktısında arayın.
3. `BootCodeSHA256` ve `HiddenSectors` değerlerini ana bilgisayarlar arasında karşılaştırın. Farklı bir
   önyükleme kodunu bir disassembler ve diskin bölüm tablosuyla inceleyin ve önyüklemeyle ilgili olay
   günlüğü girdilerini arayın.
4. Bir birim imajında `$MFT` yapısını bulmak için `MFT-Offset` ve `ClusterSize` değerlerini kullanın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: NTFS overview](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: NTFS format documentation](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (birim başlığı)
- [The Sleuth Kit: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`boot` crate'i)
