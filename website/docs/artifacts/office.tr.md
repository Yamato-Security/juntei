# Microsoft Office (Son Kullanılan Dosyalar, Güvenilen Belgeler, Makro Ayarları) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## Office neleri kaydeder { #what-office-records }

Microsoft Office, kullanıcı başına `NTUSER.DAT` içinde ve sürüm ve uygulama başına şunları tutar:

- **Son kullanılan dosyalar ve konumlar:** bir uygulamanın açtığı belgeler olan `File MRU` ve
  bunların açıldığı klasörler olan `Place MRU`; her öğe en son açıldığı zamanla birlikte.
  Microsoft 365 ayrıca oturum açmış her hesap için bir liste tutar (`User MRU\<account>`).
- **Güvenilen Belgeler:** bir kullanıcı bir belgede *Düzenlemeyi Etkinleştir* veya *İçeriği
  Etkinleştir* düğmesine tıkladığında, Office bunu `TrustRecords` içinde hatırlar: belgenin yolu, ne
  zaman güvenilir kabul edildiği ve makrolarının etkinleştirilip etkinleştirilmediği.
- **Makro ayarları:** `VBAWarnings` (hangi makroların çalıştığı), `BlockContentExecutionFromInternet`
  (İnternet'ten gelen dosyalardaki makroların engellenip engellenmediği), `AccessVBOM` (programların
  VBA projelerini değiştirip değiştiremeyeceği) ve belgelerindeki makroların sormadan çalıştığı
  klasörler olan **Güvenilen Konumlar**. Grup İlkesi aynı ayarları `Software\Policies` altında
  yapabilir.

Kötü amaçlı bir belge genellikle kullanıcının makrolarını etkinleştirmesini gerektirir: makroları
etkinleştirilmiş bir güven kaydı belgeyi, kullanıcıyı ve bir zamanı birbirine bağlar.

<span class="evidence-strength">En iyi sonuç:</span> kullanıcı bu belgeyi bu uygulamada açtı (ve en
son bu zamanda); kullanıcı bu belgenin düzenlemesini veya makrolarını bu zamanda etkinleştirdi; Office
makroları bu şekilde çalıştıracak biçimde yapılandırılmıştı.

## Konum { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

Sürümler: `16.0` Office 2016 ve sonrası (2019, 2021, 2024, Microsoft 365), `15.0` Office 2013,
`14.0` Office 2010, `12.0` Office 2007'dir.

Bir öğe `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx` biçimindedir: `T`
son açılma zamanıdır (onaltılık bir FILETIME), yol ise `*` işaretinden sonra gelir. Bir güven kaydı 24
bayttır: belgenin güvenilir kabul edildiği FILETIME, ardından son 4 bayttaki bayrakları; makroları
etkinleştirildiyse `0x7FFFFFFF`, yalnızca düzenlemesi etkinleştirildiyse `0x00000001`.

## Analist için uyarılar { #analyst-cautions }

- **Zamanlar.** `LastOpenedUTC` her öğenin kendi zamanıdır. `TrustedAtUTC`, kullanıcının belgeye
  güvendiği, yani *Düzenlemeyi Etkinleştir* veya *İçeriği Etkinleştir* düğmesine tıklandığı andır;
  bu, en son açılıştan önce olabilir.
- **Makrolar etkinleştirildi, çalıştırılmadı.** `MacrosEnabled`, kullanıcının belgenin makrolarını
  etkinleştirdiğini söyler; bir makronun çalıştığını veya ne yaptığını kanıtlamaz. İşlem oluşturma,
  Prefetch, PowerShell günlükleri ve belgenin kendisiyle ilişkilendirin.
- **Yollar Office'in sakladığı biçimdedir.** Güven kayıtları belgeyi `%USERPROFILE%`, eğik çizgiler,
  bir `file:///` URL'si ve URL kodlaması (`%20`) kullanabilen bir yolla adlandırır. Bulut
  belgelerinin MRU öğeleri URL'lerdir.
- **Varsayılanlar ve etkin ilke.** Office varsayılan güvenilen konumlar oluşturur (`Templates`,
  `XLSTART`, `STARTUP`, …). Bir konumun `Date` değeri yararlı bir ipucudur, ancak onu kimin
  oluşturduğunun kanıtı değildir: yöneticiler, betikler, ilkeler ve kopyalanmış kayıt defteri
  ayarları da onu doldurabilir. `VBAWarnings` değerinin bulunmaması tek başına etkin makro ilkesini
  belirlemez. Office sürümü, İnternet dosya işaretlemesi ve diğer yönetim ayarları da önemlidir.
  [Microsoft, İnternet makrolarının engellenmesini açıklıyor](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked).
- **Silinmiş belgeler.** Listeler belgelerden daha uzun yaşar; eksik bir dosya burada yine de
  görünebilir. Listeler kısadır ve Office'in *Sabitlenmemiş öğeleri temizle* özelliği ve temizleme
  araçlarıyla silinir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

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
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

Özet; makroları etkinleştirilmiş belgeleri, makroların daha serbest çalışmasına izin veren ayarları
(`VBAWarnings` 1, `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1,
`AllowNetworkLocations` 1), kullanıcıların eklediği veya bir ilkenin ayarladığı güvenilen konumları ve
en son açılan belgeleri listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Ana bilgisayar, profil sahibi (yoldan veya kabuk klasörlerinden), sistemin `ProfileList` kaydından gelen SID ve kovan. |
| `OfficeVersion`, `App` | Sürümün anahtarı (`16.0`) ve uygulama (`Word`, `Excel`, …). |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security` veya `Trusted Locations`. |
| `Account` | Bir `User MRU` listesinin Microsoft 365 hesabı. |
| `Policy` | Ayarın veya konumun ilke tarafından ayarlanıp ayarlanmadığı. |
| `MRU-Position`, `IsMostRecent` | Bir öğenin listesindeki yeri (`Item 1` 0'dır, yani en yenisi). |
| `Path` | Belge veya klasör; güvenilen bir belgenin saklandığı biçimdeki yolu; güvenilen bir konumun `Path` değeri. |
| `LastOpenedUTC` | Öğenin en son açıldığı zaman. |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | Belgenin ne zaman güvenilir kabul edildiği, makrolarının etkinleştirilip etkinleştirilmediği ve kaydın bayrakları. Bilinmeyen bayraklar `MacrosEnabled` alanını boş bırakır ve bir tanılama iletisi üretir. |
| `Setting`, `Value` | Bir makro ayarı ve anlamıyla birlikte değeri (`1 (all macros enabled)`); güvenilen bir konumun anahtarı (`Location3`) ve diğer değerleri (`AllowSubFolders=1; Date=…; Description=…`). |
| `KeyPath`, `KeyLastWrittenUTC` | Anahtar ve son yazılma zamanı. |
| `DataRaw` | Kodu çözülemediğinde değerin baytları (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | `*` içermeyen veya geçerli olmayan bir zamana sahip öğeler, 24 bayt olmayan güven kayıtları, `Path` içermeyen konumlar, yanlış türler. |

## İnceleme stratejisi { #investigation-strategy }

1. `MacrosEnabled` değeri true olanlarla başlayın: belgeler, ne zaman ve kim tarafından. Dosyaları
   (`$MFT`, USN Journal, `Zone.Identifier`), kaynaklarını (tarayıcı ve posta geçmişi) ve ardından
   neyin çalıştığını bulun.
2. Makro ayarlarını ve güvenilen konumları denetleyin: tüm makroların etkin olması, İnternet
   makrolarının engellenmemesi, VBA projesi erişimi veya kullanıcının eklediği bir konum
   (`Downloads`, `Temp` veya bir ağ paylaşımında) varsayılanları zayıflatır.
3. Çıkarılabilir ve ağ konumları dahil olmak üzere belgelerin yolları için `LastOpenedUTC` değerini
   LNK dosyaları, jump list'ler ve RecentDocs ile birlikte zaman çizelgesine yerleştirin.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Microsoft 365 dosyalarında makroları etkinleştirme veya devre dışı bırakma](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: Güvenilen belgeler](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: Güvenilen konum ekleme, kaldırma veya değiştirme](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
