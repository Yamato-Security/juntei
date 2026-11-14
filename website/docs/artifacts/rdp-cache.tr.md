# RDP Bitmap Önbelleği { #rdp-bitmap-cache }

## RDP bitmap önbelleği neleri kaydeder { #what-the-rdp-bitmap-cache-records }

Uzak Masaüstü istemcisi (`mstsc.exe`), kalıcı bitmap önbelleği etkinken (varsayılan ayar budur) bağlandığında,
sonraki oturumların bunları yeniden almak zorunda kalmaması için uzak ekranın parçalarını diske kaydeder. Her parça
en fazla 64 × 64 piksellik bir karodur (tile). Karolar bir araya geldiğinde, kullanıcının uzak bilgisayarlarda
gördüklerinin küçük parçalarını oluşturur: pencereler, metinler, dosya adları, komutlar ve masaüstleri.

| İstemci | Önbellek dosyaları | Karolar |
|---|---|---|
| Windows 7 ve sonrası | `Cache0000.bin`, `Cache0001.bin`, …, her biri en fazla 100 MB | 32 bit, art arda |
| Windows XP, Server 2003 ve öncesi | `bcache2.bmc` (piksel başına 8 bit), `bcache22.bmc` (16), `bcache23.bmc` (24) ve `bcache24.bmc` (32), her biri en fazla 10 MB | 64 × 64'lük her yuva için bir karo; bazıları sıkıştırılmış |

Windows 10 ve sonrası, `.bin` dosyalarının yanında boş bir `bcache24.bmc` de bırakır.

<span class="evidence-strength">En iyi sonuç:</span> profili önbelleği içeren kullanıcı, Uzak Masaüstü istemcisiyle
önbellekteki içeriği gösteren bir bilgisayara bağlanmıştır. Önbellek bunun hangi bilgisayar olduğunu veya ne zaman
olduğunu söylemez.

## Konumlar ve toplama { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

`HKCU\Software\Microsoft\Terminal Server Client` anahtarındaki `BitmapPersistCacheLocation` değeri önbelleğin yerini
değiştirebilir. Her profilin `Cache` klasörünün tamamını, birim gölge kopyalarından da toplayın: bir önbellek
dosyasının daha eski bir kopyası, güncel dosyada artık bulunmayan karoları içerebilir.

## Analist için uyarılar { #analyst-cautions }

- Karoların zamanı yoktur ve önbellek, karoların geldiği bilgisayarları adlandırmaz. Karoları başka kanıtlardan
  bilinen RDP bağlantılarıyla ilişkilendirin: istemcinin `Terminal Server Client` kayıt defteri anahtarları
  (`parse-registry remote-access`), `Default.rdp` ve diğer `.rdp` dosyaları, `mstsc.exe` uygulamasının Jump Lists
  verileri, önbellek dosyalarının `$MFT` içindeki zamanları ve uzak bilgisayarların oturum açma olayları.
- Bir önbellek dosyası birçok oturumun ve birkaç uzak bilgisayarın karolarını bir arada içerir. Dosya doldukça
  istemci eski karoları değiştirir; bu nedenle bir karo günler ya da aylar öncesinden kalmış olabilir.
- Karolar ekranların parçalarıdır, ekranın tamamı değildir. Ekranda yan yana olan karolar dosyada da çoğu zaman yan
  yanadır, ancak her zaman değil. Ekranları [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher) ile ya
  da kolajlara bakarak gözle yeniden oluşturun.
- Karoların çoğu 64 × 64 pikseldir. Bir ekranın sağ ve alt kenarlarındaki karolar, çözünürlüğe bağlı olarak daha dar
  veya daha kısadır: örneğin 58 ya da 62 piksel genişliğinde veya 28 ya da 35 piksel yüksekliğinde. Juntei her karoyu
  kendi boyutunda okur. Her karoyu 64 piksel genişliğinde okuyan araçlar dar karoları çarpık gösterir.
- Tek renkli karoların (`SingleColor`) çoğu arka plandır. İçeriği bulmak için bunları filtreleyerek dışarıda bırakın.
- `.bmc` dosyalarında 8 bitlik karolar, önbelleğin saklamadığı oturum paletinin renkleridir. Juntei Windows'un
  varsayılan paletini kullanır; bu nedenle bu karoların renkleri yanlış olabilir. Sıkıştırılmış karolar Uzak Masaüstü
  protokolünün serpiştirmeli RLE'sini (interleaved RLE, MS-RDPBCGR 3.1.9) kullanır; Juntei bunların sıkıştırmasını
  spesifikasyonun tanımladığı şekilde açar.
- 64 × 64'lük yuvasından küçük olan bir `.bmc` karosu, arkasında daha önceki bir karonun sonunu bırakır. Juntei bu
  artık satırları ayrı bir görüntü (`RemnantImage`) olarak kaydeder. Bu satırlar yuvadaki karoya değil, daha eski bir
  karoya aittir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```

```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```

`-d` ile Juntei, adı `Cache` ile başlayıp rakamlarla süren ve `.bin` ile biten dosyaları, ayrıca adı `bcache` ile
başlayıp rakamlarla süren ve `.bmc` ile biten dosyaları okur. `RDP8bmp` imzasıyla başlamayan bir `Cache????.bin`
dosyası başka bir programa aittir: özette sayılır ve atlanır. `-f` ile Juntei dosyayı adı ne olursa olsun okur.

### Görüntüler { #images }

`--extract`, her önbellek dosyası için bir klasör oluşturur: `<DIR>/<user>/<cache file>/`. İki önbellek dosyası aynı
klasöre düşecek olduğunda (örneğin bir önbellek dosyası ve onun bir birim gölge kopyasından alınmış kopyası) ikinci
klasör numaralandırılır (`Cache0001.bin_2`). Klasörde şunlar bulunur:

- `<cache file>_<index>.bmp`: her karo, kendi boyutunda. Adlar bmc-tools'un kullandığı adlarla aynıdır; bu sayede
  RdpCacheStitcher klasörü açabilir.
- `<cache file>_old_<index>.bmp`: yalnızca `.bmc`; daha küçük bir karodan sonra kalan, daha önceki bir karonun sonu.
- `<cache file>_collage.png`: dosya sırasıyla tüm karolar, her satırda 64 tane (`--collage-width`); her karo 64 × 64'lük
  bir hücrenin sol üst köşesinden başlayarak yerleştirilir. *i* indeksli karo, 0'dan sayılarak *i* / 64 numaralı
  satırda ve *i* % 64 numaralı sütundadır. Hücrelerin boş kısımları macenta ile doldurulur: dar bir karonun yanı,
  kısa bir karonun altı ve bir karonun okunamadığı yerler.

`--collage-only` yalnızca kolajları kaydeder. Zaten var olan bir görüntü `-C` olmadan değiştirilmez ve bu durum ilgili
karonun `Problems` alanında belirtilir. Önbellek dosyalarının kendisi `-C` ile bile asla değiştirilmez.

## CSV çıktı alanları { #csv-output-fields }

| Alan | Anlamı |
|---|---|
| `SourceFile` | Önbellek dosyası. |
| `User` | Önbellek dosyasının bulunduğu profil klasörünün kullanıcısı; dosyanın yolundan alınır. |
| `TileIndex` | Karonun dosyadaki numarası; görüntüsünün adında olduğu gibi 0'dan sayılır. |
| `Width`, `Height` | Karonun piksel cinsinden boyutu. |
| `SingleColor` | Karonun tüm pikselleri aynı renkte olduğunda `true`. |
| `TileImage` | `--extract` ile: karonun BMP dosyası. |
| `Collage` | `--extract` ile: önbellek dosyasının kolajı. |
| `RemnantImage` | `--extract` ile `.bmc`: yuvada kalan, daha önceki bir karonun sonu. |
| `Key` | Karonun Uzak Masaüstü protokolündeki 64 bitlik anahtarı, onaltılık olarak. |
| `Format`, `FormatVersion` | İmzadan sonraki sürümüyle birlikte `bin` (Windows 10 ve sonrasında 6) veya `bmc`. |
| `BitsPerPixel` | `.bin` için 32; `.bmc` için 8, 16, 24 veya 32. |
| `Compressed`, `Flags` | `.bmc`: karonun sıkıştırılmış olup olmadığı ve başlığının bayrakları. |
| `Offset` | Karonun başlığının dosyada başladığı konum. |
| `Problems` | Hasar: bir karonun ortasında biten bir dosya, hiçbir karonun sahip olamayacağı bir karo başlığı, karonun boyutuna uymayan veriler, sıkıştırması açılamayan sıkıştırılmış veriler, kaydedilemeyen bir görüntü. |

JSON ve JSONL aynı alanlara sahiptir; sayılar ve boole değerleri kendi türleriyle, bir karoda bulunmayan değerler
`null` olarak ve `Problems` bir liste olarak verilir. Tek bir karoya ait olmayan hasarlar, örneğin son karodan sonra
gelen veriler, çalıştırmanın mesajlarında bildirilir.

## Otomatik ayrıştırma { #automatic-parsing }

`auto-parse`, `LateralMovement/RdpCache.csv` dosyasını (`-t json` veya `-t jsonl` ile `RdpCache.json` ya da
`RdpCache.jsonl`) `LateralMovement/RdpCache-logs.txt` ile birlikte yazar ve her önbellek dosyasının kolajını bir
`LateralMovement/RdpCache/` klasörüne kaydeder. Her karoyu da kaydetmek için `--extract-rdp-tiles` ekleyin; bu, tek bir
kullanıcı için on binlerce dosya anlamına gelebilir. Disk imajlarından önbellek dosyaları kopyalanır. Birleştirilmiş
`-j` zaman çizelgesi her satırı `Artifact` tanımlayıcısı `rdp-cache` ile birlikte içerir. Karoların zamanı olmadığından
[`super-timeline`](../commands/super-timeline.md) bunlardan olay oluşturmaz.

## İnceleme stratejisi { #investigation-strategy }

1. Kolajları açın ve metin arayın: bilgisayar ve kullanıcı adları, dosya yolları, komutlar, konsol pencereleri,
   tarayıcı adres çubukları ve iletişim kutuları.
2. `--extract` ile kaydedilmiş bir karo klasöründen RdpCacheStitcher ile ekranları yeniden oluşturun.
3. Kullanıcının bağlandığı bilgisayarları istemcinin kayıt defterinden (`parse-registry
   remote-access`), `.rdp` dosyalarından ve `mstsc.exe` uygulamasının Jump Lists verilerinden bulun. Bu
   bilgisayarlarda eşleşen RDP oturum açma olaylarını ve Kullanıcı Erişim Günlüğü (UAL) kayıtlarını arayın.
4. Karoların ne zaman yazıldığını daraltmak için önbellek dosyalarının ve bunların birim gölge kopyalarındaki
   kopyalarının `$MFT` zamanlarını kullanın.

## Daha fazla ayrıntı { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: RDP bitmap önbelleği (Fransızca)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — RDP cache plugin](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — persistent bitmap cache](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
