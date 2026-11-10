# USN Günlüğü (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## USN Journal nedir { #what-the-usn-journal-is }

NTFS, dosya ve dizin değişikliklerini Güncelleme Sıra Numarası (Update Sequence Number, USN) Değişiklik
Günlüğü'ne kaydeder. Bir kayıt oluşturmayı, verinin genişletilmesini/üzerine yazılmasını, öznitelik
veya güvenlik değişikliklerini, yeniden adlandırmayı, kapatmayı ve silmeyi tanımlayabilir. Bir dosya
başvurusu, üst dizin başvurusu, ad, nedenler, öznitelikler, zaman damgası ve monoton olarak artan bir
USN saklar.

Günlük, özellikle dosyalar silinmişse, saldırganın art arda gerçekleştirdiği dosya işlemlerini yeniden
oluşturmak için mükemmeldir. Değişen baytları değil, **meta verinin/içeriğin değiştiğini** kaydeder.

<span class="evidence-strength">En iyi sonuç:</span> NTFS, başvurulan dosya için belirtilen değişiklik
nedenlerini zaman damgasındaki anda kaydetti. Nedenler çoğu zaman bir `CLOSE` kaydına kadar birikir.

## Konum ve toplama { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J`, NTFS meta verisinin adlandırılmış bir akışıdır. Ham erişimi destekleyen bir adli bilişim aracıyla
toplayın. USN kayıtları tam yollar yerine üst dosya başvurularını sakladığından, aynı birimin `$MFT`
dosyasını da toplayın.

## Analist için uyarılar { #analyst-cautions }

- Günlük döngüseldir ve boyutu sınırlıdır. Eski kayıtların üzerine yazılır.
- `FILE_DELETE`, NTFS'nin silme işlemini gerçekleştirdiğini söyler; içerik başka bir yerde
  kurtarılabilir durumda kalabilir.
- `DATA_OVERWRITE` gibi bir neden, baytları ya da sorumlu işlemi veya kullanıcıyı ortaya koymaz.
- Bir kayıtta birkaç neden birleştirilebilir; `CLOSE` çoğu zaman biriken değişikliklerin sonunu
  işaretler.
- Yollar yeniden oluşturulur. Üst dizinler yeniden adlandırılmış, silinmiş veya yeniden kullanılmış
  olabilir.
- Dosya başvurusu sıra numaraları önemlidir. Girdi numarası tek başına yanlış nesille
  ilişkilendirilebilir.
- Oyma (carving), geçerli kayıtları özgün günlük bağlamları olmadan kurtarabilir; bu ayrımı raporlayın.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## CSV çıktı alanları { #csv-output-fields }

| Alan | Yorumlama |
|---|---|
| `Name` | Bu USN kaydında saklanan dosya/dizin adı. Tam bir yol değildir. |
| `Extension` | `Name` değerinden ayrıştırılan kolaylık amaçlı uzantı; güvenilir bir içerik türü olarak kullanmayın. |
| `EntryNumber`, `SequenceNumber` | Dosyanın NTFS başvurusu. Kaydın neslini belirlemek için ikisini birlikte kullanın. |
| `ParentEntryNumber`, `ParentSequenceNumber` | Değişiklik anında üst dizinin NTFS başvurusu. |
| `ParentPath` | Juntei'nin zamanı dikkate alarak yeniden oluşturduğu üst yol. Bilinmeyen işaretleri, üst öğe zincirinin çözümlenemediği anlamına gelir. |
| `UpdateSequenceNumber` | USN sıralama değeri. Kararlı sıralama ve Amcache/MFT pivotları için yararlıdır. |
| `UpdateTimestamp` | V2/V3 kayıtlarında saklanan zaman damgası. Oyma bağlamından gelen zaman damgalarını da her kayıtta olduğu gibi doğrulayın. |
| `UpdateReasons` | `|` ile birleştirilmiş bir veya daha fazla adlandırılmış değişiklik bayrağı. Birleşimleri ve `CLOSE` değerini birlikte yorumlayın. |
| `FileAttributes` | Değişiklik anında kaydedilen NTFS öznitelikleri; bilinmeyen bitler onaltılık olarak kalır. |
| `OffsetToData` | Girdideki bayt ofseti. Doğrulama ve oyulmuş kayıtlar için vazgeçilmezdir. |
| `SourceFile` | Girdi günlüğü/imajı; ana bilgisayarları ve birimleri birbirinden ayırır. |
| `SourceInfo` | Değişikliğin günlüğe nasıl girdiğini açıklayan kaynak bayrakları. Genellikle bağlamdır, atıf değildir. |
| `SecurityID` | Kayıt için NTFS güvenlik tanımlayıcısı indeksi; bir kullanıcı SID'i değildir. |
| `Version` | USN kayıt sürümü. Juntei CSV'ye V2/V3 yazar; JSON, V4 aralık izleme kapsamlarını (extent) korur. |

JSON/JSONL, 128 bit V3 başvurularını doğru şekilde temsil eder ve V4 aralık izleme kayıtlarını ve
kapsamlarını korur. Düşük düzeyli, sürüme özgü alanlar önemli olduğunda bunu kullanın.

## İnceleme stratejisi { #investigation-strategy }

1. Zamana, ada, uzantıya, üst yola veya dosya başvurusuna göre filtreleyin.
2. Günlükteki kesin sırayı görmek için zamanın yanı sıra `UpdateSequenceNumber` değerine göre de
   sıralayın.
3. Taşımalar/yeniden adlandırmalar için `RENAME_OLD_NAME` ile `RENAME_NEW_NAME` kayıtlarını eşleştirin.
4. Kayıt durumunu, yolları, boyutları ve zaman damgalarını doğrulamak için `$MFT` kullanın.
5. İşlem atfını olay günlükleri, EDR, Prefetch, hizmetler/görevler veya Defender ile ilişkilendirin.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Change Journal neden bayrakları](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [ys-ntfs-parser'ın usn crate'i](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
