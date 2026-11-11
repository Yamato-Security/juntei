# Кошик { #recycle-bin }

## Що записує кошик { #what-the-recycle-bin-records }

Коли користувач видаляє файл або папку в Провіднику файлів, Windows переміщує їх до кошика того
тому, на якому вони розташовані, і записує поруч невеликий індексний запис: шлях, за яким був
елемент, його розмір і час видалення. Кожен користувач має на кожному томі власну папку кошика,
названу за його SID, тож папка також показує, який обліковий запис видалив елемент.

| Версія Windows | Індекс | Видалений вміст |
|---|---|---|
| Vista і новіші | Один файл `$I…` на кожен елемент (формат версії 1 у Vista–8.1, версії 2 у Windows 10 і новіших) | `$R…` з тим самим іменем після `$R`: сам файл або папка разом із вмістом |
| NT 4.0, 2000, XP, Server 2003 | Один файл `INFO2` на кошик: записи по 800 байтів зі шляхом в ANSI і шляхом у Unicode | `D<drive letter><index><extension>`, наприклад `Dc1.txt` |
| 95, 98, ME | Один файл `INFO2` (`INFO` у Windows 95): записи по 280 байтів лише зі шляхом в ANSI | Такі самі імена `D…` |

<span class="evidence-strength">Найобґрунтованіший висновок:</span> обліковий запис, у кошику якого
міститься запис, перемістив елемент із початкового шляху до кошика в записаний час. Видалення
клавішами Shift+Delete, з командного рядка або з більшості програм відбувається в обхід кошика й не
залишає запису.

## Розташування та збирання { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

Збирайте файли `$I` або INFO2 з кошика кожного тому. Якщо зібрати також файли `$R` і `D…`, буде
видно, які видалені елементи досі там є; без них `RecycledFileFound` дорівнює `false` для кожного
елемента. Видалений вміст сам по собі є доказом і може виявитися шкідливим програмним
забезпеченням: поводьтеся з ним відповідно.

## Застереження для аналітика { #analyst-cautions }

- `DeletedTime` — це час переміщення елемента до кошика за системним годинником, збережений як
  FILETIME в UTC. Це не час очищення кошика. Власні часи створення та зміни елемента містяться в
  `$MFT` і у файлі `$R`.
- SID належить обліковому запису, який видалив елемент, і не обов'язково власнику файлу. Якщо для
  SID немає профілю в `ProfileList` куща SOFTWARE (видалений обліковий запис або доменний обліковий
  запис, який жодного разу не входив у систему), `User` залишається порожнім.
- Коли кошик очищують або елемент відновлюють чи видаляють із кошика, Windows Vista і новіші
  видаляють його файли `$I` і `$R`. Їхні імена можуть залишатися в журналі USN, `$MFT` і slack
  `$I30`; Juntei не відновлює з них видалені записи `$I`. INFO2 зберігає такі записи й обнуляє
  перший байт їхнього шляху в ANSI: `RemovedFromBin` дорівнює `true`, а Juntei відновлює цей символ
  за номером диска.
- `FileSize` у файлі `$I` — це розмір елемента, а для папки — загальний розмір її вмісту. В INFO2
  це місце, яке елемент займав на диску, у цілих кластерах (4 096 байтів для невеликого файлу).
- Шлях ANSI в INFO2 часто записано у формі 8.3, і для нього потрібна кодова сторінка системи:
  використовуйте `--code-page 932` для японської Windows, 936 або 950 — для китайської. Шлях у
  Unicode, якщо він є, міститься в `OriginalPath`.
- Деякі системи Windows Vista записували файли `$I` розміром 543 байти, у яких поле розміру
  коротше на один байт. Juntei читає їхні час і шлях, а `FileSize` залишає порожнім і повідомляє
  про проблему.
- `RecycledFileFound` описує зібрані дані, а не кінцеву точку: `false` може означати, що під час
  збирання файли `$R` не взяли, або що елемента вже немає в кошику.
- Кошики на томах без інсталяції Windows беруть імена користувачів з єдиної інсталяції Windows у
  зібраних даних, якщо вона рівно одна.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

З `-d` Juntei читає файли, імена яких складаються з `$I` і шести літер або цифр (з будь-яким
розширенням), файли `INFO2`, а також `INFO` у папці `RECYCLED`. З `-f` він читає файл незалежно від
його імені.

## Поля виводу CSV { #csv-output-fields }

| Поле | Значення |
|---|---|
| `SourceFile` | Файл `$I` або INFO2. |
| `DeletedTime` | Коли елемент перемістили до кошика (UTC). |
| `OriginalPath` | Де був елемент. Для INFO2 — шлях у Unicode, якщо він є в записі, інакше шлях в ANSI. |
| `FileSize` | `$I`: розмір елемента в байтах (для папки — загальний). INFO2: місце, яке він займає на диску, у цілих кластерах. Порожнє, якщо значення не вдається прочитати. |
| `User`, `SID` | SID папки кошика та ім'я користувача, за яким названо його профіль у `ProfileList`. |
| `RecycledFile`, `RecycledFileFound` | Шлях до видаленого вмісту поруч з індексом (`$R…`, `D<drive><index><extension>`) і чи є він у зібраних даних. |
| `Format`, `FormatVersion` | `$I` 1 (Vista–8.1) або 2 (Windows 10 і новіші); `INFO2` 0 (95), 2 (NT 4.0), 4 (98) або 5 (ME, від 2000 до Server 2003). |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2: номер запису (число в імені його файлу `D…`), диск і місце, де запис починається в INFO2. Шлях UNC не має диска. |
| `RemovedFromBin` | INFO2: `true`, якщо елемент було відновлено або видалено з кошика. |
| `ShortPath` | INFO2: шлях в ANSI, якщо він відрізняється від `OriginalPath` (часто це імена 8.3). |
| `Problems` | Пошкодження: обрізаний або задовгий файл, недійсний номер диска, шлях, що не є коректним UTF-16, час видалення, який не є дійсним часом. |

JSON і JSONL мають ті самі поля, але числа та логічні значення в них типізовані, значення, яких
запис не має, дорівнюють `null`, а `Problems` подано як список.

## Автоматичний аналіз { #automatic-parsing }

`auto-parse` записує `FileExistence/RecycleBin.csv` (`RecycleBin.json` або `RecycleBin.jsonl` з
`-t json` чи `-t jsonl`) разом із `FileExistence/RecycleBin-logs.txt`. Його `--code-page`
застосовується до INFO2. З образів дисків файли `$I` і INFO2 копіюються, а файли `$R` і `D…`
стають порожніми заповнювачами, тож `RecycledFileFound` однаково заповнюється. Об'єднана хронологія
`-j` містить кожен рядок з ідентифікатором `Artifact` `recycle-bin`, `-Z` перетворює `DeletedTime`,
а [`super-timeline`](../commands/super-timeline.md) створює одну подію
`Recycle Bin deletion (moved to the bin)` для кожного елемента.

## Стратегія розслідування { #investigation-strategy }

1. Відсортуйте записи за `DeletedTime` для кожного користувача й порівняйте з часом активності, яку
   ви розслідуєте.
2. Пошукайте `OriginalPath` в LNK, Jump Lists, ShellBags і RecentDocs, щоб з'ясувати, чи
   використовувався елемент до видалення.
3. У журналі USN видалення виглядає як перейменування в `$Recycle.Bin` приблизно в час
   `DeletedTime`; `$MFT` дає власні часи елемента.
4. Якщо `RecycledFileFound` дорівнює `true`, обчисліть хеш і дослідіть вміст `$R` або `D…`.
5. Якщо кошик очистили, шукайте імена `$I` і `$R` у журналі USN і `$MFT`.

## Додаткові відомості { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
