# Artefakt Kılavuzu { #artifact-guide }

Windows; dosya sisteminde, kayıt defterinde, uyumluluk veritabanlarında, kullanıcı arabirimi geçmişinde,
kaynak kullanım muhasebesinde ve güvenlik ürünlerinde birbiriyle örtüşen izler saklar. Juntei bu
kaynakları ayrıştırır, ancak her kaynağın ne anlama geldiğini yine de inceleyicinin yorumlaması gerekir.

## Soruya göre artefakt seçin { #choose-an-artifact-by-question }

| İnceleme sorusu | Buradan başlayın | Şunlarla doğrulayın |
|---|---|---|
| Bu yürütülebilir dosya çalıştı mı? | [Prefetch](prefetch.md) | [ShimCache](shimcache.md), [SRUM](srum.md), olay günlükleri |
| Bu dosya hiç var oldu mu? | [Amcache](amcache.md), [$MFT](mft.md) | [USN Günlüğü](usn-journal.md), [$I30](i30.md), ShimCache |
| Bir dosya ne zaman oluşturuldu, yeniden adlandırıldı veya silindi? | [USN Günlüğü](usn-journal.md) | [$MFT](mft.md), [$I30](i30.md), Defender karantinası |
| Bir meta veri değişikliğinden önce hangi ad veya zaman damgası vardı? | [$LogFile](logfile.md) | [$MFT](mft.md), [USN Günlüğü](usn-journal.md), [$I30](i30.md) |
| Bir dosyanın sahibi kim ve onu kim değiştirebilir? | [$MFT](mft.md) ile birlikte [$Secure:$SDS](secure-sds.md) | [SAM](sam.md), Security olay günlükleri |
| Bir kısayolun veya Prefetch dosyasının hedefi hangi birimdeydi? | [$Boot](boot.md) | [LNK](lnk.md), [Prefetch](prefetch.md), [USB aygıtları](usb.md) |
| Bir kullanıcı neyi açtı? | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| Bir program büyük miktarda veri aktardı mı? | [SRUM](srum.md) | Güvenlik duvarı, proxy, DNS, paket veya EDR telemetrisi |
| Kalıcılık yapılandırıldı mı? | [Hizmetler](services.md), [TaskCache](taskcache.md), [Kayıt Defteri](registry.md) | Olay günlükleri, Prefetch, `$MFT` |
| Defender neyi kaldırdı? | [Defender Karantinası](defender-quarantine.md) | Olay günlükleri, `$MFT`, USN Journal |
| Yerel hesaplar eklendi veya değiştirildi mi? | [SAM](sam.md) | Security olay günlükleri ve kayıt defteri işlem geçmişi |

## Alanlardan önce uyarıları okuyun { #read-warnings-before-fields }

Her artefakt sayfası aynı sırayı izler:

1. **Nedir** ve hangi soruları yanıtlayabilir.
2. **Nerede bulunur** ve toplama onu nasıl etkiler.
3. **Neyi kanıtlar ve neyi kanıtlamaz.**
4. Juntei ile **nasıl ayrıştırılır**.
5. **Her çıktı alanı nasıl okunur.**
6. Bağımsız kanıtlarla **nasıl ilişkilendirilir**.
7. Düşük seviyeli araştırma için **Daha fazla ayrıntı**.

!!! danger "Bir ayrıştırıcı bulgusunu kötü amaçlı olmakla eşdeğer saymayın"
    Juntei bulguları, `System32` dışındaki bir sistem ikili dosyası adı gibi incelemeye değer kalıpları
    belirler. Yükleyiciler, yöneticiler, uyumluluk katmanları ve yazılım güncellemeleri olağandışı kayıtlar
    üretebilir. Bağlam ve bağımsız kanıtlarla doğrulayın.
