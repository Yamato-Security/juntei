# Disk İmajlarını Bağlama { #mounting-disk-images }

`juntei mount`, bir VHDX disk imajının içindeki dosyaları, çalıştığı sürece bir dizinde salt okunur
olarak okunabilir hâle getirir. Böylece KAPE'nin `--vhdx` koleksiyonları ve Hyper-V diskleri, Hayabusa
veya Juntei'nin kendi komutları gibi herhangi bir araçla taranabilir; bunun için hiçbir şeyi dışarı
çıkarmak, root yetkisi veya işletim sisteminin VHDX ya da NTFS desteklemesi gerekmez. FUSE kullanır
ve Linux'ta çalışır. Okuyucu ve bağlama kitaplığı [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) deposunda geliştirilmektedir.

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

İkinci bir terminalde:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

Bağlamayı kaldırmak için ilk terminalde Ctrl+C tuşlarına basın veya `fusermount3 -u /mnt/host1`
komutunu çalıştırın.

## Bağlamanın gösterdikleri { #what-the-mount-shows }

- Bağlama noktasında NTFS biriminin dosyaları: bir KAPE koleksiyonu için toplanan dosyaları barındıran
  `C` klasörü, KAPE'nin `LongFileNames` klasörü ve birimin `$MFT` gibi kendi meta veri dosyaları. Birden
  fazla NTFS birimi içeren bir imaj, bunları `Volume1`, `Volume2` vb. olarak gösterir.
- Her dosya salt okunurdur ve bağlama noktasının sahibine aittir. Dosya zamanları, NTFS
  `$STANDARD_INFORMATION` zamanlarıdır.
- Adlar NTFS'in sakladığı şekliyle listelenir ve birebir eşleşmeyle aranır. DOS 8.3 adları ve
  alternatif veri akışları gösterilmez.
- Sıkıştırılmış dosyaların sıkıştırması açılır (her biri için en fazla 1 GiB). EFS ile şifrelenmiş
  dosyalar okunamaz.

## Kanıt değiştirilmez { #the-evidence-is-not-changed }

İmaj salt okunur olarak açılır ve bağlama, yazma işlemlerini reddeder. Düzgün kapatılmamış bir VHDX,
bekleyen günlüğü bellekte uygulanarak okunur ve imaja asla geri yazılmaz. KAPE, VHDX dosyalarını
DiscUtils ile yazar; DiscUtils, NTFS önyükleme sektörünü atlama komutu ve `55 AA` imzası olmadan
bırakır. Juntei bu birimleri oldukları gibi okur.

## Kullanım { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## auto-parse ile her imajı ayrıştırma { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx`, ayrıca bağlama gerektirmeden, kanıt dizinindeki her VHDX disk imajının (veya
`-d` ile verilen tek imajın) içindeki Windows sistemlerini ayrıştırır:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- Her imaj, yeni bir geçici dizinde, imajın adını taşıyan bir klasöre (`host1-triage`) salt okunur
  olarak bağlanır ve orada bulunan sistemler, diskteki bir koleksiyonda olduğu gibi ayrıştırılır.
  İmajların bağlantısı, çalıştırmanın sonunda ve Ctrl+C ile kaldırılır.
- Terminal, imajları ve her sistemin imajını listeler:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- Bildirim dosyasında `EvidenceDirectory` verilen dizindir ve her sistemin `Image` değeri o sistemin
  imajını belirtir. `EvidenceRoot` ve ayrıştırıcı komutları bağlama klasörünü (`./host1-triage/C`)
  kullanır.
- KAPE, Volume Shadow Copies'ten kopyaladığı dosyaları `VSS32` gibi klasörlerde saklar. Bunların her
  biri bulunur ve ayrı bir sistem (`HOST1_2`, …) olarak ayrıştırılır.
- Bağlanamayan bir imaj raporlanır; diğerleri yine de ayrıştırılır ve çalıştırma 1 çıkış durumuyla
  sona erer.

Bir NTFS birimi açılamazsa Juntei sorunu bildirir ve okunabilen birimleri korur. Kısmen okunabilen imajlarda `auto-parse --vhdx` çıkış kodu 1 döndürür; `mount` da ayırma işleminden sonra 1 döndürür.

## Gereksinimler { #requirements }

- Linux: FUSE 3 (`fusermount3`, `fuse3` paketi) ve normal kullanıcıların genellikle sahip olduğu
  `/dev/fuse` erişimi. Root gerekmez.
- Windows ve macOS desteklenmez. Windows'ta bunun yerine imajı yönetici olarak çalıştırılan bir
  PowerShell'de salt okunur olarak ekleyin ve araçlara sürücü harfini verin:

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
