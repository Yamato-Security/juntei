# Índice de Windows Search { #windows-search-index }

## Qué registra el índice { #what-the-index-records }

Windows Search (el servicio `WSearch`) rastrea los perfiles de usuario, el menú Inicio, los buzones de Outlook, el
historial de Internet Explorer, los archivos sin conexión y todo lo demás que esté dentro de su ámbito, y guarda lo
que leyó en `Windows.edb`, una base de datos ESE. Dos partes de ella importan en una investigación:

- el **almacén de propiedades** (property store; `SystemIndex_PropertyStore`, o `SystemIndex_0A` en Windows 7),
  con una fila por elemento indizado: su ruta, nombre, tipo y tamaño, su propietario, las horas de creación,
  modificación y acceso que el indizador copió del archivo, cuándo lo leyó el indizador, el título y el autor de
  un documento, el comienzo de su texto, el nombre del equipo y cientos de propiedades más;
- las tablas del **recopilador** (gatherer; `SystemIndex_Gthr`, `SystemIndex_GthrPth`), con el registro propio del
  rastreador de cada elemento: su ruta URL, su hora de modificación cuando se puso en cola, el número y los
  indicadores del rastreo, y un contador de eliminaciones.

Un elemento permanece en el índice después de que el archivo, el mensaje o la página hayan desaparecido, hasta que
el rastreador advierte la eliminación. Por ello, el índice nombra archivos que ya no existen, con sus metadatos y un
extracto de su texto tal como estaban cuando se indizaron.

<span class="evidence-strength">Mejor conclusión:</span> el indizador leyó el elemento en la ruta registrada, con
los metadatos registrados, en `GatherTime`, en el equipo indicado. El índice demuestra que el archivo existía y qué
contenía. No demuestra que un usuario lo abriera.

## Ubicación y recopilación { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

De Windows 7 a Windows 11 21H2, y las versiones de Windows Server de los mismos años, guardan el índice en
`Windows.edb`. Windows 11 22H2 y posteriores lo guardan en bases de datos SQLite en la misma carpeta
(`Windows.db`, `Windows-gather.db`, `Windows-usn.db`); Juntei las encuentra y las nombra, y las notifica como el
formato más reciente no admitido.

Recopile toda la carpeta `Applications\Windows` desde una imagen de disco, una instantánea de volumen o tras detener
el servicio `WSearch`: la base de datos está abierta mientras el servicio se ejecuta, y sus cambios más recientes
solo están en los registros de transacciones `MSS*.log`, que Juntei no reproduce. Un índice copiado de un sistema en
ejecución queda sucio; la salida lo indica y lee lo que hay en disco. La entrada de directorio y `auto-parse` leen
cada `Windows.edb` a cualquier profundidad; utilice `-f` para un solo archivo.

## Precauciones para el analista { #analyst-cautions }

- Analiza una copia de evidencia inmutable. Juntei mapea el archivo en memoria para acceder a él eficientemente; no modifiques ni trunques la base de datos mientras se lee.

- `GatherTime` es cuándo leyó el indizador el elemento, no cuándo un usuario lo creó, modificó o abrió. La
  indización se ejecuta en segundo plano, a menudo de minutos a horas después de un cambio, y se detiene mientras
  el equipo está ocupado o funciona con batería.
- `DateCreated`, `DateModified` y `DateAccessed` son copias que el indizador tomó del archivo. Se actualizan
  cuando el elemento se indiza de nuevo, por lo que describen el archivo tal como estaba en `GatherTime`.
- `ItemPath` es la ruta de visualización. `ItemURL` y `GatherPath` son las URL del rastreador: `file:` para
  archivos (con el volumen como `[GUID]` en Windows 8 y posteriores), `iehistory://{SID}/` para el historial de
  Internet Explorer, `mapi://` para elementos de Outlook, `csc://{SID}/` para archivos sin conexión y
  `winrt://{SID}/` para el contenido de las aplicaciones de Windows.
- `User` es el perfil nombrado en la ruta del elemento (`C:\Users\<name>\…`). Indica dónde estaba el elemento,
  no quién lo creó; `FileOwner` es el propietario del archivo tal como lo vio el indizador.
- `AutoSummary` es el comienzo del texto del elemento. Puede citar un documento que desde entonces se eliminó o
  se cifró, y es un extracto truncado, no el documento.
- Una base de datos sucia puede contener páginas a las que sus páginas de rama todavía no llegan. Juntei sigue la
  cadena propia de las páginas hoja e indica en el resumen cuántas leyó de esa forma. Sus cambios más recientes
  siguen estando en los registros de transacciones.
- Un elemento en las tablas del recopilador sin fila en el almacén de propiedades (`InTables` es `Gthr`) se puso
  en cola o se rastreó, y sus propiedades aún no se escribieron o ya se eliminaron. Su ruta, su nombre y sus horas
  del recopilador siguen ahí.
- Windows 7 almacena las cadenas en una codificación propia, comprimida y ofuscada. Juntei la decodifica; un valor
  que no se decodifica se queda en hexadecimal, con una nota en `Problems`.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

CSV es el formato de archivo predeterminado. Con `-t csv`, `-t json` o `-t jsonl` y sin `-o`, los registros se
escriben en stdout. Sin ninguna de las dos opciones, solo se muestra un resumen. Añada `-C` para sobrescribir una salida
existente. La salida nunca se escribe sobre uno de los archivos de entrada, ni siquiera con `-C`.

Juntei escribe un registro por cada elemento indizado, uniendo el almacén de propiedades y las tablas del
recopilador por el identificador del elemento. El resumen cuenta las bases de datos por esquema, las sucias, los
elementos presentes en ambas tablas, solo en el recopilador y solo en el almacén de propiedades, las carpetas, los
elementos marcados como eliminados, los elementos por almacén, los elementos sin hora almacenada y los
problemas.

Los daños no detienen la ejecución. Una base de datos truncada conserva los elementos que contienen sus páginas
restantes, y el resumen indica lo que no se pudo leer. Un archivo llamado `Windows.edb` que no es un índice de
Windows Search se notifica y se omite. Las demás bases de datos se siguen leyendo.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `DocumentID` | El identificador del elemento en el índice: el `WorkID` del almacén de propiedades (`DocID` en Windows 7) y el `DocumentID` de la fila del recopilador. |
| `InTables` | `Gthr+PropertyStore`, `Gthr` (un elemento en cola o rastreado sin propiedades) o `PropertyStore`. |
| `ItemPath` | `System_ItemPathDisplay`: la ruta tal como la muestra Windows. Vacío cuando el almacén de propiedades no tiene fila. |
| `ItemURL` | `System_ItemUrl`: la URL del rastreador para el elemento (`file:`, `iehistory://`, `mapi://`, `csc://`, `winrt://`). |
| `GatherPath` | La ruta del recopilador, compuesta a partir del árbol de ámbitos y del nombre de archivo de la fila. |
| `Name` | `System_ItemName`, o el nombre de archivo de la fila del recopilador. |
| `ItemType`, `ItemTypeText` | `System_ItemType` (la extensión, o `Directory`) y su nombre para mostrar. |
| `Kind` | `System_Kind`: `document`, `folder`, `link`, `program`, `email`, `picture`, …; varios se unen con `\|`. |
| `MimeType` | `System_MIMEType`. |
| `Store` | `System_Search_Store`: `file`, `mapi` (Outlook), `iehistory`, `csc` (archivos sin conexión) o `winrt` (aplicaciones de Windows). |
| `IsFolder`, `IsDeleted` | `System_IsFolder` y `System_IsDeleted`, tal como están almacenados. |
| `DeletedCount` | El contador de eliminaciones de la fila del recopilador. |
| `Size` | `System_Size` en bytes. Vacío en las carpetas, que contienen un marcador de posición. |
| `FileAttributes` | `System_FileAttributes` como número (16 directorio, 32 archivo [archive], 2 oculto, …). |
| `FileOwner` | `System_FileOwner` (`DOMAIN\user`) tal como lo leyó el indizador. |
| `DateCreated`, `DateModified`, `DateAccessed` | Las horas del archivo tal como las copió el indizador (UTC). |
| `ItemDate` | `System_ItemDate`: la fecha propia del elemento, como la hora de envío de un mensaje o la fecha de captura de una foto; en un archivo, normalmente su hora de modificación con precisión de segundos. |
| `GatherTime` | `System_Search_GatherTime`: cuándo leyó el indizador el elemento (UTC). |
| `GatherLastModified` | El `LastModified` de la fila del recopilador: la hora de modificación del elemento cuando el rastreador lo puso en cola (UTC). |
| `Title`, `Author` | `System_Title` y `System_Author`. |
| `AutoSummary` | `System_Search_AutoSummary`: el comienzo del texto del elemento. |
| `CompName` | `System_ComputerName`: el equipo al que pertenece el índice. |
| `TargetURL`, `DateVisited` | La URL y la hora de visita de un elemento del historial de Internet Explorer (`System_Link_TargetUrl`, `System_Link_DateVisited`). |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | Un elemento del historial de actividades de Windows 10 (`System_Activity_*`, `System_ActivityHistory_*`). |
| `User`, `UserSource` | El perfil nombrado en la ruta del elemento, y de qué ruta se leyó (`ItemPath`, `ItemURL` o `GatherPath`). Vacío fuera de un perfil. |
| `ScopeID`, `SDID` | Los identificadores de ámbito y de descriptor de seguridad de la fila del recopilador. |
| `CrawlNumber` | El `CrawlNumberCrawled` de la fila del recopilador. |
| `GatherFlags` | Los `TransactionFlags` de la fila del recopilador. |
| `Priority` | La prioridad de rastreo de un elemento todavía en cola. ESE la marca como NULL en cuanto el elemento se rastrea, por lo que está vacía en la mayoría de los elementos. |
| `PropertyCount` | Cuántas propiedades contiene el almacén de propiedades para el elemento. |
| `OtherProperties` | Todas las demás propiedades, como `Name=Value` separadas por `; `. |
| `SourceFile` | La base de datos leída, relativa al directorio de entrada, o su nombre con `-f`. |
| `Problems` | Valores que no se decodificaron, horas que no son horas, filas duplicadas y daños. No lo suprima. |

JSON y JSONL conservan los mismos campos con valores tipados: los números siguen siendo números y los valores ausentes son
`null`. En lugar de `OtherProperties` tienen `Properties`, cada propiedad del elemento con su nombre, su
identificador numérico, su tipo de columna y su valor, y en el caso de una hora su número bruto de ticks FILETIME
junto a la hora que representa. `Gather` contiene las columnas de la fila del recopilador.

## Comportamiento en la línea de tiempo { #timeline-behavior }

`auto-parse` guarda los registros como `FileExistence/WindowsSearch.csv` (o `.json`/`.jsonl`) y los añade a la
línea de tiempo combinada de `-j`. `super-timeline` genera eventos a partir de `DateCreated`, `DateModified` y
`DateAccessed` (las horas del archivo tal como las registró Windows Search), de `GatherTime` (cuándo se indizó el
elemento), del `DateVisited` de un elemento del historial de Internet Explorer y de `ActivityStartTime` y
`ActivityEndTime` de una actividad. `GatherLastModified` solo se convierte en evento en un elemento para el que el
almacén de propiedades no contiene ninguna hora de modificación. Un elemento sin ninguna de estas horas permanece en
la salida analizada y se cuenta como sin hora.

## Estrategia de investigación { #investigation-strategy }

1. Filtre `Store` por `file` y busque en `ItemPath`, `Name` y `AutoSummary` los archivos, nombres y palabras del
   caso. El índice nombra archivos que desde entonces se eliminaron.
2. Lea `GatherTime` como la última hora en que el archivo existía con los metadatos registrados, y `DateModified`
   como su hora de modificación en ese momento.
3. Use `FileOwner` y `User` para asociar un archivo a una cuenta, y confírmelo después con `$MFT`, el Diario USN
   y `$Secure:$SDS`.
4. Para el correo electrónico (`Store` `mapi`) y el historial de Internet Explorer (`iehistory`), pivote hacia los
   archivos de Outlook y la WebCache para obtener los registros completos.
5. Compare el índice entre equipos: `CompName` nombra el equipo al que pertenece el índice, y un índice recopilado
   de una imagen nombra los archivos de esa imagen.

## Versiones probadas { #tested-versions }

Las pruebas usan índices públicos con resultados que otras herramientas leyeron de forma independiente:

- El índice de Windows 7 de plaso (`SystemIndex_0A`), cotejado con `esedbexport` de libyal, incluidas las
  codificaciones de cadenas de Windows 7.
- El índice de Windows 10 de DFIRArtifactMuseum, dejado sucio, cotejado con SIDR de Stroz Friedberg, y su índice
  de Windows 11 (revisión 200 del formato ESE), cotejado con `esedbexport`.
- El índice SQLite de Windows 11 22H2 de DFIRArtifactMuseum, notificado como el formato más reciente no admitido.
- Bases de datos truncadas, vacías o de otro tipo.

## Más información { #further-details }

- [Microsoft: información general sobre Windows Search](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb: formato de archivo de las bases de datos ESE](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb: Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg: SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft: código fuente de Extensible Storage Engine](https://github.com/microsoft/Extensible-Storage-Engine)
