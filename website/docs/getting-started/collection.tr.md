# Kanıt Toplama { #collecting-evidence }

Juntei, Windows'tan önceden toplanmış dosyaları ayrıştırır. Güvenilir bir edinim aracı kullanın ve mümkün olduğunda özgün meta verileri koruyun.

[auto-parse --vhdx](../commands/auto-parse.md#vhdx-disk-images): Linux / macOS / Windows.

[auto-parse --e01](../commands/auto-parse.md#e01-and-ex01-disk-images): Linux / macOS / Windows.

## Yaygın konumlar { #common-locations }

| Artefakt | Tipik konum |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | NTFS birimi meta veri dosyası `C:\$MFT` |
| `$LogFile` | NTFS birimi meta veri dosyası `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| USN Journal | NTFS meta veri akışı `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| Kullanıcı Erişim Günlüğü (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| Internet Explorer ve eski Edge (WebCache) | `%LOCALAPPDATA%\Microsoft\Windows\WebCache\` |
| Outlook | `%LOCALAPPDATA%\Microsoft\Outlook\*.ost`, `%USERPROFILE%\Documents\Outlook Files\*.pst` (Outlook 2010'dan önce `%LOCALAPPDATA%\Microsoft\Outlook\*.pst`), `%APPDATA%\Microsoft\Templates\*.oft` ve kaydedildikleri her yerdeki `.msg` dosyaları |
| Web tarayıcıları | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\` ve diğer tarayıcıların profil klasörleri |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` ve diğer kullanıcı/sistem konumları |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` ve `CustomDestinations` |
| ShimCache | `SYSTEM` kovanı: `...\Session Manager\AppCompatCache\AppCompatCache` |
| Defender karantinası | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| Makine kayıt defteri | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| Kullanıcı kayıt defteri | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## Birlikte toplanması gereken dosyalar { #files-that-must-travel-together }

- Kayıt defteri kovanlarını yanlarındaki `.LOG`, `.LOG1` ve `.LOG2` dosyalarıyla birlikte toplayın.
  Juntei, kovan kirli olduğunda kayıt defteri komutları ve ShimCache için günlükleri yeniden
  oynatabilir.
- SRUM'un `SRUDB.dat` dosyasını tutarlı bir şekilde toplayın. Juntei şu anda kirli bir ESE
  veritabanını raporlar, ancak `SRU*.log` dosyalarını yeniden oynatmaz; yalnızca bu günlüklerde
  bulunan kayıtlar kullanılamaz.
- Mümkün olduğunda `$MFT` dosyasını `$Extend/$J` ile birlikte toplayın. USN kaydı tam bir yol değil,
  üst dosya başvurularını saklar; MFT yol çözümlemesini iyileştirir.
- `$LogFile` dosyasını aynı birimin `$MFT` dosyasıyla zaman olarak mümkün olduğunca yakın toplayın.
  `$LogFile` hızla başa sarar; daha sonra yapılan toplama, önceki redo/undo kayıtlarını tamamen
  kaybedebilir.
- Hesap ve Wi-Fi profil adlarını çözümlemek için SRUM'u eşleşen `SOFTWARE` kovanıyla birlikte toplayın.
- Kullanıcı Erişim Günlüğü'nün `Sum` dizininin tamamını toplayın: `SystemIdentity.mdb` rolleri ve yıllık
  `{GUID}.mdb` veritabanlarını adlandırır. Juntei bunun `Svc*.log` işlem günlüklerini de yeniden oynatmaz.
- Tarayıcı veritabanlarını yanlarındaki `-wal` dosyalarıyla birlikte toplayın (`places.sqlite-wal`,
  `cookies.sqlite-wal`): en yeni değişiklikler ve silinmiş girdilerin kurtarıldığı eski sayfa sürümleri
  yalnızca tarayıcı bir denetim noktası (checkpoint) uygulayana kadar orada bulunur.
- `WebCache` klasörünün tamamını toplayın: Windows, kullanıcı oturum açtığı sürece `WebCacheV01.dat`
  dosyasını açık tutar ve en yeni değişiklikleri yalnızca `V01*.log` işlem günlüklerinde bulunur; Juntei
  bu günlükleri yeniden oynatmaz.
- Outlook'un OST ve PST dosyalarını Outlook kapalıyken ya da bir disk imajından veya bir birim gölge
  kopyasından toplayın: Outlook çalıştığı sürece bunları açık tutar. PST'leri varsayılan klasörlerin
  dışında da arayın: kullanıcılar arşivleri başka klasörlere, sürücülere ve dosya paylaşımlarına taşır.
- Entries, ResourceData ve Resources'ın eşleştirilebilmesi için Defender `Quarantine` ağacının
  tamamını toplayın.

## Canlı sistem uyarıları { #live-system-cautions }

Açık dosyalar kopyalama sırasında değişebilir. Kayıt defteri kovanları ve ESE veritabanları dahili
olarak kirli olabilir. ShimCache, bellekteki en yeni girdilerini kapatmaya kadar diske yazmayabilir.
Birim anlık görüntüleri veya ham erişim kullanan bir adli bilişim toplama aracı, sıradan bir dosya
kopyalamaya tercih edilir.

## Güvenlik { #safety }

Kanıtlar kötü amaçlı yazılımlar, istismar örnekleri, kötü amaçlı LNK dosyaları ve elektronik tablo
formülü metni içerebilir. Yalıtılmış bir sistemde analiz yapın, kurtarılan akışları çalıştırmayın ve
`--extract` seçeneğini yalnızca tehlikeli içerikle çalışmayı amaçladığınızda kullanın. Dokunulmamış
kaynağı ayrı olarak saklayın.
