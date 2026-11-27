# Carving de registros de eventos { #carve-evtx }

Recupera los bloques (chunks) de registros de eventos que quedan después de vaciar o eliminar un registro: de una imagen de disco, del espacio no asignado, de un archivo de paginación o de una imagen de memoria. Cada bloque EVTX de 64 KiB lleva sus propias tablas de cadenas y de plantillas, por lo que sus registros pueden leerse sin el resto del registro de eventos. Juntei escribe los bloques recuperados como archivos `.evtx` normales, uno por canal, para abrirlos con Hayabusa o con el Visor de eventos.

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
```

Sin `-o`, solo se muestra el resumen.

## Búsqueda de bloques { #finding-chunks }

- La firma de bloque se busca en cada desplazamiento de byte, y cada encabezado debe ser plausible (tamaño del encabezado, números de registro, desplazamientos). Las sumas de comprobación se informan, pero no se exigen: un bloque sobrescrito en parte todavía contiene registros legibles.
- Los bloques de registros de eventos con compresión NTFS se descomprimen (LZNT1, unidades de compresión de 16 clústeres). En una imagen de disco se usa el tamaño de clúster del volumen; para otros archivos, `--cluster-size <BYTES>` (predeterminado: 4096).
- Los registros se recorren según su tamaño hasta el desplazamiento del último registro que indica el encabezado, y un ID de registro repetido nunca detiene el recorrido. Tras un registro dañado, el recorrido continúa en el siguiente registro válido. También se enumeran los registros antiguos que quedan en el espacio residual del bloque.

## Imágenes de disco y volúmenes NTFS { #disk-images }

Con una imagen E01 o VHDX, o con un disco o volumen sin formato (raw) que contenga NTFS, cada bloque se clasifica según el clúster en el que se encuentra:

| Ubicación | Significado |
|---|---|
| `FreeSpace` | Un clúster que no usa ningún archivo. |
| `DeletedLog` | Un clúster libre que todavía figura en el registro de la `$MFT` de un archivo EVTX eliminado, o un archivo EVTX reconstruido a partir de esos clústeres. |
| `LiveLog` | Un clúster de un registro de eventos en uso. Se omite salvo con `--keep-live`. |
| `AllocatedFile` | Un clúster de otro archivo, como `pagefile.sys` o el archivo de almacenamiento de una instantánea de volumen. |
| `AllocatedUnknown` | Un clúster asignado que no figura en ningún registro de la `$MFT`. |
| `OutsideVolumes` | Fuera de todos los volúmenes NTFS del disco. |
| `Unclassified` | La entrada no contiene ningún volumen NTFS (un archivo de paginación o una imagen de memoria). |

Los archivos EVTX eliminados cuyo registro de la `$MFT` solo enumera clústeres libres se reconstruyen completos, incluidos los comprimidos, y se guardan en `Recovered/`. `--free-space-only` solo busca en los clústeres libres: es más rápido, pero omite el archivo de paginación, las instantáneas de volumen y otros archivos.

## Deduplicación { #deduplication }

Cada bloque se escribe una sola vez. Un bloque se omite cuando pertenece a un registro de eventos activo, cuando es idéntico a otro bloque o cuando todos sus registros ya están en los registros de eventos activos o en un bloque ya escrito. Los bloques con más registros se consideran primero, de modo que una copia más antigua y más corta de un bloque no aporta nada. Para una entrada sin NTFS, indique los registros de eventos activos con `--live-logs <DIR>`.

## Resultados { #results }

- `Carved/<CHANNEL>.evtx`: Un registro de eventos por canal, con el mismo nombre de archivo que usa Windows (`Microsoft-Windows-Sysmon%4Operational.evtx`). Los números de registro de los encabezados de los bloques se renumeran y las sumas de comprobación se recalculan. Un registro dañado y todo lo que le sigue pasan a ser espacio residual. Por lo demás, cada bloque se conserva entero, incluido su espacio residual, para que la recuperación de registros, como la opción `-x` de Hayabusa, siga funcionando con él.
- `Recovered/Volume<N>/...`: Archivos EVTX eliminados, reconstruidos a partir de sus registros de la `$MFT`.
- `chunks.csv` / `chunks.jsonl`: Todos los bloques encontrados: dónde estaban, qué se hizo con ellos (`Status`), su canal, los ID de registro, las marcas de tiempo y el estado de la suma de comprobación.
- `records.csv` / `records.jsonl`: Todos los registros de los bloques escritos. `OnlyInCarvedData` marca los registros que no están en los registros de eventos activos.

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

El carving solo encuentra bloques contiguos. Un bloque repartido entre clústeres no contiguos, o entre páginas físicas no contiguas en memoria, queda incompleto, y sus últimos registros pueden estar dañados o faltar. Descomprima primero `hiberfil.sys`. El código de salida es 0 cuando la búsqueda se completa y 1 si se produce un error.

## Más detalles { #further-details }

- [Uso detallado y referencia de campos (inglés)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.md)
- [Uso detallado y referencia de campos (japonés)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.ja.md)
- [Formato EVTX (documentación de libevtx)](https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc)
