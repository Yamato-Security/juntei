# Списки MRU — нещодавно використані елементи (RunMRU, TypedPaths, WordWheelQuery, TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## Що це за списки { #what-the-lists-are }

Windows і її програми зберігають короткі списки того, що користувач останнім часом вводив або відкривав, у файлі
`NTUSER.DAT` користувача:

- **RunMRU:** команди, введені в діалоговому вікні «Виконати» (Win+R): `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`.
- **TypedPaths:** шляхи, введені в адресному рядку Провідника.
- **WordWheelQuery:** пошукові запити, введені в полі пошуку Провідника.
- **TypedURLs:** адреси, введені в адресному рядку Internet Explorer, а починаючи з Windows 8 —
  `TypedURLsTime`, тобто час введення кожної з них.
- **Paint і WordPad:** файли, які кожна з програм відкривала останніми (`Recent File List`).
- **ACMru (Windows XP):** запити помічника з пошуку (Search Assistant) за типами пошуку.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> користувач ввів цю команду, шлях,
пошуковий запит або адресу, або програма відкрила цей файл. Датується лише найновіший запис кожного списку —
за часом останнього запису його ключа, за винятком введених адрес, які датує `TypedURLsTime`.

## Розташування { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

Підрозділи `ACMru` відповідають типам пошуку: `5603` — імена файлів і папок, `5604` — слова чи фрази у
файлах, `5647` — комп'ютери та люди, `5001` — Інтернет.

Збирайте `NTUSER.DAT` разом із журналами транзакцій для кожного профілю.

## Застереження для аналітика { #analyst-cautions }

- **Один час на список.** Час останнього запису ключа змінюється під час зміни списку, тому він датує
  лише найновіший запис (`IsMostRecent`, `LastUsedUTC`). Решта записів упорядковані, але не датовані.
  `TypedURLsTime` датує кожну введену адресу (`TypedTimeUTC`).
- **Введено, а не виконано.** RunMRU фіксує те, що було введено в діалоговому вікні «Виконати», незалежно від того, чи
  було виконання успішним. Windows зберігає кожну команду з кінцевим `\1`; Juntei його видаляє. Зіставляйте
  з Prefetch, BAM, UserAssist і подіями створення процесів.
- **Списки короткі.** Кожен список зберігає лише найновіші записи, а параметри конфіденційності та засоби очищення
  їх стирають. Для видалених значень використовуйте `parse-registry read --recover-deleted`.
- **Запити — це те, що було введено.** WordWheelQuery і ACMru зберігають введений текст, а не те, що було знайдено.
- **Порядок визначає список.** `MRUList` у RunMRU та `MRUListEx` у WordWheelQuery задають
  порядок; для решти його визначає номер значення (`url1`, `File1` і `000` — найновіші).

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

Зведення наводить кількість записів у кожному списку та датовані записи (введені адреси й найновіший запис
кожного списку), починаючи з найновіших.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Хост, власник профілю (зі шляху або з папок оболонки), SID із `ProfileList` системи та кущ реєстру. |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad` або `ACMru`. |
| `Subkey` | Підрозділ `ACMru`: тип пошуку (`5603`, `5604`, `5647`, `5001`). |
| `MRU-Position`, `IsMostRecent` | Місце у списку (0 — найновіший); порожнє, якщо список не називає це значення. |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | Команда (без `\1`), шлях, пошуковий запит, адреса або файл. |
| `TypedTimeUTC` | Час введення адреси, з `TypedURLsTime`. |
| `LastUsedUTC` | Час останнього запису ключа — лише для найновішого запису. |
| `KeyPath`, `KeyLastWrittenUTC` | Ключ і час його останнього запису: це час найновішого запису, а не кожного рядка. |
| `DataRaw` | Байти значення (у шістнадцятковому вигляді), якщо їх не вдається декодувати. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Значення, які список називає, але яких немає в ключі; значення, яких список не називає; неправильно названі значення; неправильні типи; обрізані запити; часові позначки, що не є FILETIME. |

## Стратегія розслідування { #investigation-strategy }

1. Перегляньте RunMRU на наявність команд, скриптів, мережевих спільних папок та засобів адміністрування, введених користувачем.
2. Перегляньте TypedPaths і WordWheelQuery на наявність папок і файлів, які шукав користувач:
   облікові дані, експортовані дані, архіви, знімні та мережеві розташування.
3. Нанесіть введені адреси на часову шкалу за допомогою `TypedTimeUTC`.
4. Зіставляйте з Prefetch, BAM, UserAssist, ShellBags, файлами LNK, Jump Lists та історією браузера.

## Додаткові відомості { #further-details }

- [libyal winreg-kb: ключі нещодавно використаних елементів (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
