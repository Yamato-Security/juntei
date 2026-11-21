# Кеш растрових зображень RDP { #rdp-bitmap-cache }

## Що записує кеш растрових зображень RDP { #what-the-rdp-bitmap-cache-records }

Коли клієнт віддаленого робочого стола (`mstsc.exe`) підключається з увімкненим постійним кешуванням растрових
зображень, що є типовим налаштуванням, він зберігає на диск фрагменти віддаленого екрана, щоб пізнішим сеансам не
доводилося отримувати їх знову. Кожен фрагмент — це плитка (tile) розміром щонайбільше 64 × 64 пікселі. Разом плитки
складають невеликі уривки того, що користувач бачив на віддалених комп'ютерах: вікон, тексту, імен файлів, команд і
робочих столів.

| Клієнт | Файли кешу | Плитки |
|---|---|---|
| Windows 7 і новіші | `Cache0000.bin`, `Cache0001.bin`, …, до 100 МБ кожен | 32-бітні, одна за одною |
| Windows XP, Server 2003 і старіші | `bcache2.bmc` (8 бітів на піксель), `bcache22.bmc` (16), `bcache23.bmc` (24) і `bcache24.bmc` (32), до 10 МБ кожен | По одній на кожен слот 64 × 64; деякі стиснені |

Windows 10 і новіші також залишають порожній `bcache24.bmc` поруч із файлами `.bin`.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> користувач, у профілі якого міститься кеш,
підключався за допомогою клієнта віддаленого робочого стола до комп'ютера, який показував кешований вміст. Кеш не
повідомляє, який це був комп'ютер і коли.

## Розташування та збирання { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

Значення `BitmapPersistCacheLocation` у `HKCU\Software\Microsoft\Terminal Server Client` може перемістити кеш
в інше місце. Збирайте всю папку `Cache` кожного профілю, зокрема й із тіньових копій тому: старіша копія файлу кешу
може містити плитки, яких у поточному файлі вже немає.

## Застереження для аналітика { #analyst-cautions }

- Плитки не мають часу, а кеш не називає комп'ютерів, з яких вони надійшли. Пов'язуйте їх із підключеннями RDP за
  іншими доказами: ключами реєстру `Terminal Server Client` клієнта (`parse-registry remote-access`), `Default.rdp`
  та іншими файлами `.rdp`, Jump Lists для `mstsc.exe`, часом файлів кешу в `$MFT` і подіями входу на віддалених
  комп'ютерах.
- Файл кешу змішує плитки багатьох сеансів і кількох віддалених комп'ютерів. Коли файл заповнюється, клієнт замінює
  старі плитки, тож плитці може бути кілька днів або навіть місяців.
- Плитки — це фрагменти екранів, а не цілі екрани. Плитки, що були поруч на екрані, часто розташовані поруч і у
  файлі, але не завжди. Відновлюйте екрани за допомогою
  [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher) або на око за колажами.
- Більшість плиток мають розмір 64 × 64 пікселі. Плитки біля правого й нижнього країв екрана вужчі або нижчі залежно
  від роздільної здатності: наприклад, 58 або 62 пікселі завширшки чи 28 або 35 пікселів заввишки. Juntei читає
  кожну плитку з її власним розміром. Інструменти, які читають кожну плитку так, ніби вона має 64 пікселі завширшки,
  показують вузькі плитки перекошеними.
- Більшість одноколірних плиток (`SingleColor`) — це тло. Відфільтруйте їх, щоб знайти вміст.
- У файлах `.bmc` 8-бітні плитки містять кольори з палітри сеансу, яку кеш не зберігає. Juntei використовує типову
  палітру Windows, тому їхні кольори можуть бути неправильними. Стиснені плитки використовують перемежоване RLE
  (interleaved RLE) протоколу Remote Desktop (MS-RDPBCGR 3.1.9), яке Juntei розпаковує так, як його визначає
  специфікація.
- Плитка `.bmc`, менша за свій слот 64 × 64, залишає після себе кінець попередньої плитки. Juntei зберігає ці
  залишкові рядки як окреме зображення (`RemnantImage`). Вони належать старішій плитці, а не плитці в слоті.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```

```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```

З `-d` Juntei читає файли, імена яких складаються з `Cache`, цифр і `.bin`, а також з `bcache`, цифр і `.bmc`.
Файл `Cache????.bin`, який не починається із сигнатури `RDP8bmp`, належить іншій програмі: його враховують у
підсумку й пропускають. З `-f` Juntei читає файл незалежно від його імені.

### Зображення { #images }

`--extract` створює папку для кожного файлу кешу: `<DIR>/<user>/<cache file>/`. Якщо два файли кешу мали б потрапити
в одну папку, як-от файл кешу та його копія з тіньової копії тому, друга папка отримує номер (`Cache0001.bin_2`).
Папка містить:

- `<cache file>_<index>.bmp`: кожна плитка з її власним розміром. Імена такі самі, як у bmc-tools, тож
  RdpCacheStitcher може відкрити цю папку.
- `<cache file>_old_<index>.bmp`: лише для `.bmc` — кінець попередньої плитки, що залишився після меншої.
- `<cache file>_collage.png`: усі плитки в порядку файлу, по 64 у рядку (`--collage-width`), кожна у клітинці
  64 × 64, починаючи з її верхнього лівого кута. Плитка з індексом *i* розташована в рядку *i* / 64 і стовпці
  *i* % 64, рахуючи від 0. Пурпуровий колір (magenta) заповнює порожні частини клітинок: поруч із вузькою плиткою,
  під низькою та там, де плитку не вдалося прочитати.

`--collage-only` зберігає лише колажі. Наявне зображення не замінюється без `-C`, і про це повідомляє `Problems`
відповідної плитки. Самі файли кешу ніколи не замінюються, навіть з `-C`.

## Поля виводу CSV { #csv-output-fields }

| Поле | Значення |
|---|---|
| `SourceFile` | Файл кешу. |
| `User` | Користувач, у папці профілю якого розташований файл кешу (визначається за шляхом). |
| `TileIndex` | Номер плитки у файлі, рахуючи від 0, як в імені її зображення. |
| `Width`, `Height` | Розмір плитки в пікселях. |
| `SingleColor` | `true`, якщо всі пікселі плитки мають однаковий колір. |
| `TileImage` | З `--extract`: BMP-файл плитки. |
| `Collage` | З `--extract`: колаж файлу кешу. |
| `RemnantImage` | `.bmc` з `--extract`: кінець попередньої плитки, що залишився в слоті. |
| `Key` | 64-бітний ключ плитки в протоколі Remote Desktop, у шістнадцятковому вигляді. |
| `Format`, `FormatVersion` | `bin` з версією, що йде після сигнатури (6 у Windows 10 і новіших), або `bmc`. |
| `BitsPerPixel` | 32 для `.bin`; 8, 16, 24 або 32 для `.bmc`. |
| `Compressed`, `Flags` | `.bmc`: чи була плитка стиснена, і прапорці її заголовка. |
| `Offset` | Де у файлі починається заголовок плитки. |
| `Problems` | Пошкодження: файл, що закінчується посеред плитки, заголовок плитки, якого не могла б мати жодна плитка, дані, що не відповідають розміру плитки, стиснені дані, які неможливо розпакувати, зображення, яке не вдалося зберегти. |

JSON і JSONL мають ті самі поля, але числа та логічні значення в них типізовані, значення, яких плитка не має,
дорівнюють `null`, а `Problems` подано як список. Пошкодження поза межами окремої плитки, як-от дані після останньої
плитки, наводяться в повідомленнях запуску.

## Автоматичний аналіз { #automatic-parsing }

`auto-parse` записує `LateralMovement/RdpCache.csv` (`RdpCache.json` або `RdpCache.jsonl` з `-t json` чи
`-t jsonl`) разом із `LateralMovement/RdpCache-logs.txt` і зберігає колаж кожного файлу кешу в папці
`LateralMovement/RdpCache/`. Додайте `--extract-rdp-tiles`, щоб також зберегти кожну плитку; для одного користувача
це можуть бути десятки тисяч файлів. З образів дисків файли кешу копіюються. Об'єднана хронологія `-j` містить кожен
рядок з ідентифікатором `Artifact` `rdp-cache`. Плитки не мають часу, тому
[`super-timeline`](../commands/super-timeline.md) не створює з них подій.

## Стратегія розслідування { #investigation-strategy }

1. Відкрийте колажі й шукайте текст: імена комп'ютерів і користувачів, шляхи до файлів, команди, вікна консолі,
   адресні рядки браузерів і діалогові вікна.
2. Відновіть екрани за допомогою RdpCacheStitcher з папки плиток, збережених з `--extract`.
3. Знайдіть комп'ютери, до яких підключався користувач, за реєстром клієнта (`parse-registry
   remote-access`), файлами `.rdp` і Jump Lists для `mstsc.exe`. На цих комп'ютерах шукайте відповідні входи через
   RDP і записи журналювання доступу користувачів (UAL).
4. Використовуйте часи `$MFT` файлів кешу та їхніх копій у тіньових копіях тому, щоб звузити проміжок часу, коли
   було записано плитки.

## Додаткові відомості { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: кеш растрових зображень RDP (французькою)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — RDP cache plugin](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — persistent bitmap cache](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
