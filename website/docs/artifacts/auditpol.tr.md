# Denetim İlkesi { #audit-policy }

## Nedir { #what-it-is }

Denetim ilkesi, Windows'un hangi güvenlik olaylarını Security olay günlüğüne kaydedeceğini belirler.
Windows Vista ve sonrasında alt kategori başına (Windows 10 ve 11'de 59), Windows XP ve Server 2003'te
ise kategori başına ayarlanır. Her biri başarıları, başarısızlıkları, her ikisini veya hiçbirini kaydeder.

LSA, ilke ister `auditpol`, ister yerel güvenlik ilkesi, ister Group Policy üzerinden gelmiş olsun,
yürürlükteki ilkeyi `SECURITY` kovanında tutar.

<span class="evidence-strength">En iyi sonuç:</span> kovan en son yazıldığında sistem bu olayları
kaydedecek (veya kaydetmeyecek) şekilde ayarlanmıştı. Bir alt kategori denetlenmediğinde, eksik bir olay
etkinliğin gerçekleşip gerçekleşmediği hakkında hiçbir şey söylemez.

## Konum { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista ve sonrası 12 baytlık bir başlık, ardından kategori kategori her alt kategori için bir ayar,
ardından da her kategorinin kaç alt kategorisi olduğunu tutar. Sürümler yalnızca bu sayılarda farklılık
gösterir: her sürüm bir kategorinin sonuna alt kategoriler ekler (okunan Windows 8.x ve Server 2012 R2
kovanlarında 56, Windows 10, 11 ve Server 2022'de 59). Windows XP ve Server 2003, denetimin açık olup
olmadığını (bir bayt, ardından Windows'un temizlemediği dolgu) ve dokuz kategori ayarını tutar. Bu düzen,
NIST'in CFReDS Hacking Case'inin hiçbir şeyi denetlemeyen herkese açık Windows XP imajında kontrol
edilmiştir; kategori ayarlarının değerleri Microsoft'un belgelerini izler.

`SECURITY` kovanını işlem günlükleriyle birlikte ve ana bilgisayarı adlandıran sistemin `SYSTEM`
kovanıyla birlikte toplayın.

## Analist için uyarılar { #analyst-cautions }

- **Bilinmeyen veriler bilinmeyen olarak kalır.** Desteklenmeyen revizyonlar tanılama bilgileriyle
  korunur. Bilinmeyen ayar bayrakları `Success` ve `Failure` alanlarını boş bırakır ve denetlenen ayar
  sayımlarının dışında tutulur. Hasarlı bir sayı tablosu, ayrılmış sözcüğü başka bir denetim ayarına
  yükseltemez. Yakalamalar, ad/GUID kataloğunun ötesinde ek yuvalar içerebilir. Bunların ayarları ve sayısal
  `SubcategoryId` değerleri boş adlar/GUID'lerle korunur; eşlenmemiş bir yuva kendi başına bozulma kanıtı
  değildir.


- **Sıra Windows'un kendi sırasıdır.** Bir kategori içinde ayarlar, alt kategorilerin GUID'lerinin değil
  ileti kimliklerinin sırasındadır. Special Logon (`%%12548`, Logon/Logoff'un 4. dizini) IPsec Quick
  Mode'dan önce gelir. Bunları GUID sırasıyla okuyan araçlar, Windows 10'un varsayılanını Special Logon
  "No Auditing" ve IPsec Quick Mode "Success" olarak raporlar. `SubcategoryId`, 4719 olayının (System audit
  policy was changed) verdiği kimlikle aynıdır; böylece ikisi eşleştirilebilir.
- **Kaynağı değil, yürürlükteki ilke.** Group Policy ve yerel ilke aynı değeri yazar. Group Policy'nin
  `audit.csv` dosyası (SYSVOL içinde ve `C:\Windows\System32\GroupPolicy` içinde) neyin uygulanmasının
  amaçlandığını gösterir (bir ilkenin `Machine\Microsoft\Windows NT\Audit\audit.csv` dosyası).
- **Zaman içindeki değişiklikler:** `KeyLastWrittenUTC`, yaklaşık olarak ilkenin en son ne zaman
  değiştiğini gösterir. 4719 olayı her değişikliği kaydeder (Security günlüğü onu tuttuğu sürece). Bir
  saldırgan denetimi kapatıp sonra yeniden açabilir.
- **Alt kategoriler ve kategoriler.** Windows Vista ve sonrasında Windows'un kullandığı alt kategorilerdir
  (`SCENoApplyLegacyAuditPolicy` güvenlik seçeneği kapalı olmadıkça ve eski bir kategori ilkesi bunları
  geçersiz kılmadıkça).
- **Windows XP ve 2003:** denetim kapalıyken (`AuditingEnabled` False), ayarı ne olursa olsun hiçbir
  kategori denetlenmez.
- **Olaylar:** `Events`, Microsoft'un gelişmiş denetim ilkesi ayarları belgelerine göre her alt kategorinin
  başlıca olaylarını listeler. Bunlar, alt kategorinin kaydettiği her olay değil, bir incelemenin genellikle
  aradığı olaylardır.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

Özet, her kovan için düzenini, kaç ayarın denetlendiğini ve ilkenin en son ne zaman yazıldığını verir.
Başarı ve başarısızlık için, yalnızca başarı için ve yalnızca başarısızlık için denetlenenleri, ardından
denetlenmeyen alt kategorileri başlıca olaylarıyla birlikte listeler: bu olaylar yakalanan ilkede devre
dışıdır; bu, daha önceki etkinlik sırasındaki ilkeyi kanıtlamaz.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath` | Ana bilgisayar (sistemin `SYSTEM` kovanından) ve `SECURITY` kovanı. |
| `Layout` | `subcategories` (Windows Vista ve sonrası) veya `categories` (XP ve 2003); değerin kodu çözülemediğinde boştur. |
| `Category`, `Subcategory` | `auditpol` bunları nasıl adlandırıyorsa öyle. |
| `SubcategoryGUID`, `SubcategoryID` | Alt kategorinin GUID'i ve 4719 olayının verdiği biçimdeki ileti kimliği (`%%12548`). |
| `Setting` | `none`, `success`, `failure` veya `success_and_failure`. |
| `Success`, `Failure` | Başarıların ve başarısızlıkların kaydedilip kaydedilmediği. |
| `AuditingEnabled` | Windows XP ve 2003: denetimin genel olarak açık olup olmadığı. |
| `Events` | Alt kategorinin başlıca olay kimlikleri. |
| `KeyLastWrittenUTC` | `PolAdtEv` değerinin son yazılması: yaklaşık olarak ilkenin en son ne zaman değiştiği. |
| `DataRaw` | Değerin kodu tamamen çözülemediğinde baytları (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Bilinmeyen bir düzen veya boyut, tutmayan sayılar ve 3'ten büyük değerler. |

## İnceleme stratejisi { #investigation-strategy }

1. Bir şeyin gerçekleşmediği sonucuna varmadan önce alt kategorisinin denetlenip denetlenmediğini kontrol
   edin: Process Creation (4688), Logon (4624, 4625), Special Logon (4672), Other Object Access Events
   (zamanlanmış görevler, 4698), Security System Extension (hizmet kurulumları, 4697), User Account and
   Security Group Management (4720, 4732), File Share (5140), Kerberos (4768, 4769).
2. İlkeyi Security günlüğüyle karşılaştırın: şu anda denetlenmeyen alt kategorilere ait olaylar, ilkenin
   değiştiği anlamına gelir. 4719 olayını ve yakın tarihli bir `KeyLastWrittenUTC` değerini arayın.
3. Etki alanı genelindeki bir ilkeyi yerel bir değişiklikten ayırt etmek için Group Policy'nin `audit.csv`
   dosyasıyla karşılaştırın.
4. `PolAdtEv` değerinin önceki sürümleri için `parse-registry read --recover-deleted` komutunu ve işlem
   günlüklerini kullanın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Advanced security audit policy settings](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: Auditing constants (subcategory GUIDs)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
