# Microsoft Office (нещодавні файли, надійні документи, параметри макросів) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## Що записує Office { #what-office-records }

Microsoft Office зберігає для кожного користувача в `NTUSER.DAT`, а також для кожної версії та програми:

- **Нещодавні файли та розташування:** `File MRU` — документи, які відкривала програма, і `Place MRU` —
  папки, з яких вона їх відкривала, кожен елемент із часом останнього відкриття. Microsoft 365
  також веде окремий список для кожного облікового запису, з яким виконано вхід (`User MRU\<account>`).
- **Надійні документи (Trusted Documents):** коли користувач натискає *Enable Editing* (Увімкнути редагування) або *Enable Content* (Увімкнути вміст) у документі,
  Office запам'ятовує це в `TrustRecords`: шлях документа, час, коли його визнано надійним, і чи було
  увімкнуто його макроси.
- **Параметри макросів:** `VBAWarnings` (які макроси виконуються), `BlockContentExecutionFromInternet`
  (чи блокуються макроси у файлах з Інтернету), `AccessVBOM` (чи можуть програми
  змінювати проєкти VBA) і **надійні розташування (Trusted Locations)** — папки, макроси документів у яких виконуються без
  запиту. Групова політика може задавати те саме в `Software\Policies`.

Шкідливий документ зазвичай потребує, щоб користувач увімкнув його макроси: запис довіри з увімкнутими
макросами пов'язує документ, користувача і час.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> користувач відкривав цей документ у цій
програмі (востаннє — у цей час); користувач увімкнув редагування або макроси цього документа в
цей час; Office було налаштовано виконувати макроси саме так.

## Розташування { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

Версії: `16.0` — це Office 2016 і новіші (2019, 2021, 2024, Microsoft 365), `15.0` — Office 2013,
`14.0` — Office 2010, `12.0` — Office 2007.

Елемент має вигляд `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx`: `T`
— час останнього відкриття (FILETIME у шістнадцятковому вигляді), шлях іде після `*`. Запис довіри має розмір 24
байти: FILETIME, коли документ визнано надійним, потім його прапорці в останніх 4 байтах — `0x7FFFFFFF`,
коли було увімкнуто його макроси, і `0x00000001`, коли було увімкнуто лише редагування.

## Застереження для аналітика { #analyst-cautions }

- **Час.** `LastOpenedUTC` — це власний час кожного елемента. `TrustedAtUTC` — це час, коли користувач визнав
  документ надійним, тобто момент натискання *Enable Editing* або *Enable Content*, який може передувати
  останньому відкриттю.
- **Макроси увімкнуто, а не виконано.** `MacrosEnabled` вказує, що користувач увімкнув макроси документа; це
  не доводить, що макрос виконувався або що він робив. Зіставляйте зі створенням процесів, Prefetch, журналами
  PowerShell і самим документом.
- **Шляхи у тому вигляді, як їх зберігає Office.** Записи довіри називають документ шляхом, який може містити
  `%USERPROFILE%`, прямі скісні риски, URL `file:///` і URL-кодування (`%20`). Елементи MRU
  хмарних документів є URL-адресами.
- **Типові значення та фактична політика.** Office створює типові надійні розташування (`Templates`,
  `XLSTART`, `STARTUP`, …). `Date` розташування — корисна підказка, але не доказ того, хто його створив:
  його також можуть заповнити адміністратори, скрипти, політики та скопійовані параметри реєстру. Відсутність
  `VBAWarnings` сама по собі не встановлює фактичної політики макросів. Значення мають також версія Office, позначення
  файлів з Інтернету та інші параметри керування.
  [Microsoft пояснює блокування макросів з Інтернету](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked).
- **Видалені документи.** Списки існують довше за документи; відсутній файл усе ще може бути показано
  тут. Списки короткі, і їх очищують функція Office *Clear unpinned items* (Очистити незакріплені елементи) та засоби очищення.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

Зведення перелічує документи, макроси яких було увімкнуто, параметри, що дозволяють макросам виконуватися
вільніше (`VBAWarnings` 1, `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1,
`AllowNetworkLocations` 1), надійні розташування, які додали користувачі або встановила політика, і документи,
відкриті останніми.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Хост, власник профілю (зі шляху або папок оболонки), SID із `ProfileList` системи та кущ. |
| `OfficeVersion`, `App` | Ключ версії (`16.0`) і програма (`Word`, `Excel`, …). |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security` або `Trusted Locations`. |
| `Account` | Обліковий запис Microsoft 365 списку `User MRU`. |
| `Policy` | Чи встановлено параметр або розташування політикою. |
| `MRU-Position`, `IsMostRecent` | Місце елемента в його списку (`Item 1` — це 0, найновіший). |
| `Path` | Документ або папка; шлях надійного документа у збереженому вигляді; `Path` надійного розташування. |
| `LastOpenedUTC` | Коли елемент відкривали востаннє. |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | Коли документ визнано надійним, чи було увімкнуто його макроси, і прапорці запису. Невідомі прапорці залишають `MacrosEnabled` порожнім і створюють діагностичне повідомлення. |
| `Setting`, `Value` | Параметр макросів і його значення з поясненням (`1 (all macros enabled)`); ключ надійного розташування (`Location3`) та його інші значення (`AllowSubFolders=1; Date=…; Description=…`). |
| `KeyPath`, `KeyLastWrittenUTC` | Ключ і його останній запис. |
| `DataRaw` | Байти значення (hex), якщо вони не декодуються. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Елементи без `*` або з часом, який не є часом, записи довіри не на 24 байти, розташування без `Path`, неправильні типи. |

## Стратегія розслідування { #investigation-strategy }

1. Почніть із `MacrosEnabled` зі значенням true: документи, коли і ким. Знайдіть файли (`$MFT`,
   журнал USN, `Zone.Identifier`), їхнє джерело (історія браузера та пошти) і те, що виконувалося далі.
2. Перевірте параметри макросів і надійні розташування: усі макроси увімкнуто, макроси з Інтернету не
   блокуються, доступ до проєктів VBA або розташування, додане користувачем (у `Downloads`, `Temp`, мережевому спільному ресурсі),
   послаблюють типові налаштування.
3. Нанесіть `LastOpenedUTC` на хронологію разом із файлами LNK, Jump Lists і RecentDocs для
   шляхів документів, включно зі знімними та мережевими розташуваннями.

## Додаткові відомості { #further-details }

- [Microsoft: увімкнення або вимкнення макросів у файлах Microsoft 365](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: надійні документи](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: додавання, видалення або змінення надійного розташування](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
