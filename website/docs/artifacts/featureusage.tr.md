# FeatureUsage, RecentApps ve Görev Çubuğuna Sabitlenmiş Öğeler { #featureusage-recentapps-and-taskbar-pins }

## Bunlar nedir { #what-they-are }

Explorer, bir kullanıcının uygulamaları nasıl kullandığına dair birkaç kaydı kullanıcının `NTUSER.DAT` kovanında tutar:

- **FeatureUsage** (Windows 10 ve sonrası): kullanım türüne göre uygulama başına sayaçlar.
  - `AppSwitched`: kullanıcının uygulamanın penceresine kaç kez geçtiği (görev çubuğuna tıklama,
    Alt+Tab).
  - `AppLaunch`: görev çubuğundan (sabitlenmiş bir öğeden) kaç kez başlatıldığı.
  - `ShowJumpView`: atlama listesinin (jump list) kaç kez açıldığı (görev çubuğunda sağ tıklama).
  - `AppBadgeUpdated`: görev çubuğu rozetinin (okunmamış sayısı) kaç kez değiştiği.
  - `TrayButtonClicked`: her görev çubuğu düğmesine (`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …) kaç kez tıklandığı.

  Bir uygulama; bir uygulama kullanıcı modeli kimliği (`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`), ilk bölümü bilinen bir klasörün GUID'i olabilen bir yol
  (`{1AC14E77-…}\cmd.exe`, `%windir%\system32\cmd.exe` demektir) veya Explorer'ın bir uygulamayla
  eşleştiremediği pencereler için bir işlem kimliğidir (`*PID00001d74`).
- **RecentApps** (Windows 10, 1709 sürümüne kadar): kullanıcının yakın zamanda çalıştırdığı uygulamalar; başlatma
  sayısı ve son zamanla birlikte. Her birinin altında açtığı dosyalar bulunur (`RecentItems`).
- **Görev çubuğuna sabitlenmiş öğeler** (`Taskband`): görev çubuğuna sabitlenmiş programlar; kabuk öğeleri
  (`Favorites`) ve sabitlenmiş her kısayolun bağlantısı (`FavoritesResolve`) olarak. Windows 10 ve
  sonrasında her sabitlenmiş öğe, onu Windows'un mu (`SystemPinned`) yoksa kullanıcının mı (`UserPinned`) sabitlediğini belirtir.
- **ProgramsCache** (`StartPage`, `StartPage2`; Windows XP'den 7'ye kadar): Başlat menüsünün önbelleğe alınmış
  programları (`ProgramsCache`) ve Windows 7'de Başlat menüsüne ve görev çubuğuna sabitlenmiş olanlar
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`).

<span class="evidence-strength">En iyi sonuç:</span> bu kullanıcı bu uygulamayı bu şekilde bu
kadar kez kullanmıştır (FeatureUsage), bu zamanda çalıştırmıştır (RecentApps) veya sabitlemiştir.
Sayaçların kendilerine ait bir zamanı yoktur; yalnızca anahtarın son yazılma zamanı ve RecentApps zamanları
bir şeyin tarihini belirler.

## Konum { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

`FeatureUsage` anahtarının `KeyCreationTime` değeri, saymaya başladığı zamandır; özet bunu gösterir. Her profil için
`NTUSER.DAT` kovanını işlem günlükleriyle birlikte, ana bilgisayar ve kullanıcının SID'i için de sistemin `SYSTEM` ve
`SOFTWARE` kovanlarını toplayın.

## Analist için uyarılar { #analyst-cautions }

- **Zamansız sayılar.** Bir FeatureUsage sayısı ne zaman olduğunu değil, ne sıklıkta olduğunu söyler. Türe ait anahtarın
  son yazılma zamanı (`KeyLastWrittenUTC`), o türdeki herhangi bir sayıdaki en son değişikliktir.
- **Her çalıştırma sayılmaz.** `AppSwitched` başlatmaları değil, pencere geçişlerini sayar; penceresiz çalışan
  veya kullanıcının hiç geçiş yapmadığı bir uygulama sayılmaz. `AppLaunch` yalnızca görev çubuğundan yapılan
  başlatmaları sayar.
- **Paketlenmiş uygulamalar ve yollar.** Bir uygulama, Windows'un pencerelerini gruplandırdığı şekilde adlandırılır:
  paketlenmiş bir uygulamanın kimliği, bir programın kendisi için ayarladığı bir kimlik (`Chrome`, `VMware.Workstation.vmui`) veya bir
  yol. `AppPath`, bilinen bir klasörün GUID'inin o klasörün varsayılan yoluyla değiştirildiği
  yolu verir.
- **Varsayılan sabitlenmiş öğeler.** Yeni profiller varsayılan sabitlenmiş öğelerle gelir (File Explorer, Edge, Store); Windows 10
  ve sonrasında bunlar `SystemPinned` olarak görünür. `UserPinned` olan bir öğe kullanıcının tercihidir.
- **İki listede sabitlenmiş öğeler.** `Favorites` ve `FavoritesResolve` aynı sabitlenmiş öğeleri aynı
  sırayla (`Index`) tanımlar, ancak ayrı kayıtlar olarak verilir.
- **ProgramsCache yolları** görelidir: bir klasörü (`Programs\Accessories`) içindeki kısayollar
  (`Notepad.lnk`) izler.
- **Hasarlı değerler.** `REG_DWORD` olmayan bir sayı, `REG_QWORD` olmayan bir zaman, anlaşılamayan bir sabitlenmiş öğe
  veya bağlantı olmayan bir bağlantı baytlarını (`DataRaw`) korur ve `Problems` alanında raporlanır.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

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
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

Özet; kayıtları kaynağa göre, her kovanın FeatureUsage sayımına ne zaman başladığını, en çok geçiş yapılan
uygulamaları, son uygulamaları ve öğeleri (en yeniden başlayarak) ve görev çubuğuna sabitlenmiş öğeleri listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Ana bilgisayar, profil klasörünün adı, sistemin `ProfileList` verisinden kullanıcının SID'i ve kovan. |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked` (FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` veya `ProgramsCache`. |
| `App` | Bir FeatureUsage değerinin adı, son kullanılan bir uygulamanın `AppId` değeri veya sabitlenmiş bir öğenin uygulama kullanıcı modeli kimliği. |
| `AppPath` | Yol olarak `App`; bilinen bir klasörün GUID'i varsayılan yoluyla değiştirilmiştir. Kimlikler için boştur. |
| `Count` | Bir FeatureUsage sayısı veya son kullanılan bir uygulamanın `LaunchCount` değeri. |
| `LastAccessedUTC` | Son kullanılan bir uygulamanın veya öğenin `LastAccessedTime` değeri. |
| `ItemPath` | Son kullanılan bir öğenin `Path` değeri (veya son kullanılan bir uygulamanın `AppPath` değeri); sabitlenmiş bir öğenin veya önbelleğe alınmış bir programın kabuk öğesi yolu (`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`); bir `FavoritesResolve` bağlantısının yolu. |
| `DisplayName` | Son kullanılan bir öğenin `DisplayName` değeri. |
| `PinType` | Bir `Favorites` sabitlenmiş öğesi için `SystemPinned` veya `UserPinned`. |
| `Index` | Sabitlenmiş bir öğenin veya önbelleğe alınmış bir programın listesindeki yeri. |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | Kaydın geldiği değer ve anahtar ile anahtarın son yazılma zamanı. |
| `DataRaw` | Kodu çözülemeyen baytlar (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Yanlış türdeki değerler, anlaşılamayan sabitlenmiş öğeler veya listeler, bağlantı olmayan bağlantılar. |

## İnceleme stratejisi { #investigation-strategy }

1. Kullanıcının en çok çalıştığı uygulamalar için `AppSwitched` kayıtlarını `Count` değerine göre sıralayın ve
   yönetim, uzaktan erişim, arşivleme ve aktarım araçlarını ve kullanıcının yazabildiği yerlerdeki yolları
   (`\Users\`, `Downloads`, `Temp`) arayın.
2. Sabitlenmiş öğeleri kontrol edin: `UserPinned` olan bir araç veya kısayolu olağandışı bir yerde bulunan bir sabitlenmiş öğe.
3. Kullanımın tarihini belirlemek için RecentApps zamanlarını UserAssist, Prefetch, BAM ve Jump Lists ile birlikte kullanın.
4. `KeyCreationTime` değerini profilin oluşturulma zamanıyla karşılaştırın: sayımlar o zaman başlar.
5. Silinmiş sayaçlar ve sabitlenmiş öğeler için `parse-registry read --recover-deleted` kullanın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Application User Model IDs](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: Programs cache](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
