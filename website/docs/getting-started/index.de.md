# Juntei installieren { #install-juntei }

## Release-Binärdateien { #release-binaries }

Laden Sie die ZIP-Datei für Ihr System von der
[Release-Seite von Juntei](https://github.com/Yamato-Security/juntei/releases) herunter. Falls noch keine Release-Binärdatei
veröffentlicht ist, kompilieren Sie aus dem Quellcode wie unten beschrieben. Bewahren Sie das Archiv oder den Quell-Commit sowie die
Prüfsumme bei Ihren Fallnotizen auf, damit andere Analysten den Lauf reproduzieren können.

| System | ZIP-Datei |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` oder `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` oder `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple Silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

Jede ZIP-Datei enthält nur die Binärdatei, benannt nach Version und Plattform (zum Beispiel
`juntei-1.0.0-win-x64.exe`): Juntei benötigt keine weiteren Dateien, und die Windows-Binärdatei benötigt nicht das
Visual C++ Redistributable. Die musl-Linux-Binärdateien sind statisch gelinkt und laufen auf jeder
Distribution; die gnu-Varianten benötigen eine aktuelle glibc. Erlauben Sie unter macOS die heruntergeladene Binärdatei unter
Systemeinstellungen > Datenschutz & Sicherheit, oder entfernen Sie ihr Quarantäneattribut mit
`xattr -d com.apple.quarantine juntei-<version>-mac-*`.

```bash
juntei --version
juntei --help
```

Die Beispiele verwenden `juntei` für die Binärdatei; benennen Sie sie um oder geben Sie ihren vollständigen Namen ein.

## Aus dem Quellcode kompilieren { #compile-from-source }

Juntei erfordert derzeit die in `Cargo.toml` angegebene Rust-Version (Rust 1.99 zum Zeitpunkt, als diese
Seite geschrieben wurde).

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` verwendet die exakten Abhängigkeitsrevisionen, die in `Cargo.lock` festgehalten sind.

## Grundlegender Aufbau eines Befehls { #basic-command-shape }

Jeder Parser akzeptiert entweder eine einzelne Datei (`-f`) oder ein Sicherungsverzeichnis (`-d`):

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

Lassen Sie sich die aktuellen Optionen anzeigen, statt einen alten Befehl aus einem Bericht zu kopieren:

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

Fahren Sie mit [Erste Untersuchung](first-investigation.md) fort, um einen sicheren Arbeitsablauf kennenzulernen.
