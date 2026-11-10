# Jump Lists { #jump-lists }

## Qué son las Jump Lists { #what-jump-lists-are }

Las Jump Lists respaldan los menús que se muestran desde las entradas de aplicaciones de la barra de tareas y del menú Inicio. Las Jump Lists
automáticas son historiales gestionados por Windows de los elementos que ha utilizado una aplicación. Las Jump Lists personalizadas son
categorías y tareas definidas por la aplicación. Ambas pueden incrustar estructuras LNK con rutas de destino, horas,
contexto de volumen y de red, referencias MFT y argumentos de inicio.

Los registros automáticos pueden revelar archivos o carpetas abiertos por una aplicación, el orden y la hora del último uso, el
recuento de interacciones y el estado de anclaje. Las listas personalizadas pueden mostrar páginas «cerradas recientemente» o tareas de la aplicación,
incluidos argumentos de comando maliciosos.

<span class="evidence-strength">Mejor conclusión:</span> la aplicación propietaria registró o expuso
el elemento o la tarea en su Jump List. El comportamiento varía según la aplicación; corrobore la ejecución o el acceso reales.

## Ubicaciones y recopilación { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

El prefijo hexadecimal del nombre de archivo es un AppID. Recopile ambos directorios y conserve la ruta del perfil
de usuario. Las listas automáticas son contenedores OLE Compound File con un flujo `DestList` y flujos LNK
numerados; las listas personalizadas utilizan su propio formato de categorías y vínculos.

## Precauciones para el analista { #analyst-cautions }

- Las descripciones de AppID proceden de una tabla de correspondencias conocida; un AppID desconocido no es sospechoso por sí mismo.
- Los campos `LastModified` y de último uso pertenecen a una entrada de DestList; las horas del destino incrustado son distintas.
- Los recuentos de interacciones solo existen en las versiones compatibles y no son un recuento completo de aperturas.
- El anclaje conserva los elementos más allá de la recencia normal; la limpieza o la configuración de la aplicación pueden eliminar otros.
- Una lista puede contener flujos obsoletos o no vinculados, o bytes de una lista personalizada anterior y más larga.
- Los argumentos y las rutas de los LNK incrustados están controlados por el atacante. La asociación con una aplicación es contexto, no garantía de confianza.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

Con `auto-parse`, las tablas CSV son `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`
y `FileAndFolderOpening/JumpLists-CustomDestinations.csv`, directamente bajo cada equipo.
El registro compartido es `JumpLists-logs.txt` en esa categoría. Los manifiestos enumeran las rutas exactas
en `OutputFiles`. Las exportaciones CSV independientes conservan `AutomaticDestinations.csv` y
`CustomDestinations.csv` en el directorio que elija. Las exportaciones automáticas JSON y JSONL son
archivos únicos `JumpLists.json` o `JumpLists.jsonl`; la estructura de los registros no cambia.

## Campos CSV de AutomaticDestinations { #automaticdestinations-csv-fields }

| Grupo de campos | Campos e interpretación |
|---|---|
| Origen | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: el contenedor de la Jump List y sus horas del sistema de archivos. |
| Aplicación | `AppId`, `AppIdDescription`: AppID del nombre de archivo y correspondencia con una aplicación conocida. No deduzca la identidad a partir de un hash desconocido. |
| Contenedor | `HasSps`, `DestListVersion`, `LastUsedEntryNumber`: estructura y versión de DestList y contexto del encabezado. |
| Orden | `MRU`, `EntryNumber`: posición de uso más reciente e identificador hexadecimal de la entrada o del flujo. |
| Metadatos de uso | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition`: ciclo de vida, uso y estado de anclaje en DestList. Interprételos según la versión. |
| Host y ruta | `Hostname`, `MacAddress`, `Path`, `DisplayPath`: ubicación del destino y contexto de seguimiento. El host o la MAC no bastan como única base de atribución. |
| Identificadores de seguimiento | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid`: correlacionan destinos movidos y el seguimiento del LNK incrustado. |
| Notas | `Notes`: informa de flujos no vinculados, restos recuperados u otro contexto del analizador. |
| LNK incrustado | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Args`: interprételos como se describe en [Accesos directos LNK](lnk.md). |
| LNK adicional | `NetworkPath`, `Name`, `IconLocation`: campos de red y de presentación del acceso directo incrustado. |
| Validación | `Problems`: flujos ausentes, truncamiento, recuentos discrepantes o vínculos dañados. |

## Campos CSV de CustomDestinations { #customdestinations-csv-fields }

La salida personalizada empieza por `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription` y `EntryName` (nombre de la categoría o `Tasks`), seguidos de todos los campos de LNK incrustado
indicados arriba, y después `Notes`, `NetworkPath`, `Name`, `IconLocation` y `Problems`.

Además, JSON/JSONL conserva el encabezado de DestList, la puntuación, la suma de comprobación y el almacén de propiedades de cada entrada,
`DestListPropertyStore`, los flujos del archivo compuesto, las categorías personalizadas, los bytes posteriores a la última categoría
y los campos anidados de cada LNK incrustado.

## Estrategia de investigación { #investigation-strategy }

1. Identifique el usuario y el perfil de origen, y después asocie el AppID con una aplicación con cautela.
2. Ordene las entradas automáticas por `LastModified` y `MRU`; separe los elementos anclados.
3. Revise las rutas, los recursos compartidos de red, los números de serie de volúmenes extraíbles, las referencias MFT y los argumentos.
4. Inspeccione `Notes` y los vínculos no vinculados o sobrantes: allí pueden sobrevivir elementos eliminados o anteriores.
5. Correlacione con RecentDocs, los LNK independientes de Recent, Prefetch, SRUM, `$MFT` y los registros de eventos.

## Más información { #further-details }

- [Microsoft: formato Compound Binary File (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: formato Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [JLECmd de Eric Zimmerman](https://github.com/EricZimmerman/JLECmd)
- [Proyecto de prueba y referencia JumpList parser](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
