# Önyükleme Yapılandırma Verileri (BCD) { #boot-configuration-data-bcd }

## BCD deposu nedir { #what-the-bcd-store-is }

Windows Vista ve sonraki sürümler önyükleme yapılandırmalarını bir kayıt defteri kovanında, yani BCD
deposunda tutar: BIOS sistemlerinde `\Boot\BCD`, UEFI sistemlerinde ise EFI sistem bölümündeki
`\EFI\Microsoft\Boot\BCD`. `bcdedit` ve Windows yükleyicisi bu depoyu düzenler; önyükleme yöneticisi
onu her önyüklemede okur.

Depo, her biri bir GUID ile adlandırılan nesneler içerir:

- varsayılan girdiyi, görüntüleme sırasını ve menü zaman aşımını içeren **Windows önyükleme
  yöneticisi** (`{bootmgr}`); UEFI sistemlerinde ayrıca bellenimin önyükleme sırasını içeren
  **bellenim önyükleme yöneticisi** (`{fwbootmgr}`);
- her Windows kurulumu (kurtarma ortamı dahil) için bir **Windows önyükleme yükleyicisi**, bir
  **sürdürme uygulaması** (hazırda bekletme) ve **bellek test aracı**;
- ayarları diğer nesneler tarafından devralınan **devralınan ayarlar** nesneleri
  (`{globalsettings}`, `{bootloadersettings}`, …);
- **aygıt** nesneleri (kurtarma ortamının ramdisk seçenekleri).

Her nesnenin öğeleri vardır: bir açıklama, bir aygıt ve yol ile `recoveryenabled`,
`bootstatuspolicy`, `testsigning`, `nointegritychecks` ve `safeboot` gibi ayarlar.

Bunları iki tür saldırı değiştirir:

- **Fidye yazılımları**, sistemin onarılamaması için kurtarmayı kapatır:
  `bcdedit /set {default} recoveryenabled No` ve
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`. Bazı aileler, Windows'un güvenlik
  ürünlerinin çalışmadığı güvenli modda yeniden başlaması için `safeboot` ayarını yapar.
- **Rootkit'ler ve imzasız sürücüler**, önyükleme zamanı denetimlerini zayıflatmak için
  `testsigning on` veya `nointegritychecks on` kullanabilir. Bu ayarlar tek başına bir rootkit'in
  çalıştığını kanıtlamaz.

<span class="evidence-strength">En iyi sonuç:</span> toplama anında bu önyükleme girdisinde bu ayar
vardı. Öğe anahtarının son yazma zamanı, ayarın en son yazıldığı zamandır.

## Konum { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

Bir öğenin türü 8 onaltılık basamaktan oluşur: sınıfı (`1` her uygulama, `2` tek bir uygulama türü,
`3` aygıtlar), biçimi (`1` aygıt, `2` dize, `3` nesne, `4` nesne listesi, `5` tamsayı, `6` boole,
`7` tamsayı listesi) ve numarası. Komut, öğeleri Microsoft'un belgelediği şekilde adlandırır
(`BcdLibraryBoolean_AutoRecoveryEnabled`).

Toplama araçları çoğu zaman depoyu kopyalamaz: EFI sistem bölümü bağlı değildir ve `\Boot` gizlidir.
Depoyu açıkça toplayın veya bölümü bağlayın. `-d` ayrıca `BCD-Template` dosyasını da okur (hangisi
olduğunu `HiveType` belirtir); şablon, Windows'un önyükleme yaptığı depo değil, başlangıç noktası
olarak aldığı şablondur.

## Analist için uyarılar { #analyst-cautions }

- **Ayarlar devralınır.** Bir önyükleme yükleyicisi, `BcdLibraryObjectList_InheritedObjects`
  içindeki nesnelerden devralır (genellikle `{globalsettings}` öğesini devralan
  `{bootloadersettings}`). `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks`, `SafeBoot` ve
  `BootStatusPolicy`, çözümlenmiş depolanmış değerlerdir (kendi değeri, yoksa devralınan değer);
  `Inherited` ise hangilerinin başka bir nesneden geldiğini belirtir. Devralma en fazla sekiz düzey
  izlenir ve döngüler sınırlandırılır. Başvurulan bir nesne kısmi bir depoda bulunmayabilir. Boş
  alanlar, etkin bir Windows varsayılanını kanıtlamaz. Bellenim ve Secure Boot da depolanmış bir
  seçeneğin etkili olmasını engelleyebilir.
- **Ayarlanmamış, kapalı demek değildir.** Boş bir `RecoveryEnabled`, öğenin bulunmadığı anlamına
  gelir; Windows'un varsayılanı geçerlidir.
- **Zamanlar öğe başınadır.** Her öğenin anahtar zamanı (`Elements` içindeki `key_last_written`),
  öğenin en son yazıldığı zamandır. Windows bazı öğeleri güncellemeler ve yükseltmeler sırasında
  yeniden yazar; diğer öğelerin zamanlarıyla karşılaştırın.
- **Aygıtların kodu kısmen çözülür.** Bir GPT bölümü, bölümünün ve diskinin GUID'leri olarak; bir
  ramdisk ise seçenekler nesnesi olarak verilir. Diğer aygıtlar yalnızca baytlarını korur
  (`data_raw`).
- **Belgelenmemiş öğelerin** (`0x15000065`, `0x4…` sınıfı) adı yoktur ve baytları korunur.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

Özet; her deponun varsayılan girdisini ve her test imzalama, bütünlük denetimi, kurtarma, önyükleme
durumu ilkesi (herhangi bir `Ignore…`) ve güvenli mod ayarını, ayarlandığı zamanla veya devralındığı
nesneyle birlikte listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Ana bilgisayar, kovan ve `BCD` veya `BCD-Template`. |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | Nesnenin GUID'i, türü (`0x10200003`) ve ne olduğu (`Windows boot loader`, `inherited settings for Windows boot loader`, …). |
| `Description`, `Path`, `Device` | Açıklama, uygulamanın yolu ve aygıtı (bir GPT bölümü ve disk ya da bir ramdisk'in seçenekler nesnesi). |
| `IsDefault`, `DisplayOrder` | Windows önyükleme yöneticisinin onu varsayılan olarak önyükleyip önyüklemediği ve önyükleme menüsündeki yeri. |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | Depodan çözümlendiği şekliyle `recoveryenabled`, `testsigning`, `nointegritychecks`. |
| `SafeBoot`, `BootStatusPolicy` | Sayısal değeriyle birlikte `safeboot` (`Minimal`, `Network`, `DsRepair`) ve `bootstatuspolicy` (`IgnoreAllFailures`, …). |
| `Inherited` | Bunlardan hangilerinin başka bir nesneden geldiği ve hangi nesneden geldiği. |
| `Elements` | Her öğe: türü, adı, değeri, baytları (aygıtlar, belgelenmemiş türler ve kodu çözülemeyen veriler için) ve anahtar zamanı. |
| `KeyLastWrittenUTC` | Nesne anahtarının son yazılma zamanı. |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Türü olmayan nesneler, verileri biçimlerine uymayan öğeler. |

## İnceleme stratejisi { #investigation-strategy }

1. Varsayılan girdiyi ve her önyükleme yükleyicisini `TestSigning`, `NoIntegrityChecks`,
   `RecoveryEnabled = false`, `Ignore…` türünde bir `BootStatusPolicy` ve `SafeBoot` açısından
   denetleyin.
2. Değişikliği tarihlendirmek için öğe zamanlarını kullanın ve Prefetch, Amcache, işlem oluşturma
   olayları (4688, Sysmon 1) ve System olay günlüğündeki `bcdedit.exe` kayıtlarıyla ilişkilendirin.
3. Orada olmaması gereken önyükleme girdilerini veya aygıtları (başka bir yükleyici yolu, bir
   ramdisk) ve değiştirilmiş bir varsayılanı veya görüntüleme sırasını arayın.
4. Depoyu `BCD-Template` ile ve aynı derlemeye sahip başka bir sistemle karşılaştırın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: BCD önyükleme seçenekleri başvurusu](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (nesne türleri)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
