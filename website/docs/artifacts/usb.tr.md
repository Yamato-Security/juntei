# USB Aygıtları { #usb-devices }

## USB geçmişi nedir { #what-the-usb-history-is }

Windows kurduğu her aygıtı kaydeder. USB depolama, diğer USB aygıtları ve taşınabilir aygıtlar (telefonlar,
kameralar, medya oynatıcılar) için `SYSTEM` kovanı, her aygıt için adını, seri numarasını ve Windows'un onu
kurduğu, en son bağlandığını gördüğü ve en son çıkarıldığını gördüğü zamanları içeren bir anahtar tutar.
`MountedDevices`, birimlerine verilen birim GUID'lerini ve sürücü harflerini kaydeder. `SOFTWARE` kovanı,
Explorer'ın taşınabilir aygıtlara ve birimlere verdiği adları ve ReadyBoost'un birim seri numaraları
kaydını tutar. Her kullanıcının `NTUSER.DAT` dosyası, o kullanıcının bağladığı birimleri kaydeder.

`parse-registry usb` bunları aygıt ve birim başına tek bir kayıtta birleştirir: hangi aygıt olduğu, ne
zaman kurulduğu ve bağlandığı, hangi sürücü harfine ve birime sahip olduğu ve o birimi hangi kullanıcıların
bağladığı.

<span class="evidence-strength">En iyi sonuç:</span> bu satıcı, ürün ve seri numarasına sahip bir aygıt
bu sisteme ilk ve son olarak bu zamanlarda bağlanmıştır; biriminin sürücü harfi buydu; bu kullanıcılar onu
bağlamıştı. Kayıt defteri kayıtları hangi dosyaların kopyalandığını göstermez.

## Konum { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| Aygıt özelliği | Alan | Anlamı (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: sürücüsünün en son kurulduğu zaman |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: aygıtın ilk kurulduğu zaman |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: en son bağlandığı zaman (Windows 8 ve sonrası) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: en son çıkarıldığı zaman (Windows 8 ve sonrası) |

Windows 8 ve sonrası her zamanı `0064` … `0067` anahtarının varsayılan değeri olarak tutar. Windows 7 ise
`00000064\00000000\Data` içinde tutar. Juntei her ikisini de okur.

Birçok DFIR kaynağı `0064` değerini ilk kurulum, `0065` değerini ise kurulum olarak etiketler. Juntei
yukarıdaki Microsoft `devpkey.h` tanımlarını izler ve sütunları her zamanın neyi kaydettiğine göre adlandırır.

`-f` veya `-d` seçeneğine `SYSTEM` kovanını verin. Sistemin `SOFTWARE` kovanı ve kullanıcılarının
`NTUSER.DAT` kovanları, bir toplamanın onları tuttuğu şekilde (`Windows\System32\config\SOFTWARE`,
`Users\<user>\NTUSER.DAT`) veya kovanlar birlikte dışa aktarıldıysa aynı dizinde ve bir alt düzeyde, onun
yanında bulunur. Hepsini işlem günlükleriyle birlikte toplayın.

## Kayıtlar nasıl birleştirilir { #how-the-records-are-joined }

- **Aygıtlar:** aygıt anahtarı başına bir kayıt: `usbstor`, `usb` ve `wpdbusenum` (`DeviceClass`). Bir USB
  depolama aygıtı, satıcı ve ürün kimliklerini aynı kapsayıcı kimliğine (container ID) (yoksa aynı seri
  numarasına) sahip USB aygıtından alır.
- **Birimler:** bir USB depolama aygıtının birimleri, verisi aygıtı adlandıran bir yol olan `MountedDevices`
  `\??\Volume{GUID}` değerleridir. İki birimi olan bir aygıtın iki kaydı vardır. Sürücü harfleri, aynı veriye
  sahip `\DosDevices\X:` değerleridir.
- **Adlar ve seri numaraları:** birim adları `Windows Portable Devices` kaydından ve aynı aygıtı adlandıran
  taşınabilir aygıtlardan gelir. Birim etiketleri ve seri numaraları, onu adlandıran `EMDMgmt`
  anahtarlarından gelir.
- **Kullanıcılar:** `MountPoints2` anahtarında birimin GUID'i bulunan kullanıcılar, o anahtarın son yazılma
  zamanıyla birlikte.
- **Aygıt anahtarı olmayan birimler** (`volume`): hiçbir aygıt anahtarının adlandırmadığı bir
  `MountedDevices` birimi (dahili bir diskin veya `Enum` anahtarı kaybolmuş, çıkarılmış bir aygıtın), hiçbir
  birim GUID değeriyle veri paylaşmayan bir sürücü harfi ve yalnızca kullanıcıların `MountPoints2`
  anahtarlarında bulunan bir birim.
- **Yalnızca `SOFTWARE` kovanının adlandırdığı aygıtlar** (`portable_device`, `readyboost`).

`SourceKeys`, bir kaydın birleştirildiği her anahtarı ve değeri kovanıyla birlikte listeler.

## Analist için uyarılar { #analyst-cautions }

- **Sürücü harfleri yeniden kullanılır.** `MountedDevices`, her harfin verildiği son aygıtı tutar. Bir harf
  yalnızca verisi hâlâ o harfin verisi olan aygıtta gösterilir.
- **Seri numaraları uydurulmuş olabilir.** Bir örneğin (instance) ikinci karakteri `&` olduğunda, aygıt seri
  numarası bildirmediği için Windows onu uydurmuştur; bu durumda `Serial` boştur. Böyle bir örnek başka bir
  bilgisayarda farklı olabilir.
- **USB depolama örnekleri mantıksal birimle biter** (`&0`). `Serial` bunu dışarıda bırakır; `Instance`
  korur.
- **Zamanlar Windows sürümüne göre değişir.** Windows 7'de yalnızca kurulum zamanları bulunur.
  `LastArrivalUTC` ve `LastRemovalUTC` yalnızca son bağlantıdır, her bağlantı değil. Olay günlükleri
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`) ve
  `setupapi.dev.log` daha fazlasını verir.
- **Denetim kümeleri:** varsayılan olarak Windows'un en son kullandığı denetim kümesi okunur.
  `--all-control-sets` her denetim kümesini okur; daha eski bir küme, geçerli kümede artık bulunmayan bir
  aygıtı hâlâ listeliyor olabilir.
- **Bir anahtarın son yazılma zamanı bir bağlantı zamanı değildir.** Windows aygıt anahtarlarını birçok
  nedenle yeniden yazar.
- **Kaldırılan aygıtlar iz bırakır.** Satıcı ve ürün bilgisi olup aygıt anahtarı olmayan bir birim kaydı,
  `Enum` anahtarı (bir kaldırma işlemi veya temizlik araçlarıyla) kaldırılmış, ancak birimi `MountedDevices`
  içinde hâlâ bulunan bir aygıttır.
- **ReadyBoost seri numaraları**, LNK dosyalarının ve Jump Lists'in de kaydettiği birimin seri numarasıdır
  (`VolumeSerials`, `dir` komutunun gösterdiği biçimde).

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

Özet, en son bağlanan USB depolama aygıtlarını (en yeniden başlayarak) sürücü harfleriyle birlikte ve aygıt
anahtarı olmayan USB depolama aygıtlarına ait kaç birim olduğunu listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host` | `SYSTEM` kovanından alınan bilgisayar adı. |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum` veya aygıt anahtarı yoksa `volume`, `portable_device` ya da `readyboost`. |
| `ControlSet` | Aygıt anahtarının denetim kümesi. |
| `Vendor`, `Product`, `Revision` | USB depolama: aygıtın adından (`Disk&Ven_…&Prod_…&Rev_…`); bir birimin veya `SOFTWARE` anahtarının adlandırdığı bir aygıt için de. |
| `Vid`, `PID` | USB satıcı ve ürün kimlikleri. |
| `Serial`, `Instance` | Seri numarası ve yazıldığı biçimde örnek anahtarının adı. |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | Yazıldığı biçimde aygıt anahtarının değerleri (`DeviceDesc` çoğunlukla `@<inf>,%…%;<text>` biçimindedir). Taşınabilir bir aygıtın `FriendlyName` değeri genellikle biriminin etiketi veya sürücüsüdür. |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | Aygıt özelliği zamanları (yukarıya bakın). |
| `DriveLetters` | `MountedDevices` verisi birimin verisiyle aynı olan sürücü harfleri. |
| `VolumeGUID`, `VolumeData` | Birimin GUID'i ve `MountedDevices` verisinin birimin nerede olduğunu söylediği: bir aygıt yolu, bir MBR disk imzası ve bölüm ofseti veya bir GPT bölümü. |
| `VolumeNames`, `VolumeSerials` | Explorer'ın aygıta veya birimine verdiği adlar, ReadyBoost etiketleri ve birim seri numaraları (`1234-ABCD`). |
| `User` | `MountPoints2` anahtarında birim bulunan kullanıcılar, o anahtarın son yazılma zamanıyla birlikte (JSON: `User`, `MountPoints2LastWritten`, `SourcePath`). |
| `FoundIn`, `SourceKeys` | Nerede bulunduğu (`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`) ve birleştirildiği her anahtar ve değer. |
| `KeyPath`, `KeyLastWrittenUTC` | Aygıtın örnek anahtarı ve son yazılma zamanı. |
| `SourcePath`, `SoftwarePath` | `SYSTEM` kovanı ve bir şey sağladığında `SOFTWARE` kovanı. |
| `TransactionLogStatus`, `LogsApplied` | Kaydın birleştirildiği kovanların kayıt defteri kurtarma bağlamı. |
| `Problems` | Okunamayan anahtarlar ve FILETIME olmayan aygıt özellikleri (baytlarıyla birlikte). |

## İnceleme stratejisi { #investigation-strategy }

1. USB depolama aygıtlarını `LastArrivalUTC` (yoksa `DriverLastInstalledUTC`) alanına göre sıralayın ve
   bilgisayarın kendi okuyucularını ve hub'larını ayırın.
2. Her aygıtın satıcısını, ürününü, seri numarasını, birim GUID'ini, sürücü harfini ve birim seri numarasını
   not edin ve bunları LNK dosyalarında, Jump Lists'te, ShellBags'te ve `$MFT` içinde (o sürücü harfindeki veya
   birimdeki yollar) arayın.
3. Birimi kimin ve ne zaman bağladığını (`MountPoints2` anahtarının son yazılma zamanı) görmek için `User`
   alanını kontrol edin.
4. Bir USB depolama aygıtını adlandıran `volume` kayıtlarını arayın: anahtarları kaldırılmış aygıtlar.
5. Yalnızca sonuncusunu değil, her bağlantıyı görmek için olay günlükleriyle ilişkilendirin.
6. Silinmiş aygıt anahtarları için `parse-registry read --recover-deleted` komutunu kullanın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Device properties (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: Supporting mount manager requests](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (Wine başlık dosyalarındaki hâliyle)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): aygıt özelliği numaraları
- [libyal winreg-kb: USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md),
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) ve
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

Kullanıcı birleştirmeleri her kovanın kaynak kimliğini kullanır; bu nedenle aynı profil adına sahip dışa
aktarılmış kovanlar ayrı kalır. `SourceKeys`, katkıda bulunan kullanıcı kovanı yollarını korur. Eşlik eden
kullanıcı kovanlarının bulunması, özyinelemeli kovan bulmayla tutarlı olarak sembolik bağlantıları atlar.
VID/PID ilişkilendirmesi, eşleşen kullanılabilir bir kapsayıcı kimliği gerektirir; böyle bir kimlik yoksa,
seri numarası eşleştirmesi belirsizliğe yer bırakmayan kimlikler vermelidir. Çelişen adaylar bir sorun
kaydıyla birlikte bilinmeyen olarak kalır.
