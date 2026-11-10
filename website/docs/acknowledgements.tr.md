# Teşekkürler { #acknowledgements }

Juntei, DFIR ve açık kaynak toplulukları ayrıştırıcılar, biçim araştırmaları, referans çıktılar,
spesifikasyonlar ve test kanıtları yayımladığı için var olabilmiştir. Aşağıdaki projeler uygulamaya,
doğrulamaya, alan adlandırmaya veya beklenen analist çıktısına yön vermiştir. Burada listelenmiş
olmak, bu projelerin kodunun Juntei'ye gömülü olduğu anlamına gelmez; lisansları için her projenin
kendisine başvurun.

## Yamato Security tarafından sürdürülen ayrıştırıcı temelleri { #parser-foundations-maintained-by-yamato-security }

| Juntei alanı | Sürdürülen ayrıştırıcı | Üst kaynak/temel |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`mft` crate'i) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| USN Journal | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`usn` crate'i) | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`logfile` crate'i) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| Dizin indeksleri (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`mft` crate'i, `i30` modülü) | Aşağıda teşekkür edilen index slack araştırmaları |
| NTFS `$Boot`, `$Secure:$SDS` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`boot` ve `secure` crate'leri) | Microsoft'un MS-DTYP belgesi ve aşağıda teşekkür edilen NTFS araştırmaları |
| SRUM, UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) ve [ese](https://github.com/Sootmark/ese) |
| Web tarayıcıları / SQLite | [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) | [Sootmark browser](https://github.com/Sootmark/browser), [sqlite](https://github.com/Sootmark/sqlite) ve [plist](https://github.com/Sootmark/plist) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Sootmark shell'in bileşik dosya ve Jump List çalışmaları |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| VHDX disk imajları (`mount`, `auto-parse --vhdx`) | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) ve [ntfs-core](https://github.com/SecurityRonin/ntfs-forensic), [fuser](https://github.com/cberner/fuser) ile sunulur | Microsoft'un MS-VHDX belgesi |
| Kayıt Defteri | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | Aşağıda teşekkür edilen kayıt defteri biçimi ve kurtarma araştırmaları |

## Referans araçlar ve bağımsız doğrulama { #reference-tools-and-independent-validation }

- Hexastrike tarafından geliştirilen [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper),
  yerel Rust `persistence-hunter` komutu için kalıcılık denetimi kataloğunu, varsayılan izin listesini,
  LOLBAS anlık görüntüsünü ve referans davranışı sağlamıştır. MIT lisansı ve telif hakkı bildirimi
  `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt` dosyasında korunmaktadır. Özgün
  [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) Python projesine ilham vermiştir;
  soy ağacının tamamı için [kalıcılık teşekkürleri](persistence-hunter/acknowledgements.md) sayfasına bakın.

- Maxim Suhanov'un **regf spesifikasyonu** ve **yarp** aracı; kayıt defteri yapılarına, işlem günlüğü
  yeniden oynatmaya ve silinmiş kayıt kurtarmaya yön vermiştir; **dfir_ntfs** aracı ise INDX kayıtları
  ve index slack oyma (carving) için referans olmuştur.
- Harel Segev'in **INDXRipper** aracı, index slack'ten dosya adlarını oymak için ek bir referanstır.
- **dfir_ntfs**, Joakim Schicht'in **LogFileParser** aracı ve Stroz Friedberg'in **ntfs-linker** aracı
  `$LogFile` doğrulamasına ve yorumlanmasına yön vermiştir; Velocidex'in **go-ntfs** projesi herkese açık
  NTFS test kanıtları sağlamıştır.
- Joachim Metz / libyal projeleri **libregf**, **libfsntfs**, **libscca**, **libfwsi** ve
  **libesedb**, ayrıntılı biçim araştırmaları ve bağımsız ayrıştırma referansları sunar.
- Fox-IT'in **dissect.esedb**, **dissect.ntfs** ve **dissect.target** projeleri, doğrulama sırasında
  kullanılan bağımsız SRUM, UAL, `$Secure:$SDS`, SAM ve Defender yorumlarını sağlamıştır.
- **The Sleuth Kit**'in `fsstat` aracı, `mkntfs` ile oluşturulan birimlerin NTFS önyükleme sektörleri
  için referans olmuştur.
- Eric Zimmerman'ın bazı araçları, karşılaştırmalarda kullanılan alan adlarını ve inceleyici
  beklentilerini belirlemiştir.
- **regipy**, **python-registry**, **Notatin** ve **artemis**, kayıt defteri uygulamaları, testler veya
  örnek yorumlar sağlamıştır.
- **LnkParse3**, AbdulRhmanAlfaifi'nin **lnk_parser** ve **jumplist_parser** araçları ve JLECmd,
  bağımsız LNK/Jump List davranışı ve herkese açık test senaryoları sağlamıştır.
- Security Ronin'in **prefetch-forensic** aracı ve **forensicnomicon** projesi, Juntei'nin yüksek
  değerli Prefetch/Amcache/ShimCache gözlemlerine yön vermiştir.
- **defender-dump** ve dissect.target, Defender karantina yapılarını ve güvenli çıkarma davranışını
  anlamak için ayrı uygulamalar sunar.
- **Plaso**, **Chainsaw** ve **Velociraptor**, çeşitli artefaktlar genelinde ayrıştırıcı
  karşılaştırmaları, test kanıtları veya toplama odaklı referans davranış sağlamıştır.
- Obsidian Forensics'in **Hindsight** aracı ve Velocidex'in **SQLiteHunter** tanımları, tarayıcı
  geçmişi, indirmeler, çerezler ve form geçmişi için referans olmuştur.

## Spesifikasyonlar ve araştırmalar { #specifications-and-research }

Shell Links, Compound Binary Files, NTFS/SAM protokolleri, kayıt defteri/hizmet/görev yapıları ve API
belgeleri için Open Specifications yayımladığı için Microsoft'a teşekkür ederiz. Bu artefaktları
uygulayıcılar için anlaşılır kılan AppCompatCache, Amcache, SRUM, Prefetch, kayıt defteri ve NTFS adli
bilişim araştırmalarının yazarlarına da teşekkür ederiz.

## Test kanıtları { #test-evidence }

Andrew Rathbun'un
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) projesi, NIST CFReDS, Plaso,
Regipy, Artemis, ayrıştırıcı test depoları ve uygun şekilde lisanslanmış diğer koleksiyonlardaki herkese
açık kanıtlar, gerçek Windows verilerine karşı regresyon testini mümkün kılar. Juntei, dahil edilen test
dosyalarının yanında atıf/lisans notlarını tutar.

## Katkıda bulunanlar { #contributors }

Her Juntei ve Yamato Security ayrıştırıcı katkıcısına, gözden geçirene, sorun bildirene ve uç durumlar
sağlayan her kullanıcıya teşekkür ederiz. Güncel katkıda bulunanlar GitHub tarafından
[Juntei katkıda bulunanlar sayfasında](https://github.com/Yamato-Security/juntei/graphs/contributors)
listelenir.

## Lisans { #license }

Juntei,
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) altında yayımlanmıştır ve telif hakkı Shirofune Security Co., Ltd.'ye aittir. Kaynak kodu
1.0.0 sürümünden önce AGPLv3 altında yayımlanmıştı. Derlenmesinde kullanılan crate'lerin ve diğer
üçüncü taraf kod ve verilerin lisansları
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES)
içinde yer alır.
