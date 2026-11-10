# Súper línea de tiempo { #super-timeline }

`super-timeline` convierte los artefactos analizados en eventos con un esquema común y los ordena por tiempo. Un registro Prefetch puede contener varios tiempos de ejecución; un registro MFT puede contener ocho o más marcas de tiempo de archivos. Cada uno se convierte en un evento etiquetado por separado. Esto hace que sea más fácil comparar evidencia de diferentes fuentes sin perder lo que realmente significa cada marca de tiempo.

Comience con los resultados de [`auto-parse`](auto-parse.md), en lugar del directorio de evidencia original:

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

Conserve los archivos analizados, los manifiestos y las pruebas originales. La línea de tiempo es una proyección para el análisis; no reemplaza esos registros ni crea un archivo de almacenamiento nativo de Plaso.

## Uso { #usage }

```bash
# Native CSV (default)
juntei super-timeline -d parsed/ -o SuperTimeline.csv

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
  --memory-limit 64 -T 4 --temp-dir scratch/
```

`--hayabusa FILE` se puede repetir o utilizar solo. Importa marcas de tiempo de eventos/alertas existentes y contexto de reglas sin ejecutar Hayabusa. Se admiten eventos informativos cuyo `Details` sea una cadena. `--artifact mft,prefetch` restringe la proyección a identificadores de artefactos seleccionados. Los límites de tiempo son inclusivos y requieren una compensación UTC o `Z` explícita.

### Elegir una entrada { #choosing-an-input }

| Entrada | como se identifica |
|---|---|
| `-d DIR` | `manifest.json` proporciona tareas primarias exitosas, ID de artefactos, computadoras y archivos de salida exactos. Las tareas secundarias `auto-parse -j` se excluyen para evitar la importación de ambas copias. |
| `-f FILE` | JSON/JSONL nativo utiliza los campos `Artifact` y `Comp` integrados. Las exportaciones sin etiquetar necesitan `--input-artifact`; El CSV independiente siempre lo requiere. |
| `--computer NAME` | Respaldo cuando el registro de entrada no tiene nombre de computadora; de lo contrario, una computadora desconocida es explícitamente `Unknown`. |
| `--hayabusa FILE` | JSONL con marcas de tiempo Hayabusa, metadatos de reglas, computadora y, cuando esté presente, `EvtxFile`. |

Los manifiestos actuales admiten la reubicación del directorio analizado: las rutas de salida absolutas registradas se resuelven en el nuevo directorio. Las entradas deben permanecer dentro de él, incluidos los objetivos de enlaces simbólicos. Se admiten los directorios heredados por artefacto Amcache, SRUM y Jump Lists; Los directorios de categorías necesitan `OutputFiles` explícito.

Se admiten manifiestos actuales con rutas relativas a la salida y exportaciones aplanadas de varias tablas. El comando reconoce todos los artefactos programados mediante análisis automático, incluido el navegador más nuevo, las tablas WebCache, Outlook y UAL.

Cuando un manifiesto identifica una imagen VHDX o E01, las referencias de origen incluyen la imagen y la ruta interna, y el nombre de la imagen se conserva en el contexto del evento. El espacio de trabajo de preparación temporal no es necesario para leer los resultados analizados.

Se toma una fuente sin procesar del registro o de un comando manifiesto inequívoco. Las tablas SRUM CSV se pueden asignar a una base de datos solo cuando el host tiene un SRUDB.dat recopilado de forma única e identificable. De lo contrario, `SourceFile` apunta al archivo analizado y `Details.RawSourceAvailable` es `false`. No confunda ese recurso alternativo con un camino de evidencia cruda.

## Campos de eventos comunes { #common-event-fields }

| campo | Significado |
|---|---|
| `Timestamp` | UTC RFC 3339, escrito con nueve dígitos fraccionarios para realizar pedidos coherentes. El relleno no añade precisión. |
| `Comp` | Computadora asociada al registro de entrada. |
| `Artifact` | Identificador de artefacto estable como `mft`, `prefetch`, `srum` o `hayabusa`. |
| `TimestampType` | Descripción del significado de la marca de tiempo, como `MFT SI Data modified`. |
| `User` | Cuenta registrada o SID cuando el adaptador tenga una; puede estar vacío. |
| `Path` | Archivo afectado, clave de registro u otro objeto específico del artefacto. |
| `Message` | Breve descripción para un visor de línea de tiempo. |
| `SourceFile` | Ruta de evidencia cruda cuando sea resoluble; de lo contrario, se utilizará el recurso alternativo del archivo analizado descrito anteriormente. |
| `SourceField` | Campo analizado exacto o ubicación anidada utilizada para la marca de tiempo. |
| `RecordID` | Identidad de registro específica del artefacto. No es necesariamente un Registro EVTX ID. |
| `Details` | Contexto estructurado que incluye la marca de tiempo original, la precisión, la base de tiempo, la función de tiempo, el archivo analizado, el registro/ubicación de entrada y los metadatos de regla/recuperación disponibles. |

En CSV, `Details` es una celda JSON cotizada. En JSONL es un objeto. El perfil Timesketch cambia `Timestamp`, `TimestampType` y `Message` a `datetime`, `timestamp_desc` y `message`, respectivamente, manteniendo los otros campos de procedencia. Esos son los [tres campos de importación de Timesketch requeridos](https://timesketch.org/guides/user/import-from-json-csv/).

`Details` contiene deliberadamente contexto seleccionado, en lugar de datos binarios completos o cada campo de registro nativo. Las cadenas largas, matrices y objetos están delimitados y marcados como `ProjectionOmitted`. Busque `Details.ParsedFile`, `InputRecord` y `Location` para inspeccionar el registro nativo completo. Para CSV, el número de registro cuenta los registros CSV después del encabezado, no las líneas físicas dentro de las celdas entrecomilladas. Para JSONL es el número de línea; para JSON es el número de documento de nivel superior más una ubicación anidada.

## lo que significan los tiempos { #what-the-times-mean }

Una marca de tiempo es evidencia sobre un objeto registrado. No es, por sí solo, prueba de ejecución o de ataque. Los adaptadores mantienen intencionadamente estas distinciones:

| Artefactos | Interpretación de la línea de tiempo |
|---|---|
| MFT | Separe los tiempos de modificación, acceso, cambio de metadatos y creación de SI y FN; Los atributos denominados FN siguen siendo distintos. Los campos SI repetidos en las filas de nombres de archivos están consolidados. Se excluyen los tiempos del archivo base compartido de las filas ADS. Los nombres flojos del registro FILE recuperado mantienen eventos FN recuperados separados. |
| I30 | Metadatos de índice/FN almacenados, con contexto de holgura/recuperación cuando esté disponible. Estas no son marcas de tiempo de la escritura del índice del directorio. |
| USN Diario | Marca de tiempo del registro del diario y motivos de actualización. Un registro puede describir varios cambios; no necesariamente registra su hora de inicio precisa. |
| LogFile | Instantáneas de marca de tiempo rehacer/deshacer almacenadas y metadatos de nombre de archivo. Rehacer y deshacer permanecen separados. Los LSN y las compensaciones de archivos son identificadores, nunca convertidos en fechas. |
| Prefetch | Tiempos de ejecución registrados, consolidados con la exportación de línea de tiempo Prefetch coincidente. Los metadatos de creación de volúmenes a los que se hace referencia están etiquetados por separado. |
| Bases de datos SQLite del navegador | Visitas registradas, metadatos de la última visita de la página recuperada, inicio/fin de descarga, creación/acceso/caducidad de cookies y lápidas de eliminación o primer/último uso del historial de formularios. El final de una descarga no prueba por sí solo el éxito o la ejecución; la caducidad de una cookie es una fecha límite configurada. El estado de recuperación, la confianza y el origen permanecen en contexto. |
| Internet Explorer/Edge heredado WebCache | Acceso a entradas almacenadas, creación/modificación, caducidad y sincronización, inicios de descarga, metadatos de cookies y acceso/eliminación de contenedores. La contabilidad de caché no es una prueba automática de que un usuario abre una página. |
| Outlook PST, OST, MSG y OFT | Envío/entrega de mensajes; creación/modificación de carpetas, elementos y archivos adjuntos; inicio/final/recordatorios del calendario; inicio/vencimiento/finalización de tarea/recordatorios; y propiedades de tiempo escritas explícitamente. Los horarios programados no prueban que se haya producido una reunión, que se haya activado un recordatorio o que se haya ejecutado un programa. Un tiempo de entrega almacenado no prueba que el mensaje haya sido leído. |
| Registro de acceso de usuario (UAL) | Metadatos de primer/último acceso, inserción/creación de bases de datos, tiempo de inicio del sistema operativo visto y almacenado por primera/última vez. La función, la cuenta, la dirección de origen y los contadores diarios siguen siendo contexto. Las fechas/conteos diarios no son inicios de sesión de medianoche inventados; La identidad sin tiempo y las filas DNS permanecen disponibles en el informe complementario. |
| Amcache | Últimas escrituras de clave de inventario, contexto de clave de volumen principal, fechas de instalación registradas y metadatos de enlace PE. La presencia de inventario o LinkDate por sí sola no prueba la ejecución. |
| ShimCache | Modificación de archivos almacenados, metadatos de actualización de caché y última escritura de clave de caché. Ninguno es automáticamente una marca de tiempo de ejecución. |
| SRUM | Marca de tiempo/depósito del proveedor, inicio de la conexión, campos explícitos de intervalo de inicio/finalización y metadatos del identificador de la aplicación. Los bytes, la duración y otros contadores no son fechas. |
| LNK | Destino MAC marcas de tiempo, creación de identificadores de seguimiento y tiempos de elementos de shell. Describen los metadatos de destino almacenados, no necesariamente cuándo se abrió el acceso directo. |
| Jump Lists | Ingrese metadatos de último uso, creación de identificadores de seguimiento y tiempos objetivo LNK integrados. El valor predeterminado omite los tiempos de los archivos del contenedor del lado de la colección. |
| Defender cuarentena | Marca de tiempo de entrada y tiempos de creación/acceso/escritura de recursos almacenados. Los tiempos de extracción no son evidencia de actividad en los terminales. |
| BAM, UserAssist, FeatureUsage | Los campos de actividad registrados estaban presentes, con el contexto clave de última escritura mantenido distinto. Un contador sin tiempo no crea un evento. |
| RecentDocs, MRU, Abrir/Guardar MRU, ShellBags, Oficina | Tiempos registrados de uso/confianza/elementos shell y contexto clave. El momento clave de una lista no es el momento del último uso individual de cada elemento de la lista. |
| Secretos SAM y LSA | Inicio de sesión/contraseña/vencimiento de cuenta registrados o metadatos de conjunto secreto y horas clave relacionadas. Estos no reemplazan los registros de eventos de autenticación. |
| Servicios, TaskCache, persistencia del registro, BCD | Metadatos clave/de configuración y tiempos de tareas registrados explícitamente. Las escrituras de claves de servicio por sí solas no prueban la ejecución del servicio. |
| Redes y USB | Campos de perfil/arrendamiento de red e instalación/llegada/eliminación de dispositivo, con contexto de clave de registro. El usuario nativo USB MountPoints2 conserva la fuente de subárbol de usuario correspondiente. |
| Información del sistema, programas instalados, AppCompat, historial de aplicaciones | Campos explícitos de instalación, apagado, inventario/actividad y metadatos clave. Los conjuntos de versiones/perfiles de información del sistema nativo conservan sus tiempos anidados. |
| Política de auditoría, configuración de seguridad, MuiCache, registro genérico | Horas clave de última escritura cuando se suministran; Los valores/contadores sin tiempo permanecen disponibles en la entrada nativa. |
| Persistence Hunter | Última escritura de la clave de soporte y tiempo de cambio correlacionado. Un hallazgo no es en sí mismo una instalación con marca de tiempo ni un veredicto de compromiso. |
| Hayabusa | Marca de tiempo de evento/alerta con regla, nivel, canal, evento ID y contexto detallado disponible. Varias reglas y distintas filas de correlación permanecen separadas. |

Se conservan varias marcas de tiempo iguales con diferentes significados o fuentes de evidencia. La identidad duplicada incluye la computadora, la fuente, el artefacto, el registro, el significado del tiempo y el estado de recuperación. Las filas consolidadas registran `DuplicateCount` y rutas alternativas/campos de origen delimitados. Las fuentes distintas nunca se fusionan simplemente porque sus rutas y marcas de tiempo se parecen. Cuando la fuente sin procesar no se puede resolver, la identidad del registro de entrada se conserva para evitar colapsar diferentes capturas.

### Precisión, horas locales y suposiciones. { #precision-local-times-and-assumptions }

Los campos UTC conocidos y las marcas de tiempo con compensaciones explícitas están normalizados a UTC. Se conservan los valores originales y su precisión registrada, incluidos 100 ns FILETIMEs. Es posible que un analizador de origen o un visor descendente ya haya redondeado un tiempo; este comando no puede restaurar la precisión perdida.

Algunos campos shell-item/FAT tienen semántica de hora local incluso cuando un analizador ascendente los serializa con `Z`. La línea de tiempo conserva el valor analizado y el origen del campo; no puede recuperar la zona horaria del punto final que falta ni validar una conversión ascendente. Verifique el artefacto sin procesar y la configuración del punto final antes de utilizar dicho tiempo para una correlación precisa entre hosts.

Se diagnostica una marca de tiempo ingenua sin zona horaria definida por artefactos, en lugar de tratarse silenciosamente como UTC. Si la evidencia lo justifica, proporcione una compensación fija firmada:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Los valores de solo fecha también requieren esa opción. Su ancla de medianoche está etiquetada como una suposición con **precisión del día**; no es un evento observado a medianoche. No se aceptan nombres de zonas horarias ni reglas de horario de verano. Los valores faltantes, cero, no válidos y fuera de rango no producen eventos de época sintéticos. Se conserva una fecha 1970 válida registrada explícitamente.

Se excluyen dos categorías opcionales de forma predeterminada:

- `--include-collection-times`: LNK/Metadatos del archivo fuente de Jump List en el sistema de archivos de recopilación/análisis. Estos eventos están etiquetados como `collection_filesystem`, separados de la actividad del terminal.
- `--include-interpreted-times`: valores genéricos del registro QWORD representados como FILETIME por el lector del registro. Están etiquetados como `interpreted_value` porque muchos de esos valores son en realidad contadores.

## Informes complementarios { #companion-reports }

Para `SuperTimeline.csv` o `SuperTimeline.jsonl`, el comando también escribe:

- `SuperTimeline-Untimed.jsonl`: ubicaciones de registros nativos sin una marca de tiempo de evento mapeada utilizable.
- `SuperTimeline-Diagnostics.jsonl`: tiempos no válidos/ambiguos, metadatos compartidos omitidos intencionalmente y limitaciones de fidelidad que requieren el registro analizado original.
- `SuperTimeline-Summary.json`: archivos de entrada, recuentos por artefacto, eventos filtrados/candidatos/consolidados, recuento de salida, trabajadores, clasificación de presupuesto/fragmentos/pases y tiempo transcurrido.

Los registros con tiempos válidos fuera del rango solicitado se cuentan como filtrados, no sin tiempo. Los recuentos de eventos candidatos preceden a la consolidación de duplicados. `Complete: true` significa que todas las entradas analizadas seleccionadas se procesaron correctamente; no significa que todos los artefactos hayan sido recolectados, que cada campo tenga una marca de tiempo o que cada ataque sea detectable.

Las exportaciones de CSV pueden aplanar los metadatos anidados: los perfiles de información del sistema, los puntos de montaje del usuario USB, los elementos del shell y otros contextos estructurados pueden ser más ricos en JSON/JSONL nativo. Utilice entradas nativas cuando esos campos sean importantes. La línea de tiempo no puede reconstruir los datos omitidos por una exportación CSV. Los ID de artefactos no admitidos y los archivos de entrada con formato incorrecto provocan un error en lugar de un éxito que contiene resultados parciales.

## Rendimiento y seguridad de salida { #performance-and-output-safety }

Las entradas se transmiten y procesan en paralelo. Los buffers de clasificación se extienden a un directorio temporal privado, luego los pases de fusión limitados producen una salida cronológica determinista. La clasificación utiliza segundos y nanosegundos enteros, incluidas las fechas anteriores a 1970. La ordenación en tiempos iguales es determinista en todos los recuentos de trabajadores y presupuestos de clasificación.

`--memory-limit` es el **presupuesto de búfer de clasificación** agregado, no un límite de proceso RSS. Los registros de entrada, la sobrecarga de cadena/objeto JSON, los buffers CSV y los registros de combinación requieren memoria adicional. El número de trabajadores se reduce para presupuestos pequeños. `--max-record-size` limita un registro de entrada analizado (16 MiB predeterminado); una bonita matriz/informe JSON cuenta como un documento de entrada de nivel superior. Los documentos más grandes pueden necesitar un límite más alto o, preferiblemente, JSONL. La lectura anticipada del analizador CSV también contribuye a este límite.

El espacio temporal en disco debe acomodar los eventos expandidos y los pases de fusión, además de los resultados finales. Un registro puede generar muchos eventos, por lo que la línea de tiempo puede ser mucho mayor que la entrada. Filtre el rango de tiempo/artefactos cuando sea apropiado y seleccione un volumen temporal con espacio libre adecuado.

Las salidas existentes requieren `-C`. La línea de tiempo y sus acompañantes no deben sobrescribir entradas, manifiestos, evidencia local referenciada o sus directorios de evidencia protegidos. Los errores de entrada o de clasificación dejan intactas las salidas existentes. Los archivos completos se colocan junto al destino y se reemplazan individualmente, publicándose la línea de tiempo al final. Esto **no** es una transacción entre los cuatro archivos: una falla de E/S durante la publicación final puede dejar algunos informes complementarios reemplazados. Mantenga un nuevo nombre base de salida para cada ejecución cuando sea importante conservar una exportación completa anterior.

## Más detalles { #further-details }

- [Timesketch CSV/JSONL importar](https://timesketch.org/guides/user/import-from-json-csv/)
- [Expansión de marca de tiempo de Plaso y legado CSV](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Salida y formateo de Plaso](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Evidencia e interpretación](../overview/evidence-model.md)
- [Formatos de salida](../getting-started/output-formats.md)

Las entradas Juntei actuales utilizan etiquetas abreviadas como `Comp`, `Filename`, `Args` y `Attr`. MFT CSV también acepta los actuales `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` y Encabezados `HasADS`, manteniendo intactas las generaciones de números de secuencia y las distinciones DOS/ADS. Se siguen aceptando etiquetas exportadas más antiguas y `SourceField` conserva la ortografía en la entrada real. La entrada Hayabusa conserva su propio campo `Computer`. Los metadatos del sector de arranque y del descriptor de seguridad se conservan en el informe sin tiempo porque no proporciona una marca de tiempo del evento.

## Analizadores de tablas múltiples más nuevos { #newer-multi-table-parsers }

JSON/JSONL nativo lleva cada nombre de tabla. Para un CSV independiente, el navegador, WebCache y los adaptadores de Outlook identifican su tabla a partir del esquema de columnas, por lo que cambiar el nombre de CSV o utilizar los sufijos de origen del análisis automático no cambia la interpretación de la marca de tiempo. UAL mantiene sus campos de marca de tiempo conocidos explícitos; Los contadores de acceso y el texto que busca fechas no relacionados no se adivinan como fechas.

Las filas de propiedades de Outlook se proyectan solo cuando su tipo declarado es de tiempo o de tiempo múltiple. Una cadena que contiene una fecha sigue siendo una cadena. Los mensajes incrustados mantienen sus ID de elementos y referencias principales, y los tiempos de extracción de archivos adjuntos no sustituyen a las marcas de tiempo de archivos adjuntos almacenados. Las filas de archivos, destinatarios, contactos o propiedades sin tiempo permanecen rastreables en la salida nativa y en el informe sin tiempo.

Los mismos esquemas de salida CSV y JSONL se aplican a estos analizadores; agregar un nuevo artefacto no agrega cientos de columnas CSV. El contexto específico del artefacto permanece en `Details`.
