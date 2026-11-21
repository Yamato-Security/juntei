# Artefakt Kılavuzu { #artifact-guide }

Windows; dosya sisteminde, kayıt defterinde, uyumluluk veritabanlarında, kullanıcı arabirimi geçmişinde,
kaynak kullanım muhasebesinde ve güvenlik ürünlerinde birbiriyle örtüşen izler saklar. Juntei bu
kaynakları ayrıştırır, ancak her kaynağın ne anlama geldiğini yine de inceleyicinin yorumlaması gerekir.

## Soruya göre artefakt seçin { #choose-an-artifact-by-question }

| İnceleme sorusu | Buradan başlayın | Şunlarla doğrulayın |
|---|---|---|
| Hangi PowerShell komut satırları kaydedilmiş? | [PowerShell geçmişi](powershell-history.md) | PowerShell olay günlükleri, süreç telemetrisi, Prefetch |
| Bu yürütülebilir dosya çalıştı mı? | [Prefetch](prefetch.md) | [ShimCache](shimcache.md), [SRUM](srum.md), olay günlükleri |
| Bu dosya hiç var oldu mu? | [Amcache](amcache.md), [$MFT](mft.md) | [USN Günlüğü](usn-journal.md), [$I30](i30.md), [Windows Search Dizini](windows-search.md), ShimCache |
| Bu bilgisayarda hangi dosyalar, e-postalar ve sayfalar vardı ve bir belgede ne yazıyordu? | [Windows Search Dizini](windows-search.md) | [$MFT](mft.md), [USN Günlüğü](usn-journal.md), [Outlook](outlook.md), [Web Tarayıcıları](browser.md) |
| Bir dosya ne zaman oluşturuldu, yeniden adlandırıldı veya silindi? | [USN Günlüğü](usn-journal.md) | [$MFT](mft.md), [$I30](i30.md), Defender karantinası |
| Bir kullanıcı neyi ne zaman Geri Dönüşüm Kutusu'na taşıdı? | [Geri Dönüşüm Kutusu](recycle-bin.md) | [USN Günlüğü](usn-journal.md), [$MFT](mft.md), [LNK](lnk.md), [Jump Lists](jump-lists.md) |
| Bir meta veri değişikliğinden önce hangi ad veya zaman damgası vardı? | [$LogFile](logfile.md) | [$MFT](mft.md), [USN Günlüğü](usn-journal.md), [$I30](i30.md) |
| Bir dosyanın sahibi kim ve onu kim değiştirebilir? | [$MFT](mft.md) ile birlikte [$Secure:$SDS](secure-sds.md) | [SAM](sam.md), Security olay günlükleri |
| Bir kısayolun veya Prefetch dosyasının hedefi hangi birimdeydi? | [$Boot](boot.md) | [LNK](lnk.md), [Prefetch](prefetch.md), [USB aygıtları](usb.md) |
| Bir kullanıcı neyi açtı? | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| Bir program büyük miktarda veri aktardı mı? | [SRUM](srum.md) | Güvenlik duvarı, proxy, DNS, paket veya EDR telemetrisi |
| Bir kullanıcı hangi web sitelerini ziyaret etti ve neler indirdi? | [Web Tarayıcıları](browser.md), [Internet Explorer ve Eski Edge](webcache.md) | Proxy ve DNS günlükleri, `$MFT` ve USN Günlüğü, Defender, SRUM |
| Bir kullanıcı kimlerle e-posta alışverişinde bulundu ve iletilere neler eklenmişti? | [Outlook](outlook.md) | Posta sunucusu ve ağ geçidi günlükleri, `$MFT` ve USN Günlüğü, Defender, LNK ve Jump Lists |
| Bu sunucuyu hangi istemciler ve hesaplar, hangi günlerde kullandı? | [Kullanıcı Erişim Günlüğü (UAL)](ual.md) | Security olay günlükleri, SMB ve RDP günlükleri, güvenlik duvarı günlükleri |
| Bir kullanıcı Uzak Masaüstü ile ulaştığı bilgisayarlarda ne gördü? | [RDP Bitmap Önbelleği](rdp-cache.md) | [Uzaktan Erişim (Kullanıcı Tarafı)](remote-access.md), [Jump Lists](jump-lists.md), uzak bilgisayarların oturum açma olayları ve [UAL](ual.md) |
| Kalıcılık yapılandırıldı mı? | [Hizmetler](services.md), [TaskCache](taskcache.md), [Kayıt Defteri](registry.md) | Olay günlükleri, Prefetch, `$MFT` |
| Defender neyi tespit etti ve buna karşı ne yaptı? | [Defender Tespit Geçmişi](defender-history.md) | [Defender Karantinası](defender-quarantine.md), olay günlükleri |
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

!!! danger "Olağandışı kayıtları kötü amaçlı faaliyetin kanıtı saymayın"
    Juntei, analistin yorumlayacağı kanıtları ayrıştırır. Persistence Hunter denetimleri kullanılabilir olmaya devam eder; ancak yapılandırılmış bir başlangıç girdisi meşru yönetimin parçası da olabilir. Kötü amaçlı olduğuna karar vermeden önce gözlemleri bağlam ve bağımsız kanıtlarla ilişkilendirin.
