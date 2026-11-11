# Papelera de reciclaje { #recycle-bin }

## Qué registra la Papelera de reciclaje { #what-the-recycle-bin-records }

Cuando un usuario elimina un archivo o una carpeta en el Explorador de archivos, Windows lo mueve a la Papelera de
reciclaje de su volumen y escribe junto a él un pequeño registro de índice: la ruta que tenía, su tamaño y cuándo se
eliminó. Cada usuario tiene una carpeta de papelera en cada volumen, con el SID del usuario como nombre, por lo que la
carpeta también indica qué cuenta eliminó el elemento.

| Versión de Windows | Índice | Contenido eliminado |
|---|---|---|
| Vista y posteriores | Un archivo `$I…` por elemento (versión de formato 1 de Vista a 8.1, versión 2 en Windows 10 y posteriores) | `$R…`, con el mismo nombre después de `$R`: el archivo, o la carpeta con su contenido |
| NT 4.0, 2000, XP, Server 2003 | Un archivo `INFO2` por papelera: registros de 800 bytes con una ruta ANSI y otra Unicode | `D<drive letter><index><extension>`, como `Dc1.txt` |
| 95, 98, ME | Un archivo `INFO2` (`INFO` en Windows 95): registros de 280 bytes con solo una ruta ANSI | Los mismos nombres `D…` |

<span class="evidence-strength">Mejor conclusión:</span> la cuenta en cuya papelera está el registro movió el
elemento de la ruta original a la Papelera de reciclaje en la hora registrada. Eliminar con Mayús+Supr, desde la
línea de comandos o desde la mayoría de los programas omite la papelera y no deja ningún registro.

## Ubicaciones y recopilación { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

Recopile los archivos `$I` o el INFO2 de la papelera de cada volumen. Si también se recopilan los archivos `$R` y
`D…`, se ve qué elementos eliminados siguen ahí; sin ellos, `RecycledFileFound` es `false` para todos los elementos.
El contenido eliminado es una evidencia por sí mismo y puede ser malware: trátelo como tal.

## Precauciones para el analista { #analyst-cautions }

- `DeletedTime` es cuándo se movió el elemento a la papelera, según el reloj del sistema, como FILETIME en UTC. No es
  cuándo se vació la papelera. Las horas de creación y modificación propias del elemento están en `$MFT` y en el
  archivo `$R`.
- El SID corresponde a la cuenta que eliminó el elemento, no necesariamente el propietario del archivo. Un SID sin perfil en el
  `ProfileList` del hive SOFTWARE (una cuenta eliminada o una cuenta de dominio que nunca inició sesión) no tiene
  `User`.
- Cuando se vacía la papelera, o cuando un elemento se restaura o se elimina de la papelera, Windows Vista y versiones
  posteriores eliminan sus archivos `$I` y `$R`. Sus nombres pueden permanecer en el Diario USN, en `$MFT` y en el
  espacio residual (slack) de `$I30`; Juntei no recupera de ellos registros `$I` eliminados. INFO2 conserva esos
  registros y pone a cero el primer byte de su ruta ANSI: `RemovedFromBin` es `true`, y Juntei restituye ese carácter
  a partir del número de unidad.
- `FileSize` en un archivo `$I` es el tamaño del elemento; en una carpeta, el tamaño total de su contenido. En INFO2
  es el espacio que ocupaba el elemento en el disco, en clústeres completos (4096 bytes para un archivo pequeño).
- La ruta ANSI de INFO2 suele estar en formato 8.3 y requiere la página de códigos del sistema: utilice
  `--code-page 932` para Windows en japonés, 936 o 950 para Windows en chino. La ruta Unicode, cuando existe, está en
  `OriginalPath`.
- Algunos sistemas Windows Vista escribieron archivos `$I` de 543 bytes cuyo campo de tamaño tiene un byte de menos.
  Juntei lee su hora y su ruta y deja `FileSize` vacío y notifica un problema.
- `RecycledFileFound` describe la recopilación, no el equipo investigado: `false` puede significar que el recopilador
  no tomó los archivos `$R` o que el elemento ya no está en la papelera.
- Las papeleras de volúmenes no pertenecientes al sistema solo usan perfiles del mismo equipo recopilado, identificado por los metadatos o una estructura conocida de carpetas de unidades/volúmenes. Si la atribución es ambigua, el usuario permanece desconocido; no se toman perfiles de otro host.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

Con `-d`, Juntei lee los archivos cuyo nombre es `$I` seguido de seis letras o dígitos (con cualquier extensión),
`INFO2` e `INFO` dentro de una carpeta `RECYCLED`. Con `-f`, lee el archivo sea cual sea su nombre.

## Campos de salida CSV { #csv-output-fields }

| Campo | Significado |
|---|---|
| `SourceFile` | El archivo `$I` o INFO2. |
| `DeletedTime` | Cuándo se movió el elemento a la papelera (UTC). |
| `OriginalPath` | Dónde estaba el elemento. En INFO2, la ruta Unicode si el registro la tiene; si no, la ruta ANSI. |
| `FileSize` | `$I`: el tamaño del elemento en bytes (el total, en una carpeta). INFO2: el espacio que ocupa en el disco, en clústeres completos. Vacío cuando no se puede leer. |
| `User`, `SID` | El SID de la carpeta de la papelera y el usuario que da nombre a su perfil en `ProfileList`. |
| `RecycledFile`, `RecycledFileFound` | La ruta del contenido eliminado junto al índice (`$R…`, `D<drive><index><extension>`) y si está en la recopilación. |
| `Format`, `FormatVersion` | `$I` 1 (Vista a 8.1) o 2 (Windows 10 y posteriores); `INFO2` 0 (95), 2 (NT 4.0), 4 (98) o 5 (ME, 2000 a Server 2003). |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2: el número del registro (el número de su nombre de archivo `D…`), la unidad y dónde empieza el registro dentro de INFO2. Una ruta UNC no tiene unidad. |
| `RemovedFromBin` | INFO2: `true` cuando el elemento se restauró o se eliminó de la papelera. |
| `ShortPath` | INFO2: la ruta ANSI, cuando difiere de `OriginalPath` (a menudo nombres 8.3). |
| `Problems` | Daños: un archivo truncado o demasiado largo, un número de unidad no válido, una ruta que no es UTF-16 válido, una hora de eliminación que no es una hora válida. |

JSON y JSONL tienen los mismos campos, con los números y los booleanos tipados, `null` para los valores que un
registro no tiene y `Problems` como lista.

Si no se puede listar el directorio o el nombre del contenido es ambiguo, `RecycledFileFound` es desconocido (celda CSV vacía o JSON `null`); la causa figura en `Problems`. La salida no puede sobrescribir contenido reciclado, ni mediante enlaces físicos, ni escribirse dentro de una carpeta reciclada. Si no se puede leer ningún índice, se conservan los resultados existentes.

## Análisis automático { #automatic-parsing }

`auto-parse` escribe `FileExistence/RecycleBin.csv` (`RecycleBin.json` o `RecycleBin.jsonl` con
`-t json` o `-t jsonl`), junto con `FileExistence/RecycleBin-logs.txt`. Su opción `--code-page` se aplica a
INFO2. Desde imágenes de disco se copian los archivos `$I` e INFO2; los archivos `$R` y `D…` son marcadores vacíos,
de modo que `RecycledFileFound` se sigue rellenando. La línea de tiempo combinada `-j` incluye todas las filas, con el
identificador de `Artifact` `recycle-bin`; `-Z` convierte `DeletedTime`, y
[`super-timeline`](../commands/super-timeline.md) genera un evento `Recycle Bin deletion (moved to the bin)`
por elemento.

## Estrategia de investigación { #investigation-strategy }

1. Ordene por `DeletedTime` para cada usuario y compare con las horas de la actividad que investiga.
2. Busque `OriginalPath` en LNK, Jump Lists, ShellBags y RecentDocs para ver si el elemento se usó antes de
   eliminarse.
3. En el Diario USN, la eliminación aparece como un cambio de nombre hacia `$Recycle.Bin` en torno a `DeletedTime`;
   `$MFT` proporciona las horas propias del elemento.
4. Cuando `RecycledFileFound` es `true`, calcule el hash del contenido `$R` o `D…` y examínelo.
5. Si se vació la papelera, busque nombres `$I` y `$R` en el Diario USN y en `$MFT`.

## Más información { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
