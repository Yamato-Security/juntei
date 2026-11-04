# UserAssist { #userassist }

## UserAssist nedir { #what-userassist-is }

Windows Explorer, bir kullanıcının Explorer kabuğundan başlattığı programları ve kısayolları sayar: Başlat
menüsü, görev çubuğu, masaüstü ve klasörlerdeki çift tıklamalar. Bu sayıları kullanıcının `NTUSER.DAT`
dosyasında tutar. Her öğenin şunları vardır:

- bir çalıştırma sayısı;
- Windows 7 ve sonrasında, kaç kez ve ne kadar süre (milisaniye cinsinden) odakta kaldığı;
- en son ne zaman çalıştığı.

Değer adları ROT13 ile kodlanmıştır. Bir programın adı genellikle bilinen bir klasörün GUID'iyle başlar
(`{1AC14E77-…}`, `System32` anlamına gelir).

UserAssist, belirli bir kullanıcının etkileşimli olarak neyi başlattığını gösterir; diğer yürütme
kanıtlarının çoğu bunu bir kullanıcıya atfetmez.

<span class="evidence-strength">En iyi sonuç:</span> kullanıcı bu programı veya kısayolu Explorer
üzerinden bu kadar kez başlatmıştır; en son bu zamanda. Başka bir yolla (komut satırı, hizmet, zamanlanmış
görev, başka bir program) başlatılan programlar sayılmaz.

## Konum { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | İçeriği |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | Programlar (`executable`), Windows 7 ve sonrası |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | Kısayollar (`shortcut`), Windows 7 ve sonrası |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | Programlar ve kısayollar (`active_desktop`), Windows 2000'den Vista'ya |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | Internet araç çubuğu (`internet_toolbar`), Windows 2000'den Vista'ya |

Diğer GUID'ler `other` olarak raporlanır. Her profil için `NTUSER.DAT` dosyasını işlem günlükleriyle
birlikte ve kullanıcının SID'sini veren sistemin `SOFTWARE` kovanını toplayın.

## Analist için uyarılar { #analyst-cautions }

- **Yeni profiller önceden doldurulur.** Windows 7 ve sonrasında yeni bir profil varsayılan öğelerle
  (Getting Started, Calculator, Paint, Sticky Notes, …) başlar. Bunların çalıştırma sayıları 14'ten geriye
  doğru sayar ve hepsi, profilin oluşturulduğu zamana yakın tek bir son çalıştırma zamanını paylaşır.
  Bunlar kullanıcı etkinliği değildir.
- **Sürüm 3, 5'ten saymaya başlar** (Windows 2000'den Vista'ya). Saklanan sayı 5'ten başlar, bu nedenle
  ondan 5 çıkarılır. 5'in altındaki bir saklanan sayı `Problems` içinde raporlanır.
- **Yollar bilinen klasörün varsayılan yolunu kullanır** (`%windir%\system32`, `%ProgramFiles%`,
  `%APPDATA%\…`). Başka bir yere yönlendirilmiş bir klasör yine de varsayılanını gösterir. Uygulama
  kimliklerinin (`Microsoft.Windows.Explorer`, `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) ve Denetim
  Masası öğelerinin (`::{…}`) yolu yoktur.
- **İzleme kapatılabilir:** `Explorer\Advanced` `Start_TrackProgs` = 0, `Policies\Explorer`
  `NoInstrumentation` = 1 veya `UserAssist\Settings` `NoLog` = 1 ile. Özet, izlemenin kapalı olduğu
  kovanları listeler. Etkin bir profilde boş veya eksik bir UserAssist, adli bilişim karşıtı (anti-forensics)
  faaliyet anlamına gelebilir.
- **`NoEncrypt` = 1**, Windows'un adları ROT13 olmadan yazmasını sağlar. Bu tür adların kodu yanlış çözülür;
  bu nedenle o kovanın her öğesi bunu `Problems` içinde belirtir (kayıtlar stdout'a gidiyorsa stderr'de de).
- **Son çalıştırma zamanı ve sayılar öğe başına tutulur,** ancak anahtarın son yazılma zamanı
  (`KeyLastWrittenUTC`) yalnızca herhangi bir öğedeki en son güncellemenin zamanıdır.
- **Çalıştırma sayıları 0 olabilir;** yalnızca odakta kalmış öğeler için. Odak sayıları ve süreleri
  yalnızca sürüm 5'te (Windows 7 ve sonrası) bulunur.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

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
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Özet, en son çalıştırılan öğeleri (en yeniden başlayarak) kullanıcılarıyla birlikte ve UserAssist'in
kapalı olduğu kovanları listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Ana bilgisayar, profil klasörünün adı, sistemin `ProfileList` kaydından (sistemin `SOFTWARE` kovanı ile) kullanıcının SID'si ve kovan. |
| `GUID`, `Kind` | `{GUID}` anahtarı ve içeriği (`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`). |
| `NameRot13`, `Name` | Saklandığı biçimde ve kodu çözülmüş olarak değerin adı. |
| `Path` | Bilinen bir klasörün GUID'i varsayılan yoluyla değiştirilmiş olarak, yol biçiminde kodu çözülmüş ad; uygulama kimlikleri ve kabuk öğeleri için boştur. |
| `RunCount` | Explorer'dan kaç kez başlatıldığı (sürüm 3: saklanan sayı eksi 5). |
| `FocusCount`, `FocusTimeMs` | Kaç kez odakta kaldığı ve milisaniye cinsinden ne kadar süre (sürüm 5). |
| `LastRunUTC` | En son ne zaman başlatıldığı. Windows 0 sakladığında veya saklanan zaman 9999'u aştığında boştur (bu durumda `Problems` onu verir ve `DataRaw` baytları tutar). |
| `Format`, `SessionID` | Verinin düzeni (5: 72 bayt, 3: 16 bayt) ve en son yazıldığı oturum numarası. |
| `KeyLastWrittenUTC` | `Count` anahtarının son yazılma zamanı: öğelerinden herhangi birindeki en son güncelleme. |
| `DataRaw` | Kodu çözülemediğinde değerin baytları (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Bilinmeyen boyutta veri, anahtarın `Version` değeriyle uyuşmayan bir boyut, 5'in altında bir sürüm 3 sayısı, 9999'u aşan bir son çalıştırma, `NoEncrypt`. |

## İnceleme stratejisi { #investigation-strategy }

1. `LastRunUTC` alanına göre sıralayın ve önceden doldurulmuş öğeleri ayırın: bunlar profilin oluşturulma
   zamanına yakın tek bir zamanı paylaşır.
2. Kullanıcının yazabildiği konumlardaki (`%USERPROFILE%`, `Downloads`, `Temp`, `ProgramData`),
   çıkarılabilir sürücülerdeki ve ağ yollarındaki programları ve yönetim ve uzaktan erişim araçlarını arayın.
3. Prefetch (çalıştırma zamanlarıyla yürütme), Amcache ve ShimCache (varlık), Jump Lists ve LNK dosyaları
   (onlarla neyin açıldığı) ve program dosyaları için `$MFT` ile ilişkilendirin.
4. Temizleme izleri için izlemenin kapalı olduğu kovanları ve UserAssist'i boş olan profilleri kontrol edin.
5. Silinmiş UserAssist değerleri için `parse-registry read --recover-deleted` komutunu kullanın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: User Assist keys](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
