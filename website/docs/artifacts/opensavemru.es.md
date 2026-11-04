# Cuadros de diálogo Abrir y Guardar como (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## Qué es el historial de los cuadros de diálogo { #what-the-dialog-history-is }

Cuando un programa muestra el cuadro de diálogo común Abrir o Guardar como, Windows registra en el
`NTUSER.DAT` del usuario:

- **qué archivo se abrió o guardó:** `OpenSavePidlMRU` tiene una subclave por extensión (`docx`,
  `ps1`, …) y `*` para todas las extensiones, cada una con una lista de los usados más
  recientemente;
- **qué programa usó el cuadro de diálogo y en qué carpeta estaba:** `LastVisitedPidlMRU` (y
  `LastVisitedPidlMRULegacy` para los programas que usan los cuadros de diálogo antiguos);
- **programas que mostraron el cuadro de diálogo:** `CIDSizeMRU` (el tamaño del cuadro de diálogo
  por programa) y `FirstFolder` (la ruta de un programa y una carpeta);
- en Windows 2000 y XP, `OpenSaveMRU` (rutas por extensión) y `LastVisitedMRU` (programas y sus
  carpetas como rutas).

Desde Windows Vista, los archivos y carpetas son listas de elementos del shell (shell items), las
mismas estructuras que el destino de un acceso directo: cada elemento es una parte de la ruta (Este
equipo, una unidad, una carpeta, el archivo), y un elemento de archivo o carpeta puede llevar sus
propias horas y su referencia de archivo NTFS.

<span class="evidence-strength">Mejor conclusión:</span> un programa usó el cuadro de diálogo Abrir
o Guardar como para abrir o guardar este archivo, o en esta carpeta, en la sesión de este usuario.
Solo la entrada más reciente de cada lista tiene fecha: la hora de última escritura de su clave.

## Ubicación { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

Recopile `NTUSER.DAT` con sus registros de transacciones para cada perfil.

## Precauciones para el analista { #analyst-cautions }

- **Una hora por lista.** La hora de última escritura de una clave cambia cuando una de sus
  entradas pasa al principio de la lista, por lo que solo fecha la entrada de la posición 0
  (`IsMostRecent`, `LastUsedUTC`). Las demás entradas están ordenadas, de la más reciente a la más
  antigua según se desciende por la lista, pero sin fecha.
- **El mismo archivo en dos listas.** Un archivo aparece en la lista de su extensión y en `*`; no lo
  cuente como dos usos. `*` proporciona el orden entre extensiones.
- **Abierto o guardado.** Las listas no indican cuál de los dos. Correlacione con los registros de
  `$MFT` y del Diario USN del archivo, los archivos LNK y las Jump Lists.
- **Las horas del elemento son las del archivo, no las del cuadro de diálogo.** `ItemModifiedUTC`,
  `ItemCreatedUTC` e `ItemAccessedUTC` son las horas del archivo cuando se escribió el elemento,
  con una precisión de 2 segundos (FAT). Los elementos de carpetas conocidas
  (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …) a menudo no tienen ninguna. Estas marcas de
  tiempo FAT no tienen desplazamiento de zona horaria. El sufijo de columna `UTC` es una convención
  de presentación, no una evidencia de que el reloj original estuviera en UTC; correlacione con la
  zona horaria del sistema de origen y con otras marcas de tiempo antes de compararlas con los
  FILETIME del Registro. Consulte las
  [indicaciones de Microsoft sobre las horas de archivo](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times).
- **Las rutas son rutas del shell.** Empiezan en el escritorio: `This PC\C:\Users\…`, una carpeta
  conocida, una biblioteca (`UsersLibraries\Documents\…`) o una ubicación de red. Una ruta que
  empieza por una carpeta o un archivo está en el escritorio. Una ruta vacía es el propio
  escritorio.
- **Elementos no decodificados.** Un elemento de un tipo que el analizador de elementos del shell no
  conoce se muestra mediante su clase (`[class 0x00]`), su tipo es `unknown` y los bytes de la
  lista están en `DataRaw`.
- **Los programas son nombres.** `LastVisitedPidlMRU` y `CIDSizeMRU` guardan un nombre de archivo
  (`WINWORD.EXE`), no una ruta. `FirstFolder` guarda una ruta.
- **Las listas son cortas y se limpian.** Cada lista conserva un número limitado de entradas, y las
  herramientas de limpieza eliminan las claves. Utilice `parse-registry read --recover-deleted`
  para los valores eliminados.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

El resumen enumera las entradas de cada lista, los programas que usaron los cuadros de diálogo y el
archivo más reciente de cada hive (la primera entrada de `*`).

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, el propietario del perfil (a partir de la ruta o de las carpetas del shell), el SID del `ProfileList` del sistema y el hive. |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU` o `LastVisitedMRU`. |
| `Extension` | La subclave de la extensión (`docx`, `*`); vacío en las demás listas. |
| `MRU-Position`, `IsMostRecent` | La posición en la lista de la clave (0 es la más reciente); vacío cuando la lista no menciona el valor. |
| `ValueName` | El número (`MRUListEx`) o la letra (`MRUList`) del valor. |
| `Program` | El programa que mostró el cuadro de diálogo. |
| `Path` | El archivo o la carpeta: los nombres de los elementos del shell unidos, o la ruta almacenada como texto. |
| `ItemType` | El tipo del último elemento del shell: `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | Las horas del archivo o de la carpeta, tal como las registró su elemento del shell. |
| `MFT-Entry`, `MFT-Sequence` | La referencia de archivo NTFS que registró el elemento del shell. |
| `LastUsedUTC` | La hora de última escritura de la clave, solo para la entrada más reciente. |
| `KeyPath`, `KeyLastWrittenUTC` | La clave y su última escritura: la hora de su entrada más reciente, no la de cada fila. |
| `DataRaw` | Los bytes del valor (en hexadecimal) cuando no se decodifican, o la lista de elementos del shell cuando el tipo de un elemento es desconocido. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Valores que la lista menciona pero que la clave no contiene, valores que no menciona, tipos incorrectos, y nombres o listas truncados. |

## Estrategia de investigación { #investigation-strategy }

1. Lea `*` en orden para cada usuario, y `LastUsedUTC` para el archivo más reciente.
2. Busque scripts, archivos comprimidos, ejecutables y documentos en ubicaciones en las que el
   usuario puede escribir, extraíbles o de red, así como archivos con nombres que sugieran
   credenciales o exportaciones.
3. Utilice `LastVisitedPidlMRU` para vincular un programa con la última carpeta que usó, y
   `CIDSizeMRU` y `FirstFolder` para identificar los programas que llegaron a usar los cuadros de
   diálogo.
4. Correlacione las rutas y `MftEntry` con la `$MFT`, el Diario USN, los archivos LNK, las Jump
   Lists, ShellBags y RecentDocs.

## Más información { #further-details }

- [libyal winreg-kb: claves Most recently used (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: formato Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
