# Дані конфігурації завантаження (BCD) { #boot-configuration-data-bcd }

## Що таке сховище BCD { #what-the-bcd-store-is }

Windows Vista і новіші версії зберігають конфігурацію завантаження в кущі реєстру — сховищі BCD:
`\Boot\BCD` у системах BIOS, `\EFI\Microsoft\Boot\BCD` на системному розділі EFI у системах
UEFI. Його редагують `bcdedit` та інсталятор Windows; диспетчер завантаження читає його під час кожного завантаження.

Сховище містить об'єкти, кожен з яких названо за GUID:

- **диспетчер завантаження Windows** (`{bootmgr}`) із типовим записом, порядком відображення та
  тайм-аутом меню; у системах UEFI також **диспетчер завантаження мікропрограми** (`{fwbootmgr}`) із
  порядком завантаження мікропрограми;
- по одному **завантажувачу Windows** для кожної інсталяції Windows (включно із середовищем відновлення),
  **програма відновлення роботи** (гібернація) і **засіб перевірки пам'яті**;
- об'єкти **успадкованих параметрів** (`{globalsettings}`, `{bootloadersettings}`, …), параметри яких
  успадковують інші об'єкти;
- об'єкти **пристроїв** (параметри ramdisk середовища відновлення).

Кожен об'єкт має елементи: опис, пристрій і шлях, а також параметри, як-от
`recoveryenabled`, `bootstatuspolicy`, `testsigning`, `nointegritychecks` і `safeboot`.

Їх змінюють атаки двох видів:

- **Програми-вимагачі** вимикають відновлення, щоб систему не можна було відремонтувати:
  `bcdedit /set {default} recoveryenabled No` і
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`. Деякі сімейства встановлюють `safeboot`, щоб
  Windows перезапустилася в безпечному режимі, де засоби захисту не працюють.
- **Руткіти та непідписані драйвери** можуть використовувати `testsigning on` або `nointegritychecks on`, щоб послабити перевірки під час завантаження. Самі ці параметри не доводять, що руткіт запускався.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> на момент збирання цей запис завантаження
мав цей параметр. Час останнього запису ключа елемента — це час, коли параметр було записано востаннє.

## Розташування { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

Тип елемента — це 8 шістнадцяткових цифр: його клас (`1` — усі програми, `2` — один вид
програм, `3` — пристрої), його формат (`1` — пристрій, `2` — рядок, `3` — об'єкт, `4` — список об'єктів, `5`
— ціле число, `6` — логічне значення, `7` — список цілих чисел) і його номер. Команда називає елементи так,
як їх документує Microsoft (`BcdLibraryBoolean_AutoRecoveryEnabled`).

Інструменти збирання часто не копіюють сховище: системний розділ EFI не змонтовано, а `\Boot`
приховано. Зберіть його явно або змонтуйте розділ. `-d` також читає `BCD-Template`
(`HiveType` вказує, що саме); шаблон — це те, з чого Windows починає, а не сховище, з яким вона завантажується.

## Застереження для аналітика { #analyst-cautions }

- **Параметри успадковуються.** Завантажувач успадковує параметри від об'єктів у своєму
  `BcdLibraryObjectList_InheritedObjects` (зазвичай `{bootloadersettings}`, який успадковує
  `{globalsettings}`). `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks`, `SafeBoot` і
  `BootStatusPolicy` — це розв'язані збережені значення (власні, інакше успадковані), а `Inherited` вказує, які з них
  надходять з іншого об'єкта. Успадкування відстежується щонайбільше на вісім рівнів, цикли обмежено. Об'єкт, на який є посилання, може бути відсутнім у частковому сховищі. Порожні поля не встановлюють фактичного типового значення Windows. Мікропрограма та Secure Boot також можуть перешкодити набуттю чинності збереженого параметра.
- **Не встановлено — не означає вимкнено.** Порожнє `RecoveryEnabled` означає, що елемента немає; застосовується
  типове значення Windows.
- **Час — для кожного елемента окремо.** Час ключа кожного елемента (`key_last_written` в `Elements`) — це час, коли його
  було записано востаннє. Windows перезаписує деякі елементи під час оновлень і переходу на новіші версії; порівнюйте з часом інших
  елементів.
- **Пристрої декодуються частково.** Розділ GPT подається як GUID його розділу та диска;
  ramdisk — як його об'єкт параметрів. Для інших пристроїв зберігаються лише байти (`data_raw`).
- **Недокументовані елементи** (`0x15000065`, клас `0x4…`) не мають назви, і для них зберігаються байти.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

Зведення перелічує типовий запис кожного сховища та кожен параметр тестового підписування, перевірки цілісності, відновлення,
політики стану завантаження (будь-яке `Ignore…`) і безпечного режиму разом із часом його встановлення або об'єктом,
від якого його успадковано.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Хост, кущ і `BCD` або `BCD-Template`. |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | GUID об'єкта, його тип (`0x10200003`) і що це таке (`Windows boot loader`, `inherited settings for Windows boot loader`, …). |
| `Description`, `Path`, `Device` | Опис, шлях програми та її пристрій (розділ GPT і диск або об'єкт параметрів ramdisk). |
| `IsDefault`, `DisplayOrder` | Чи завантажує його диспетчер завантаження Windows за замовчуванням, і його місце в меню завантаження. |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | `recoveryenabled`, `testsigning`, `nointegritychecks` у тому вигляді, як їх розв'язано зі сховища. |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`, `Network`, `DsRepair`) і `bootstatuspolicy` (`IgnoreAllFailures`, …) разом із числом. |
| `Inherited` | Які з цих значень надходять з іншого об'єкта і з якого саме. |
| `Elements` | Кожен елемент: його тип, назва, значення, байти (для пристроїв, недокументованих типів і даних, які не декодуються) і час ключа. |
| `KeyLastWrittenUTC` | Останній запис ключа об'єкта. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Об'єкти без типу, елементи, дані яких не відповідають їхньому формату. |

## Стратегія розслідування { #investigation-strategy }

1. Перевірте типовий запис і кожен завантажувач на `TestSigning`, `NoIntegrityChecks`,
   `RecoveryEnabled = false`, `BootStatusPolicy` зі значенням `Ignore…` і `SafeBoot`.
2. Використовуйте час елементів, щоб датувати зміну, і зіставляйте з `bcdedit.exe` у Prefetch,
   Amcache, подіях створення процесів (4688, Sysmon 1) і журналі подій System.
3. Шукайте записи завантаження або пристрої, яких не повинно бути (інший шлях завантажувача, ramdisk), а
   також змінений типовий запис або порядок відображення.
4. Порівняйте сховище з `BCD-Template` та з іншою системою тієї самої збірки.

## Додаткові відомості { #further-details }

- [Microsoft: довідник параметрів завантаження BCD](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (типи об'єктів)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
