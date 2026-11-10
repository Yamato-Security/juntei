# Встановлення Juntei { #install-juntei }

## Двійкові файли випусків { #release-binaries }

Завантажте zip-архів для своєї системи зі
[сторінки випусків Juntei](https://github.com/Yamato-Security/juntei/releases). Якщо двійковий файл випуску
ще не опубліковано, скомпілюйте програму з вихідного коду, як описано нижче. Зберігайте архів або коміт вихідного коду та
контрольну суму разом із нотатками у справі, щоб інший аналітик міг відтворити запуск.

| Система | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` або `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` або `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

Кожен zip-архів містить лише двійковий файл, назва якого включає версію та платформу (наприклад,
`juntei-1.0.0-win-x64.exe`): Juntei не потребує жодних інших файлів, а двійковому файлу для Windows не потрібен
Visual C++ Redistributable. Двійкові файли musl для Linux зібрано статично, і вони працюють у будь-якому
дистрибутиві; файлам gnu потрібна нещодавня версія glibc. У macOS дозвольте завантажений двійковий файл у
System Settings > Privacy & Security або видаліть його атрибут карантину командою
`xattr -d com.apple.quarantine juntei-<version>-mac-*`.

```bash
juntei --version
juntei --help
```

У прикладах двійковий файл називається `juntei`; перейменуйте його або вводьте його повну назву.

## Компіляція з вихідного коду { #compile-from-source }

Наразі Juntei потребує версії Rust, зазначеної в `Cargo.toml` (Rust 1.99 на момент написання цієї
сторінки).

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` використовує точні версії залежностей, записані в `Cargo.lock`.

## Базова форма команди { #basic-command-shape }

Кожен парсер приймає або один файл (`-f`), або каталог зібраних даних (`-d`):

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

Виводьте актуальні параметри замість того, щоб копіювати стару команду зі звіту:

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

Перейдіть до розділу [Перше розслідування](first-investigation.md), щоб ознайомитися з безпечним робочим процесом.
