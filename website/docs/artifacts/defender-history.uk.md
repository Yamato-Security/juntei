# Історія виявлень Microsoft Defender { #microsoft-defender-detection-history }

## Що записує історія { #what-the-history-records }

Microsoft Defender записує окремий файл історії виявлень для кожного виявлення. Кожен файл містить:

- ідентифікатор, назву, рівень серйозності та категорію загрози;
- що зробив Defender і чи вдалося це;
- чи була загроза запущена;
- причетних користувача та процес, а також те, що виявило загрозу;
- запис карантину, якщо він є;
- кожен ресурс, у якому її знайдено: файли, елементи архівів, завантаження, процеси, значення реєстру,
  служби, командні рядки та поведінку.

Defender записує ці файли й для загроз, які він ніколи не поміщав у карантин: тих, які він лише
виявив, тих, які було дозволено, і тих, які йому не вдалося видалити. Журнал захисту в Безпеці Windows і
клас WMI `MSFT_MpThreatDetection` (`Get-MpThreatDetection`) показують ці записи. Коли Orange
Cyberdefense змінювала значення у файлі, WMI повідомляв змінені значення.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> Defender зафіксував виявлення
названої загрози в перелічених ресурсах із записаними станом і часами. Назва загрози — це вердикт
продукту. Сам лише запис про виявлення не показує, що загрозу було запущено або що її було повністю
видалено.

## Розташування та збирання { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

Кожен файл названо за його `DetectionID`. Збирайте всю папку `DetectionHistory`. Вхідний каталог і
`auto-parse` читають кожен файл усередині папки `DetectionHistory` на будь-якій глибині, якщо він
починається як файл історії виявлень. Для копії, що зберігається поза такою папкою, використовуйте
`-f`. Defender видаляє елементи з історії сканування через налаштовувану кількість днів
(`ScanPurgeItemsAfterDelay` у `Set-MpPreference`), тому давніших виявлень уже може не бути.

Карантин — окремий артефакт: див. [Карантин Defender](defender-quarantine.md). `QuarantineID`
виявлення має те саме написання, що й `EntryID` у виводі карантину (великі літери, без фігурних
дужок), тож їх можна об'єднати.

## Застереження для аналітика { #analyst-cautions }

- Виявлення, спроба усунення загрози та успішне видалення — це окремі стани. Читайте
  `ThreatStatus` (`Detected`, `Quarantined`, `Removed`, `Allowed`, `RemoveFailed`, …) разом із
  `ThreatStatusErrorCode`, що дорівнює 0, якщо остання зміна стану була успішною.
- `ExecutionStatus` — це погляд Defender на момент виявлення (`Blocked`, `Allowed`, `Executing`,
  `NotExecuting`, `Unknown`). Це не доказ ні того, що загрозу було запущено, ні того, що її ніколи не
  запускали. Зіставляйте з Prefetch, Amcache, журналами подій і телеметрією процесів.
- `ThreatName` — це назва сигнатури на той момент. Можливі хибні спрацьовування та пізніша
  перекласифікація. Тестові файли, як-от EICAR, повідомляються як справжні загрози.
- `User` — це значення `DomainUser`, яке записав Defender (`UserSource` дорівнює `DomainUser`). Воно
  може бути порожнім, обліковим записом служби або іншим користувачем, ніж той, хто створив файл.
- Defender може записати кілька файлів для однієї загрози, і Juntei зберігає кожен із них. Два файли
  можуть навіть містити однаковий `DetectionID`, як буває зі скопійованим або відновленим файлом.
  Групуйте за `DetectionID` і `SourceFile` разом.
- Значення, зміст яких невідомий, зберігаються без інтерпретації: їхні розділ і позиція наведені в
  `UnknownValues`.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

Типовий формат файлу — CSV. З `-t csv`, `-t json` або `-t jsonl` без `-o` записи надходять у stdout.
Без обох параметрів відображається лише підсумок. Додайте `-C`, щоб перезаписати наявний результат.
Вивід ніколи не записується поверх одного з вхідних файлів, навіть із `-C`.

Juntei створює один запис на кожен ресурс кожного виявлення. Виявлення без ресурсів отримує один
запис із порожніми полями ресурсу. Записи одного виявлення мають спільний `DetectionID` і нумеруються
за `ResourceIndex` у порядку зберігання.

Пошкодження не зупиняє роботу. Обрізаний файл зберігає значення, прочитані до місця пошкодження, а
`Problems` повідомляє, чого бракує. Про файл у папці `DetectionHistory`, який не є файлом історії
виявлень, повідомляється, і його пропущено. Інші файли все одно читаються.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Record` | `resource` або `detection` для виявлення без ресурсів. |
| `InitialDetectionTime` | Коли Defender уперше виявив загрозу (UTC). Порожнє, якщо файл містить 0; значення, яке не є часом, зберігається в `UnknownValues`. |
| `LastThreatStatusChangeTime` | Коли востаннє змінився стан загрози (UTC). |
| `RemediationTime` | Коли Defender усунув загрозу (UTC). Це час спроби, а не доказ видалення; перевіряйте `ThreatStatus`. |
| `ThreatName`, `ThreatID` | Назва сигнатури Defender і числовий ідентифікатор (`Get-MpThreatCatalog`). |
| `Severity`, `SeverityID` | `Low`, `Moderate`, `High`, `Severe` або `Unknown` разом зі збереженим числом. |
| `Category`, `CategoryID` | Категорія загрози, як-от `TROJAN`, `TOOL` або `BEHAVIOR`, разом зі збереженим числом. |
| `ThreatStatus`, `ThreatStatusID` | Стан загрози за даними Defender. Число без задокументованої назви зберігається з порожнім `ThreatStatus`. |
| `ThreatStatusErrorCode` | Код помилки останньої зміни стану; 0 означає успіх. |
| `ExecutionStatus`, `ExecutionStatusID` | Чи бачив Defender, що загроза виконувалася в момент виявлення. |
| `DetectionSource`, `DetectionSourceTypeID` | Що виявило загрозу: `Real-time`, `IOAV` (завантаження та вкладення), `User`, `System`, `AMSI` та інші. |
| `User`, `UserSource` | Значення `DomainUser`, яке записав Defender, і його підстава (`DomainUser`). Порожнє, якщо у файлі не вказано користувача. |
| `Process` | Причетний процес (`ProcessName`); може бути `Unknown`. |
| `ResourceIndex`, `ResourceCount` | Позиція цього ресурсу та кількість ресурсів у виявленні. |
| `ResourceType` | `file`, `containerfile`, `webfile`, `process`, `regkeyvalue`, `service`, `CmdLine`, `behavior`, `amsi` та інші — так, як їх називає Defender. |
| `Path` | Ресурс: шлях, елемент архіву (`archive.zip->member`), значення реєстру, командний рядок або процес. Для `webfile` також указано URL-адресу та процес, що завантажив файл (`pid:…,ProcessStart:…`). Зберігайте точне написання. |
| `ResourceFlags` | Прапорці ресурсу в шістнадцятковому вигляді; їхнє значення не задокументовано. |
| `TrackingStartTime` | `ThreatTrackingStartTime` ресурсу — ціле число, прочитане як FILETIME (UTC), так само як plaso датує виявлення. |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | Хеші та розмір із даних відстеження ресурсу. Вони описують файл на момент сканування Defender. |
| `TrackingID` | `ThreatTrackingId` ресурсу. |
| `OtherTracking` | Усі інші значення відстеження у вигляді `Key=Value` в порядку зберігання, як-от `ThreatTrackingScanSource` або `ThreatTrackingBMContextRich` для поведінкових виявлень. |
| `QuarantineID` | Запис карантину (`Quarantine\Entries\{GUID}`); порожнє, якщо файл містить нульовий GUID. |
| `DetectionID` | GUID виявлення; також назва файлу. Розбіжність повідомляється в `Problems`. |
| `MagicVersion` | `Magic.Version:1.2` із заголовка; порожнє у файлах, записаних без нього, як-от деякі з Windows Server 2016. |
| `UnknownValues` | Значення з невідомим змістом у вигляді `position=value` (наприклад, `6.13=NT AUTHORITY\SYSTEM`); список із лічильником після стану подано як `2.list`. |
| `SourceFile` | Прочитаний файл: шлях відносно вхідного каталогу або його назва з `-f`. |
| `Problems` | Пошкодження, відсутні значення, неочікувані типи або назва файлу, що не збігається. Не приховуйте. |

JSON і JSONL мають ті самі поля з типізованими значеннями: числа залишаються числами, а відсутні
значення дорівнюють `null`. Замість `OtherTracking` вони мають `Tracking` — кожне значення відстеження
з ключем Defender, його типом і значенням. `UnknownValues` стає списком позицій із типами.
`CountedList` містить список із лічильником. Збережений FILETIME зберігає сире число тактів поруч із
часом, який із нього прочитано.

## Поведінка в хронології { #timeline-behavior }

`auto-parse` зберігає записи як `SecurityProducts/DefenderHistory.csv` (або `.json`/`.jsonl`) і додає
їх до об'єднаної хронології `-j`. `super-timeline` створює події з `InitialDetectionTime`,
`LastThreatStatusChangeTime` і `RemediationTime` у записі кожного ресурсу, а стан, код помилки, стан
виконання та джерело виявлення потрапляють до подробиць події. `TrackingStartTime` зберігається як ціле
число, тому стає подією лише з `--include-interpreted-times`. Власні часи файлу ніколи не
використовуються. Запис без придатного часу залишається в результатах розбору й зараховується до
записів без часу.

## Стратегія розслідування { #investigation-strategy }

1. Почніть із `ThreatName`, `Path`, `SHA256`, `InitialDetectionTime` і `ThreatStatus`.
2. Перш ніж робити висновок, що загрозу видалено, перевірте `ThreatStatusErrorCode` і `Problems`.
3. Об'єднайте `QuarantineID` з `EntryID` карантину, щоб відновити байти з карантину.
4. Використовуйте хеші та шляхи як точки переходу до `$MFT`, журналу USN, Prefetch, Amcache, історії
   браузера та журналів подій (операційний журнал Defender: події 1116, 1117, 1118 і 1119).
5. Для ресурсів `webfile` використовуйте URL-адресу та час запуску процесу для дослідження початкового
   доступу й перевіряйте їх доказами з браузера, проксі та DNS.

## Перевірені версії { #tested-versions }

Тести використовують публічні файли, результати яких незалежно прочитали інші інструменти:

- Зразок Windows 10 від Orange Cyberdefense (Defender 4.18.24090.11), звірений зі значеннями, які
  повідомив WMI.
- Два тестові файли Windows із plaso, звірені з парсером `windefender_history` у plaso.
- 14 файлів із віртуальної машини симулятора APT у DFIRArtifactMuseum.
- Копія, записана без версії в заголовку, як деякі файли з Windows Server 2016.
- Обрізані й порожні файли, а також файли іншого типу.

## Додаткові відомості { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [Orange Cyberdefense wddh-parser](https://github.com/cert-orangecyberdefense/wddh-parser)
- [Парсер історії Defender у plaso](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: клас MSFT_MpThreatDetection](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: ідентифікатори подій Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
