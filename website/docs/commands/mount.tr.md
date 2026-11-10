# Disk İmajlarını Bağlama { #mounting-disk-images }

`juntei mount`, çalıştığı sürece VHDX / E01 / Ex01 disk imajındaki dosyaları bir dizinde salt okunur olarak sunar. Hayabusa ve Juntei gibi araçlar dosyaları çıkarmadan veya root yetkisi gerektirmeden inceleyebilir. FUSE kullanır ve Linux'ta çalışır. Okuyucu ve bağlama kitaplığı [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) deposunda geliştirilmektedir.

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
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## İmajları yerel olarak analiz etme { #parsing-every-image-with-auto-parse }

`--vhdx` ile `auto-parse`, [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) üzerinden **Linux, macOS ve Windows** üzerinde VHDX’i yerel olarak okur. OS bağlaması, FUSE veya yönetici yetkisi gerekmez.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Ayrıştırıcılar dosya yolu gerektirir; Juntei sunulan NTFS dosyalarını sınırlı tamponlarla özel bir geçici dizine kopyalar. **Yalnızca sonuç üreten eserler değil, tüm bu dosyalar için alan gerekir.** Destekleniyorsa sıfır alanlar seyrek yazılır. Adlar, dizin yapısı, sabit bağlantılar ve değiştirme zamanları korunur. LNK ve Jump List kaynak zamanları geçici dosyalardan değil özgün NTFS metadata anlık görüntüsünden alınır. Ayrıştırıcı hatalarıyla da normal bitiş kopyaları siler; zorla sonlandırma dosya bırakabilir. Kaynak salt okunurdur; bekleyen VHDX günlükleri yalnızca bellekte uygulanır.

Bilgisayar keşfi, CSV/JSON/JSONL ve `-j` normal işlem hattını kullanır. `manifest.json` içindeki `Image` özgün imajı, `EvidenceRoot` iç göreli yolu tanımlar. Okuma hataları ve güvensiz adlar `ImageErrors` ve `manifest.csv` içindeki başarısız satırlara yazılır. Okunabilir sonuçlar korunur; eksik girdiler 1 döndürür. Güvensiz veya belirsiz adlar yeniden adlandırılmaz. Adlandırılmış NTFS akışları ek dosya olarak çıkarılmaz; normal dosya olarak toplanmış eserler kullanılabilir.

Ayrı [`mount` komutu](mount.md) dosya sistemini başka programlara sunar ve Linux FUSE gerektirir. `auto-parse --vhdx` bunu kullanmaz.

Hazırlama alanı varsayılan olarak çıktı dosya sistemindeki özel bir dizini kullanır. Başka bir diskteki mevcut dizini seçmek için `--temp-dir` kullanın. Özgün meta veriler çıktı dosya sisteminde ayrı tutulur; böylece hazırlama diski dolsa da okunabilen dosyalar ayrıştırılabilir. Hem kopyalar hem sonuçlar için yer ayırın; çıktı dosya sistemi doluysa sonuçlar kaydedilemez. Ctrl+C ayrıştırıcı süreçlerini durdurur, özel çalışma alanını temizler ve 130 koduyla çıkar. SIGKILL veya güç kesintisi kopya bırakabilir. İmaj ararken okunamayan alt dizinler atlanır; okunamayan giriş kökü hata olmaya devam eder.

Hazırlama dizini çıktı dizininin dışında olmalıdır. Linux ve macOS üzerinde SIGTERM ve SIGHUP da ayrıştırıcı süreçlerini durdurur ve çalışma alanını temizler.

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## E01 ve Ex01 disk imajları { #e01-and-ex01-disk-images }

`--e01`, fiziksel EWF1 (`.E01`) ve EWF2 (`.Ex01`) imajlarını **Linux, macOS ve Windows** üzerinde yerel olarak okur; FUSE, işletim sistemi bağlaması veya yönetici yetkisi gerekmez. [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) ve yukarıdaki NTFS ayrıştırma hattını kullanır.

Bölünmüş imajın ilk parçasını belirtin. `.E02` dahil tüm parçaları özgün adlarıyla aynı dizinde tutun. Tam diskler (MBR/GPT) ve tek NTFS birimleri desteklenir. Karışık dizinler için `--vhdx --e01` kullanın.

Aynı geçici alan ve kaynak meta verisi gereksinimleri geçerlidir. İmajlar salt okunurdur. Edinim boşlukları ve açma veya okuma sırasında bulunan hasarlar `ImageErrors` ve `manifest.csv` içinde bildirilir; okunabilen sonuçlar korunur ve çıkış kodu 1 olur. Yalnızca erişilen veriler denetlenir; tüm parçalar veya medyanın tam özeti doğrulanmaz. Mantıksal `.L01` ve `.Lx01` dosyaları reddedilir; parola seçeneği yoktur.

Ayrı `mount` komutu hâlâ Linux FUSE gerektirir; yerel otomatik ayrıştırma gerektirmez.

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## OS tarafından görülen bağlama gereksinimleri { #os-visible-mount-requirements }

`juntei mount`, Linux FUSE 3 (`fusermount3`) ve `/dev/fuse` erişimi gerektirir. Windows ve macOS’ta yukarıdaki yerel analizi kullanın.
