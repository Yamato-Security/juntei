# Persistence Hunter { #persistence-hunter }

La persistencia permite que un programa vuelva a ejecutarse después de un reinicio, un inicio de sesión, un desencadenador programado u otro
evento. Un atacante puede utilizar con este fin la configuración normal de Windows. Muchos programas legítimos
utilizan exactamente los mismos mecanismos, por lo que encontrar una entrada de persistencia es un punto de partida para el análisis.

`juntei persistence-hunter` es una adaptación nativa en Rust de
[PyrsistenceSniper de Hexastrike](https://github.com/hexastrike/PyrsistenceSniper), con 131 comprobaciones: el
catálogo de 130 comprobaciones de PyrsistenceSniper más una comprobación nativa de COM del menú Inicio.

Para una explicación de cada mecanismo, sus ubicaciones de evidencia y cómo investigar un hallazgo,
consulte la [guía de persistencia de 131 comprobaciones](../persistence-hunter/index.md). Los
[agradecimientos de persistencia](../persistence-hunter/acknowledgements.md) describen los proyectos de Python y
PowerShell en los que se basa el catálogo.

## Uso { #usage }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Primer análisis { #first-scan }

```bash
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

La evidencia puede ser un volumen de Windows montado, una recopilación de Velociraptor o un directorio que contenga
varios equipos recopilados. Los equipos se mantienen separados durante el análisis, y cada resultado incluye
`Comp`. El escape de los nombres de archivo recopilados se decodifica al resolver las rutas de Windows.

Para un solo hive:

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

Un solo hive no puede proporcionar las comprobaciones que necesitan otro hive o evidencias del sistema de archivos. Lea el informe
de cobertura antes de considerar significativo un resultado vacío.

Enumere los ID de comprobación y sus correspondencias con ATT&CK:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

Utilice los ID de comprobación que muestra `--list-checks` o un ID de ATT&CK como `T1547.001`.
Un ID principal como `T1547` selecciona sus subtécnicas. `--technique` es un alias de `--check`.
Cada comprobación incluye en los hallazgos su técnica, su descripción y enlaces de referencia.

## Análisis en paralelo { #parallel-scans }

El comando utiliza un grupo compartido de trabajadores que, de forma predeterminada, tiene tantos trabajadores como CPU lógicas. Establezca
`-T, --threads <N>` para elegir el número de trabajadores, o `-T 1` para un análisis con un solo hilo:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

La detección de evidencias, las comprobaciones independientes, los metadatos de las cargas útiles y las fuentes de línea de tiempo de MFT/EVTX se ejecutan en paralelo. Los
hives del Registro cargados de un host se comparten en modo de solo lectura, y los metadatos en caché de una misma carga útil se inicializan una sola vez.
Los hosts se cargan de uno en uno para no multiplicar la memoria de los hives en toda la recopilación. Los hallazgos,
la cobertura, la selección de marcas de tiempo y el orden de las advertencias no varían con distintos números de hilos.

`auto-parse` ya ejecuta procesos de análisis en paralelo, por lo que asigna a cada proceso hijo de `persistence-hunter`
un trabajador. Su propia opción `-T` controla el número de procesos de análisis simultáneos.

## Evidencias que recopilar { #evidence-to-collect }

| Origen | Ejemplos de comprobaciones |
|---|---|
| `SYSTEM` y los registros de transacciones adyacentes | Servicios, comandos de recuperación de servicios, paquetes de LSA, configuración de arranque, proveedores de autenticación |
| `SOFTWARE` y los registros de transacciones adyacentes | Claves Run, Winlogon, IFEO, registros COM, TaskCache, configuración de Office |
| `NTUSER.DAT` y `UsrClass.dat` de cada usuario | Inicios automáticos del usuario, variables de entorno, invalidaciones de COM, configuración de Office |
| `SAM` y `SOFTWARE` | Anomalías de RID, cuentas ocultas, pertenencia inusual al grupo local Administradores |
| `Windows/System32/Tasks` | Comandos y controladores COM de las tareas programadas; comparación con TaskCache |
| Archivos `.library-ms` del menú Inicio de cada usuario y carpetas con nombre de CLSID | Candidatos a desencadenadores de activación COM, unidos con los registros de clases de usuario o de equipo coincidentes |
| Carpetas Startup, perfiles de PowerShell, plantillas de Office, configuración de Terminal | Archivos y comandos ejecutados por desencadenadores del usuario o del sistema |
| `OBJECTS.DATA` de WMI | Instancias de consumidores de eventos y enlaces de filtro a consumidor |
| `qmgr.db` o `qmgr*.dat` de BITS | Comandos de notificación recuperados y SID de los propietarios |
| `$MFT` y los archivos EVTX pertinentes | Correlación independiente de marcas de tiempo |
| Ejecutables, DLL y archivos de catálogo referenciados | Hashes SHA256 y metadatos descriptivos de PE y certificados |

Los registros de transacciones del Registro se reproducen cuando están disponibles. `--no-logs` lee los hives tal como se recopilaron;
un hive con cambios pendientes no reproducido puede omitir cambios recientes y se notifica en la cobertura. Para las comprobaciones del sistema activo
se utiliza el conjunto de controles actual. La comprobación de puerta trasera DSRM examina además
`ControlSet001`, `ControlSet002` y `CurrentControlSet` cuando existen; compare la ruta notificada
con `SYSTEM\\Select` antes de describir esa configuración como activa. Los demás conjuntos de controles históricos
pueden examinarse con los comandos individuales de análisis del Registro.

Las lecturas de hives comparten un presupuesto de entrada de 1 GiB por equipo. Los hives del equipo se intentan antes que los hives de usuario;
las entradas o los registros de transacciones que no caben se indican en la cobertura parcial. Los demás hives legibles y
las comprobaciones basadas en archivos continúan. Solo las ubicaciones canónicas de los hives o las exportaciones proporcionadas explícitamente establecen
la identidad de un hive; una copia situada en el directorio temporal de un usuario no puede sustituir al hive real de ese usuario.
La unidad del sistema original registrada en SOFTWARE se asigna al volumen del sistema capturado, de modo que montar
un volumen `C:` original con una letra de unidad de evidencia distinta no descarta los metadatos de las cargas útiles.

`auto-parse` también ejecuta el buscador y guarda `persistence/PersistenceHunter.csv` (o `.json`/`.jsonl`)
dentro del directorio de cada equipo, con `PersistenceHunter-logs.txt` a su lado. Sus hallazgos también se incorporan a
la línea de tiempo SIEM opcional de `-j <FILE>`. El análisis automático utiliza `--min-severity info` para conservar el
inventario completo de la configuración, incluidas las entradas que coinciden con reglas de permitidos.

## Registros COM y desencadenadores de activación { #com-registrations-and-activation-triggers }

La comprobación `scheduled_task_files` selecciona la entidad de seguridad de cada acción mediante el `Actions Context`
del XML de la tarea y el `Principal id` coincidente. Una entidad de seguridad de grupo se conserva como grupo;
no permite establecer qué registros de clases de qué usuario concreto se utilizarían.

Los candidatos a controlador COM se resuelven por separado en las vistas de clases nativa y de 32 bits. Solo se tienen en cuenta
el `UsrClass.dat`/`NTUSER.DAT` capturado del usuario coincidente y el `SOFTWARE` del equipo.
Las entidades de seguridad con SID se unen a través de las rutas de `ProfileList` capturadas. Los nombres de cuenta, incluidos
`HOST\user` y `DOMAIN\user`, dan preferencia a un perfil `user.DOMAIN` coincidente frente a un perfil
`user` simple. Los alias del host local y `.\user` utilizan los nombres de host capturados para la coincidencia de sufijos;
un sufijo exacto tiene prioridad sobre los demás alias del host. El perfil simple solo se tiene en cuenta
cuando no coincide ningún perfil cualificado. Cada una de estas coincidencias se etiqueta explícitamente como
**heurística por nombre de carpeta** en `Description`; no demuestra la titularidad de la cuenta ni la pertenencia al dominio. Las coincidencias ambiguas
quedan sin resolver y se notifican tanto en la descripción del hallazgo como en la cobertura; la ambigüedad
entre perfiles cualificados no provoca que se recurra al perfil simple. Esto no depende
de `Volatile Environment`, que normalmente solo existe en memoria y no se guarda con los hives del Registro. Un CLSID de usuario duplicado
oculta los elementos secundarios del CLSID del equipo. Las entidades de seguridad de grupo utilizan los candidatos COM del equipo y conservan
esa limitación en la descripción del hallazgo; una entidad de seguridad de grupo normal no hace por sí misma
que la cobertura sea parcial ni consume las entradas del resumen de errores.

El resolvedor sigue `InprocServer32` y `LocalServer32`, utiliza las rutas `CodeBase` administradas
(incluidas las subclaves de versión) y une el `LocalService` del AppID con el `ImagePath` del servicio
de SYSTEM recopilado. Las asociaciones de nombre de ejecutable del AppID se conservan, pero un nombre base no establece
la ruta completa de un ejecutable. Las URL `file:` locales de CodeBase se decodifican; las URL remotas se registran
sin acceder a ellas. Los registros ausentes conservan el CLSID con
`ResolutionStatus: unresolved_com_class`, los campos de ruta y hash vacíos y `Exists` desconocido.

La comprobación `start_menu_com` lee las URL del XML de los archivos `.library-ms` del menú Inicio de cada usuario y recopila
las carpetas con nombre de CLSID, incluidos los directorios vacíos. Notifica la ruta del desencadenador, el CLSID, el servidor
candidato y la procedencia en el Registro. Una adquisición solo de archivos no puede conservar un directorio vacío,
y la ausencia de hives de usuario impide una búsqueda completa de los registros por usuario. Los vínculos simbólicos
no se siguen. Se trata de candidatos de configuración: la arquitectura, la elevación, el UAC, las directivas y el
contexto de activación pueden influir en que un registro se utilice realmente. Corrobore la ejecución
con evidencias de procesos, de carga de módulos u otras evidencias independientes.

## Clasificación y perfiles { #classification-and-profiles }

La clasificación mínima predeterminada es `medium`. Para inspeccionar el inventario completo, incluidas
las entradas que coinciden con la lista de permitidos incluida:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "Utilice todas las gravedades al revisar recopilaciones incompletas"
    Una regla de permitidos amplia puede coincidir parcialmente con el comando de un atacante aunque su firmante falte
    o sea distinto. El proyecto original clasifica las coincidencias parciales como `LOW`, que el umbral predeterminado
    `medium` del comando manual oculta. Esto puede afectar a tareas programadas maliciosas situadas en directorios de Windows.
    Utilice `--min-severity info` para una investigación completa y revise conjuntamente `PolicyReason`, `SignerStatus`,
    el comando y la evidencia de origen. `auto-parse` ya conserva estas entradas.

| Clasificación | Significado |
|---|---|
| `HIGH` | Una regla de bloqueo configurada coincidió por completo |
| `MEDIUM` | No coincidió ninguna regla de permitidos ni de bloqueo |
| `LOW` | Las condiciones estrictas de una regla de permitidos coincidieron, pero los metadatos del firmante no coincidieron o no estaban disponibles |
| `INFO` | Una regla de permitidos coincidió por completo |

Se trata de clasificaciones de directiva, no de probabilidades medidas de compromiso. Una entrada `MEDIUM`
predeterminada no demuestra la presencia de malware. `INFO` no establece que un archivo tenga una firma válida ni que
un atacante no haya abusado de él. Una coincidencia parcial con la lista de permitidos queda por debajo del umbral predeterminado, por lo que debe incluir
`--min-severity low` o `info` al revisar recopilaciones escasas o al ajustar la lista de permitidos.

Proporcione un perfil YAML con `--profile profile.yaml`. **Sustituye** al perfil incluido. Las reglas globales
y por comprobación del archivo seleccionado se combinan; prevalece cualquier regla de bloqueo que coincida por completo.
Por ejemplo:

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

Las condiciones de regla admitidas son `path_matches`, `value_matches`, `hash` (64 dígitos hexadecimales de
SHA256), `not_lolbin` y `signer` (subcadena sin distinción de mayúsculas y minúsculas). Todas las condiciones estrictas indicadas deben
coincidir. Los metadatos del firmante son la condición flexible descrita anteriormente. Las expresiones regulares utilizan la sintaxis
de expresiones regulares de Rust: no se admiten las aserciones de búsqueda (lookaround) ni las referencias inversas. Los ID de comprobación desconocidos, las expresiones no válidas
y los campos de perfil desconocidos producen un error en lugar de debilitar silenciosamente la directiva.

## Campos de salida { #output-fields }

CSV es el formato de archivo predeterminado. Utilice `-t json` para obtener objetos JSON legibles o `-t jsonl` para una salida compacta
con un objeto por línea. Sin `-o`, un `-t` explícito escribe solo datos en stdout; las advertencias de cobertura
se envían a stderr. Sin ninguna de estas opciones, Juntei muestra un resumen y hasta 30 candidatos.

| Campos | Cómo interpretarlos |
|---|---|
| `Comp`, `CheckID`, `Technique`, `MitreID` | Qué equipo y qué mecanismo de persistencia describe la fila |
| `Path`, `Value` | La ubicación del Registro, el archivo, la tarea o el registro recuperado y su contenido configurado |
| `Description`, `References` | Por qué es importante el mecanismo y dónde obtener más información |
| `AccessGained` | El ámbito habitual del mecanismo (`USER` o `SYSTEM`; BITS también puede identificar `LOCAL_SERVICE`/`NETWORK_SERVICE`); no demuestra que un comando concreto obtuviera ese privilegio |
| `Severity`, `PolicyReason` | Clasificación del perfil, explicación de la directiva coincidente y cualquier matiz de coincidencia parcial o de firmante no verificado |
| `SourceFile`, `RecordStatus` | Origen recopilado exacto, con su ruta empezando por la entrada tal como se proporcionó (`./HOST/...` desde auto-parse), y contexto de asignación, reproducción o recuperación; sígalo hasta la evidencia original |
| `KeyLastWrite` | Marca de tiempo de la **clave** del Registro. No fecha un valor individual |
| `LastChange`, `ChangeSource`, `ChangeEvidence` | Hora del cambio correlacionada de forma independiente, prueba de origen y si la correlación tuvo éxito |
| `ResolvedPath`, `ResolutionStatus`, `Exists` | Ruta de la carga útil y resultado de la resolución offline; la evidencia no recopilada es desconocida, no `false` |
| `SHA256` | SHA256 de los bytes de la carga útil recopilada, cuando se pueden leer |
| `Signer`, `SignerStatus` | Metadatos descriptivos del certificado incrustado o del catálogo y su disponibilidad; no se verifican la confianza del certificado ni la validez de la firma |
| `IsLolbin`, `Launcher` | Si el iniciador o la carga útil configurados figuran en la lista LOLBAS incluida, y el iniciador utilizado para una carga útil delegada |
| `IsBuiltin`, `IsInOS-Directory` | Contexto para los comandos habituales de Windows y los directorios del sistema; ninguno de los dos campos establece la legitimidad |

Un comando `rundll32.exe` puede nombrar una carga útil DLL. Juntei conserva el iniciador mientras resuelve y
calcula el hash de la carga útil. Una referencia externa al equipo recopilado, una unidad ausente, una coincidencia ambigua
sin distinción de mayúsculas y minúsculas o una variable de entorno sin resolver no se sustituyen por un archivo del
equipo del analista.

## Marcas de tiempo y limitaciones { #timestamps-and-limitations }

El buscador no utiliza como marcas de tiempo del ataque las horas de modificación de los archivos copiados en la máquina virtual del analista.
Cuando están disponibles, correlaciona las escrituras en el Registro con el evento 13 de Sysmon, la instalación de tareas con
el 4698 de Security o el 106 del Programador de tareas, la instalación de servicios con el 7045 de System, y la configuración
basada en archivos con las marcas de tiempo originales de `$MFT`. La procedencia aparece en `ChangeSource`.
Un evento puede fechar un cambio de configuración sin demostrar que la carga útil configurada se ejecutara.
El evento 6011 del registro System local puede demostrar un nombre de equipo anterior. El buscador sigue las cadenas de cambio de nombre
verificadas y ancladas al equipo recopilado, conservando su valor `Comp` actual y registrando en `ChangeSource`
el equipo original del evento y la prueba del registro de cambio de nombre. Los nombres ajenos no demostrados,
los registros archivados, los registros reenviados y otra instalación de Windows no se aceptan mediante esta regla.

ComputerName/ActiveComputerName, el Hostname/NV Hostname de TCP/IP y el nombre de host/FQDN de la información del cliente capturados
proporcionan alias del host, lo que abarca las diferencias entre DNS y NetBIOS y los cambios de nombre pendientes. Si no hay ninguna identidad capturada,
un único nombre en los registros principales puede respaldar la datación con una procedencia explícita de identidad no verificada
y una advertencia de cobertura. Varios nombres no verificados no se atribuyen al host. Los nombres de Computer
rechazados se notifican como una limitación de la cobertura de la correlación; sus canales siguen marcándose como recopilados.

Utilice `--mft <FILE>` para seleccionar una MFT recopilada externamente para un único equipo. Utilice `--no-timeline`
para omitir la correlación de marcas de tiempo. `NOT_RUN`, `NOT_APPLICABLE`, `NO_ARTIFACT`, `NO_MATCH` y `REJECTED`
explican un `LastChange` vacío; un campo vacío no es evidencia de que la entrada sea inofensiva.

La recuperación de WMI y BITS es una recuperación (carving) acotada, no un analizador completo de transacciones de bases de datos.
Las cadenas o los enlaces recuperados pueden incluir datos antiguos o eliminados; estos registros deben corroborarse.
Los esquemas de clases de WMI por sí solos no se cuentan como suscripciones. Una entrada de TaskCache sin un archivo XML
de tarea recopilado es una discrepancia que revisar; puede que una recopilación de triaje simplemente haya omitido ese archivo.
Los metadatos de catálogo y PE no están verificados y pueden faltar en una recopilación. Las comprobaciones de sustitución
de archivos de accesibilidad dependen de metadatos de firma de referencia adecuados; la falta de evidencias de catálogo
no debe hacer que se marquen todos los ejecutables de accesibilidad.

La cobertura se registra para cada comprobación, incluidas las comprobaciones sin hallazgos. `complete` significa que se leyó
el origen recopilado aplicable; no demuestra que la recopilación esté completa ni que
el equipo no tenga otra persistencia. `partial`, `not_collected`, `failed` y `disabled` explican
los límites. Conserve `--coverage Coverage.csv` y el registro del analizador junto con sus hallazgos.

La adaptación utiliza los formatos CSV/JSON/JSONL de Juntei. No llama al programa Python original, no consulta
servicios de enriquecimiento en línea ni reproduce la interfaz HTML/XLSX del proyecto original.

## Más información { #further-details }

- [Implementación y perfiles de PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper)
- [Técnicas de persistencia de MITRE ATT&CK](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft: Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [Inventario de inicios automáticos del Registro](../artifacts/persistence.md)
- [Servicios](../artifacts/services.md) y [TaskCache](../artifacts/taskcache.md)
