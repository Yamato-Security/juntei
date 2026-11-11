# Kayıt Defteri Komutları { #registry-commands }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry <COMMAND> [OPTIONS]

Commands:
  amcache             Parse Amcache inventories
  app-compat          Parse application compatibility data
  app-history         Parse third-party tools' traces
  auditpol            Parse the audit policy
  bam                 Parse BAM and DAM last runs
  bcd                 Parse boot configuration
  feature-usage       Parse Explorer use counts and pins
  help                Print this message or the help of the given subcommand(s)
  installed-programs  Parse installed programs
  lsa-secrets         Parse LSA secrets' names and times
  mru                 Parse most-recently-used lists
  muicache            Parse MuiCache program names
  networks            Parse network profiles, interfaces, shares, port proxies and routes
  office              Parse Office recent files, trusted documents and macro settings
  open-save-mru       Parse Open and Save As dialog history
  persistence         Parse autostart locations
  read                Read keys and values
  recent-docs         Parse recent documents
  remote-access       Parse outbound RDP, PuTTY, WinSCP and mapped drive connections
  sam                 Parse local accounts
  security-settings   Parse security settings: Defender, UAC, credentials, RDP, firewall, logging
  services            Parse services
  shell-bags          Parse ShellBags (folders opened in Explorer)
  shimcache           Parse ShimCache entries
  system-info         Parse system information
  taskcache           Parse scheduled tasks
  usb                 Parse USB devices and volumes
  user-assist         Parse UserAssist entries

Options:
  -h, --help  Print help
```

Standart komut adları `app-compat`, `feature-usage`, `open-save-mru`, `recent-docs`,
`user-assist` ve `shell-bags` şeklindedir. Mevcut betikler için bunların eski yazımları (`appcompat`, `featureusage`,
`opensavemru`, `recentdocs`, `userassist` ve `shellbags`) gizli takma adlar olarak kabul edilmeye
devam eder. Otomatik ayrıştırma standart adları kullanır; artefakt tanımlayıcıları,
sonuç şemaları ve çıktı dosya adları değişmemiştir.

Kayıt defteri komutları `-f <HIVE>` veya `-d <COLLECTION>` ile yapılandırılmış çıktı seçeneklerini kabul eder.
`amcache`, kovan dirty durumundaysa yanındaki Amcache işlem günlüklerini otomatik olarak yeniden oynatır; ancak günlükleri belirtme veya yeniden oynatmayı atlama seçeneği yoktur. `shimcache`
varsayılan olarak yanındaki SYSTEM günlüklerini yeniden oynatır ve dışa aktarılmış ham bir AppCompatCache değerini de kabul eder.
Diğer kayıt defteri komutları yanlarındaki günlükleri otomatik olarak yeniden oynatır; günlükleri açıkça belirtmek için
`--log1` / `--log2`, birincil kovanı yeniden oynatma yapmadan incelemek için `--no-logs` kullanın.

## `parse-registry amcache` { #parse-registry-amcache }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Amcache hives (Amcache.hve)
  -f, --file <FILE>      File path to one Amcache.hve

Records:
      --files    Include every inventoried executable (InventoryApplicationFile)
      --devices  Include every inventoried PnP device (InventoryDevicePnp)
  -a, --all      Include executables and devices (default for -o and for json, jsonl, csv)

Output:
  -o, --output <PATH>       Save the results to a file (text, json, jsonl) or a directory (csv)
  -t, --output-type <TYPE>  Output type: csv (default with -o), json, jsonl, text (default on the terminal)
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-registry amcache -f Amcache.hve --files
juntei parse-registry amcache -d evidence/ -o amcache-csv
juntei parse-registry amcache -f Amcache.hve -t jsonl | jq 'select(.RecordType == "file")'
```

Bkz. [Amcache](../artifacts/amcache.md).

## `parse-registry bam` { #parse-registry-bam }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives
  -f, --file <FILE>      File path to one SYSTEM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Kullanıcılar, ana bilgisayarla aynı şekilde bulunan sistemin `SAM` ve `SOFTWARE` (`ProfileList`) kovanlarından adlandırılır.
Bkz. [BAM ve DAM](../artifacts/bam.md).

## `parse-registry installed-programs` { #parse-registry-installed-programs }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      File path to one SOFTWARE or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

`Uninstall` girdileri, Windows Installer ürünleri, App Paths, WSL dağıtımları ve paketlenmiş uygulamalar.
Bir kullanıcı kovanının kullanıcısı, profil klasöründen ve sistemin `ProfileList` verisinden gelir. Bkz.
[Yüklü Programlar](../artifacts/installed-programs.md).

## `parse-registry muicache` { #parse-registry-muicache }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      File path to one UsrClass.dat (Windows Vista and later) or NTUSER.DAT (Windows XP) hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Kullanıcılar kovanın koleksiyondaki konumundan, SID'leri ise bir `UsrClass.dat` kovanının
kök anahtarından veya sistemin `SOFTWARE` (`ProfileList`) kovanından adlandırılır. Bkz. [MuiCache](../artifacts/muicache.md).

## `parse-registry open-save-mru` { #parse-registry-open-save-mru }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (8.3 names, network shares): the system's ANSI code page (932 for Japanese) [default: 1252]

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Her kabuk öğesi listesinin kodu ys-lnk-parser ile çözülür ve bir yol olarak birleştirilir. Bkz.
[Aç ve Farklı Kaydet İletişim Kutuları](../artifacts/opensavemru.md).

## `parse-registry remote-access` { #parse-registry-remote-access }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Kullanıcıların SID'leri, ana bilgisayarla aynı şekilde bulunan sistemin `SOFTWARE` (`ProfileList`) kovanından gelir. Saklanan
parolalar mevcut olarak raporlanır, kodları asla çözülmez. Bkz.
[Uzaktan Erişim (Kullanıcı Tarafı)](../artifacts/remote-access.md).


## `parse-registry system-info` { #parse-registry-system-info }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM, SOFTWARE, SAM and SECURITY hives (those in one directory are one system's)
  -f, --file <FILE>      File path to a SYSTEM, SOFTWARE, SAM or SECURITY hive of one system; repeat for each (the others are looked for next to them)
      --no-logs          Read dirty hives as they are, without their transaction logs (default: <hive>.LOG, .LOG1 and .LOG2 next to each are replayed)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Sistem başına bir kayıt: adları, etki alanı veya çalışma grubu, Windows sürümü ve yükleme tarihi, saat
dilimi, son kapatma, son kullanıcı, profiller ve SID'ler. Bir dizindeki kovanlar tek bir sisteme aittir;
`-f` ile verilen kovanlar ve yanlarında bulunan dört kovandan diğerleri kullanılır.
Bkz. [Sistem Bilgileri](../artifacts/system-info.md).


## `parse-registry mru` { #parse-registry-mru }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --list <LISTS>  Only these lists (comma separated): runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Bkz. [En Son Kullanılanlar (MRU) Listeleri](../artifacts/mru.md).


## `parse-registry app-history` { #parse-registry-app-history }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Sysinternals araçları, WinRAR, 7-Zip ve WinZip geçmişi, OneDrive hesapları, Adobe son dosyaları,
TeamViewer ve CCleaner. Bkz. [Uygulama Geçmişi](../artifacts/app-history.md).


## `parse-registry feature-usage` { #parse-registry-feature-usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry feature-usage <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

`FeatureUsage`, `RecentApps`, `Taskband` ve `ProgramsCache` değerlerini okur. Bkz.
[FeatureUsage, RecentApps ve Görev Çubuğuna Sabitlenmiş Öğeler](../artifacts/featureusage.md).


## `parse-registry security-settings` { #parse-registry-security-settings }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      File path to one SOFTWARE, SYSTEM or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive (ControlSet001, ControlSet002, ...), not only the one Windows used last
      --weakened-only     Only settings whose value weakens what Windows does by default

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Bulunan her ayar; değeri, anlamı ve Windows'un varsayılan olarak yaptığını zayıflatıp zayıflatmadığı
bilgisiyle bir kayıttır; `--weakened-only` yalnızca zayıflatanları tutar. Bkz.
[Güvenlik Ayarları](../artifacts/security-settings.md).


## `parse-registry office` { #parse-registry-office }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Bkz. [Microsoft Office](../artifacts/office.md).

### Örnekler { #examples }

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --service ExampleSvc -t jsonl
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry office -d collection/ -o office.csv
juntei parse-registry taskcache -f SOFTWARE --task ScheduledDefrag -t jsonl
juntei parse-registry sam -f SAM --rid 500 -t jsonl
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry user-assist -d collection/ -o userassist.csv
```

Seçeneklerin tamamı ve alan anlamları şu sayfalardadır: [Kayıt Defteri Kovanları](../artifacts/registry.md),
[Windows Hizmetleri](../artifacts/services.md), [RecentDocs](../artifacts/recentdocs.md),
[Microsoft Office](../artifacts/office.md),
[Görev Zamanlayıcı Önbelleği](../artifacts/taskcache.md), [SAM Yerel Hesapları](../artifacts/sam.md),
[Kayıt Defteri Otomatik Başlatmaları](../artifacts/persistence.md) ve [UserAssist](../artifacts/userassist.md).


## `parse-registry auditpol` { #parse-registry-auditpol }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SECURITY hives
  -f, --file <FILE>      File path to one SECURITY hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

`Policy\PolAdtEv` içindeki denetim ilkesinin alt kategorisi (Windows Vista ve sonrası) veya kategorisi (XP ve 2003)
başına bir kayıt: neyi günlüğe kaydettiği, GUID'i ve ileti kimliği ve başlıca olayları. Bkz.
[Denetim İlkesi](../artifacts/auditpol.md).


## `parse-registry app-compat` { #parse-registry-app-compat }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      File path to one SOFTWARE or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Uyumluluk katmanları, Program Uyumluluk Yardımcısı'nın `Store` ve `Persisted` verileri, CIT ve özel
shim veritabanları. ShimCache, `parse-registry shimcache` komutunun kapsamındadır. Bkz.
[Uygulama Uyumluluğu](../artifacts/appcompat.md).


## `parse-registry bcd` { #parse-registry-bcd }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for BCD hives (and BCD-Template)
  -f, --file <FILE>      File path to one BCD hive (\Boot\BCD, \EFI\Microsoft\Boot\BCD)
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Deponun nesnesi başına bir kayıt; test imzalama, bütünlük denetimleri, kurtarma, önyükleme durumu
ilkesi ve güvenli mod devralma yoluyla çözümlenmiş olarak. Bkz.
[Önyükleme Yapılandırma Verileri (BCD)](../artifacts/bcd.md).


## `parse-registry lsa-secrets` { #parse-registry-lsa-secrets }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SECURITY hives
  -f, --file <FILE>      File path to one SECURITY hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

LSA sırrı (`Policy\Secrets\<name>`) başına bir kayıt; türü, adlandırdığı hizmet veya kullanıcı,
değerlerinin ne zaman ayarlandığı ve veri içerip içermedikleriyle birlikte; ayrıca önbelleğe alınmış oturum açma bilgileri
(`Cache`) için bir kayıt. Yalnızca adlar ve zamanlar: hiçbir şeyin şifresi çözülmez. Bkz. [LSA Sırları](../artifacts/lsa-secrets.md).


## `parse-registry networks` { #parse-registry-networks }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE and SYSTEM hives
  -f, --file <FILE>      File path to one SOFTWARE or SYSTEM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Profil zamanları, ana bilgisayarla aynı şekilde bulunan sistemin `SYSTEM` kovanındaki saat dilimiyle UTC'ye
dönüştürülür ve bulunduğu hâliyle de saklanır. Bkz. [Ağlar](../artifacts/networks.md).


## `parse-registry persistence` { #parse-registry-persistence }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      File path to one SOFTWARE, SYSTEM, NTUSER.DAT or UsrClass.dat hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry read` { #parse-registry-read }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for registry hives (every file that is one, logs aside)
  -f, --file <FILE>      File path to one registry hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
  -k, --key <PATH>             Only the key at this path (in any case; ex: Microsoft\Windows\CurrentVersion\Run). It may start with HKLM\ and the hive's name, and CurrentControlSet is the control set Windows used last
      --key-regex <REGEX>      Only keys whose path matches REGEX (ignoring case)
      --value <NAME>           Only the value named NAME (in any case; "" is the key's default value). With --key or --key-regex, a key without it gives an absent_value record
      --value-regex <REGEX>    Only values whose name matches REGEX (ignoring case; the default value's name is empty)
      --data-regex <REGEX>     Only values whose data, as written, matches REGEX (ignoring case, so a GUID matches in any case; binary data in hex)
      --timeline-start <DATE>  Only keys last written on or after DATE, in UTC unless an offset is given (ex: 2025-10-19, 2025-10-19T13:45:30Z)
      --timeline-end <DATE>    Only keys last written on or before DATE
      --recover-deleted        Also deleted keys and values: in free cells, the slack of cells in use and the data past the hive bins, and what only the transaction logs kept (allocation_status says how complete; recovered_from where found)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
      --raw-data            Write every value's bytes in hex (value_data_raw), not only those that don't decode as their type
      --qword-as-filetime   Also write REG_QWORD values as FILETIMEs (value_filetime_utc), keeping the number (value_data, value_u64)
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-registry read -f SYSTEM --key 'CurrentControlSet\Services' -o services-raw.csv
juntei parse-registry read -f NTUSER.DAT --recover-deleted -t jsonl | jq .
```

## `parse-registry recent-docs` { #parse-registry-recent-docs }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry sam` { #parse-registry-sam }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SAM hives
  -f, --file <FILE>      File path to one SAM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --rid <RID>  Only the account with this RID (ex: 500, the built-in Administrator)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry services` { #parse-registry-services }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives
  -f, --file <FILE>      File path to one SYSTEM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --service <NAME>    Only the service of this name (its key's name, in any case)
      --all-control-sets  Every control set (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry shell-bags` { #parse-registry-shell-bags }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      File path to one UsrClass.dat or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Parsing:
      --code-page <NUMBER>  Windows code page for the ANSI names of shell items (8.3 names, network shares) [default: 1252]

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry shimcache` { #parse-registry-shimcache }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives
  -f, --file <FILE>      File path to one SYSTEM hive, or an AppCompatCache value saved to a file
      --no-logs          Read a dirty hive as it is, without its transaction logs (SYSTEM.LOG1, SYSTEM.LOG2)

Output:
  -o, --output <FILE>       Save every entry to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, the entries are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-registry shimcache -f SYSTEM
juntei parse-registry shimcache -d evidence/ -o shimcache.csv
```

Bkz. [ShimCache](../artifacts/shimcache.md).

## `parse-registry taskcache` { #parse-registry-taskcache }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE hives
  -f, --file <FILE>      File path to one SOFTWARE hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --task <NAME>  Only the task of this name, in any case: its whole name (\Microsoft\Windows\Defrag\ScheduledDefrag) or its last part (ScheduledDefrag)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry usb` { #parse-registry-usb }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives (each system's SOFTWARE and users' NTUSER.DAT hives are read with it)
  -f, --file <FILE>      File path to one SYSTEM hive (its system's SOFTWARE and users' NTUSER.DAT hives are read with it)
      --log1 <FILE>      The SYSTEM hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The SYSTEM hive's second transaction log (.LOG2), for -f
      --no-logs          Read dirty hives as they are, without their transaction logs

Filtering:
      --all-control-sets  Every control set (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry user-assist` { #parse-registry-user-assist }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry user-assist <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### Örnekler { #examples_1 }

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --service ExampleSvc -t jsonl
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry taskcache -f SOFTWARE --task ScheduledDefrag -t jsonl
juntei parse-registry sam -f SAM --rid 500 -t jsonl
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry usb -d collection/ -o usb.csv
juntei parse-registry user-assist -d collection/ -o userassist.csv
juntei parse-registry open-save-mru -d collection/ -o opensavemru.csv
juntei parse-registry bam -d collection/ --all-control-sets -o bam.csv
juntei parse-registry installed-programs -d collection/ -o installed.csv
juntei parse-registry muicache -d collection/ -o muicache.csv
juntei parse-registry shell-bags -d collection/ -o shellbags.csv
```

Seçeneklerin tamamı ve alan anlamları şu sayfalardadır: [Kayıt Defteri Kovanları](../artifacts/registry.md),
[Windows Hizmetleri](../artifacts/services.md), [RecentDocs](../artifacts/recentdocs.md),
[Görev Zamanlayıcı Önbelleği](../artifacts/taskcache.md), [SAM Yerel Hesapları](../artifacts/sam.md),
[Kayıt Defteri Otomatik Başlatmaları](../artifacts/persistence.md), [UserAssist](../artifacts/userassist.md),
[BAM ve DAM](../artifacts/bam.md), [Yüklü Programlar](../artifacts/installed-programs.md),
[MuiCache](../artifacts/muicache.md),
[ShellBags](../artifacts/shellbags.md) ve
[USB Aygıtları](../artifacts/usb.md).

Ayrıca bkz. [Aç ve Farklı Kaydet İletişim Kutuları](../artifacts/opensavemru.md).

Ayrıca bkz. [remote-access](../artifacts/remote-access.md).

Ayrıca bkz. [system-info](../artifacts/system-info.md).

Ayrıca bkz. [mru](../artifacts/mru.md).

Ayrıca bkz. [app-history](../artifacts/app-history.md).

Ayrıca bkz. [featureusage](../artifacts/featureusage.md).

Ayrıca bkz. [security-settings](../artifacts/security-settings.md).

Ayrıca bkz. [office](../artifacts/office.md).

Ayrıca bkz. [auditpol](../artifacts/auditpol.md).

Ayrıca bkz. [appcompat](../artifacts/appcompat.md).

Ayrıca bkz. [bcd](../artifacts/bcd.md).

Ayrıca bkz. [lsa-secrets](../artifacts/lsa-secrets.md).

Ayrıca bkz. [networks](../artifacts/networks.md).
