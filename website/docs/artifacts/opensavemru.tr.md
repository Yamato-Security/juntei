# Aç ve Farklı Kaydet İletişim Kutuları (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## İletişim kutusu geçmişi nedir { #what-the-dialog-history-is }

Bir program ortak Aç veya Farklı Kaydet iletişim kutusunu gösterdiğinde, Windows kullanıcının
`NTUSER.DAT` dosyasına şunları kaydeder:

- **hangi dosyanın açıldığı veya kaydedildiği:** `OpenSavePidlMRU`, her uzantı (`docx`, `ps1`, …)
  için bir alt anahtar ve tüm uzantılar için `*` içerir; her biri en son kullanılanlar (MRU)
  listesidir;
- **iletişim kutusunu hangi programın kullandığı ve hangi klasörde olduğu:** `LastVisitedPidlMRU`
  (ve eski iletişim kutularını kullanan programlar için `LastVisitedPidlMRULegacy`);
- **iletişim kutusunu gösteren programlar:** `CIDSizeMRU` (program başına iletişim kutusunun boyutu)
  ve `FirstFolder` (bir programın yolu ve bir klasör);
- Windows 2000 ve XP'de `OpenSaveMRU` (uzantı başına yollar) ve `LastVisitedMRU` (programlar ve
  klasörleri, yol olarak).

Windows Vista'dan bu yana dosyalar ve klasörler, bir kısayolun hedefiyle aynı yapılar olan shell item
listeleridir: her öğe yolun bir parçasıdır (Bu Bilgisayar, bir sürücü, bir klasör, dosya) ve bir dosya
veya klasör öğesi kendi zamanlarını ve NTFS dosya başvurusunu taşıyabilir.

<span class="evidence-strength">En iyi sonuç:</span> bir program, bu kullanıcının oturumunda bu
dosyayı açmak veya kaydetmek için ya da bu klasörde Aç veya Farklı Kaydet iletişim kutusunu kullandı.
Her listenin yalnızca en son girdisi tarihlidir: anahtarının son yazılma zamanı.

## Konum { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

Her profil için `NTUSER.DAT` dosyasını işlem günlükleriyle birlikte toplayın.

## Analist için uyarılar { #analyst-cautions }

- **Liste başına tek bir zaman.** Bir anahtarın son yazılma zamanı, girdilerinden biri listenin
  başına geçtiğinde değişir; dolayısıyla yalnızca 0 konumundaki girdiyi tarihlendirir
  (`IsMostRecent`, `LastUsedUTC`). Diğer girdiler listede yeniden eskiye doğru sıralıdır, ancak
  tarihsizdir.
- **Aynı dosya iki listede.** Bir dosya hem kendi uzantısının listesinde hem de `*` içindedir; bunu
  iki kullanım olarak saymayın. `*`, uzantılar arasındaki sırayı verir.
- **Açıldı mı, kaydedildi mi.** Listeler hangisi olduğunu söylemez. Dosyanın `$MFT` ve USN Journal
  kayıtları, LNK dosyaları ve jump list'lerle ilişkilendirin.
- **Öğe zamanları iletişim kutusuna değil, dosyaya aittir.** `ItemModifiedUTC`, `ItemCreatedUTC` ve
  `ItemAccessedUTC`, öğe yazıldığında dosyanın sahip olduğu zamanlardır ve 2 saniye hassasiyetindedir
  (FAT). Bilinen klasörlerin öğelerinde (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …) çoğu zaman
  hiç zaman yoktur. Bu FAT zaman damgalarında saat dilimi ofseti yoktur. `UTC` sütun soneki bir
  görüntüleme kuralıdır; özgün saatin UTC olduğunun kanıtı değildir. Bunları kayıt defteri
  FILETIME'larıyla karşılaştırmadan önce kaynak sistemin saat dilimi ve diğer zaman damgalarıyla
  ilişkilendirin. Microsoft'un
  [dosya zamanı kılavuzuna](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times) bakın.
- **Yollar kabuk yollarıdır.** Masaüstünden başlarlar: `This PC\C:\Users\…`, bilinen bir klasör, bir
  kitaplık (`UsersLibraries\Documents\…`) veya bir ağ konumu. Bir klasörle veya dosyayla başlayan bir
  yol masaüstündedir. Boş bir yol masaüstünün kendisidir.
- **Kodu çözülmeyen öğeler.** Shell item ayrıştırıcısının bilmediği türde bir öğe sınıfıyla
  (`[class 0x00]`) gösterilir, türü `unknown` olur ve listenin baytları `DataRaw` içinde yer alır.
- **Programlar addır.** `LastVisitedPidlMRU` ve `CIDSizeMRU` yol değil, bir dosya adı
  (`WINWORD.EXE`) tutar. `FirstFolder` bir yol tutar.
- **Listeler kısadır ve temizlenir.** Her liste sınırlı sayıda girdi tutar ve temizleme araçları
  anahtarları siler. Silinmiş değerler için `parse-registry read --recover-deleted` kullanın.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

Özet; liste başına girdileri, iletişim kutularını kullanan programları ve her kovanın en son dosyasını
(`*` listesinin ilk girdisi) listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Ana bilgisayar, profil sahibi (yoldan veya kabuk klasörlerinden), sistemin `ProfileList` kaydından gelen SID ve kovan. |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU` veya `LastVisitedMRU`. |
| `Extension` | Uzantının alt anahtarı (`docx`, `*`); diğer listeler için boş. |
| `MRU-Position`, `IsMostRecent` | Anahtarın listesindeki yer (0 en yenisidir); liste değeri adlandırmadığında boş. |
| `ValueName` | Değerin numarası (`MRUListEx`) veya harfi (`MRUList`). |
| `Program` | İletişim kutusunu gösteren program. |
| `Path` | Dosya veya klasör: shell item adlarının birleşimi ya da metin olarak saklanan yol. |
| `ItemType` | Son shell item'ın türü: `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | Shell item'ın kaydettiği şekliyle dosyanın veya klasörün zamanları. |
| `MFT-Entry`, `MFT-Sequence` | Shell item'ın kaydettiği NTFS dosya başvurusu. |
| `LastUsedUTC` | Yalnızca en son girdi için anahtarın son yazılma zamanı. |
| `KeyPath`, `KeyLastWrittenUTC` | Anahtar ve son yazılma zamanı: her satırın değil, en son girdisinin zamanı. |
| `DataRaw` | Kodu çözülemediğinde değerin baytları (onaltılık) veya bir öğenin türü bilinmediğinde shell item listesi. |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Listenin adlandırdığı ancak anahtarda olmayan değerler, listenin adlandırmadığı değerler, yanlış türler, kesik adlar veya listeler. |

## İnceleme stratejisi { #investigation-strategy }

1. Her kullanıcı için `*` listesini sırasıyla ve en son dosya için `LastUsedUTC` değerini okuyun.
2. Kullanıcının yazabildiği, çıkarılabilir veya ağ konumlarındaki betikleri, arşivleri, yürütülebilir
   dosyaları ve belgeleri ve kimlik bilgisi veya dışa aktarma gibi adlandırılmış dosyaları arayın.
3. Bir programı en son kullandığı klasöre bağlamak için `LastVisitedPidlMRU`, iletişim kutularını hiç
   kullanmış programlar için ise `CIDSizeMRU` ve `FirstFolder` kullanın.
4. Yolları ve `MftEntry` değerini `$MFT`, USN Journal, LNK dosyaları, jump list'ler, ShellBags ve
   RecentDocs ile ilişkilendirin.

## Daha fazla ayrıntı { #further-details }

- [libyal winreg-kb: En son kullanılan (MRU) anahtarlar](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: Windows Shell Item biçimi](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
