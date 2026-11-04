# Artefaktlar Arası İnceleme Akışları { #cross-artifact-workflows }

Hiçbir artefakt önemli bir sonucu tek başına taşımamalıdır. Bu iş akışları, bir ipucunun savunulabilir bir
kanıt zincirine nasıl dönüştürüleceğini gösterir.

## Şüpheli yürütülebilir dosya yürütmesi { #suspicious-executable-execution }

1. **Prefetch:** saklanan çalıştırma zamanını, yol/karma bağlamını, çalıştırma sayısını ve yüklenen dosyaları
   belirleyin.
2. **`$MFT`:** yolu, boyutu, zaman damgalarını, ADS'yi, silinme durumunu ve MFT referansını bulun.
3. **USN Journal:** oluşturma, yazma, yeniden adlandırma ve silme işlemlerini yeniden oluşturun.
4. **Amcache:** SHA-1, PE meta verileri, sürüm ve geçmişteki yolu ekleyin.
5. **ShimCache:** uyumluluk önbelleği yolunu/sırasını ve desteklenen yürütme bayrağını ekleyin.
6. **SRUM:** aynı yürütülebilir dosya/kullanıcı tarafından yapılan kaynak/ağ kullanımını arayın.
7. **Defender:** karantina zamanını, özgün yolu, karmayı ve indirme kaynağını kontrol edin.
8. **Olay günlükleri/EDR:** üst işlemi, komut satırını, hesabı, ağı ve güvenlik bağlamını belirleyin.

Sonuç; dosya varlığını, yürütmeyi ve bir kişiye veya saldırıya atfı birbirinden ayırmalıdır.

## Fidye yazılımı dosya etkinliği { #ransomware-file-activity }

1. `$MFT` içinde şifrelenmiş uzantıları ve kurban yollarını belirleyin.
2. Toplu yazma/yeniden adlandırma/silme işlemlerini yeniden oluşturmak için USN nedenlerini ve sırasını
   kullanın.
3. Aday şifreleyicileri ve çalıştırma zamanlarını belirlemek için Prefetch kullanın.
4. Aday işlemin CPU/disk/ağ kullanımındaki ani artışlar için SRUM kullanın.
5. Dağıtım/kalıcılık için hizmetleri ve TaskCache'i kontrol edin.
6. Tespit/düzeltme için Defender karantinasını ve olay günlüklerini kontrol edin.

MFT/USN, hiçbir yürütme artefaktı kalmadığında bile dosya sistemi üzerindeki etkiyi kanıtlayabilir. Ancak
işlem telemetrisi veya ilişkilendirme olmadan sorumlu işlemi genellikle belirlemezler.

## Kötü amaçlı belge / kısayol ile ilk erişim { #malicious-document-shortcut-initial-access }

1. RecentDocs ve Jump Lists, kullanıcı/uygulama etkileşimini ve MRU bağlamını belirler.
2. Hedef, birim/paylaşım, MFT referansı ve bağımsız değişkenler için eşleşen Recent klasörü LNK dosyasını
   ayrıştırın.
3. Kaynak URL'leri/bölgeleri için `$MFT` yerleşik ADS'sinden veya Defender içeriğinden Zone.Identifier
   kullanın.
4. Belge uygulaması ve alt işlemleri için Prefetch/olay günlüklerini kullanın.
5. Ayıklanan yükün oluşturulmasını ve silinmesini izlemek için USN/MFT kullanın.

## Hizmet kalıcılığı { #service-persistence }

1. Hizmet adının, `ImagePath`, `ServiceDll`, hesap, başlatma türü ve hata eylemlerinin kodunu çözün.
2. Ortam değişkenlerini ve tırnaklı/tırnaksız yol davranışını çözümleyin.
3. Yürütülebilir dosyayı/DLL'i `$MFT` ve USN geçmişinde bulun.
4. Gerçek hizmet/işlem başlatması için Prefetch ve olay günlüklerini kontrol edin.
5. Önceki yapılandırma için geçerli olmayan denetim kümelerini ve kurtarılan kayıt defteri kayıtlarını
   karşılaştırın.

## Zamanlanmış görev kalıcılığı { #scheduled-task-persistence }

1. TaskCache eylemlerinin, sorumlu/bağlam bilgisinin, kayıt/çalışma zamanı meta verilerinin ve gizli durumun
   kodunu çözün.
2. `C:\Windows\System32\Tasks` XML'i ile karşılaştırın.
3. Görev adını/GUID'ini Security ve Task Scheduler Operational olaylarıyla ilişkilendirin.
4. Yürütülebilir dosyayı/betiği bulun ve karmasını alın; Prefetch ve dosya sistemi geçmişini inceleyin.
5. Görev kaldırıldıysa silinmiş TaskCache kayıtlarını kurtarın.

## Şüpheli veri sızdırma { #suspected-exfiltration }

1. Uygulama/kullanıcı/profil tarafından gönderilen olağandışı bayt miktarlarını ve ölçüm dönemini belirlemek
   için SRUM kullanın.
2. Yürütülebilir dosyanın kimliğini Prefetch, Amcache ve `$MFT` ile doğrulayın.
3. Erişilen/açılan dosyaları Jump Lists, LNK, RecentDocs ve olay/EDR verileri aracılığıyla belirleyin.
4. Hedef/protokol için proxy, güvenlik duvarı, DNS, paket veya EDR telemetrisini kullanın.

SRUM'daki hacim bir veri sızdırma hipotezini destekler, ancak aktarılan içeriği belirlemez veya tek başına
kötü niyeti kanıtlamaz.

## Korunmaya değer ilişkilendirme anahtarları { #correlation-keys-worth-preserving }

| Anahtar | Artefaktlar |
|---|---|
| Tam yol / dosya adı | Neredeyse her artefakt |
| SHA-1 / SHA-256 | Amcache, Defender, kurtarılan dosyalar |
| MFT girdisi + sıra numarası | `$MFT`, Prefetch, LNK, Jump Lists, USN |
| USN | Amcache, `$MFT`, USN Journal |
| Birim seri numarası / droid'ler | Prefetch, LNK, Jump Lists, MFT nesne kimlikleri |
| SID / kullanıcı adı | SRUM, SAM, kayıt defteri, olay günlükleri |
| Görev GUID'i / görev adı | TaskCache, görev XML'i, olay günlükleri |
| Hizmet adı | Hizmetler, olay günlükleri, işlem/sürücü verileri |
| Defender EntryId / ScanId | Karantina, Defender geçmişi, olay günlükleri |
