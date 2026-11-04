# Historial de aplicaciones { #application-history }

## Qué dejan las herramientas { #what-tools-leave-behind }

Muchas herramientas de terceros guardan su configuración y su historial en el `NTUSER.DAT` del usuario. Varias
figuran entre las herramientas que utilizan los atacantes, o que emplean para preparar datos antes de llevárselos: herramientas de Sysinternals, compresores,
clientes de sincronización en la nube y herramientas de acceso remoto. Juntei lee las siguientes:

| Aplicación | Clave bajo `HKCU\Software` | Qué muestra |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | Cada herramienta cuya licencia aceptó el usuario (`EulaAccepted`): PsExec, ProcDump, Sysmon, … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | Archivos comprimidos abiertos; nombres de archivos comprimidos y carpetas de extracción escritos en sus cuadros de diálogo |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | Archivos comprimidos creados; carpetas de extracción; carpetas examinadas y de destino de copias en su administrador de archivos |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | Archivos comprimidos de su menú Archivo, carpetas de extracción, archivos comprimidos recientes |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | Dirección de correo electrónico, carpeta local y nombres de cada cuenta; bibliotecas sincronizadas (`ScopeIdToMountPointPathCache`) y carpetas (`Tenants\<tenant>`) |
| Adobe Acrobat y Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | Últimos archivos PDF y carpetas abiertos, con nombres de archivo, tamaños y fechas |
| TeamViewer | `TeamViewer` (y `Version<N>` en versiones antiguas) | Todos los valores: nombres de cuenta y de usuario, carpetas iniciales de transferencia de archivos (`<partner ID>?<local>|<remote>`), … |
| CCleaner | `Piriform\CCleaner` | Todos los valores: qué está configurado para limpiar, `UpdateKey`, … |

<span class="evidence-strength">Mejor conclusión:</span> este usuario ejecutó la herramienta (o aceptó su
licencia), o la herramienta abrió, creó o extrajo estos archivos comprimidos y carpetas, o esta cuenta en la nube
se configuró para el usuario. La mayoría de los historiales no tienen una hora por entrada: solo la entrada más reciente recibe
la hora de última escritura de la clave de su lista.

## Ubicación { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

Recopile el `NTUSER.DAT` de cada usuario junto con sus registros de transacciones, y el `SOFTWARE` del sistema para obtener
los SID de los usuarios.

## Precauciones para el analista { #analyst-cautions }

- **Aceptación del EULA de Sysinternals.** `FirstUsedUTC` es una estimación del primer uso basada en la hora de última escritura
  de la clave cuando su único valor es un `EulaAccepted` legible establecido en `1`. Un EULA rechazado o
  dañado no recibe esta estimación. La configuración del Registro también puede preconfigurarse,
  copiarse o editarse: corrobore la ejecución y su hora con otros artefactos.
  Las herramientas que guardan allí su configuración (Process Explorer, Autoruns, …) reescriben la clave más tarde:
  en ese caso `FirstUsedUTC` está vacío y `KeyLastWrittenUTC` es solo el último cambio de configuración. Una herramienta
  ejecutada con `-accepteula`, o como otro usuario (el lado remoto de PsExec se ejecuta como SYSTEM), escribe la
  clave de ese usuario.
- **Posiciones, no horas.** Las entradas de las listas están ordenadas (`MruPosition` 0 es la más reciente), pero
  solo la última escritura de la clave de la lista fecha la más reciente. Las entradas anteriores se usaron antes, en un
  momento desconocido.
- **Nombres escritos.** El `ArcName` de WinRAR es el nombre del archivo comprimido escrito al crear uno: a menudo, un nombre
  de archivo sin carpeta.
- **Las rutas de Adobe** se conservan tal como Adobe las escribe (`/C/Users/alice/a.pdf` es
  `C:\Users\alice\a.pdf`). Su `sDate` (en `Details`) es una cadena de fecha PDF, conservada tal como se escribió.
- **El `UpdateKey` de CCleaner** es una hora local sin zona; se conserva tal como se escribió.
- **TeamViewer.** Los registros de conexión son archivos (`Connections_incoming.txt`, `TeamViewer*_Logfile.log`),
  no valores del Registro: recopílelos también.
- **Ausencia.** Los usuarios y las herramientas pueden borrar estos historiales (el propio CCleaner puede hacerlo).

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

El resumen enumera las estimaciones de primer uso de Sysinternals más recientes, los archivos comprimidos y carpetas que cada lista usó
en último lugar y las cuentas de OneDrive.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath` | Host y `NTUSER.DAT`. |
| `User`, `UserSID` | Nombre de la carpeta del perfil y su SID según el `ProfileList` del sistema. |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`. |
| `Item` | La herramienta de Sysinternals, la lista (`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …), la cuenta de OneDrive o la clave de sus carpetas sincronizadas, o el nombre del valor. |
| `MRU-Position` | Posición de la entrada en su lista; 0 es la más reciente. |
| `Path` | El archivo comprimido, la carpeta o el documento. |
| `Value` | `EulaAccepted`, la dirección de correo electrónico de una cuenta de OneDrive o el identificador de una biblioteca sincronizada, un valor de TeamViewer o de CCleaner. |
| `FirstUsedUTC` | Estimación del primer uso a partir de la aceptación del EULA (consulte las precauciones anteriores). |
| `Details` | `DisplayName`, `UserCID`, `ConfiguredTenantId`, … de una cuenta de OneDrive; `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate` de un documento de Adobe. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | De dónde procede el registro y cuándo cambió esa clave por última vez. |
| `DataRaw` | Bytes del valor (en hexadecimal) cuando no se pueden decodificar. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Datos que no se pudieron decodificar ni leer. |

## Estrategia de investigación { #investigation-strategy }

1. Busque herramientas de Sysinternals que un usuario corriente no ejecuta (PsExec, ProcDump, PsLoggedOn,
   SDelete) y sus marcas de tiempo del EULA.
2. Revise los nombres de archivos comprimidos y las carpetas de extracción en torno al incidente: archivos comprimidos creados en
   `ProgramData`, `Temp` o la carpeta de un usuario antes de que los datos salieran del sistema.
3. Compruebe las cuentas de OneDrive que no pertenecen a la organización, y los nombres de cuenta y las carpetas de transferencia
   de archivos de TeamViewer.
4. Correlacione con Prefetch, Amcache, BAM y UserAssist (la ejecución de la herramienta), la `$MFT` y el Diario
   USN (el archivo comprimido), los archivos LNK y las Jump Lists (documentos abiertos) y los registros de eventos.

## Más información { #further-details }

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
