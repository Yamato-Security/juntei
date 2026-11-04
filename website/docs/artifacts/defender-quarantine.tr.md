# Microsoft Defender Karantinası { #microsoft-defender-quarantine }

## Karantina neleri içerir { #what-the-quarantine-contains }

Microsoft Defender karantinası, bir tespite ilişkin meta verileri ve karantinaya aldığı kaynakların
şifrelenmiş içeriğini korur. Juntei; tehdit etiketini, karantina zamanını, özgün mantıksal ve fiziksel yolları,
dosya meta verilerini, sahibi, indirme kaynağını gösteren `Zone.Identifier` verisini, karmaları ve bütünlük
sorunlarını raporlamak için Entries, ResourceData ve Resources kayıtlarını birbiriyle ilişkilendirir.

Bu artefakt, tespiti/düzeltmeyi ilk kaydeden olay günlüklerinden daha uzun süre varlığını sürdürebilir.
Ayrıca yalıtılmış tersine mühendislik için şüpheli baytların birebir aynısını da koruyabilir.

<span class="evidence-strength">En iyi sonuç:</span> Defender, adı geçen tehdit/kaynak için karantina
kaydını kaydedilen zamanda oluşturmuştur. Ürün etiketi kötü amaçlı yazılımın bağımsız bir kanıtı değildir
ve karantina tek başına önceden yürütme gerçekleştiğini kanıtlamaz.

## Konum ve toplama { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

`Quarantine` dizininin tamamını toplayın. Eşleşen içerik/kayıt dosyaları olmayan bir girdi yine de
kullanışlı meta veriler sağlar, ancak dosya baytlarını veya hesaplanan karmaları veremez.

## Güvenlik ve analist için uyarılar { #safety-and-analyst-cautions }

- Çıkarılan içerik büyük olasılıkla kötü amaçlıdır. Yalıtılmış bir analiz sistemi kullanın ve asla çalıştırmayın.
- `ThreatName`, Defender'ın o andaki imzası/sınıflandırmasıdır. Hatalı pozitifler ve sonradan
  yeniden sınıflandırma mümkündür.
- Karantina zamanı düzeltme/karantina zamanıdır; indirme, oluşturma veya yürütme zamanı olmak zorunda değildir.
- Karantinaya alınmış bir dosya, yürütülmeden önce veya çalıştıktan sonra tespit edilmiş olabilir.
- Özgün yol ve Zone.Identifier değerlidir, ancak bulunmayabilir veya saldırganın denetiminde olabilir.
- Sahipsiz (orphan) kayıtlar; saklama/temizleme, eksik toplama veya bozulma sonucunda oluşabilir.
- Juntei boyutları, CRC'leri, kimlikleri ve karmaları doğrular; kurtarılan baytlara güvenmeden önce `Problems` alanını inceleyin.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

Örneğin `tool.exe`, `tool.exe.quarantined` olur. Yardımcı akışların adında türleri de yer alır;
örneğin `tool.exe.Zone.Identifier.quarantined`. Juntei ayrıca yürütme iznini de kaldırır.
Uzantı ve izinler kazara yürütme olasılığını azaltır, ancak kurtarılan baytlar yine de
kötü amaçlı yazılım olarak ele alınmalıdır.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Record` | `resource`, kaynağı olmayan girdi, `orphan-content` veya `orphan-record`. Sahipsiz kayıtlar eksik bağlantıyı/toplamayı ortaya çıkarır. |
| `QuarantineTime` | UTC olarak Defender karantina/düzeltme zamanı; yürütme zamanı değildir. |
| `ThreatName`, `ThreatID` | Defender sınıflandırması ve sayısal kimliği. Üretici kararı/bağlamı olarak değerlendirin. |
| `ResourceType` | Dosya, kayıt defteri anahtarı, hizmet, başlangıç öğesi, zamanlanmış görev veya başka bir kaynak kategorisi. |
| `Path`, `PhysicalPath` | Özgün mantıksal/fiziksel konum. Birincil ilişkilendirme pivot noktasıdır; yazımı birebir koruyun. |
| `FileSize` | Kaynak meta verilerinin kaydettiği boyut. `ContentSize` ve MFT ile karşılaştırın. |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | Karantinaya alınan kaynak için yakalanan meta veriler; karantina zamanından ayrıdır ve zaman damgası oynamaya (timestomping) açıktır. |
| `Attr` | Kaydedilen dosya öznitelikleri. Gizli/sistem durumu faydalı bağlam sağlayabilir. |
| `Owner` | Kurtarılan güvenlik tanımlayıcısındaki SID; sahiplik, yürüten kullanıcı olmak zorunda değildir. |
| `ZoneID`, `ReferrerURL`, `HostURL` | `Zone.Identifier` içindeki indirme bölgesi ve URL değerleri. Mevcut olduğunda kaynağa ilişkin güçlü ipuçlarıdır; ADS bulunmayabilir/kaldırılmış olabilir. |
| `ContentSize` | Kurtarılan birincil içeriğin gerçek boyutu. Kaydedilen `FileSize` ile karşılaştırın. |
| `MD5`, `SHA1`, `SHA256` | Kurtarılan baytlardan hesaplanır. Modern içerik tanımlaması için SHA-256 kullanın; birlikte çalışabilirlik için hepsini saklayın. |
| `Streams` | Kurtarılan akışların türleri/adları/boyutları; ADS, güvenlik verileri veya bir kayıt defteri kovanı içerebilir. |
| `ResourceID` | Kaynak meta verisi/içerik kayıtlarını birbirine bağlayan tanımlayıcı. |
| `EntryID` | Karantina girdisi GUID'i; Defender geçmişi/olay kanıtlarıyla da bağlantı sağlar. |
| `ScanID` | Kaynakları/eylemleri gruplamak için Defender tarama tanımlayıcısı. |
| `QuarantineFolder` | Kök kaynak klasörü/ana bilgisayar ayrımı. |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | Satırı oluşturmak için kullanılan kaynak dosyaların tam yolları. Doğrulama için vazgeçilmezdir. |
| `ExtractedFile` | `--extract` tarafından yazılan birincil karantina dosyası; birincil çıkarma yapılmadığında boştur. Yardımcı akışlar yanına yazılır. `auto-parse -x` bunu kendi çıktı dizinine göre verir (`./HOST1/SecurityProducts/DefenderQuarantined/…`). |
| `Problems` | Bütünlük uyuşmazlığı, eksik içerik, kimlik uyuşmazlığı, hasar veya sahipsiz kayıt bağlamı. Gizlemeyin. |

JSON/JSONL, CSV'nin birleştirilmiş metni yerine yapılandırılmış kaynağı, akış listesini, kayıt girdilerini ve
tüm ilişki ayrıntılarını korur.

## Çıkarma davranışı { #extraction-behavior }

Karantina girdisi özgün dosya adını kaydetmişse Juntei bu adı, sonuna `.quarantined` uzantısı
ekleyerek kullanır. Sahipsiz veya dosya olmayan kaynaklar için içerik tanımlayıcısını kullanır ve dosya adı
çakışmalarını çözmek için tanımlayıcıyı ekler. Windows'ta geçersiz olan dosya adı karakterleri değiştirilir. Güvenlik
akışları ve alternatif akışlar birincil dosyanın yanına yazılır. Her güvenlik akışı için ayrıca okunabilir
bir metin raporu da oluşturulur. Juntei yürütme iznini kaldırır;
farklı içeriğe sahip mevcut dosyaların üzerine `-C` olmadan yazılmaz. Bu denetimler kazaları azaltır, ancak
baytları güvenli hâle getirmez.

### Kurtarılan güvenlik tanımlayıcısının okunması { #reading-the-recovered-security-descriptor }

Bir `.security.quarantined` dosyası ikili biçimde Windows güvenlik meta verileri içerir, bu nedenle bir
metin düzenleyicide okunamaz. Bu dosya ayrı bir yedek akıştır, yürütülebilir dosyanın başka bir kopyası değildir. Juntei,
`parse-file defender-quarantine --extract` veya `auto-parse -x` ile çıkarma yapılırken bunun kodunu otomatik olarak çözer:

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

Metin dosyasının adı, varsa çakışma tanımlayıcısı da dahil olmak üzere gerçek ikili dosyanın adını izler. Üretilen
metin sıradan UTF-8'dir ve diğer çıkarılan dosyalarla aynı üzerine yazma korumalarını kullanır. Raporun
kaynağıyla eşleştirilebilmesi için ikili dosyanın adını, bayt sayısını ve SHA-256 değerini içerir.

| Rapor alanı | Nasıl yorumlanır |
|---|---|
| Sahip ve birincil grup | Tanımlayıcıda kaydedilen SID'ler. Bilinen SID'lere `BUILTIN\Administrators` veya `NT AUTHORITY\SYSTEM` gibi adlar verilir. Diğer SID'ler çözümlenmeden kalır; tanıdık bir RID tek başına yerel/etki alanı hesabını adlandırmak için yeterli değildir. Sahiplik, bir dosyayı kimin oluşturduğunu veya çalıştırdığını kanıtlamaz. |
| Denetim bayrakları | Özgün sayısal bayraklar ve adları; ACL varlığı, devralma ve koruma dahil. |
| DACL | Sıralı erişim denetimi girdileri (ACE'ler): izin verme/reddetme türü, hesap SID'i, izin maskesi, haklar ve devralma bayrakları. Bunlar kaydedilmiş girdilerdir; bir kullanıcının belirteci (token) için etkin erişimin hesaplanması değildir. |
| SACL | Kaydedilmiş denetim (audit) veya zorunlu bütünlük girdileri. Denetim başarı/başarısızlık bayrakları bir erişim geçmişini değil, yapılandırmayı açıklar. SACL'nin bulunmaması tüm denetimin devre dışı olduğunu göstermez. |
| Nesne GUID'leri | Nesneye özgü ACE'ler için isteğe bağlı GUID'ler; nesneye özgü haklar, yanlışlıkla dosya hakları olarak etiketlenmek yerine sayısal olarak korunur. |
| SDDL | Sahip, grup ve desteklenen ACL girdilerinin Security Descriptor Definition Language gösterimi. Denetim bayraklarına ayrıca bakın. Herhangi bir bileşen hasarlı veya desteklenmiyorsa Juntei kısmi bir SDDL dizesi üretmez. |
| Kod çözme durumu ve sorunlar | Gerekçeleriyle birlikte `complete` veya `incomplete`. Desteklenmeyen ACE'ler raporda ham onaltılık biçimleriyle korunur. Çıkarma başarılı olduğunda özgün ikili dosya her zaman korunur. Kod çözme sorunları karantina kaydının `Problems` alanında da görünür. |

Bulunmayan bir DACL, NULL DACL ve boş DACL ayrı ayrı raporlanır. Bulunmayan veya NULL bir DACL
isteğe bağlı (discretionary) hiçbir erişim kısıtlaması sağlamaz; boş bir DACL ise erişim veren hiçbir ACE içermez. Diğer Windows erişim
denetimleri ve ayrıcalıkları yine de önemlidir. Devralınan ve yalnızca devralınan (inherit-only) ACE'ler açıkça işaretlenmiş olarak kalır.

Örneğin, Administrators grubuna devralınmış tam denetim veren bir girdi şöyle görünebilir:

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

Kod çözücü; Windows, macOS ve Linux üzerinde Windows API'leri, Python veya hesap araması olmadan çevrimdışı çalışır.
Temel izin verme/reddetme/denetim/alarm ACE'lerinin, nesne ACE'lerinin ve zorunlu bütünlük etiketlerinin kodunu çözer. Koşullu,
geri çağırma (callback), kaynak özniteliği ve diğer desteklenmeyen ACE'ler açıkça tanımlanır ve ham
biçimde korunur; erişim koşulları değerlendirilmez. Hasarlı bir tanımlayıcı, diğer akışların
çıkarılmasını durdurmaz.

## İnceleme stratejisi { #investigation-strategy }

1. `ThreatName`, `Path`, `QuarantineTime`, `EntryId` ve karmalardan başlayın.
2. `Problems` alanını doğrulayın ve kaydedilen boyutu kurtarılan içerikle karşılaştırın.
3. SHA-256 değerini diğer ana bilgisayarlara, EDR, sandbox ve tehdit istihbaratı verilerine pivot olarak kullanın.
4. Özgün yolu/zamanı `$MFT`, USN Journal, Prefetch, Amcache ve olay günlükleriyle ilişkilendirin.
5. İlk erişimi araştırmak için URL/bölge alanlarını kullanın, ancak tarayıcı/proxy/DNS kanıtlarıyla doğrulayın.

## Daha fazla ayrıntı { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [dissect.target Defender eklentisi](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: Microsoft Defender Antivirus düzeltme ve karantina](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft: Güvenlik tanımlayıcıları](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: Security Descriptor String Format](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: ACE dizeleri](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
