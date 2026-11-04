# Uygulama Uyumluluğu { #application-compatibility }

## Windows neleri saklar { #what-windows-keeps }

ShimCache (`parse-registry shimcache`) ve Amcache (`parse-registry amcache`) dışında,
Windows'un uygulama uyumluluğu özellikleri şunları
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags` altında saklar:

| Kaynak | Kovan ve anahtar | Gösterdiği |
|---|---|---|
| `layers` | `SOFTWARE` (ve `Wow6432Node`) ile `NTUSER.DAT` `Layers` | Uyumluluk katmanlarıyla çalışacak şekilde ayarlanmış programlar: `RUNASADMIN` (her zaman yönetici olarak), `WIN7RTM`, `HIGHDPIAWARE`, … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 ve sonrası) | Program Uyumluluk Yardımcısı'nın (Program Compatibility Assistant) kullanıcının çalıştırdığını gördüğü programlar |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | Aynısı, Windows 7'de |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | İzlenen bir modülü yükleyen programlar ve bunu en son ne zaman yaptıkları |
| `cit_system` | `SOFTWARE` `CIT\System` (Windows 7'den ilk Windows 10 sürümlerine kadar) | Sıkıştırılmış CIT veritabanlarındaki program yolları ve komut satırları |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | `sdbinst` ile yüklenen özel shim veritabanları: dosya, açıklama, yükleme zamanı |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | Her programa uygulanan özel shim veritabanları ve yükleme zamanları |

<span class="evidence-strength">En iyi sonuç:</span> katmanlar ve shim veritabanları, bir programın
nasıl çalıştığını değiştiren yapılandırmalardır; Program Uyumluluk Yardımcısı ve CIT girdileri
bir programın bu kullanıcı olarak veya bu sistemde çalıştığını (ya da bir modül yüklediğini) gösterir.

Özel shim veritabanları bir kalıcılık ve enjeksiyon tekniğidir (MITRE ATT&CK T1546.011): bir shim,
program her çalıştığında programa bir DLL yükleyebilir veya başka bir programı başlatabilir.

## Konum { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

`SOFTWARE` kovanını, tüm `NTUSER.DAT` kovanlarını ve bunların işlem günlüklerini ve bir shim
veritabanının yolunun adlandırdığı `.sdb` dosyalarını (genellikle `C:\Windows\AppPatch\Custom\`) toplayın.

## Analist için uyarılar { #analyst-cautions }

- **CIT genişletmesi sınırlıdır.** Sıkıştırılmamış boyutu 64 MiB'tan fazla olarak bildirilen değerler
  bir tanılama iletisiyle birlikte ham bayt olarak korunur. Bozuk sıkıştırma ve boyut uyuşmazlıkları raporlanır.


- **Program Uyumluluk Yardımcısı girdileri için çalışma zamanı yoktur.** `Store` verisi (`SACP`,
  `DataRaw` içinde tutulur) tek bir zaman içerir; ancak Windows 8.1, 10 ve 11'de bu zaman bir kovandaki
  tüm programlar için aynıdır, diğer sistemlerde de programın çalıştığı zaman değildir. En son değişikliğin
  tarihini yalnızca `Store` anahtarının son yazılma zamanı verir.
- **Çıkarılabilir ortamdaki programlar** `SIGN.MEDIA=<id> <path>` olarak adlandırılır: `Path` ortamdaki
  yoldur, `Details` ise ortam kimliğini içerir.
- **CIT veritabanları** LZNT1 ile sıkıştırılmıştır ve yapıları kamuya açık olarak belgelenmemiştir: Juntei
  her birinde bulunan program yollarını ve komut satırlarını, veritabanı başına bir kez, sayı veya zaman
  olmadan listeler. Bir komut satırı kendi başına bir kayıttır, programına bağlı değildir. Windows 10 ve
  sonrası burada az veri tutar.
- **Shim veritabanları.** Her `custom_sdb` ve `installed_sdb` kaydını kontrol edin: meşru yazılımlar
  nadiren böyle bir veritabanı yükler. Shim'in ne yaptığını `.sdb` dosyasının kendisi söyler.
- Bir kullanıcının ayarladığı **katmanlar** yalnızca o kullanıcıya uygulanır; `SOFTWARE` içindekiler herkese.
  `RUNASADMIN`, programın her seferinde yükseltme istemesine neden olur.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

Özet; özel shim veritabanlarını, katman atanmış programları ve CIT modüllerinin son
kullanımlarını listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Ana bilgisayar, kovan dosyası ve `SOFTWARE` veya `NTUSER.DAT`. |
| `User`, `UserSID` | Bir kullanıcı kovanının profil klasörü adı ve sistemin `ProfileList` verisinden alınan SID'i. |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb`; `machine` veya `user`. |
| `Path` | Program (değerin adı), bir CIT veritabanındaki yol veya komut satırı ya da bir shim veritabanının uygulandığı program. |
| `Layers` | Yazıldığı hâliyle uyumluluk katmanları (`~ RUNASADMIN`). |
| `LastSeenUTC` | Bir programın bir CIT modülünü en son ne zaman yüklediği. |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | Bir shim veritabanının GUID'i, dosyası ve yükleme zamanı. |
| `Details` | Bir CIT modülünün adı, bir CIT veritabanının değer adı, bir programın çıkarılabilir ortam kimliği (`SignMedia`), bir shim veritabanının `DatabaseDescription` ve `DatabaseType` değerleri. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | Kaydın nereden geldiği, o anahtarın en son ne zaman değiştiği ve `Wow6432Node` altında olup olmadığı. |
| `DataRaw` | `Store` değerinin `SACP` verisi veya olması gerektiği gibi olmayan veriler (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Kodu çözülemeyen veya okunamayan veriler. |

## İnceleme stratejisi { #investigation-strategy }

1. Her özel shim veritabanını inceleyin: `.sdb` dosyasını, yükleme zamanını ve uygulandığı
   programları; Prefetch, Amcache ve işlem oluşturma olaylarındaki `sdbinst.exe` ile ilişkilendirin.
2. Program Uyumluluk Yardımcısı girdileri arasında kullanıcının yazabildiği klasörlerdeki programları
   ve çıkarılabilir ortamdan çalışan programları arayın.
3. Olağandışı programlardaki `RUNASADMIN` ve diğer katmanları kontrol edin.
4. Yürütme zamanları için ShimCache, Amcache, BAM, Prefetch ve UserAssist ile ilişkilendirin.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Uygulama uyumluluğu veritabanı](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
