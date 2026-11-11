# Windows Kayıt Defteri Kovanları { #windows-registry-hives }

## Kayıt defteri kovanları nedir { #what-registry-hives-are }

Windows Kayıt Defteri, sistem ve kullanıcı yapılandırmasını ikili kovan dosyalarında saklar.
Anahtarlar bir ağaç oluşturur; bir anahtarın altındaki değerler türlendirilmiş veriler içerir. Kayıt
defteri artefaktları kalıcılığı, yüklü yazılımları, hizmetleri, hesapları, ağ yapılandırmasını,
kullanıcı etkinliğini, güvenlik ayarlarını ve Windows durumunun neredeyse her parçasını tanımlar.

Juntei'nin genel okuyucusu herhangi bir birincil kovanın kodunu çözer, işlem günlüklerini yeniden
oynatır, hasarı raporlar ve silinmiş/boş hücre, hücre slack'i, kalıntı, yeniden oynatma öncesi ve
yalnızca işlem günlüğünde bulunan kayıtları kurtarabilir.

<span class="evidence-strength">En iyi sonuç:</span> kovan, anahtar/değer durumunu içeriyordu veya
kurtarılabilir biçimde koruyordu. Bir anahtarın son yazılma zamanı, hangi değerin neden değiştiğini
değil, anahtarın değiştiğini gösterir.

## Yaygın kovanlar ve konumları { #common-hives-and-locations }

| Kovan | Tipik yol | Kanıt örnekleri |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | Hizmetler, sürücüler, denetim kümeleri, ana bilgisayar adı, bağlı aygıtlar, ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | Yüklü yazılımlar, TaskCache, profiller, Windows yapılandırması |
| `SAM` | `C:\Windows\System32\config\SAM` | Yerel hesaplar ve hesap meta verileri |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | Yerel güvenlik ilkesi/sır meta verileri (erişime bağlı) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | Varsayılan sistem profili yapılandırması |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | Kullanıcı tercihleri, Run anahtarları, RecentDocs, uygulama etkinliği |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | Kabuk sınıfları, COM/kullanıcı kabuğu etkinliği |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | Uygulama envanteri (anlamsal çıktı için `parse-registry amcache` kullanın) |

Yanındaki `.LOG`, `.LOG1` ve `.LOG2` dosyalarını da toplayın. Windows çalışırken kopyalanan bir kovan
kirli (dirty) olabilir; işlem günlükleri, birincil dosyanın tek başına henüz yansıtmadığı
kaydedilmiş (commit edilmiş) durumu içerir.

## Analist için uyarılar { #analyst-cautions }

- Bir anahtarın son yazılma zaman damgası tek bir değere değil, **anahtara** aittir.
- Kayıt defteri değerlerinin genellikle bağımsız bir zaman damgası yoktur.
- `CurrentControlSet`, `SYSTEM\Select` üzerinden çözümlenen bir analist/Windows takma adıdır;
  gerçekte saklanan bir anahtar değildir.
- Silinmiş kayıtlar kısmi, güncelliğini yitirmiş, kurtarma kaynakları arasında yinelenmiş veya
  üzerine yazılmış olabilir.
- Yalnızca işlem günlüğünde bulunan kayıtlar geçmiş/ara durumu gösterir. `recovered_from` değerini
  açıklayın.
- `absent_value` sonucu, sorgulanan anahtarın var olduğu ancak ayrıştırılan durumda o değere sahip
  olmadığı anlamına gelir; değerin hiç var olmadığını kanıtlamaz.
- Dizeler, yollar ve komut satırları yapılandırmadır; yapılandırılan eylemin çalıştığının kanıtı
  değildir.
- Uygulanabilir günlükleri olmayan kirli bir kovan, durumu eksik gösterebilir veya güncelliğini
  yitirmiş durumu ortaya çıkarabilir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host` | Mevcut sistem bağlamından türetilen ana bilgisayar adı; Juntei onu güvenle belirleyemediğinde boştur. |
| `HiveType` | Algılanan kovan kategorisi (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT` vb.). Yolları yorumlamak için kullanın. |
| `SourcePath` | Tam girdi kovanı. Dizin modu ana bilgisayarları/kullanıcıları birleştirdiğinde vazgeçilmezdir. |
| `RecordType` | `key`, `value`, `default_value` veya `absent_value`. Yokluk yalnızca sorguya/duruma özgüdür. |
| `KeyPath` | Kovan kökünün altındaki yol. `[unknown]`, çözümlenemeyen silinmiş üst öğeleri işaretler. |
| `KeyLastWrittenUTC` | Anahtarın bütün olarak son değiştirilme zamanı. Hangi değerin değiştiğini belirleyemez. |
| `ValueName` | Adlandırılmış değer; varsayılan/adsız bir değer için boştur. |
| `ValueType` | Kayıt defteri türü (`REG_SZ`, `REG_DWORD`, `REG_BINARY` vb.). Tür, yorumlamayı etkiler. |
| `ValueData` | Türe göre işlenmiş değer. Çoklu dizeler JSON'da liste olarak kalır. |
| `DataSize` | Değerin kaydettiği boyut. Ham baytlar ve `Problems` ile karşılaştırın. |
| `ValueDataRaw` | Kod çözme başarısız olduğunda veya `--raw-data` kullanıldığında tam baytlar. Doğrulama için saklayın. |
| `ValueU64` | DWORD/QWORD için JavaScript hassasiyet kaybı olmadan tam sayısal gösterim. |
| `ValueFiletimeUTC` | İsteğe bağlı QWORD-FILETIME gösterimi. Yalnızca anahtarın şeması FILETIME tanımladığında anlamlıdır. |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only` veya `transaction_log_recovered`. Güven düzeyini belirler. |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay` veya `SYSTEM.LOG1#1234` gibi bir günlük girdisi. |
| `TransactionLogStatus` | Bu kayıt/kovan bağlamı için `clean`, `replayed`, `unchanged` veya `not_applied`. |
| `LogsApplied` | Juntei'nin yeniden oynattığı işlem günlüğü dosya adları. Bunları rapora kaydedin. |
| `Problems` | Hasar, eksik hücreler, tür uyuşmazlığı, çözümlenemeyen listeler/üst öğeler veya diğer sınırlamalar. |

## Kurtarma durumunun yorumlanması { #recovery-status-interpretation }

| Durum | Anlamı |
|---|---|
| `deleted_complete` | Silinmiş anahtar/değer, bilinen tam yolu/verisiyle kurtarıldı. Yine de etkin durum değil, geçmiş durumdur. |
| `deleted_partial` | Yolun, değer listesinin, değerin veya verinin bir kısmının üzerine yazılmış ya da okunamaz durumda. Boşlukları yeniden kullanılmış hücrelerden doldurmayın. |
| `residual_value_only` | Değer, yeterli anahtar bağlamı olmadan korunmuş; `KeyPath` bulunmayabilir. |
| `transaction_log_recovered` | Durum yalnızca yeniden oynatma öncesi/günlük sürümlerinde var; mevcut ayrılmış/boş kayıtlarda yok. |

## İnceleme stratejisi { #investigation-strategy }

1. Biliniyorsa tam bir anlamsal anahtarla başlayın; keşif için regex/veri aramasını kullanın.
2. İçerikten önce `TransactionLogStatus`, `AllocationStatus`, `RecoveredFrom` ve `Problems`
   değerlerini denetleyin.
3. Verileri anahtarın belgelenmiş şemasına göre yorumlayın; `REG_BINARY` kendini açıklamaz.
4. Yapılandırmayı çalışma zamanı kanıtlarıyla (olay günlükleri, Prefetch, SRUM, hizmetler/görevler)
   ilişkilendirin.
5. Yeni veya tartışmalı alanlar için ham verileri saklayın.

## Daha fazla ayrıntı { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Maxim Suhanov'un kayıt defteri biçimi spesifikasyonu](https://github.com/msuhanov/regf)
- [yarp kayıt defteri ayrıştırıcısı](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Microsoft: Kayıt defteri değer türleri](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
