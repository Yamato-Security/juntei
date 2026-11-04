# Подяки щодо Persistence Hunter { #persistence-hunter-acknowledgements }

Команда `persistence-hunter` у Juntei спирається на два покоління напрацювань спільноти з пошуку механізмів закріплення.
Дякуємо авторам і учасникам, які зробили свої методики, реалізації та дослідження
доступними для слідчих.

## PyrsistenceSniper: проєкт на Python { #pyrsistencesniper-the-python-project }

[**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) від
[Hexastrike](https://github.com/Hexastrike) — безпосередній еталон для цієї функції. Juntei переносить
його каталог із 130 перевірок на нативний Rust разом із типовим списком дозволених елементів, знімком LOLBAS і
поведінкою класифікації. Еталонну версію зафіксовано на
[коміті `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f).

Проєкт на Python поширюється за ліцензією MIT. Juntei зберігає його повідомлення про авторські права та ліцензію в
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt).
Реалізація Juntei працює офлайн без інтерпретатора Python. Обробку зібраних даних,
звітування про покриття та консервативну інтерпретацію неповних доказів задокументовано в
[посібнику з команди](../commands/persistence-hunter.md).

## PersistenceSniper: оригінальний інструмент на PowerShell { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) — оригінальний проєкт на PowerShell
для пошуку механізмів закріплення, який надихнув PyrsistenceSniper. PyrsistenceSniper зазначає авторство
Федеріко Лаграсти; оригінальний репозиторій також зазначає Ріккардо Анкарані та інших
учасників. Дякуємо оригінальному проєкту та його учасникам за те, що вони спростили дослідження механізмів закріплення
Windows.

Безпосереднім еталоном для перенесення в Juntei є проєкт на Python. Оригінальний інструмент на PowerShell згадано
як частину цієї історії походження. Juntei також незалежно реалізує механізми, запропоновані
[hongson11698](https://github.com/hongson11698) в оригінальному проєкті:
[PR #31 щодо розв'язання COM запланованих завдань](https://github.com/last-byte/PersistenceSniper/pull/31)
і [PR #32 щодо бібліотек меню «Пуск»/папок CLSID](https://github.com/last-byte/PersistenceSniper/pull/32).
Вони доповнюють еталонний каталог Python; це не означає, що кожну функцію для працюючих систем з будь-якого з цих першоджерельних проєктів
реалізовано в офлайн-команді Juntei.

## Допоміжні дослідження та інструменти { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/) надає ідентифікатори технік, що містяться в
  каталозі. Зіставлення з технікою описує можливе використання механізму; відповідне значення реєстру
  або файл саме по собі не є атакою, підтвердженою ATT&CK.
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  також визнано в PyrsistenceSniper джерелом натхнення для дослідження автозапуску Windows.
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) документує легітимні двійкові файли Windows, якими можна
  зловживати. Розпізнавання одного з цих файлів допомагає інтерпретувати команду, але не робить цю
  команду автоматично шкідливою.

Про основи парсерів реєстру, NTFS, Shell Link, журналів подій та інших, що використовуються в усьому Juntei,
див. [подяки щодо всього проєкту](../acknowledgements.md).

Повернутися до [посібника з механізмів закріплення](index.md).
