# Olay Günlüğü Oyma { #carve-evtx }

Temizlenen veya silinen bir günlükten geriye kalan olay günlüğü chunk'larını bir disk imajından, ayrılmamış alandan, sayfa dosyasından (pagefile) veya bir bellek imajından kurtarın. Her 64 KiB'lık EVTX chunk'ı kendi dize ve şablon tablolarını taşıdığından kayıtları günlüğün geri kalanı olmadan okunabilir. Juntei kurtarılan chunk'ları, Hayabusa veya Olay Görüntüleyicisi ile açılabilen sıradan `.evtx` dosyaları olarak, kanal başına bir dosya halinde yazar.

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
```

`-o` verilmezse yalnızca özet gösterilir.

## Chunk'ları bulma { #finding-chunks }

- Chunk imzası her bayt ofsetinde aranır ve her başlığın tutarlı görünmesi gerekir (başlık boyutu, kayıt numaraları, ofsetler). Sağlama toplamları raporlanır ama şart koşulmaz: kısmen üzerine yazılmış bir chunk yine de okunabilir kayıtlar içerir.
- NTFS sıkıştırmalı günlüklerin chunk'larının sıkıştırması açılır (LZNT1, 16 kümelik birimler). Disk imajında birimin küme boyutu kullanılır; diğer dosyalarda `--cluster-size <BYTES>` geçerlidir (varsayılan 4096).
- Kayıtlar, başlıktaki son kayıt ofsetine kadar boyutlarına göre sırayla izlenir; yinelenen bir kayıt kimliği bu taramayı hiçbir zaman durdurmaz. Hasarlı bir kayıttan sonra tarama bir sonraki geçerli kayıttan devam eder. Chunk'ın slack alanındaki eski kayıtlar da listelenir.

## Disk imajları ve NTFS birimleri { #disk-images }

E01 veya VHDX imajında ya da NTFS içeren ham bir disk veya birimde her chunk, bulunduğu kümeye göre sınıflandırılır:

| Konum | Anlamı |
|---|---|
| `FreeSpace` | Hiçbir dosyanın kullanmadığı bir küme. |
| `DeletedLog` | Silinmiş bir günlüğün `$MFT` kaydının hâlâ listelediği boş bir küme veya bu kümelerden yeniden oluşturulan bir günlük. |
| `LiveLog` | Kullanımdaki bir olay günlüğüne ait küme. `--keep-live` verilmedikçe dışarıda bırakılır. |
| `AllocatedFile` | `pagefile.sys` veya bir gölge kopyanın depo dosyası gibi başka bir dosyaya ait küme. |
| `AllocatedUnknown` | Hiçbir `$MFT` kaydının listelemediği ayrılmış bir küme. |
| `OutsideVolumes` | Diskteki tüm NTFS birimlerinin dışında. |
| `Unclassified` | Girdide NTFS birimi yoktur (sayfa dosyası veya bellek imajı). |

`$MFT` kaydı yalnızca boş kümeleri listeleyen silinmiş günlükler, sıkıştırılmış olanlar dahil eksiksiz olarak yeniden oluşturulur ve `Recovered/` altına kaydedilir. `--free-space-only` yalnızca boş kümelerde arama yapar: daha hızlıdır, ancak sayfa dosyasını, gölge kopyaları ve diğer dosyaları atlar.

## Tekilleştirme { #deduplication }

Her chunk yalnızca bir kez yazılır. Bir chunk canlı bir günlüğe aitse, başka bir chunk ile aynıysa veya tüm kayıtları canlı günlüklerde ya da daha önce yazılmış bir chunk'ta bulunuyorsa dışarıda bırakılır. Daha fazla kayıt içeren chunk'lar önce değerlendirilir; böylece bir chunk'ın daha eski ve daha kısa bir kopyası yeni bir şey eklemez. NTFS içermeyen bir girdi için canlı günlükleri `--live-logs <DIR>` ile belirtin.

## Sonuçlar { #results }

- `Carved/<CHANNEL>.evtx`: Kanal başına bir günlük; Windows'un günlükleri adlandırdığı biçimde adlandırılır (`Microsoft-Windows-Sysmon%4Operational.evtx`). Chunk başlıklarındaki kayıt numaraları yeniden numaralandırılır ve sağlama toplamları yeniden hesaplanır. Hasarlı bir kayıt ve ondan sonraki her şey slack olur. Bunun dışında her chunk, slack alanı dahil bütün olarak korunur; böylece Hayabusa'nın `-x` seçeneği gibi kayıt kurtarma yöntemleri bu dosyalarda da çalışır.
- `Recovered/Volume<N>/...`: `$MFT` kayıtlarından yeniden oluşturulan silinmiş günlükler.
- `chunks.csv` / `chunks.jsonl`: Bulunan her chunk: nerede olduğu, nasıl işlendiği (`Status`), kanalı, kayıt kimlikleri, zamanları ve sağlama toplamı durumu.
- `records.csv` / `records.jsonl`: Yazılan chunk'ların tüm kayıtları. `OnlyInCarvedData`, canlı günlüklerde bulunmayan kayıtları işaretler.

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

Oyma yalnızca bitişik chunk'ları bulur. Bitişik olmayan kümelere veya bellekte bitişik olmayan fiziksel sayfalara bölünmüş bir chunk eksiktir ve sonraki kayıtları hasarlı ya da kayıp olabilir. `hiberfil.sys` dosyasının sıkıştırmasını önce açın. Çıkış kodu, arama tamamlandığında 0, hata durumunda 1'dir.

## Ayrıntılar { #further-details }

- [Ayrıntılı kullanım ve alan başvurusu (İngilizce)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.md)
- [Ayrıntılı kullanım ve alan başvurusu (Japonca)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.ja.md)
- [EVTX biçimi (libevtx belgeleri)](https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc)
