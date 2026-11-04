# Ekran Görüntüleri { #screenshots }

Juntei'nin komut satırını, otomatik ayrıştırmasını ve düzenli inceleme sonuçlarını keşfedin.
Tam boyutta görüntülemek için bir ekran görüntüsüne tıklayın.

## 1. Başlangıç ve komutlar { #1-startup-and-commands }

<figure markdown>

[![CODE BLUE sürümünü ve kullanılabilir komutları gösteren Juntei başlangıç ekranı](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>Juntei'nin başlangıç ekranı otomatik ayrıştırmayı, kaynağa özgü ayrıştırıcıları ve Persistence Hunter'ı listeler.</figcaption>

</figure>

## 2. 20 saniyeden kısa sürede otomatik ayrıştırma { #2-automatic-parsing-in-under-20-seconds }

Bir **MacBook Pro M4** üzerinde aşağıda gösterilen çalıştırma, **altı sistemden** toplanan
**8,7 GiB Windows verisini** ayrıştırdı ve **20 saniyeden kısa sürede 4,4 GiB çıktı** kaydetti.
[auto-parse](commands/auto-parse.md) komutu 16 paralel işlem kullanarak 240 ayrıştırıcı görevi
çalıştırdı ve 482 sonuç dosyası üretti. **210 görev başarılı oldu, artefaktlar toplanmadığı için
30 görev atlandı ve hiç hata olmadı**.

<figure markdown>

[![Altı sistemi, 8,7 GiB girdiyi ve 4,4 GiB çıktıyı 20 saniyeden kısa sürede tamamlayan Juntei auto-parse](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>Otomatik ana bilgisayar keşfi, paralel ayrıştırma ve girdi boyutunu, çıktı boyutunu, görev sonuçlarını ve geçen süreyi gösteren bir tamamlanma özeti.</figcaption>

</figure>

## 3. İncelemeye göre düzenlenmiş sonuçlar { #3-results-organized-for-investigation }

<figure markdown>

[![Bir sistemin DFIR kategorilerine göre gruplandırılmış çıktı klasörleri, hesap CSV dosyaları ve bunların ayrıştırıcı günlükleri](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>Her bilgisayarın sonuçları DFIR kategorisine göre gruplandırılır. Artefakt CSV dosyaları ve bunlarla eşleşen <code>-logs.txt</code> dosyaları bir arada tutulur; böylece hem kayıtları hem de ayrıştırma ayrıntılarını bulmak kolaylaşır.</figcaption>

</figure>
