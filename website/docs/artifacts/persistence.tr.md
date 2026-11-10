# Kayıt Defteri Otomatik Başlatmaları { #registry-autostarts }

## Bunlar nedir { #what-they-are }

Windows birçok kayıt defteri konumundan program başlatır ve DLL yükler: önyüklemede, oturum açmada, bir
program başladığında veya kapandığında, `cmd.exe` açıldığında, bir COM sınıfı oluşturulduğunda ya da
yazdırma biriktiricisi (spooler), LSA, netsh veya zaman hizmeti eklentilerini yüklediğinde. Saldırganlar
bu konumları kalıcılık, ayrıcalık yükseltme ve DLL enjeksiyonu için kullanır; meşru yazılımlar da bunları
kullanır. Juntei bunları `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` ve `UsrClass.dat` kovanlarından tek bir tabloya
okur. Program, DLL veya paket başına bir kayıt yazar ve her birini MITRE ATT&CK tekniğiyle etiketler.

<span class="evidence-strength">En iyi sonuç:</span> kovan, bu programı veya DLL'i başlatılacak ya da
yüklenecek şekilde yapılandırmıştır. Bu, programın çalıştığını kanıtlamaz. Çalışması için mekanizmanın
belirttiği koşullar (bir oturum açma, hata ayıklanan programın başlaması, sınıfın oluşturulması) ve diğer
yürütme kanıtları gerekir.

## Konumlar { #locations }

| Kovan | Okunanlar |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce` (+`Setup`), `RunOnceEx`, `RunServices(Once)` ve `Policies\Explorer\Run`, bunların `Wow6432Node` ve Terminal Server yükleme modu kopyalarıyla birlikte; `Explorer\StartupApproved\Run`, `Run32` ve `StartupFolder`; `Winlogon` (`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … ve `Notify`); `LoadAppInit_DLLs` ile birlikte `Windows` `AppInit_DLLs`; `Image File Execution Options\*` `Debugger` (`UseFilter` ile birlikte) ve `VerifierDlls`; `SilentProcessExit\*` `MonitorProcess`; `Active Setup\Installed Components\*` `StubPath`; `NetSh`; `Command Processor` `AutoRun` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, `Environment` .NET profil oluşturucu değişkenleri); `Control\Lsa` `Authentication`, `Security` ve `Notification Packages` (+`OSConfig`); `SecurityProviders`; `Control\Print` izleyicileri, sağlayıcıları ve yazdırma işlemcileri; `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | kullanıcının `Run` anahtarları ve `StartupApproved`; `Windows NT\CurrentVersion\Windows` `Load` ve `Run`; `Winlogon` `Shell`; `Environment` (`UserInitMprLogonScript`, .NET profil oluşturucu); `Control Panel\Desktop` `SCRNSAVE.EXE`; `PackagedAppXDebug`; `Command Processor` `AutoRun`; `Software\Classes` (`UsrClass.dat` olarak) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` ve `LocalServer32` (ve `Wow6432Node` altındakiler); program türlerinin (`exefile`, `batfile`, `scrfile`, …) ve otomatik yükselen programların başlattıklarının (`ms-settings`, `mscfile`, `Folder`, …) `shell\open\command` anahtarları; `ActivatableClasses\…\DebugInformation` |

Her kovanı işlem günlükleriyle birlikte toplayın. Kullanıcı kovanları `Users\<name>\NTUSER.DAT` ve
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat` konumlarındadır.

## Analist için uyarılar { #analyst-cautions }

- **Varsayılanlar da raporlanır.** Temiz bir sistemdeki kayıtların çoğu Windows varsayılanlarıdır:
  - `Userinit`, `userinit.exe` değeridir;
  - `Shell`, `explorer.exe` değeridir;
  - `KnownDLLs`, LSA paketleri, yazdırma izleyicileri, netsh yardımcıları ve Active Setup bileşenlerinin tümünün standart girdileri vardır.

  Aynı sürümün temiz bir kurulumundan farklı olanları arayın.
- **`StartupApproved` yalnızca Görev Yöneticisi'nin Başlangıç listesini kapsar.** Bir `Run` girdisinin
  veya bir Startup klasörü dosyasının orada devre dışı bırakılıp bırakılmadığını ve ne zaman bırakıldığını
  söyler. Eşleşen bir `Run` değeri olmayan bir girdi "only in StartupApproved" ayrıntısıyla raporlanır.
  Bu, kaldırılmış bir otomatik başlatmayı gösterebilir.
- **`OverridesMachineRegistration` sistemin `SOFTWARE` kovanını gerektirir.** Bir kullanıcının COM sınıfı
  veya komutu onunla karşılaştırılır ve bu kovan, ana bilgisayarın `SYSTEM` kovanıyla aynı şekilde bulunur.
  Geçersiz kılma, COM ele geçirme (COM hijacking) kalıbıdır (T1546.015). OneDrive ve diğer kullanıcı başına
  kurulumlar da hiçbir şeyi geçersiz kılmayan kendi sınıflarını kaydeder.
- **`ImagePath` ve `Args`, komut satırından yalnızca ayırt edilebildikleri yerde ayrılır.**
  - Tırnak içindeki bir program, kapanış tırnağından ayrılır.
  - Klasörsüz bir ilk sözcük program olarak alınır (`cmd /c …`).
  - Aksi hâlde ayırma, bir program uzantısıyla biten en kısa başlangıçtan yapılır.

  Boşluk içeren tırnaksız bir yol, mevcutsa farklı bir program (`C:\Program.exe`) çalıştırabilir.
- **`VerifierDlls` ve `MonitorProcess` için `Enabled` değerini `GlobalFlag` belirler.** `VerifierDlls`
  yalnızca `0x100` ile yüklenir. `MonitorProcess` yalnızca `0x200` ve `SilentProcessExit\<program>`
  `ReportingMode` değerinin `0x1` biti ile çalışır. `2` (döküm) veya `4` (bildirim) modu hiçbir şey
  çalıştırmaz ve `Details` modun bayraklarını adlandırır.
- **`KeyLastWrittenUTC` değerin değil, anahtarın son yazılma zamanıdır.** Birkaç değer bunu paylaşır.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

Özet, kayıtları mekanizmaya göre sayar. Ayrıca `Run` değeri olmadan kalmış `StartupApproved\Run` ve
`Run32` girdilerini (Startup klasörü dosyalarının kaybedecek böyle bir değeri yoktur) ve makinenin
kayıtlarını geçersiz kılan kullanıcı COM sınıflarını ve komutlarını listeler.

Her `parse-registry` komutu gibi bu komut da, `-C` ile bile, okuduğu dosyalardan biri olan bir `-o`
değerini reddeder. Bu; kovanı, işlem günlüklerini ve sistemin SYSTEM ve SOFTWARE kovanlarını yol,
sembolik bağlantı veya sabit bağlantı yoluyla kapsar. Bunu ayırt edemediğinde yazmaz.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Ana bilgisayar bağlamı, kaynak kovan ve türü. |
| `User`, `UserSID` | Kullanıcı kovanları için: profil klasörü ve SID. SID, `UsrClass.dat` kök anahtar adından, aksi hâlde sistemin `ProfileList` kaydından gelir. |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | Uyan bir teknik varsa MITRE ATT&CK tekniği (`T1547.001`, `T1546.012`, …). |
| `KeyPath`, `ValueName` | Kovanın kökü altındaki anahtar (adlar yazıldığı gibi) ve değer (boş: varsayılan değer). |
| `Data` | Değerin metni veya bir listenin bir öğesi. Okunamadığında veya girdiyi yalnızca `StartupApproved` adlandırdığında boştur. |
| `ImagePath`, `Args` | Ayırt edilebildikleri yerde program veya DLL ve bağımsız değişkenler (yukarıya bakın). |
| `Enabled`, `DisabledAtUTC` | `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag` (`ReportingMode` ile birlikte), `UseFilter`, `IsInstalled`, zaman sağlayıcısının `Enabled` değeri veya `ScreenSaveActive` ne söylüyorsa. Hiçbiri bir şey söylemiyorsa boştur. |
| `Details` | Ek bilgiler: bekleyen bir yeniden adlandırmanın ne yaptığı, COM sınıfının adı, `ReportingMode`, `FilterFullPath`, bir Active Setup `ShellComponent` değeri, … |
| `OverridesMachineRegistration` | Bir kullanıcının COM sınıfı veya komutu için: makinenin aynısını kaydedip kaydetmediği. Sistemin `SOFTWARE` kovanı olmadan boştur. |
| `WOW64` | `Wow6432Node` altında: 32 bit programlar için. |
| `KeyLastWrittenUTC` | Anahtarın son yazılma zamanı. |
| `DataRaw` | Metin olmadıklarında veya geçerli UTF-16 olmadıklarında değerin baytları (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Yanlış değer türleri, okunamayan veriler, yanlış boyuttaki `StartupApproved` girdileri. |

## İnceleme stratejisi { #investigation-strategy }

1. Özetle başlayın:
   - makinenin kayıtlarını geçersiz kılan COM sınıfları ve komutlar;
   - `Run` değeri olmayan `StartupApproved` girdileri;
   - nadiren girdisi olan mekanizmalar (`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`).
2. LSA paketlerini, `BootExecute`, `KnownDLLs`, yazdırma izleyicilerini ve netsh yardımcılarını aynı
   Windows sürümünün temiz bir kurulumuyla karşılaştırın.
3. `ImagePath` değerini dosya sistemine göre çözümleyin ($MFT, karmalar ve imzalar) ve kullanıcı profili,
   `ProgramData` ve `Temp` yollarını kontrol edin.
4. `KeyLastWrittenUTC` değerini Security ve Sysmon kayıt defteri olayları, işlem oluşturma, Prefetch ve
   Amcache ile ilişkilendirin.
5. Bu anahtarlardan kaldırılmış otomatik başlatmaları bulmak için `parse-registry read --recover-deleted`
   komutunu kullanın.

## Daha fazla ayrıntı { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: Run and RunOnce registry keys](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
