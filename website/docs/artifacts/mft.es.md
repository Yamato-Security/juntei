# Tabla maestra de archivos (`$MFT`) { #master-file-table-mft }

## Qué es la MFT { #what-the-mft-is }

NTFS representa cada archivo y directorio con un registro en la tabla maestra de archivos (Master File Table). Los registros contienen
atributos como nombres, referencias al directorio principal, marcas de tiempo, tamaños, indicadores, identificadores de seguridad,
información de puntos de reanálisis y metadatos de los flujos de datos. El contenido de los archivos pequeños y los flujos de datos alternativos pueden
ser **residentes**, es decir, sus bytes se almacenan dentro del propio registro de la MFT.

La MFT es fundamental porque abarca los registros asignados y eliminados de todo el volumen.
Puede mostrar que un archivo existió, dónde estaba vinculado, si se eliminó y cómo cambiaron sus metadatos,
incluso cuando la exploración normal de directorios ya no lo muestra.

<span class="evidence-strength">Mejor conclusión:</span> un registro de archivo NTFS con los metadatos
mostrados existía en el momento de la recopilación o sobrevivió en un registro no asignado. La MFT por sí sola no
demuestra la ejecución ni la intención del usuario.

## Ubicación y recopilación { #location-and-collection }

```text
C:\$MFT
```

`$MFT` son metadatos protegidos de NTFS. Recopílela con una herramienta de adquisición forense o de triaje capaz
de acceder a NTFS sin procesar (raw); una copia normal en modo de usuario no suele poder leerla. Combínela con `$J`, `$LogFile`
y los metadatos del volumen siempre que sea posible.

## Precauciones para el analista { #analyst-cautions }

- Un registro eliminado puede estar parcialmente sobrescrito o su número de registro puede haberse reutilizado.
- `SN` distingue las generaciones de un mismo número de entrada. Haga coincidir la entrada **y** la secuencia.
- Las marcas de tiempo de `$STANDARD_INFORMATION` (`0x10`) y `$FILE_NAME` (`0x30`) se actualizan de forma
  diferente. Ninguna es inmune a la manipulación.
- `SI<FN`, `ZeroU-Secs` y `Copied` son pistas, no veredictos automáticos de timestomping.
- Los vínculos físicos generan varios atributos `$FILE_NAME` y varias rutas para un mismo registro de archivo.
- NTFS puede almacenar varios nombres (incluido el DOS 8.3), flujos de datos alternativos y registros de extensión.
- Las actualizaciones de la hora de acceso pueden estar deshabilitadas, retrasarse o agruparse.
- La ruta completa se reconstruye a partir de los registros de los directorios principales. `[Orphaned]`, `[Unknown]` o `PathUnknown`
  indican una ascendencia ausente, reutilizada o dañada, no un nombre de directorio literal.
- El **espacio residual** (slack) de un registro FILE (sus bytes más allá del tamaño en uso) puede conservar nombres que contuvo antes: las
  entradas de índice de un directorio pequeño cuyo índice se trasladó a una asignación de índice (la mayoría siguen
  en uso allí), un nombre anterior o el ocupante previo de un registro reutilizado. Los nombres recuperados
  no tienen `$STANDARD_INFORMATION`, y el encabezado de entrada de un nombre puede haberse sobrescrito. Cuando el
  tamaño en uso o los atributos de un registro no indican dónde empieza su espacio residual, no se recupera nada de él
  y el campo `ParseErrors` de su fila indica `slack not read` y el motivo.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

No ejecute nunca los flujos residentes extraídos con `-e`; Juntei les asigna nombres `.dontrun` por un motivo.
Las columnas de datos residentes de `-i` contienen los mismos bytes: trátelas como contenido que no es de confianza.

`-D` imprime una entrada: su encabezado, todos los atributos (con las series de datos) y los nombres de su espacio residual tal como
los escribe `-t json -x`, y después los bytes del registro. `-L` enumera todos los nombres de un directorio, en uso y
eliminados, con su número de entrada y de secuencia, su tipo, su tamaño y su hora de modificación. Un directorio se indica por
entrada (`30`, `0x1E`, `30-1`) o por ruta (`\Windows\System32`, `C:/Windows`; sin distinguir mayúsculas de minúsculas).

## Campos de salida CSV { #csv-output-fields }

| Campo | Interpretación |
|---|---|
| `EntryNum` | Número de registro de la MFT. Combínelo con `SN`; los números de registro se reutilizan. |
| `SN` | Contador de generaciones utilizado por las referencias de NTFS. Una discrepancia puede revelar relaciones obsoletas o eliminadas. |
| `InUse` | Indicador de asignación en el momento de la recopilación. `false` significa que el registro está libre o eliminado, no que todos sus bytes estén intactos. |
| `ParentEntryNum`, `ParentSN` | Referencia al directorio principal procedente de `$FILE_NAME`. Valide ambas partes. |
| `ParentPath` | Ruta reconstruida del directorio principal. Los marcadores de desconocido o huérfano ponen de manifiesto una ascendencia sin resolver. |
| `File` | Nombre procedente del `$FILE_NAME` o del flujo de la fila actual. Un registro puede tener varias filas. |
| `Ext` | Extensión obtenida del nombre por comodidad; no es una determinación fiable del tipo de archivo. |
| `Size` | Tamaño lógico asociado a la fila o al flujo. La asignación dispersa o comprimida puede diferir. |
| `RefCnt` | Recuento de referencias de vínculos físicos de NTFS; compárelo con cautela con el número de nombres que sobreviven. |
| `ReparseTarget` | Destino decodificado de los puntos de reanálisis compatibles, como uniones o vínculos simbólicos. Las rutas pueden redirigir. |
| `Dir` | El registro tiene el indicador de directorio. |
| `HasADS` | El registro contiene flujos de datos alternativos con nombre. Esto no lo convierte en malicioso. |
| `IsADS` | Esta fila representa un flujo de datos alternativo con nombre y no los datos sin nombre del archivo. |
| `SI<FN` | La creación de SI es anterior a una hora de creación de FN distinta. Investigue la copia, el timestomping y las causas legítimas. |
| `ZeroU-Secs` | La hora de creación o modificación de SI cae exactamente en un límite de milisegundo. Solo es una heurística débil de timestomping. |
| `Copied` | La hora de modificación es anterior a la hora de creación. Suele ser coherente con una copia de archivo; no demuestra la acción de un atacante. |
| `SI-Flags` | Atributos e indicadores de archivo de `$STANDARD_INFORMATION`. Los indicadores de oculto o de sistema pueden ser pivotes útiles. |
| `Type` | Espacio de nombres de `$FILE_NAME` (Win32, DOS, POSIX o combinado). Utilícelo para no contar dos veces los nombres. |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | Horas de SI. Campos habituales de la línea de tiempo, pero las herramientas en modo de usuario pueden modificar varios de ellos. |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | Horas de FN. Se actualizan de forma distinta y, en general, son más difíciles de alterar con las API normales; aun así, no son inmutables. |
| `USN` | Último USN asociado al atributo SI del registro. Pivote hacia `$J` si ese USN sobrevive. |
| `LogfileSN` | Número de secuencia de `$LogFile` de NTFS procedente del encabezado del registro FILE; útil en el análisis de transacciones a bajo nivel. |
| `SID` | Índice en `$Secure` de NTFS; no es un SID de Windows por sí mismo. Combínelo con el `SecurityID` de [`parse-ntfs secure-sds`](secure-sds.md) para obtener el propietario y las ACL del archivo. |
| `ObjectID-FileDroid` | Identificador de objeto o de seguimiento de vínculos distribuidos, si existe; correlaciónelo con los droids de LNK y Jump List. |
| `LoggedUtilStream` | Contexto decodificado de `$LOGGED_UTILITY_STREAM`, si es compatible. Revise los valores desconocidos. |
| `ZoneID-Contents` | Contenido residente de `Zone.Identifier`, como ZoneId y las URL de descarga. Puede estar ausente, haberse eliminado o no ser residente. |
| `SourceFile` | La `$MFT` de entrada, esencial al combinar hosts o volúmenes. |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | Con `-i`: el contenido del `$DATA` residente de la fila (el del archivo o el del flujo de datos alternativo), en Base64, como bytes (`41-42`) y como texto cuando es ASCII imprimible. Vacío cuando los datos no son residentes o se han filtrado con `-E`/`-M`. |
| `ParseErrors` | Daños o relaciones no compatibles que afectan a esta fila. No descarte nunca esta columna al filtrar. |
| `FromSlack` | Con `-x`: `True` para un nombre recuperado del espacio residual de un registro FILE. Su `EntryNum` y su `SN` son los del archivo al que apuntaba el nombre (vacíos cuando se sobrescribió el encabezado de su entrada), sus marcas de tiempo recuperadas están en las columnas `(FN)`, e `InUse` es `False`. |
| `SlackRecord`, `SlackOffset` | Con `-x`: el registro FILE en cuyo espacio residual estaba el nombre, según su posición en la `$MFT` y no según el número de registro escrito en su interior (el propio de un registro de extensión, aunque sus nombres se enumeren con su registro base), y el desplazamiento en bytes del nombre en la `$MFT`. Los `ParseErrors` de una fila del espacio residual son los problemas de ese registro, como `fixup values do not match`: los bytes recuperados pueden estar afectados. |
| `CopyOfEntryInUse` | Con `-x`: un archivo en uso sigue teniendo este nombre en el mismo directorio (y es el mismo archivo, cuando la referencia ha sobrevivido). Los nombres que no tiene ningún archivo en uso son los interesantes: eliminados, renombrados o el ocupante previo de un registro reutilizado. |

Las columnas de marcas de tiempo siguen el orden MACB (Modified, Accessed, Changed, Birth/creación):
`DataModified` SI/FN, `Accessed` SI/FN, `MetadataChanged` SI/FN y después `Created` SI/FN.

`(SI)` significa `$STANDARD_INFORMATION`; `(FN)` significa `$FILE_NAME`. Estos nombres describen el
origen habitual en un registro completo. En una fila de nombre de archivo normal sin atributo SI, el
mecanismo de reserva existente coloca la creación de FN en `Created (FN)` y los valores de modificación, cambio de registro y acceso
de FN en `DataModified (SI)`, `MetadataChanged (SI)` y `Accessed (SI)`, respectivamente.
Las filas recuperadas del espacio residual colocan las cuatro horas en las columnas `(FN)` y dejan vacías las columnas SI.
Los valores de reserva de las filas normales **no** son evidencia SI independiente. Utilice los atributos JSON originales
y la procedencia del espacio residual para determinar el origen de la marca de tiempo.

`-t ys-ntfs-parser-csv` (antes `ys-mft-parser-csv`, que se sigue aceptando) utiliza el esquema orientado a entradas
del analizador mantenido. Estos nombres de columna legibles se aplican a la salida predeterminada `-t csv`,
incluido el `MFT.csv` de `auto-parse`. JSON/JSONL conserva todos los
atributos (incluido el `$DATA` residente, en hexadecimal) y `FullPath`; utilícelo cuando el CSV
de investigación plano omita la estructura de bajo nivel. Con `-x`, cada entrada JSON también contiene
`SlackFileNames`.

## Estrategia de investigación { #investigation-strategy }

1. Filtre una ventana temporal, pero conserve los registros eliminados y ambos conjuntos de marcas de tiempo.
2. Busque rutas completas, dobles extensiones sospechosas, ADS, directorios de preparación y hashes o nombres conocidos.
   Con `-x`, incluya los nombres del espacio residual que no tiene ningún archivo en uso (`CopyOfEntryInUse` `False`); con
   `-i -E ps1,bat,vbs,js`, lea los scripts pequeños directamente desde el CSV.
3. Pivote la entrada y la secuencia y el `USN` hacia el Diario USN.
4. Compare las referencias MFT de Prefetch, LNK y Jump List con el registro de destino.
5. Correlacione la eliminación con la cuarentena de Defender o con la actividad `FILE_DELETE` o de cambio de nombre del USN.

## Más información { #further-details }

- [Microsoft: tabla maestra de archivos](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Crate mft de ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [Analizador mft de Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [Documentación del formato de libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
