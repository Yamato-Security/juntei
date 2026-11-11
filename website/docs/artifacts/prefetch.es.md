# Prefetch { #prefetch }

## Qué es Prefetch { #what-prefetch-is }

Los archivos Prefetch de Windows registran información que se utiliza para optimizar el inicio de las aplicaciones. Un archivo `.pf`
específico de un programa suele almacenar el nombre y el hash del ejecutable, el recuento de ejecuciones, las horas de las ejecuciones recientes, información
del volumen, los directorios y los archivos a los que se hizo referencia durante el inicio.

En las versiones de Windows compatibles, un registro Prefetch de aplicación válido suele ser una evidencia sólida
de que el ejecutable se ejecutó. Es uno de los artefactos de ejecución ajenos a los registros de eventos más útiles.

<span class="evidence-strength">Mejor conclusión:</span> el ejecutable indicado se inició a la
hora u horas de ejecución reciente registradas, con sujeción al comportamiento del sistema operativo y a la validez del análisis.

## Ubicación y recopilación { #location-and-collection }

```text
C:\Windows\Prefetch\*.pf
```

Juntei admite las versiones de formato 17 a 31 (de Windows XP a Windows 11), incluidos los archivos comprimidos
de Windows 8 y posteriores. Recopile el directorio completo para poder comparar los hashes de los nombres de ejecutable y los registros
contiguos.

## Precauciones para el analista { #analyst-cautions }

- Windows solo almacena una hora de ejecución histórica en las versiones antiguas y hasta ocho en las más recientes.
  `RunCount` puede superar el número de marcas de tiempo disponibles.
- Prefetch puede estar deshabilitado, limpiado, excluido o no disponible en algunos sistemas o funciones. Su ausencia no
  demuestra que un programa no se ejecutara.
- Las listas de archivos cargados describen las dependencias y los accesos durante el inicio; no demuestran que todos los archivos enumerados
  fueran maliciosos, se abrieran deliberadamente o se leyeran por completo.
- El nombre del ejecutable en el encabezado de Prefetch tiene una longitud limitada; Juntei resuelve una ruta a partir de los archivos
  cargados cuando es posible hacerlo de forma unívoca.
- Las marcas de tiempo del sistema de archivos de un archivo de origen `.pf` son independientes de las horas de ejecución incrustadas.
- Los `Findings` son observaciones para la búsqueda de amenazas. Los instaladores y la administración legítima suelen utilizar rutas de preparación.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.ExecutableName? | test("powershell|rundll32"; "i"))'
```

## Campos CSV estándar { #standard-csv-fields }

| Campo | Interpretación |
|---|---|
| `SourceFile` | `.pf` de entrada; consérvelo para distinguir hosts y nombres de ejecutable duplicados. |
| `ExecutableName` | Nombre del ejecutable en el encabezado. No distingue mayúsculas de minúsculas en la mayoría de las búsquedas; conserve la grafía original. |
| `ExecutablePath` | Ruta cargada resuelta por Juntei cuando es posible. Confírmela con `Files` y con otros artefactos. |
| `PrefetchHash` | Hash que Windows utiliza en el nombre del archivo `.pf`; no es un hash criptográfico del contenido del archivo. |
| `FormatVersion` | Versión de la estructura de Prefetch; determina los campos disponibles y el número de marcas de tiempo. |
| `RunCount` | Contador acumulado registrado por Windows. Útil para la frecuencia, no para el historial exacto. |
| `LastRun` | Hora de ejecución conservada más reciente. |
| `PreviousRun1` … `PreviousRun7` | Horas de ejecución conservadas anteriores, de la más reciente a la más antigua. Vacías cuando el formato o el historial no las tienen. |
| `VolumeCount` | Número de volúmenes de origen registrados. |
| `VolumeDevicePaths` | Rutas de dispositivo NT de los volúmenes utilizados durante el inicio. Correlaciónelas con los números de serie. |
| `VolumeSerialNumbers` | Identificadores de volumen; útiles para vincular evidencias de unidades extraíbles o reformateadas. |
| `VolumeCreated` | Horas de creación de los volúmenes registradas, no horas de creación del ejecutable. |
| `DirectoryCount` | Número de cadenas de directorio registradas. |
| `FileCount` | Número de referencias a archivos cargados registradas. |
| `Findings` | Observaciones de Juntei, como binarios del sistema reubicados o rutas de ejecución sospechosas. Revíselas manualmente. |
| `Files` | Archivos cargados o referenciados con las referencias NTFS disponibles. Excelente pivote de DLL y rutas, pero no demuestra la intención sobre un archivo. |
| `Dir` | Cadenas de directorio asociadas al inicio. Útiles cuando las rutas de los archivos cargados individuales están incompletas. |

## Campos CSV de la línea de tiempo (`-T`) { #timeline-csv-fields-t }

| Campo | Interpretación |
|---|---|
| `RunTime` | Una hora de ejecución conservada en Prefetch. |
| `ExecutableName`, `ExecutablePath` | Identidad del programa y contexto de la ruta. |
| `PrefetchHash` | Separa variantes o rutas que comparten un nombre de ejecutable. |
| `RunCount` | Contador del archivo Prefetch; se repite en cada fila de la línea de tiempo. |
| `SourceFile` | `.pf` de origen para la validación. |

JSON/JSONL conserva los volúmenes, los directorios, los archivos cargados y las referencias a archivos NTFS como matrices
estructuradas, en lugar de celdas CSV unidas por comas.

## Estrategia de investigación { #investigation-strategy }

1. Utilice `-T` para añadir las ejecuciones conservadas a una línea de tiempo.
2. Agrupe por nombre de ejecutable y hash de Prefetch; un mismo nombre puede tener rutas y hashes distintos.
3. Revise `ExecutablePath`, las rutas de preparación, los hallazgos de binarios del sistema reubicados y las DLL cargadas.
4. Haga coincidir las referencias MFT de destino y las rutas de archivo con los registros de `$MFT` y del USN.
5. Correlacione las horas de ejecución con los registros de eventos, SRUM, la telemetría de red, los servicios, las tareas y los artefactos del usuario.

## Más información { #further-details }

- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [Security Ronin prefetch-forensic](https://github.com/SecurityRonin/prefetch-forensic)
