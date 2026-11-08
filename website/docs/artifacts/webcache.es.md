# Internet Explorer y Edge heredado { #internet-explorer-and-legacy-edge }

## Qué es la base de datos WebCache { #what-the-webcache-database-is }

`WebCacheV01.dat` es la base de datos de las funciones de Internet de Windows (WinINet) en el perfil de cada usuario,
una base de datos ESE. Internet Explorer 10 y 11 guardan en ella su historial, sus descargas, su caché y sus cookies, y
también el Edge heredado, el Edge de Windows 10 anterior a que Edge pasara a basarse en Chromium. Cada tipo de registro
es un contenedor, una tabla propia:

- **Historial** (`History`, y `MSHist01…` para un día o una semana): las URL visitadas, cada una con la cuenta para la
  que se registró, cuándo se visitó por última vez y cuántas veces. El Explorador de archivos también registra allí los
  archivos locales que abre, como URL `file:///`.
- **Descargas** (`iedownload`): la URL de la que procede un archivo, dónde se guardó y dónde se escribió durante la
  descarga, su tamaño, cuándo empezó la descarga, la página de referencia y el tipo MIME.
- **Cookies**: en Windows 10 y versiones posteriores, las propias cookies (tablas `CookieEntryEx`), con su host,
  nombre, valor y horas; antes, los archivos de cookies que las contenían.
- **Caché** (`Content`): los archivos descargados para mostrar páginas, cada uno con su URL, dónde está la copia en el
  disco y los encabezados de la respuesta HTTP. Otros programas que usan WinINet también almacenan aquí en caché sus
  descargas.
- Almacenamiento web (`DOMStore`), listas de compatibilidad (`iecompat`) y otros.

La base de datos sobrevive a los navegadores: Internet Explorer está retirado y el Edge basado en Chromium mantiene sus
propias bases de datos SQLite (consulte [Navegadores web](browser.md)), pero la base de datos sigue estando en todos los
perfiles, con lo que se registró antes y lo que el Explorador de archivos y otros programas siguen registrando.

<span class="evidence-strength">Mejor conclusión:</span> el perfil del usuario registró la URL, la descarga o el
archivo almacenado en caché a la hora registrada; la base de datos no indica quién estaba frente al teclado, ni que se
abriera un archivo almacenado en caché.

## Ubicación y recopilación { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

Los archivos que nombran los contenedores (archivos en caché, archivos de cookies) están en sus propias carpetas:
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\`, `INetCookies\`, `History\`, y en `%LOCALAPPDATA%\Packages\` para las
aplicaciones.

Recopile toda la carpeta `WebCache`. Windows mantiene abierta la base de datos mientras el usuario tiene la sesión
iniciada, por lo que una copia de un sistema en ejecución suele estar sucia (dirty): sus cambios más recientes solo
están en los registros de transacciones `V01*.log`.

!!! warning "Registros de transacciones"
    Juntei no reproduce los registros de transacciones de ESE. Una base de datos que no se cerró correctamente se cuenta
    en el resumen como sucia y se notifica como un problema: faltan los cambios que todavía solo están en sus registros.
    Para incluirlos, reproduzca primero los registros en una copia de la carpeta, con el comando `esentutl /r V01 /d` de
    Windows ejecutado en esa copia.

## Precauciones para el analista { #analyst-cautions }

- Las horas están en UTC, con precisión de 100 nanosegundos, salvo el `ModifiedTime` del contenedor del historial de
  un día o de una semana: es la hora local del sistema, escrita sin desplazamiento, y `-Z` la deja tal cual.
  Conviértala con la zona horaria que tenía el sistema de origen en ese momento. Su diferencia con `AccessedTime`
  suele ser el desplazamiento de esa zona horaria, pero no siempre: ambas pueden ser momentos distintos.
- Una entrada del historial es una URL, no una visita: `AccessedTime` es su última visita y `AccessCount`, cuántas
  hubo. El contenedor `History` guarda una entrada por URL; el contenedor de un día o de una semana guarda las de ese
  periodo, y su nombre indica su primer día y el día siguiente al último (`MSHist012026110120261102`: 1 de noviembre de
  2026).
- Las URL `file:///` del historial son archivos locales abiertos desde el Explorador de archivos y otros programas, no
  páginas web.
- `Account` es la cuenta para la que se registró una entrada (`Visited: alice@…`); `User` es la carpeta de perfil en la
  que está la ruta de la base de datos. Pueden ser distintos: una carpeta de perfil no siempre lleva el nombre de su
  cuenta, y la ruta de una base de datos copiada a otro lugar no contiene ninguna carpeta de perfil.
- `Package` indica de quién es una entrada: del Edge heredado (`microsoft.microsoftedge_8wekyb3d8bbwe`), de Internet
  Explorer en Modo protegido mejorado (`windows_ie_ac_001`) o de otra aplicación. Si está vacío, la entrada es del
  propio usuario: Internet Explorer, el Explorador de archivos y otros programas.
- Microsoft no documenta el registro de descarga. Juntei lee su hora de inicio y su tamaño solo cuando su encabezado
  cabe dentro del registro, y sus rutas y demás cadenas solo cuando las dos últimas son una URL y una ruta de Windows.
  En caso contrario, la fila tiene la URL propia de la entrada, y su hora de acceso cuando no se puede leer el inicio,
  con un problema. `Publisher` es el nombre que registró el navegador, no una comprobación de la firma del archivo.
- Una entrada de caché indica que el archivo se descargó y se almacenó en caché. La copia en el disco puede haber
  desaparecido, haber sido reemplazada o no haberse abierto nunca.
- Todavía no se recuperan las entradas eliminadas: ESE puede conservar un registro eliminado en sus páginas hasta que
  las limpia, pero Juntei solo lee los registros activos.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` busca `WebCacheV01.dat` por su nombre; `-f` lee una base de datos sea cual sea su nombre.

## Tablas de salida { #output-tables }

CSV crea cinco archivos, cada uno solo con su encabezado cuando no tiene filas. Las filas JSON y JSONL incluyen `Table`
(`History`, `Downloads`, `Cookies`, `Entries` o `Containers`) después de `SourceFile`.

| Archivo CSV | Contenido |
|---|---|
| `WebCache-History.csv` | Las entradas de los contenedores del historial. |
| `WebCache-Downloads.csv` | Descargas. |
| `WebCache-Cookies.csv` | Cookies (Windows 10 y versiones posteriores). |
| `WebCache-Entries.csv` | Las entradas de todos los demás contenedores: archivos en caché, archivos de cookies, almacenamiento web, listas de compatibilidad y el resto. |
| `WebCache-Containers.csv` | Los contenedores. |

## Campos { #fields }

Todas las tablas empiezan con estos campos:

| Campo | Interpretación |
|---|---|
| `SourceFile` | La base de datos de la que se leyó la fila. |
| `User` | El usuario de Windows en cuyo perfil está la ruta (la carpeta que sigue a `Users`). |
| `Package` | La aplicación a la que pertenecen los datos de la fila: la carpeta bajo `Packages` del directorio de su contenedor (o de su tabla de cookies). Vacío para los datos propios del usuario. |

### Historial { #history }

| Campo | Interpretación |
|---|---|
| `Container` | El contenedor: `History`, o `MSHist01` seguido del primer día de su periodo y del día siguiente al último. |
| `AccessedTime` | La última visita a la URL (UTC); en el contenedor de un día o de una semana, la última de ese periodo. |
| `ModifiedTime` | Cuándo se modificó la entrada por última vez (UTC); en el contenedor de un día o de una semana, la hora local del sistema, sin desplazamiento. |
| `CreationTime` | Cuándo se creó la entrada (UTC). |
| `ExpiryTime`, `SyncTime` | Cuándo debe salir la entrada del historial y cuándo se sincronizó por última vez (UTC). |
| `Account` | La cuenta para la que se registró la entrada. |
| `URL` | La página o el archivo, sin el prefijo `Visited: account@`. |
| `AccessCount`, `SyncCount` | Cuántas veces se visitó la URL y cuántas se sincronizó la entrada. |
| `EntryType`, `Flags` | El tipo y los indicadores de la entrada, tal como están almacenados. |
| `EntryID`, `ContainerID` | El id de la entrada en su contenedor y el id del contenedor. |

### Descargas { #downloads }

| Campo | Interpretación |
|---|---|
| `StartTime` | Cuándo empezó la descarga (UTC). |
| `URL` | De dónde procede el archivo. |
| `TargetPath`, `CurrentPath` | Dónde se guardó el archivo, y dónde se escribió durante la descarga: un archivo `.partial` en el Edge heredado, la copia de la caché en Internet Explorer. |
| `TotalBytes` | El tamaño del archivo. |
| `Referrer` | La página de referencia. |
| `MimeType` | El tipo del archivo tal como lo sirvió el servidor (Internet Explorer). |
| `Publisher` | El nombre que el Edge heredado registró para el editor del archivo. |
| `AccessedTime` | La hora de acceso propia de la entrada (UTC), casi siempre el inicio. |
| `DownloadID`, `ContainerID` | El id de la entrada en su contenedor y el id del contenedor. |

### Cookies { #cookies }

| Campo | Interpretación |
|---|---|
| `LastModified`, `Expires` | Cuándo se estableció la cookie y cuándo caduca (UTC). |
| `Host`, `Name`, `Path` | El sitio para el que es la cookie, su nombre y su ruta. |
| `Value` | El valor tal como está almacenado. |
| `Flags` | Los indicadores de la cookie, tal como están almacenados. |
| `Directory` | El directorio que la base de datos asigna a la tabla de cookies: de quién son las cookies (`…\INetCookies\ESE\`, `…\MicrosoftEdge\Cookies\ESE\` del Edge heredado, el de una aplicación). |
| `CookieTable`, `EntryID` | La tabla (`CookieEntryEx_<n>`) y el id de la cookie en ella. |

### Entradas { #entries }

| Campo | Interpretación |
|---|---|
| `Container` | El contenedor: `Content` (archivos en caché), `Cookies` (archivos de cookies), `DOMStore` (almacenamiento web), `iecompat`, …. |
| `AccessedTime` | Cuándo se usó la entrada por última vez (UTC). |
| `ModifiedTime` | La última modificación del recurso, tal como la indicó el servidor (UTC). |
| `CreationTime` | Cuándo se creó la entrada (UTC). |
| `ExpiryTime`, `SyncTime` | Cuándo caduca y cuándo se comprobó por última vez con el servidor (UTC). |
| `URL` | La URL tal como está almacenada: una URL para un archivo en caché, `Cookie:alice@example.com/` para un archivo de cookie, `DOMStore:…`, `iecompat:…`. |
| `Account` | La cuenta de un archivo de cookie. |
| `Filename`, `FilePath` | El archivo de la entrada y dónde está: el directorio del contenedor y la subcarpeta que nombra la entrada. |
| `FileSize` | El tamaño del archivo. |
| `AccessCount`, `SyncCount` | Cuántas veces se usó la entrada y cuántas se comprobó con el servidor. |
| `EntryType`, `Flags` | El tipo y los indicadores de la entrada, tal como están almacenados. |
| `RedirectURL` | Adónde redirigió la URL. |
| `ResponseHeaders`, `RequestHeaders` | Los encabezados de la respuesta HTTP (`HTTP/1.1 200 OK`, `Content-Type`, `Server`, …) y los de la solicitud, tal como están almacenados, con todas sus líneas. |
| `EntryID`, `ContainerID` | El id de la entrada en su contenedor y el id del contenedor. |

### Contenedores { #containers }

| Campo | Interpretación |
|---|---|
| `ContainerID`, `Name` | El id y el nombre del contenedor. |
| `Directory` | Dónde están sus archivos en el disco. |
| `SecureDirectories` | Las subcarpetas de `Directory` en las que están sus archivos (separadas por comas en CSV). |
| `PartitionID` | `M` para los contenedores propios del usuario, `L` para los del Modo protegido de Internet Explorer (las carpetas `Low`), el SID de un contenedor de aplicación para los de una aplicación. |
| `LastAccessTime`, `LastScavengeTime` | Cuándo se usó el contenedor por última vez y cuándo se limpió por última vez (UTC). |

## Análisis automático { #automatic-parsing }

`auto-parse` escribe las tablas en `BrowserActivity` como `WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv` y `WebCache-Containers.csv` (un único archivo `WebCache.json` o
`WebCache.jsonl` con `-t json` o `-t jsonl`), junto con `BrowserActivity/WebCache-logs.txt`. La línea de tiempo
combinada `-j` incluye todas las filas, con `Comp` y el identificador de `Artifact` `webcache`. `-Z` convierte todas las
horas, los campos que terminan en `Time`, `LastModified` y `Expires`, salvo el `ModifiedTime` local del contenedor
del historial de un día o de una semana.

## Más información { #further-details }

- [Microsoft — Acerca de WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — formato de archivo de base de datos ESE](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — analizador de MSIE WebCache](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
