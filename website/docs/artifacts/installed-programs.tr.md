# Yüklü Programlar { #installed-programs }

## Windows neleri saklar { #what-windows-keeps }

Windows ve yükleyiciler, yüklü yazılımları birkaç kayıt defteri listesine kaydeder. Her listede program, ürün,
dağıtım veya paket başına bir anahtar bulunur; Juntei her anahtar için bir kayıt verir.

| Kaynak | Kovan ve anahtar | İçerdiği |
|---|---|---|
| `uninstall` | `SOFTWARE` ve `NTUSER.DAT` `…\CurrentVersion\Uninstall\*` ile `SOFTWARE\Wow6432Node\…` | Ayarlar'ın ve *Programlar ve Özellikler*'in listelediği bilgiler: ad, sürüm, yayımcı, yükleme tarihi, klasörler, kaldırma komutları, boyut, gizli olup olmadığı |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | Windows Installer ürünleri: ürün adı ve sürümü, paketin dosya adı ve yüklendiği klasör |
| `app_path` | `SOFTWARE` ve `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | Windows'un adıyla başlattığı programlar (`Win+R`, `start tool`): programın yolu ve klasörü |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | Windows Subsystem for Linux dağıtımları: ad, dosyalarının veya diskinin nerede olduğu, WSL 1 veya 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | Paketlenmiş (Store) uygulamalar: tüm kullanıcılar için sağlananlar, Windows'un kendi uygulamaları, hazırlananlar (staged), sağlaması kaldırılanlar (deprovisioned) ve her kullanıcınınkiler |

<span class="evidence-strength">En iyi sonuç:</span> bu program, ürün veya paket sisteme tüm
kullanıcılar için veya bu kullanıcı için yüklenmiştir (ya da kaydedilmiştir). Yükleme tarihi yükleyicinin
yazdığı değerdir; anahtarın son yazılma zamanı ise girdinin en son ne zaman değiştiğidir.

## Konum { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

`SOFTWARE` kovanını, tüm `NTUSER.DAT` kovanlarını ve bunların işlem günlüklerini toplayın. `SYSTEM` ana bilgisayarı
adlandırır; `SOFTWARE` içindeki `ProfileList` ise her kullanıcının SID'ini verir.

## Analist için uyarılar { #analyst-cautions }

- **Yükleme tarihi ve anahtar zamanı.** `InstallDate`, yükleyicinin yerel saatle yazdığı bir tarihtir (saat içermez);
  bazı yükleyiciler bunu yazmaz, bazıları da başka bir biçimde yazar (`InstallDateRaw` içinde yazıldığı gibi
  korunur). Bir güncelleme tarihi değiştirmeden girdiyi yeniden yazabilir veya program pek değişmeden tarihi
  değiştirebilir: `InstallDate` ile `KeyLastWrittenUTC` değerlerini karşılaştırın.
- **Kaldırılmış yazılımlar.** Düzgün biçimde kaldırılan bir program anahtarlarını kaybeder. Silinmiş `Uninstall`
  anahtarları için `parse-registry read
  --recover-deleted`, geride kalan izler için de Amcache, Prefetch ve `$MFT` kullanın.
- **Kullanıcı başına kayıtlar.** `NTUSER.DAT` girdileri bir profil için kaydedilmiş yazılımları belirtir.
  Bu kapsam, yükleyiciyi kimin çalıştırdığını veya yönetici haklarının kullanılıp kullanılmadığını göstermez.
- **Gizli girdiler.** `SystemComponent` bir girdiyi *Programlar ve Özellikler*'den gizler; `DisplayName`
  içermeyen girdiler (güncellemeler, bileşenler) de gösterilmez. Bunların tümü raporlanır.
- **Ürün kodları.** GUID biçimli bir `Uninstall` anahtarı, `ProductCode` aracılığıyla paketlenmiş bir
  `msi_product` anahtarıyla eşleştirilebilir. GUID biçimli ad tek başına girdinin
  Windows Installer tarafından oluşturulduğunu kanıtlamaz.
- **App Paths.** Bunlar yürütme kayıtları değil, programları bulmak için kullanılan
  kayıtlardır. `Path` değeri birden fazla arama dizini içerebilir ve yükleme dizini olmak
  zorunda değildir.
- **Yayımcı.** Bu, yükleyicinin sağladığı kayıt defteri meta verisidir; doğrulanmış imza
  bilgisi değildir.
- **Mimari.** `Wow6432Node`, 64 bit Windows'ta 32 bit programları tutar (`Architecture` `x86`,
  `Wow64` `True`); diğer girdiler sistemin kendi mimarisindedir (boş). Bir paketin
  mimarisi kendisine aittir (`x64`, `neutral`, …).
- **WSL.** `Version`, dağıtımın `Flags` değerinden alınan WSL sürümüdür (`0x8`: WSL 2'nin
  sanal makinesinde çalışır). Dağıtımın kendi `Version` değeri (`Details` içinde) dosya sisteminin
  sürümüdür ve WSL 1 dağıtımlarında da 2'dir. `BasePath`, dağıtımın `ext4.vhdx` (WSL 2) veya `rootfs`
  (WSL 1) konumunu tutar: içine bakmak için bunu toplayın.
- **Paketler.** `<SID>` listeleri her kullanıcı için kaydedilmiş paketlerdir; çoğu Windows'un
  kendi paketleridir. `Deprovisioned` paketler yeni kullanıcılar için, `EndOfLife\<SID>` paketleri ise o kullanıcı için kaldırılmıştır.
- **Hasarlı değerler.** Bozuk sayılar, yanlış değer türleri ve geçersiz UTF-16 `Problems` alanında
  raporlanır ve `DataRaw` içinde korunur; sıfır bayt uzunluğunda mevcut bir değer için boş bir onaltılık dize de buna dahildir.
  İsteğe bağlı sayısal bir alandaki temiz boş dize, bulunmuyor olarak değerlendirilir. Bir paket
  sürümü, her biri işaretsiz 16 bit aralığında olan dört ondalık bileşen içermelidir.
  `-t` kayıtları stdout'a aktarırken kovan düzeyindeki uyarılar stderr'de kalır.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

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
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

Özet; en son yüklenen programları (`InstallDate` değerine göre), kullanıcıların kendileri için yüklediklerini
ve WSL dağıtımlarını listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Ana bilgisayar, kovan dosyası ve `SOFTWARE` veya `NTUSER.DAT`. |
| `User`, `UserSID` | Bir kullanıcı kovanının kullanıcısı (profil klasörü ve `ProfileList` SID'i) veya bir paketin listelendiği kullanıcı. |
| `Source`, `Scope` | Liste (`uninstall`, `msi_product`, `app_path`, `wsl`, `package`) ve `machine` veya `user`. |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher`; bir ürünün `ProductName` ve `Version` değerleri; bir App Path'in program dosyası adı; bir dağıtımın adı ve WSL sürümü; bir paketin adı ve sürümü. |
| `InstallDate`, `InstallDateRaw` | `YYYYMMDD` olarak yazıldığında tarih olarak `InstallDate`; aksi hâlde yazıldığı gibi. |
| `KeyLastWrittenUTC` | Girdinin anahtarının en son ne zaman değiştiği. |
| `InstallLocation`, `InstallSource` | `InstallLocation` (bir App Path'in `Path` değeri, bir dağıtımın `BasePath` değeri, bir paketin `Path` değeri); `InstallSource` (bir ürünün en son kullanılan kaynak klasörü). |
| `Program` | Bir App Path'in programı. |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`. |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | KiB cinsinden `EstimatedSize`; gizli olup olmadığı; Windows Installer tarafından yüklenip yüklenmediği. |
| `ProductCode` | Windows Installer ürün kodu (`{GUID}`). |
| `Architecture`, `WOW64` | `Wow6432Node` altında `x86` veya bir paketin mimarisi; anahtarın `Wow6432Node` altında olup olmadığı. |
| `Details` | Ek bilgiler: `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User` (bir Inno Setup yükleyicisini çalıştıran hesap); bir ürünün `PackageCode`, `PackageName` değerleri; bir dağıtımın `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName`, `Default` değerleri; bir paketin `List`, `ResourceId`, `PublisherId` değerleri. |
| `KeyPath` | Girdinin anahtarı. |
| `DataRaw` | Adlarının söylediği gibi olmayan değerlerin baytları (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Okunamayan veya adlarının söylediği gibi olmayan değerler. |

## İnceleme stratejisi { #investigation-strategy }

1. Olay zamanı civarında `InstallDate` ve `KeyLastWrittenUTC` değerlerine göre sıralayın; uzaktan erişim
   araçlarına, tünelleme ve dosya aktarım istemcilerine, arşivleyicilere, betik çalışma ortamlarına ve güvenlik araçlarına bakın.
2. `Scope` değeri `user` olan kayıtları inceleyin: yönetici hakları olmadan yüklenmiş yazılımlar.
3. `InstallLocation`, `InstallSource` ve `UninstallCommand` değerlerinde kullanıcının yazabildiği klasörleri
   (`Downloads`, `AppData`, `Temp`, `ProgramData`) kontrol edin.
4. WSL dağıtımlarını arayın ve bunların `BasePath` konumlarını toplayın.
5. Amcache ve Prefetch (yükleyici ve çalışan program), `$MFT` (yükleme klasörünün
   oluşturulması) ve Application olay günlüğü (MsiInstaller olayları) ile ilişkilendirin.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Uninstall kayıt defteri anahtarı](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: Uygulama kaydı (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: Paket kimliğine genel bakış](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
