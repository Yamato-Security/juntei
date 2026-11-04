# RecentDocs { #recentdocs }

## Що таке RecentDocs { #what-recentdocs-is }

Провідник Windows зберігає списки нещодавно використаних файлів і папок у `NTUSER.DAT` кожного користувача.
Кореневий список охоплює нещодавні елементи всіх типів; підрозділи групують розширення, як-от `.docx`, а також є
список `Folder`. `MRUListEx` визначає порядок нумерованих значень.

RecentDocs допомагає визначити документи, з якими взаємодіяв користувач, зокрема файли, які згодом було
видалено або переміщено. Значення також можуть містити назву ярлика, який Windows використала в папці Recent
користувача; її можна зіставити з доказами LNK.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> Провідник записав елемент у список
RecentDocs цього користувача. Час ключа датує лише найновіший елемент у цьому конкретному списку.

## Розташування { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

Збирайте `NTUSER.DAT` і його журнали транзакцій для кожного відповідного профілю, а також каталог LNK Recent
користувача.

## Застереження для аналітика { #analyst-cautions }

- `MRUListEx` дає порядок, а не часову позначку для кожного елемента.
- Час останнього запису ключа відповідає найновішому оновленню; лише позиція MRU 0 отримує
  обґрунтоване значення `LastOpenedUTC`, похідне від нього.
- Елемент може з'являтися і в кореневому списку, і в списку розширення. Не рахуйте це як два відкриття.
- Поведінка програм і Провідника може відрізнятися; деякі способи доступу можуть не оновлювати RecentDocs.
- Засоби очищення/захисту приватності та обмеження розміру списку видаляють старіші докази.
- Ім'я файлу може контролювати зловмисник, і воно не доводить тип вмісту.
- Користувача визначають за шляхами профілю або даними папок оболонки; перевіряйте порожні/неоднозначні імена користувачів.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

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
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host` | Хост, визначений із контексту збирання/системи, якщо він доступний. |
| `User` | Власник профілю, визначений за шляхом або шляхами папок оболонки; якщо поле порожнє, зберігайте невизначеність. |
| `SourcePath` | Вхідний `NTUSER.DAT`. Необхідний під час об'єднання даних кількох користувачів. |
| `Extension` | Підсписок (`.docx`, `Folder` тощо); null для кореневого списку RecentDocs. |
| `MRU-Position` | Нуль — найновіший елемент у цьому списку. Це відносний порядок, а не минулий час. |
| `ValueIndex` | Нумероване значення реєстру, на яке посилається `MRUListEx`. Лише структурна точка переходу. |
| `Filename` | Декодована назва елемента. Повний шлях зіставляйте через LNK, Jump List, MFT або дані програм. |
| `LinkName` | Назва ярлика, закодована в елементі оболонки; часто трапляється в каталозі Recent користувача. |
| `KeyLastWrittenUTC` | Час останньої зміни ключа списку. Стосується списку, а не кожного рядка. |
| `LastOpenedUTC` | Заповнюється лише для позиції MRU 0, де час ключа підтверджує таку інтерпретацію. |
| `ValueDataRaw` | Необроблені байти, якщо декодування не вдалося або докази потребують перевірки. |
| `TransactionLogStatus`, `LogsApplied` | Чи було відтворено журнали реєстру і які саме. |
| `Problems` | Відсутні значення MRU, значення поза списком, неправильні типи або пошкодження під час декодування. |

## Стратегія розслідування { #investigation-strategy }

1. Розділяйте кореневий список і списки розширень кожного користувача; обережно усувайте дублікати того самого елемента.
2. Використовуйте позицію MRU для впорядкування, а для тверджень із часовою позначкою — лише `LastOpenedUTC`.
3. Зіставляйте `LinkName`/`Filename` з LNK у Recent, Jump List, `$MFT` і даними USN.
4. Перевіряйте назви-приманки, подвійні розширення, мережеві/знімні цілі та документи, близькі в часі до початкового доступу.
5. Якщо важливе виконання, зіставляйте з Prefetch/SRUM програм і журналами подій.

## Додаткові відомості { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [RECmd і Registry Explorer від Eric Zimmerman](https://github.com/EricZimmerman/RECmd)
- [Плакат SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/)
