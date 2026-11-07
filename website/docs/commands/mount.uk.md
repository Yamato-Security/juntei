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

## Аналіз кожного образу за допомогою auto-parse { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` аналізує системи Windows усередині кожного образу диска VHDX у каталозі доказів (або в
одному образі, указаному через `-d`) без окремого монтування:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- Кожен образ монтується лише для читання в новому тимчасовому каталозі, у папці з назвою образу
  (`host1-triage`), а знайдені там системи аналізуються так само, як у збірці на диску. Образи
  відмонтовуються наприкінці роботи, а також після Ctrl+C.
- Термінал перелічує образи та образ кожної системи:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- У маніфесті `EvidenceDirectory` — це вказаний каталог, а поле `Image` кожної системи називає її образ.
  `EvidenceRoot` і команди парсерів використовують папку монтування (`./host1-triage/C`).
- Якщо образ містить файли Volume Shadow Copies у папках на кшталт `VSS32`, кожна папка виявляється й аналізується як інша система (`HOST1_2`, …).
- Про образ, який не вдається змонтувати, буде повідомлено; решта однаково аналізуються, а запуск завершується
  з кодом виходу 1.

Якщо том NTFS не відкривається, Juntei повідомляє про проблему та зберігає доступ до читабельних томів. Для частково читабельного образу `auto-parse --vhdx` повертає код завершення 1; `mount` також повертає 1 після відмонтування.

## Вимоги { #requirements }

- Linux: FUSE 3 (`fusermount3`, пакет `fuse3`) і доступ до `/dev/fuse`, який зазвичай мають звичайні
  користувачі. Права root не потрібні.
- Windows і macOS не підтримуються. У Windows натомість підключіть образ лише для читання в PowerShell,
  запущеному з підвищеними правами, і передайте інструментам його літеру диска:

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
