---
hide:
  - navigation
  - toc
---

<div class="jt-hero" markdown>

![Juntei, Dijital Adli Bilişim Tanrıçası](assets/juntei-goddess.png){ .jt-hero-art width="1448" height="1086" }

<p class="tagline">
<strong>Juntei</strong>, Yamato Security'nin olay günlüğü <em>olmayan</em> Windows adli artefaktları için
geliştirdiği platformlar arası ayrıştırıcıdır. Toplanan kanıtları tutarlı CSV, JSON ve JSONL
kayıtlarına dönüştürürken bir inceleyicinin yanlış sonuçlara varmaktan kaçınması için gereken bağlamı korur.
</p>

<div class="jt-cta" markdown>
[Buradan başlayın :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Artefaktları öğrenin :material-archive-search:](artifacts/index.md){ .md-button }
[Komut başvurusu :material-console:](commands/index.md){ .md-button }
[GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/juntei){ .md-button }
</div>

</div>

## Ham artefakttan savunulabilir sonuca { #from-raw-artifact-to-defensible-conclusion }

Juntei bir artefaktın neyi kaydettiğini size söyleyebilir; bu site ise o kaydın **neyi kanıtladığını ve neyi
kanıtlamadığını** açıklar. Daha önce hiç Windows DFIR çalışması yapmamış olsanız bile bir artefakt sayfasıyla başlayın.
Her kılavuz; amacını, diskteki konumunu, saklama ve toplamaya ilişkin uyarıları, ayrıştırma komutunu,
çıktı alanlarını, yorumlamayı, ilişkilendirmeleri ve ileri okumaları kapsar.

<div class="grid cards" markdown>

-   :material-file-tree:{ .lg .middle } __Dosya sistemi geçmişi__

    ---

    Dosyaların varlığını, adlarını, taşınmasını, silinmesini, meta veri değişikliklerini ve alternatif
    veri akışlarını yeniden oluşturmak için `$MFT` ve USN Journal'ı kullanın.

-   :material-play-circle-outline:{ .lg .middle } __Program kanıtları__

    ---

    Prefetch yürütmeyi güçlü biçimde destekleyebilir. Amcache ve ShimCache envanter, yol, karma,
    uyumluluk ve geçmiş bağlamı ekler; ancak dikkatli yorumlama gerektirir.

-   :material-account-clock:{ .lg .middle } __Kullanıcı etkinliği__

    ---

    LNK dosyaları, Jump Lists, RecentDocs ve SRUM; açılan öğeleri, uygulama etkinliğini,
    çıkarılabilir veya ağ konumlarını, kaynak kullanımını ve geçmişe dair kullanıcı bağlamını ortaya çıkarır.

-   :material-shield-bug:{ .lg .middle } __Güvenlik kanıtları__

    ---

    Defender karantinası tehdit etiketlerini, özgün yolları, indirme kaynaklarını, karmaları ve
    sahipleri koruyabilir; hatta karantinaya alınmış baytları yalıtılmış analiz için güvenli biçimde kurtarabilir.

-   :material-database-search:{ .lg .middle } __Kayıt defteri durumu__

    ---

    Ayrılmış ve kurtarılabilir kayıt defteri verilerini okuyun, işlem günlüklerini yeniden oynatın ve hizmetleri,
    zamanlanmış görevleri, yerel hesapları ve son belgelerin durumunu inceleyin.

-   :material-link-variant:{ .lg .middle } __Tahmin etmeyin, ilişkilendirin__

    ---

    Hiçbir artefakt tek başına hikâyenin tamamını anlatmaz. Kılavuzlar, bağımsız kaynakları kanıta dayalı
    bir zaman çizelgesinde nasıl birleştireceğinizi gösterir.

</div>

!!! warning "Adli çıktı bir hüküm değil, kanıttır"
    Şüpheli görünen bir yol bir ipucudur. Bir zaman damgası yürütmeyi değil, bir dosyanın meta verilerini
    açıklıyor olabilir. Bir kaydın bulunmaması, bir eylemin gerçekleşmediğini kanıtlamaz. Kaynak kanıtı koruyun,
    araç sürümlerini ve komutları kaydedin ve önemli sonuçları doğrulayın.
