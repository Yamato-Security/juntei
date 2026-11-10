# BAM / DAM { #bam-and-dam }

## BAM ve DAM nedir { #what-bam-and-dam-are }

Background Activity Moderator (BAM) ve Desktop Activity Moderator (DAM), uygulamaların arka planda neler
yapabileceğini yöneten Windows hizmetleridir (Windows 10 1709 ve sonrası). Her kullanıcı için her
programın en son çalıştığı zamanı `SYSTEM` kovanında tutarlar:

- kullanıcı başına, kullanıcının SID'si ile adlandırılmış bir anahtar;
- program başına bir değer: adı, programın aygıtı üzerindeki yolu
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) veya paketlenmiş bir uygulamanın kimliğidir
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`);
- değer verisinin ilk 8 baytı, programın en son çalıştığı zamandır (FILETIME).

BAM, kullanıcının kendi kovanı olmadan bir programı bir kullanıcıya ve bir zamana bağlar ve çıkarılabilir
aygıtlardan ve ağ aygıtlarından çalıştırılan programları listeler.

<span class="evidence-strength">En iyi sonuç:</span> bu programa ait bir işlem bu kullanıcının
oturumunda çalışmıştır; en son bu zamanda çalışmıştır. BAM çalıştırma sayısını tutmaz ve program
silindikten sonra da bir girdi kalabilir.

## Konum { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

Her kullanıcının anahtarında ayrıca `Version` ve `SequenceNumber` değerleri bulunur; bunlar program
değildir ve dışarıda bırakılır. 1709 veya 1803'ten yükseltilmiş bir sistemde her iki düzen de bulunabilir.

`SYSTEM` kovanını işlem günlükleriyle birlikte ve sistemin `SAM` ve `SOFTWARE` kovanlarıyla birlikte
toplayın: kullanıcıları bunlar adlandırır.

## Analist için uyarılar { #analyst-cautions }

- **Program ve kullanıcı başına tek zaman.** Yalnızca en son çalıştırma tutulur. Önceki çalıştırmaları
  Prefetch, Amcache ve olay günlükleri verir.
- **Yakın tarihli etkinlik.** Bir süredir çalışmamış programların girdilerinin Windows başlarken kaldırıldığı
  bildirilmektedir; bu nedenle BAM çoğunlukla yakın tarihli etkinliği gösterir: BAM'de bulunmayan bir program
  yine de uzun zaman önce çalışmış olabilir.
- **Sürücü harfleri değil, aygıt yolları.** `\Device\HarddiskVolume3`, `C:` değil, Windows'un numaralandırdığı
  üçüncü birimdir. `DevicePath` ve `Path`, aygıtı onun üzerindeki yoldan ayırır; bir sürücü harfi yazmadan
  önce birimi `MountedDevices`, `$MFT` veya diğer artefaktlarla eşleştirin.
- **Her program değil.** BAM ve DAM kullanıcıların oturumlarındaki programları izler; hizmetler çoğunlukla
  görünmez. `S-1-5-18` girdileri genellikle kurulumun ve oturum açma ekranının işlemleridir
  (`FirstLogonAnim.exe`); `S-1-5-90-0-<n>` girdileri ise Desktop Window Manager'dır (`DWM-<n>`).
- **Denetim kümeleri.** Varsayılan olarak Windows'un en son kullandığı denetim kümesi okunur.
  `--all-control-sets` hepsini okur: daha eski bir denetim kümesi, geçerli kümede artık bulunmayan bir
  girdiyi tutuyor olabilir.
- **Kullanıcı adları.** Yerel bir hesabın adı `SAM` kovanından (bilgisayarın SID'si ve hesabın RID'si),
  bir etki alanı hesabının adı ise `ProfileList` içindeki profil klasöründen gelir. `UserSource` hangisi
  olduğunu belirtir (`sam`, `profile_list`, `well_known`). Bunlar olmadan yalnızca SID verilir. `SAM`
  kovanında artık bulunmayan bir RID (silinmiş bir hesap) adsız kalır.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Özet, her kullanıcıyı SID'si, adı ve girdi sayısıyla ve en son çalıştırılan programları (en yeniden
başlayarak) listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath` | Ana bilgisayar ve `SYSTEM` kovanı. |
| `ControlSet`, `IsCurrentControlSet` | Girdinin bulunduğu denetim kümesi ve bunun Windows'un en son kullandığı küme olup olmadığı. |
| `Moderator` | `bam` veya `dam`. |
| `UserSID`, `User`, `UserSource` | Anahtarın adını aldığı SID, kullanıcının adı ve adın nereden geldiği (`sam`, `profile_list`, `well_known`). |
| `Program` | Windows'un yazdığı biçimde değerin adı: bir aygıt yolu veya paketlenmiş bir uygulamanın kimliği. |
| `DevicePath`, `Path` | Bir aygıt yolu için aygıt (`\Device\HarddiskVolume3`) ve onun üzerindeki yol (`\Windows\System32\cmd.exe`); paketlenmiş bir uygulama için boştur. |
| `LastRunUTC` | Programın en son çalıştığı zaman (100 ns hassasiyetinde). |
| `KeyPath`, `KeyLastWrittenUTC` | Kullanıcının anahtarı ve son yazılma zamanı: girdilerinden herhangi birindeki en son güncelleme. |
| `DataRaw` | Değer bir zaman içermediğinde baytları (onaltılık): `REG_BINARY` değilse veya 8 bayttan kısaysa. |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Zaman içermeyen veriler ve okunamayan değerler. |

## İnceleme stratejisi { #investigation-strategy }

1. `LastRunUTC` alanına göre sıralayın ve kullanıcının yazabildiği konumlardaki (`\Users\`, `Downloads`,
   `Temp`, `ProgramData`), sistem birimi dışındaki aygıtlardaki programlara ve yönetim ve uzaktan erişim
   araçlarına bakın.
2. Bunları hangi kullanıcının çalıştırdığını ve geçerli bir profille eşleşmeyen hesapları kontrol edin.
3. Prefetch (çalıştırma sayıları ve önceki çalıştırmalar), UserAssist (Explorer başlatmaları), Amcache ve
   ShimCache (varlık), `$MFT` (programın dosyası) ve olay günlükleriyle ilişkilendirin.
4. Silinmiş BAM değerleri için `--all-control-sets` ve `parse-registry read --recover-deleted` kullanın.

## Daha fazla ayrıntı { #further-details }

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: Security identifiers (well-known SIDs)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

Aralık dışındaki son çalıştırma FILETIME değerleri `LastRunUTC` alanını boş bırakır ve özgün değeri
`DataRaw` içinde korur; açıklaması `Problems` alanındadır. Yakın tarihli etkinlik özeti her ana
bilgisayarı tanımlar ve ayrı kovan/kullanıcı gözlemlerini birbirinden ayrı tutar.
