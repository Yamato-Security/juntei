# Outlook { #outlook }

## Outlook dosyaları nedir { #what-outlooks-files-are }

Windows için Outlook; postaları, kişileri, takvimleri ve görevleri kendine ait dosyalarda tutar:

- **OST** (`.ost`): bir posta kutusunun çevrimdışı kopyası; Outlook bunu Exchange ve Microsoft 365
  hesapları için, Outlook 2013'ten beri de IMAP hesapları için tutar. Outlook 2013 ve sonrasının
  OST dosyalarında sayfalar 4 KiB'tır ve bloklar sıkıştırılmıştır.
- **PST** (`.pst`): kişisel bir depo: POP3 hesapları, arşivler (`archive.pst`), dışa aktarmalar ve
  kullanıcıların oluşturduğu depolar. ANSI PST'ler Outlook 97 ile 2002 arasındaki sürümlere, Unicode
  PST'ler ise Outlook 2003 ve sonrasına aittir.
- **MSG** (`.msg`): dosya olarak kaydedilmiş tek bir öğe; öğe Outlook'tan sürüklenip dışarı
  bırakıldığında, **Farklı Kaydet** ile kaydedildiğinde veya başka bir program tarafından dışa
  aktarıldığında oluşur. Ek olarak eklenmiş iletiler de MSG dosyalarıdır.
- **OFT** (`.oft`): bir Outlook şablonu: yeni öğelerin oluşturulduğu bir MSG dosyası.

Her depo klasörleri, her klasör de kendi öğelerini tutar: e-postalar, toplantı istekleri ve
yanıtları, kişiler ve dağıtım listeleri, takvim öğeleri, görevler, notlar, günlük girdileri ve
raporlar. Her klasör ayrıca gizli öğeler, yani klasörle ilişkili bilgiler (FAI) de tutar:
görünümler, kurallar, formlar ve Outlook'un ayarları (bir Exchange posta kutusunun otomatik
tamamlama listesi gibi `IPM.Configuration.*` öğeleri). Bir öğenin ekleri dosyalar, dosyalara
başvurular, OLE nesneleri veya başka öğeler olabilir (ek olarak iletilen bir ileti, yinelenen bir
randevunun özel durumları); bu öğelerin de kendi ekleri vardır.

<span class="evidence-strength">En iyi sonuç:</span> depo veya dosya, toplandığı sırada öğeyi bu
gönderenler, alıcılar, zamanlar ve eklerle birlikte tutuyordu. Kullanıcının öğeyi okuduğunu,
yazdığını ya da gönderdiğini söylemez; gönderenin, öğenin belirttiği kişi olduğunu da söylemez.

## Konum ve toplama { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

PST dosyaları her yerde olabilir: kullanıcılar arşivleri başka klasörlere, sürücülere ve dosya
paylaşımlarına taşır. Outlook, her profilin depolarının yollarını kullanıcının kayıt defterinde
tutar. Kullanıcının Outlook'tan açtığı ekler `%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\`
altındaki bir klasöre (Windows 8.1'den önce `Temporary Internet Files\Content.Outlook`) kopyalanır ve
orada kalabilir.

Outlook çalıştığı sürece depolarını açık tutar ve bir OST onlarca gigabayt olabilir. Bunları Outlook
kapalıyken ya da bir disk imajından veya bir birim gölge kopyasından toplayın. Juntei her depoyu bir
akış olarak okur: hiçbir zaman bir deponun tamamını bellekte tutmaz.

## Analist için uyarılar { #analyst-cautions }

- Juntei hoşgörülü okur: hasarlı kısımları atlar, okunabileni okur ve geri kalanını sorun olarak
  raporlar. Sorunlar ekrana yazdırılır (her dosyanın ilk 10'u), özette ve Dosyalar tablosunda
  sayılır ve ilgili oldukları öğelerin ve eklerin `Problems` alanında listelenir. Hiç okunamayan bir
  öğenin de Öğeler tablosunda `ItemID` değeri ve nedeniyle birlikte bir satırı bulunur.
- Zamanlar UTC'dir ve 100 nanosaniye hassasiyetindedir. `SubmitTime`, gönderenin istemcisinin iletiyi
  gönderime sunduğu zamandır ve o bilgisayarın saatine göredir; `DeliveryTime` ise iletinin bu posta
  kutusuna ulaştığı zamandır. `CreationTime`, öğenin bu kopyasının oluşturulduğu zamandır: posta
  kutusuna teslim edildiği veya orada oluşturulduğu, OST ile eşitlendiği, PST'ye kopyalandığı veya içe
  aktarıldığı ya da MSG dosyası olarak kaydedildiği zaman. İletinin yazıldığı zaman değildir.
  `LastModifiedTime`, öğenin depoda en son değiştirildiği zamandır.
- Bir görevin `StartTime` ve `DueTime` değerleri kullanıcının saat dilimindeki tarihlerdir ve Outlook
  bunları gece yarısı olarak saklar. Juntei bunları fark (offset) belirtmeden yazar
  (`2017-02-19T00:00:00`) ve `-Z` onları olduğu gibi bırakır.
- Outlook, bir görevin bitiş tarihi veya bir anımsatıcının zamanı gibi bazı tarihlerde **Yok** (None)
  için 1 Ocak 4501 gece yarısını yazar. Juntei bunu saklandığı haliyle yazar:
  `4501-01-01T00:00:00Z`; bir görevin `StartTime` ve `DueTime` değerlerinde ise `Z` olmadan.
- `Read`, depo toplandığı andaki okundu bayrağıdır: biri ya da bir kural öğeyi okundu olarak
  işaretlemiştir. Bayrak, bunu kimin ya da ne zaman yaptığını söylemez.
- Gönderenin ve alıcının adları ve adresleri, gönderen programın ve posta sunucularının
  kaydettikleridir ve sahte olabilir. `EX` adresleri Exchange ayırt edici adlarıdır (distinguished
  name) (`/o=…/ou=…/cn=Recipients/cn=…`); öğe SMTP adresini bunun yanında ayrıca saklıyorsa bu adres
  `SenderSMTP-Address` ve `SMTP-Address` alanlarındadır. Bir iletinin nereden geldiğini denetlemek için
  İnternet üst bilgilerini okuyun: Özellikler tablosundaki `0x007D001F` veya `0x007D001E`
  (`PidTagTransportMessageHeaders`).
- Outlook'ta silinen öğeler, klasör boşaltılana kadar **Silinmiş Öğeler** klasöründe bulunur ve diğer
  öğeler gibi okunur. Artık hiçbir klasörün tablosunda bulunmayan öğeler ise, bir depo onları alanlarını
  yeniden kullanana kadar tutabilse de, henüz okunmamaktadır.
- `ItemID`, iletinin kendi PST veya OST dosyasındaki düğüm kimliğidir (node id): deponun başka bir
  kopyasındaki aynı öğenin kimliği farklı olabilir. Gömülü bir öğenin kimliği, üst öğesinin kimliği ve
  ekin dizinidir (`2097348/0`, `0/0/1`); bir `.msg` veya `.oft` dosyasının öğesinin kimliği ise `0` olur.
- `User` yalnızca yoldan gelir (`Users` sonrasındaki klasör) ve dosya başka bir yere kopyalandığında
  kaybolur. Bir PST herhangi bir kullanıcıya ait olabilir.
- Unicode olmayan metin, öğenin kod sayfasıyla okunur (öğenin kendi kod sayfası, yerel ayarınınki ya
  da İnternet kod sayfası). `--code-page`, kod sayfası belirtmeyen öğelerin kod sayfasını verir: onları
  yazan sistemin ANSI kod sayfası, Japonca bir sistem için `932`.
- Bir PST'nin parolası onu şifrelemez: Juntei parola korumalı PST'leri de diğerleri gibi okur. Bir
  deponun `Encryption` değeri (`Permute`, yani Outlook'un "sıkıştırılabilir şifreleme"si, ya da
  `Cyclic`) yalnızca bloklarını gizler. S/MIME ile şifrelenmiş iletiler gövdelerini ve eklerini şifreli
  bir ekte (`smime.p7m`) tutar; bu ekin şifresi çözülmez.
- Ekler çoğu zaman kötü amaçlı yazılımdır. `--extract-attachments`, verilerini SHA-256 değerleriyle
  adlandırılmış dosyalara, Unix üzerinde yeni dosyalar için `0600` moduyla yazar; bunları asla çalıştırmayın ve yalıtılmış bir
  analiz sisteminde tutun.

### Henüz okunmayanlar { #not-read-yet }

Çevrimdışı adres defterleri (OAB), otomatik tamamlama akışı (`Stream_Autocomplete_*.dat`) ve `.nk2`
dosyaları, gönderme/alma ayarları (`.srs`), Outlook'un ayar önbellekleri (RoamCache, Autodiscover),
Outlook'un SQLite veritabanları ve artık hiçbir klasörün tablosunda bulunmayan iletiler.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pst, *.ost, *.msg and *.oft
  -f, --file <FILE>      Parse one PST, OST, MSG or OFT file, whatever its name
      --code-page <CP>   Code page of items that don't give theirs (default 1252)

Output:
  -o, --output <PATH>              Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>         csv, json, or jsonl
      --properties                 Add the Properties table: every property as stored
      --extract-attachments <DIR>  Write each attachment's data once, named by its SHA-256
  -C, --clobber                    Overwrite files when saving or extracting
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl \
  | jq 'select(.Table == "Items" and .Hidden == false) | {DeliveryTime, SenderAddress, Subject}'
juntei parse-file outlook -f suspicious.msg -o msg-csv --properties --extract-attachments isolated/
```

`-d`, büyük/küçük harf ayrımı yapmadan `.pst`, `.ost`, `.msg` ve `.oft` adlarını arar, ardından başlıkları denetler: PST veya OST için `!BDN`, MSG veya OFT için bileşik dosya imzası. Tcl ileti katalogları gibi aynı uzantıları kullanan ilgisiz dosyalar atlanır. Okunamayan dosyalar ve imzası tanınabilen hasarlı kapsayıcılar hata olarak kalır. `-f`, adı ne olursa olsun verilen dosyayı doğrular ve geçersiz girdiyi hata olarak bildirir. Hiçbir dosya okunamazsa çıkış durumu 1 olur.

`--extract-attachments`, her ekin verisini bir kez yazar: `<SHA-256>` adıyla ve ekin uzantısı 1 ila
10 ASCII harf ve rakamdan oluşuyorsa bu uzantıyla birlikte (`0c7d…1311.jpg`); bir öğeden gelen hiçbir
ad yolda kullanılmaz. `-C` verilmedikçe dizin boş olmalıdır; `-C` ile, aynı adı taşıyan ve başka bir
içerik barındıran dosya değiştirilir.

## Çıktı tabloları { #output-tables }

CSV sekiz dosya oluşturur, `--properties` ile dokuz; satırı olmayan dosya yalnızca başlık satırını
içerir. JSON ve JSONL satırları, `SourceFile` alanından sonra `Table` alanını (`Files`, `Folders`,
`Items`, `Recipients`, `Attachments`, `Contacts`, `Calendar`, `Tasks` veya `Properties`) taşır. Bir
öğenin satırları öğenin ardından, gömülü öğelerinin satırları da onların ardından gelir; bir dosyanın
Dosyalar satırı, o dosyanın diğer tüm satırlarından sonra gelir.

| CSV dosyası | İçerik |
|---|---|
| `Outlook-Files.csv` | Okunan her dosya için bir satır: depo veya dosya ve içerdikleri. |
| `Outlook-Folders.csv` | PST ve OST dosyalarının klasörleri. |
| `Outlook-Items.csv` | Tüm öğeler: klasörlerin iletileri, gizli öğeleri, `.msg` ve `.oft` dosyalarının öğeleri ve bunların eklerine gömülü öğeler. |
| `Outlook-Recipients.csv` | Öğelerin alıcıları. |
| `Outlook-Attachments.csv` | Öğelerin ekleri. |
| `Outlook-Contacts.csv` | Kişilerin ve dağıtım listelerinin alanları. |
| `Outlook-Calendar.csv` | Takvim öğelerinin ve toplantı iletilerinin alanları. |
| `Outlook-Tasks.csv` | Görevlerin ve görev isteklerinin alanları. |
| `Outlook-Properties.csv` | `--properties` ile: depoların, klasörlerin, öğelerin, alıcıların ve eklerin tüm özellikleri, saklandıkları haliyle. |

## Alanlar { #fields }

Her tablo şu alanlarla başlar:

| Alan | Yorumlama |
|---|---|
| `SourceFile` | Satırın okunduğu dosya. |
| `User` | Yolun içinde bulunduğu profilin sahibi olan Windows kullanıcısı (`Users` sonrasındaki klasör). |

### Dosyalar (Files) { #files }

| Alan | Yorumlama |
|---|---|
| `FileType` | `PST`, `OST`, `MSG`, `OFT` ya da öğe olarak okunan ve sınıf kimliği (class ID) ne bir iletininki ne de bir şablonunki olan bir bileşik dosya için `CFB` (Outlook dışındaki birçok program bu kimliği boş bırakır). |
| `Format` | Bir deponun düzeni: `ANSI` (Outlook 97 ile 2002 arası), `Unicode` (Outlook 2003 ve sonrası) veya `Unicode 4K` (Outlook 2013 ve sonrasının OST dosyaları); bir MSG veya OFT dosyası için bileşik dosya sürümü (`CFB v3`, `CFB v4`). |
| `Version`, `ClientVersion` | Deponun başlığındaki `wVer` ve `wVerClient`. |
| `Encryption` | Deponun bloklarının nasıl kodlandığı: `None`, `Permute` veya `Cyclic`. |
| `DisplayName` | Deponun adı (`Personal Folders`, bir e-posta adresi). |
| `RecordKey` | Deponun kimliği (onaltılık); öğelerinin giriş kimlikleri (entry id) bu değeri içerir. |
| `Folders` | Deponun klasörleri. |
| `Items` | Klasörlerinin iletileri; bir MSG veya OFT dosyası için 1. |
| `HiddenItems` | Klasörlerinin gizli öğeleri (klasörle ilişkili bilgiler). |
| `EmbeddedItems` | Her düzeyde eklere gömülü öğeler. |
| `Attachments` | Tüm bu öğelerin ekleri. |
| `Problems` | Dosyada bulunan sorunlar. |

### Klasörler (Folders) { #folders }

| Alan | Yorumlama |
|---|---|
| `FolderID`, `ParentID` | Klasörün düğüm kimliği ve üst klasörününki (kök klasör için boş). |
| `FolderPath` | Kökten aşağıya doğru klasörlerin adları: `/Top of Personal Folders/Inbox`, `/Root - Mailbox/IPM_SUBTREE/Sent Items`. |
| `Name` | Klasörün adı. |
| `ContainerClass` | Klasörün ne tuttuğu: `IPF.Note` (posta), `IPF.Appointment`, `IPF.Contact`, `IPF.Task`, `IPF.Imap`, …. |
| `ContentCount`, `UnreadCount` | Klasörün sahip olduğunu söylediği iletiler ve okunmamış iletiler. |
| `Messages`, `HiddenItems`, `Subfolders` | İçerik tablosunun, ilişkili içerik tablosunun ve hiyerarşi tablosunun satırları: Juntei'nin okudukları. `ContentCount` ile bir fark incelemeye değer. |
| `CreationTime`, `LastModifiedTime` | Depo bunları tutuyorsa, klasörün oluşturulduğu ve en son değiştirildiği zaman (UTC). |
| `Problem` | Klasörün neden açılamadığı; bu durumda alt klasörleri okunmaz. |

### Öğeler (Items) { #items }

| Alan | Yorumlama |
|---|---|
| `FolderPath` | Öğenin klasörü; bir MSG veya OFT dosyasında boştur. |
| `ItemID` | Öğenin kimliği: PST veya OST içinde düğüm kimliği, bir MSG veya OFT dosyasında `0`, gömülü bir öğe için üst öğesinin kimliği ve ekin dizini (`2097348/0`). |
| `Hidden` | Öğenin klasörle ilişkili bilgi, yani gizli bir öğe olup olmadığı. |
| `EmbeddedIn` | Öğenin ekli olduğu öğenin kimliği; diğerleri için boş. |
| `ItemType` | İleti sınıfına göre öğenin ne olduğu: `Mail`, `MeetingMessage`, `Appointment`, `Contact`, `DistributionList`, `Task`, `TaskRequest`, `StickyNote`, `Journal`, `Post`, `Report`, `Configuration` veya `Other`. |
| `MessageClass` | Saklandığı haliyle ileti sınıfı (`IPM.Note`, `IPM.Note.SMIME`, `IPM.Schedule.Meeting.Request`, `REPORT.IPM.Note.NDR`, …). |
| `SubmitTime` | Gönderenin istemcisinin iletiyi gönderime sunduğu zaman (UTC, gönderenin saatine göre). |
| `DeliveryTime` | İletinin bu posta kutusuna teslim edildiği zaman (UTC). |
| `CreationTime`, `LastModifiedTime` | Bu kopyanın depoda veya dosyada oluşturulduğu ve en son değiştirildiği zaman (UTC). |
| `Subject` | Önekiyle birlikte konu (`RE: `). |
| `SenderName`, `SenderAddressType`, `SenderAddress` | İletiyi kimin gönderdiği: adı, adres türü (`SMTP`, `EX`) ve o türdeki adresi. |
| `SenderSMTP-Address` | Başka türde bir adresin yanında ayrıca saklandığında, gönderenin SMTP adresi. |
| `SentRepresentingName`, `SentRepresentingAddress` | İletinin kimin adına gönderildiği (bir temsilci bir yönetici adına gönderir; paylaşılan bir posta kutusu); saklanmışsa SMTP adresiyle, aksi halde kendi türündeki adresle. |
| `To`, `Cc`, `Bcc` | Alıcıların öğenin gösterdiği haliyle adları (adresleri için Alıcılar tablosuna bakın). `Bcc` yalnızca gönderenin kopyasında bulunur. |
| `InternetMessageID`, `InReplyTo` | İletinin `Message-ID` değeri ve yanıtladığı iletininki. |
| `ConversationTopic` | Öneki olmadan konu; bir konuşmanın iletileri bunu paylaşır. |
| `Importance`, `Sensitivity` | `Low`, `Normal` veya `High`; `Normal`, `Personal`, `Private` veya `Confidential`. |
| `Read` | Öğenin okundu olarak işaretlenip işaretlenmediği. |
| `Flags` | İleti bayrakları (`PidTagMessageFlags`, onaltılık): `0x1` okundu, `0x2` değiştirilmemiş, `0x4` gönderime sunulmuş, `0x8` gönderilmemiş (taslak), `0x10` ekleri var, `0x20` benden, `0x40` gizli, `0x80` yeniden gönderilmiş, `0x400` en az bir kez okunmuş, `0x2000` İnternet'ten. |
| `Size` | Ekleriyle birlikte öğenin depodaki boyutu. |
| `Recipients`, `Attachments` | Kaç alıcısı ve kaç eki olduğu. |
| `Categories` | Kategorileri; aralarında bir `;` bulunur. |
| `BodyPreview` | Düz metin gövdesinin başlangıcı; boşlukları daraltılmış, en fazla 256 karakter. Yalnızca HTML veya RTF gövdesi olduğunda boştur. |
| `Problems` | Öğede okunamayanlar (en fazla 10, ardından kaç tane daha olduğu). |

### Alıcılar (Recipients) { #recipients }

| Alan | Yorumlama |
|---|---|
| `ItemID` | Öğenin kimliği. |
| `Index` | Alıcının, öğenin alıcı tablosundaki satırı (0'dan başlayarak). |
| `RecipientType` | `To`, `Cc`, `Bcc`, `Originator` ya da saklandığı haliyle değer (onaltılık). |
| `Name`, `AddressType`, `Address` | Alıcının adı, adres türü ve adresi. |
| `SMTP-Address` | Başka türde bir adresin yanında ayrıca saklandığında SMTP adresi. |

### Ekler (Attachments) { #attachments }

| Alan | Yorumlama |
|---|---|
| `ItemID`, `Index` | Öğenin kimliği ve ekin, öğenin ek tablosundaki satırı (0'dan başlayarak). |
| `Method` | `ByValue` (bir dosya), `ByReference`, `ByReferenceResolve`, `ByReferenceOnly` (bir dosyanın yolu), `EmbeddedMessage` (bir öğe), `OLE` (bir OLE nesnesi), `ByWebReference` ya da sayı. |
| `Filename` | Ekin adı: uzun dosya adı, kısa dosya adı veya görünen adı. |
| `Extension` | Saklandığı haliyle uzantısı (`.pdf`), aksi halde adının uzantısı. |
| `MimeType`, `ContentID` | MIME türü ve bir HTML gövdesinin ona başvururken kullandığı içerik kimliği. |
| `Inline` | Gövdenin eki listelemek yerine gösterip göstermediği (HTML gövdesindeki bir resim). |
| `Hidden` | Outlook'un eki ek listesinden gizleyip gizlemediği. |
| `Size` | Deponun verdiği haliyle, özellikleriyle birlikte boyutu. |
| `DataSize`, `SHA256` | Verisinin boyutu ve SHA-256 değeri: dosya ya da PST veya OST içindeki bir OLE nesnesinin bileşik dosyası. Gömülü bir öğe ve başvuruyla eklenmiş bir ek için boştur. |
| `Pathname` | Başvuruyla eklenmiş bir ekin yolu. |
| `CreationTime`, `LastModifiedTime` | Ekin zamanları (UTC): çoğu zaman ekin oluşturulduğu dosyanınkiler. |
| `EmbeddedSubject` | Gömülü bir öğenin konusu; bu öğenin kendi satırlarında `ItemID` değeri `<ItemID>/<Index>` olur. |
| `ExtractedFile` | `--extract-attachments` ile, verisinin o dizinde yazıldığı dosya. |
| `Problems` | Ekte okunamayan veya yazılamayanlar. |

### Kişiler (Contacts) { #contacts }

| Alan | Yorumlama |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | Öğeler tablosundaki gibi: `Contact` veya `DistributionList`. |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | Kişinin adları. |
| `FileUnder` | Kişinin altında dosyalandığı ad. |
| `Company`, `Title`, `Department` | Kişinin çalıştığı yer. |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | Kişinin üç e-posta adresi; her biri görünen adıyla birlikte. |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | Kişinin telefon numaraları. |
| `InstantMessaging` | Kişinin anlık mesajlaşma adresi. |
| `CreationTime`, `LastModifiedTime` | Öğeler tablosundaki gibi. |

### Takvim (Calendar) { #calendar }

| Alan | Yorumlama |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Öğeler tablosundaki gibi: `Appointment` veya `MeetingMessage`. |
| `StartTime`, `EndTime` | Randevunun başladığı ve bittiği zaman (UTC); yinelenen bir randevu için ilk yinelemeninki. |
| `DurationMinutes` | Dakika cinsinden süresi. |
| `Location` | Yeri. |
| `AllDay`, `Recurring` | Tüm gün süren bir etkinlik olup olmadığı ve yinelenip yinelenmediği (özel durumları gömülü öğelerdir). |
| `BusyStatus` | `Free`, `Tentative`, `Busy`, `OutOfOffice` veya `WorkingElsewhere`. |
| `ResponseStatus` | Kullanıcının yanıtı: `None`, `Organized`, `Tentative`, `Accepted`, `Declined` veya `NotResponded`. |
| `Organizer` | Öğeyi gönderenin adı. |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | Öğenin listelediği haliyle katılımcılar. |
| `GlobalObjectID` | Toplantının posta kutuları arasındaki kimliği (onaltılık): düzenleyicinin ve katılımcıların takvimlerindeki aynı toplantı. |
| `ReminderSet`, `ReminderTime` | Bir anımsatıcı ayarlanıp ayarlanmadığı ve anımsatıcının ait olduğu zaman (UTC). |
| `CreationTime`, `LastModifiedTime` | Öğeler tablosundaki gibi. |

### Görevler (Tasks) { #tasks }

| Alan | Yorumlama |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Öğeler tablosundaki gibi: `Task` veya `TaskRequest`. |
| `Status` | `NotStarted`, `InProgress`, `Complete`, `Waiting` veya `Deferred`. |
| `PercentComplete` | Görevin ne kadarının tamamlandığı, 0 ile 100 arasında. |
| `StartTime`, `DueTime` | Görevin başlangıç ve bitiş tarihi: kullanıcının saat dilimindeki tarihler, fark belirtilmeden. |
| `CompletedTime` | Görevin tamamlandığı zaman (UTC). |
| `Complete` | Görevin tamamlanıp tamamlanmadığı. |
| `Owner`, `Assigner` | Görevin sahibi ve görevi atayan kişi. |
| `ReminderSet`, `ReminderTime` | Takvim tablosundaki gibi. |
| `CreationTime`, `LastModifiedTime` | Öğeler tablosundaki gibi. |

### Özellikler (Properties) { #properties }

| Alan | Yorumlama |
|---|---|
| `ItemID` | Öğenin kimliği; deponun kendi özellikleri için `store`, bir klasörünkiler için `folder:<FolderID>`. |
| `Scope` | Özelliğin kime ait olduğu: `Store`, `Folder`, `Item`, `Recipient` veya `Attachment`. |
| `Index` | Alıcının veya ekin dizini; diğerleri için boş. |
| `PropertyTag` | Özelliğin kimliği ve türü, `0x0037001F` biçiminde (`PidTagSubject`, bir dize). |
| `PropertyName` | Adlandırılmış bir özellik için (`0x8000` ve üzerindeki kimlikler) özellik kümesi ve adı: `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords` ya da iyi bilinen bir küme değilse kümenin GUID'si. |
| `Type` | Özellik türü: `PtypString`, `PtypInteger32`, `PtypTime`, `PtypBinary`, `PtypMultipleString`, …. |
| `Value` | Değer: dizeler için metin, zamanlar için ISO 8601 UTC (`-Z` bunları dönüştürmez), ikili değerler için onaltılık (ilk 4.096 bayt, ardından `…(N bytes)`), çok değerli bir özelliğin değerleri için bir liste (CSV'de aralarında `, ` bulunur), gömülü bir öğe veya bir ekin verisi için `(object)`, kodu çözülmeyen türde bir değer için bir notla birlikte onaltılık baytlar, hasarlı bir değer için ise kurtarılabilen kısmı ve ardından onaltılık baytlarını içeren bir not (hiçbir şey kurtarılamadıysa yalnızca baytlar ve not). |

## Otomatik ayrıştırma { #automatic-parsing }

`auto-parse`, tabloları yeni `Communication` kategorisine `Outlook-Files.csv`,
`Outlook-Folders.csv`, `Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`,
`Outlook-Contacts.csv`, `Outlook-Calendar.csv` ve `Outlook-Tasks.csv` olarak (`-t json` veya
`-t jsonl` ile tek bir `Outlook.json` ya da `Outlook.jsonl` dosyası olarak) ve
`Communication/Outlook-logs.txt` ile birlikte yazar. Ayrıştırıcıya kendi `--code-page` değerini
iletir; Özellikler tablosunu eklemez ve ekleri dışarı çıkarmaz: bunlar için `parse-file outlook`
komutunu çalıştırın. Birleştirilmiş `-j` zaman çizelgesi her satırı `Comp` ve `Artifact` tanımlayıcısı
`outlook` ile birlikte içerir. `-Z`, bir görevin `StartTime` ve `DueTime` değerleri dışında adı
`Time` ile biten her alanı dönüştürür.

## Daha fazla ayrıntı { #further-details }

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Personal Folder File (PFF) format](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
