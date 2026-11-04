# Görev Zamanlayıcı Önbelleği { #task-scheduler-cache }

## TaskCache nedir { #what-taskcache-is }

Windows Görev Zamanlayıcı (Task Scheduler), `SOFTWARE` altında görev yollarını GUID'lere, tetikleyicilere/listelere,
sorumlulara (principal), eylemlere, çalışma zamanı meta verilerine ve güvenlik tanımlayıcılarına bağlayan bir kayıt defteri önbelleği tutar. Juntei;
yürütme ve COM işleyici eylemlerinin yanı sıra eski e-posta/ileti eylemlerinin, görev zaman damgalarının ve normal
Görev Zamanlayıcı arayüzünden gizlenmiş görev belirtilerinin kodunu çözer.

Görevler yaygın bir kalıcılık ve uzaktan yürütme mekanizmasıdır. Kayıt defteri önbelleği, bir saldırgan görevin
XML dosyasını sildiğinde veya kurcaladığında da değerlidir.

<span class="evidence-strength">En iyi sonuç:</span> TaskCache görevi/eylemi yapılandırmış veya
korumuştur. Belirli bir yürütmeyi kanıtlamak için çalışma zamanı alanları ve bağımsız günlükler gerekir.

## Konumlar { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

`SOFTWARE` kovanını işlem günlükleriyle birlikte ve `System32\Tasks` ağacının tamamını toplayın. Task Scheduler
Operational ve Security olay günlükleri oluşturma/güncelleme/çalışma zamanı kanıtları ekler.

## Analist için uyarılar { #analyst-cautions }

- Kayıt defteri önbelleği ve görev XML'i; kurcalama, silme, eksik kayıt veya hasar sonrasında birbiriyle çelişebilir.
- Juntei'nin `Hidden` değeri, yalnızca normal bir “Gizli” kullanıcı arayüzü bayrağıyla yapılandırılmış bir görevi değil,
  yapısal gizlemeyi (ilgili koşullar altında eksik Tree bağlantısı/güvenlik tanımlayıcısı) belirtir.
- Windows 7 görev eylemlerini önbelleğin `Actions` değerinde değil, XML'de saklar; orada boş bir eylem
  otomatik olarak şüpheli değildir.
- `LastRun` ve `LastResult` faydalıdır, ancak eksiksiz ve kurcalamaya karşı korumalı bir yürütme geçmişi değildir.
- `Context`, görevi kaydeden kullanıcıyı değil, yapılandırılmış sorumluyu/bağlamı adlandırır.
- COM işleyici CLSID'leri ayrıca kayıt defteri/dosya çözümlemesi gerektirir.

## Juntei ile ayrıştırma { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## Çıktı alanları { #output-fields }

| Alan | Yorumlama |
|---|---|
| `Host`, `SourcePath` | Ana bilgisayar bağlamı ve kaynak `SOFTWARE` kovanı. |
| `TaskName` | Tam zamanlayıcı yolu. XML ve olay günlüklerine birincil pivot noktası. |
| `TaskGUID` | Tree ve Tasks önbellek girdilerini birleştiren GUID. Güçlü bir ilişkilendirme anahtarı. |
| `TaskIndex` | Zamanlayıcının kullandığı Tree/indeks meta verisi; bağlamsaldır, bir yürütme sayısı değildir. |
| `TaskLists` | Tetiklemeyi/işlemeyi sınıflandıran önbellek listeleri (`Boot`, `Logon`, `Plain`, `Maintenance`). |
| `Hidden` | Yapısal gizli görev göstergesi. Kurcalama sonucuna varmadan önce `InTree`, `InTasks` ve tanımlayıcı durumunu inceleyin. |
| `InTree`, `InTasks` | Önbelleğin her iki tarafının da mevcut olup olmadığı. Eksik bağlantı silme/kurcalama/eksik kaydı gösterebilir. |
| `HasSecurityDescriptor` | Tree girdisinin `SD` değerine sahip olup olmadığı; bulunmaması ilgili sürümlerde gizli görev davranışına katkıda bulunabilir. |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | Önbelleğin her iki tarafı için anahtar düzeyindeki değişiklikler; kesin oluşturma/çalıştırma zamanları değildir. |
| `ActionIndex` | Görev içindeki sıfır tabanlı eylem. Bir görev birden fazla satır üretebilir. |
| `ActionType` | `exec`, `com_handler`, `send_email` veya `show_message`. |
| `Executable`, `Args`, `WorkingDirectory` | Etkin yürütme eylemi. Tırnaklamayı, değişkenleri, göreli yolları ve LOLBin'leri yorumlayın. |
| `COM-ClassID`, `ActionData` | COM işleyici CLSID'i ve verisi ya da eski e-posta/ileti yükü. CLSID'i ayrıca çözümleyin. |
| `Context` | Eylemlerle birlikte kodlanan sorumlu bağlamı (`LocalSystem`, `Author` vb.); kaydeden değil, yapılandırılmış kimliktir. |
| `Author`, `URI`, `Description` | Görev meta verileri. Saldırgan tarafından kolayca denetlenebilir ve güven kanıtı değildir. |
| `LastRegisteredUTC` | Mevcut olduğunda DynamicInfo kayıt/güncelleme zamanı. |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | Önbelleğe alınmış çalışma zamanı/sonuç meta verileri. Operasyonel günlükler ve işlem kanıtlarıyla ilişkilendirin. |
| `ActionsRaw` | Kod çözme eksikse veya doğrulama gerekiyorsa özgün eylem baytları. |
| `TransactionLogStatus`, `LogsApplied` | Kayıt defteri kurtarma bağlamı. |
| `Problems` | Yol/GUID uyuşmazlığı, yanlış değer türü, kesilmiş eylem yapısı veya eksik parçalar. |

## İnceleme stratejisi { #investigation-strategy }

1. Önce gizli/tutarsız görevleri ve Microsoft dışı yolları inceleyin.
2. Bağımsız değişkenler, çalışma dizini, ortam değişkeni genişletmesi ve COM çözümlemesi dahil olmak üzere etkin komutları yeniden oluşturun.
3. Kayıt defteri kaydını görev XML'i ile karşılaştırın ve başvurulan yürütülebilir dosyanın karmasını/zaman damgalarını alın.
4. `TaskName`/GUID/zaman bilgisini Task Scheduler olayları, Security 4698/4702, işlem günlükleri, Prefetch ve MFT/USN ile ilişkilendirin.
5. Bir saldırgan görevi kaldırdığında silinmiş/kurtarılmış kayıt defteri verilerinde arama yapın.

## Daha fazla ayrıntı { #further-details }

- [Microsoft: Task Scheduler şeması](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: Task Scheduler olay günlüğü](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
