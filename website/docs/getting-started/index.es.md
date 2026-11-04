# Instalar Juntei { #install-juntei }

## Binarios de las versiones { #release-binaries }

Descargue el archivo zip correspondiente a su sistema desde la
[página de versiones de Juntei](https://github.com/Yamato-Security/juntei/releases). Si todavía no se ha publicado ningún binario,
compile desde el código fuente como se describe más abajo. Conserve el archivo comprimido o el commit del código fuente y la
suma de comprobación junto con las notas del caso para que otro analista pueda reproducir la ejecución.

| Sistema | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` o `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` o `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

Cada zip contiene únicamente el binario, cuyo nombre incluye la versión y la plataforma (por ejemplo,
`juntei-1.0.0-win-x64.exe`): Juntei no necesita ningún otro archivo, y el binario de Windows no requiere el
Visual C++ Redistributable. Los binarios musl de Linux están enlazados estáticamente y se ejecutan en cualquier
distribución; los gnu necesitan una versión reciente de glibc. En macOS, permita el binario descargado en
Configuración del Sistema > Privacidad y seguridad, o elimine su atributo de cuarentena con
`xattr -d com.apple.quarantine juntei-<version>-mac-*`.

```bash
juntei --version
juntei --help
```

Los ejemplos utilizan `juntei` como nombre del binario; cámbiele el nombre o escriba su nombre completo.

## Compilar desde el código fuente { #compile-from-source }

Actualmente, Juntei requiere la versión de Rust declarada en `Cargo.toml` (Rust 1.99 en el momento de redactar esta
página).

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` utiliza las revisiones exactas de las dependencias registradas en `Cargo.lock`.

## Forma básica de los comandos { #basic-command-shape }

Todos los analizadores aceptan un único archivo (`-f`) o una recopilación en un directorio (`-d`):

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

Muestre las opciones actuales en lugar de copiar un comando antiguo de un informe:

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

Continúe con [Primera investigación](first-investigation.md) para seguir un flujo de trabajo seguro.
