# Súper línea de tiempo { #super-timeline }

`super-timeline` convierte los artefactos analizados en eventos con un esquema común. La salida **no se ordena de forma predeterminada**: sigue el orden en que se descubrieron los archivos de entrada y el orden de proyección de cada analizador, con independencia del orden en que terminen los hilos de trabajo. Añada `--sort` para obtener un orden cronológico y consolidar los duplicados. Un registro de Prefetch puede contener varias horas de ejecución; un registro de la MFT puede contener ocho o más marcas de tiempo de archivo. Cada una se convierte en un evento con su propia etiqueta. Así resulta más fácil comparar evidencias de distintas fuentes sin perder lo que significa realmente cada marca de tiempo.

Comience por los resultados de [`auto-parse`](auto-parse.md), no por el directorio de evidencias original:

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

Conserve los archivos analizados, los manifiestos y las evidencias originales. La línea de tiempo es una proyección para el análisis; no sustituye a esos registros ni crea un archivo de almacenamiento nativo de Plaso.

## Uso { #usage }

```bash
# Native CSV (default)
juntei super-timeline -d parsed/ -o SuperTimeline.csv

# Chronological CSV with duplicate consolidation
juntei super-timeline -d parsed/ -o SortedTimeline.csv --sort

# Native JSON Lines
juntei super-timeline -d parsed/ -o SuperTimeline.jsonl -t jsonl

# Combine native records and an existing Hayabusa JSONL timeline
juntei super-timeline -f NativeRecords.jsonl --hayabusa Hayabusa.jsonl \
  -o SuperTimeline.jsonl -t jsonl

# Timesketch import profile; also available with CSV
juntei super-timeline -d parsed/ -o Timesketch.jsonl -t jsonl --profile timesketch

# One standalone CSV needs an artifact identifier and, if unavailable, a computer name
juntei super-timeline -f MFT.csv --input-artifact mft --computer HOST01 \
  -o Host01Timeline.csv

# Reduce the time range, memory used by sorting, or number of workers
juntei super-timeline -d parsed/ -o IncidentTimeline.jsonl -t jsonl \
  --timeline-start "2026-10-20T06:00:00Z" --timeline-end "2026-10-20T14:00:00Z" \
  --sort --memory-limit 64 -T 4 --temp-dir scratch/
```

`--hayabusa FILE` puede repetirse o usarse por sí solo. Importa las marcas de tiempo de eventos y alertas existentes y el contexto de las reglas sin ejecutar Hayabusa. Se admiten los eventos informativos cuyo `Details` es una cadena. `--artifact mft,prefetch` limita la proyección a los identificadores de artefacto seleccionados. Los límites temporales son inclusivos y requieren un desplazamiento UTC explícito o `Z`.

### Elección de la entrada { #choosing-an-input }

| Entrada | Cómo se identifica |
|---|---|
| `-d DIR` | `manifest.json` proporciona las tareas primarias completadas correctamente, los ID de artefacto, los equipos y los archivos de salida exactos. Las tareas secundarias de `auto-parse -j` se excluyen para no importar ambas copias. |
| `-f FILE` | El JSON/JSONL nativo usa los campos `Artifact` y `Comp` incrustados. Las exportaciones sin etiquetar necesitan `--input-artifact`; un CSV independiente siempre lo requiere. |
| `--computer NAME` | Valor de reserva cuando el registro de entrada no tiene nombre de equipo; si no se indica, un equipo desconocido figura explícitamente como `Unknown`. |
| `--hayabusa FILE` | JSONL con las marcas de tiempo de Hayabusa, los metadatos de las reglas, el equipo y, cuando existe, `EvtxFile`. |

Los manifiestos actuales permiten reubicar el directorio de resultados del análisis: las rutas de salida absolutas registradas se resuelven dentro del nuevo directorio. Las entradas deben permanecer dentro de él, incluidos los destinos de los enlaces simbólicos. Se admiten los directorios heredados por artefacto de Amcache, SRUM y Jump Lists; los directorios de categoría necesitan `OutputFiles` explícitos.

Se admiten los manifiestos actuales con rutas relativas a la salida y las exportaciones de varias tablas aplanadas. El comando reconoce todos los artefactos que programa el análisis automático, incluidas las tablas más recientes de navegadores, WebCache, Outlook y UAL.

Cuando un manifiesto identifica una imagen VHDX o E01, las referencias de origen incluyen la imagen y la ruta interna, y el nombre de la imagen se conserva en el contexto del evento. No se necesita el espacio temporal de preparación para leer los resultados analizados.

El origen sin procesar se toma del registro o de un comando del manifiesto que no sea ambiguo. Con `-d`, las referencias al origen sin procesar son relativas al directorio de evidencias, y `Details.ParsedFile` y los `InputFiles` del resumen son relativos al directorio de resultados del análisis. Los nombres de salida y de informe del resumen son relativos al directorio de salida de la línea de tiempo. Las tablas CSV de SRUM solo se asignan a una base de datos cuando el equipo tiene un único SRUDB.dat recopilado que se pueda identificar sin ambigüedad. En caso contrario, `SourceFile` apunta al archivo analizado y `Details.RawSourceAvailable` es `false`.

Una entrada independiente se referencia por su nombre de archivo. Las entradas con el mismo nombre de archivo usan referencias `./Inputs/<ID>/<filename>` distintas. Las rutas absolutas de origen sin procesar situadas fuera de las raíces de evidencias conocidas usan identificadores `./Sources/<ID>/<filename>`. Estas referencias opacas mantienen la distinción entre orígenes sin exportar los nombres de directorio absolutos del analista; para identificar el origen real, consulte el registro nativo mediante `ParsedFile` y `InputRecord`. Las rutas registradas en la evidencia, como las rutas de ejecutables de Windows, no se modifican.

## Campos comunes de los eventos { #common-event-fields }

| Campo | Significado |
|---|---|
| `Timestamp` | Hora UTC en formato RFC 3339, escrita con nueve dígitos fraccionarios para que el orden sea coherente. Los ceros de relleno no añaden precisión. |
| `Comp` | Equipo asociado al registro de entrada. |
| `Artifact` | Identificador estable del artefacto, como `mft`, `prefetch`, `srum` o `hayabusa`. |
| `TimestampType` | Descripción del significado de la marca de tiempo, como `MFT SI Data modified`. |
| `User` | Cuenta o SID registrados, cuando el adaptador dispone de ellos; puede estar vacío. |
| `Path` | Archivo, clave del Registro u otro objeto afectado, según el artefacto. |
| `Message` | Descripción breve para un visor de líneas de tiempo. |
| `SourceFile` | Referencia al origen sin procesar relativa a las evidencias, o un identificador de origen relativo cuando se desconoce su raíz; en los demás casos, el archivo analizado como alternativa, tal como se describe más arriba. |
| `SourceField` | Campo analizado exacto o ubicación anidada de la que procede la marca de tiempo. |
| `RecordID` | Identidad del registro, propia de cada artefacto. No es necesariamente un Record ID de EVTX. |
| `Details` | Contexto estructurado que incluye la marca de tiempo original, la precisión, la base horaria, el papel de la hora, el archivo analizado, el registro o la ubicación de entrada y los metadatos de recuperación o de reglas disponibles. |

En CSV, `Details` es una celda JSON entre comillas. En JSONL es un objeto. El perfil de Timesketch cambia `Timestamp`, `TimestampType` y `Message` por `datetime`, `timestamp_desc` y `message`, respectivamente, y mantiene los demás campos de procedencia. Esos son los [tres campos obligatorios para importar en Timesketch](https://timesketch.org/guides/user/import-from-json-csv/).

`Details` contiene deliberadamente un contexto seleccionado, no los datos binarios completos ni todos los campos del registro nativo. Las cadenas, matrices y objetos largos se acotan y se marcan con `ProjectionOmitted`. Para examinar el registro nativo completo, consulte `Details.ParsedFile`, `InputRecord` y `Location`. En CSV, el número de registro cuenta los registros CSV posteriores al encabezado, no las líneas físicas dentro de celdas entre comillas. En JSONL es el número de línea; en JSON es el número de documento de nivel superior más una ubicación anidada.

## Qué significan las marcas de tiempo { #what-the-times-mean }

Una marca de tiempo es una evidencia sobre un objeto registrado. Por sí sola, no prueba una ejecución ni un ataque. Los adaptadores mantienen deliberadamente estas distinciones:

| Artefactos | Interpretación en la línea de tiempo |
|---|---|
| MFT | Las horas de modificación, acceso, cambio de metadatos y creación de SI y de FN se mantienen separadas; cada atributo FN con nombre sigue siendo distinto. Con `--sort`, se consolidan los campos SI repetidos en las filas de nombres de archivo. En ambos modos se excluyen las horas del archivo base que comparten las filas de ADS. Los nombres recuperados del espacio residual (slack) de los registros FILE conservan sus propios eventos de FN recuperado. |
| I30 | Metadatos de FN o del índice almacenados, con el contexto de espacio residual o de recuperación cuando está disponible. No son marcas de tiempo de la escritura en el índice del directorio. |
| Diario USN | Marca de tiempo del registro del diario y motivos de actualización. Un registro puede describir varios cambios; no registra necesariamente la hora exacta en que empezaron. |
| LogFile | Instantáneas almacenadas de marcas de tiempo en los datos de rehacer y deshacer (redo/undo), y metadatos de nombres de archivo. Rehacer y deshacer se mantienen separados. Los LSN y los desplazamientos dentro del archivo son identificadores y nunca se convierten en fechas. |
| Prefetch | Horas de ejecución registradas. Con `--sort`, se consolidan con la exportación de la línea de tiempo de Prefetch correspondiente. Los metadatos de creación de los volúmenes referenciados se etiquetan por separado. |
| Bases de datos SQLite de navegadores | Visitas registradas, metadatos de última visita de páginas recuperadas, inicio y fin de descargas, creación, acceso y caducidad de cookies, y primer y último uso del historial de formularios o sus registros de eliminación. El fin de una descarga no prueba por sí solo que esta se completara ni que se ejecutara nada; la caducidad de una cookie es un plazo configurado. El estado de recuperación, la confianza y el origen se conservan en el contexto. |
| WebCache de Internet Explorer y del Edge heredado | Acceso, creación y modificación, caducidad y sincronización de las entradas almacenadas, inicios de descarga, metadatos de cookies y acceso y limpieza de contenedores. Los datos de gestión interna de la caché no son automáticamente una evidencia de que un usuario abriera una página. |
| Archivos PST, OST, MSG y OFT de Outlook | Envío y entrega de mensajes; creación y modificación de carpetas, elementos y adjuntos; inicio, fin y recordatorios del calendario; inicio, vencimiento, finalización y recordatorios de tareas; y propiedades con un tipo de hora explícito. Las horas programadas no prueban que se celebrara una reunión, que saltara un recordatorio ni que se ejecutara un programa. Una hora de entrega almacenada no prueba que se leyera el mensaje. |
| Registro de acceso de usuarios (UAL) | Metadatos de primer y último acceso, inserción o creación en la base de datos, primera y última vez visto, y la hora de arranque del sistema operativo almacenada. El rol, la cuenta, la dirección de origen y los contadores diarios se mantienen como contexto. Las fechas y los recuentos diarios no se convierten en inicios de sesión ficticios a medianoche; las filas de identidad y DNS sin marca de tiempo siguen disponibles en el informe complementario. |
| Amcache | Últimas escrituras de las claves de inventario, contexto de la clave del volumen principal, fechas de instalación registradas y metadatos de enlace del PE. La presencia en el inventario o LinkDate no prueban por sí solos la ejecución. |
| ShimCache | Hora de modificación del archivo almacenada, metadatos de actualización de la caché y última escritura de la clave de la caché. Ninguna es automáticamente una marca de tiempo de ejecución. |
| SRUM | Marca de tiempo o intervalo de agregación del proveedor, inicio de la conexión, campos explícitos de intervalo de inicio y fin, y metadatos del identificador de la aplicación. Los bytes, la duración y otros contadores no son fechas. |
| LNK | Marcas de tiempo MAC del destino, creación del identificador de seguimiento y horas de los elementos del shell. Describen metadatos almacenados del destino, no necesariamente cuándo se abrió el acceso directo. |
| Jump Lists | Metadatos de último uso de las entradas, creación del identificador de seguimiento y horas del destino de los LNK incrustados. De forma predeterminada se omiten las horas de los archivos contenedores en el sistema de recopilación. |
| Cuarentena de Defender | Marca de tiempo de la entrada y horas almacenadas de creación, acceso y escritura del recurso. Las horas de extracción no son evidencia de actividad en el equipo investigado. |
| BAM, UserAssist, FeatureUsage | Campos de actividad registrados, cuando existen, con el contexto de la última escritura de la clave diferenciado. Un contador sin hora no genera un evento. |
| RecentDocs, MRU, MRU de Abrir/Guardar, ShellBags, Office | Horas registradas de uso, de confianza y de los elementos del shell, y contexto de la clave. La hora de la clave de una lista no es la hora de último uso de cada uno de sus elementos. |
| SAM y secretos LSA | Metadatos registrados de inicio de sesión, contraseña y caducidad de las cuentas o de establecimiento de secretos, y horas de las claves relacionadas. No sustituyen a los registros de eventos de autenticación. |
| Servicios, TaskCache, inicios automáticos del Registro, BCD | Metadatos de claves y de configuración, y horas de tareas registradas explícitamente. Las escrituras en claves de servicio no prueban por sí solas que el servicio se ejecutara. |
| Redes y USB | Campos de perfil y concesión de red, y de instalación, llegada y retirada de dispositivos, con el contexto de la clave del Registro. En la salida nativa de USB, las horas de MountPoints2 de cada usuario conservan como origen el hive de usuario correspondiente. |
| Información del sistema, programas instalados, AppCompat, historial de aplicaciones | Campos explícitos de instalación, apagado, inventario o actividad, y metadatos de las claves. Las matrices nativas de perfiles y versiones de Información del sistema conservan sus horas anidadas. |
| Directiva de auditoría, configuración de seguridad, MuiCache, Registro genérico | Horas de última escritura de las claves, cuando se proporcionan; los valores y contadores sin hora siguen disponibles en la entrada nativa. |
| Persistence Hunter | Última escritura de la clave que respalda el hallazgo y hora del cambio correlacionado. Un hallazgo no es en sí mismo una instalación con marca de tiempo ni un veredicto de que el sistema esté comprometido. |
| Hayabusa | Marca de tiempo del evento o de la alerta con la regla, el nivel, el canal, el ID de evento y el contexto de detalle disponible. Las coincidencias de varias reglas y las filas de correlación distintas se mantienen separadas. |

La exportación predeterminada conserva todas las proyecciones de marcas de tiempo, incluidos los metadatos repetidos. No realiza ninguna ordenación global ni consolidación de duplicados. Con `--sort`, las proyecciones duplicadas se consolidan, por lo que puede haber menos filas que en una exportación sin ordenar de las mismas entradas. En ambos modos se conservan las marcas de tiempo iguales que tienen significados o fuentes de evidencia diferentes. Para identificar un duplicado se tienen en cuenta el equipo, el origen, el artefacto, el registro, el significado de la hora y el estado de recuperación. Las filas consolidadas registran `DuplicateCount` y un número limitado de rutas y campos de origen alternativos. Nunca se fusionan orígenes distintos solo porque sus rutas y marcas de tiempo se parezcan. Cuando no se puede resolver el origen sin procesar, se conserva la identidad del registro de entrada para no fusionar capturas diferentes.

### Precisión, horas locales y supuestos { #precision-local-times-and-assumptions }

Los campos que se sabe que están en UTC y las marcas de tiempo con un desplazamiento explícito se normalizan a UTC. Se conservan los valores originales y su precisión registrada, incluidos los FILETIME de 100 ns. Es posible que un analizador de origen o un visor posterior ya hayan redondeado una hora; este comando no puede recuperar la precisión perdida.

Algunos campos de elementos del shell o FAT tienen semántica de hora local aunque un analizador previo los serialice con `Z`. La línea de tiempo conserva el valor analizado y el origen del campo; no puede recuperar la zona horaria del equipo investigado, que no consta, ni validar una conversión previa. Compruebe el artefacto sin procesar y la configuración del equipo antes de usar una hora de este tipo para una correlación precisa entre equipos.

Una marca de tiempo sin zona horaria (naive) para la que el artefacto no define ninguna zona se registra en los diagnósticos, en lugar de tratarse en silencio como UTC. Si la evidencia lo justifica, indique un desplazamiento fijo con signo:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Los valores que solo contienen una fecha también requieren esa opción. Su anclaje a medianoche se etiqueta como un supuesto con **precisión de día**; no es un evento observado a medianoche. No se aceptan nombres de zonas horarias ni reglas de horario de verano. Los valores ausentes, cero, no válidos o fuera de rango no generan eventos sintéticos en la fecha de época (epoch). Sí se conserva una fecha válida de 1970 registrada explícitamente.

Dos categorías opcionales se excluyen de forma predeterminada:

- `--include-collection-times`: metadatos de los archivos de origen LNK y Jump List en el sistema de archivos de recopilación o de análisis. Estos eventos se etiquetan como `collection_filesystem`, separados de la actividad del equipo investigado.
- `--include-interpreted-times`: valores QWORD genéricos del Registro que el lector del Registro representa como FILETIME. Se etiquetan como `interpreted_value` porque muchos de esos valores son en realidad contadores.

## Informes complementarios { #companion-reports }

Para `SuperTimeline.csv`, el comando también escribe:

- `SuperTimeline.csv-Untimed.jsonl`: ubicaciones de registros nativos sin una marca de tiempo de evento asignada que se pueda usar.
- `SuperTimeline.csv-Diagnostics.jsonl`: horas no válidas o ambiguas, metadatos compartidos omitidos intencionadamente y limitaciones de fidelidad que obligan a consultar el registro analizado original.
- `SuperTimeline.csv-Summary.json`: archivos de entrada, recuentos por artefacto, eventos filtrados, candidatos y consolidados, número de eventos de salida, hilos de trabajo, si la ordenación estaba habilitada, presupuesto, fragmentos y pasadas de ordenación, y tiempo transcurrido.

Los nombres de los informes complementarios incluyen el nombre completo del archivo de salida. Por ejemplo, `SuperTimeline.jsonl` tiene su propio `SuperTimeline.jsonl-Summary.json` y sus informes correspondientes de registros sin marca de tiempo y de diagnóstico. Las exportaciones CSV y JSONL con la misma raíz de nombre pueden coexistir; `-C` solo es necesario para sustituir un conjunto de resultados existente.

Las fechas válidas pendientes de un supuesto de zona horaria justificado incrementan `UnresolvedTimeZones`, no `InvalidTimestamps`. Ambos casos siguen siendo rastreables en los diagnósticos; no se inventa ninguna marca de tiempo.

Los registros con horas válidas fuera del intervalo solicitado se cuentan como filtrados, no como registros sin marca de tiempo. Los recuentos de eventos candidatos son anteriores a la consolidación de duplicados. `Sorted` y `DuplicateConsolidation` registran si se habilitó `--sort`. Sin esta opción, `DuplicateEvents`, `LargestSerializedSortRowBytes`, `InitialSortChunks` y `MergePasses` valen cero, y `SortMemoryBudgetMiB` es `null` porque no se asigna ningún búfer de ordenación. `Complete: true` significa que todas las entradas analizadas seleccionadas se procesaron correctamente; no significa que se recopilaran todos los artefactos, que todos los campos tengan marca de tiempo ni que todos los ataques puedan detectarse.

Las exportaciones CSV pueden aplanar metadatos anidados: los perfiles de Información del sistema, los puntos de montaje de usuario de USB, los elementos del shell y otros contextos estructurados pueden ser más completos en JSON/JSONL nativo. Use la entrada nativa cuando esos campos sean importantes. La línea de tiempo no puede reconstruir datos que una exportación CSV haya omitido. Los ID de artefacto no admitidos y los archivos de entrada mal formados provocan un error, en lugar de un resultado correcto con datos parciales.

## Rendimiento y seguridad de la salida { #performance-and-output-safety }

Las entradas se leen en flujo y se procesan en paralelo. De forma predeterminada, cada hilo de trabajo escribe un fragmento de eventos con formato; los fragmentos se concatenan en el orden de los archivos de entrada, sin ordenarlos ni volver a leer sus estructuras de eventos. El CSV tiene un único encabezado aunque se procesen varios archivos. El orden de salida es determinista sea cual sea el número de hilos de trabajo; no implica un orden cronológico.

`Input files` muestra el tamaño combinado de las entradas analizadas seleccionadas y las líneas de tiempo Hayabusa. `Results saved` muestra el tamaño del archivo de línea de tiempo guardado. Ambos utilizan las mismas unidades `KiB`/`MiB`/`GiB` legibles por humanos que `auto-parse`.

Con `--sort`, los búferes se vuelcan a un directorio temporal privado y, a continuación, unas pasadas de fusión acotadas producen una salida cronológica. La ordenación usa los segundos enteros completos y los nanosegundos, incluidas las fechas anteriores a 1970. El orden entre horas iguales es determinista sea cual sea el número de hilos de trabajo y el presupuesto de ordenación. Una barra de progreso avanza por cada evento procesado durante las fusiones intermedias y finales, incluidas las pasadas repetidas. Aparece en una terminal y se oculta cuando la salida se redirige.

`--memory-limit` solo se aplica con `--sort`: es el **presupuesto total del búfer de ordenación**, no un límite de RSS del proceso. Los registros de entrada, la sobrecarga de los objetos y cadenas JSON, los búferes CSV y los registros de fusión requieren memoria adicional. El número de hilos de trabajo se reduce con presupuestos pequeños. `--max-record-size` limita el tamaño de un registro de entrada analizado (16 MiB de forma predeterminada); una matriz o un informe JSON con formato legible cuenta como un único documento de entrada de nivel superior. Los documentos más grandes pueden necesitar un límite mayor o, preferiblemente, JSONL. La lectura anticipada del analizador CSV también cuenta para este límite.

El espacio temporal en disco debe dar cabida a los fragmentos de eventos con formato (modo predeterminado) o a los registros de ordenación expandidos y las pasadas de fusión (`--sort`), además de a los resultados finales. Ambos modos mantienen la preparación temporal de la salida y la protección que impide publicar resultados de entradas fallidas. Un registro puede generar muchos eventos, por lo que la línea de tiempo puede ser mucho mayor que la entrada. Filtre por intervalo de tiempo o por artefactos cuando proceda y elija un volumen temporal con suficiente espacio libre.

Ctrl+C cancela de forma cooperativa, elimina los directorios privados de ordenación y de preparación, y sale con el código `130`. En Linux y macOS, esto también se aplica a SIGTERM y SIGHUP. La cancelación se comprueba durante la lectura, la normalización, la fusión y la escritura de informes, antes de que empiece la publicación. Una terminación que no se pueda interceptar o un corte de energía todavía pueden dejar archivos temporales.

Las salidas existentes requieren `-C`. La línea de tiempo y sus informes complementarios no deben sobrescribir las entradas, los manifiestos, las evidencias locales referenciadas ni los directorios de evidencias protegidos que las contienen. Los errores de entrada o de ordenación dejan intactas las salidas existentes. Los archivos completados se preparan junto al destino y se sustituyen uno a uno, y la línea de tiempo se publica en último lugar. Esto **no** es una transacción que abarque los cuatro archivos: un error de E/S durante la publicación final puede dejar sustituidos algunos informes complementarios. Use un nombre base de salida nuevo en cada ejecución cuando importe conservar una exportación completa anterior.

## Más información { #further-details }

- [Importación CSV/JSONL en Timesketch](https://timesketch.org/guides/user/import-from-json-csv/)
- [Expansión de marcas de tiempo y CSV heredado de Plaso](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Salida y formato de Plaso](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Evidencia e interpretación](../overview/evidence-model.md)
- [Formatos de salida](../getting-started/output-formats.md)

Las entradas actuales de Juntei usan etiquetas abreviadas como `Comp`, `Filename`, `Args` y `Attr`. El CSV de la MFT también acepta los encabezados actuales `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` y `HasADS`, de modo que se conservan las generaciones de números de secuencia y las distinciones de DOS y ADS. Se siguen aceptando las etiquetas de exportación anteriores, y `SourceField` conserva la grafía de la entrada real. La entrada de Hayabusa conserva su propio campo `Computer`. Los metadatos del sector de arranque y de los descriptores de seguridad se conservan en el informe de registros sin marca de tiempo porque no aportan una marca de tiempo de evento.

## Analizadores de varias tablas más recientes { #newer-multi-table-parsers }

El JSON/JSONL nativo incluye el nombre de cada tabla. En un CSV independiente, los adaptadores de navegadores, WebCache y Outlook identifican la tabla por su esquema de columnas, de modo que cambiar el nombre del CSV o usar los sufijos de origen del análisis automático no altera la interpretación de las marcas de tiempo. UAL mantiene sus campos de marca de tiempo conocidos y explícitos; los contadores de acceso y el texto no relacionado que parezca una fecha no se interpretan como fechas por suposición.

Las filas de propiedades de Outlook solo se proyectan cuando su tipo declarado es un tipo de hora simple o multivalor. Una cadena que contiene una fecha sigue siendo una cadena. Los mensajes incrustados conservan sus ID de elemento y sus referencias al elemento principal, y las horas de extracción de los adjuntos no sustituyen a las marcas de tiempo almacenadas de los adjuntos. Las filas de archivos, destinatarios, contactos o propiedades sin marca de tiempo siguen siendo rastreables en la salida nativa y en el informe de registros sin marca de tiempo.

Los mismos esquemas de salida CSV y JSONL se aplican a estos analizadores; añadir un artefacto nuevo no añade cientos de columnas CSV. El contexto específico de cada artefacto permanece en `Details`.
