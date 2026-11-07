# Ayrıştırıcı Doğrulaması { #parser-validation }

Juntei'nin sürdürülen ayrıştırıcı crate'leri herkese açık kanıtlara ve bağımsız uygulamalara karşı test
edilir. Bu karşılaştırmalar gerilemeleri ve yorumlama farklarını tespit etmeye yardımcı olur; hiçbir
ayrıştırıcıyı yanılmaz kılmaz. Kaynak kanıtı koruyun, her kaydın sorun veya hasar alanlarını inceleyin
ve önemli sonuçları başka bir artefakt veya uygulamayla doğrulayın.

Aşağıdaki ifadeler, daha önce Juntei'nin ana README dosyasında tutulan doğrulama bilgilerini korur.
Güncel test verileri (fixture) ve uygulama ayrıntıları için bağlantı verilen ayrıştırıcı depolarını
izleyin.

## Amcache { #amcache }

Juntei, Security Ronin'in
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) projesinin Apache-2.0 lisanslı
bir çatalı olan [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) kullanır.
Onun [çıktı belgeleri](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
her yürütülebilir dosya, aygıt, bulgu ve özet alanını tanımlar.

## Ana Dosya Tablosu { #master-file-table }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) içindeki mft crate'i, Omer
Ben-Amram'ın [mft](https://github.com/omerbenamram/mft) crate'inin sürdürülen bir çatalıdır. Sekiz
`$MFT` dosyasında (crate'in örneği, NIST hacking vakası ve altı DFIR Artifact Museum dosyası)
1.048.733 CSV satırı ve sütunu bir referans ayrıştırıcıyla sırasıyla eşleşti. Karşılaştırmalar
varsayılan çıktıyı, `--all-times`, `--short-names` ve `--cutoff` seçeneklerini kapsadı; zaman damgası
gösterimi kasıtlı farktı.

## NTFS işlem günlüğü ($LogFile) { #ntfs-transaction-log-logfile }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) içindeki logfile crate'i,
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog) temel alınarak sürdürülmektedir. Doğrulaması
altı eksiksiz DFIR Artifact Museum günlüğünü, herkese açık 1.1 ve 2.0 sürüm test verilerini, kodu
çözülmüş redo/undo meta verilerini ve bağımsız olarak ayrıştırılmış NTFS verilerine karşı
karşılaştırılan günlük yazmalarını kapsar. Ayrıntılı kapsam ve bilinen farklar için ayrıştırıcı
deposuna bakın.

Juntei, herkese açık go-ntfs test biriminde (2.129 kayıt) her CSV alanını JSON ve JSONL ile
karşılaştıran, ayrı redo/undo verilerini ve ham yük uzunluklarını doğrulayan, hasarlı günlükleri test
eden, kanıtların üzerine yazmayı reddeden ve güncel MFT yollarının yeniden kullanılmış kayıt
numaralarını izlemesini önleyen CLI testleri ekler. Gerileme senaryoları, etkilenen MFT kayıtlarını
üzerinden yazdıkları açık öznitelikten ayırır, düz dizin girdisi/veri özniteliği meta verilerini
denetler ve birimler arası MFT eşleştirmesini önler. Çok birimli `auto-parse` testleri, ayrı CSV
dosyalarını ve köken bilgisi içeren tek bir JSONL dosyasını kapsar.

Komut ayrıca özel olarak toplanmış altı Windows günlüğünde denetlendi: dışa aktarılan 708.465 CSV
kaydının tümü JSONL ile alan alan eşleşti ve ana bilgisayar başına `auto-parse` dışa aktarımları aynı
kayıt sayılarını korudu. Kaynak kanıtlar depoyla birlikte dağıtılmaz. Bu denetimler, günlüğe
kaydedilen her işlemin tamamlanmış bir kullanıcı eylemini temsil ettiğini değil, çıktı tutarlılığını
doğrular.

## Prefetch { #prefetch }

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser), ForensicRS'in
[frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) projesinin sürdürülen bir çatalıdır.
Kodu çözülmüş çıktısı, libyal'ın bağımsız Prefetch uygulaması olan
[libscca](https://github.com/libyal/libscca) ile karşılaştırıldı.

## USN Günlüğü { #usn-journal }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) içindeki usn crate'i,
[Sootmark usn](https://github.com/Sootmark/usn) projesinin sürdürülen bir çatalıdır. Altı gerçek
günlükte Juntei'nin zamanı dikkate alan üst yolları, bağımsız bir ayrıştırıcının yol çözümlediği her
yerde onunla eşleşti. Juntei ayrıca karşılaştırmanın çözümlenmemiş bıraktığı 4.321 kaydı da çözümledi.
Oyma bileşeni, 880 MB `$MFT` verisine karşı test edildiğinde hiçbir sahte kayıt üretmedi.

## Dizin İndeksleri ($I30) { #directory-indexes-i30 }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) mft crate'inin `i30` modülü INDX
kayıtlarını ve slack alanlarını okur. ntfs-3g ile oluşturulmuş bir birimin iki `$I30` akışında
(Juntei'nin `test_files/i30/` dizini) ve Windows tarafından yazılmış herkese açık üç `$I30` örneğinde,
kullanımdaki ve slack'teki girdileri Maxim Suhanov'un [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs)
aracının girdileriyle aynı adlar ve zamanlarla eşleşti. dfir_ntfs'in 32 biti aşan bir üst dizin
başvurusu raporladığı yerlerde (üst dizin başvurusunun üzerine yazılmış baytlar), Juntei üst dizini
boş bırakır.

## Önyükleme Sektörü ($Boot) { #boot-sector-boot }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) içindeki boot crate'i Yamato
Security'nin kendi geliştirmesidir. DFIR Artifact Museum'un herkese açık beş `$Boot` dosyasında
(Windows 10, 11 ve Server 2012 R2) her alan bir referans ayrıştırıcıyla eşleşti. 512, 1.024 ve
4.096 baytlık sektörlere ve 4 KiB ile 64 KiB arası kümelere sahip `mkntfs` ile oluşturulmuş
birimlerde; boyutlar, `$MFT` ve `$MFTMirr` kümeleri ve seri numaraları The Sleuth Kit'in `fsstat`
aracıyla eşleşti. Museum'un Windows 11 `$Boot` dosyası, aynı ana bilgisayarın LNK dosyasının
kaydettiği birim seri numarasına sahiptir ve beş dosyanın önyükleme kodu aynıdır.

## Güvenlik Tanımlayıcıları ($Secure:$SDS) { #security-descriptors-securesds }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) içindeki secure crate'i Yamato
Security'nin kendi geliştirmesidir. Museum'un beş `$SDS` akışında (7.808 tanımlayıcı, 60.781 ACE)
güvenlik kimlikleri, karmalar, ofsetler, sahipler, gruplar, denetim bayrakları ve ACE sayıları bir
referans ayrıştırıcıyla eşleşti; her ACE'nin türü, bayrakları, maskesi, SID'i ve nesne türleri de
`dissect.ntfs` ile eşleşti. Kaydedilen her karma kendi tanımlayıcısına aittir, her ayna kopya kendi
girdisiyle eşleşir ve 3.384 geri çağırma (callback) ACE'sinin koşulları dahil her tanımlayıcının
SDDL'si vardır.

## SRUM { #srum }

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser), Sootmark'ın
[ese](https://github.com/Sootmark/ese) ve [srum](https://github.com/Sootmark/srum) crate'lerine
dayanır. 11 gerçek veritabanında, çıktıya yazılan 45.255 girdideki her sütun, `dissect.esedb`
tarafından sunulan değerlerin bağımsız bir kod çözümüyle eşleşti.

## Kullanıcı Erişim Günlüğü { #user-access-logging }

`parse-ese ual`, UAL veritabanlarını aynı ESE okuyucusuyla okur. 16 gerçek veritabanında (308 satır,
bunların 145'i istemci satırı), her değer `dissect.esedb` tarafından sunulan değerlerin bağımsız bir kod
çözümüyle eşleşti. Her istemci satırında, sayı bulunan günler `InsertDate` değerinin UTC gününden
`LastAccess` değerinin UTC gününe kadar uzandı ve toplamları `TotalAccesses` değerine eşit oldu.

## LNK kısayolları { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser), Sootmark'ın
[shell](https://github.com/Sootmark/shell) crate'inin sürdürülen bir çatalıdır. Herkese açık 1.118
kısayol üzerinde karşılaştırıldı. Referans ayrıştırıcıyla paylaşılan 30 CSV sütununda 16 tekil değer
farklıydı: hasarlı, kesik, yer tutucu, MTP veya Denetim Masası öğeleri içeren 13 hedef yolu; bilinmeyen
veya kesik bloklar içeren iki ek veri bloğu listesi; ve referans ayrıştırıcının farklı baytlardan
okuduğu bir MFT başvurusu. Juntei, hasarlı yapıları sessizce normalleştirmek yerine raporlar.

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser), Sootmark'ın bileşik
dosya ve Jump List çalışmalarını kullanır; gömülü bağlantıların kodu `ys-lnk-parser` ile çözülür.
Herkese açık 77 Jump List'te, 100 ns'ye kadar zaman damgaları dahil paylaşılan 13.057 karşılaştırma
hücresinin tümü referans ayrıştırıcıyla eşleşti. Referansın reddettiği altı dosya (boş özel listeler,
eksik bir altbilgi veya Jump List olmayan bir dosya) Juntei tarafından ya kodu çözüldü ya da açıkça
raporlandı.

## ShimCache { #shimcache }

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser), kayıt defteri kovanı
ve işlem günlüğü desteğiyle Security Ronin'in
[shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) projesinin sürdürülen bir
çatalıdır. Herkese açık 20 `SYSTEM` kovanında (24 denetim kümesi ve 7.290 girdi) ve 17 bağımsız
AppCompatCache değerinde, her girdi referans ayrıştırıcının ürettiği yol, zaman, yürütme bayrağı,
bayraklar ve konumla eşleşti.

## Microsoft Defender karantinası { #microsoft-defender-quarantine }

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser),
Sootmark'ın [defender](https://github.com/Sootmark/defender) crate'inin sürdürülen bir çatalıdır.
Ayrıştırıcının test araçları tarafından Defender biçiminde yazılan bir karantina klasörünün kodu,
bağımsız `dissect.target` karantina okuyucusu tarafından tutarlı biçimde çözülür.

## Kayıt defterinden türetilen artefaktlar { #registry-derived-artifacts }

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser); ayrılmış anahtarlar,
değerler, kirli kovan işlem yeniden oynatması, boş hücre kurtarma, hücre slack'i, kovan bin'lerinin
ötesindeki veriler ve yeniden oynatmayla kaldırılan veya değiştirilen kayıtlar için
[yarp](https://github.com/msuhanov/yarp) ile karşılaştırılır. Artefakta özgü karşılaştırmalar
şunları içerir:

- Windows 7 ve kirli Windows 11 `SYSTEM` kovanlarındaki hizmetler, değerler ve hata eylemleri;
- herkese açık dokuz `NTUSER.DAT` kovanında yarp ve libfwsi ile kodu çözülen RecentDocs;
- Windows 7–11, Server 2012 R2 ve Server 2022'yi kapsayan herkese açık yedi `SOFTWARE` kovanında yarp
  ile bağımsız olarak kodu çözülen zamanlanmış görevler; ve
- herkese açık yedi `SAM` kovanında `dissect.target` ile karşılaştırılan SAM hesap yapıları.

Referans araçların, biçim araştırmalarının, spesifikasyonların ve herkese açık kanıt kaynaklarının
tam listesi için [Teşekkürler](../acknowledgements.md) sayfasına bakın.
