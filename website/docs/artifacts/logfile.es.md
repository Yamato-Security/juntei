# Registro de transacciones NTFS (`$LogFile`) { #ntfs-transaction-log-logfile }

## Qué es y por qué es importante { #what-it-is-and-why-it-matters }

NTFS es el sistema de archivos que Windows utiliza habitualmente para su volumen del sistema. Para recuperarse de un bloqueo,
registra los cambios que pretende realizar en los metadatos del sistema de archivos en un archivo oculto del volumen llamado
`$LogFile`. Cada actualización puede contener bytes de **rehacer** (redo, el nuevo estado) y bytes de **deshacer** (undo, el estado
anterior necesario para revertir un cambio). Es independiente de los registros de eventos de Windows, de los archivos `.LOG1`
del Registro y del [Diario USN](usn-journal.md).

Para un investigador, el estado anterior resulta especialmente útil. Un registro de deshacer puede conservar el nombre de un archivo antes de un cambio de nombre,
una entrada de directorio eliminada o las marcas de tiempo anteriores a una actualización de metadatos.
Algunos registros también contienen bytes que NTFS escribió en el Diario USN, incluidos registros que ya no están en el
`$J` recopilado. Sin embargo, muchas transacciones corresponden a la actividad rutinaria del sistema operativo y no
describen un ataque.

**Ejemplo:** una entrada de índice eliminada menciona `report.txt`; una entrada añadida posteriormente menciona
`report.txt.encrypted`. Emparejar la referencia de archivo completa, el elemento principal, la cadena de transacciones y otros
artefactos puede respaldar un cambio de nombre. Esos dos nombres por sí solos no identifican el ransomware, a su operador
ni la hora exacta del cifrado.

## Ubicación y recopilación { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

Cada volumen NTFS tiene su propio registro. Adquiéralo con una herramienta que lea los metadatos de NTFS o que lo exporte
desde una imagen forense o una instantánea; es posible que una copia de archivos normal de Windows no pueda acceder a él. Recopile la
`$MFT` y `$Extend\$UsnJrnl:$J` del mismo volumen lo más cerca posible en el tiempo. Conserve los archivos
originales y sus hashes. Una adquisición posterior puede contener un tramo de actividad diferente.

`$LogFile` es un **registro circular**: los registros nuevos sobrescriben los antiguos. La retención depende del tamaño del registro
y de la carga de trabajo, y puede ser de solo minutos u horas. La ausencia de un nombre de archivo o de una operación no es
evidencia de que nunca ocurriera.

## Qué demuestra un registro y precauciones { #what-a-record-proves-and-cautions }

- Una actualización registra los metadatos que NTFS anotó para la recuperación. Una sola entrada de rehacer no demuestra que
  se completara toda una acción de nivel superior ni que la iniciara una persona concreta. Correlacione
  los registros relacionados, la finalización de la transacción, el sistema de archivos resultante y evidencias independientes.
- **No existe una marca de tiempo general de la transacción.** Las marcas de tiempo incrustadas describen metadatos de archivos,
  que a veces están desactualizados, copiados o modificados por el atacante. No sitúe `CreatedOn` en una cronología del incidente
  como la hora de un cambio de nombre o de un cambio de marca de tiempo. Los registros USN pueden proporcionar una marca de tiempo independiente.
- Los LSN (números de secuencia del registro) ordenan los registros **dentro del registro de un volumen**. No son horas de reloj
  y no deben compararse entre hosts ni volúmenes. Los registros de pasadas anteriores del búfer circular
  pueden sobrevivir junto a la actividad más reciente.
- Las ranuras de la tabla de transacciones se reutilizan. Agrupe con `TransactionFirstLsn`, `TransactionId`, la identidad
  del cliente y el origen, en lugar de solo con `TransactionId`. Las cadenas pueden apuntar fuera de los datos conservados.
- Las referencias de archivo contienen un número de entrada **y un número de secuencia**. De lo contrario, la reutilización de una entrada de la MFT puede
  hacer que archivos no relacionados parezcan el mismo archivo.
- Las rutas proporcionadas por `$MFT` describen el **estado de la MFT recopilada**, no una ruta histórica
  reconstruida. Juntei empareja los números de secuencia y deja explícitos los componentes desconocidos. Un elemento principal podría
  haber cambiado de nombre después de la transacción, y los vínculos físicos pueden ofrecer más de un nombre actual.
- `TargetInferred` significa que una asignación de atributo abierto procede de un tramo diferente del registro.
  Trate esa asignación con especial cuidado. Las entradas de índice no resueltas pueden decodificarse solo por su
  estructura; valide sus nombres con los bytes sin procesar y otras fuentes.
- Una página rota se omite; los registros legibles que la rodean siguen disponibles. `Truncated` marca un
  registro cuya continuación falta. Revise los diagnósticos del analizador antes de considerar completa la salida.
  Una entrada de longitud cero es un error; un registro no escrito, todo `FF`, legítimamente no tiene filas.
- El analizador **no reproduce** las transacciones para reconstruir una MFT histórica. No
  recupera contenidos arbitrarios de archivos no residentes ni ejecuta ningún dato recopilado. Las páginas protegidas con CRC de DAX
  no se admiten. La validación con evidencias reales cubre principalmente páginas de registro de 4 KiB.

## Análisis con Juntei { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

Consulte la [referencia de comandos](../commands/filesystem.md#parse-ntfs-logfile) para ver todas las opciones. La detección
en directorios acepta `$LogFile`, `LogFile`, sus formas `.bin` y nombres como `C_$LogFile` o
`C_$LogFile.bin`, sin distinguir mayúsculas de minúsculas. Un único archivo pasado con `-f` puede tener cualquier nombre.

El emparejamiento automático con la MFT respeta el prefijo de volumen de una exportación: `C_$LogFile` utiliza `C_$MFT`, y
`D_$LogFile.bin` prueba primero `D_$MFT.bin` y después `D_$MFT`. Los registros con prefijo nunca utilizan un `$MFT` sin prefijo
como alternativa. `LogFile` sin prefijo también acepta `MFT` o `$MFT`. Utilice `-m` con `-f` para seleccionar
explícitamente una MFT coincidente cuando su exportación siga otro esquema de nombres.

La salida es un registro por LSN, incluidos los registros de actualización, de punto de control y de cliente desconocido. JSON es un
flujo de objetos con formato legible; JSONL tiene un objeto por línea. CSV tiene los mismos nombres de campo;
las matrices y los campos estructurados son JSON compacto dentro de una celda CSV. Los escalares desconocidos están vacíos en
CSV y son `null` en JSON. Las matrices vacías significan que no se decodificó ningún elemento, no prueban que no existiera ninguno.

Se mantiene en memoria un registro completo y sus registros analizados a la vez. La resolución de las rutas de la MFT añade
memoria y trabajo. `--raw-data` puede aumentar mucho el tamaño de la salida. En `auto-parse`, utilice `-T` para
limitar los procesos de análisis simultáneos en un equipo con memoria limitada.

Si no se puede leer una MFT adyacente, Juntei muestra una advertencia y exporta el registro sin rutas; `MftSource`
permanece entonces nulo. Este fallo de enriquecimiento no descarta las evidencias principales legibles.
Los emparejamientos con la MFT ausentes o ambiguos también generan una advertencia y un recuento `Logs without a matching MFT`
en el resumen; no cuentan como fallos de análisis.

## Campos de salida { #output-fields }

### Identidad, integridad y procedencia del registro { #record-identity-integrity-and-provenance }

| Campo | Significado e interpretación |
|---|---|
| `SourceFile`, `FileOffset` | Ruta de entrada y desplazamiento en bytes donde se leyó el encabezado de este registro; utilícelos para validar los bytes. |
| `LSN` | Número de secuencia del registro. Los valores grandes son enteros, no marcas de tiempo. |
| `PreviousLSN`, `UndoNextLSN` | Vínculos a los registros anterior y de reversión. Cero significa que no hay vínculo; los vínculos distintos de cero pueden apuntar fuera de los datos conservados. |
| `TransactionID`, `TransactionFirstLSN` | Ranura de la tabla de transacciones y LSN más antiguo vinculado a esta transacción; la ranura por sí sola no es única. |
| `RecordType` | `ClientRecord` (actualización), `ClientRestart` (punto de control) u `Other`. |
| `ClientIndex`, `ClientSequence` | Identidad del cliente del registro; normalmente, NTFS. No son una cuenta de usuario ni un identificador de proceso. |
| `Flags`, `SpansPages` | Indicadores sin procesar del registro y si los datos del cliente continúan en otra página. Abarcar varias páginas es normal. |
| `Truncated` | Verdadero si el analizador detectó que falta una continuación para este LSN. No resume todos los daños de todo el archivo. |
| `ClusterSize`, `ClusterSizeSource` | Bytes por clúster utilizados para los desplazamientos de destino; el origen es `Caller`, `Checkpoint` o `Inferred`. Proporcione el valor real con `--cluster-size` si lo conoce. |
| `LogVersion` | Versión de la última página de reinicio legible, normalmente `1.1` o `2.0`; nula si no sobrevive ninguna página de reinicio legible. |
| `MFT-Source` | La MFT utilizada para enriquecer las rutas actuales, si existe. Debe pertenecer al mismo volumen. |

En JSON/JSONL de `auto-parse`, se añaden `Comp` y `Artifact` como en los demás analizadores. El registro `.txt`
de la tarea contiene los diagnósticos; los manifiestos indican si la tarea se completó correctamente, falló o se omitió.
Los daños recuperables pueden producir una tarea correcta con un recuento `Parse problems` distinto de cero en su registro.

### Operaciones y destino { #operations-and-target }

| Campo | Significado e interpretación |
|---|---|
| `RedoOperation`, `UndoOperation` | Nombres de operación de NTFS, o hexadecimal para un código desconocido. Rehacer describe la actualización prevista; deshacer describe su inversa. |
| `TargetEntryNumber`, `TargetSequenceNumber` | Para una operación sobre un registro de archivo: el número de registro de la MFT afectado, sin inferir ningún número de secuencia. En otro caso: la referencia del archivo propietario del atributo abierto resuelto. Una actualización de índice de directorio en línea tiene como destino el directorio, no el elemento secundario descrito por su entrada. |
| `TargetAttributeType`, `TargetAttributeName` | Tipo numérico y nombre del atributo abierto utilizado por la operación. En las operaciones sobre registros de archivo, normalmente es el `$DATA` de `$MFT`. Tipos habituales: 16 (`$STANDARD_INFORMATION`), 48 (`$FILE_NAME`), 128 (`$DATA`), 144 (`$INDEX_ROOT`), 160 (`$INDEX_ALLOCATION`), 176 (`$BITMAP`). |
| `TargetPath` | Ruta actual en la MFT cuando se conoce la referencia completa del destino, o componentes desconocidos. Vacío en las operaciones sobre registros de archivo cuya generación histórica se desconoce; un número de registro por sí solo no debe resolverse a su ocupante actual. |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | Archivo propietario del atributo abierto a través del cual escribe la operación, con su ruta actual en la MFT. En las operaciones sobre registros de archivo, conservan el contexto de `$MFT` por separado del registro afectado. |
| `TargetAttribute` | Desplazamiento en la tabla de atributos abiertos del registro; es distinto del tipo del atributo o de su identificador en la MFT. |
| `TargetInferred` | Verdadero si la asignación del destino procede de otro tramo del registro. Un destino nulo sigue sin resolverse aunque este indicador sea falso. |
| `TargetVCN`, `Lcns` | Clúster virtual en el destino y números de clúster físicos; útiles para la validación de bajo nivel más que para la cronología. |
| `RecordOffset`, `AttributeOffset` | Desplazamientos del atributo o la entrada y de la actualización dentro de él. |
| `ClusterBlockOffset`, `TargetBlockSize` | Desplazamiento y tamaño del destino en unidades de 512 bytes. |
| `OffsetInTarget` | Desplazamiento en bytes calculado en el registro de archivo o el flujo no residente afectados. Depende de los tamaños de clúster y de registro de la MFT. |
| `RedoLength`, `UndoLength` | Longitudes de datos declaradas. Una operación de puesta a cero puede declarar una longitud sin llevar esa cantidad de bytes de carga útil. |
| `RedoBytesAvailable`, `UndoBytesAvailable` | Bytes de carga útil realmente disponibles para la decodificación. Compárelos con las longitudes declaradas y la semántica de la operación. |

### Columnas prácticas de metadatos de archivos { #convenient-file-metadata-columns }

Estas columnas muestran un nombre de archivo preferido que no es DOS del lado de rehacer y, si no existe, del de deshacer.
Si ninguno de los lados contiene un nombre de archivo, los campos de actualización de marcas de tiempo dan preferencia a las marcas de tiempo de rehacer y después a las de deshacer.
Las actualizaciones de entradas de directorio también exponen directamente sus horas, su tamaño y sus indicadores, con preferencia de rehacer sobre
deshacer. Los atributos `$DATA` decodificados exponen su tamaño aunque no haya ningún nombre de archivo. El tamaño del `$DATA` sin nombre
de un registro de archivo decodificado tiene prioridad sobre el tamaño del nombre de archivo, que puede estar desactualizado.
Son una ayuda para la exploración; utilice los campos de cada lado por separado para una comparación completa de antes y después.

| Campo | Significado e interpretación |
|---|---|
| `MetadataSide` | `Redo` o `Undo`, que identifica el lado que proporciona las columnas prácticas de nombre de archivo y marcas de tiempo. |
| `EntryNumber`, `SequenceNumber` | Archivo descrito por los metadatos decodificados de nombre de archivo, registro de archivo o atributo, o por una actualización parcial de marcas de tiempo de `$STANDARD_INFORMATION`, con la secuencia cuando se conoce. Vacío para una actualización de entrada de directorio que no identifica el elemento secundario y para las actualizaciones sin procesar de tamaño o de lista de ejecuciones. Utilice `TargetEntryNumber` para el registro afectado. |
| `Filename` | Nombre decodificado preferido; todos los nombres permanecen en las matrices de cada lado, incluidos los alias DOS y los vínculos físicos. |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | Referencia del elemento principal y ruta actual en la MFT; no es una reconstrucción de la ruta histórica. |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | Horas de metadatos en UTC, con precisión de 100 ns. No son marcas de tiempo de la transacción. Las horas ausentes o no válidas quedan vacías o nulas. |
| `FileSize`, `FileAttributes` | Tamaño e indicadores numéricos de Windows de los metadatos seleccionados; los valores de las entradas de directorio pueden estar desactualizados. Los atributos `$DATA` y los registros de archivo decodificados pueden proporcionar un tamaño más directo. |
| `AttrType` | Tipo numérico de un registro de atributo decodificado; 48 para una actualización de metadatos de entrada de directorio decodificada. Independiente del atributo abierto al que se dirige la operación. |

### Metadatos y cargas útiles completos de rehacer y deshacer { #complete-redoundo-metadata-and-payloads }

| Campo | Significado e interpretación |
|---|---|
| `RedoFileNames`, `UndoFileNames` | Matrices de todos los `$FILE_NAME` decodificados en ese lado, que conservan los nombres añadidos o nuevos frente a los eliminados o antiguos. |
| `RedoTimes`, `UndoTimes` | Marcas de tiempo de `$STANDARD_INFORMATION` escritas por cada lado; las actualizaciones parciales dejan ausentes las marcas de tiempo no modificadas en lugar de suponerlas. |
| `RedoDecoded`, `UndoDecoded` | Información estructurada de registro de archivo, atributo, entrada de índice o entrada de directorio, identificada por `Kind`. Nulo puede significar una operación no admitida o bytes insuficientes. |
| `JournalWriteOffset`, `JournalWriteLength` | Desplazamiento y número de bytes de los datos de rehacer que escriben en `$DATA:$J`. Identifica una posible fuente de registros USN que de otro modo se perderían; no exporta un diario reconstruido. |
| `Checkpoint` | Versión del punto de control, LSN inicial, tamaño de clúster y LSN y longitud de los volcados de atributos abiertos, nombres de atributos, páginas modificadas y tabla de transacciones. |
| `OtherRecordType` | Tipo de registro de cliente sin procesar para un cuerpo no reconocido o no decodificable. |
| `RedoData`, `UndoData`, `OtherData` | Cargas útiles en hexadecimal solo con `--raw-data`; en otro caso, nulas o vacías. Consérvelas para validar estructuras no admitidas o dañadas. |

Cada objeto de nombre de archivo incluye las referencias del archivo y del elemento principal (`Raw`, `EntryNumber`,
`SequenceNumber`), `Name`, `NameEscaped`, `Namespace`, el `ParentPath` actual, cuatro marcas de tiempo,
`Size`, `AllocatedSize`, `Attr` y `ReparseValue`. `NameEscaped` identifica el UTF-16 no válido
representado con secuencias de escape. Un valor de reanálisis puede ser una etiqueta de reanálisis o el tamaño de los atributos extendidos.

Los objetos de marca de tiempo conservan tanto `Utc` como el entero `Filetime`; un valor centinela o no válido puede
tener una representación UTC nula y conservar su entero original. `RedoTimes` y `UndoTimes`
contienen `Created`, `Modified`, `Changed` y `Accessed`; los valores de los miembros ausentes son nulos.

Los objetos `FileRecord` decodificados incluyen el número, la secuencia, el recuento de vínculos físicos, los indicadores, la referencia base,
si solo sobrevive el encabezado y los atributos presentes. Cada atributo decodificado conserva el
tipo, el nombre, el estado residente, los indicadores, el identificador, el tamaño y el tamaño asignado. `$STANDARD_INFORMATION` añade las
cuatro horas, los atributos, los identificadores de propietario y de seguridad, la cuota y el USN; estos identificadores requieren una resolución aparte y
no prueban qué usuario modificó el archivo. Las entradas de índice incluyen los indicadores, la longitud de la clave y un
nombre de archivo opcional. Las actualizaciones de entradas de directorio conservan las cuatro horas, los tamaños, los atributos y el valor de
reanálisis aunque el propio nombre no se haya registrado.

## Flujo de trabajo de investigación { #investigation-workflow }

1. Revise los diagnósticos en busca de páginas de reinicio ausentes, páginas rotas y registros truncados.
2. Busque un nombre de archivo o una extensión sospechosos tanto en `RedoFileNames` como en `UndoFileNames`.
3. Empareje los números de entrada **y de secuencia** y, a continuación, examine la cadena de LSN y las operaciones de la transacción.
4. Compare las marcas de tiempo de rehacer y deshacer en busca de cambios. Las copias, extracciones y restauraciones rutinarias también
   pueden cambiar las marcas de tiempo; una diferencia es una pista, no un veredicto automático de timestomping.
5. Correlacione con el [Diario USN](usn-journal.md), la [$MFT](mft.md) y [$I30](i30.md). Utilice los registros de
   eventos, [Prefetch](prefetch.md) y los artefactos de actividad del usuario para establecer la ejecución o la atribución.

Por ejemplo, para buscar todos los registros que contienen un nombre de archivo cifrado en cualquiera de los lados:

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

Se trata de una búsqueda por nombre de archivo, no de un detector completo de ransomware. Para inspeccionar las cargas útiles del diario,
exporte con `--raw-data` y seleccione los registros con un `JournalWriteOffset` no nulo; el hexadecimal de `RedoData`
representa los bytes escritos en ese desplazamiento. El análisis y la eliminación de duplicados de esos registros USN incrustados
son un paso aparte. Juntei aún no los combina con la salida de `parse-ntfs usn-journal`.

## Más información { #further-details }

- [Crate logfile de ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile): estructuras admitidas, corpus de validación y limitaciones conocidas.
- [ntfslog de Sootmark](https://github.com/Sootmark/ntfslog): implementación original en Rust.
- [Documentación del formato NTFS de libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation): páginas de registro, correcciones (fixups) y estructuras de metadatos.
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs): investigación e implementación independientes del análisis de NTFS.
- [LogFileParser de Joakim Schicht](https://github.com/jschicht/LogFileParser): análisis detallado del registro de transacciones.
- [ntfs-linker de Stroz Friedberg](https://github.com/strozfriedberg/ntfs-linker): correlación de fuentes NTFS en un historial de archivos.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum): muestras reales de NTFS de Windows.
- [go-ntfs de Velocidex](https://github.com/Velocidex/go-ntfs): implementación de NTFS y volumen de prueba público.
