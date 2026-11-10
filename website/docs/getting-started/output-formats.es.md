# Formatos de salida { #output-formats }

## Resúmenes en la terminal { #terminal-summaries }

Sin `-o` ni un `-t` estructurado, los comandos suelen mostrar recuentos, intervalos de tiempo,
daños y hallazgos de alto valor. Los resúmenes ayudan a comprender el alcance antes de generar una
salida voluminosa.

## CSV { #csv }

CSV es el formato de guardado predeterminado. Resulta cómodo para hojas de cálculo, importaciones
SQL y visores de líneas de tiempo. Algunos analizadores crean varias tablas porque sus artefactos
contienen distintos tipos de registro (Amcache, Jump Lists y SRUM).

!!! warning "Trate las columnas de evidencia como texto"
    Una ruta o una línea de comandos puede empezar por `=`, `+`, `-` o `@`. Las hojas de cálculo
    pueden interpretarla como una fórmula. Importe todos los campos como texto o utilice un visor
    CSV seguro desde el punto de vista forense. Juntei conserva las cadenas de evidencia en lugar de
    modificarlas.

## Nombres de campo { #field-names }

Los encabezados de columna CSV y los nombres de campo JSON/JSONL usan PascalCase, incluidos Amcache
y los manifiestos de `auto-parse`. Las siglas permanecen en mayúsculas: `ProcessID`,
`TimestampUTC`, `UserSID`, `SHA256`. Una palabra que sigue a unas siglas se separa con un guion:
`IP-Address`, `MFT-Entry`, `USN-Journal`. Con `jq`, entrecomille los campos con guion:
`.["IP-Address"]`.

El CSV predeterminado de la MFT utiliza encabezados más cortos y legibles, como `EntryNum`, `Ext`,
`Dir`, `HasADS`, `Size`, `Created (SI)` y `Created (FN)`. Consulte la
[referencia de campos de la MFT](../artifacts/mft.md) para ver su esquema CSV completo. Los pares de
marcas de tiempo siguen el orden MACB: DataModified, Accessed, MetadataChanged, Created, con SI
antes que FN. En las herramientas CSV, entrecomille los encabezados que contengan espacios o signos
de puntuación.

Los valores de evidencia conservan su grafía original. Las claves de diccionario proporcionadas por
la fuente, como los nombres de valores del Registro bajo `Details`/`DataRaw` y los nombres de
`Zone.Identifier` bajo `ZoneIdentifier`, tampoco se modifican. Los diccionarios `ValueState` y
`Raw` de SRUM usan las mismas etiquetas que sus columnas de datos. Si se produce una colisión de
normalización, se notifica en lugar de descartar un campo de forma silenciosa.

Esto modifica los esquemas de exportación anteriores. Vuelva a generar los resultados existentes y
actualice las consultas guardadas (por ejemplo, `.record_type` pasa a ser `.RecordType` y el
`full_path` de Amcache pasa a ser `FullPath`). Los informes de texto legibles por personas y los
formatos de entrada y configuración mantienen sus convenciones actuales.

## Nombres de campo abreviados { #abbreviated-field-names }

Las siguientes etiquetas se aplican a las exportaciones CSV, JSON y JSONL, incluidos el análisis
automático y los manifiestos. Actualice los filtros guardados de `jq`, `xsv` y hojas de cálculo al
volver a generar resultados antiguos. Solo cambian las etiquetas de salida; los valores de
evidencia y los nombres de valores del Registro sin procesar conservan su grafía original.

| Etiqueta anterior | Etiqueta actual |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

La salida CSV de MFT utiliza columnas específicas: `File` (Filename), `SN` (Sequence Number), `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number), `USN` (Update Sequence Number), `LogfileSN` (Logfile Sequence Number), `SID` (Security ID), `Type` (Name Type) y `RefCnt` (Reference Count). También se aplican a los CSV de MFT de `auto-parse`; consulta los [campos de salida de MFT](../artifacts/mft.md#csv-output-fields).

La hora del JSON nativo de información del sistema es `CompNameLastWritten` (CSV:
`CompNameLastWrittenUTC`). `HasADS` es un indicador booleano; `Dir` en Prefetch contiene la lista
de directorios registrada. `User` conserva el tipo propio de cada artefacto: una cadena para un
nombre de cuenta o la lista de usuarios asociados en la salida de USB. No implica una identidad que
haya ejecutado algo. Entrecomille los signos de puntuación en las consultas: `."SI<FN"` y
`.["ZeroU-Secs"]`.

## JSON { #json }

JSON almacena el resultado completo como un único documento. Conserva matrices, estructuras
anidadas, valores nulos, enteros grandes y distinciones que un CSV plano no siempre puede
representar.

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines { #json-lines }

JSONL escribe un objeto JSON independiente por línea. Funciona bien en flujo y se integra de forma
natural con `jq`:

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

Cuando la salida estructurada se dirige a la salida estándar, Juntei envía el progreso y los
mensajes a la salida de error estándar para que una tubería reciba únicamente datos.

`auto-parse -j timeline.jsonl` crea un conjunto de datos JSONL adicional para una recopilación
completa de varios hosts, sin dejar de generar los resultados CSV clasificados. Añada `-t json` o
`-t jsonl` para cambiar el formato de cada resultado individual. Cada registro tiene los campos
`Comp` y `Artifact` para que su origen siga siendo claro tras la ingesta en un SIEM.

## UTC y precisión { #utc-and-precision }

Juntei da formato a las horas analizadas en UTC con RFC 3339 cuando el formato lo permite. Algunos
artefactos tienen una precisión de 100 nanosegundos; otros almacenan segundos, hora FAT, intervalos
de agregación o ninguna hora. Más dígitos no significan más exactitud: entienda siempre el campo de
origen que se describe en la página de su artefacto.

UTC utiliza la forma de los registros de eventos de Windows `2026-10-20T08:46:38.2001668Z`,
conservando la precisión fraccionaria disponible. `auto-parse -Z "+09:00"` muestra los instantes
analizados con un desplazamiento fijo explícito en todos sus formatos de resultado y en su
exportación `-j` opcional. El valor predeterminado es UTC, independientemente del equipo en el que
se ejecute Juntei. Los valores sin procesar y las horas locales sin zona se mantienen tal como se
registraron; consulte [zonas horarias de salida](../commands/auto-parse.md#output-time-zone).

## Daños y valores ausentes { #damage-and-missing-values }

Los valores vacíos, `null`, `NA`, `Problems` y los campos de estado específicos de cada analizador
tienen significado. Pueden indicar que el artefacto no almacenaba un valor, que el registro estaba
dañado, que el formato no lo admite o que el analizador no pudo resolverlo. No sustituya todos los
valores vacíos por cero.

## Línea de tiempo normalizada { #normalized-timeline }

[`super-timeline`](../commands/super-timeline.md) crea un esquema de eventos CSV/JSONL común, con
el significado de cada marca de tiempo etiquetado y referencias de origen. La salida no se ordena
de forma predeterminada; `--sort` habilita la ordenación cronológica y la consolidación de
duplicados. También admite un perfil de importación para Timesketch. Las exportaciones nativas de
los analizadores y `auto-parse -j` mantienen sus esquemas actuales.
