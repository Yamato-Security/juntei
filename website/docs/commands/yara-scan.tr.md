# YARA Taraması { #yara-scan }

Toplanmış dosyaları taşınabilir Rust motoruyla tarayın. Windows, macOS ve Linux desteklenir. Dosyalar yalnızca veri olarak okunur, hiçbir zaman çalıştırılmaz. Bu komut otomatik bulgu ayrıştırmasından bağımsızdır.

## Kuralları seçme { #choose-rules }

Varsayılan Core doğruluk ve performansa öncelik verir. Extended avcılık kapsamını genişletir. Full en geniş kapsama sahiptir, ancak daha fazla yanlış pozitif üretebilir.

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
```

`--rules <FILE_OR_DIRECTORY>`: İndirme yapmadan yerel UTF-8 kural dosyası veya dizini kullanın. Dizinler özyinelemeli aranır. Include işlemleri seçilen kural dizini içinde kalır ve bağımlılık sırasıyla bir kez genişletilir. Geçersiz veya desteklenmeyen kurallar sessizce atılmak yerine derlemeyi durdurur.

## Sınırlar ve kapsam { #limits-and-scope }

Karantinadan kurtarılan içerikler dahil tüm normal dosyalar uygundur. Sembolik bağlantılar ve özel dosyalar atlanır. Arşivler ve disk görüntüleri açılmaz; canlı bellek taranmaz. Sabit çıkarılmış dosyalar ve kanıtların dışında yeni bir çıktı dizini kullanın.

- `-T, --threads <N>`: Paralel tarama sayısı; varsayılan CPU sayısıdır, en fazla 8.
- `--timeout <SECONDS>`: Dosya başına değerlendirme zaman aşımı, saniye olarak; varsayılan 60.
- `--max-file-mib <MIB>`: MiB olarak isteğe bağlı en büyük dosya boyutu; varsayılan sınırsızdır. Ayrı ayar verilmezse MFT dosyalarına da uygulanır. Belirlenen sınırı aşan dosyalar atlandı olarak raporlanır.
- `--rules-release YYYYMMDD`: Forge sürüm tarihi veya varsayılan latest. Tekrarlanabilirlik için indirilen kaynaklar ve özetleri saklanır.

## Sonuçlar { #results }

- `YaraDetections.csv` / `YaraDetections.jsonl`: Göreli yollar, kanıt bilgisayar adları, özetler, meta veriler ve sınırlı eşleşme konumlarıyla kural/dosya eşleşmeleri. Eşleşen baytlar dışa aktarılmaz.
- `YaraFiles.csv` / `YaraFiles.jsonl`: Atlamalar, okuma hataları, zaman aşımları ve kısmi sonuçlar dahil dosya başına kapsam.
- `Summary.json`: Tamamlanma durumu, ayarlar, sayılar, motor sürümü ve kaynak özetleri.
- `Rules.yar`: İndirilen kuralların aynısı veya include işlemleri genişletilmiş yerel kural anlık görüntüsü.
- `CompilationWarnings.txt` / `CompilationError.txt`: Derleme uyarıları veya derleme başarısız olduğunda hata.

Çıkış kodları: 0 tamamlandı, 3 eksik kapsam, 2 kurulum/çıktı hatası, 130 iptal anlamına gelir. Eşleşme bir araştırma ipucudur, ihlal kanıtı değildir; eşleşme olmaması da güvenliği kanıtlamaz. Kural başlığını ve bulgu bağlamını inceleyin. İçeriği tanımadan önce karantinayı çözün.

## MFT eşleşmeleri ve kurtarma { #mft-recovery }

MFT dosyaları varsayılan boyut sınırı olmadan tek tek ham baytlar olarak taranır. Eşleşme konumlarındaki kayıtlar ayrıştırılır ve yerleşik DATA akışları bağımsız taranır. Yalnızca tek başına eşleşen akışlar kurtarılır. Meta veri, kayıtlar arası ve yerleşik olmayan eşleşmeler içerik çıkarılmadan raporlanır.

`--max-file-mib <MIB>` → `--max-mft-mib <MIB>` · `--max-mft-recoveries <N>`

Varsayılan kurtarma sınırı çalıştırma başına 1000 akıştır. Sınırlar, hasarlı kayıtlar ve zaman aşımları görünürdür. Kaydedilen adlar karantina uzantısıyla biter; kanıt dizinleri yeniden oluşturulmaz.

`YaraMFTResults.csv` / `YaraMFTResults.jsonl` · `RecoveredMFT/` · `.quarantined`

## Ayrıntılar { #further-details }

- [Motor: Boreal](https://github.com/vthib/boreal)
- [Kural paketleri: YARA Forge](https://github.com/YARAHQ/yara-forge)
- [Ayrıntılı kullanım ve alan başvurusu (İngilizce)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.md)
- [Ayrıntılı kullanım ve alan başvurusu (Japonca)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.ja.md)
