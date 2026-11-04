# Політика аудиту { #audit-policy }

## Що це таке { #what-it-is }

Політика аудиту визначає, які події безпеки Windows записує до журналу подій Security. У Windows Vista і новіших
вона задається для кожної підкатегорії (59 у Windows 10 і 11), а у Windows XP і Server 2003 — для кожної категорії.
Кожен параметр записує успіхи, збої, і те, і інше або нічого.

LSA зберігає чинну політику в кущі `SECURITY` незалежно від того, чи її задано через `auditpol`, локальну
політику безпеки або групову політику.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> на момент останнього запису куща система була налаштована записувати (або
не записувати) ці події. Якщо аудит підкатегорії не ввімкнено, відсутність події нічого не говорить
про те, чи відбулася відповідна активність.

## Розташування { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista і новіші зберігають 12-байтовий заголовок, потім по одному параметру на підкатегорію, категорія за
категорією, а потім кількість підкатегорій у кожній категорії. Версії відрізняються лише цими лічильниками:
кожна додає підкатегорії в кінець категорії (56 у прочитаних кущах Windows 8.x і Server 2012 R2,
59 у Windows 10, 11 і Server 2022). Windows XP і Server 2003 зберігають ознаку того, чи ввімкнено аудит
(один байт, далі заповнення, яке Windows не очищає), і дев'ять параметрів категорій. Цю структуру
перевірено на публічному образі Windows XP із CFReDS Hacking Case від NIST, у якому аудит нічого не охоплює;
значення параметрів його категорій відповідають документації Microsoft.

Збирайте `SECURITY` разом із журналами транзакцій, а також системний кущ `SYSTEM`, який містить ім'я
хоста.

## Застереження для аналітика { #analyst-cautions }

- **Невідомі дані залишаються невідомими.** Непідтримувані ревізії зберігаються з діагностичними даними. Невідомі
  прапорці параметрів залишають `Success` і `Failure` порожніми та виключаються з підрахунку параметрів аудиту.
  Пошкоджена таблиця лічильників не може перетворити зарезервоване слово на інший параметр аудиту.
  Знімки можуть містити додаткові слоти поза каталогом назв/GUID. Їхні параметри та
  числовий `SubcategoryId` зберігаються з порожніми назвами/GUID; неспівставлений слот сам по собі не є
  доказом пошкодження.


- **Порядок визначає сама Windows.** У межах категорії параметри йдуть у порядку ідентифікаторів повідомлень
  підкатегорій, а не їхніх GUID. Special Logon (`%%12548`, індекс 4 у Logon/Logoff)
  іде перед IPsec Quick Mode. Інструменти, які читають їх у порядку GUID, повідомляють, що за замовчуванням у Windows 10
  Special Logon має значення «No Auditing», а IPsec Quick Mode — «Success». `SubcategoryId` — це
  той самий ідентифікатор, який наводить подія 4719 (System audit policy was changed), тож їх можна зіставити.
- **Чинна політика, а не її джерело.** Групова політика та локальна політика записують однакове значення.
  `audit.csv` групової політики (у SYSVOL і `C:\Windows\System32\GroupPolicy`) показує, що
  передбачалося застосувати (`Machine\Microsoft\Windows NT\Audit\audit.csv` політики).
- **Зміни з часом:** `KeyLastWrittenUTC` приблизно вказує, коли політика востаннє змінювалася. Подія 4719
  записує кожну зміну (доки журнал Security її зберігає). Зловмисник може вимкнути аудит,
  а потім знову його ввімкнути.
- **Підкатегорії та категорії.** У Windows Vista і новіших Windows використовує саме підкатегорії
  (якщо тільки параметр безпеки `SCENoApplyLegacyAuditPolicy` не вимкнено і застаріла політика
  категорій не перевизначає їх).
- **Windows XP і 2003:** якщо аудит вимкнено (`AuditingEnabled` має значення False), жодна категорія не підлягає аудиту
  незалежно від її параметра.
- **Події:** `Events` перелічує основні події кожної підкатегорії згідно з документацією Microsoft
  щодо параметрів розширеної політики аудиту. Це ті події, які зазвичай шукають під час розслідування, а не
  всі події, які записує підкатегорія.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

Для кожного куща підсумок наводить його структуру, кількість параметрів з увімкненим аудитом і час останнього
запису політики. Він перелічує, що підлягає аудиту успіхів і збоїв, лише успіхів і лише
збоїв, а потім підкатегорії без аудиту з їхніми основними подіями: ці події
вимкнено в зібраній політиці; це не встановлює, якою була політика під час попередньої активності.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath` | Хост (із системного куща `SYSTEM`) і кущ `SECURITY`. |
| `Layout` | `subcategories` (Windows Vista і новіші) або `categories` (XP і 2003); порожнє, якщо значення не декодується. |
| `Category`, `Subcategory` | Так, як їх називає `auditpol`. |
| `SubcategoryGUID`, `SubcategoryID` | GUID підкатегорії та її ідентифікатор повідомлення в тому вигляді, як його наводить подія 4719 (`%%12548`). |
| `Setting` | `none`, `success`, `failure` або `success_and_failure`. |
| `Success`, `Failure` | Чи записуються успіхи та збої. |
| `AuditingEnabled` | Windows XP і 2003: чи ввімкнено аудит загалом. |
| `Events` | Ідентифікатори основних подій підкатегорії. |
| `KeyLastWrittenUTC` | Останній запис `PolAdtEv`: приблизний час останньої зміни політики. |
| `DataRaw` | Байти значення (hex), якщо воно не декодується повністю. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Невідома структура або розмір, лічильники, що не сходяться, і значення понад 3. |

## Стратегія розслідування { #investigation-strategy }

1. Перш ніж робити висновок, що щось не відбувалося, перевірте, чи підлягала аудиту відповідна підкатегорія:
   Process Creation (4688), Logon (4624, 4625), Special Logon (4672), Other Object Access Events
   (заплановані завдання, 4698), Security System Extension (встановлення служб, 4697), User Account and
   Security Group Management (4720, 4732), File Share (5140), Kerberos (4768, 4769).
2. Порівняйте політику з журналом Security: події підкатегорій, які зараз не підлягають аудиту,
   означають, що політику змінювали. Шукайте подію 4719 і нещодавній `KeyLastWrittenUTC`.
3. Порівняйте з `audit.csv` групової політики, щоб відрізнити політику на рівні домену від локальної зміни.
4. Використовуйте `parse-registry read --recover-deleted` і журнали транзакцій для отримання попередніх версій
   `PolAdtEv`.

## Додаткові відомості { #further-details }

- [Microsoft: параметри розширеної політики аудиту безпеки](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: константи аудиту (GUID підкатегорій)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
