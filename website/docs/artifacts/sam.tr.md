# SAM Yerel Hesapları { #sam-local-accounts }

## SAM neleri içerir { #what-the-sam-contains }

Güvenlik Hesapları Yöneticisi (`SAM`) kovanı, yerel hesap kayıtlarını saklar. Juntei; hesap adlarının,
RID'lerin, profil/açıklama alanlarının, hesap denetim bayraklarının, son oturum açma/parola zaman
damgalarının, sayaçların, parola ipuçlarının, Microsoft hesabı bağlantısının, profil yollarının ve
Windows'un hesabı oturum açma arayüzünden gizleyip gizlemediğinin kodunu çözer.

Bu, yeni oluşturulmuş yerel yöneticileri, etkinleştirilmiş uykudaki hesapları, parola
değişikliklerini, gizli hesapları ve yanal hareket için hazırlanmış hesapları bulmak için yararlıdır.

<span class="evidence-strength">En iyi sonuç:</span> yerel SAM, hesabı kaydedilmiş durumla temsil
ediyordu. Bu, eksiksiz bir kimlik doğrulama geçmişi değildir ve etki alanı hesaplarını tanımlamaz.

## Konum ve toplama { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

Kovanı ve günlükleri yeterli ayrıcalıklarla/ham erişimle toplayın. Ana bilgisayar kimliği, bir
koleksiyondaki ilgili `SYSTEM` kovanıyla zenginleştirilebilir.

## Analist için uyarılar { #analyst-cautions }

- Bu komut Active Directory etki alanı kullanıcılarını değil, **yerel** hesapları ayrıştırır.
- Juntei, parola karmalarını kasıtlı olarak çıkarmaz.
- RID 500, yeniden adlandırılmış olsa bile yerleşik Administrator hesabıdır; ad tek başına yeterli
  değildir.
- `LastLogon` ve sayaçlar kayıt defteri durumudur; oturum başına eksiksiz bir denetim izi değildir.
- `HiddenFromLogonUI` bir hesabı oturum açma ekranından gizler; hesabı devre dışı bırakmaz.
- Yerleşik `DefaultAccount` ve `WDAGUtilityAccount` meşru olarak gizli olabilir.
- Hasarlı/eksik bir `F` değerinden kaynaklanan boş hesap bayrakları, etkin olarak yorumlanmamalıdır.
- `NamesKeyLastWrittenUTC`, hesap adının kaydedilme bağlamına yaklaşık bir değer verir; ancak garanti
  edilmiş bir oluşturma zamanı değildir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath` | Ana bilgisayar bağlamı ve tam SAM kovanı. |
| `RID` | Göreli Tanımlayıcı (Relative Identifier). Güçlü bir hesap kimliği pivotu; 500 yerleşik Administrator hesabıdır. |
| `User` | Güncel yerel hesap adı. Yeniden adlandırılabilir; RID ile ilişkilendirin. |
| `FullName`, `Comment`, `UserComment` | Açıklayıcı metin; saldırgan tarafından kontrol edilebilir ve kılık değiştirme (masquerading) incelemesi için yararlıdır. |
| `AccountFlags` | Devre dışı, kilitli, parola gerekmez, normal hesap, parola süresi dolmaz gibi kodu çözülmüş durumlar. |
| `AccountFlagsValue` | Doğrulama ve desteklenmeyen bitler için ham sayısal bayraklar. |
| `CountryCode`, `CodePage` | Hesabın yerel ayar/kod sayfası meta verileri; genellikle bağlamdır, atıf değildir. |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | Tam hassasiyette ve tam saniye uyumluluğunda aynı son oturum açma durumu. Eksiksiz bir geçmiş değildir. |
| `PasswordLastSetUTC` | Kaydedilmiş parola değiştirme zamanı. Beklenmedik derecede yeni/eski değerleri inceleyin. |
| `AccountExpirationUTC` | Sonlu olduğunda yapılandırılmış sona erme tarihi. Null, hiçbir zaman/ayarlanmamış anlamına gelebilir. |
| `LastFailedLogonUTC` | SAM durumunda kaydedilmiş son başarısız yerel oturum açma; Security olaylarıyla doğrulayın. |
| `FailedLogonCount`, `LogonCount` | Windows'a özgü güncelleme davranışına sahip kümülatif/durum sayaçları; kesin olay sayısı olarak değil, bağlam olarak kullanın. |
| `PasswordHint` | Kullanıcının sağladığı hassas ipucu; parola kalıplarını açığa çıkarabilir. Uygun şekilde işleyin. |
| `InternetUserName` | Varsa bağlı Microsoft hesabı kimliği. |
| `HiddenFromLogonUI` | Arayüzde gizleme durumu. Yüksek RID'leri/beklenmedik özel hesapları inceleyin; devre dışı anlamına gelmez. |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | Yerel hesap ortamı/oturum açma yapılandırması; olağandışı betikler/UNC yolları kalıcılığa işaret edebilir. |
| `KeyLastWrittenUTC` | RID hesap anahtarının değişme zamanı. Alana özgü değil, anahtar düzeyindedir. |
| `NamesKeyLastWrittenUTC` | Kullanıcı adı eşleme anahtarının değişme zamanı; hesap yaşam döngüsü için yararlı bir yaklaşık değer. |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | Doğrulama ve özel gözcü (sentinel) değerlerin işlenmesi için korunan ham 64 bit FILETIME değerleri. |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri yeniden oynatma bağlamı. |
| `Problems` | F/V/Names arasındaki uyuşmazlık, eksik girdiler, bozuk ofsetler veya kod çözme hasarı. |

## İnceleme stratejisi { #investigation-strategy }

1. RID, ad, bayraklar, profil yolu ve `NamesKeyLastWrittenUTC` değerlerini ana bilgisayarın temel
   durumuyla karşılaştırın.
2. Etkin RID 1000+ hesapları, parola gerektirmeyen, süresi dolmayan, gizli veya yakın zamanda
   değiştirilmiş hesapları inceleyin.
3. Security 4720/4722/4724/4732 olayları, oturum açma olayları, profil oluşturma ve uzak etkinlikle
   ilişkilendirin.
4. Bir hesap toplamadan önce kaldırıldıysa kayıt defteri kurtarma geçmişinde arayın.
5. Meşru hazırlama/varsayılan hesapları, saldırgan tarafından etkileşimli olarak oluşturulmuş
   hesaplardan ayırt edin.

## Daha fazla ayrıntı { #further-details }

- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [dissect.target SAM eklentisi](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
- [Eric Zimmerman'ın Registry Explorer aracı](https://ericzimmerman.github.io/#!index.md)
