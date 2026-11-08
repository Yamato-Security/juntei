# Navegadores web { #web-browsers }

## Qué son las bases de datos de los navegadores { #what-the-browser-databases-are }

Los navegadores guardan su historial, sus descargas, sus cookies y sus entradas de formularios en bases de datos SQLite
dentro del perfil de cada usuario. Los navegadores basados en Chromium (Chrome, Edge, Brave, Opera, Vivaldi) y los
basados en Firefox usan cada uno sus propios archivos y tablas, pero registran los mismos tipos de actividad:

- **Historial**: las páginas visitadas, cada visita con su hora, cómo llegó el navegador a la página (escribiendo la
  dirección, un enlace, una redirección, un marcador) y, en Chromium, cuánto tiempo estuvo la página en primer plano.
- **Descargas**: la URL de la que procede un archivo (y las redirecciones por el camino), dónde se guardó, su tamaño,
  cuándo empezó y terminó la descarga y si se completó.
- **Cookies**: los sitios que las establecieron, cuándo se establecieron, cuándo se enviaron por última vez y cuándo
  caducan.
- **Historial de formularios**: los valores escritos en formularios web y cuadros de búsqueda, cuántas veces, y la
  primera y la última vez.

La actividad del navegador suele ser el registro más directo de lo que hizo un usuario: la página de phishing a la que
accedió, la herramienta que descargó, el correo web que usó, los términos de búsqueda que escribió.

<span class="evidence-strength">Mejor conclusión:</span> el perfil del navegador registró una visita a la URL a la hora
registrada, o guardó la descarga en la ruta registrada; la base de datos no indica quién estaba frente al teclado.

## Ubicación y recopilación { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` es `Default`, `Profile 1`, `Profile 2`, etc. Chromium trasladó `Cookies` a `Network` en la versión 96; las
versiones anteriores lo guardan junto a `History`. Hasta la versión 25, Firefox guardaba sus descargas en
`downloads.sqlite`; las versiones posteriores las guardan en `places.sqlite`.

Recopile todos los archivos del perfil, y cada base de datos con los archivos que están junto a ella y comparten su
nombre: `places.sqlite-wal`, `cookies.sqlite-wal`, `formhistory.sqlite-wal`, `History-journal`.

!!! warning "Registros de escritura anticipada (WAL)"
    Firefox mantiene sus bases de datos en modo de registro de escritura anticipada (WAL): los cambios más recientes
    solo están en el archivo `-wal` hasta que el navegador los copia a la base de datos (un punto de control o
    checkpoint). Juntei lee cada base de datos con su archivo `-wal`, como lo haría el navegador. Sin él, faltan visitas,
    cookies y entradas de formularios recientes, y las entradas eliminadas en el WAL siguen leyéndose como activas.

## Entradas eliminadas { #deleted-entries }

Cuando un navegador elimina historial, las filas desaparecen de las tablas, pero sus registros pueden permanecer en el
espacio libre de la base de datos, en su lista de páginas libres (freelist) o en versiones anteriores de sus páginas: en
el archivo `-wal`, o en el propio archivo de la base de datos cuando un `-wal` ha reemplazado esas páginas. Juntei
recupera de ellos visitas, páginas, descargas y entradas de formularios eliminadas, como filas cuyo `Status` es
`Deleted`:

- `Confidence` indica con qué seguridad se asocia el registro a su tabla: `High` para un registro completo en el que cada
  valor tiene el tipo de su columna; `Medium` o `Low` para registros dañados o registros que encajan en varias tablas.
- `RecoveredFrom` indica dónde estaba el registro: la zona de la página (una celda, un freeblock, espacio no asignado,
  una página libre), qué página y qué versión de ella, y el desplazamiento del registro en la página. Los frames del WAL
  se numeran desde 0.
- `LostColumns` enumera las columnas que un registro dañado ya no contiene: sus campos quedan vacíos.

Los navegadores eliminan con `secure_delete` de SQLite: el espacio liberado en la base de datos se rellena con ceros. Lo
que se puede recuperar está entonces, sobre todo, en versiones anteriores de las páginas: las que todavía contiene el
archivo `-wal` y las copias que el propio archivo de la base de datos conserva de las páginas que un `-wal` reemplazó,
hasta el siguiente punto de control. Copie el archivo `-wal` junto con su base de datos, y hágalo antes de que el
navegador vuelva a ejecutarse.

## Precauciones para el analista { #analyst-cautions }

- Las horas están en UTC. Chromium y Firefox las registran con precisión de microsegundos; el historial de formularios
  de Chromium, con precisión de segundos.
- Una visita no significa que el usuario escribiera o hiciera clic: las redirecciones, los marcos (frames) y las páginas
  abiertas por otros programas también son visitas. Lea `Transition`, `Typed` y `Hidden`.
- Chromium cifra los valores de las cookies: `Value` está vacío para ellas. Juntei no los descifra.
- Una fila `Removed` del historial de formularios procede de la tabla `moz_deleted_formhistory` de Firefox, que Firefox
  para Android rellena para Sync. Firefox de escritorio normalmente la deja vacía.
- Una entrada eliminada con confianza `Low` puede ser un fragmento de otro registro: contrástela con las filas activas y
  con el resto de la evidencia.
- Que no haya filas eliminadas no significa que no se eliminara nada: el espacio rellenado con ceros y un punto de
  control no dejan nada que recuperar. Las cookies eliminadas todavía no se recuperan.
- El contenido determina la familia del navegador. Una ruta de perfil compatible precisa `Browser` como Chrome o Firefox; una ruta contradictoria no cambia la familia. En otro caso se usa `Chromium-based`, `Firefox-based` o `Safari` (`Unknown` si no se puede determinar). `User` y `Profile` proceden de la ruta y pueden perderse al copiar archivos.
- No se leen los diarios de reversión (`-journal`), ni el `WebCacheV01.dat` de Internet Explorer y del Edge heredado.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` busca los archivos por su nombre e identifica cada uno por sus tablas: se omite un archivo `History` de otro
programa. `-f` lee una base de datos sea cual sea su nombre.

## Tablas de salida { #output-tables }

CSV crea cuatro archivos, cada uno solo con su encabezado cuando no tiene filas. Las filas JSON y JSONL incluyen `Table`
(`History`, `Downloads`, `Cookies` o `FormHistory`) después de `SourceFile`.

| Archivo CSV | Contenido |
|---|---|
| `Browser-History.csv` | Visitas, activas y eliminadas; páginas eliminadas cuya URL no tiene ninguna página activa. |
| `Browser-Downloads.csv` | Descargas, activas y eliminadas. |
| `Browser-Cookies.csv` | Cookies. |
| `Browser-FormHistory.csv` | Entradas de formularios, activas y eliminadas; registros de eliminación de Firefox. |

## Campos { #fields }

Todas las tablas empiezan con estos campos:

| Campo | Interpretación |
|---|---|
| `SourceFile` | La base de datos de la que se leyó la fila. |
| `User` | El usuario de Windows en cuyo perfil está la ruta (la carpeta que sigue a `Users`). |
| `Browser` | Familia según el contenido, precisada por una ruta de perfil compatible. |
| `Profile` | El perfil del navegador: la carpeta de la base de datos. |
| `Status` | `Live`, `Deleted` (recuperada) o `Removed` (un registro de eliminación de Firefox). |

### Historial { #history }

| Campo | Interpretación |
|---|---|
| `RecordType` | `Visit`, o `Page` para una página eliminada cuyas visitas también pueden haber desaparecido. |
| `VisitTime` | Cuándo empezó la visita (UTC). |
| `URL`, `Title` | La página y su título, tal como se vieron por última vez. |
| `Transition` | Cómo llegó el navegador a la página. Chromium: el tipo principal y después sus calificadores (`LINK`, `TYPED`, `AUTO_BOOKMARK`, `FORM_SUBMIT`, `RELOAD`, … con `CHAIN_START`, `SERVER_REDIRECT`, `FROM_ADDRESS_BAR`, …); Firefox: `LINK`, `TYPED`, `BOOKMARK`, `EMBED`, `REDIRECT_PERMANENT`, `REDIRECT_TEMPORARY`, `DOWNLOAD`, `FRAMED_LINK`, `RELOAD`. |
| `VisitDuration` | Segundos que la página estuvo en primer plano (Chromium; 0 cuando no se midió). |
| `VisitCount` | Visitas totales a la página. |
| `Typed`, `TypedCount` | Si la URL se escribió alguna vez, y cuántas veces (Chromium). |
| `Hidden` | Oculta en la lista del historial: submarcos, orígenes de redirecciones, errores. |
| `Frecency` | La clasificación de la página por Firefox para las sugerencias de la barra de direcciones. |
| `VisitID`, `FromVisitID` | El id de fila de la visita y el de la visita de la que procede (la página de referencia, el origen de la redirección). |
| `LastVisitTime` | La última visita de una página eliminada (UTC). |
| `PageSource` | De dónde proceden la URL y el título de una visita eliminada: `Live` (una página activa), `Recovered` (una página eliminada) o `NotFound`. |
| `Confidence`, `RecoveredFrom`, `LostColumns` | Cómo se recuperó una entrada eliminada; consulte [Entradas eliminadas](#deleted-entries). |

### Descargas { #downloads }

| Campo | Interpretación |
|---|---|
| `StartTime`, `EndTime` | Cuándo empezó y terminó la descarga (UTC). |
| `URL`, `RedirectChain` | De dónde procede el archivo, y cada URL por el camino (Chromium; separadas por ` -> ` en CSV). |
| `TargetPath`, `CurrentPath` | Dónde se guardó el archivo (un URI `file://` en Firefox), y dónde se escribió durante la descarga. |
| `ReceivedBytes`, `TotalBytes` | Bytes recibidos, y el tamaño del archivo. |
| `State` | `Complete`, `InProgress`, `Cancelled`, `Interrupted`, `Paused`, `Blocked` o `Dirty` (bloqueada como malware). |
| `DangerType` | El veredicto de Chromium sobre el archivo: 0 no peligroso, 1 tipo de archivo peligroso, 2 URL peligrosa, 3 contenido peligroso, 4 posiblemente peligroso, 5 poco común, 6 conservado por el usuario, 7 host peligroso, 8 potencialmente no deseado, … |
| `InterruptReason` | Por qué se detuvo una descarga interrumpida (Chromium; 0 si no hay motivo). |
| `Referrer`, `TabURL` | La página de referencia, y la página de la pestaña que inició la descarga. |
| `MimeType`, `Opened` | El tipo del archivo tal como lo sirvió el servidor, y si se abrió desde el navegador (Chromium). |
| `FileDeletedByUser` | Si el usuario eliminó del disco el archivo descargado desde la lista de descargas del navegador (Firefox). |
| `DownloadID` | El id de fila de la descarga. |

### Cookies { #cookies }

| Campo | Interpretación |
|---|---|
| `Created`, `LastAccessed`, `Expires` | Cuándo se estableció la cookie, cuándo se envió por última vez y cuándo caduca (UTC; `Expires` vacío para una cookie de sesión). |
| `Host`, `Name`, `Path` | El sitio para el que es la cookie, su nombre y su ruta. |
| `Value` | El valor tal como está almacenado: vacío cuando Chromium lo cifró. |
| `Secure`, `HttpOnly`, `Persistent` | Enviada solo por HTTPS, oculta para los scripts, conservada entre sesiones (Chromium). |
| `GoogleAnalytics` | Los campos decodificados de una cookie de Google Analytics (`__utma`, `__utmb`, `__utmt`, `__utmz`): horas de visita, sesiones, y el origen de la campaña y los términos de búsqueda. |
| `RowID` | El id de fila de la cookie. |

### Historial de formularios { #form-history }

| Campo | Interpretación |
|---|---|
| `FirstUsed`, `LastUsed` | Cuándo se introdujo el valor por primera y por última vez (UTC). |
| `RemovedTime` | Cuándo se eliminó la entrada según un registro de eliminación de Firefox (UTC). |
| `FieldName`, `Value` | El campo del formulario (`email`, `q`, la barra de búsqueda de Firefox: `searchbar-history`) y lo que se escribió. |
| `Count` | Cuántas veces se introdujo el valor. |
| `GUID` | El identificador de la entrada en Firefox, que nombran los registros de eliminación. |
| `Sources` | Desde dónde guardó Firefox la entrada, en las versiones que lo conservan (separados por una barra vertical en CSV). |
| `RowID` | ID de fila; vacío/null si se perdió durante la recuperación. Un cero realmente registrado permanece como `0`. |

## Análisis automático { #automatic-parsing }

`auto-parse` escribe las tablas en `BrowserActivity` como `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` y `Browser-FormHistory.csv` (un único archivo `Browser.json` o `Browser.jsonl` con `-t json` o
`-t jsonl`), junto con `BrowserActivity/Browser-logs.txt`. La línea de tiempo combinada `-j` incluye todas las filas, con
`Comp` y el identificador de `Artifact` `browser`. `-Z` convierte `VisitTime`, `LastVisitTime`, `StartTime`, `EndTime`,
`Created`, `LastAccessed`, `Expires`, `FirstUsed`, `LastUsed` y `RemovedTime`.

## Más información { #further-details }

- [Chromium — tipos de transición de página](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — tipos de peligro de las descargas](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — base de datos Places](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — registro de escritura anticipada](https://www.sqlite.org/wal.html)
- [Plaso — analizadores SQLite de navegadores](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
