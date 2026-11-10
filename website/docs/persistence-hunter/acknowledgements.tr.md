# Persistence Hunter Teşekkürleri { #persistence-hunter-acknowledgements }

Juntei'nin `persistence-hunter` komutu, topluluğun iki nesil kalıcılık avcılığı çalışmasına dayanır.
Tekniklerini, uygulamalarını ve araştırmalarını incelemecilerin kullanımına sunan yazarlara ve
katkıda bulunanlara teşekkür ederiz.

## PyrsistenceSniper: Python projesi { #pyrsistencesniper-the-python-project }

[Hexastrike](https://github.com/Hexastrike) tarafından geliştirilen
[**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper), bu özelliğin doğrudan
referansıdır. Juntei, onun 130 denetimlik kataloğunu varsayılan izin listesi, LOLBAS anlık görüntüsü ve
sınıflandırma davranışıyla birlikte yerel Rust'a taşır. Referans sürüm
[`b99c117` commit'ine](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f)
sabitlenmiştir.

Python projesi MIT lisanslıdır. Juntei, telif hakkı ve lisans bildirimini
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt)
dosyasında korur. Juntei'nin uygulaması bir Python yorumlayıcısı olmadan çevrimdışı çalışır.
Koleksiyon işleme yöntemi, kapsam raporlaması ve eksik kanıtların ihtiyatlı yorumlanması
[komut kılavuzunda](../commands/persistence-hunter.md) belgelenmiştir.

## PersistenceSniper: özgün PowerShell aracı { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper), PyrsistenceSniper'a ilham
veren özgün PowerShell kalıcılık avcılığı projesidir. PyrsistenceSniper, Federico Lagrasta'yı
anar; özgün depo ayrıca Riccardo Ancarani'yi ve diğer katkıda bulunanları anar. Windows kalıcılık
mekanizmalarının incelenmesini kolaylaştırdıkları için özgün projeye ve katkıda bulunanlarına teşekkür
ederiz.

Juntei'nin doğrudan taşıma referansı Python projesidir. Özgün PowerShell aracı bu soy ağacı nedeniyle
anılmaktadır. Juntei ayrıca [hongson11698](https://github.com/hongson11698) tarafından özgün projenin
[zamanlanmış görev COM çözümlemesi PR #31](https://github.com/last-byte/PersistenceSniper/pull/31)
ve [Başlat Menüsü kitaplığı/CLSID klasörü PR #32](https://github.com/last-byte/PersistenceSniper/pull/32)
isteklerinde önerilen mekanizmaları bağımsız olarak uygular.
Bunlar Python referans kataloğuna eklemelerdir; bu, iki üst kaynak projedeki her canlı sistem
özelliğinin Juntei'nin çevrimdışı komutu tarafından uygulandığı anlamına gelmez.

## Destekleyici araştırmalar ve araçlar { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/), katalogda yer alan teknik tanımlayıcılarını sağlar. Bir
  teknik eşlemesi, bir mekanizmanın olası bir kullanımını tanımlar; eşleşen bir kayıt defteri değeri
  veya dosya tek başına ATT&CK ile doğrulanmış bir saldırı değildir.
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns),
  PyrsistenceSniper tarafından da Windows otomatik başlatmalarını incelemek için bir ilham kaynağı
  olarak anılmaktadır.
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS), kötüye kullanılabilen meşru Windows ikili
  dosyalarını belgeler. Bu ikili dosyalardan birini tanımak bir komutu yorumlamaya yardımcı olur,
  ancak o komutu otomatik olarak kötü amaçlı yapmaz.

Juntei genelinde kullanılan kayıt defteri, NTFS, Shell Link, olay günlüğü ve diğer ayrıştırıcı
temelleri için [proje genelindeki teşekkürler](../acknowledgements.md) sayfasına bakın.

[Kalıcılık mekanizmaları kılavuzuna](index.md) geri dönün.
