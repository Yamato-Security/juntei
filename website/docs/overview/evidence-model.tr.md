# Kanıtlar ve Yorumlama { #evidence-and-interpretation }

Dijital adli bilişim yalnızca alanları okumaktan ibaret değildir. Alanın hangi gözlemi desteklediğine,
bu desteğin ne kadar güçlü olduğuna ve aynı gözlemi masum bir şekilde neyin üretebileceğine karar vermektir.

## Dört faydalı kanıt kategorisi { #four-useful-evidence-categories }

| Kategori | Anlamı | Örnekler |
|---|---|---|
| Varlık | Bir yol veya dosya Windows tarafından biliniyordu | Amcache dosya kaydı, ShimCache girdisi, `$MFT` kaydı |
| Yürütme | Windows bir program başlatmasını veya yürütme durumunu kaydetti | Prefetch çalışma zamanı; desteklenen ShimCache yürütme bayrağı |
| Kullanıcı etkileşimi | Bir kullanıcı veya uygulama bir öğeye başvurdu/öğeyi açtı | RecentDocs, Jump List, Recent klasöründeki LNK |
| Sistem etkinliği | Windows kaynak kullanımını veya yapılandırmayı kaydetti | SRUM sayaçları, hizmet yapılandırması, TaskCache eylemi |

Bir artefakt birden fazla kategoriyi destekleyebilir, ancak onu asla otomatik olarak üst kategoriye taşımayın. Örneğin,
şüpheli bir yola sahip bir Amcache girdisi **varlığı** destekler. Eşleşen bir Prefetch çalışma zamanı daha
güçlü bir **yürütme** kanıtı ekler. Bir USN oluşturma kaydı ve `$MFT` yolu dosyayı diske yerleştirebilir ve
Defender karantinası dosyanın neden ortadan kaybolduğunu açıklayabilir.

## Zaman damgaları evrensel gerçek değil, beyanlardır { #timestamps-are-statements-not-universal-truth }

Her zaman damgasına neyin neden olduğunu sorun:

- `$STANDARD_INFORMATION` ve `$FILE_NAME` zamanları NTFS meta verilerini açıklar ve birbirinden ayrışabilir.
- Bir Prefetch çalışma zamanı kaydedilmiş bir başlatmayı açıklar, ancak yalnızca son çalışma zamanları kalır.
- Bir ShimCache zamanı genellikle önbelleğe alınmış dosyanın son değiştirilme zamanıdır; yürütme zamanı değildir.
- Bir kayıt defteri anahtarının son yazma zamanı tek bir değere değil, anahtarın bütününe uygulanır.
- RecentDocs, anahtarın zamanını en son öğeye verir; daha eski öğelerin sırası vardır ancak genellikle açılma zamanı yoktur.
- SRUM zaman damgaları çoğunlukla tam ağ işleminden ziyade bir toplama aralığını belirtir.

Analiz sırasında UTC kullanın, mümkün olduğunda özgün değeri saklayın ve yapılan her dönüştürmeyi belgeleyin.

## Olumlu ve olumsuz kanıtlar { #positive-and-negative-evidence }

İyi anlaşılmış bir kaydın varlığı güçlü bir olumlu kanıt olabilir. Yokluk ise genellikle daha
zayıftır; çünkü artefaktların üzerine yazılır, önbelleklerin üst sınırı vardır, toplama eksik olabilir, ayarlar değişkenlik gösterir
ve temizleme araçları mevcuttur. “Gerçekleşmedi” yerine “toplanan artefaktta gözlemlenmedi” deyin.

## Doğrulama merdiveni { #corroboration-ladder }

1. Kaynak dosyayı ve toplama yolunu doğrulayın.
2. Ayrıştırıcının ilgili bir hasar veya uygulanmamış işlem günlüğü bildirmediğini teyit edin.
3. Alanı artefakt sayfasına göre yorumlayın.
4. Aynı eylemi destekleyen bağımsız bir artefakt bulun.
5. Mümkün olduğunda olay günlüklerini, ağ telemetrisini, belleği veya dosya içeriğini kontrol edin.
6. Tam Juntei sürümünü, komutu, filtreleri ve kaynak karmasını kaydedin.

!!! tip "Sonuçları kanıtın gücüne uygun yazın"
    “Saldırgan kötü amaçlı yazılımı tam olarak 10:02:11Z'de yürüttü” yerine “Prefetch, `tool.exe` dosyasının
    10:02:11Z'de çalıştığını kaydediyor” ifadesini tercih edin. İkinci ifade, Prefetch'in
    tek başına ortaya koyamayacağı kimlik ve niyet bilgisini ekler.
