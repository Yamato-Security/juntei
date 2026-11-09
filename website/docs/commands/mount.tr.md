# Disk İmajlarını Bağlama { #mounting-disk-images }

`juntei mount`, çalıştığı sürece bir disk imajındaki (VHDX veya E01 imajı) dosyaları bir dizinde salt okunur olarak sunar. Hayabusa ve Juntei gibi araçlar dosyaları çıkarmadan veya root yetkisi gerektirmeden inceleyebilir. FUSE kullanır ve Linux'ta çalışır. VHDX okuyucusu ve bağlama kitaplığı [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) deposunda, E01 okuyucusu ise [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) deposunda geliştirilmektedir.

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

Bir E01 imajı (EnCase'in Expert Witness Format'ı, `.Ex01` dahil) aynı şekilde bağlanır. Parçalı bir imaj ilk segmentiyle verilir ve Juntei diğer segmentleri (`host1.E02`, `host1.E03`, …) onun yanında bulur:

```bash
juntei mount host1.E01 /mnt/host1
```

İkinci bir terminalde:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

Bağlamayı kaldırmak için ilk terminalde Ctrl+C tuşlarına basın veya `fusermount3 -u /mnt/host1`
komutunu çalıştırın.

## Bağlamanın gösterdikleri { #what-the-mount-shows }

- Bağlama noktasında NTFS biriminin dosyaları görünür. Toplanmış bir imajda `C` ve `LongFileNames` gibi dizinler ile `$MFT` gibi birim meta verileri bulunabilir. Birden çok NTFS birimi `Volume1`, `Volume2` vb. olarak gösterilir.
- Her dosya salt okunurdur ve bağlama noktasının sahibine aittir. Dosya zamanları, NTFS
  `$STANDARD_INFORMATION` zamanlarıdır.
- Bölüm tablosu olmayan, tek bir birimin imajı o birimin kendisidir.
- Adlar NTFS'in sakladığı şekliyle listelenir ve birebir eşleşmeyle aranır. DOS 8.3 adları ve
  alternatif veri akışları gösterilmez.
- Sıkıştırılmış dosyaların sıkıştırması açılır (her biri için en fazla 1 GiB). EFS ile şifrelenmiş
  dosyalar okunamaz.

## Kanıt değiştirilmez { #the-evidence-is-not-changed }

İmaj salt okunur açılır ve bağlama yazma işlemlerini reddeder. Düzgün kapatılmamış bir VHDX'in bekleyen günlüğü bellekte uygulanır, imaja geri yazılmaz. Bazı DiscUtils yazıcıları NTFS önyükleme sektörünün atlama komutunu ve `55 AA` imzasını sıfır bırakır; Juntei bu birimleri olduğu gibi okur.

Bir E01 imajının segmentleri de salt okunur açılır. Her öbek okunurken sağlama toplamıyla denetlenir: Hasarlı bir öbek okunamaz, imajın geri kalanı ise okunabilir. Edinim sırasında okunamayan sektörler ve tamamlanmamış bir edinim uyarı olarak bildirilir. Mantıksal kanıt dosyası (`.L01`, `.Lx01`) disk değil dosya içerir ve bağlanmaz.

## Kullanım { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## auto-parse ile her imajı ayrıştırma { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx`, ayrıca bağlama gerektirmeden, kanıt dizinindeki her VHDX disk imajının (veya
`-d` ile verilen tek imajın) içindeki Windows sistemlerini, `auto-parse --e01` ise her E01 imajının
içindekileri ayrıştırır. İki seçenek birlikte verilebilir:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d /cases/e01 --e01 -o parsed
```

- Her imaj, yeni bir geçici dizinde, imajın adını taşıyan bir klasöre (`host1-triage`) salt okunur
  olarak bağlanır ve orada bulunan sistemler, diskteki bir koleksiyonda olduğu gibi ayrıştırılır.
  İmajların bağlantısı, çalıştırmanın sonunda ve Ctrl+C ile kaldırılır.
- Terminal, imajları ve her sistemin imajını listeler:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- `--e01`, her E01 imajının ilk segmentini (`*.E01`, `*.Ex01`) alır; diğer segmentler onunla birlikte
  okunur.
- Bildirim dosyasında `EvidenceDirectory` verilen dizindir ve her sistemin `Image` değeri o sistemin
  imajını belirtir. `EvidenceRoot` ve ayrıştırıcı komutları bağlama klasörünü (`./host1-triage/C`)
  kullanır.
- İmaj, Volume Shadow Copies dosyalarını `VSS32` gibi klasörlerde içeriyorsa her klasör başka bir sistem (`HOST1_2`, …) olarak bulunur ve ayrıştırılır.
- Bağlanamayan bir imaj raporlanır; diğerleri yine de ayrıştırılır ve çalıştırma 1 çıkış durumuyla
  sona erer.

Bir NTFS birimi açılamazsa Juntei sorunu bildirir ve okunabilen birimleri korur. Kısmen okunabilen imajlarda `auto-parse --vhdx` ve `--e01` çıkış kodu 1 döndürür; `mount` da ayırma işleminden sonra 1 döndürür.

## Gereksinimler { #requirements }

- Linux: FUSE 3 (`fusermount3`, `fuse3` paketi) ve normal kullanıcıların genellikle sahip olduğu
  `/dev/fuse` erişimi. Root gerekmez.
- Windows ve macOS desteklenmez. Windows'ta bunun yerine bir VHDX imajını yönetici olarak çalıştırılan
  bir PowerShell'de salt okunur olarak ekleyin ve araçlara sürücü harfini verin. Windows bir E01 imajını
  ekleyemez: İmajı bunu yapabilen bir araçla salt okunur olarak bağlayın ve araçlara o sürücüyü verin.

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
