# RecentDocs { #recentdocs }

## RecentDocs nedir { #what-recentdocs-is }

Windows Explorer, dosya ve klasörlerin en son kullanılanlar (MRU) listelerini her kullanıcının `NTUSER.DAT` kovanında saklar.
Kök liste tüm türlerdeki son öğeleri kapsar; alt anahtarlar `.docx` gibi uzantıları gruplar ve ayrıca bir
`Folder` listesi bulunur. `MRUListEx` numaralı değerlerin sırasını belirtir.

RecentDocs, bir kullanıcının etkileşimde bulunduğu belgeleri, daha sonra silinmiş veya taşınmış dosyalar da dahil
olmak üzere belirlemek için faydalıdır. Değerler ayrıca Windows'un kullanıcının Recent klasöründe kullandığı kısayol
adını da içerebilir; bu ad LNK kanıtlarıyla ilişkilendirilebilir.

<span class="evidence-strength">En iyi sonuç:</span> Explorer, öğeyi o kullanıcının
RecentDocs listesine kaydetmiştir. Anahtar zamanı yalnızca o belirli listedeki en son öğenin tarihini belirler.

## Konum { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

İlgili her profil için `NTUSER.DAT` kovanını ve işlem günlüklerini, ayrıca kullanıcının Recent
LNK dizinini toplayın.

## Analist için uyarılar { #analyst-cautions }

- `MRUListEx` her öğe için bir zaman damgası değil, sıra verir.
- Anahtarın son yazılma zamanı en son güncellemeye karşılık gelir; yalnızca MRU konumu 0 bundan türetilen
  savunulabilir bir `LastOpenedUTC` değeri alır.
- Bir öğe hem kök listede hem de bir uzantı listesinde görünebilir. Bunu iki açılış olarak saymayın.
- Uygulama ve Explorer davranışı değişebilir; bazı erişim yöntemleri RecentDocs'u güncellemeyebilir.
- Temizleme/gizlilik araçları ve liste boyutu sınırları daha eski kanıtları kaldırır.
- Bir dosya adı saldırganın denetiminde olabilir ve içerik türünün kanıtı değildir.
- Kullanıcı çıkarımı profil yollarından veya kabuk klasörü verilerinden gelir; boş/belirsiz kullanıcı adlarını inceleyin.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

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
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host` | Mevcut olduğunda toplama/sistem bağlamından türetilen ana bilgisayar. |
| `User` | Yoldan veya kabuk klasörü yollarından çıkarılan profil sahibi; boş olduğunda belirsizliği koruyun. |
| `SourcePath` | Girdi `NTUSER.DAT` kovanı. Kullanıcıları birleştirirken vazgeçilmezdir. |
| `Extension` | Alt liste (`.docx`, `Folder` vb.); kök RecentDocs listesi için null. |
| `MRU-Position` | Sıfır, bu liste içindeki en son öğedir. Bu, geçen süre değil, göreli sıradır. |
| `ValueIndex` | `MRUListEx` tarafından başvurulan numaralı kayıt defteri değeri. Yalnızca yapısal bir pivot noktasıdır. |
| `Filename` | Kodu çözülmüş öğe adı. Tam yolu LNK, Jump List, MFT veya uygulama verileri aracılığıyla ilişkilendirin. |
| `LinkName` | Kabuk öğesinde kodlanmış kısayol adı; genellikle kullanıcının Recent dizininde bulunur. |
| `KeyLastWrittenUTC` | Liste anahtarının en son değiştiği zaman. Her satıra değil, listeye uygulanır. |
| `LastOpenedUTC` | Yalnızca anahtar zamanının bu yorumu desteklediği MRU konumu 0 için doldurulur. |
| `ValueDataRaw` | Kod çözme başarısız olduğunda veya kanıtın doğrulanması gerektiğinde ham baytlar. |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri günlüklerinin yeniden oynatılıp oynatılmadığı ve hangilerinin oynatıldığı. |
| `Problems` | Eksik MRU değerleri, listede olmayan değerler, yanlış türler veya kod çözme hasarı. |

## İnceleme stratejisi { #investigation-strategy }

1. Her kullanıcının kök ve uzantı listelerini ayırın; aynı öğenin yinelenen kayıtlarını dikkatle ayıklayın.
2. Sıralama için MRU konumunu, zaman damgalı bir iddia için ise yalnızca `LastOpenedUTC` değerini kullanın.
3. `LinkName`/`Filename` değerlerini Recent LNK, Jump List, `$MFT` ve USN verileriyle eşleştirin.
4. Tuzak adları, çift uzantıları, ağ/çıkarılabilir hedefleri ve ilk erişim zamanına yakın belgeleri inceleyin.
5. Yürütme önemli olduğunda uygulama Prefetch/SRUM verileri ve olay günlükleriyle ilişkilendirin.

## Daha fazla ayrıntı { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [Eric Zimmerman'ın RECmd ve Registry Explorer araçları](https://github.com/EricZimmerman/RECmd)
- [SANS Windows Forensic Analysis posteri](https://www.sans.org/posters/windows-forensic-analysis/)
