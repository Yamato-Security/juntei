# Güvenlik Ayarları { #security-settings }

## Bunlar nedir { #what-they-are }

Saldırganlar gizli kalmak ve kimlik bilgilerine ulaşmak için Windows'un güvenlik ayarlarını değiştirir. Microsoft Defender'ı
kapatır veya araçlarını onun kapsamı dışında bırakır, Windows'un parolaları açık metin olarak tutmasını sağlar,
Uzak Masaüstü'nü açar, güvenlik duvarını kapatır ve PowerShell günlüğünü susturur. Kayıt defteri
bu ayarları tutar:

- **`SOFTWARE`:** Defender, dışlamaları ve ilkeleri; UAC; Uzak Masaüstü ve güvenlik duvarı
  ilkeleri; PowerShell yürütme ilkesi, günlük ilkeleri ve AMSI sağlayıcıları; Sistem Geri Yükleme;
  olay günlüğü kanalları.
- **`SYSTEM`** (denetim kümesi başına): `WDigest`, LSA koruması, Credential Guard, Restricted Admin,
  NTLM; Uzak Masaüstü; güvenlik duvarı profilleri; SMBv1; son erişim zamanları; her olay günlüğünün boyutu.
- **`NTUSER.DAT`:** kullanıcının PowerShell yürütme ilkesi ve günlük ilkeleri ile
  Windows Script Host için AMSI.

`parse-registry security-settings`, bulduğu her ayarı bir kayıt olarak raporlar. Her kayıt
ayarın değerini, değerin ne anlama geldiğini ve **varsayılanı zayıflatıp zayıflatmadığını** içerir: yani bir
korumayı veya kanıt kaynağını kapatıp kapatmadığını ya da onu güncel Windows'un (10 ve 11) bu değer
olmadan yaptığının altına düşürüp düşürmediğini.

<span class="evidence-strength">En iyi sonuç:</span> kovan kaydedildiğinde bu ayar bu değere
sahipti ve anahtar en son bu zamanda yazıldı. Zayıflatılmış bir ayar bir ipucudur, ele geçirilmenin kanıtı değildir.
Yöneticiler, Grup İlkesi, diğer güvenlik ürünleri ve eski Windows sürümleri de bu değerlerin bazılarını
ayarlar; bu durum yaygın olduğunda anlam alanı bunu belirtir.

## Konum { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

Üç kovanın tamamını işlem günlükleriyle birlikte toplayın. Toplamadan kısa süre önce değiştirilen bir ayar
yalnızca günlüklerde bulunabilir.

## Analist için uyarılar { #analyst-cautions }

- **Saklanan yapılandırma, etkin koruma durumu değildir.** Toplanan kayıt defteri değerleri ve hasarlı değer listeleri için tanılama satırları
  raporlanır. Eksik veriler varsayılanları, toplanmamış kaynakları veya başka bir yönetim
  mekanizmasını yansıtabilir; korumanın etkin veya devre dışı olduğunun kanıtı değildir. `WeakensDefault`, uygulandıklarında
  belirtilen temel düzeyi düşürebilecek ayarları işaretler.
- **Eski Defender bayrakları yok sayılabilir.** `DisableAntiSpyware` ve `DisableAntivirus`, Defender'ın durduğunu
  kanıtlamaz. Uygulanabilirlik platform sürümüne, eklemeye (onboarding) ve kurcalama korumasına (tamper
  protection) bağlıdır. [Microsoft bu eski ayarları açıklar](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware).
- **Modern yönetilen dışlamalar kayıt defteri yakalamalarında bulunmayabilir.** Platform
  4.18.25110.6 sürümünden itibaren, Defender for Endpoint yapılandırma yönetimini kullanan aygıtlar, dışlama yapılandırmasını
  almak için desteklenen Defender PowerShell cmdlet'lerini gerektirir. Bu nedenle çevrimdışı kayıt defterinde bulunmaması,
  dışlama olmadığını göstermez.
  [Microsoft yapılandırma kılavuzu](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings).
- **Zayıflatılmış, kötü amaçlı demek değildir.** Örnekler:
  - `DisableAntiSpyware` = 1, başka bir antivirüs kayıtlıyken Windows tarafından da ayarlanır.
  - Uzak Masaüstü sunucularda bilinçli olarak açılır.
  - Windows 7 daha küçük olay günlükleri tutardı ve son erişim zamanlarını güncellemezdi.
- **Varsayılanlar sürüme bağlıdır.** Bazı değerler eskiden varsayılandı:
  - `NtfsDisableLastAccessUpdate` = 1, Windows 10 1803'ten önce varsayılandı. Bir hüküm olmadan
    raporlanır; ancak daha sonraki bir sürümde elle ayarlanan `0x80000001` zayıflatılmış sayılır.
  - `RunAsPPL`, `LsaCfgFlags` ve `UseLogonCredential` güncel Windows ile karşılaştırılır.
- **Defender dışlamaları:** her yol, uzantı, işlem veya adres bir kayıttır. Her dışlama
  Defender'ı zayıflatır. Defender'ın kendisinin eklediği `TemporaryPaths` için hüküm verilmez. Kullanıcının yazabildiği klasörleri,
  `.ps1`/`.exe` uzantılarını ve LOLBin'leri (`rundll32.exe`, `powershell.exe`) arayın.
- **İlke anahtarları** (`Policy` = True) Grup İlkesi, araçlar ve ilkelerin yerel ayarlara üstün geldiğini bilen
  saldırganlar tarafından yazılır. Bunları etki alanının GPO'larıyla ilişkilendirin.
- **Olay günlüğü kanalları:** yalnızca güvenlikle ilgili 25 kanal, Windows'un her birini varsayılan olarak
  açıp açmadığı bilgisiyle birlikte raporlanır. Örneğin `TaskScheduler/Operational` ve `DNS-Client/Operational`
  varsayılan olarak kapalıdır. Diğerleri için `parse-registry read` kullanın.
- **Anahtar zamanları:** `KeyLastWrittenUTC` anahtara aittir; bu nedenle bu değerin değiştiği zaman olmak
  zorunda değildir. Bir `Policies` anahtarının tamamının tek seferde yazılması yaygındır.
- **Denetim kümeleri:** varsayılan olarak Windows'un en son kullandığı denetim kümesi okunur. `--all-control-sets`
  tümünü okur.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

Özet, ayarları alana göre sayar. Varsayılanı zayıflatan ayarları ve tüm
Defender dışlamalarını listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Ana bilgisayar, kovan ve türü (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`). |
| `ControlSet`, `IsCurrentControlSet` | `SYSTEM` için: denetim kümesi ve Windows'un en son kullandığı küme olup olmadığı; `Select\Current` eksik veya geçersiz olduğunda bilinmez. |
| `User`, `UserSID` | `NTUSER.DAT` için: profil klasörünün adı ve sistemin `ProfileList` verisinden kullanıcının SID'i. |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`. |
| `Setting` | Ayarın ne olduğu: `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | Değer: ondalık bir sayı, bir dize veya dışlamanın kendisi. Değer ayarın türünde değilse boştur. |
| `Meaning` | Değerin ne anlama geldiği. |
| `WeakensDefault` | Değer, güncel Windows'un o değer olmadan yaptığını zayıflatıyorsa `True`, zayıflatmıyorsa `False`; bu uygulanamıyorsa (bir boyut, bir bağlantı noktası, bir AMSI sağlayıcısı) veya belirlenemiyorsa boş. |
| `Policy` | Anahtarın bir `Policies` anahtarı altında olup olmadığı. |
| `KeyPath`, `ValueName` | Anahtar ve değer. Bir dışlamanın değer adı dışlanan öğedir; bir AMSI sağlayıcısının değer adı yoktur. |
| `KeyLastWrittenUTC` | Anahtarın son yazılma zamanı. |
| `DataRaw` | Ayarın türünde olmadığında değerin baytları (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Yanlış türdeki değerler ve okunamayan anahtarlar. |

## İnceleme stratejisi { #investigation-strategy }

1. Her ana bilgisayarda `--weakened-only` ile çalıştırın ve ana bilgisayarları birbiriyle karşılaştırın: yalnızca
   tek bir ana bilgisayarda bulunan bir ayar dikkat çeker.
2. Her Defender dışlamasını ana bilgisayarda ve Defender günlüklerinde bulunan araçlarla karşılaştırın
   (Olay 5007 yapılandırma değişikliklerini kaydeder).
3. Anahtar zamanlarını oturum açmalar, hizmet yüklemeleri ve işlem oluşturma ile ilişkilendirin.
   `reg.exe`, `Set-MpPreference`, `netsh advfirewall` ve `powershell` komut satırlarını arayın (olay
   4688, Sysmon 1 ve 13, PowerShell 4104).
4. Kimlik bilgisi ayarları (`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`) için sonrasında gerçekleşen kimlik bilgisi dökümü ve pass-the-hash etkinliklerini arayın.
5. Bir varsayılanı geri yüklemek için silinen değerler için `parse-registry read --recover-deleted`, eski
   denetim kümeleri için `--all-control-sets` kullanın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Microsoft Defender Virüsten Koruma için dışlamaları yapılandırma ve doğrulama](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: Kullanıcı Hesabı Denetimi ayarları ve yapılandırması](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: Ek LSA korumasını yapılandırma](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
