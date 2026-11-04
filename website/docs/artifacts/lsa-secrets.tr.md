# LSA Sırları { #lsa-secrets }

## Bunlar nedir { #what-they-are }

Yerel Güvenlik Yetkilisi (Local Security Authority, LSA), sırları `SECURITY` kovanında tutar. Bunlar arasında
otomatik oturum açma (autologon) parolası, hizmetlerin altında çalıştığı hesapların parolaları, bilgisayar hesabının etki alanı
parolası, DPAPI sistem anahtarları ve önbelleğe alınmış etki alanı oturum açma bilgilerini şifrelemek için kullanılan anahtar bulunur.
LSA ayrıca etki alanı kullanıcılarının bir etki alanı denetleyicisi olmadan oturum açabilmesi için önbelleğe alınmış etki alanı oturum açma bilgilerini de tutar.

`parse-registry lsa-secrets`, bir sistemin hangi sırları tuttuğunu ve bunların ne zaman ayarlandığını, ayrıca kaç tane
önbelleğe alınmış oturum açma bilgisi tuttuğunu ve kullandığını listeler. **Yalnızca adları ve zamanları okur: hiçbir şeyin
şifresi çözülmez ve hiçbir sırrın baytları okunmaz veya yazılmaz.** Juntei kanıtları okur; kimlik bilgilerini çıkarmaz.

<span class="evidence-strength">En iyi sonuç:</span> sistem, bu zamanda ayarlanmış bu adda bir sırrı
tutmuştur. Bir hizmetin sırrı (`_SC_<service>`), LSA'nın o hizmetin altında çalıştığı hesabın parolasını
sakladığı anlamına gelir.

## Konum { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| Ad | Tür | Nedir |
|---|---|---|
| `DefaultPassword` | `autologon` | Winlogon'un otomatik olarak oturum açarken kullandığı parola. |
| `_SC_<service>` | `service_account` | `<service>` hizmetinin altında çalıştığı hesabın parolası. |
| `$MACHINE.ACC` | `machine_account` | Bilgisayar hesabının kendi etki alanındaki parolası. |
| `DPAPI_SYSTEM` | `dpapi_system` | Sistemin DPAPI anahtarları. |
| `NL$KM` | `cached_logon_key` | Önbelleğe alınmış oturum açma bilgilerini şifrelemek için kullanılan anahtar. |
| `L$_SQSA_<SID>` | `security_questions` | Yerel bir hesabın güvenlik soruları (Windows 10 ve sonrası). |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | Çevirmeli bağlantı ve VPN kimlik bilgileri. |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | `LsaStorePrivateData` tarafından verilen öneke göre diğer özel veriler. |

`SECURITY` kovanını işlem günlükleriyle birlikte ve ana bilgisayarı adlandıran sistemin `SYSTEM` kovanını
toplayın.

## Analist için uyarılar { #analyst-cautions }

- **Değerler değil, adlar ve zamanlar.** Bir değerin veri içerip içermediği boyutundan anlaşılır. İçeriği
  okunmaz.
- **`DefaultPassword` otomatik oturum açma anlamına gelmez.** Bu sır, hiçbir zaman otomatik olarak oturum açmayan
  sistemler de dahil olmak üzere çoğu sistemde bulunur. Otomatik oturum açmanın yapılandırılıp yapılandırılmadığını
  `SOFTWARE` kovanındaki `Microsoft\Windows NT\CurrentVersion\Winlogon` (`AutoAdminLogon`, `DefaultUserName`)
  belirtir. Yakın tarihli bir `CurrentSetUTC`, açıklanması gereken bir değişikliktir.
- **Hizmet sırları:** `_SC_<service>`, bir hizmet parolası olan bir hesap olarak çalışacak şekilde ayarlandığında
  görünür. Bunu `parse-registry services` (`ObjectName`) ile karşılaştırın ve yakın zamanda yüklenmiş veya
  bilinmeyen hizmetleri arayın. Sır, hizmet kaldırıldıktan sonra da kalabilir.
- **Zamanlar:** `CurrentSetUTC` ve `OldSetUTC`, Windows'un geçerli ve önceki değerleri ayarladığı zamanlardır.
  `$MACHINE.ACC` parolası varsayılan olarak her 30 günde bir değişir, bu nedenle zamanları da değişir.
  `KeyLastWrittenUTC` ve `CurrVal`/`OldVal` anahtar zamanları, sırrın en son ne zaman değiştiğine
  ilişkindir.
- **Önbelleğe alınmış oturum açma bilgileri:** bir yuva, kaydı bir kullanıcıyı adlandırdığında kullanımdadır. Kaydın başlığı
  şifrelenmemiştir; yalnızca kullanıcı adı uzunluğu okunur. `CachedLogonsKept`, `CachedLogonsCount` değerini (varsayılan olarak 10)
  izleyen `NL$Control` sayısıdır. Kullanımdaki yuvalar, etki alanı kullanıcılarının etkileşimli olarak oturum açtığı
  anlamına gelir ve bu kullanıcıların kimlik bilgileri kovandan çevrimdışı olarak kırılabilir. Kullanıcıların adları
  şifrelidir ve kodları çözülmez. Yalnızca eksiksiz bir başlığa ve tutarlı bir kullanıcı adı uzunluğuna sahip doğrulanmış ikili kayıtlar kullanımda sayılır. `Problems` hasarlı yuvalar bildiriyorsa, kullanımdaki sayısı yalnızca doğrulanmış yuvaları kapsar; hasarlı yuvaları boş olarak yorumlamayın.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

Özet, her kovan için her sırrı türüyle ve ne zaman ayarlandığıyla birlikte listeler. Ardından hesap
parolaları LSA tarafından saklanan hizmetleri adlandırır ve kullanımdaki önbelleğe alınmış oturum açma yuvalarını sayar.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath` | Ana bilgisayar (sistemin `SYSTEM` kovanından) ve `SECURITY` kovanı. |
| `Secret`, `Kind` | Sırrın adı ve ne olduğu (yukarıdaki tabloya bakın); önbelleğe alınmış oturum açma bilgileri kaydı için `Cache` ve `cached_logons`. |
| `ServiceName`, `UserSID` | Bir `_SC_` sırrının ait olduğu hizmet ve bir adın verdiği kullanıcı SID'i (`L$_SQSA_`, `RasDialParams`). |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` ve `OupdTime`: geçerli ve önceki değerlerin ayarlandığı zamanlar. |
| `HasCurrentValue`, `HasOldValue` | `CurrVal` ve `OldVal` değerlerinin veri içerip içermediği. |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | `CurrVal`, `OldVal` ve sır anahtarlarının son yazılma zamanları. |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | Önbelleğe alınmış oturum açma bilgileri kaydı: yuvalar, kullanımdaki yuvalar ve `NL$Control` sayısı. |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Bozuk veya aralık dışı zamanlar (yalnızca zaman meta verileri gösterilir), hasarlı önbellek başlıkları, yanlış önbellek değer türleri ve okunamayan anahtarlar. |

## İnceleme stratejisi { #investigation-strategy }

1. Bilinmeyen veya yakın zamanda yüklenmiş hizmetlere ait `_SC_` sırlarını arayın ve bunları
   `parse-registry services` ve 7045 olayı (bir hizmet yüklendi) ile eşleştirin.
2. `DefaultPassword` zamanlarını `Winlogon` otomatik oturum açma ayarlarıyla karşılaştırın.
3. Etki alanı üyelerinde kullanımdaki önbelleğe alınmış oturum açma yuvalarını ve `Cache` anahtarının en son ne zaman yazıldığını not edin.
4. Sırların zamanlarını zaman çizelgesiyle karşılaştırın: bir izinsiz giriş zaman aralığında ayarlanan bir sır,
   yeni bir hizmet hesabına, otomatik oturum açma veya makine hesabı değişikliğine işaret edebilir.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Özel veri nesnesi](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: Önbelleğe alınmış etki alanı oturum açma bilgileri](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: Cached credentials](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
