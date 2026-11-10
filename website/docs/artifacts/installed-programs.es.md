# Programas instalados { #installed-programs }

## Qué conserva Windows { #what-windows-keeps }

Windows y los instaladores registran el software instalado en varias listas del Registro. Cada lista tiene una clave
por programa, producto, distribución o paquete; Juntei genera un registro por clave.

| Origen | Hive y clave | Qué contiene |
|---|---|---|
| `uninstall` | `SOFTWARE` y `NTUSER.DAT` `…\CurrentVersion\Uninstall\*`, y `SOFTWARE\Wow6432Node\…` | Lo que muestran Configuración y *Programas y características*: nombre, versión, editor, fecha de instalación, carpetas, comandos de desinstalación, tamaño y si está oculto |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | Productos de Windows Installer: nombre y versión del producto, nombre de archivo del paquete y carpeta desde la que se instaló |
| `app_path` | `SOFTWARE` y `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | Programas que Windows inicia por nombre (`Win+R`, `start tool`): la ruta del programa y su carpeta |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | Distribuciones del Subsistema de Windows para Linux: nombre, ubicación de sus archivos o disco, WSL 1 o 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | Aplicaciones empaquetadas (Store): aprovisionadas para todos los usuarios, propias de Windows, preparadas, desaprovisionadas y las de cada usuario |

<span class="evidence-strength">Mejor conclusión:</span> este programa, producto o paquete se
instaló (o registró) en el sistema, para todos los usuarios o para este usuario. La fecha de instalación es
la que escribió el instalador; la última escritura de la clave indica cuándo cambió la entrada por última vez.

## Ubicación { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

Recopile `SOFTWARE`, todos los `NTUSER.DAT` y sus registros de transacciones. `SYSTEM` proporciona el nombre del host y
el `ProfileList` de `SOFTWARE` proporciona el SID de cada usuario.

## Precauciones para el analista { #analyst-cautions }

- **Fecha de instalación y hora de la clave.** `InstallDate` es una fecha (sin hora) escrita por el instalador, en
  hora local; algunos instaladores no la escriben y otros utilizan otro formato (se conserva tal como está escrito en
  `InstallDateRaw`). Una actualización puede reescribir la entrada sin cambiar la fecha, o cambiar la fecha
  sin que el programa cambie mucho: compare `InstallDate` con `KeyLastWrittenUTC`.
- **Software desinstalado.** Un programa eliminado correctamente pierde sus claves. Utilice `parse-registry read
  --recover-deleted` para las claves `Uninstall` eliminadas, y Amcache, Prefetch y la `$MFT` para los rastros
  que hayan quedado.
- **Registros por usuario.** Las entradas de `NTUSER.DAT` identifican software registrado para un perfil.
  Ese ámbito no establece quién ejecutó el instalador ni si se utilizaron derechos de administrador.
- **Entradas ocultas.** `SystemComponent` oculta una entrada en *Programas y características*; las entradas
  sin `DisplayName` (actualizaciones, componentes) tampoco se muestran. Todas se notifican.
- **Códigos de producto.** Una clave `Uninstall` con forma de GUID puede emparejarse con una clave
  `msi_product` empaquetada mediante `ProductCode`. El nombre con forma de GUID por sí solo no demuestra que
  Windows Installer creara la entrada.
- **App Paths.** Son registros utilizados para localizar programas, no registros de
  ejecución. El valor `Path` puede contener varios directorios de búsqueda y no es necesariamente el
  directorio de instalación.
- **Editor.** Son metadatos del Registro proporcionados por el instalador, no información de firma
  verificada.
- **Arquitectura.** `Wow6432Node` contiene programas de 32 bits en Windows de 64 bits (`Architecture` `x86`,
  `Wow64` `True`); las demás entradas son de la arquitectura propia del sistema (vacío). La
  arquitectura de un paquete es la suya propia (`x64`, `neutral`, …).
- **WSL.** `Version` es la versión de WSL según los `Flags` de la distribución (`0x8`: se ejecuta en la
  máquina virtual de WSL 2). El propio valor `Version` de la distribución (en `Details`) es la versión de su sistema de archivos,
  que también es 2 en las distribuciones de WSL 1. `BasePath` contiene su `ext4.vhdx` (WSL 2) o `rootfs`
  (WSL 1): recopílelo para examinar su contenido.
- **Paquetes.** Las listas `<SID>` son los paquetes registrados para cada usuario; la mayoría son propios
  de Windows. Los paquetes `Deprovisioned` se quitaron para los usuarios nuevos; los de `EndOfLife\<SID>`, para ese usuario.
- **Valores dañados.** Los números con formato incorrecto, los tipos de valor erróneos y el UTF-16 no válido se notifican en
  `Problems` y se conservan en `DataRaw`, incluida una cadena hexadecimal vacía para un valor presente con
  cero bytes. Una cadena vacía limpia en un campo numérico opcional se trata como ausente. La versión de un
  paquete debe contener cuatro componentes decimales, cada uno dentro del rango de 16 bits sin signo.
  Las advertencias a nivel de hive permanecen en stderr cuando `-t` transmite los registros a stdout.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

El resumen enumera los últimos programas instalados (según `InstallDate`), lo que los usuarios instalaron para
sí mismos y las distribuciones de WSL.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, archivo del hive y `SOFTWARE` o `NTUSER.DAT`. |
| `User`, `UserSID` | Usuario de un hive de usuario (su carpeta de perfil y el SID de `ProfileList`) o usuario para el que figura un paquete. |
| `Source`, `Scope` | La lista (`uninstall`, `msi_product`, `app_path`, `wsl`, `package`) y `machine` o `user`. |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher`; `ProductName` y `Version` de un producto; nombre del archivo de programa de un App Path; nombre y versión de WSL de una distribución; nombre y versión de un paquete. |
| `InstallDate`, `InstallDateRaw` | `InstallDate` como fecha cuando está escrito como `YYYYMMDD`; en caso contrario, tal como está escrito. |
| `KeyLastWrittenUTC` | Cuándo cambió por última vez la clave de la entrada. |
| `InstallLocation`, `InstallSource` | `InstallLocation` (el `Path` de un App Path, el `BasePath` de una distribución, el `Path` de un paquete); `InstallSource` (la última carpeta de origen utilizada por un producto). |
| `Program` | El programa de un App Path. |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`. |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | `EstimatedSize` en KiB; si está oculto; si lo instaló Windows Installer. |
| `ProductCode` | El código de producto de Windows Installer (`{GUID}`). |
| `Architecture`, `WOW64` | `x86` en `Wow6432Node`, o la arquitectura de un paquete; si la clave se encuentra en `Wow6432Node`. |
| `Details` | Más información: `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User` (la cuenta que ejecutó un instalador de Inno Setup); `PackageCode` y `PackageName` de un producto; `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName` y `Default` de una distribución; `List`, `ResourceId` y `PublisherId` de un paquete. |
| `KeyPath` | La clave de la entrada. |
| `DataRaw` | Los bytes (hexadecimal) de los valores que no son lo que indican sus nombres. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Valores que no se pudieron leer o que no son lo que indican sus nombres. |

## Estrategia de investigación { #investigation-strategy }

1. Ordene por `InstallDate` y `KeyLastWrittenUTC` en torno al incidente; fíjese en las herramientas de acceso
   remoto, los clientes de tunelización y transferencia de archivos, los compresores, los entornos de ejecución de scripts y las herramientas de seguridad.
2. Revise los registros con `Scope` `user`: software instalado sin derechos de administrador.
3. Revise `InstallLocation`, `InstallSource` y `UninstallCommand` en busca de carpetas con permisos de escritura para el usuario
   (`Downloads`, `AppData`, `Temp`, `ProgramData`).
4. Busque distribuciones de WSL y recopile su `BasePath`.
5. Correlacione con Amcache y Prefetch (el instalador y el programa en ejecución), la `$MFT` (la creación
   de la carpeta de instalación) y el registro de eventos Application (eventos de MsiInstaller).

## Más información { #further-details }

- [Microsoft: clave del Registro Uninstall](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: registro de aplicaciones (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: información general sobre la identidad de paquete](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
