# Монтування образів дисків { #mounting-disk-images }

`juntei mount` надає доступ до файлів образу диска VHDX через каталог лише для читання протягом своєї роботи. Hayabusa, Juntei та інші інструменти можуть аналізувати їх без розпакування й прав root. Функція використовує FUSE та працює в Linux. Модуль читання та бібліотека монтування підтримуються в [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

В іншому терміналі:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

Щоб відмонтувати образ, натисніть Ctrl+C у першому терміналі або виконайте `fusermount3 -u /mnt/host1`.

## Що показує монтування { #what-the-mount-shows }

- У точці монтування відображаються файли тому NTFS. Зібраний образ може містити каталоги `C` і `LongFileNames` та метадані тому, наприклад `$MFT`. Кілька томів NTFS відображаються як `Volume1`, `Volume2` тощо.
- Кожен файл доступний лише для читання й належить власникові точки монтування. Позначки часу файлів — це
  позначки часу NTFS `$STANDARD_INFORMATION`.
- Імена перелічуються так, як їх зберігає NTFS, і шукаються за точним збігом. Імена DOS 8.3 та альтернативні
  потоки даних не показуються.
- Стиснені файли розпаковуються (до 1 ГіБ кожен). Файли, зашифровані EFS, прочитати неможливо.

## Докази не змінюються { #the-evidence-is-not-changed }

Образ відкривається лише для читання, а монтування відхиляє запис. Для некоректно закритого VHDX відкладений журнал застосовується в пам'яті й ніколи не записується назад. Деякі компоненти DiscUtils залишають нульовими інструкцію переходу й сигнатуру `55 AA` у завантажувальному секторі NTFS; Juntei читає такі томи без змін.

## Використання { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Нативний аналіз образів { #parsing-every-image-with-auto-parse }

З `--vhdx` команда `auto-parse` читає VHDX нативно в **Linux, macOS і Windows** через [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser). Системне монтування, FUSE та права адміністратора не потрібні.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Парсерам потрібні шляхи: Juntei копіює доступні NTFS-файли у приватний тимчасовий каталог з обмеженими буферами. **Потрібне місце для всіх цих файлів**, а не лише артефактів із результатами. Нульові області записуються як розріджені, якщо це підтримується. Зберігаються імена, структура, жорсткі посилання та час зміни. Часи джерел LNK і Jump Lists беруться зі знімка оригінальних метаданих NTFS, ніколи з тимчасових копій. Нормальне завершення, навіть з помилками парсерів, видаляє копії; примусове завершення може залишити їх. Джерело лише для читання; незастосований журнал VHDX відтворюється тільки в пам’яті.

Пошук хостів, CSV/JSON/JSONL і `-j` використовують звичайний конвеєр. `Image` у `manifest.json` позначає оригінальний образ, `EvidenceRoot` — внутрішній відносний шлях. Помилки читання й небезпечні імена записуються в `ImageErrors` та рядки помилок `manifest.csv`. Читабельні результати зберігаються; неповний ввід повертає код 1. Небезпечні чи неоднозначні імена не перейменовуються. Іменовані потоки NTFS не видобуваються як окремі файли; артефакти, зібрані як звичайні файли, доступні.

Окрема [команда `mount`](mount.md) надає файлову систему іншим програмам і досі потребує Linux FUSE. `auto-parse --vhdx` її не використовує.

## Вимоги до монтування, видимого ОС { #os-visible-mount-requirements }

`juntei mount` потребує Linux FUSE 3 (`fusermount3`) і доступу до `/dev/fuse`. У Windows і macOS використовуйте нативний аналіз вище.
