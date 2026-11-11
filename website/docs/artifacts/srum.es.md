# SRUM { #srum }

## Qué es SRUM { #what-srum-is }

System Resource Usage Monitor (SRUM) almacena una base de datos ESE sobre el uso de recursos por
parte de aplicaciones y servicios. Según la versión de Windows, registra bytes de red,
identificadores de interfaz y de perfil, conexiones, tiempo de CPU, actividad en primer y segundo
plano, actividad de disco, consumo de energía, notificaciones push y datos de la línea de tiempo de
aplicaciones. Las entradas se asocian con identificadores de aplicación y SID de usuario mediante
`SruDbIdMapTable`.

SRUM es especialmente útil cuando se ha eliminado un ejecutable: su identificador de aplicación, su
usuario y sus filas de uso de recursos pueden sobrevivir al archivo. Los totales de red pueden
respaldar una hipótesis de exfiltración, pero SRUM no es una captura de paquetes y, por lo general,
no identifica todos los extremos remotos ni todos los archivos transferidos.

<span class="evidence-strength">Mejor conclusión:</span> Windows atribuyó el uso de recursos
registrado a la aplicación o cuenta durante el periodo de medición de la tabla.

## Ubicación y recopilación { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM existe en Windows 8 y versiones posteriores. Copie `SRUDB.dat` de forma coherente,
idealmente desde una instantánea o con un recopilador forense. Recopile también `SOFTWARE` y los
archivos de transacciones y de registro de ESE del directorio `sru`.

El hive SOFTWARE correspondiente es independiente de la base ESE: si está sucio, Juntei reproduce automáticamente en memoria sus registros de transacciones del Registro adyacentes `SOFTWARE.LOG`, `.LOG1` y `.LOG2`. Recopílelos también. Se muestran detalles de recuperación y advertencias; sin estos registros, los nombres de cuentas o Wi-Fi pueden estar incompletos o desactualizados. Esto no reproduce los registros de la base SRUM.

!!! warning "Bases de datos sucias"
    Juntei lee un `SRUDB.dat` sucio (dirty) tal como se recopiló e informa de ello. Actualmente no
    reproduce `SRU*.log`, por lo que faltan los cambios que solo existen en esos archivos de
    registro. No describa de forma tácita una base de datos sucia como completa.

## Precauciones para el analista { #analyst-cautions }

- Muchas filas representan una agregación periódica, normalmente en torno a los límites de cada
  hora, no la hora exacta de una acción.
- Los totales de bytes no identifican por sí solos el contenido, la intención, el protocolo ni el
  destino.
- La identidad de una aplicación puede ser una ruta, un servicio, una aplicación empaquetada o un
  objeto; examine `AppType`.
- Los nombres de usuario y de Wi-Fi son un enriquecimiento procedente del hive `SOFTWARE`
  correspondiente. La ausencia de nombres no significa que falte el identificador subyacente.
- `null`, cero, columnas ausentes, tipos no admitidos y valores dañados son estados distintos.
- Los esquemas de los proveedores varían según la versión de Windows. Conserve las tablas
  desconocidas y los nombres de columna originales.
- Los servicios de Windows en segundo plano generan legítimamente grandes totales de recursos.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## Tablas de salida { #output-tables }

CSV crea `SRUM-IdentityMap.csv` para `SruDbIdMapTable` y una tabla por proveedor. Entre los nombres de salida conocidos se incluyen:

| Proveedor/tabla | Finalidad para la investigación |
|---|---|
| `NetworkUsages` | Bytes enviados/recibidos por aplicación, usuario, interfaz y perfil. |
| `NetworkConnections` | Actividad de tipo recuento o duración de conexiones y horas de inicio de las conexiones. |
| `AppResourceUseInfo` | Uso de CPU, disco, red y otros recursos de las aplicaciones en primer y segundo plano. |
| `AppTimelineProvider` | Actividad de la línea de tiempo de aplicaciones con campos de hora de finalización y estado de uso. |
| `EnergyUsage` | Mediciones de energía y del proveedor a corto y largo plazo; `IsLt` las distingue. |
| `PushNotifications` | Actividad de notificaciones push. |
| `VfuProvider` | El proveedor `vfuprov`; su finalidad detallada no está bien documentada, por lo que conviene interpretar sus campos con cautela. |
| `EnergyEstimation`, `TaggedEnergy` | Proveedores de energía y uso específicos de cada versión. |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Tablas de proveedores de Windows Server. |
| Nombre de tabla desconocido | Juntei conserva los proveedores desconocidos con su GUID o nombre de tabla y genera todas las columnas del catálogo. |

Como SRUM se basa en un catálogo y Windows añade columnas a los proveedores, intencionadamente no
existe una lista fija y finita de campos específicos de cada proveedor. Juntei genera **todas las
columnas presentes en la base de datos con las etiquetas de campo en PascalCase de Juntei**. Los
campos siguientes constituyen el contrato de salida estable y común.

## Salida clasificada de auto-parse { #categorized-auto-parse-output }

`auto-parse` coloca cada tabla de SRUM directamente en su categoría de investigación. CSV, JSON y
JSONL usan el mismo prefijo de nombre de archivo `SRUM-`; no hay ningún directorio `SRUM/` que las
englobe.

| Categoría | Archivos CSV |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`, `SRUM-EnergyEstimation.csv`, `SRUM-TaggedEnergy.csv`, `SRUM-SizeInBytes.csv`, `SRUM-TotalUsed.csv`, `SRUM-IdentityMap.csv`, `SRUM-VfuProvider.csv` y las tablas de proveedores desconocidos |

Solo se exportan las tablas presentes en la captura. Los contadores de red y los registros de
notificaciones se ubican según su principal uso en la investigación; esto no significa que cada
fila describa una conexión remota. El mapa de identidades sirve a varias categorías. VFU y los
proveedores desconocidos permanecen en `SystemInformation` porque su mera presencia no demuestra
una ejecución.

`SystemInformation/SRUM-logs.txt` contiene los diagnósticos del analizador y la correspondencia
entre la salida original y la final. Ambos manifiestos enumeran las rutas reales de las tablas en
`OutputFiles`; el `Output` de la tarea SRUM es el directorio del equipo y `Category` identifica la
ubicación compartida de su registro y metadatos.

Con varias bases de datos, los nombres de archivo reciben `-Source1`, `-Source2`, etc., antes de la
extensión. El registro conserva los alias originales de la captura, y las filas JSON/JSONL
mantienen su procedencia `SourceFile`, `Table`, `Provider`, `Comp` y `Artifact`. Ninguna tabla se
copia en varias categorías. La línea de tiempo SIEM combinada `-j` incluye una sola vez cada
registro de SRUM conservado.

El comando independiente `parse-ese srum` mantiene la ubicación de salida solicitada. Sus archivos
de tabla CSV también llevan el prefijo `SRUM-`; los nombres de archivo de salida JSON/JSONL
explícitos los sigue eligiendo el analista.

## Campos de salida comunes { #common-output-fields }

| Campo | Interpretación |
|---|---|
| `SourceFile` | Base de datos de entrada; presente en JSON/JSONL y útil entre hosts. |
| `Table` | Nombre o GUID original de la tabla ESE. Consérvelo al comparar versiones o proveedores desconocidos. |
| `Provider` | Nombre descriptivo del proveedor o de la salida, o el nombre original de la tabla si es desconocido. |
| `ID` | El `AutoIncId` de la fila; un identificador de fila de la base de datos, no un identificador de proceso. |
| `Timestamp` | Cuándo SRUM registró o agregó las cifras, redondeado a milisegundos. Normalmente no es la hora exacta de un evento. |
| `EXE-Info` | Identidad resuelta de la aplicación, servicio o paquete a partir de `AppID`. Pivote principal del programa. |
| `EXE-InfoDescription` | Descripción extraída de los nombres compuestos de aplicaciones de SRUM. Solo contexto. |
| `EXE-Timestamp` | Hora incrustada en una identidad de aplicación compuesta, si existe; no es la hora de uso de la fila. |
| `AppType` | `NormalApp`, `Service`, `ModernApp` u `Object`; determina cómo debe interpretarse `EXE-Info`. |
| `SID` | SID de la cuenta resuelto a partir de `UserID`. Pivote sólido de identidad de usuario o servicio. |
| `User` | Nombre de perfil descriptivo enriquecido a partir de `SOFTWARE`; puede ser nulo aunque `SID` siga siendo válido. |
| `UserID`, `AppID` | Referencias numéricas sin procesar a `SruDbIdMapTable`; consérvelas para nombres no resueltos o duplicados. |
| `IsLt` | `true` para la tabla EnergyUsage a largo plazo, `false` para la de corto plazo. |
| `InterfaceLUID` | LUID sin procesar de la interfaz de red de las filas del proveedor. |
| `InterfaceType` | Clase de interfaz decodificada (por ejemplo, `IF_TYPE_IEEE80211`); es una clase, no un adaptador único. |
| `L2ProfileID` | Referencia numérica al perfil de red. |
| `ProfileName` | Nombre de Wi-Fi o de perfil enriquecido a partir de `SOFTWARE`; potencialmente sensible y no es prueba de una conexión actual. |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | Columnas FILETIME conocidas de los proveedores decodificadas a UTC. Su significado exacto depende del proveedor; no son una hora de ejecución universal. |
| Columnas específicas del proveedor | Columnas exportadas del proveedor como `BytesSent`, `BytesReceived` (la columna `BytesRecvd` de la base de datos), contadores de CPU/disco, duraciones, recuentos, indicadores o estimaciones. Interprete las unidades según el proveedor y compare filas dentro del mismo esquema. |

## Campos de diagnóstico (`--include-value-state`) { #diagnostic-fields-include-value-state }

| Campo | Interpretación |
|---|---|
| `ValueState` | Mapa de cada campo de la base de datos a `parsed`, `null`, `absent`, `unsupported` o `damaged`. Distingue la ausencia de evidencia de un fallo del analizador. |
| `Raw` | Bytes originales en hexadecimal de los valores que tienen bytes. Úselos para la validación y para futuras decodificaciones. |

`SRUM-IdentityMap.csv` (de `SruDbIdMapTable`) contiene `IdIndex`, `IdType` e `IdBlob`: el índice sin procesar, la categoría de identidad
y el texto decodificado del programa, SID u objeto que usan las filas de los proveedores.

## Estrategia de investigación { #investigation-strategy }

1. Identifique el programa con `EXE-Info`, `AppType` y `AppID`; no confunda `Id` con el PID.
2. Agrupe las filas de red por programa, usuario, perfil, interfaz e intervalo de tiempo.
3. Compare los bytes enviados y recibidos con las líneas base normales y con otros hosts.
4. Correlacione los intervalos sospechosos con Prefetch, los registros de eventos, los datos de
   DNS/proxy/cortafuegos y el acceso a archivos.
5. Indique en el informe el estado de base de datos sucia y la falta de reproducción de los
   registros de transacciones como una limitación de la recopilación.

## Más información { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [SrumECmd de Eric Zimmerman](https://github.com/EricZimmerman/Srum)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
