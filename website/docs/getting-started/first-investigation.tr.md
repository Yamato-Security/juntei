# İlk İnceleme { #first-investigation }

Bu iş akışı, `evidence/` içinde çevrimdışı bir triyaj koleksiyonu aldığınızı varsayar.

## 1. Kaynağı koruyun { #1-preserve-the-source }

Doğrulanmış bir kopya üzerinde çalışın. Karmaları, toplama zamanını, saat dilimini, ana bilgisayarı, birimi
ve toplama aracını kaydedin. Juntei kanıtları okur, ancak yerleşik akış veya karantina ayıklama gibi
seçenekler yeni çıktı dosyaları oluşturur; bunları ayrı bir analiz dizinine yönlendirin.

## 2. Elinizdekilerin envanterini çıkarın { #2-inventory-what-you-have }

[Kanıt Toplama](collection.md) sayfasındaki yolları arayın. Eksik artefaktlar normaldir. Bir koleksiyonun
yalnızca bir kayıt defteri kovanı içerdiği için işlem günlüklerini de içerdiğini varsaymayın.

## 3. Önce özetleri çalıştırın { #3-run-summaries-first }

Birçok komut, çıktı dosyası istenmediğinde kısa bir özet yazdırır:

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

Kayıtlara güvenmeden önce hasar, kirli veritabanı ve uygulanmamış günlük uyarılarını okuyun.

Birden fazla bilgisayarı kapsayan bir koleksiyondaki desteklenen her artefaktı tek bir işlemde ayrıştırmak
için şunu kullanın:

```bash
juntei auto-parse -d evidence/ -o analysis/
```

Ana bilgisayar keşfi, paralellik, çıktı düzeni ve SIEM'e aktarım için birleştirilmiş JSONL hakkında bilgi
için [Toplanan Verilerin Otomatik Ayrıştırılması](../commands/auto-parse.md) sayfasına bakın.

## 4. Yapılandırılmış çıktıyı kaydedin { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

Betik yazma ve iç içe değerler için JSONL'yi, elektronik tablo veya zaman çizelgesi görüntüleyici için
CSV'yi tercih edin.

## 5. Güvenilir bir tanımlayıcıdan pivot yapın { #5-pivot-from-a-reliable-identifier }

Bir dosya adı, tam yol, SHA-1/SHA-256, kullanıcı SID'si, MFT girdi/sıra numarası, görev GUID'i, hizmet adı
veya zaman damgası penceresiyle başlayın. Büyük/küçük harfi yalnızca Windows'un büyük/küçük harf duyarsız
olarak ele aldığı alanlar için normalleştirin.

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. İlişkilendirin { #6-correlate }

[Artefaktlar Arası İnceleme Akışları](../resources/workflows.md) sayfasını kullanın. Pratik bir sıralama şöyledir:

1. Yürütme ve son çalıştırma zamanları için Prefetch.
2. Oluşturma, yollar, yeniden adlandırmalar ve silme için `$MFT` ve USN Journal.
3. Geçmişteki varlık ve meta veriler için Amcache ve ShimCache.
4. Kullanıcı/uygulama bağlamı için LNK, Jump Lists, RecentDocs ve SRUM.
5. Kalıcılık için kayıt defteri hizmetleri ve TaskCache.
6. Tespit, kaldırma, köken ve karmalar için Defender karantinası.

## 7. Yeniden üretilebilirliği belgeleyin { #7-document-reproducibility }

Kaynak yolunu, kaynak karmasını, Juntei sürümünü, tam komutu, çıktı karmasını, sistem saat dilimini ve sonraki
her türlü filtrelemeyi kaydedin. Üretilen kanıt çıktısını asla sessizce düzenlemeyin.
