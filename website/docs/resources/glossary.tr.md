# DFIR Sözlüğü { #dfir-glossary }

| Terim | Sade dilde anlamı |
|---|---|
| Ayrılmış (Allocated) | Kayıt/nesne, artefakta göre şu anda kullanımdadır. |
| Alternatif Veri Akışı (Alternate Data Stream, ADS) | Bir dosyaya, normal adsız içeriğinin yanı sıra eklenmiş adlandırılmış NTFS verisi; örneğin `Zone.Identifier`. |
| Artefakt | Bir işletim sisteminin/uygulamanın ürettiği ve adli bilişim sorularını yanıtlayabilen kalıcı veri. |
| Oyma (Carving) | Kayıtları, normal kapsayıcılarına/indekslerine dayanmadan yapılarına/imzalarına göre bulma. |
| Denetim kümesi (Control set) | SYSTEM yapılandırmasının bir sürümü (`ControlSet001` vb.); `Select\Current` etkin olanı belirtir. |
| Doğrulama (Corroboration) | Aynı sonucu destekleyen bağımsız kanıt. |
| Kirli veritabanı/kovan | Birincil dosya toplandığında tam olarak eşitlenmemişti; gerekli durum işlem günlüklerinde bulunabilir. |
| Droid | Dosyaları/birimleri taşımalar boyunca izlemek için kullanılan, GUID benzeri dağıtılmış bağlantı izleme tanımlayıcısı. |
| ESE / JET Blue | Microsoft'un SRUM ve diğer Windows verileri tarafından kullanılan Extensible Storage Engine veritabanı biçimi. |
| FILETIME | 1601-01-01 UTC'den bu yana geçen 100 nanosaniyelik aralıkların sayısı; bir sayı yalnızca şeması öyle söylediğinde zaman olarak anlamlıdır. |
| Dosya başvurusu (File reference) | Bir MFT kaydının bir neslini tanımlayan NTFS girdi numarası artı sıra numarası. |
| Kovan (Hive) | Bir Windows Kayıt Defteri ağacı içeren ikili dosya. |
| INDX kaydı | Bir NTFS dizin indeksinin (`$I30`) dizindeki dosyaları listeleyen bloğu (genellikle 4 KiB); slack alanı, silinmiş veya yeniden adlandırılmış dosya adlarına ilişkin olası kanıtları tutabilir. |
| LNK | Bir hedef ve kapsamlı meta veriler içeren Windows Shell Link/kısayol dosyası. |
| MRU | En Son Kullanılan (Most Recently Used) sıralaması; genellikle her öğe için bir zaman değil, göreli bir sıradır. |
| MFT | Dosya/dizin kayıtlarını ve özniteliklerini içeren NTFS Ana Dosya Tablosu (Master File Table). |
| Prefetch karması | `.pf` adlandırmasında kullanılan Windows tanımlayıcısı; kriptografik bir içerik karması değildir. |
| Kayıt defteri anahtarının son yazılma zamanı | Anahtarın değiştiği zaman; Windows genellikle hangi değerin değiştiğini söylemez. |
| Yerleşik veri (Resident data) | Harici veri dizileri (data run) yerine doğrudan bir MFT özniteliğinde saklanan dosya/akış baytları. |
| RID | Göreli Tanımlayıcı (Relative Identifier); bir Windows SID yetkilisi/etki alanı altındaki son hesap/grup bileşeni. |
| SDDL | Güvenlik Tanımlayıcısı Tanım Dili (Security Descriptor Definition Language): metin olarak yazılmış bir güvenlik tanımlayıcısı; örneğin `O:S-1-5-18D:(A;;0x001f01ff;;;S-1-5-32-544)`. |
| Güvenlik tanımlayıcısı (Security descriptor) | Bir dosyanın sahibi, birincil grubu, DACL'si (kimin ne yapabileceği) ve SACL'si (denetim ve bütünlük etiketi). NTFS her farklı tanımlayıcıyı bir güvenlik kimliği altında `$Secure:$SDS` içinde bir kez tutar. |
| SID | Bir hesap, hizmet, grup veya güvenlik sorumlusu için Windows Güvenlik Tanımlayıcısı (Security Identifier). |
| Slack | Ayrılmış bir yapının içinde/sonrasında, eski verileri tutabilen kullanılmayan baytlar. |
| SRUM | Uygulama/kullanıcı kaynak kullanımına ilişkin System Resource Usage Monitor veritabanı. |
| Zaman damgası manipülasyonu (Timestomping) | İncelemeyi yanıltmak için zaman damgalarının kasıtlı olarak değiştirilmesi; buluşsal yöntemler kanıt değildir. |
| İşlem günlüğü (Transaction log) | Bir kayıt defterini veya veritabanını tutarlı bir duruma kurtarmak için kullanılan bekleyen/kaydedilmiş değişiklikler. |
| USN | NTFS değişiklik günlüğü kayıtları ve ilgili meta veriler tarafından kullanılan Güncelleme Sıra Numarası (Update Sequence Number). |
| UTC | Eşgüdümlü Evrensel Zaman; sistemler arası zaman çizelgeleri için tercih edilen ortak zaman temeli. |
| Birim seri numarası (Volume serial number) | NTFS'nin bir birimi biçimlendirirken ona verdiği numara (`$Boot`); düşük 32 biti, LNK, Jump List ve Prefetch dosyalarının bir dosyanın birimine ilişkin kaydettiği değerdir. |
