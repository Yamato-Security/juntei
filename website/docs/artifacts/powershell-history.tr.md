# PowerShell geçmişi { #powershell-history }

## Geçmişin kaydettikleri { #what-the-history-records }

PSReadLine, etkileşimli bir PowerShell konağında girilen komut satırlarını kaydeder. Dosya, komut başına zaman damgası olmadan metni kaydedilme sırasıyla içerir. Geçmiş düzenlenebilir, silinebilir, devre dışı bırakılabilir veya filtrelenebilir; bir komutun başarıyla tamamlandığını kanıtlamaz. Yürütmeyi ve zamanlamayı PowerShell olay günlükleri, süreç telemetrisi ve diğer yürütme artefaktlarıyla doğrulayın.

## Konum ve toplama { #location-and-collection }

Windows PowerShell 5.1 ve PowerShell 7+ için Windows'taki varsayılan konum şöyledir:

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

Diğer konaklar bu dizinde `Visual Studio Code Host_history.txt` gibi adlar kullanır. Linux ve macOS üzerinde PowerShell genellikle `~/.local/share/powershell/PSReadLine/<Host>_history.txt` kullanır. PSReadLine'ın `HistorySavePath` ayarı dosyayı başka bir konuma yönlendirebilir; bu varsayılan yollar keşif varsayımlarıdır, tüm konumları kapsamaz.

Dizin girdisi ve `auto-parse`, doğrudan bir `PSReadLine` klasöründeki `<Host>_history.txt` dosyalarını ASCII büyük/küçük harf ayrımı yapmadan özyinelemeli olarak seçer. Başka yerlerdeki rastgele `*_history.txt` dosyalarını seçmezler. Yeniden adlandırılmış dosyalar veya `PSReadLine` dizini dışındaki kopyalar için açıkça `-f` girdisi kullanın. Kullanıcı ilişkilendirmesi için mümkünse özgün profil yapısını koruyun.

## Juntei ile ayrıştırma { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

Varsayılan dosya biçimi CSV'dir. `-t csv`, `-t json` veya `-t jsonl` verilip `-o` verilmezse kayıtlar stdout'a yazılır. İki seçenek de verilmezse yalnızca özet gösterilir. JSON ayrı, okunabilir biçimlendirilmiş nesnelerden oluşur; JSONL her satıra bir nesne yazar. Var olan çıktının üzerine yazmak için `-C` ekleyin; kaynak dosyaya işaret eden alternatif adlar değiştirilmeye karşı korunur.

`auto-parse`, VHDX/E01 imajlarında da `ShellHistory/PowerShellHistory.csv` (veya seçilen JSON biçimini) kaydeder. Geçmiş, `auto-parse -j` veya `super-timeline` için olay üretmez.

## Kodlama ve satır işleme { #encoding-and-line-handling }

- Dosyanın başındaki UTF-8, UTF-16LE veya UTF-16BE bayt sırası işareti kodlamayı belirler ve kaldırılır. BOM yoksa girdi her zaman UTF-8 olarak işlenir. BOM'suz UTF-16 tahmin edilmez; NUL karakterleri korunur ve olası kodlama hasarı olarak bildirilir.
- Geçersiz UTF-8 dizileri ve eşleşmemiş UTF-16 vekil kod birimleri U+FFFD'ye dönüştürülür; hasar `Problems` içinde bildirilir. Sonda kalan tek UTF-16 baytı ve okuma hataları da bildirilir; okunabilir satırlar korunur. Eksiksizliğe güvenmeden önce hem `Problems` alanını hem ayrıştırıcı günlüğünü inceleyin.
- Dosyalar akış halinde işlenir. Fiziksel satır başına en fazla 1,048,576 girdi baytı tutulur; BOM ve LF ayırıcı hariçtir (LF öncesindeki CR sınıra dahildir). Daha uzun satırlar tam bir karakter sınırında kesilir, kalan baytlar tüketilir ve `Problems` atlanan bayt sayısını bildirir. Sınır karakter sayısına değil, kodlanmış girdi baytlarına uygulanır.
- LF ve CRLF satır sonları kaldırılır. Sonlandırılmamış satırın sonundaki CR de kaldırılır. Diğer boşluklar ve sekmeler korunur. Boş satırlar kayıt üretmez, ancak satır numaralarında sayılır ve devam zincirini sonlandırır; yalnızca boşluk içeren satırlar kayıt olarak kalır. Okuma hataları, okumanın durduğu yeri gösteren boş bir tanılama satırı üretebilir.
- PSReadLine çok satırlı girdileri, devam eden her fiziksel satırın sonunda ters tırnakla saklar. Satırlar ayrı kalır; ters tırnaklar ve girintiler korunur. `EntryStartLine`, önceki satırları ters tırnakla biten ardışık zincirin ilk satırını gösterir. Kabuk sözdizimi yorumlanmaz ve komutlar yeniden oluşturulmaz. Kesilme, devam ilişkilendirmesini etkileyebilir; bu satırları gruplamadan önce `Problems` alanını inceleyin.

## Çıktı alanları { #output-fields }

| Alan | Yorum |
|---|---|
| `SourceFile` | Dizin girdisine göre göreli yol; `/` kullanır. `-f` için yalnızca dosya adı. Mutlak kaynak yolu dışa aktarılmaz. |
| `User` | Korunan yoldan çıkarılan profil adı. Dizin girdisi, girdi kökü altındaki `Users`/`Documents and Settings` yapılarını da tanır; `-f` olağan PSReadLine profil yolu son ekini gerektirir. Bilinmeyen değerler CSV'de boş, JSON'da `null` olur. |
| `ShellHost` | `_history.txt` öncesindeki dosya adı öneki; örneğin `ConsoleHost`. Bu son eki taşımayan yeniden adlandırılmış dosyalarda bilinmez. |
| `LineNumber` | Atlanan boş satırlar dahil, 1'den başlayan fiziksel satır numarası. |
| `EntryStartLine` | Devam zincirinin başlangıç fiziksel satırı veya bu satırın kendi numarası. |
| `Command` | Satır sonu çıkarılmış, kodu çözülmüş fiziksel satır; belgelenen sınır ve CSV korumasına tabidir. |
| `FileLastWriteTime` | Her satırda tekrarlanan kaynak dosya değiştirme zamanı. **Dosya meta verisidir, asla komut yürütme zamanı değildir.** İmaj ayrıştırması, hazırlama zamanı yerine özgün kaynak meta verisini kullanır. |
| `FileSize` | Kaynak dosyanın bayt cinsinden boyutu. |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE` veya `UTF-16BE`. |
| `Problems` | Satır hasarı ve kesilme tanılamaları; CSV'de noktalı virgülle ayrılır, JSON/JSONL'de dizidir. Dosya düzeyindeki tanılamalar günlükte de görünür. |

`FileLastWriteTime` varsayılan olarak ISO 8601 biçiminde UTC kullanır; örneğin `2026-10-20T08:46:38.5Z`. Gösterilen saat farkını değiştiren `-Z, --timezone` seçeneğini yalnızca `auto-parse` kabul eder. Bağımsız `parse-file powershell-history` komutunda saat dilimi seçeneği yoktur.

Her CSV metin hücresi elektronik tablo formülü önekleri açısından denetlenir (`=`, `+`, `-`, `@`; tam genişlikli biçimler ve boşluk veya belirli görünmez/denetim karakterlerinden sonraki önekler dahil). Etkilenen hücrelerin ve denetim karakteriyle başlayan hücrelerin başına kesme işareti eklenir. Bu, CSV'deki komut ve yol metnini değiştirebilir. JSON ve JSONL, CSV kesme işareti olmadan kodu çözülmüş metni korur; kod çözme sırasında değiştirme ve kesilme yine uygulanır. Kaydedilmiş komut metnini karşılaştırırken JSONL kullanın ve kaynak geçmiş dosyalarını her zaman koruyun.
