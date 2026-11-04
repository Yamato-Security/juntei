# En Son Kullanılan (MRU) Listeleri (RunMRU, TypedPaths, WordWheelQuery, TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## Listeler nedir { #what-the-lists-are }

Windows ve programları, kullanıcının en son yazdığı veya açtığı öğelerin kısa listelerini kullanıcının
`NTUSER.DAT` dosyasında tutar:

- **RunMRU:** Çalıştır iletişim kutusuna (Win+R) yazılan komutlar: `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`.
- **TypedPaths:** Explorer'ın adres çubuğuna yazılan yollar.
- **WordWheelQuery:** Explorer'ın arama kutusunda aranan terimler.
- **TypedURLs:** Internet Explorer'ın adres çubuğuna yazılan adresler ve Windows 8'den itibaren
  her birinin ne zaman yazıldığını gösteren `TypedURLsTime`.
- **Paint ve WordPad:** her birinin en son açtığı dosyalar (`Recent File List`).
- **ACMru (Windows XP):** Arama Yardımcısı'nın (Search Assistant) arama türüne göre terimleri.

<span class="evidence-strength">En iyi sonuç:</span> kullanıcı bu komutu, yolu,
arama terimini veya adresi yazmıştır ya da program bu dosyayı açmıştır. Her listenin yalnızca en son girdisi
anahtarın son yazma zamanıyla tarihlendirilir; bunun istisnası, `TypedURLsTime` ile tarihlendirilen yazılmış adreslerdir.

## Konum { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

`ACMru`'nun alt anahtarları arama türleridir: `5603` dosya ve klasör adları, `5604` dosyalardaki sözcükler veya
ifadeler, `5647` bilgisayarlar ve kişiler, `5001` Internet.

Her profil için `NTUSER.DAT` dosyasını işlem günlükleriyle (transaction logs) birlikte toplayın.

## Analist için uyarılar { #analyst-cautions }

- **Liste başına tek zaman.** Bir anahtarın son yazma zamanı listesi değiştiğinde değişir; bu nedenle yalnızca
  en son girdiyi tarihlendirir (`IsMostRecent`, `LastUsedUTC`). Diğerleri sıralıdır ancak tarihsizdir.
  `TypedURLsTime` her yazılmış adresi tarihlendirir (`TypedTimeUTC`).
- **Yazılmış, çalıştırılmış değil.** RunMRU, başarıyla çalışıp çalışmadığına bakılmaksızın Çalıştır iletişim kutusuna
  yazılanı kaydeder. Windows her komutu sonuna `\1` ekleyerek saklar; Juntei bunu kaldırır. Prefetch,
  BAM, UserAssist ve işlem oluşturma olaylarıyla ilişkilendirin.
- **Listeler kısadır.** Her biri yalnızca en son girdilerini tutar; gizlilik ayarları ve temizleme
  araçları bunları siler. Silinmiş değerler için `parse-registry read --recover-deleted` kullanın.
- **Terimler yazılan şeylerdir.** WordWheelQuery ve ACMru bulunan sonucu değil, yazılan metni tutar.
- **Sıra listeden gelir.** RunMRU'nun `MRUList` ve WordWheelQuery'nin `MRUListEx` değerleri
  sırayı verir; diğerlerinde sırayı değerin numarası belirler (`url1`, `File1` ve `000` en son olanlardır).

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

Özet, liste başına girdileri ve tarihlendirilmiş son girdileri (yazılmış adresler ve her
listenin en son girdisi) en yeniden eskiye doğru listeler.

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Ana bilgisayar, profil sahibi (yoldan veya kabuk klasörlerinden), sistemin `ProfileList` anahtarından alınan SID ve kovan. |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad` veya `ACMru`. |
| `Subkey` | `ACMru`'nun alt anahtarı: arama türü (`5603`, `5604`, `5647`, `5001`). |
| `MRU-Position`, `IsMostRecent` | Listedeki yer (0 en son olandır); liste değeri adlandırmıyorsa boştur. |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | Komut (`\1` olmadan), yol, arama terimi, adres veya dosya. |
| `TypedTimeUTC` | Adresin ne zaman yazıldığı, `TypedURLsTime` değerinden. |
| `LastUsedUTC` | Anahtarın son yazma zamanı, yalnızca en son girdi için. |
| `KeyPath`, `KeyLastWrittenUTC` | Anahtar ve son yazma zamanı: her satırın değil, en son girdisinin zamanı. |
| `DataRaw` | Kodu çözülemediğinde değerin baytları (onaltılık). |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Listenin adlandırdığı ancak anahtarda bulunmayan değerler, listenin adlandırmadığı değerler, yanlış adlandırılmış değerler, yanlış türler, kesik terimler, FILETIME olmayan zamanlar. |

## İnceleme stratejisi { #investigation-strategy }

1. Kullanıcının yazdığı komutlar, betikler, ağ paylaşımları ve yönetim araçları için RunMRU'yu okuyun.
2. Kullanıcının aradığı klasörler ve dosyalar için TypedPaths ve WordWheelQuery'yi okuyun:
   kimlik bilgileri, dışa aktarımlar, arşivler, çıkarılabilir ve ağ konumları.
3. Yazılmış adresleri `TypedTimeUTC` ile zaman çizelgesine yerleştirin.
4. Prefetch, BAM, UserAssist, ShellBags, LNK dosyaları, Jump Lists ve tarayıcı geçmişiyle ilişkilendirin.

## Daha fazla ayrıntı { #further-details }

- [libyal winreg-kb: Most recently used (MRU) keys](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
