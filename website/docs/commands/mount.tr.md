# Disk İmajlarını Bağlama { #mounting-disk-images }

`juntei mount`, çalıştığı sürece VHDX disk imajındaki dosyaları bir dizinde salt okunur olarak sunar. Hayabusa ve Juntei gibi araçlar dosyaları çıkarmadan veya root yetkisi gerektirmeden inceleyebilir. FUSE kullanır ve Linux'ta çalışır. Okuyucu ve bağlama kitaplığı [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) deposunda geliştirilmektedir.

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

- Bağlama noktasında NTFS biriminin dosyaları görünür. Toplanmış bir imajda `C` ve `LongFileNames` gibi dizinler ile `$MFT` gibi birim meta verileri bulunabilir. Birden çok NTFS birimi `Volume1`, `Volume2` vb. olarak gösterilir.
- Her dosya salt okunurdur ve bağlama noktasının sahibine aittir. Dosya zamanları, NTFS
  `$STANDARD_INFORMATION` zamanlarıdır.
- Adlar NTFS'in sakladığı şekliyle listelenir ve birebir eşleşmeyle aranır. DOS 8.3 adları ve
  alternatif veri akışları gösterilmez.
- Sıkıştırılmış dosyaların sıkıştırması açılır (her biri için en fazla 1 GiB). EFS ile şifrelenmiş
  dosyalar okunamaz.

## Kanıt değiştirilmez { #the-evidence-is-not-changed }

İmaj salt okunur açılır ve bağlama yazma işlemlerini reddeder. Düzgün kapatılmamış bir VHDX'in bekleyen günlüğü bellekte uygulanır, imaja geri yazılmaz. Bazı DiscUtils yazıcıları NTFS önyükleme sektörünün atlama komutunu ve `55 AA` imzasını sıfır bırakır; Juntei bu birimleri olduğu gibi okur.

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

## İmajları yerel olarak analiz etme { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx`, VHDX imajlarını **Linux, macOS ve Windows** üzerinde yerel olarak okur; işletim sistemi bağlaması, FUSE veya yönetici yetkisi gerekmez.

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Ayrıştırıcılar dosya yolu gerektirdiğinden Juntei, NTFS okuyucusunun sunduğu dosyaları özel bir geçici dizine kopyalar. **Bu dosyalar için yeterli geçici disk alanı ayırın.** Kaynak imaj salt okunur kalır; normal tamamlanmada geçici kopyalar silinir.

Okuma hataları ve güvenli olmayan adlar `ImageErrors` ve `manifest.csv` içindeki başarısız satırlara kaydedilir. Okunabilen sonuçlar korunur; eksik girdiler çıkış kodu 1 döndürür.

Dosyaları diğer programlara sunan ayrı `mount` komutu için hâlâ Linux, FUSE 3 ve `/dev/fuse` erişimi gerekir.

[English: implementation details and limitations](https://yamato-security.github.io/juntei/commands/auto-parse/#vhdx-disk-images).
