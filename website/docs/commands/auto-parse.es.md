# Análisis automático de recopilaciones { #automatic-collection-parsing }

`auto-parse` es la forma más rápida de convertir una recopilación de triaje de Windows de varios
equipos en un directorio de análisis estructurado. Indíquele la raíz de las evidencias y un nuevo
directorio de salida. Descubre los sistemas recopilados, identifica los nombres de sus equipos y
ejecuta en paralelo todos los analizadores de Juntei compatibles.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

La salida predeterminada es CSV. Añada una línea de tiempo JSONL para `jq` o para importarla en un
SIEM sin renunciar a esos resultados CSV:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "Mantenga separados los árboles de salida y de evidencias"
    Juntei rechaza un directorio de salida que esté dentro de `<EVIDENCE>` o que contenga
    `<EVIDENCE>`. Así se evita que los datos generados se vuelvan a descubrir como evidencias y que
    `--clobber` toque archivos de la recopilación de origen.

## Imágenes de disco VHDX { #vhdx-disk-images }

Con `--vhdx`, `auto-parse` analiza los sistemas contenidos en cada imagen de disco VHDX del directorio de
evidencias, como las recopilaciones `--vhdx` de KAPE. Cada imagen se monta en modo de solo lectura con FUSE
mientras se analiza, así que no se extrae nada. Consulte [Montar imágenes de disco](mount.md).

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

Si no se puede abrir un volumen NTFS, Juntei informa del problema y conserva los volúmenes legibles. Con imágenes parcialmente legibles, `auto-parse --vhdx` devuelve el código 1; `mount` también devuelve 1 después de desmontar.

## Zona horaria de salida { #output-time-zone }

Las marcas de tiempo analizadas están en UTC de forma predeterminada, independientemente de la zona
horaria local del equipo del analista. UTC utiliza la forma de los registros de eventos de Windows
`2026-10-20T08:46:38.2001668Z`: una fecha y hora ISO 8601, `T` entre la fecha y la hora, y `Z` para
UTC. Se conservan los dígitos fraccionarios disponibles. El `GeneratedUTC` de `manifest.json`
también utiliza esta forma. Los valores `Start time` y `End time` de la consola están en la hora
local del equipo del analista, como `2026-11-04 09:15:02`, sea cual sea la zona horaria de salida.

Elija un desplazamiento de presentación fijo con `-Z, --timezone`:

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

Un desplazamiento es un valor `HH:MM` con signo y de dos dígitos (horas `00–23`, minutos `00–59`).
Los nombres como `JST`, los valores abreviados como `+9` y los rangos no válidos producen un error
con un ejemplo de sintaxis. Omita la opción o use `--timezone "+00:00"` para UTC. Un desplazamiento
fijo no aplica las reglas del horario de verano.

La opción se aplica a las marcas de tiempo analizadas de los resultados CSV, JSON y JSONL de cada
artefacto y a la exportación JSONL adicional `-j`. Por ejemplo, `2026-10-20T08:46:38.2001668Z` pasa
a ser `2026-10-20T17:46:38.2001668+09:00`; ambos describen el mismo instante.

Los nombres de campo se mantienen estables, incluidos los nombres existentes que terminan en `UTC`.
Los valores convertidos llevan explícitamente el desplazamiento seleccionado. `OutputTimeZone` en
ambos manifiestos registra el desplazamiento de presentación solicitado, mientras que
`GeneratedUTC` sigue siendo un metadato de ejecución en UTC. Los registros de cada analizador conservan los resúmenes nativos en UTC de los analizadores.

Se conservan los números FILETIME/epoch sin procesar, los diccionarios de valores del Registro, la
configuración de zona horaria de origen, los metadatos de texto, los valores que solo contienen una
fecha y los valores de reloj local sin un desplazamiento conocido. Juntei no añade `Z` a una hora
local desconocida. Los relojes de elementos del shell derivados de FAT (`Item*UTC` en
ShellBags/OpenSaveMRU y `TargetItems` de LNK/Jump List) no tienen un desplazamiento registrado; sus
etiquetas UTC heredadas o el sufijo `Z` mostrado no prueban que estén en UTC, por lo que la opción
de desplazamiento los conserva.
Las columnas CSV que contienen resúmenes de texto libre conservan
su texto registrado; JSON/JSONL conserva como campos estructurados las marcas de tiempo analizadas anidadas.

UTC utiliza directamente la salida existente del analizador. Un desplazamiento explícito distinto
de cero añade una pasada de formato en flujo dentro de cada proceso de trabajo del analizador;
cambia los valores mostrados sin modificar la evidencia.

Si un desplazamiento llevara una marca de tiempo fuera de los años `0000–9999`, ese valor conserva
su desplazamiento explícito original. Los valores con más de nueve dígitos fraccionarios tampoco se
modifican, en lugar de redondearse. Juntei registra una advertencia en el archivo `*-logs.txt` del
analizador y convierte las demás marcas de tiempo. Un CSV/JSON mal formado o un error de salida
siguen haciendo fallar la tarea y dejan sin cambios el archivo de resultados original afectado.

## Qué descubre { #what-it-discovers }

Juntei busca de forma recursiva directorios llamados `Windows` que contengan un directorio
`System32`, `Prefetch` o `AppCompat`, sin distinguir mayúsculas y minúsculas. Reconocer estos dos
últimos permite también procesar recopilaciones dirigidas que solo contienen Prefetch o Amcache.
Esto excluye directorios de perfiles de usuario como `AppData\Roaming\Microsoft\Windows`. Las
instalaciones heredadas anidadas, como `C:\Windows.old\Windows`, se asignan al volumen recopilado
exterior en lugar de convertirse en equipos duplicados. La profundidad máxima predeterminada es 10.
Esto abarca rutas habituales de Velociraptor como:

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

Cambie el límite solo cuando la recopilación esté anidada a mayor profundidad:

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

Para cada sistema descubierto, el nombre del equipo se elige en este orden:

1. `hostname` en el `client_info.json` del antecesor más cercano (su valor se conserva como `FQDN`
   en el manifiesto).
2. `ComputerName` en un hive del Registro `SYSTEM` recopilado.
3. El nombre útil más cercano de un directorio de la recopilación, omitiendo la unidad y la
   estructura `uploads/auto` de Velociraptor, y quitando un sufijo como `_F.…`.

Los nombres se adaptan para que sean seguros como nombres de directorio. Los nombres duplicados
reciben un sufijo numérico tanto en su directorio de salida como en su valor `Comp`, para que los
sistemas distintos sigan pudiendo distinguirse en JSON. El manifiesto conserva el nombre de host
exactamente como se descubrió en un campo `Hostname` independiente.

Después de enumerar los sistemas, `auto-parse` muestra una línea `Timezone:` justo encima de
`Parser tasks`. Indica la zona horaria de salida seleccionada: `UTC` de forma predeterminada, o un
valor como `UTC+09:00` cuando se usa `--timezone "+09:00"`. La etiqueta es verde y el valor usa el
color de texto predeterminado de la terminal, como los demás valores del resumen.
`-K, --no-color` mantiene la línea sin formato.
La configuración de zona horaria de los sistemas recopilados está disponible en los resultados de
[Información del sistema](../artifacts/system-info.md).

## Analizadores que ejecuta { #parsers-it-runs }

El fallo de un analizador no detiene a los demás. `auto-parse` ejecuta estos trabajos para cada
equipo descubierto. Los trabajos usan los mismos comandos agrupados por fuente que el análisis
manual, como `parse-ntfs mft`, `parse-registry amcache`, `parse-ese srum` y `parse-file prefetch`:

- Amcache
- `$MFT`
- Registros redo/undo de NTFS de `$LogFile`
- Registros de Prefetch y una línea de tiempo de ejecución de Prefetch independiente
- Diario USN
- Índices de directorios `$I30`
- Sectores de arranque `$Boot` y descriptores de seguridad `$Secure:$SDS`
- SRUM
- Registro de acceso de usuarios (UAL)
- Accesos directos LNK
- Jump Lists
- ShimCache
- Persistence Hunter (131 comprobaciones de persistencia entre fuentes con perfiles de permitidos y
  bloqueados)
- Metadatos de la cuarentena de Defender
- Historial de los cuadros de diálogo Abrir y Guardar como
- todas las claves y valores del Registro, incluidos los datos eliminados recuperables
- servicios, RecentDocs, TaskCache, cuentas SAM, ubicaciones de inicio automático del Registro,
  UserAssist, BAM, programas instalados, MuiCache, ShellBags y dispositivos USB
- información del sistema: nombres, versión de Windows, zona horaria, último apagado, perfiles y
  SID
- la directiva de auditoría: qué eventos de seguridad estaba configurado para registrar el sistema
- configuración de arranque (BCD y BCD-Template)
- perfiles de red, interfaces, recursos compartidos, proxies de puertos y rutas
- nombres y horas de los secretos LSA, e inicios de sesión de dominio en caché en uso (no se
  descifra nada)

Persistence Hunter conserva todas las gravedades (`--min-severity info`), incluidas las
coincidencias parciales con la lista de permitidos. `LOW` es una clasificación que hay que
investigar; la ausencia de metadatos de firma también puede producirla para comandos maliciosos. El
inventario automático no descarta estas entradas con el umbral medio predeterminado del comando
manual.

El cazador utiliza un único proceso de trabajo dentro de cada proceso de análisis automático. Así
se evita que `auto-parse -T <N>` multiplique su presupuesto de procesos por otro grupo del tamaño
del número de CPU en cada proceso hijo. El comando independiente `persistence-hunter -T <N>`
controla los hilos de trabajo propios del cazador.

La salida CSV de `$MFT` incluye todas las marcas de tiempo y los nombres cortos. La salida del
Registro incluye los datos sin procesar y una interpretación FILETIME de los valores `REG_QWORD`.
La salida JSON y JSONL de SRUM incluye metadatos del estado de los valores. Los archivos de
Defender y los flujos residentes de `$MFT` **no se extraen** automáticamente.

Con `--mft-slack`, la salida de `$MFT` también incluye los nombres de archivo que quedan en el slack de los registros FILE, tal como los escribe [`parse-ntfs mft -x`](../artifacts/mft.md): filas marcadas con `FromSlack` en CSV y `SlackFileNames` en JSON y JSONL. `--code-page` se aplica a los archivos LNK, las Jump Lists, ShellBags y OpenSaveMRU.

Un artefacto que no se recopiló es una tarea `skipped` normal, no un fallo.

## Estructura de la salida { #output-layout }

Los resultados se organizan por equipo y por categoría de investigación. Los directorios de
categoría usan PascalCase, como `Accounts` y `FileAndFolderOpening`. Las tablas de Amcache y de Jump
Lists se ubican directamente en sus categorías con prefijos en el nombre de archivo. Los nombres de los directorios de equipo conservan la grafía con la que se descubrieron. Los directorios de salida
existentes no se migran; elija un nuevo directorio de salida al pasar de la nomenclatura antigua. Cada analizador escribe un
archivo `<artifact>-logs.txt` junto a sus resultados, también cuando el artefacto no se recopiló.
CSV mantiene tablas separadas porque cada artefacto tiene columnas diferentes:

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| Categoría | Artefactos |
|---|---|
| `Accounts` | Cuentas locales SAM, nombres y horas de los secretos LSA |
| `Applications` | Historial de aplicaciones (rastros de herramientas de terceros) |
| `Applications` | Programas instalados |
| `ExternalDevices` | Dispositivos y volúmenes USB |
| `FileAndFolderOpening` | Jump Lists, accesos directos LNK, Microsoft Office, listas MRU, cuadros de diálogo Abrir y Guardar como, RecentDocs, ShellBags |
| `FileExistence` | Amcache, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, Diario USN |
| `NetworkActivity` | Perfiles de red, interfaces, recursos compartidos, proxies de puertos y rutas; conexiones, tráfico y notificaciones push de SRUM |
| `LateralMovement` | Conexiones salientes de Escritorio remoto, PuTTY y WinSCP de los usuarios, unidades asignadas y recursos compartidos; registro de acceso de usuarios (los clientes de los roles de un servidor) |
| `Persistence` | Configuración de arranque (BCD), inicios automáticos del Registro, servicios, TaskCache |
| `ProcessExecution` | Compatibilidad de aplicaciones, BAM y DAM, FeatureUsage y elementos anclados, MuiCache, Prefetch, línea de tiempo de Prefetch, actividad de aplicaciones/CPU de SRUM, UserAssist |
| `Registry` | Claves y valores generales de los hives, incluidos los registros recuperados |
| `SecurityProducts` | Metadatos de la cuarentena de Defender y contenido extraído opcional; configuración de seguridad (Defender, UAC, protección de credenciales, RDP, cortafuegos, registro) |
| `SystemInformation` | Nombres, versión de Windows, zona horaria, último apagado, perfiles, SID y directiva de auditoría; energía/capacidad de SRUM, mapas de identidades y proveedores desconocidos |

Las categorías ayudan a organizar las investigaciones; no son una prueba de una actividad
concreta. Por ejemplo, las entradas de Amcache y ShimCache por sí solas no demuestran una
ejecución. SRUM registra el uso de recursos; consulte la guía del artefacto antes de sacar
conclusiones sobre la ejecución de un programa.

CSV, JSON y JSONL usan las mismas categorías. Un archivo `<artifact>-logs.txt` por cada trabajo de
analizador contiene su comando exacto y sus mensajes; por ejemplo, `HOST1/FileSystem/MFT-logs.txt`
y `HOST1/ProcessExecution/Prefetch-logs.txt`. JSON y JSONL usan los mismos nombres de archivo de
registro. Amcache y Jump Lists tienen cada uno un registro compartido junto a sus tablas con prefijo. Las tablas de proveedores de SRUM se ubican directamente
en `NetworkActivity`, `ProcessExecution` y `SystemInformation`, con un archivo compartido
`SystemInformation/SRUM-logs.txt`. No se crea ningún subdirectorio `logs/`.

Los nombres de archivo CSV predeterminados son `Registry-SAM.csv`, `LNK.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv` y
`DefenderQuarantine.csv`. Las tablas derivadas del Registro usan el prefijo `Registry-`, incluidas
`Registry-ShimCache.csv`, `Registry-UserAssist.csv`, `Registry-BAM.csv` y `Registry-SystemInfo.csv`.
Las claves y valores generales de los hives están en `Registry-Dump.csv`. Las tablas de Amcache son
`FileExistence/Registry-Amcache-Devices.csv`, `Registry-Amcache-Files.csv`,
`Registry-Amcache-Findings.csv` y `Registry-Amcache-Summary.csv`. Las Jump Lists usan
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` y
`JumpLists-CustomDestinations.csv`. No hay subdirectorios de Amcache ni de Jump Lists.
Las tablas de SRUM usan nombres de archivo `SRUM-` directamente bajo su categoría, por ejemplo
`NetworkActivity/SRUM-NetworkConnections.csv`.
Las tablas del registro de acceso de usuarios están en `LateralMovement`, un archivo por tabla, como
`UAL-Clients.csv` y `UAL-SystemIdentity.csv`, junto con `UAL-logs.txt`; consulte
[Registro de acceso de usuarios (UAL)](../artifacts/ual.md#automatic-parsing).
Consulte la [tabla completa de ubicación de SRUM](../artifacts/srum.md#categorized-auto-parse-output).

Los registros coinciden con el nombre del artefacto en mayúsculas, como `Registry-SAM-logs.txt`,
`PrefetchTimeline-logs.txt` y `Registry-UserAssist-logs.txt`. Los archivos JSON/JSONL usan las
mismas raíces, por ejemplo `MFT.jsonl`; el identificador `Artifact` del registro es `mft` para las
consultas. Las salidas con varias tablas tienen `Registry-Amcache-logs.txt` o `JumpLists-logs.txt`
junto a sus tablas. El registro compartido de SRUM está en `SystemInformation`.


`manifest.json` registra el `Hostname` original descubierto, la identidad `Comp` única de la salida
y de los registros, la procedencia del nombre de host, las rutas de las evidencias, la `Category`
del artefacto, las líneas de comandos, la duración, el código de salida, la ruta del resultado, la
ruta del registro, la línea de tiempo JSONL solicitada y cualquier error de exportación, si se
solicitó la extracción de Defender,
y si cada tarea
se completó correctamente, se omitió o falló.

Auto-parse no escribe ninguna ruta absoluta, por lo que los resultados pueden compartirse sin
revelar dónde se encuentran las evidencias, los resultados o Juntei en el sistema de análisis:

- Las rutas de las evidencias son relativas al directorio de evidencias. La terminal enumera cada
  equipo con su directorio dentro de las evidencias (`HOST1: ./HOST1_F`), y los analizadores se
  ejecutan en el directorio de evidencias y reciben rutas de ese tipo. Las columnas de ruta de
  origen de los resultados (`SourceFile`, `SourcePath`, …), los registros y los campos
  `EvidenceRoot` y `WindowsDirectories` de los manifiestos indican dónde está cada archivo dentro
  de las evidencias (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`).
- Las rutas de resultados de los manifiestos y registros (`Output`, `OutputFiles`, `Log`,
  `JSON-Timeline`, las rutas de salida en `Command`) y el `ExtractedFile` de Defender son relativas
  al directorio de salida (`./HOST1/ProcessExecution/Prefetch.csv`, `../siem.jsonl` para una línea
  de tiempo situada junto a él). `Command` empieza por el nombre del programa (`juntei`); sus rutas
  de evidencias son relativas al directorio de evidencias y sus rutas de salida, al directorio de
  salida.
- `EvidenceDirectory` y `OutputDirectory` son los nombres de los dos directorios.
  `ManifestVersion` 2 marca los manifiestos que registran las rutas de resultados desde el
  directorio de salida; los manifiestos anteriores no tienen `ManifestVersion` y las registran tal
  como se pasaron a `-o`. Una nueva ejecución con `-C` lee ambos.
- La terminal muestra una ruta de salida o de línea de tiempo tal como se indicó cuando es
  relativa, y desde el directorio actual cuando es absoluta (`../results/`).

Después del número de procesos en paralelo, la terminal muestra el tamaño total de los archivos de
los sistemas Windows encontrados (`Total filesize: 26.1 MiB`).

El mensaje de progreso muestra el equipo y el artefacto activos seguidos de un tamaño de entrada
aproximado, por ejemplo `HOST1 / prefetch (~2.0 MiB)`. Un grupo de archivos usa su tamaño
combinado; cada tarea de MFT, LogFile o Diario USN muestra el tamaño de su entrada individual. Las
estimaciones excluyen los registros de transacciones auxiliares y los archivos de enriquecimiento,
y no predicen el tamaño de la salida analizada.

`manifest.csv` proporciona el mismo índice de tareas como una tabla apta para hojas de cálculo, con
una fila por tarea de analizador. Incluye `Comp`, el `Hostname` original, `FQDN`, `HostnameSource`,
`EvidenceRoot`, `Artifact`, `Category`, `OutputType`, `Purpose`, `Status`, `DurationMs`,
`ExitCode`, `Output`, `OutputFiles`, `Log`, `Error` y `Command`. `Purpose` es `artifact` para los
resultados clasificados o `json_timeline` para la pasada JSONL nativa adicional que se usa con CSV.
`OutputType` es el formato de salida real del analizador. `Command` es una matriz JSON con el
ejecutable y sus argumentos, que conserva los espacios y las comillas.
Para SRUM, `Output` es el directorio del equipo, `OutputFiles` es una matriz JSON con las rutas de
salida exactas de sus proveedores y `Category` indica la ubicación del registro compartido
(`SystemInformation`). Los archivos se clasifican individualmente; `OutputFiles` también aparece en
el manifiesto JSON.

Filtre `Status` por `skipped` o `failed` para encontrar las tareas que requieren revisión; el
`Error` de una tarea omitida puede estar vacío, con la explicación en su archivo `Log`.

El resumen de la terminal incluye el número total y el tamaño de los archivos normales que hay en
ese momento bajo el directorio de salida, incluidos ambos manifiestos, los registros de los
analizadores y los archivos extraídos. No se siguen los enlaces simbólicos. Los tamaños se escalan
automáticamente con el mismo formateador que `Total filesize`, con unidades como `KiB`, `MiB` y
`GiB`:

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

Las tablas de Amcache y SRUM usan `-Source1`, `-Source2`, etc. cuando se recopilaron varias bases de
datos de origen. El registro compartido del analizador registra la correspondencia entre las rutas
originales y las finales, y los registros JSON/JSONL conservan la procedencia de la fuente. Las
tareas CSV de Amcache/Jump Lists enumeran las rutas exactas de sus tablas en el campo
`OutputFiles` del manifiesto; su `Output` es el directorio de la categoría de investigación. Las
exportaciones JSON y JSONL de Amcache y Jump Lists ya son archivos únicos llamados
`Registry-Amcache.json[l]` y `JumpLists.json[l]`. La línea de tiempo combinada `-j` conserva el
mismo contenido de los registros y los mismos valores de `Artifact`.
`--clobber` utiliza el manifiesto anterior para eliminar las salidas registradas, incluidos los
nombres anteriores, conservando los archivos de categoría no relacionados. Una recopilación normal de un solo volumen escribe los registros de `$MFT` y del Diario USN directamente en
`FileSystem/MFT.csv` y `FileSystem/UsnJournal.csv`. Los volúmenes recopilados adicionales usan
archivos hermanos numerados, como `FileSystem/MFT-2.csv` y `FileSystem/MFT-2-logs.txt`.

## JSON e ingesta en SIEM { #json-and-siem-ingestion }

Cada registro JSON o JSONL recibe dos campos de nivel superior:

- `Comp`: el sistema de origen.
- `Artifact`: el analizador de Juntei que generó el registro, como `mft` o `prefetch-timeline`.

`-j, --json-timeline <FILE>` crea un archivo JSONL adicional, independientemente del formato de los
resultados individuales. Se conservan todas las salidas clasificadas y sus registros:

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

El nombre de archivo se usa tal como se indica; las rutas relativas lo son respecto al directorio
de trabajo actual. Puede estar en la raíz de la salida, en otro subdirectorio de salida no
reservado o fuera del directorio de salida. No se pueden usar como destino las evidencias, los
manifiestos, los resultados o registros de cada equipo ni los directorios temporales de los
analizadores. Un archivo existente requiere `-C`; una exportación fallida lo deja intacto.

En **modo CSV**, Juntei realiza una pasada JSONL nativa adicional para que el archivo del SIEM
conserve los tipos numéricos y booleanos, los datos anidados, los campos sin procesar y los
metadatos que las tablas CSV pueden omitir. Los trabajos adicionales comparten el mismo límite de
concurrencia `-T`, no vuelven a extraer el contenido de la cuarentena y tienen registros auxiliares
`*-json-timeline-logs.txt` independientes. Ambos manifiestos distinguen su finalidad y su formato de
salida. Los archivos JSONL temporales se eliminan tras una ejecución correcta; se conservan si un
analizador o la exportación fallan. Tenga en cuenta el tiempo de procesamiento adicional y el
espacio temporal en disco.

En **modo JSON**, Juntei transmite los resultados existentes a JSONL compacto sin volver a analizar
las evidencias. Los informes de Amcache se expanden en los mismos registros individuales de
resumen, hallazgo, archivo y dispositivo que el JSONL nativo. En **modo JSONL**, los registros
anotados existentes se combinan directamente. Todos los modos usan la misma estructura de registro
para el SIEM. El conjunto de datos conserva las marcas de tiempo nativas y los campos de cada
artefacto; no inventa una marca de tiempo de evento común ni ordena globalmente artefactos
heterogéneos.

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json` registra la ruta `JSON-Timeline` solicitada y cualquier `JSON-TimelineError`. Sus
registros de tareas, y `manifest.csv`, siguen apuntando a las salidas de artefactos conservadas.
Los trabajos CSV correctos dedicados solo a la línea de tiempo hacen referencia al archivo JSONL
final en lugar de a los archivos temporales eliminados. Si algún analizador falla, la ejecución
termina con error y la línea de tiempo solo contiene los resultados correctos; revise los
manifiestos y los registros de las categorías antes de considerarla completa. El resumen de número
y tamaño de archivos abarca los archivos del directorio de salida; una línea de tiempo externa se
indica por separado como `JSONL timeline: <FILE>`.

## Rendimiento { #performance }

Los trabajos de los analizadores son procesos independientes, por lo que los artefactos que
consumen mucha CPU pueden ejecutarse al mismo tiempo y un daño en un analizador no puede detener la
ejecución completa. De forma predeterminada, Juntei utiliza el número de CPU lógicas. Limite la
concurrencia cuando las evidencias estén en un disco lento o el sistema tenga poca memoria:

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## Uso { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse the files inside every VHDX disk image in the evidence directory, each mounted read-only with FUSE while it is parsed

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Utilice un directorio de salida nuevo siempre que sea posible. `-C` elimina o sobrescribe solo los
destinos exactos generados para esta ejecución, de modo que un artefacto que ahora falte no pueda
dejar un resultado obsoleto. No elimina archivos no relacionados ni el árbol de salida completo.
También elimina, antes del análisis, el directorio `SecurityProducts/DefenderQuarantined/` de cada
equipo y el directorio reservado `defender-quarantined/` de la estructura plana anterior, tanto si
la nueva ejecución usa `-x` como si no.

Utilice un directorio de salida nuevo al pasar de una estructura plana antigua. Los archivos
CSV/JSON anteriores no se reubican ni se eliminan automáticamente.

!!! danger "Trate el contenido extraído de Defender como malware"
    `-x, --extract-defender-quarantined` escribe en disco el contenido en cuarentena dentro del
    directorio `SecurityProducts/DefenderQuarantined/` de cada equipo. Los flujos binarios recuperados terminan en `.quarantined`, y Juntei
    elimina el permiso de ejecución. Cada descriptor de seguridad binario recibe además un informe legible `.security_descriptor.txt`
    con la propiedad, los permisos y el SDDL. No ejecute nunca las cargas útiles recuperadas. Utilice
    un sistema de análisis aislado y cuente con que el antivirus o el software EDR alerten sobre el
    contenido recuperado o lo eliminen.

Consulte [cómo leer el descriptor de seguridad recuperado](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
para conocer los campos del informe y sus limitaciones forenses.

Auto-parse también lee las conexiones remotas de los usuarios (RDP, PuTTY, WinSCP y unidades
asignadas).
