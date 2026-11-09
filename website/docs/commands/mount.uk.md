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

`auto-parse --vhdx` читає VHDX нативно в **Linux, macOS і Windows**, без системного монтування, FUSE або прав адміністратора.

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Парсерам потрібні файлові шляхи: Juntei копіює файли, доступні через NTFS-читач, у приватний тимчасовий каталог. **Забезпечте достатньо тимчасового дискового простору для цих файлів.** Джерельний образ залишається лише для читання; за нормального завершення копії видаляються.

Помилки читання та небезпечні імена записуються в `ImageErrors` і рядки помилок `manifest.csv`. Читабельні результати зберігаються; неповні вхідні дані дають код завершення 1.

Окрема команда `mount`, що надає файли іншим програмам, і далі потребує Linux, FUSE 3 та доступу до `/dev/fuse`.

[English: implementation details and limitations](https://yamato-security.github.io/juntei/commands/auto-parse/#vhdx-disk-images).
