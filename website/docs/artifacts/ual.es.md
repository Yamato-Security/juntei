# Registro de acceso de usuarios (UAL) { #user-access-logging-ual }

## Qué es UAL { #what-ual-is }

El registro de acceso de usuarios (User Access Logging, UAL) es una característica de Windows Server,
instalada y habilitada de forma predeterminada desde Windows Server 2012, que registra de forma
agregada, en bases de datos ESE, las solicitudes que los clientes hacen a los roles del servidor. Para
cada rol, dirección de cliente y cuenta, conserva cuándo se vio al cliente por primera y por última vez
en el año, el número total de solicitudes y un recuento por cada día del año. Entre los roles se
encuentran File Server (SMB), Active Directory Domain Services, Active Directory Certificate Services,
DHCP Server, Print and Document Services, Remote Access, Web Server (IIS) y Windows Server Update
Services.

UAL es muy valioso en las investigaciones de movimiento lateral: sus filas de clientes muestran qué
direcciones y cuentas usaron un servidor, y en qué días, a menudo mucho después de que los registros
de eventos se hayan sobrescrito.

<span class="evidence-strength">Mejor conclusión:</span> un cliente con la dirección registrada,
autenticado con la cuenta registrada, usó el rol cada uno de los días que tienen un recuento.

## Ubicación y recopilación { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` contiene la actividad más reciente. `SystemIdentity.mdb` describe el servidor, da nombre a
los roles e indica el archivo `{GUID}.mdb` que contiene cada año (`CHAINED_DATABASES`). El servicio de
UAL mantiene abiertas las bases de datos: recopile todo el directorio `Sum` con un recopilador forense o
desde una instantánea del volumen.

!!! warning "Bases de datos sucias"
    `Current.mdb` suele estar sucio (dirty) cuando se recopila de un servidor en ejecución. Juntei lo lee
    tal como se recopiló e informa de ello, pero no reproduce `Svc*.log`: falta la actividad que solo
    existe en los registros de transacciones.

## Precauciones para el analista { #analyst-cautions }

- UAL cuenta las solicitudes por día; no registra cada solicitud por separado. Las únicas horas de una
  fila de cliente son `InsertDate` y `LastAccess`.
- Los días de `DailyAccesses` son días UTC.
- `Address` es la dirección que vio el servidor: entre este y el cliente real puede haber una puerta de
  enlace NAT, un proxy o una VPN. `::1` y `127.0.0.1` son el propio servidor.
- Analice todas las bases de datos del directorio `Sum`: el mismo cliente puede tener filas en
  `Current.mdb` y en el `{GUID}.mdb` de su año.
- La ausencia de filas no demuestra que nada se conectara: UAL se puede desactivar (`Disable-Ual`, o
  deteniendo el servicio User Access Logging), y a una base de datos sucia le faltan sus cambios más
  recientes.
- Sin `SystemIdentity.mdb`, `RoleName` procede de los nombres que Windows Server da a sus roles.
- Un `LastAccess` igual a cero o no válido no determina el año. Juntei utiliza entonces un `InsertDate` válido y después los metadatos de la base anual; si no hay un año válido, conserva `DayOfYear`.
- Los totales del resumen suman todas las filas de origen. `Current.mdb`, las bases anuales y las instantáneas pueden solaparse; estos valores no son recuentos de conexiones o inicios de sesión sin duplicados.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Current.mdb, {GUID}.mdb and SystemIdentity.mdb
  -f, --file <FILE>      Parse one UAL database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl \
  | jq 'select(.Table == "CLIENTS") | {Address, AuthenticatedUserName, RoleName, LastAccess}'
```

## Tablas de salida { #output-tables }

CSV crea un archivo por tabla, que solo contiene el encabezado cuando la tabla está vacía. Las filas JSON
y JSONL incluyen `SourceFile` (la base de datos) y `Table` (el nombre de la tabla en la base de datos).

| Archivo CSV | Tabla | Contenido |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | Los clientes de cada rol: dirección, cuenta, primer y último acceso del año, recuento total de solicitudes y recuentos diarios. |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | Cuándo se usó cada rol por primera y por última vez. |
| `UAL-DNS.csv` | `DNS` | Pares de dirección y nombre de host, con la última vez que se vio cada uno. |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Máquinas virtuales de Hyper-V: identificadores, número de serie, horas de creación y de última actividad. |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | El servidor: nombre, dominio, versión de Windows, tipo de producto, hardware, zona horaria y último arranque; una fila nueva puede revelar un cambio de nombre, la unión a un dominio o la promoción a controlador de dominio. |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | El archivo `{GUID}.mdb` de cada año. |
| `UAL-Roles.csv` | `ROLE_IDS` | Los GUID de los roles con sus nombres. |
| `UAL-<table>.csv` | Otra tabla | Todas las columnas de una tabla que Juntei no conoce. |

## Campos { #fields }

| Campo | Interpretación |
|---|---|
| `SourceFile` | La base de datos de la que se leyó la fila. |
| `RoleGUID`, `RoleName` | El rol y su nombre, tomado del `SystemIdentity.mdb` que está junto a la base de datos (nombres integrados si no lo hay). |
| `TenantID` | El GUID del inquilino (tenant) del cliente, cuando el rol lo indica. |
| `TotalAccesses` | Solicitudes del cliente en el año de la base de datos; la suma de `DailyAccesses`. |
| `InsertDate` | Cuándo se creó la fila: la primera solicitud del cliente en el año en esta base de datos (UTC). |
| `LastAccess` | La solicitud más reciente del cliente (UTC). |
| `Address` | La dirección IPv4 o IPv6 del cliente. |
| `AuthenticatedUserName` | La cuenta con la que se autenticó el cliente (`DOMAIN\user`; las cuentas de equipo terminan en `$`). |
| `ClientName` | El nombre del cliente, cuando el rol lo indica. |
| `DailyAccesses` | Los días con solicitudes y sus recuentos, a partir de las columnas `Day1` … `Day366`, con fechas del año de `LastAccess`: `2023-01-12: 5, 2023-01-13: 2` en CSV, objetos con `Date` y `Accesses` en JSON. Un día cuyo año se desconoce tiene `DayOfYear` en lugar de `Date`. |
| `FirstSeen`, `LastSeen` | Primer y último uso de un rol (`ROLE_ACCESS`), última vez que se vio un par DNS (`DNS`), en UTC. |
| `CreationTime`, `LastSeenActive` | Cuándo se creó una máquina virtual o una fila `SYSTEM_IDENTITY`, y cuándo estuvo activa por última vez una máquina virtual, en UTC. |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server`. |
| `OS-CurrentTimeZone` | El desfase del servidor respecto a UTC, en minutos (`-420` es UTC-07:00). |
| `OS-LastBootUpTime` | El último arranque del servidor en UTC (el texto WMI tal como está almacenado cuando no se puede leer). |
| Otras columnas | Todas las demás columnas de la tabla con las etiquetas de campo en PascalCase de Juntei, por ejemplo `SystemDNS-HostName`, `SystemDomainName`, `OS-BuildNumber` y `SystemSerialNumber`. |

## Análisis automático { #automatic-parsing }

`auto-parse` escribe las tablas en `LateralMovement` como `UAL-Clients.csv`, `UAL-RoleAccess.csv`, etc.
(un único archivo `UAL.json` o `UAL.jsonl` con `-t json` o `-t jsonl`), junto con
`LateralMovement/UAL-logs.txt`. La línea de tiempo combinada `-j` incluye todas las filas, con `Comp` y
el identificador de `Artifact` `ual`. `-Z` convierte `InsertDate`, `LastAccess`, `FirstSeen`, `LastSeen`,
`CreationTime`, `LastSeenActive` y `OS-LastBootUpTime`.

## Más información { #further-details }

- [Microsoft — User Access Logging](https://learn.microsoft.com/en-us/windows-server/administration/user-access-logging/get-started-with-user-access-logging)
- [Plaso — User Access Logging parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/user_access_logging.py)
- [Velociraptor — Windows.Forensics.UserAccessLogs](https://docs.velociraptor.app/artifact_references/pages/windows.forensics.useraccesslogs/)
- [CrowdStrike — How to Leverage User Access Logging for Forensic Investigations](https://www.crowdstrike.com/en-us/blog/user-access-logging-ual-overview/)
