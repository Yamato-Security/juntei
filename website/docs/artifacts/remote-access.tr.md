# Uzaktan Erişim (Kullanıcı Tarafı) { #remote-access-user-side }

## Nedir { #what-it-is }

Bir kullanıcının `NTUSER.DAT` dosyası, kullanıcının bilgisayardan yaptığı uzak bağlantıların izlerini tutar.
Bunlar yanal hareketin kaynak tarafıdır:

- **Uzak Masaüstü (`mstsc`):**
  - istemciye yazılan bilgisayarlar, en sonuncusu ilk sırada;
  - kullanıcının oturum açtığı her bilgisayar için, kullanılan hesap adını ve kullanıcının kabul ettiği
    sunucu sertifikasının parmak izini içeren bir anahtar;
  - kullanıcının yerel aygıtları kullanmasına izin verdiği bilgisayarlar.
- **PuTTY:**
  - kaydedilmiş oturumlar: ana bilgisayar, bağlantı noktası, hesap, protokol, anahtar dosyası, proxy ve
    bağlantı noktası yönlendirmeleri;
  - en son başlatılan oturumlar;
  - kullanıcının kabul ettiği her SSH sunucu anahtarı.
- **WinSCP:** PuTTY'dekiyle aynı biçimde kaydedilmiş oturumlar ve kabul edilmiş SSH sunucu anahtarları.
- **Eşlenmiş sürücüler:** oturum açıldığında geri yüklenmek üzere eşlenmiş her sürücü ve
  *Map Network Drive* iletişim kutusuna yazılan yollar.
- **Paylaşımlar:** Explorer'ın bağladığı paylaşımlar (`MountPoints2`).

Bunlar, uzak bilgisayarın kendi günlükleri yok olduktan sonra bile bir hesabın daha sonra nereye gittiğini
ve hangi kimlik bilgilerini kullandığını gösterir.

<span class="evidence-strength">En iyi sonuç:</span> bu kullanıcının profili bu hedefe bağlanmış veya bu
hedefe bir bağlantı kaydetmiştir. Kaydedilmiş bir oturum veya yazılmış bir ad tek başına bir bağlantının
başarılı olduğunu kanıtlamaz. Kabul edilmiş bir ana bilgisayar anahtarı veya `UsernameHint` içeren bir
`Servers` anahtarı daha güçlüdür: istemci sunucuya ulaşmıştır.

## Konum { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Terminal Server Client\Default            MRU0 … MRU9
Software\Microsoft\Terminal Server Client\Servers\<host>     UsernameHint, CertHash
Software\Microsoft\Terminal Server Client\LocalDevices       <host> = devices
Software\SimonTatham\PuTTY\Sessions\<session>
Software\SimonTatham\PuTTY\Jumplist                          Recent sessions
Software\SimonTatham\PuTTY\SshHostKeys                       <type>@<port>:<host> = key
Software\Martin Prikryl\WinSCP 2\Sessions\<session>
Software\Martin Prikryl\WinSCP 2\SshHostKeys
Network\<letter>                                             RemotePath, UserName, ProviderName
Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU
Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##server#share
```

Her profil için `NTUSER.DAT` dosyasını işlem günlükleriyle birlikte toplayın. Sistemin `SOFTWARE`
kovanını da toplayın: her kullanıcının SID'sini o verir.

## Analist için uyarılar { #analyst-cautions }

- **Anahtar zamanları liste zamanlarıdır.** `Default`, `SshHostKeys` ve `Map Network Drive MRU` listeler
  tutar ve anahtarlarının son yazılma zamanı, listenin en son değiştiği zamandır. Bu zaman ilk öğeye
  (`MruPosition` 0) aittir. Diğerleri için yalnızca bir üst sınırdır. Bir `Servers\<host>`, oturum,
  `Network\<letter>` veya paylaşım anahtarının kendi zamanı vardır.
- **Kaydedilmiş olmak kullanılmış olmak değildir.** Bir PuTTY veya WinSCP oturumu hiç bağlanmadan
  kaydedilebilir. PuTTY, kaydedilmiş bir oturum başlatıldığında `Recent sessions` değerini yeniden yazar ve
  bir sunucuya ilk kez ulaşıldığında bir ana bilgisayar anahtarı ekler.
- **Parolalar:**
  - WinSCP oturum, tünel ve proxy parolalarını saklayabilir.
  - PuTTY yalnızca bir proxy'nin parolasını düz metin olarak saklar.
  - `PasswordStored` ve `Details` içindeki `StoredPasswords`, hangi değerlerin ayarlandığını belirtir.
    Bunların kodu asla çözülmez ve dışarı yazılmaz.
- **WinSCP varsayılanları dışarıda bırakır.** `FSProtocol` içermeyen bir oturum SFTP'dir ve `PortNumber`
  içermeyen bir oturum protokolün varsayılan bağlantı noktasını kullanır. Bu durumda `Protocol` ve `Port`
  tahmin yürütmek yerine boş kalır.
- **Ana bilgisayar anahtarı parmak izleri**, `ssh-keygen -l`, `ssh` ve PuTTY 0.75 ve sonrasının gösterdiği
  biçimde OpenSSH'nin `SHA256:…` biçimindedir. Juntei bunları PuTTY ve WinSCP'nin tuttuğu anahtardan
  hesaplar. Bir adın veya IP adresinin arkasındaki sunucuyu belirlemek için bunları sunucunun kendi
  anahtarıyla, `known_hosts` dosyalarıyla ve günlüklerle karşılaştırın. Bir SSH-1 anahtarının (`rsa`)
  parmak izi yoktur.
- **Eşlenmiş sürücüler:** `RemoteUsername` yalnızca sürücü başka bir hesapla eşlendiğinde ayarlanır;
  kullanıcının kendi hesabı kullanıldığında Windows bir `REG_DWORD` 0 yazar.
- **Paylaşımlar:** `MountPoints2` birimler ve sürücü harfleri için de anahtarlar içerir. Burada yalnızca
  ağ paylaşımları (`##server#share`) raporlanır; `\\server@SSL@443\DavWWWRoot` bir WebDAV paylaşımıdır.
- **Kaldırma:** listelerin temizlenmesi veya taşınabilir bir istemci kullanılması burada iz bırakmaz.
  Jump Lists, Prefetch, `.rdp` dosyaları, RDP bitmap önbelleği ve hedefin günlükleriyle karşılaştırın.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry remote-access -d evidence/ -o remote.csv
juntei parse-registry remote-access -f NTUSER.DAT -t jsonl \
  | jq 'select(.Client == "rdp" or .HostKeyFingerprint != null)'
```

Özet, bağlantıları istemciye göre sayar. Anahtarları en son yazılan bağlantıları en yeniden başlayarak ve
parola tutan oturumları listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Ana bilgisayar, profil klasörünün adı, sistemin `ProfileList` kaydından (sistemin `SOFTWARE` kovanı ile) kullanıcının SID'si ve kovan. |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive` veya `network_share`. |
| `Kind` | Kaydın nereden geldiği: `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point`. |
| `Target`, `Port` | Bilgisayar (ana bilgisayar adı veya IP adresi) ve bağlantı noktası ya da paylaşım (`\\server\share`). Bağlantı noktası yalnızca Windows'un veya istemcinin bir tane yazdığı yerde verilir. |
| `RemoteUsername` | Hedefte kullanılan hesap: `UsernameHint`, bir oturumun `UserName` değeri (veya PuTTY'nin `HostName` değerindeki `user@`), eşlenmiş bir sürücünün `UserName` değeri. |
| `Protocol` | PuTTY'nin `Protocol` değeri (`ssh`, `telnet`, `raw`, …), WinSCP'nin `FSProtocol` değeri (`sftp`, `scp`, `ftp`, `webdav`, `s3`), eşlenmiş bir sürücünün sağlayıcısı (`Microsoft Windows Network`). |
| `MRU-Position` | Listesindeki yeri (0 en sonuncusudur): `MRU<n>`, `MRUList`, PuTTY'nin `Recent sessions` değeri. |
| `SessionName` | Kaydedilmiş bir oturumun adı; `%20` ve diğer kaçış dizilerinin kodu çözülmüş olarak. |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | Kabul edilmiş bir SSH ana bilgisayar anahtarının türü (`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …), OpenSSH parmak izi ve saklandığı biçimdeki sayıları. |
| `DriveLetter` | Eşlenmiş bir sürücünün harfi. |
| `PasswordStored` | Oturumun parola tutup tutmadığı; kodu asla çözülmez. |
| `Details` | Diğer ayarlar; `Name=Value; …` (CSV) veya bir nesne (JSON) olarak: RDP sertifikasının `CertHash` değeri, `LocalDevices`, bir oturumun anahtar dosyası, proxy, SSH tüneli, bağlantı noktası yönlendirmeleri, uzak komut ve dizinler, `StoredPasswords`, bir paylaşımın `Label` değeri, bir sürücünün `ConnectionType` değeri. |
| `KeyPath`, `ValueName` | Anahtar ve tek bir değerden oluşan bir kayıt için değer. |
| `KeyLastWrittenUTC` | Anahtarın son yazılma zamanı (listeler için uyarılara bakın). |
| `DataRaw` | Bir dize olmadığında veya kodu çözülemediğinde değerin baytları (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Yanlış türdeki değerler (baytlarıyla birlikte), okunamayan ana bilgisayar anahtarları, mevcut olmayan `MRUList` harfleri ve listelemediği değerler. |

## İnceleme stratejisi { #investigation-strategy }

1. Her kullanıcının hedeflerini istemciye göre listeleyin ve o kullanıcının erişmesi gereken sistemlerle
   karşılaştırın. Yönetim paylaşımlarını (`C$`, `ADMIN$`) ve IP adreslerini arayın.
2. Kullanılan hesapları (`RemoteUsername`) not edin: etki alanı yöneticileri ve diğer makinelerdeki yerel
   hesaplar.
3. `HostKeyFingerprint` değerini sunucuların anahtarlarıyla, `CertHash` değerini RDP sunucularının
   sertifikalarıyla eşleştirin.
4. Hedeflerin oturum açma olayları (4624 tür 10 ve 3, 1149), Jump Lists ve LNK dosyaları (`mstsc.exe`,
   `putty.exe`, paylaşımlardan açılan dosyalar), Prefetch ve RDP bitmap önbelleğiyle ilişkilendirin.
5. Silinmiş girdiler için `parse-registry read --recover-deleted` komutunu, temizlenmiş listeler için
   profilin `NTUSER.DAT` dosyasının eski kopyalarını (Volume Shadow Copies, yedekler) kullanın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Remove entries from the Remote Desktop Connection Computer box](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [PuTTY FAQ: Where does PuTTY store its data?](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: Configuration storage](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
