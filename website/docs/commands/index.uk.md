# Список команд { #command-list }

Окремі парсери згруповано за типом джерела. Щоб ознайомитися з ними, виконайте `juntei --help`,
`juntei <SOURCE> --help` або `juntei <SOURCE> <ARTIFACT> --help`:

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Для збирання доказів Windows з кількох комп'ютерів використовуйте
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md). Ця команда викликає ті самі згруповані команди
та впорядковує результати й журнали за комп'ютерами та категоріями розслідування.

## [Образи дисків](mount.md) { #disk-images }

| Команда | Вхідні дані | Докладний посібник |
|---|---|---|
| `mount` | Образ диска VHDX або E01 | [Монтування образів дисків](mount.md) |

## [Команди для ESE](user-activity.md) { #ese-commands }

| Команда | Вхідний артефакт | Докладний посібник |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [Журналювання доступу користувачів (UAL)](../artifacts/ual.md) |
| `parse-ese webcache` | ESE `WebCacheV01.dat` | [Internet Explorer і застарілий Edge](../artifacts/webcache.md) |

## [Команди для файлів](execution.md) { #file-commands }

| Команда | Вхідний артефакт | Докладний посібник |
|---|---|---|
| `parse-file defender-quarantine` | Дерево або запис карантину Defender | [Карантин Defender](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [Ярлики LNK](../artifacts/lnk.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [Команди NTFS](filesystem.md) { #ntfs-commands }

| Команда | Вхідний артефакт | Докладний посібник |
|---|---|---|
| `parse-ntfs boot` | `$Boot` або образ тому NTFS | [Завантажувальний сектор ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`) або образ диска | [Індекси каталогів ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | NTFS `$LogFile` | [Журнал транзакцій NTFS](../artifacts/logfile.md) |
| `parse-ntfs mft` | NTFS `$MFT` | [Головна файлова таблиця](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [Дескриптори безпеки ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [Журнал USN](../artifacts/usn-journal.md) |

## [Команди для реєстру](security-registry.md) { #registry-commands }

| Команда | Вхідний артефакт | Докладний посібник |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [Мережі](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [Секрети LSA](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (і `BCD-Template`) | [Дані конфігурації завантаження (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [Сумісність програм](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [Політика аудиту](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage, RecentApps і закріплені на панелі завдань елементи](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [Історія програм](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [Списки MRU](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (із `SAM`, `SOFTWARE` для імен користувачів) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (`NTUSER.DAT` у Windows XP) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [Встановлені програми](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [Діалоги відкриття та збереження файлів](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [Автозапуск через реєстр](../artifacts/persistence.md) |
| `parse-registry read` | Будь-який основний кущ реєстру | [Кущі реєстру](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (із `SOFTWARE` для SID користувачів) | [Віддалений доступ (з боку користувача)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [Локальні облікові записи SAM](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [Параметри безпеки](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Служби Windows](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` або експортоване значення AppCompatCache | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` однієї системи | [Відомості про систему](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Кеш планувальника завдань](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM` разом із `SOFTWARE` і `NTUSER.DAT` | [Пристрої USB](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## [Команди для SQLite](sqlite.md) { #sqlite-commands }

| Команда | Вхідний артефакт | Докладний посібник |
|---|---|---|
| `parse-sqlite browser` | `History`, `Cookies`, `Web Data`, `places.sqlite`, `cookies.sqlite`, `formhistory.sqlite` | [Веб-браузери](../artifacts/browser.md) |

## Типи джерел і зміни форматів { #source-types-and-format-changes }

Групи описують формати, які Juntei наразі підтримує. `parse-registry amcache` зчитує
`Amcache.hve`; це не означає підтримки баз даних інвентаризації Amcache у форматі SQLite. Крейти парсерів
артефактів залишаються окремими, тож підтримку іншого формату зберігання можна додати незалежно.

Колишні пласкі назви парсерів замінено наведеними вище шляхами «джерело й артефакт». Указуйте
наявні параметри парсера після назви артефакту. `auto-parse` та наявні підкоманди реєстру
зберігають свої назви.

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` виконує пошук у кількох форматах джерел, тому це команда аналізу верхнього
рівня. Вона доповнює повну інвентаризацію конфігурації з `parse-registry persistence`
131 перевіркою, класифікацією за списками дозволу/блокування, метаданими корисного навантаження, відомостями про охоплення та зіставленням часових позначок.

## Використання верхнього рівня { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX or E01 disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  parse-sqlite        Parse SQLite databases
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Сторінки джерел відтворюють довідку щодо використання команд і наводять короткі приклади. Посібники з артефактів пояснюють
докази, поля виводу, обмеження та стратегію розслідування.
