# रेजिस्ट्री कमांड { #registry-commands }

```text
Version: 1.0.0 (CODE BLUE Release)
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

Canonical commands `app-compat`, `feature-usage`, `open-save-mru`, `recent-docs`,
`user-assist` और `shell-bags` हैं। मौजूदा scripts के लिए इनकी पुरानी वर्तनियाँ (`appcompat`, `featureusage`,
`opensavemru`, `recentdocs`, `userassist` और `shellbags`) छिपे हुए aliases के रूप में स्वीकार की जाती रहती हैं।
स्वचालित parsing canonical नामों का उपयोग करता है; इसके artifact identifiers,
result schemas और output filenames अपरिवर्तित हैं।

Registry commands `-f <HIVE>` या `-d <COLLECTION>` और structured output options स्वीकार करते हैं।
`amcache` hive dirty होने पर साथ वाले Amcache transaction logs को अपने-आप replay करता है, लेकिन logs को नामित करने या replay छोड़ने का कोई option नहीं देता। `shimcache`
default रूप से साथ वाले SYSTEM logs replay करता है और export किया गया कच्चा AppCompatCache value भी स्वीकार करता है।
अन्य registry commands साथ वाले logs को अपने-आप replay करते हैं; logs को स्पष्ट रूप से नामित करने के लिए `--log1` / `--log2` का,
या replay के बिना primary hive की जाँच करने के लिए `--no-logs` का उपयोग करें।

## `parse-registry amcache` { #parse-registry-amcache }

```text
Version: 1.0.0 (CODE BLUE Release)
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

[Amcache](../artifacts/amcache.md) देखें।

## `parse-registry bam` { #parse-registry-bam }

```text
Version: 1.0.0 (CODE BLUE Release)
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

उपयोगकर्ताओं के नाम सिस्टम के `SAM` और `SOFTWARE` (`ProfileList`) से लिए जाते हैं, जो host की तरह ही खोजे जाते हैं।
[BAM / DAM](../artifacts/bam.md) देखें।

## `parse-registry installed-programs` { #parse-registry-installed-programs }

```text
Version: 1.0.0 (CODE BLUE Release)
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

`Uninstall` entries, Windows Installer products, App Paths, WSL distributions और packaged apps।
User hive का उपयोगकर्ता उसके profile folder और सिस्टम की `ProfileList` से आता है।
[इंस्टॉल किए गए प्रोग्राम](../artifacts/installed-programs.md) देखें।

## `parse-registry muicache` { #parse-registry-muicache }

```text
Version: 1.0.0 (CODE BLUE Release)
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

उपयोगकर्ताओं के नाम collection में hive के स्थान से, और उनके SIDs `UsrClass.dat` की
root key या सिस्टम के `SOFTWARE` (`ProfileList`) से लिए जाते हैं। [MuiCache](../artifacts/muicache.md) देखें।

## `parse-registry open-save-mru` { #parse-registry-open-save-mru }

```text
Version: 1.0.0 (CODE BLUE Release)
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

हर shell item सूची ys-lnk-parser से decode की जाती है और एक path में जोड़ी जाती है।
[खोलें और इस रूप में सहेजें संवाद](../artifacts/opensavemru.md) देखें।

## `parse-registry remote-access` { #parse-registry-remote-access }

```text
Version: 1.0.0 (CODE BLUE Release)
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

उपयोगकर्ताओं के SIDs सिस्टम के `SOFTWARE` (`ProfileList`) से आते हैं, जो host की तरह ही खोजा जाता है। संग्रहीत
passwords को मौजूद के रूप में report किया जाता है, कभी decode नहीं किया जाता।
[रिमोट एक्सेस (उपयोगकर्ता पक्ष)](../artifacts/remote-access.md) देखें।


## `parse-registry system-info` { #parse-registry-system-info }

```text
Version: 1.0.0 (CODE BLUE Release)
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

प्रति सिस्टम एक record: उसके नाम, domain या workgroup, Windows version और install तिथि, time
zone, अंतिम shutdown, अंतिम उपयोगकर्ता, profiles और SIDs। एक directory के hives एक ही सिस्टम के होते हैं;
`-f` के साथ, दिए गए hives और उनके बगल में मिले चार में से अन्य hives।
[सिस्टम जानकारी](../artifacts/system-info.md) देखें।


## `parse-registry mru` { #parse-registry-mru }

```text
Version: 1.0.0 (CODE BLUE Release)
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

[MRU सूचियाँ](../artifacts/mru.md) देखें।


## `parse-registry app-history` { #parse-registry-app-history }

```text
Version: 1.0.0 (CODE BLUE Release)
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

Sysinternals tools, WinRAR, 7-Zip और WinZip इतिहास, OneDrive accounts, Adobe की हाल की फ़ाइलें,
TeamViewer और CCleaner। [ऐप्लिकेशन इतिहास](../artifacts/app-history.md) देखें।


## `parse-registry feature-usage` { #parse-registry-feature-usage }

```text
Version: 1.0.0 (CODE BLUE Release)
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

`FeatureUsage`, `RecentApps`, `Taskband` और `ProgramsCache` पढ़ता है।
[FeatureUsage, RecentApps और Taskbar पिन](../artifacts/featureusage.md) देखें।


## `parse-registry security-settings` { #parse-registry-security-settings }

```text
Version: 1.0.0 (CODE BLUE Release)
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

मिली हर setting एक record है, जिसमें उसका value, उसका अर्थ और यह होता है कि क्या वह Windows के default व्यवहार
को कमज़ोर करती है; `--weakened-only` केवल ऐसी settings रखता है।
[सुरक्षा सेटिंग](../artifacts/security-settings.md) देखें।


## `parse-registry office` { #parse-registry-office }

```text
Version: 1.0.0 (CODE BLUE Release)
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

[Microsoft Office](../artifacts/office.md) देखें।

### उदाहरण { #examples }

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

पूरे options और fields के अर्थ [रेजिस्ट्री हाइव](../artifacts/registry.md),
[Windows सेवाएँ](../artifacts/services.md), [RecentDocs](../artifacts/recentdocs.md),
[Microsoft Office](../artifacts/office.md),
[टास्क शेड्यूलर कैश](../artifacts/taskcache.md), [SAM स्थानीय खाते](../artifacts/sam.md),
[रेजिस्ट्री ऑटोस्टार्ट](../artifacts/persistence.md) और [UserAssist](../artifacts/userassist.md) पर दिए गए हैं।


## `parse-registry auditpol` { #parse-registry-auditpol }

```text
Version: 1.0.0 (CODE BLUE Release)
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

`Policy\PolAdtEv` में audit policy की प्रति subcategory (Windows Vista और बाद के) या category (XP और 2003)
एक record: यह क्या log करती है, उसका GUID और message ID, और उसके मुख्य events।
[ऑडिट नीति](../artifacts/auditpol.md) देखें।


## `parse-registry app-compat` { #parse-registry-app-compat }

```text
Version: 1.0.0 (CODE BLUE Release)
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

Compatibility layers, Program Compatibility Assistant के `Store` और `Persisted`, CIT और custom
shim databases। ShimCache `parse-registry shimcache` का विषय है।
[ऐप्लिकेशन संगतता](../artifacts/appcompat.md) देखें।


## `parse-registry bcd` { #parse-registry-bcd }

```text
Version: 1.0.0 (CODE BLUE Release)
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

Store के प्रति object एक record, जिसमें test signing, integrity checks, recovery, boot status
policy और safe mode inheritance के माध्यम से resolve किए जाते हैं।
[बूट कॉन्फ़िगरेशन (BCD)](../artifacts/bcd.md) देखें।


## `parse-registry lsa-secrets` { #parse-registry-lsa-secrets }

```text
Version: 1.0.0 (CODE BLUE Release)
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

प्रति LSA secret (`Policy\Secrets\<name>`) एक record, उसके प्रकार, उसके द्वारा नामित service या उपयोगकर्ता,
उसकी values कब सेट की गईं और क्या उनमें data है, के साथ; और cached logons के लिए एक record
(`Cache`)। केवल नाम और समय: कुछ भी decrypt नहीं किया जाता। [LSA सीक्रेट](../artifacts/lsa-secrets.md) देखें।


## `parse-registry networks` { #parse-registry-networks }

```text
Version: 1.0.0 (CODE BLUE Release)
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

Profile के समय सिस्टम के `SYSTEM` hive (जो host की तरह ही खोजा जाता है) के time zone से UTC में convert किए जाते हैं,
और जैसे मिले वैसे भी रखे जाते हैं। [नेटवर्क](../artifacts/networks.md) देखें।


## `parse-registry persistence` { #parse-registry-persistence }

```text
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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

[ShimCache](../artifacts/shimcache.md) देखें।

## `parse-registry taskcache` { #parse-registry-taskcache }

```text
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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

### उदाहरण { #examples_1 }

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

पूरे options और fields के अर्थ [रेजिस्ट्री हाइव](../artifacts/registry.md),
[Windows सेवाएँ](../artifacts/services.md), [RecentDocs](../artifacts/recentdocs.md),
[टास्क शेड्यूलर कैश](../artifacts/taskcache.md), [SAM स्थानीय खाते](../artifacts/sam.md),
[रेजिस्ट्री ऑटोस्टार्ट](../artifacts/persistence.md), [UserAssist](../artifacts/userassist.md),
[BAM / DAM](../artifacts/bam.md), [इंस्टॉल किए गए प्रोग्राम](../artifacts/installed-programs.md),
[MuiCache](../artifacts/muicache.md),
[ShellBags](../artifacts/shellbags.md) और
[USB डिवाइस](../artifacts/usb.md) पर दिए गए हैं।

[खोलें और इस रूप में सहेजें संवाद](../artifacts/opensavemru.md) भी देखें।

[remote-access](../artifacts/remote-access.md) भी देखें।

[system-info](../artifacts/system-info.md) भी देखें।

[mru](../artifacts/mru.md) भी देखें।

[app-history](../artifacts/app-history.md) भी देखें।

[featureusage](../artifacts/featureusage.md) भी देखें।

[security-settings](../artifacts/security-settings.md) भी देखें।

[office](../artifacts/office.md) भी देखें।

[auditpol](../artifacts/auditpol.md) भी देखें।

[appcompat](../artifacts/appcompat.md) भी देखें।

[bcd](../artifacts/bcd.md) भी देखें।

[lsa-secrets](../artifacts/lsa-secrets.md) भी देखें।

[networks](../artifacts/networks.md) भी देखें।
