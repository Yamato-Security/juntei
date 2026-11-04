# Uygulama Geçmişi { #application-history }

## Araçların geride bıraktıkları { #what-tools-leave-behind }

Birçok üçüncü taraf araç, ayarlarını ve geçmişlerini kullanıcının `NTUSER.DAT` dosyasında tutar. Bunların
birçoğu saldırganların kullandığı ya da verileri dışarı çıkarmadan önce hazırlamak için başvurduğu araçlar
arasındadır: Sysinternals araçları, arşivleyiciler, bulut eşitleme istemcileri ve uzaktan erişim araçları.
Juntei şunları okur:

| Uygulama | `HKCU\Software` altındaki anahtar | Gösterdiği |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | Kullanıcının lisansını kabul ettiği (`EulaAccepted`) her araç: PsExec, ProcDump, Sysmon, … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | Açılan arşivler; iletişim kutularına yazılan arşiv adları ve ayıklama klasörleri |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | Oluşturulan arşivler; ayıklama klasörleri; dosya yöneticisinde gezinilen ve kopyalama hedefi olarak kullanılan klasörler |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | File menüsündeki arşivler, ayıklama klasörleri, son arşivler |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | Her hesabın e-posta adresi, yerel klasörü ve adları; eşitlenen kitaplıklar (`ScopeIdToMountPointPathCache`) ve klasörler (`Tenants\<tenant>`) |
| Adobe Acrobat ve Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | En son açılan PDF dosyaları ve klasörler; dosya adları, boyutları ve tarihleriyle |
| TeamViewer | `TeamViewer` (ve eski bir sürümün `Version<N>` anahtarı) | Tüm değerler: hesap ve kullanıcı adları, dosya aktarımı başlangıç klasörleri (`<partner ID>?<local>|<remote>`), … |
| CCleaner | `Piriform\CCleaner` | Tüm değerler: neyi temizlemek üzere ayarlandığı, `UpdateKey`, … |

<span class="evidence-strength">En iyi sonuç:</span> bu kullanıcı aracı çalıştırmıştır (veya lisansını
kabul etmiştir) ya da araç bu arşivleri ve klasörleri açmış, oluşturmuş veya ayıklamıştır ya da bu bulut
hesabı kullanıcı için kurulmuştur. Geçmişlerin çoğunda girdi başına bir zaman yoktur: yalnızca en son girdi,
ait olduğu liste anahtarının son yazılma zamanını alır.

## Konum { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

Her kullanıcının `NTUSER.DAT` dosyasını işlem günlükleriyle birlikte, kullanıcıların SID'leri için de
sistemin `SOFTWARE` kovanını toplayın.

## Analist için uyarılar { #analyst-cautions }

- **Sysinternals EULA kabulü.** `FirstUsedUTC`, anahtarın tek değeri `1` olarak ayarlanmış okunabilir bir
  `EulaAccepted` olduğunda, anahtarın son yazılma zamanından elde edilen bir ilk kullanım tahminidir.
  Reddedilmiş veya hasarlı bir EULA bu tahmini almaz. Kayıt defteri ayarları önceden yapılandırılmış,
  kopyalanmış veya düzenlenmiş de olabilir: yürütmeyi ve zamanını diğer artefaktlarla doğrulayın.
  Ayarlarını burada tutan araçlar (Process Explorer, Autoruns, …) anahtarı daha sonra yeniden yazar:
  bu durumda `FirstUsedUTC` boştur ve `KeyLastWrittenUTC` yalnızca son ayar değişikliğidir. `-accepteula`
  ile ya da başka bir kullanıcı olarak çalıştırılan bir araç (PsExec'in uzak tarafı SYSTEM olarak çalışır),
  o kullanıcının anahtarına yazar.
- **Zamanlar değil, konumlar.** Liste girdileri sıralıdır (`MruPosition` 0 en sonuncusudur), ancak yalnızca
  liste anahtarının son yazılması en son girdiyi tarihlendirir. Daha eski girdiler daha önce, bilinmeyen bir
  zamanda kullanılmıştır.
- **Yazılan adlar.** WinRAR'ın `ArcName` değeri, arşiv oluşturulurken yazılan arşiv adıdır: çoğunlukla
  klasörsüz bir dosya adıdır.
- **Adobe yolları** Adobe'nin yazdığı biçimde tutulur (`/C/Users/alice/a.pdf`,
  `C:\Users\alice\a.pdf` anlamına gelir). `sDate` değeri (`Details` içinde) bir PDF tarih dizesidir ve
  yazıldığı gibi tutulur.
- **CCleaner'ın `UpdateKey`** değeri saat dilimi olmayan yerel saattir; yazıldığı gibi tutulur.
- **TeamViewer.** Bağlantı günlükleri kayıt defteri değerleri değil dosyalardır
  (`Connections_incoming.txt`, `TeamViewer*_Logfile.log`): bunları da toplayın.
- **Yokluk.** Kullanıcılar ve araçlar bu geçmişleri temizleyebilir (CCleaner'ın kendisi de bunu yapabilir).

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

Özet; en yeni Sysinternals ilk kullanım tahminlerini, her listenin en son kullandığı arşivleri ve
klasörleri ve OneDrive hesaplarını listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath` | Ana bilgisayar ve `NTUSER.DAT`. |
| `User`, `UserSID` | Profil klasörünün adı ve sistemin `ProfileList` kaydından alınan SID'si. |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`. |
| `Item` | Sysinternals aracı, liste (`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …), OneDrive hesabı veya eşitlenen klasörlerinin anahtarı ya da değerin adı. |
| `MRU-Position` | Girdinin listesindeki konumu; 0 en sonuncusudur. |
| `Path` | Arşiv, klasör veya belge. |
| `Value` | `EulaAccepted`, bir OneDrive hesabının e-posta adresi veya eşitlenen bir kitaplığın kimliği, bir TeamViewer veya CCleaner değeri. |
| `FirstUsedUTC` | EULA kabulünden elde edilen ilk kullanım tahmini (yukarıdaki uyarılara bakın). |
| `Details` | Bir OneDrive hesabının `DisplayName`, `UserCID`, `ConfiguredTenantId`, … değerleri; bir Adobe belgesinin `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate` değerleri. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | Kaydın nereden geldiği ve o anahtarın en son ne zaman değiştiği. |
| `DataRaw` | Kodu çözülemediğinde değerin baytları (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Kodu çözülemeyen veya okunamayan veriler. |

## İnceleme stratejisi { #investigation-strategy }

1. Sıradan bir kullanıcının çalıştırmadığı Sysinternals araçlarını (PsExec, ProcDump, PsLoggedOn,
   SDelete) ve bunların EULA zaman damgalarını arayın.
2. Olay civarındaki arşiv adlarını ve ayıklama klasörlerini gözden geçirin: veriler sistemden çıkmadan
   önce `ProgramData`, `Temp` veya bir kullanıcının klasöründe oluşturulan arşivler.
3. Kuruma ait olmayan OneDrive hesaplarını ve TeamViewer hesap adlarını ve dosya aktarımı klasörlerini
   kontrol edin.
4. Prefetch, Amcache, BAM ve UserAssist (aracın çalışması), `$MFT` ve USN Journal (arşivin dosyası), LNK
   dosyaları ve Jump Lists (açılan belgeler) ve olay günlükleriyle ilişkilendirin.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
