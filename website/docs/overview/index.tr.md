# Juntei Hakkında { #about-juntei }

Juntei, bellek açısından güvenli Rust diliyle yazılmış bir Windows adli artefakt ayrıştırıcısıdır.
[Hayabusa](https://github.com/Yamato-Security/hayabusa)'yı tamamlar: Hayabusa Windows olay günlüklerini analiz ederken
Juntei kalıcı dosya sistemi, kayıt defteri, yürütme, kullanıcı etkinliği, kaynak kullanımı
ve güvenlik ürünü artefaktlarını ayrıştırır.

Aynı ikili dosya Windows, macOS ve Linux üzerinde çalışır. Bu, bir analist KAPE,
Velociraptor, disk imajı veya başka bir çevrimdışı toplama aldığında ve her artefakt için ayrı bir program
yerine tek ve tutarlı bir arayüz istediğinde faydalıdır.

## Tasarım hedefleri { #design-goals }

- **Savunulabilir ayrıştırma:** Ham değerleri koruyun ve sessizce veri uydurmak yerine hasarı bildirin.
- **Herkes için faydalı:** Varsayılan kullanım yeni başlayanlar için kolay olmalı, ancak deneyimli kullanıcılar için de
  özelleştirilebilir ve eksiksiz özelliklere sahip olmalıdır.
- **Bağımsız ikili dosya:** Çalışmak için herhangi bir çalışma zamanı kitaplığı gerektirmez. Kutudan çıktığı gibi
  çalışır.
- **Son derece hızlı:** Büyük miktarda bellek gerektirmeden en hızlı ayrıştırıcı olmayı
  hedefler.
- **Platformlar arası kullanım:** Windows kanıtlarını bir Windows iş istasyonu gerektirmeden analiz edin.
- **Otomasyon:** Elektronik tablolar ve zaman çizelgesi araçları için CSV; `jq`, betikler ve veri gölleri için JSON/JSONL.
- **Güvenli hata yönetimi:** Bozuk biçimli kayıtlar bildirilir ve mümkün olduğunda atlanır.
- **Şeffaf köken:** Juntei, her bir uygulamanın arkasındaki açık kaynak ayrıştırıcıları ve başvuru araçlarını belirtir.

## Juntei'nin yapmadıkları { #what-juntei-does-not-do }

Juntei canlı bir diskin imajını almaz, adli bir imajı bağlamaz veya kanıt korumanın yerini almaz.
Tüm artefaktları otomatik olarak tek bir olay anlatısında birleştirmez. Sağladığınız
kanıtları ayrıştırır ve inceleme ile ilişkilendirmeye uygun kayıtlar üretir.

Juntei analizi sizin yerinize de yapmaz. Kanıtlar üzerinde analizi gerçekleştirmek için bir DFIR uzmanı gereklidir.

## Lisans { #license }

Juntei,
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) lisansı altında yayımlanmıştır ve telif hakkı Shirofune Security Co., Ltd.'ye aittir. Kaynak kodu 1.0.0 sürümünden önce
AGPLv3 lisansı altında yayımlanmıştı. Derlenmesinde kullanılan crate'lerin ve diğer üçüncü taraf kod ve verilerin
lisansları
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES) içindedir.
Juntei'nin üzerine inşa edildiği ve karşılaştırılarak doğrulandığı projeler
[Teşekkürler](../acknowledgements.md) sayfasında listelenmiştir.
