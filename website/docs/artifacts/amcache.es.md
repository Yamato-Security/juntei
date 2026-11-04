# Amcache { #amcache }

## Qué es Amcache { #what-amcache-is }

Amcache es un inventario de compatibilidad de aplicaciones de Windows almacenado en un hive del Registro. Los sistemas modernos
registran los archivos ejecutables en `InventoryApplicationFile` y los dispositivos Plug and Play en
`InventoryDevicePnp`; las versiones anteriores de Windows utilizan una estructura heredada `Root\File`.

En la investigación de ejecutables, Amcache es valioso porque puede conservar la ruta completa, el SHA-1,
el tamaño del archivo, los metadatos PE, la información de versión, el texto del editor, la fecha de vinculación y las marcas de tiempo del Registro
después de que el archivo original haya desaparecido. Los registros de dispositivos pueden identificar el hardware conectado, los controladores,
las fechas de instalación y los identificadores.

<span class="evidence-strength">Mejor conclusión:</span> Windows inventarió el archivo o
dispositivo registrado. Un registro de ejecutable es **evidencia de presencia, no prueba de ejecución**.

## Ubicación y recopilación { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

Recopile el hive de la instalación afectada. Actualmente, `parse-registry amcache` analiza el hive
principal, pero **no** reproduce los registros de transacciones de Amcache adyacentes, por lo que los registros presentes únicamente en esos
registros de transacciones pueden faltar en una recopilación en vivo o incoherente. Indique esa limitación en el informe.

## Precauciones para el analista { #analyst-cautions }

- Una entrada de ejecutable de Amcache no demuestra que un usuario iniciara el archivo.
- `Publisher` son metadatos PE o del recurso de versión registrados por Windows. No demuestra que el archivo tuviera
  una firma Authenticode válida en el momento de la recopilación.
- Los hashes pueden estar ausentes, y los esquemas heredado y moderno no exponen valores idénticos.
- `LinkDate` procede del encabezado PE y puede falsificarse, copiarse o generarse mediante compilaciones reproducibles.
- La hora de última escritura de una clave describe la actualización del registro del Registro, no necesariamente la descarga, la creación,
  la primera ejecución ni la última ejecución.
- Las rutas pueden hacer referencia a volúmenes extraíbles o que ya no están presentes.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

La salida CSV contiene las tablas de resumen, hallazgos, archivos y dispositivos. Con `auto-parse`, son
`Registry-Amcache-Summary.csv`, `Registry-Amcache-Findings.csv`, `Registry-Amcache-Files.csv` y
`Registry-Amcache-Devices.csv`, directamente en el directorio `FileExistence/` de cada equipo, con
`Registry-Amcache-logs.txt` junto a las tablas. Si hay varios hives, se añade `-Source1`, `-Source2`, etc.
antes de la extensión; `OutputFiles` en los manifiestos indica las rutas exactas. Las exportaciones CSV
independientes conservan sus nombres de tabla nativos y la ubicación de salida solicitada. Los registros JSONL utilizan
los valores de `RecordType` `summary`, `finding`, `file` y `device`.

## Campos de salida de archivos { #file-output-fields }

| Campo | Cómo debe utilizarlo el analista |
|---|---|
| `KeyName`, `SourceKeyPath` | Identifican el registro exacto de origen en el Registro y la rama del esquema. Consérvelos para la reproducibilidad. |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | Hora de actualización del registro en el Registro, en formato legible y sin procesar. No la etiquete como hora de ejecución. |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | Hora de la clave de volumen principal en los esquemas heredados; contextual, no una hora de acción sobre el archivo. |
| `FileID` | Identificador de archivo registrado por Amcache. Se conserva exactamente; no suponga que siempre es un SHA-1. |
| `SHA1`, `SHA256` | Pivotes sólidos hacia Defender, la inteligencia de amenazas, el EDR y los repositorios de archivos. Nulo significa que no se registró. |
| `Name`, `OriginalFileName`, `Description` | Nombres PE o del recurso de versión registrados. Compárelos con la ruta real para detectar suplantaciones; el atacante controla los metadatos. |
| `FullPath` | El pivote de ubicación más útil. Normalice las mayúsculas y minúsculas con cuidado, pero conserve la cadena de evidencia original. |
| `LongPathHash` | Identificador de Windows relacionado con la ruta; útil para emparejar registros, no es un hash del contenido. |
| `ProgramID` | Vincula el archivo con un registro de inventario de aplicaciones cuando está disponible. |
| `Publisher` | Texto del editor registrado. Trátelo como metadatos, no como validación de firma. |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | Contexto del recurso de versión. Las diferencias o los valores inverosímiles pueden respaldar el análisis de suplantaciones. |
| `BinaryType` | Contexto de arquitectura o tipo registrado; compárelo con el análisis PE cuando se recupere el archivo. |
| `LinkDate` | Marca de tiempo de compilación o vinculación PE tal como se registró. Útil solo con escepticismo y corroboración. |
| `Size` | Tamaño registrado; compárelo con `$MFT`, el contenido de Defender o un archivo recuperado. |
| `Language` | Identificador de idioma PE; una pista contextual débil, no una atribución. |
| `IsPE-File`, `IsOS-Component` | Indicadores de clasificación registrados por Windows. Un valor falso no demuestra que el archivo sea inofensivo. |
| `USN` | Valor de secuencia de actualización de NTFS registrado por Amcache; puede ayudar a pivotar hacia registros USN que sigan existiendo. |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | Asocian aplicaciones o componentes empaquetados con el archivo. |
| `RegistryValues` | Todos los valores originales del Registro con su tipo, valor decodificado y hexadecimal sin procesar. Utilícelo cuando falte un campo tipado o para campos futuros o heredados. |

## Campos de salida de dispositivos { #device-output-fields }

| Grupo de campos | Campos e interpretación |
|---|---|
| Origen | `KeyName`, `SourceKeyPath`, `KeyLastWrittenUTC`, `KeyLastWrittenFiletime`: registro exacto del dispositivo y hora de actualización en el Registro. |
| Identidad | `Description`, `BusDescription`, `Manufacturer`, `Model`: descripciones del dispositivo legibles para las personas; las cadenas del fabricante pueden ser genéricas. |
| Identificadores de hardware | `HWID`, `CompID`, `Class`, `ClassGUID`, `ContainerID`, `DeviceInterfaceClasses`: los pivotes más sólidos de familia e instancia del dispositivo. |
| Estado | `DeviceState`, `ProblemCode`, `ParentID`, `MatchingID`, `Enumerator`: contexto de enumeración y de relaciones de Windows. |
| Controlador | `DriverID`, `DriverName`, `DriverPackageStrongName`, `DriverVerDate`, `DriverVerVersion`, `Provider`, `Service`, `INF`: asociaciones con el paquete de controladores y el servicio; correlaciónelas con los archivos y servicios instalados. |
| Instalación | `InstallDate`, `FirstInstallDate`, `InstallState`, `ExtendedInfs`: ciclo de vida de instalación registrado. La ausencia de una fecha no demuestra que el dispositivo nunca se conectara. |
| Filtros y pila | `LowerClassFilters`, `UpperClassFilters`, `LowerFilters`, `UpperFilters`, `StackID`, `LocationPaths`: contexto avanzado de la pila de controladores; los controladores de filtro inesperados merecen revisión. |
| Conservación sin procesar | `RegistryValues`: valores de origen completos, tipados y sin procesar, para la validación y los campos no admitidos. |

Las columnas de matrices y de valores sin procesar en CSV utilizan el sufijo `JSON` (por ejemplo, `HWID-JSON` y
`RegistryValuesJSON`); sus celdas contienen JSON, mientras que JSON/JSONL conserva las matrices nativas.

## Estrategia de investigación { #investigation-strategy }

1. Pivote sobre `FullPath`, `SHA1`, `Name` y `USN`.
2. Busque un Prefetch coincidente que respalde la ejecución.
3. Utilice `$MFT` y el Diario USN para establecer la creación, el cambio de nombre, la eliminación y el tamaño del archivo.
4. Compare los metadatos del editor y la versión con un PE recuperado de forma independiente y con la validación de la firma.
5. Revise la cuarentena de Defender y los registros de eventos en busca de detecciones o eliminaciones.

## Más información { #further-details }

- [Esquema de salida de ys-amcache-parser de Juntei](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [AmcacheParser de Eric Zimmerman](https://github.com/EricZimmerman/AmcacheParser)
- [amcache-forensic de Security Ronin](https://github.com/SecurityRonin/amcache-forensic)
- [Complemento Amcache de Regipy](https://github.com/mkorman90/regipy)
