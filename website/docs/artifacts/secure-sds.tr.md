# Güvenlik Tanımlayıcıları (`$Secure:$SDS`) { #security-descriptors-securesds }

## `$SDS` nedir { #what-sds-is }

NTFS üzerindeki her dosya ve dizinin bir **güvenlik tanımlayıcısı** vardır: sahibi, birincil grubu, bir
**DACL** (onunla kimin ne yapabileceği) ve bir **SACL** (hangi erişimin denetlendiği ve bütünlük etiketi).
NTFS her farklı tanımlayıcıyı `$Secure` dosyasının (MFT girdisi 9) `$SDS` akışında bir kez saklar
ve ona bir **güvenlik kimliği** (security ID) verir. Bir dosyanın `$STANDARD_INFORMATION` özniteliği yalnızca bu güvenlik kimliğini kaydeder:
[`parse-ntfs mft`](mft.md) çıktısındaki `SecurityID` sütunu. `$SDS` bunu bir sahibe ve ACL'lere dönüştürür.

`$SDS`, her birinin ardından bir kopyası (aynası) gelen 256 KiB'lık bloklardan oluşur. Her girdi, NTFS'in
zaten var olan bir tanımlayıcıyı bulmak için kullandığı tanımlayıcı karmasını (`$SDH`) ve
güvenlik kimliğini (`$SII` bunu indeksler) içerir.

<span class="evidence-strength">En iyi sonuç:</span> `SecurityID` değeri bir tanımlayıcıyı adlandıran
dosyalar, `$MFT` kayıtları en son yazıldığında o sahibe ve o ACL'lere sahipti. Bir dosyanın
sahibi genellikle onu oluşturan hesaptır; ancak bir yönetici, bir yükleyici veya sahipliği alan bir program
bunu değiştirebilir.

## Konum ve toplama { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

Bunu ham erişimi destekleyen adli bir araçla toplayın; `:` bir dosya adında bulunamayacağı için toplamalar
genellikle onu `$Secure_$SDS` olarak adlandırır. Aynı birimin `$MFT` dosyasını da onunla birlikte toplayın.

## Analist için uyarılar { #analyst-cautions }

- Sahip bir SID'dir. Yalnızca bilinen SID'ler (`NT AUTHORITY\SYSTEM`, `BUILTIN\Administrators`,
  `NT SERVICE\TrustedInstaller`, …) ve bir makinenin veya etki alanının yerleşik hesapları (`-500`
  `Administrator`, …) adlandırılır. Diğerlerini ana bilgisayarın `SAM` kovanıyla
  ([SAM hesapları](sam.md)), profil listesiyle ([Sistem Bilgileri](system-info.md)) veya etki alanıyla çözümleyin.
- Bir tanımlayıcı, aynı sahip ve ACL'lere sahip tüm dosyalar tarafından paylaşılır ve bu dosyalar silindikten
  sonra da genellikle `$SDS` içinde kalır. `$MFT` içinde dosyası olmayan bir tanımlayıcı, tek başına
  silinmiş bir dosya anlamına gelmez.
- Bir ACE erişim geçmişi değildir. DACL bir dosyaya kimin erişebileceğini *söyler*; kimin eriştiğini olay günlükleri ve diğer
  artefaktlar söyler. SACL'nin denetim girdileri neyin günlüğe kaydedildiğini değil, yapılandırmayı açıklar.
- ACE'ler ACL içindeki sıralarına göre listelenir. Juntei etkin erişimi hesaplamaz: reddetme
  girdileri, devralma, grup üyeliği, ayrıcalıklar ve sahibin örtük hakları hep birlikte etkilidir.
- `DACL` ve `SACL` her ACE'yi `icacls` gibi yazar. `SDDL`, maskeleri onaltılık ve hesapları SID olarak içeren desteklenen izinleri temsil eder. İkili tanımlayıcının bayt bayt kopyası değildir; gösterilemeyen koşullu metin bunu boş bırakır ve bir tanılama iletisi üretilir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

Özet; tanımlayıcıları sayar ve sahiplerini her birinin kaç tanımlayıcıya sahip olduğuyla birlikte,
NULL DACL'ye sahip (hiçbir kısıtlama içermeyen) tanımlayıcıları ve varsa hasarları listeler.

## CSV çıktı alanları { #csv-output-fields }

| Alan | Yorumlama |
|---|---|
| `SourcePath` | Girdi dosyası. |
| `SecurityID` | Güvenlik kimliği: aynı birim için [`parse-ntfs mft`](mft.md) çıktısındaki `SecurityID` ile birleştirin. |
| `Hash` | Tanımlayıcıyla birlikte kaydedilen karma. |
| `EntryOffset` | Girdinin akış içindeki konumu. |
| `FromMirror` | Ayna bloğundan okunan bir kopya için `True`: girdisinden farklı olan (ondan sonra listelenen) bir kopya veya hasarlı bir girdinin kopyası. |
| `Size` | 20 baytlık başlığı dahil girdinin boyutu. |
| `Owner`, `OwnerName` | Sahibin SID'i ve biliniyorsa adı. |
| `Group`, `GroupName` | Birincil grubun SID'i ve biliniyorsa adı. Windows bunu erişim denetimleri için kullanmaz. |
| `Control` | Tanımlayıcının denetim bayrakları: `SE_DACL_PRESENT`, `SE_DACL_PROTECTED` (devralma engellenmiş), `SE_DACL_AUTO_INHERITED`, … `|` ile birleştirilmiş. |
| `DACL-ACE-Count`, `DACL` | `; ` ile ayrılmış DACL girdileri. DACL yoksa boştur; tanımlayıcı herkese her şeye izin veriyorsa `NULL DACL: no restriction`. |
| `SACL-ACE-Count`, `SACL` | SACL girdileri: denetim girdileri, zorunlu bütünlük etiketi, işlem güven etiketleri ve kaynak öznitelikleri. |
| `SDDL` | SDDL biçiminde tanımlayıcının tamamı. Yalnızca içindeki bir şeyin kodu çözülemediğinde boştur. |
| `Problems` | Tanımlayıcıya ait olmayan bir karma, kodu çözülemeyen bölümler, türü bilinmeyen ACE'ler. |

`DACL` ve `SACL` içindeki her girdi `icacls` gibi yazılır: hesap, izin verme girdisi değilse
türü, devralma bayrakları ve haklar.

| Bölüm | Anlamı |
|---|---|
| Hesap | Bilinen ad veya SID. |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | İzin verme girdisi olmadığında ACE türü. Zorunlu bir etiketin türü gösterilmez. |
| `(I)` | Üst dizinden devralınmış. |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | Dosyalar tarafından devralınır (object inherit), dizinler tarafından devralınır (container inherit), yalnızca bunlar tarafından devralınır ve bu nesneye uygulanmaz (inherit only) ve yalnızca bir alt düzeye devralınır (no propagate). |
| `(SA)`, `(FA)` | Başarılı veya başarısız erişim için bir denetim girdisi. |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | Tam denetim, değiştirme, okuma ve yürütme, okuma, yazma. |
| `(RD,WD,AD,…)` | Aksi hâlde belirli haklar: veri okuma, veri yazma, veri ekleme, genişletilmiş öznitelikleri okuma ve yazma (`REA`, `WEA`), yürütme (`X`), alt öğeyi silme (`DC`), öznitelikleri okuma ve yazma (`RA`, `WA`), silme (`DE`), denetim okuma (`RC`), DAC yazma (`WDAC`), sahip yazma (`WO`), eşitleme (`S`) ve genel haklar `GA`, `GR`, `GW`, `GE`. `N` erişim yok demektir. |
| `(NW)`, `(NR)`, `(NX)` | Zorunlu bir etiketin ilkesi: yukarı yazma yok, yukarı okuma yok, yukarı yürütme yok. |
| `if (…)` | `(WIN://SYSAPPID Contains "…")` gibi bir geri çağırma (callback) ACE'sinin koşulu: girdi yalnızca koşulun doğru olduğu işlemlere uygulanır. Windows bunları uygulama kapsayıcısı (app container) paketlerine erişim vermek için kullanır. |

İşlem güven etiketleri ve kaynak öznitelikleri maskelerini onaltılık olarak, bir kaynak
özniteliği ise talebini (`("IMAGELOAD",TU,0x0,1)`) gösterir.

JSON/JSONL aynı alanlara ve adlara sahiptir; sayılar sayı olarak, boş değerler `null` olarak ve
`Problems` bir liste olarak yer alır.

## İnceleme stratejisi { #investigation-strategy }

1. Aynı birimin `$MFT` dosyasını ayrıştırın ve `SecurityID` değerini bu çıktınınkiyle birleştirin: her dosya
   sahibini ve ACL'lerini alır.
2. Kullanıcı hesabı olan sahiplere (`S-1-5-21-…`) bakın: şüpheli bir hesabın hangi dosyalara sahip
   olduğuna ve sistem dizinlerindeki dosyalara hangi hesapların sahip olduğuna.
3. Programların çalıştığı veya yüklendiği dizinlerde aşırı izin veren ACL'leri arayın: `(F)`, `(M)` veya `(W)` ile
   `Everyone`, `Authenticated Users` veya `User` ve NULL DACL'ler.
4. Yalnızca tek bir dosyanın veya olağandışı bir dosya kümesinin kullandığı bir tanımlayıcı, normal kurulum
   dışında oluşturulmuş veya değiştirilmiş dosyaları gösterebilir.
5. Sahiplik ve izin değişikliklerini Security olay günlüğü girdileriyle (4670, SACL ile 4663) ve
   [USN Günlüğü](usn-journal.md)'nün `SECURITY_CHANGE` kayıtlarıyla doğrulayın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Güvenlik tanımlayıcıları](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (güvenlik tanımlayıcıları, ACE'ler, koşullu ifadeler)
- [Microsoft: Security Descriptor String Format (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: NTFS biçim belgeleri](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`secure` crate'i)

Yapılandırılmış çıktı stdout'a verildiğinde (`-o` olmadan `-t`), akış düzeyindeki kurtarma uyarıları stderr'e gider; böylece borular (pipe), hasarlı veya kurtarılmış bölgeleri gizlemeden okunabilir CSV/JSON/JSONL verisini korur.
