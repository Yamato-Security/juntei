# Persistence Hunter { #persistence-hunter }

La persistencia es una forma de conseguir que un código vuelva a ejecutarse: cuando se inicia Windows, cuando un usuario inicia sesión, cuando se abre una
aplicación, cuando llega una hora programada o cuando se produce otro evento. Los atacantes pueden abusar de la misma configuración
que el software legítimo utiliza para las actualizaciones, la compatibilidad con dispositivos, la seguridad y la administración.

`juntei persistence-hunter` examina offline evidencias de Windows capturadas mediante **131 comprobaciones** que abarcan
estos mecanismos y las anomalías relacionadas con cuentas y protecciones. Lee hives del Registro, archivos de tareas,
scripts, configuraciones de aplicaciones y determinados registros de bases de datos.

Esta página explica cada comprobación implementada. Para conocer todas las opciones, los campos de salida, los perfiles y el
tratamiento de las marcas de tiempo, abra la [guía del comando](../commands/persistence-hunter.md). Los
[agradecimientos](acknowledgements.md) enlazan al proyecto PyrsistenceSniper de Python y a la herramienta original
PersistenceSniper de PowerShell.

## Iniciar una investigación { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

Utilice `--min-severity info` para obtener el inventario completo de candidatos. El comando manual usa de forma predeterminada
`medium`; su perfil puede clasificar las coincidencias parciales con la lista de permitidos como `LOW` incluso cuando la falta de evidencias de la carga útil o del
firmante impide una comparación completa. Ni una coincidencia con la lista de permitidos ni una entrada desconocida determinan
si algo es malicioso. `auto-parse` ya conserva todas las gravedades y escribe los hallazgos
en la categoría `persistence/` de cada equipo.

Para centrarse en un mecanismo, copie su **ID de comprobación** de la lista siguiente:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

De forma predeterminada, el comando utiliza un grupo compartido de trabajadores. Añada `-T 4` para seleccionar cuatro trabajadores, o `-T 1`
para un análisis con un solo hilo. La entrada puede contener varios equipos; sus evidencias se mantienen separadas.

## Leer los hallazgos como evidencias { #read-findings-as-evidence }

| Pregunta | Qué inspeccionar |
|---|---|
| ¿Qué se configuró y dónde? | `CheckId`, `Path` y el `Value` completo, incluidos los argumentos. Siga `SourceFile` hasta el hive, el archivo o la base de datos capturados. |
| ¿Existe el destino en la recopilación? | `ResolvedPath`, `ResolutionStatus` y `Exists`. Las referencias no recopiladas, ambiguas o no compatibles son desconocidas; Juntei no las sustituye por un archivo del equipo del analista. |
| ¿Es efectiva la configuración? | El desencadenador real, los indicadores de habilitación, el conjunto de controles seleccionado, la versión de Windows o de la aplicación, el usuario y la arquitectura. Algunas comprobaciones conservan deliberadamente configuraciones inactivas. |
| ¿Se trata de un ataque? | `Severity` y `PolicyReason` explican la coincidencia con el perfil, no un veredicto. Compare con el software esperado, el comando completo, los hashes de los archivos, evidencias independientes de ejecución y la línea de tiempo circundante. |
| ¿Quién podría ejecutarlo? | `AccessGained` es una indicación de alcance y contexto. La configuración de todo el equipo suele indicar `SYSTEM`, pero eso no demuestra que el proceso iniciado se ejecutara como SYSTEM. Las entidades de seguridad de las tareas y los SID del propietario de BITS aportan un contexto más específico cuando están disponibles. |
| ¿Cuándo cambió? | `KeyLastWrite` fecha toda la clave del Registro. `LastChange`, `ChangeSource` y `ChangeEvidence` describen una correlación independiente con MFT/EVTX, cuando está disponible. Ninguno de estos campos fecha automáticamente la ejecución de la carga útil. |
| ¿Qué fiabilidad tiene el estado recuperado? | Lea `RecordStatus`, `Coverage.csv` y las advertencias sobre evidencias reproducidas, recuperadas mediante carving, dañadas, ausentes o limitadas. Un registro histórico recuperado puede haber dejado de estar activo. |
| ¿Es fiable un editor con nombre? | `Signer` y `SignerStatus` son metadatos descriptivos del PE, del certificado o del catálogo. Juntei no valida las firmas Authenticode ni la confianza de los certificados. |

La ausencia de hallazgos no demuestra la ausencia de persistencia. Una fila de cobertura `complete` significa
que se leyó el origen recopilado aplicable, no que la adquisición contenga todos los archivos relevantes,
todos los usuarios o todos los mecanismos de persistencia posibles de Windows. Los demás estados de cobertura explican las comprobaciones ausentes,
parciales, deshabilitadas o fallidas. El hunter no analiza la memoria en vivo, la red ni las identidades en la nube.

## Catálogo y notación de ubicaciones { #catalogue-and-location-notation }

El catálogo se basa en PyrsistenceSniper, más la comprobación nativa `start_menu_com`;
las descripciones siguientes documentan
el comportamiento nativo de Juntei, incluidas las limitaciones de sus evidencias. Hay **116 comprobaciones basadas en el
Registro y 15 comprobaciones de archivos o repositorios**. Algunas comprobaciones de archivos también combinan datos del Registro, por lo que se trata de
recuentos de implementación y no de requisitos de adquisición mutuamente excluyentes. Los mecanismos
se agrupan a continuación según su finalidad investigativa y, dentro de cada grupo, se ordenan alfabéticamente por ID de comprobación.

- `HKLM\SYSTEM`, `HKLM\SOFTWARE` y `HKLM\SAM` hacen referencia a hives del equipo capturados, recopilados normalmente
  de `Windows\System32\config`, con sus registros de transacciones cuando están disponibles.
- `HKU\<user>` designa el hive de usuario capturado correspondiente. La mayor parte de la configuración de usuario procede de NTUSER.DAT;
  los registros de clases pueden proceder de `AppData\Local\Microsoft\Windows\UsrClass.dat`, según se indique.
- `<ControlSet>` designa el conjunto de controles activo seleccionado en SYSTEM. Las ubicaciones históricas adicionales de la comprobación de DSRM
  se indican explícitamente más abajo. Consulte `SYSTEM\Select` al revisarlas.
- `<name>` es un componente variable de la ruta. En las tablas de destinos del Registro, `<child>` designa los registros
  secundarios inmediatos; no implica un recorrido recursivo ilimitado. Las comprobaciones recursivas explícitas
  describen su recorrido por separado. Los corchetes en las rutas de Office indican una versión opcional.
- Las tablas enumeran las vistas reales del equipo y del usuario que selecciona la comprobación. Se indica una vista de 32 bits
  cuando está implementada; no suponga que todas las comprobaciones examinan todas las arquitecturas o vistas posibles.
- Las comprobaciones genéricas de valores emiten las entradas configuradas no vacías, incluidos los elementos de cadenas múltiples y
  el cero numérico cuando corresponda. Por sí mismas, no requieren una carga útil maliciosa, un
  desencadenador habilitado ni una ejecución correcta. Los predicados más selectivos se indican en cada comprobación.

Los ID de ATT&CK son asignaciones heredadas del catálogo y enlaces para profundizar en el estudio. Describen un posible
abuso, no la constatación de que un atacante empleara esa técnica; las comprobaciones de configuración de protecciones y de diagnóstico
pueden describir condiciones habilitantes en lugar de un mecanismo de persistencia independiente.

| Grupo de mecanismos | Comprobaciones |
|---|---:|
| [Arranque, inicio de sesión e inicio](#boot-sign-in-and-startup) | 32 |
| [Servicios y extensiones del sistema](#services-and-system-extensions) | 13 |
| [Tareas programadas](#scheduled-tasks) | 3 |
| [WMI, BITS y otros desencadenadores de eventos](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorador, COM y controladores de archivos o URL](#explorer-com-and-file-or-url-handlers) | 15 |
| [Aplicaciones y complementos de Office](#office-applications-and-add-ins) | 7 |
| [Depuradores, controladores de bloqueos y shims de compatibilidad](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [Configuración del runtime y del generador de perfiles de .NET](#net-runtime-and-profiler-settings) | 7 |
| [Invalidaciones de la búsqueda de DLL y ejecutables](#dll-and-executable-lookup-overrides) | 18 |
| [Cuentas y visibilidad en el inicio de sesión](#accounts-and-sign-in-visibility) | 4 |
| [Componentes de autenticación y protección de credenciales](#authentication-components-and-credential-protection) | 8 |
| **Total** | **131** |

## Arranque, inicio de sesión e inicio { #boot-sign-in-and-startup }

Esta configuración hace que un comando o una biblioteca se ejecute durante el arranque, el inicio de sesión o la inicialización del escritorio. Revise la versión de Windows y el usuario aplicable: un ajuste puede estar obsoleto, deshabilitado o destinado a un quiosco y no a un atacante.

### Active Setup {#active_setup}

**ID de comprobación:** `active_setup` · **ATT&CK del catálogo:** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**Qué comprueba Juntei:** lee `StubPath` de los registros de componentes de Active Setup de todo el equipo en las vistas nativa y de 32 bits. Windows puede usar estos comandos al preparar el perfil de un usuario; la comprobación omite los valores de marcador de posición literales `U` y `/UserInstall`.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` y el árbol `Wow6432Node` correspondiente.

**Cómo investigar:** compare cada componente con su producto instalado e inspeccione los argumentos del comando. Un registro de todo el equipo puede dar lugar a una ejecución en la sesión de un usuario que inicia sesión; la etiqueta `SYSTEM` de la salida describe el alcance de la configuración, no constituye una prueba del token del proceso resultante.

### Boot Execute {#boot_execute}

**ID de comprobación:** `boot_execute` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee la cadena múltiple `BootExecute` de Session Manager y conserva cada entrada no vacía. Registra los comandos configurados para una fase temprana del arranque.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**Cómo investigar:** `autocheck autochk *` es un valor predeterminado conocido, y las herramientas de disco pueden añadir entradas legítimas. Investigue los comandos añadidos y sus archivos sin suponer que todas las entradas no predeterminadas se ejecutaron o son maliciosas.

### Programa de verificación de arranque {#boot_verification_program}

**ID de comprobación:** `boot_verification_program` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee el `ImagePath` de `BootVerificationProgram`, una ubicación de ejecutable asociada a la verificación del arranque.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**Cómo investigar:** compruebe si este mecanismo antiguo es aplicable a la versión de Windows y a la configuración de arranque capturadas. Resuelva el archivo de destino y compare su finalidad con el software de administración del arranque esperado.

### Valor Load del Explorador {#explorer_load}

**ID de comprobación:** `explorer_load` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee el valor `Load` asociado a la configuración de inicialización del inicio de sesión de Windows en los hives del equipo y de los usuarios.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**Cómo investigar:** investigue los comandos desconocidos, los destinos en los que el usuario puede escribir y los argumentos que descargan o decodifican contenido. Confirme la aplicabilidad y la ejecución por separado; un resultado vacío no abarca otros mecanismos de inicio de sesión.

### Controladores de fuentes {#font_drivers}

**ID de comprobación:** `font_drivers` · **ATT&CK del catálogo:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Qué comprueba Juntei:** enumera los valores no vacíos de la clave heredada del Registro `Font Drivers` como posibles registros de bibliotecas.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | Todos los valores no vacíos |

**Cómo investigar:** trate esto como una superficie de configuración heredada poco habitual. Compruebe la compatibilidad con la versión de Windows y la biblioteca referenciada antes de concluir que podría cargarse, y corrobore el uso real con evidencias de procesos o de carga de imágenes.

### DLL de extensiones de directiva de grupo {#gp_extension_dlls}

**ID de comprobación:** `gp_extension_dlls` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee `DllName` de cada registro inmediato de extensión de directiva de grupo bajo Winlogon. Estas DLL amplían el procesamiento de directivas.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**Cómo investigar:** los productos de administración y de seguridad pueden instalar extensiones. Haga coincidir el GUID de la extensión y la DLL con el producto esperado, y examine las evidencias de procesamiento de directivas y de carga de imágenes antes de afirmar que hubo ejecución.

### Scripts de directiva de grupo {#gp_scripts}

**ID de comprobación:** `gp_scripts` · **ATT&CK del catálogo:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Qué comprueba Juntei:** analiza los pares de comando y parámetros de los archivos de registro `scripts.ini` y `psscripts.ini` de la directiva de grupo local capturados, incluidos los almacenes de directivas locales por usuario. Las secciones de inicio y apagado, y de inicio y cierre de sesión, conservan sus comandos y argumentos configurados, y las secciones del equipo se distinguen de las del usuario.

**Evidencias examinadas:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` y los archivos `GroupPolicy\User\Scripts` equivalentes.
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`.

**Cómo investigar:** solo se examinan las ubicaciones de registro de directivas locales implementadas; no se trata de una auditoría completa de SYSVOL/GPO. Un archivo de script ubicado en otro lugar de un directorio de directivas no es suficiente. Correlacione su registro con evidencias del procesamiento de directivas y de ejecución.

### Scripts de inicio de sesión (UserInitMprLogonScript) {#logon_scripts}

**ID de comprobación:** `logon_scripts` · **ATT&CK del catálogo:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Qué comprueba Juntei:** lee el valor de entorno `UserInitMprLogonScript` del usuario, que puede seleccionar un script para el inicio de sesión de ese usuario.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**Cómo investigar:** compare con los scripts de inicio de sesión aprobados e inspeccione el script real. El valor no demuestra un inicio de sesión ni una ejecución posteriores, y los scripts proporcionados por el dominio requieren evidencias de directivas adicionales.

### Platform Execute {#platform_execute}

**ID de comprobación:** `platform_execute` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee cada entrada no vacía de `PlatformExecute` en Session Manager. Identifica los programas seleccionados para la inicialización del arranque relacionada con la plataforma.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**Cómo investigar:** los flujos de trabajo de hardware o de aprovisionamiento pueden explicar una entrada. Determine si la ruta de arranque de esta compilación de Windows la utiliza e investigue la imagen referenciada en lugar de deducir un ataque a partir del nombre del valor.

### DLL de extensión del cliente RDP {#rdp_clx_dll}

**ID de comprobación:** `rdp_clx_dll` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee `ClxDllPath` en la configuración de usuario predeterminada de Terminal Server como posible ubicación de una extensión relacionada con RDP.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**Cómo investigar:** compruebe la versión de Windows y de RDP y las extensiones instaladas. Una DLL configurada es una pista para el análisis de carga de imágenes; el valor del Registro por sí solo no muestra que se produjera una conexión RDP ni la carga de la DLL.

### DLL de canal virtual de RDP {#rdp_virtual_channel}

**ID de comprobación:** `rdp_virtual_channel` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** enumera los valores bajo los registros `Default\AddIns` de Terminal Server Client de todo el equipo. Los valores binarios se decodifican como UTF-16LE cuando es posible, con lo que se conservan las rutas de extensión utilizables.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: todos los valores.

**Cómo investigar:** los clientes RDP y los productos de redirección de dispositivos registran complementos de forma legítima. Inspeccione la DLL o el comando reales y la sesión de cliente correspondiente; no todos los valores de un complemento son rutas de ejecutables.

### Programas de inicio de RDP WDS {#rdp_wds_startup}

**ID de comprobación:** `rdp_wds_startup` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee `StartupPrograms` de la configuración del controlador de estación de Windows de RDP. Identifica los programas configurados para el inicio de las sesiones remotas.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**Cómo investigar:** `rdpclip` es una entrada conocida. Investigue las adiciones y la configuración de RDP del host, y utilice los registros de sesiones y procesos para determinar si el programa configurado llegó a iniciarse.

### Claves Run del Registro {#run_keys}

**ID de comprobación:** `run_keys` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** enumera los valores Run, RunOnce, RunEx, RunOnceEx y Run de las directivas del Explorador en los hives del equipo y de los usuarios, además de las claves de 32 bits del equipo indicadas. Las secciones RunEx/RunOnceEx se recorren con límites y comprobaciones de ciclos.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Todos los valores no vacíos |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Todos los valores no vacíos |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Todos los valores no vacíos |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Todos los valores no vacíos |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Todos los valores no vacíos |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Todos los valores no vacíos |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Todos los valores no vacíos |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Todos los valores no vacíos |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Todos los valores no vacíos |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Todos los valores no vacíos |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | Todos los valores no vacíos |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | Todos los valores no vacíos |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | Todos los valores no vacíos |

**Cómo investigar:** los actualizadores de software y las aplicaciones de la bandeja del sistema aparecen aquí con frecuencia. Compare el ejecutable, los argumentos del comando y el contexto del usuario. Una entrada RunOnce puede desaparecer tras su uso; el estado capturado no es un historial completo de los inicios de sesión anteriores.

### RunServices (heredado) {#run_services}

**ID de comprobación:** `run_services` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** enumera los valores de la clave heredada `RunServices` de todo el equipo.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | Todos los valores no vacíos |

**Cómo investigar:** se trata de una superficie de configuración histórica, no de una garantía de que una compilación moderna de Windows la ejecute. Investigue el destino y la aplicabilidad al sistema operativo y, después, busque evidencias de ejecución independientes.

### RunServicesOnce (heredado) {#run_services_once}

**ID de comprobación:** `run_services_once` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** enumera los valores de la clave heredada `RunServicesOnce` de todo el equipo.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | Todos los valores no vacíos |

**Cómo investigar:** trate los valores no vacíos como pistas de configuración. Compruebe si la versión de Windows de origen admite el mecanismo y si el software de implementación o de compatibilidad explica la entrada.

### Comando inicial de S0 {#s0_initial_command}

**ID de comprobación:** `s0_initial_command` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee `S0InitialCommand` de Session Manager como posible comando de inicialización de la sesión 0.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**Cómo investigar:** determine la aplicabilidad a la compilación de Windows y al estado de arranque capturados. Un comando configurado no es un registro con marca de tiempo de una ejecución durante el arranque; inspeccione su archivo y correlaciónelo con la telemetría de arranque y de procesos.

### Session Manager Execute {#session_manager_execute}

**ID de comprobación:** `session_manager_execute` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee las entradas `Execute` no vacías de Session Manager, lo que pone de manifiesto los programas seleccionados para la inicialización de la sesión.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**Cómo investigar:** compare con la referencia de imágenes esperada e inspeccione los ejecutables nativos o las rutas inusuales. Determine si se produjo un arranque o una sesión relevantes después del cambio antes de afirmar que hubo ejecución.

### Session Manager SubSystems {#session_manager_subsystems}

**ID de comprobación:** `session_manager_subsystems` · **ATT&CK del catálogo:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Qué comprueba Juntei:** lee el valor `Windows` de la clave `SubSystems` de Session Manager, que configura el comando del subsistema de Windows.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**Cómo investigar:** el comando normal es largo y depende de la versión. Compare el valor completo y los argumentos con una referencia de confianza de la misma compilación; modificar un ajuste crítico del subsistema también puede impedir el inicio.

### Setup Execute {#setup_execute}

**ID de comprobación:** `setup_execute` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee las entradas `SetupExecute` de Session Manager, que abarcan los comandos relacionados con la instalación configurados para el inicio.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**Cómo investigar:** la actividad de instalación, actualización y recuperación puede crear entradas legítimas. Compare con los registros de instalación y con la imagen referenciada antes de atribuir la configuración a la persistencia.

### Redirección de Startup en Shell Folders {#shell_folders_startup}

**ID de comprobación:** `shell_folders_startup` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** compara `Startup` y `Common Startup` tanto en Shell Folders como en User Shell Folders con las ubicaciones de Startup predeterminadas. Se notifican las redirecciones no predeterminadas y se inspecciona el contenido recopilado de esas carpetas.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` y `User Shell Folders`: `Common Startup`.
- La configuración equivalente de `Software\Microsoft\Windows\CurrentVersion\Explorer` en NTUSER.DAT: `Startup`, además de los archivos capturados en las ubicaciones redirigidas.

**Cómo investigar:** una carpeta Startup redirigida puede ocultar elementos de inicio automático, pero la redirección puede ser legítima. La ruta debe resolverse dentro de las evidencias capturadas; un destino no recopilado es desconocido, no una prueba de que la carpeta estuviera vacía.

### Invalidación del shell (Shell Launcher) {#shell_launcher}

**ID de comprobación:** `shell_launcher` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee el valor `Shell` de las directivas y la asignación Shell de arranque de `system.ini` en SOFTWARE. Son superficies alternativas de selección del shell.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**Cómo investigar:** los quioscos y los dispositivos de aplicación pueden sustituir el Explorador de forma intencionada. Compare el shell seleccionado con la función del equipo y confirme que la configuración era efectiva en la sesión pertinente.

### Carpeta Startup {#startup_folder}

**ID de comprobación:** `startup_folder` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** inspecciona los archivos directos de las carpetas Startup predeterminadas por usuario y comunes, siguiendo la primera redirección de la carpeta Startup disponible. Los destinos de los accesos directos se recuperan cuando es posible analizar un `.lnk` recopilado.

**Evidencias examinadas:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` y `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`.
- Las redirecciones `Common Startup` / `Startup` aplicables en la configuración de Shell Folders y User Shell Folders del equipo y de los usuarios.

**Cómo investigar:** los vínculos y archivos de Startup pueden iniciar aplicaciones legítimas al iniciar sesión. Inspeccione tanto los argumentos de destino como los nombres de archivo. La carpeta común es una ubicación de configuración compartida; no demuestra que el programa iniciado utilizara un token de SYSTEM.

### Programa inicial de Terminal Services {#ts_initial_program}

**ID de comprobación:** `ts_initial_program` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee la directiva `InitialProgram` de Terminal Services y la configuración Run/RunOnce/RunOnceEx de la instalación de Terminal Server del equipo, incluidas las secciones RunOnceEx anidadas con límites.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | Todos los valores no vacíos |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | Todos los valores no vacíos |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | Todos los valores no vacíos |

**Cómo investigar:** las implementaciones de aplicaciones remotas pueden utilizar esta configuración de forma intencionada. Determine las sesiones afectadas y siga los comandos seleccionados en las evidencias de procesos; el registro por sí solo no muestra un inicio de sesión remoto.

### Winlogon AppSetup {#winlogon_appsetup}

**ID de comprobación:** `winlogon_appsetup` · **ATT&CK del catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Qué comprueba Juntei:** lee `AppSetup` de Winlogon y divide las entradas separadas por comas en candidatos independientes.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**Cómo investigar:** compare cada componente con el software de instalación o de inicio de sesión de Windows esperado. Una entrada de la lista no es una ejecución observada independiente, y su relevancia puede depender de la versión de Windows y del estado de la implementación.

### Winlogon GinaDLL {#winlogon_gina_dll}

**ID de comprobación:** `winlogon_gina_dll` · **ATT&CK del catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Qué comprueba Juntei:** lee el ajuste heredado de sustitución `GinaDLL` de Winlogon. GINA era un modelo antiguo de extensión del inicio de sesión de Windows.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**Cómo investigar:** es relevante principalmente en sistemas anteriores a Vista. Una clave residual en un sistema más reciente no demuestra que haya una extensión de inicio de sesión efectiva; inspeccione la versión de origen y cualquier DLL referenciada.

**Más detalles:** [documentación de Microsoft](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina).

### Winlogon MPNotify {#winlogon_mpnotify}

**ID de comprobación:** `winlogon_mpnotify` · **ATT&CK del catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Qué comprueba Juntei:** lee el ajuste de comando o biblioteca `mpnotify` de Winlogon, una superficie de extensión relacionada con el inicio de sesión.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**Cómo investigar:** compare el destino y los argumentos con una referencia de Windows equivalente. Compruebe el comportamiento específico de la versión y correlaciónelo con un inicio de sesión posterior antes de afirmar que se ejecutó código.

### Paquetes Notify de Winlogon {#winlogon_notify_packages}

**ID de comprobación:** `winlogon_notify_packages` · **ATT&CK del catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Qué comprueba Juntei:** lee `DllName` de cada registro inmediato de paquetes `Notify` de Winlogon.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**Cómo investigar:** los paquetes de notificación son un mecanismo de extensión heredado. Determine si la versión de Windows capturada los carga y distinga el software de autenticación aprobado de las bibliotecas sin explicación.

### Winlogon Shell {#winlogon_shell}

**ID de comprobación:** `winlogon_shell` · **ATT&CK del catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Qué comprueba Juntei:** lee los valores `Shell` de Winlogon de todo el equipo y por usuario para poner de manifiesto la selección del shell de escritorio.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**Cómo investigar:** el Explorador es lo habitual, pero los quioscos pueden utilizar otro shell. Investigue los comandos añadidos o las rutas inusuales y determine qué configuración efectiva de usuario se aplicó al iniciar sesión.

### Winlogon System {#winlogon_system}

**ID de comprobación:** `winlogon_system` · **ATT&CK del catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Qué comprueba Juntei:** lee el valor `System` de Winlogon y separa las entradas delimitadas por comas en candidatos individuales.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**Cómo investigar:** trátelo como una pista de configuración del inicio de sesión que depende de la versión. Inspeccione cada componente referenciado y corrobórelo con evidencias de ejecución en lugar de tratar la lista como un historial de ejecuciones.

### Winlogon Taskman {#winlogon_taskman}

**ID de comprobación:** `winlogon_taskman` · **ATT&CK del catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Qué comprueba Juntei:** lee los valores `Taskman` de Winlogon del equipo y de los usuarios en busca de un comando alternativo de administración de tareas.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**Cómo investigar:** determine qué comportamiento de Windows utilizaría el ajuste y si el destino es el esperado. Una sustitución configurada no demuestra que un usuario invocara esa función.

### Winlogon Userinit {#winlogon_userinit}

**ID de comprobación:** `winlogon_userinit` · **ATT&CK del catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Qué comprueba Juntei:** lee los valores `Userinit` de Winlogon del equipo y de los usuarios, y divide los programas separados por comas ignorando los elementos finales vacíos.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**Cómo investigar:** una entrada normal suele apuntar a `userinit.exe`. Inspeccione las entradas adicionales y las rutas modificadas. Confirme el usuario aplicable y un inicio de sesión posterior; `KeyLastWrite` corresponde a toda la clave, no específicamente a este valor.

### Winlogon VMApplet {#winlogon_vmapplet}

**ID de comprobación:** `winlogon_vmapplet` · **ATT&CK del catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Qué comprueba Juntei:** lee el ajuste de comando `VMApplet` de Winlogon, que abarca una superficie antigua de inicio de applets del sistema.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**Cómo investigar:** compruebe la aplicabilidad al sistema operativo y el applet esperado. Un comando inesperado merece investigarse, pero no establece que se abriera la interfaz desencadenante.

## Servicios y extensiones del sistema { #services-and-system-extensions }

Los servicios de Windows y los componentes extensibles del sistema ofrecen muchas formas legítimas de cargar ejecutables o DLL. Compare el destino configurado con el producto esperado, la cuenta de servicio, la configuración de inicio y el historial de instalación respaldado por las fuentes.

### DLL de DiagTrack {#diagtrack_dll}

**ID de comprobación:** `diagtrack_dll` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee el `ImagePath` del servicio DiagTrack. A pesar de que el nombre del catálogo hace referencia a una DLL, esta comprobación examina el comando de imagen configurado del servicio.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**Cómo investigar:** compare con la referencia del servicio y con las comprobaciones generales de servicios. Una misma configuración puede producir más de un resultado de comprobación; los recuentos de candidatos no son recuentos de ataques distintos.

### DLL de DiagTrack Listener {#diagtrack_listener_dll}

**ID de comprobación:** `diagtrack_listener_dll` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee `FileName` de la configuración del registrador automático DiagTrack-Listener. Se trata de un ajuste del registrador automático de ETW, no de una evidencia de que se haya cargado una DLL.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**Cómo investigar:** inspeccione el valor sin procesar y la configuración de registro de Windows. El catálogo lo etiqueta como una técnica de DLL, pero la búsqueda no valida que haya una DLL cargable ni demuestra una ejecución a través de este valor.

### DLL auxiliar de Netsh {#netsh_helper}

**ID de comprobación:** `netsh_helper` · **ATT&CK del catálogo:** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**Qué comprueba Juntei:** enumera los valores no vacíos bajo la clave de registro de aplicaciones auxiliares `NetSh` del equipo. Las aplicaciones auxiliares pueden ampliar `netsh.exe` con comandos basados en DLL.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | Todos los valores no vacíos |

**Cómo investigar:** los productos de red pueden instalar aplicaciones auxiliares. Haga coincidir cada valor con un producto esperado e inspeccione la biblioteca resuelta; después, busque ejecuciones de netsh o cargas de imágenes relevantes.

### Monitores de impresión {#print_monitors}

**ID de comprobación:** `print_monitors` · **ATT&CK del catálogo:** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**Qué comprueba Juntei:** lee `Driver` bajo cada registro inmediato de monitor de impresión en el conjunto de controles activo.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**Cómo investigar:** el software de impresión y los monitores de puerto amplían legítimamente el administrador de trabajos de impresión. Compare la DLL y los metadatos del producto con el inventario de impresoras, y confirme la carga con evidencias del administrador de trabajos de impresión o de carga de imágenes.

### Procesadores de impresión {#print_processors}

**ID de comprobación:** `print_processors` · **ATT&CK del catálogo:** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**Qué comprueba Juntei:** lee `Driver` en los árboles de registro de procesadores de impresión x64 y x86.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**Cómo investigar:** las suites de impresoras de terceros pueden añadir procesadores además de los predeterminados conocidos de Windows. Revise ambas arquitecturas, la DLL real y el software de impresora instalado antes de atribuir una intención maliciosa.

### Extensión del Administrador de control de servicios {#scm_extension}

**ID de comprobación:** `scm_extension` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** enumera los valores no vacíos de `ServiceControlManagerExtension`, una superficie de extensión del administrador de servicios.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | Todos los valores no vacíos |

**Cómo investigar:** examine las bibliotecas nombradas y si esta compilación de Windows utiliza el ajuste. Correlacione con la actividad de arranque y de servicios; la comprobación inventaría registros en lugar de demostrar que se cargó una extensión.

### DLL de complemento de nivel de servidor DNS {#server_level_plugin_dll}

**ID de comprobación:** `server_level_plugin_dll` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee el ajuste `ServerLevelPluginDll` del servicio DNS en el conjunto de controles activo de SYSTEM.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**Cómo investigar:** es relevante en los sistemas que ejecutan el rol de servidor DNS, incluidos algunos controladores de dominio. Investigue la DLL, la cuenta de servicio y las evidencias de reinicio o carga del servicio; un rol inactivo o ausente cambia la conclusión.

### Comando de error del servicio {#service_failure_command}

**ID de comprobación:** `service_failure_command` · **ATT&CK del catálogo:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Qué comprueba Juntei:** lee el `FailureCommand` de cada servicio en el conjunto de controles activo, lo que pone de manifiesto un comando de recuperación configurado.

**Evidencias examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`.

**Cómo investigar:** el comando solo se ejecuta cuando las acciones de recuperación del servicio lo seleccionan y se produce el error correspondiente. Inspeccione la configuración de recuperación y los registros de errores y procesos; el valor por sí solo no establece que haya una acción de recuperación activa.

### DLL de agente de extensión SNMP {#snmp_extension_agent}

**ID de comprobación:** `snmp_extension_agent` · **ATT&CK del catálogo:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Qué comprueba Juntei:** enumera los valores bajo la clave de registro `ExtensionAgents` del servicio SNMP. Los valores pueden identificar otra ubicación del Registro en lugar de una ruta directa a una DLL.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | Todos los valores no vacíos |

**Cómo investigar:** siga manualmente un registro indirecto hasta su biblioteca y verifique que SNMP esté instalado y se utilice. Una cadena sin resolver en esta salida no demuestra que la DLL falte o sea maliciosa.

### Proveedores de hora {#time_providers}

**ID de comprobación:** `time_providers` · **ATT&CK del catálogo:** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**Qué comprueba Juntei:** lee `DllName` de los registros inmediatos `TimeProviders` de Windows Time.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**Cómo investigar:** revise por separado los ajustes de habilitación del proveedor y el estado del servicio Windows Time; la comprobación de la ruta de la DLL no requiere un proveedor habilitado. Los productos de sincronización horaria pueden proporcionar sustitutos legítimos.

### Servicio de Windows (ServiceDll) {#windows_service_dll}

**ID de comprobación:** `windows_service_dll` · **ATT&CK del catálogo:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Qué comprueba Juntei:** lee `Parameters\ServiceDll` bajo los servicios del conjunto de controles activo de SYSTEM y adjunta evidencias de marcas de tiempo específicas del servicio cuando están disponibles.

**Evidencias examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`.

**Cómo investigar:** un host compartido como `svchost.exe` no es la DLL de la carga útil. Inspeccione la biblioteca, la cuenta de servicio, el estado de inicio y las evidencias correspondientes de instalación del servicio o de carga de imágenes; un registro no demuestra que el servicio se ejecutara.

**Más detalles:** [documentación de Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Servicio de Windows (ImagePath) {#windows_service_image_path}

**ID de comprobación:** `windows_service_image_path` · **ATT&CK del catálogo:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Qué comprueba Juntei:** lee los comandos `ImagePath` de los servicios en el conjunto de controles activo. Los controladores de tipo 1, 2 u 8 sin ImagePath conservan un destino implícito `System32\drivers\<service>.sys`.

**Evidencias examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath`, `Type` y metadatos complementarios del servicio.

**Cómo investigar:** revise la cuenta de servicio y la configuración de inicio con independencia de la etiqueta de alcance del equipo que aparece en la salida. Los registros deshabilitados o de inicio a petición siguen formando parte del inventario. Para las marcas de tiempo, se da preferencia a las evidencias basadas en el nombre del servicio frente a las rutas de ejecutables compartidos ambiguas.

**Más detalles:** [documentación de Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### DLL de inicio del servicio Windows Update {#wu_service_startup_dll}

**ID de comprobación:** `wu_service_startup_dll` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee el `Parameters\ServiceDll` del servicio Windows Update en el conjunto de controles activo.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**Cómo investigar:** compare la biblioteca con la misma compilación de Windows y con el estado de mantenimiento instalado. Se solapa con el inventario genérico de ServiceDll y no representa un segundo compromiso distinto.

## Tareas programadas { #scheduled-tasks }

El Programador de tareas almacena la configuración tanto en archivos XML como en el hive SOFTWARE. Lea ambas fuentes conjuntamente. Una tarea puede estar registrada sin haberse ejecutado, y una recopilación incompleta puede parecerse a una tarea oculta o fantasma.

### Tarea programada fantasma {#ghost_task}

**ID de comprobación:** `ghost_task` · **ATT&CK del catálogo:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Qué comprueba Juntei:** compara los registros `Tree` y `Tasks` de TaskCache en SOFTWARE y, cuando hay archivos de tareas en la recopilación, busca sus archivos XML de tarea correspondientes.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` y `TaskCache\Tasks`.
- Los archivos `Windows\System32\Tasks\<task path>` capturados, para la comparación con el sistema de archivos.

**Cómo investigar:** la falta de contrapartidas puede deberse a tareas eliminadas u obsoletas, a registros integrados o a una recopilación incompleta. Si no se ha recopilado ningún XML, la comparación con el sistema de archivos no está disponible. Inspeccione la cobertura y la discrepancia exacta antes de calificar una tarea de oculta o maliciosa.

### Tarea programada oculta {#hidden_scheduled_task}

**ID de comprobación:** `hidden_scheduled_task` · **ATT&CK del catálogo:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Qué comprueba Juntei:** comprueba si en los registros Tree de TaskCache falta el valor `SD` o hay un descriptor de seguridad binario vacío, condiciones asociadas a la ocultación de una tarea frente a la enumeración ordinaria.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: presencia y datos de `SD`, con los metadatos de la contrapartida en TaskCache.

**Cómo investigar:** un SD ausente o vacío es una anomalía de configuración concreta, no una prueba de que la tarea se ejecutara ni de que sea maliciosa. El ajuste `Hidden` del XML y los problemas arbitrarios de ACL son condiciones distintas y no constituyen el predicado que se utiliza aquí.

**Más detalles:** [documentación de Microsoft](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/).

### Tarea programada (archivos XML) {#scheduled_task_files}

**ID de comprobación:** `scheduled_task_files` · **ATT&CK del catálogo:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Qué comprueba Juntei:** analiza el XML de las tareas capturadas en busca de comandos y argumentos Exec, así como de identificadores de clase y datos de controladores COM. Selecciona la entidad de seguridad mediante `Actions Context`/`Principal id`, conserva por separado la identidad de usuario y la de grupo, e informa de SYSTEM, USER o UNKNOWN. Las acciones COM sin resolver siguen siendo visibles.

**Evidencias examinadas:**

- Archivos XML `Windows\System32\Tasks\<task path>`, incluidos Exec, ComHandler y los campos de entidad de seguridad disponibles.
- El `UsrClass.dat` capturado del usuario correspondiente y `Software\Classes` de NTUSER, además de `Classes\CLSID` de SOFTWARE del equipo, en las vistas nativa y de 32 bits.
- `ProfileList` de SOFTWARE capturado para las entidades de seguridad con SID. Para las entidades de seguridad con nombre se prefiere una carpeta `user.DOMAIN` coincidente y única, después los sufijos del host local capturados y, por último, la carpeta `user` simple. Las coincidencias se etiquetan explícitamente como heurísticas; las ambigüedades reales quedan sin resolver y se indican en el hallazgo y en la cobertura. No se requieren claves volátiles del Registro.
- `CodeBase` administrado (incluidas las subclaves de versión), las asociaciones de AppID y el `ImagePath` del servicio en SYSTEM para un `LocalService` de AppID.

Los candidatos de servidor COM incluyen su origen en el Registro en `Description`. Las asociaciones de AppID
que solo contienen el nombre del ejecutable quedan sin resolver en lugar de asignárseles un directorio supuesto. Las URL locales
`file:` se decodifican; las URL remotas se conservan sin acceder a la red. Las invalidaciones de clases de usuario
se limitan a ese perfil, y un CLSID de usuario incompleto no recurre al registro del equipo
que oculta. Las tareas solo de grupo conservan los candidatos del equipo con la limitación relativa
a usuarios individuales en su descripción, sin añadir avisos rutinarios al resumen de errores de cobertura.

**Cómo investigar:** inspeccione los desencadenadores, el estado de habilitación, la entidad de seguridad y el historial de ejecuciones en el XML de origen y en los registros de eventos. Una tarea creada puede no ejecutarse nunca. Un controlador COM sin resolver sigue siendo un candidato, y no debe suponerse que unos datos de entidad de seguridad ausentes o solo de grupo signifiquen SYSTEM.

## WMI, BITS y otros desencadenadores de eventos { #wmi-bits-and-other-event-triggers }

Windows Management Instrumentation (WMI) da soporte a la administración del sistema. Background Intelligent Transfer Service (BITS) da soporte a las transferencias en segundo plano. Ambos pueden conservar comandos desencadenados por eventos. Algunos mecanismos esperan un evento de administración, una notificación de transferencia, el inicio de una aplicación o una acción del usuario. Un desencadenador almacenado es una evidencia de configuración útil, pero su presencia no establece que el desencadenador se activara.

### Puerta trasera mediante las funciones de accesibilidad {#accessibility_tools}

**ID de comprobación:** `accessibility_tools` · **ATT&CK del catálogo:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Qué comprueba Juntei:** compara siete binarios de accesibilidad recopilados de System32 mediante el OriginalFilename del PE y los metadatos de certificado o catálogo, tras exigir metadatos de catálogo de Microsoft utilizables de un binario de referencia recopilado.

**Evidencias examinadas:**

- `Windows\System32\sethc.exe`, `osk.exe`, `narrator.exe`, `magnify.exe`, `utilman.exe`, `atbroker.exe` y `DisplaySwitch.exe`.
- Metadatos de referencia de `Windows\System32\winlogon.exe`, `cmd.exe` o `notepad.exe` recopilados y los catálogos recopilados aplicables.

**Cómo investigar:** los metadatos pueden sugerir un sustituto renombrado, pero Juntei no valida las firmas ni la confianza de los certificados. Una recopilación escasa sin los catálogos necesarios produce una cobertura de comparación incompleta, en lugar de tratar como puerta trasera cualquier archivo que parezca no estar firmado.

### Funciones de accesibilidad {#assistive_technology}

**ID de comprobación:** `assistive_technology` · **ATT&CK del catálogo:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Qué comprueba Juntei:** lee los comandos `StartExe` más `StartParams` de accesibilidad registrados en las vistas nativa y de 32 bits de SOFTWARE, así como las entradas `Configuration` de accesibilidad habilitadas en los hives de usuario y DEFAULT. Se omiten los marcadores de posición StartExe numéricos o que no son cadenas.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` y `StartParams`; también la vista `Wow6432Node` del equipo.
- `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` en NTUSER.DAT y el ajuste equivalente en el hive DEFAULT.

**Cómo investigar:** el software de accesibilidad es legítimo. Distinga el nombre de una función habilitada de su registro de ejecutable real y, después, inspeccione el destino y las evidencias de ejecución en el inicio de sesión o en el escritorio. Esto es independiente de la sustitución del propio ejecutable de accesibilidad.

**Más detalles:** [Referencia de la técnica (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/).

### Comando de notificación de trabajos BITS {#bits_notify_command}

**ID de comprobación:** `bits_notify_command` · **ATT&CK del catálogo:** [T1197](https://attack.mitre.org/techniques/T1197/)

**Qué comprueba Juntei:** recupera los registros de trabajos BITS en forma de cadenas con longitud de `qmgr.db`, `qmgr0.dat` o `qmgr1.dat` capturados, y conserva los programas de notificación no vacíos, los parámetros, los ID de trabajo y los SID de los propietarios.

**Evidencias examinadas:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`, `qmgr0.dat` y `qmgr1.dat`.

**Cómo investigar:** son registros recuperados mediante carving: no se valida el estado asignado o actual del trabajo y pueden aparecer trabajos obsoletos o eliminados. La hora de escritura en la MFT de la base de datos es una evidencia débil del contenedor, no la hora de ejecución de la notificación. Los SID de los propietarios distinguen entre SYSTEM, el servicio local, el servicio de red y los usuarios.

### AutoRun del procesador de comandos {#cmd_autorun}

**ID de comprobación:** `cmd_autorun` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** lee los comandos `AutoRun` de Command Processor del equipo y de los usuarios, un ajuste que puede afectar al inicio del símbolo del sistema.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**Cómo investigar:** las herramientas de desarrollo y la personalización del shell pueden explicar un valor. `cmd.exe /d` omite AutoRun, por lo que la configuración por sí sola no demuestra que se ejecutara cada vez que apareció cmd.exe. Inspeccione el contenido del comando y los argumentos del proceso desencadenante.

**Más detalles:** [documentación de Microsoft](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd).

### Persistencia mediante ErrorHandler.cmd {#error_handler_cmd}

**ID de comprobación:** `error_handler_cmd` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** informa de la presencia del script `Windows\System32\ErrorHandler.cmd` recopilado como candidato de persistencia mediante el controlador de errores.

**Evidencias examinadas:**

- `Windows\System32\ErrorHandler.cmd`.

**Cómo investigar:** la presencia por sí sola no demuestra que la ruta de recuperación o de errores de Windows aplicable lo invocara. Lea el script, inspeccione su procedencia y correlaciónelo con evidencias de instalación, errores y procesos.

### Flujos de Power Automate Desktop {#power_automate}

**ID de comprobación:** `power_automate` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** busca directorios de flujos y scripts capturados en los almacenes `flows` y `scripts` de Power Automate Desktop de cada usuario, e informa una sola vez de cada flujo descubierto.

**Evidencias examinadas:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` y `scripts\<flow>\...`.

**Cómo investigar:** identifica automatizaciones almacenadas, no todos los desencadenadores programados ni un flujo ejecutado. Inspeccione el contenido del flujo y la configuración de inicio externa antes de tratarlo como persistencia automática o maliciosa.

### Perfil de PowerShell {#powershell_profiles}

**ID de comprobación:** `powershell_profiles` · **ATT&CK del catálogo:** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**Qué comprueba Juntei:** busca archivos del tipo profile.ps1 en los directorios de instalación y de perfiles reconocidos de Windows PowerShell y PowerShell, incluidos Documents del usuario, las carpetas Documents de OneDrive descubiertas y las redirecciones de Documents recopiladas.

**Evidencias examinadas:**

- Archivos terminados en `profile.ps1` directamente en `Windows\System32\WindowsPowerShell\v1.0` y `Windows\SysWOW64\WindowsPowerShell\v1.0`.
- `Documents\WindowsPowerShell` y `Documents\PowerShell` por usuario, incluidas las redirecciones de Documents capturadas y las carpetas Documents de OneDrive descubiertas.
- Archivos terminados en `profile.ps1` directamente en los directorios de versión de `Program Files\PowerShell` o `Program Files (x86)\PowerShell`.

**Cómo investigar:** los perfiles pueden contener personalizaciones inofensivas. Lea el script y siga las cargas útiles invocadas. Al iniciar PowerShell con `-NoProfile` se omiten; la presencia del archivo por sí sola no demuestra la ejecución del perfil ni un contenido malicioso.

### Secuestro del protector de pantalla {#screensaver}

**ID de comprobación:** `screensaver` · **ATT&CK del catálogo:** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**Qué comprueba Juntei:** lee el valor `SCRNSAVE.EXE` de Desktop de cada usuario, lo que identifica la imagen del protector de pantalla configurado.

**Evidencias examinadas:**

- `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE` en NTUSER.DAT.

**Cómo investigar:** compare con los protectores de pantalla aprobados e inspeccione el destino. Otros ajustes del escritorio, el estado de la sesión y el tiempo de espera determinan si puede activarse; una ruta guardada no demuestra que se iniciara un protector de pantalla.

### Comando de TelemetryController {#telemetry_controller}

**ID de comprobación:** `telemetry_controller` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** lee `Binary`, `Command` y `MaintenanceCommand` de TelemetryController y de sus registros secundarios inmediatos.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` y sus elementos secundarios inmediatos: `Binary`, `Command`, `MaintenanceCommand`.

**Cómo investigar:** compare el ejecutable seleccionado y los argumentos con la misma compilación de Windows y con el software de mantenimiento. Un comando del Registro no demuestra que se activara un desencadenador de mantenimiento.

### Perfiles personalizados de Windows Terminal {#windows_terminal}

**ID de comprobación:** `windows_terminal` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** analiza la configuración JSON/JSONC de Windows Terminal empaquetada, de versión preliminar y no empaquetada, e informa de los ajustes `commandline` no vacíos de los perfiles y de los valores predeterminados.

**Evidencias examinadas:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` y el equivalente de WindowsTerminalPreview.
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: matrices de perfiles heredadas y campos commandline de `profiles.list` y `profiles.defaults`.

**Cómo investigar:** es natural que aquí aparezcan shells corrientes. Determine el perfil seleccionado o predeterminado e inspeccione los comandos o argumentos de inicio inusuales. El simple hecho de guardar un perfil no lo inicia.

### Suscripción a eventos WMI {#wmi_event_subscription}

**ID de comprobación:** `wmi_event_subscription` · **ATT&CK del catálogo:** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**Qué comprueba Juntei:** recupera mediante carving las instancias reconocidas de consumidores de eventos y las referencias de enlaces entre filtros y consumidores del repositorio OBJECTS.DATA de WMI capturado. Las clases reconocidas incluyen los consumidores de línea de comandos, de script activo, de archivo de registro, de registro de eventos y de SMTP.

**Evidencias examinadas:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` y `Repository\FS\OBJECTS.DATA`.

**Cómo investigar:** una coincidencia en el repositorio sin procesar puede ser obsoleta, estar desvinculada o ser histórica; la salida conserva esa matización. El carving no reconstruye un grafo completo de suscripciones activas ni demuestra que el desencadenador se activara. La hora de escritura del archivo del repositorio no es una hora de creación o de ejecución de la suscripción.

## Explorador, COM y controladores de archivos o URL { #explorer-com-and-file-or-url-handlers }

El Modelo de objetos componentes (COM) permite a las aplicaciones activar componentes de software registrados. El shell de Windows utiliza controladores registrados para abrir archivos, URL, carpetas y objetos especiales. Los identificadores COM (CLSID) identifican componentes; los ProgID dan nombres legibles a algunos componentes. Siga cada registro hasta el comando o la DLL que selecciona.

### Secuestro de App Paths {#app_paths}

**ID de comprobación:** `app_paths` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** lee los valores predeterminados bajo los registros `App Paths` de todo el equipo, que pueden redirigir la búsqueda de aplicaciones en los inicios realizados a través del shell.

**Evidencias examinadas:**

- Valor predeterminado de `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>`.

**Cómo investigar:** un registro en App Paths es habitual en las aplicaciones instaladas y no supone una reescritura general de todos los inicios de procesos. Compare el nombre y la imagen registrados con el software esperado y, después, examine las evidencias pertinentes del shell y de los procesos.

### Secuestro de objetos COM {#com_treat_as}

**ID de comprobación:** `com_treat_as` · **ATT&CK del catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Qué comprueba Juntei:** busca redirecciones `TreatAs` de COM y registros InprocServer32/LocalServer32 por usuario que ocultan un CLSID del equipo, en los almacenes de clases nativos y de 32 bits compatibles. Los destinos de TreatAs se resuelven utilizando las clases del mismo usuario y las clases del equipo.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` y la vista `Classes\Wow6432Node` del equipo.
- `Software\Classes\CLSID` por usuario en NTUSER.DAT y `CLSID` en UsrClass.dat, incluidas sus vistas `Wow6432Node` implementadas: valores predeterminados de `TreatAs`, `InprocServer32` y `LocalServer32`.

**Cómo investigar:** las invalidaciones por usuario pueden no requerir derechos de administrador, pero el software legítimo también puede registrarlas. Siga la clase seleccionada y la DLL o el ejecutable, evalúe la arquitectura y el contexto de activación, y corrobore de forma independiente la activación COM real.

### Secuestro de controladores del Liberador de espacio en disco {#disk_cleanup_handler}

**ID de comprobación:** `disk_cleanup_handler` · **ATT&CK del catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Qué comprueba Juntei:** lee los CLSID de los controladores VolumeCaches del Liberador de espacio en disco y sigue los registros InprocServer32 de COM del equipo que se pueden resolver hasta sus bibliotecas.

**Evidencias examinadas:**

- CLSID predeterminado de `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` y `Classes\CLSID\<CLSID>\InprocServer32`.

**Cómo investigar:** para ejecutarse, el controlador debe ser invocado por el flujo de limpieza correspondiente. Esta comprobación concreta no emite los controladores sin resolver, por lo que debe utilizar el analizador del Registro para inspeccionar sus registros sin procesar si la captura está incompleta.

### Invalidación de AppKey del Explorador {#explorer_app_key}

**ID de comprobación:** `explorer_app_key` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee los valores `ShellExecute` y `Association` bajo los elementos secundarios inmediatos de `AppKey` del Explorador, que abarcan los comandos y acciones seleccionados por las teclas de aplicación.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` y `Association`.

**Cómo investigar:** las utilidades de teclado y las herramientas OEM pueden definir estos valores. Un nombre de asociación puede requerir otro pivote en el Registro en lugar de ser una ruta de archivo. Compruebe la acción del usuario que lo desencadenaría.

### Browser Helper Objects {#explorer_bho}

**ID de comprobación:** `explorer_bho` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** lee los registros de CLSID de Browser Helper Object y resuelve las bibliotecas InprocServer32 del equipo cuando están disponibles; los identificadores de clase sin resolver siguen siendo visibles.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` y el valor predeterminado de `Classes\CLSID\<CLSID>\InprocServer32` del equipo.

**Cómo investigar:** está ligado al comportamiento compatible del navegador o del Explorador y es especialmente relevante en entornos heredados. Pueden aparecer extensiones aprobadas de navegadores o de seguridad. Un BHO registrado no demuestra que un navegador actual lo cargara.

### Secuestro de CLSID del Escritorio del Explorador {#explorer_clsid_hijack}

**ID de comprobación:** `explorer_clsid_hijack` · **ATT&CK del catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Qué comprueba Juntei:** lee los comandos shell-open configurados en los tres CLSID especiales del espacio de nombres del Explorador indicados, en los almacenes de clases del equipo y de los usuarios.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**Cómo investigar:** compare los controladores de comandos de Escritorio, Mi PC y Mis documentos con una referencia equivalente. Esta comprobación examina determinados CLSID y rutas, no todos los posibles secuestros del espacio de nombres del Explorador.

### Controladores del menú contextual del Explorador {#explorer_context_menu}

**ID de comprobación:** `explorer_context_menu` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** enumera los controladores del menú contextual de Directory, Directory Background, Folder, Drive, AllFilesystemObjects y `*`, y sigue los identificadores de clase hasta los registros de DLL recopilados cuando es posible.

**Evidencias examinadas:**

- Almacenes de clases `HKLM\SOFTWARE\Classes` del equipo y UsrClass.dat del usuario: valor predeterminado de `<subject>\shellex\ContextMenuHandlers\<handler>` y los registros de bibliotecas COM correspondientes.

**Cómo investigar:** las extensiones del menú contextual de herramientas de compresión, productos de seguridad y clientes en la nube son habituales. Inspeccione el controlador real y la interacción desencadenante. Los GUID sin resolver siguen siendo pistas útiles, no una prueba de que la DLL no existiera.

### Secuestro de asociaciones de archivos {#file_association_hijack}

**ID de comprobación:** `file_association_hijack` · **ATT&CK del catálogo:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Qué comprueba Juntei:** examina los registros `shell\open\command` de la extensión directa y del ProgID resuelto para `.txt`, `.pdf`, `.doc`, `.docx`, `.html`, `.htm`, `.js`, `.vbs`, `.hta`, `.exe`, `.bat`, `.cmd` y `.ps1`.

**Evidencias examinadas:**

- Almacenes de clases `HKLM\SOFTWARE\Classes` del equipo y UsrClass.dat del usuario: valores predeterminados de las `<extension>` seleccionadas y valores predeterminados de `<extension-or-ProgID>\shell\open\command`.

**Cómo investigar:** compare con la aplicación esperada e inspeccione los argumentos del comando. Se trata de un conjunto definido de extensiones, no de todos los tipos de archivo ni de la resolución efectiva completa de UserChoice. Un comando guardado no demuestra que se abriera un documento.

### Secuestro de controladores de protocolo {#protocol_handler_hijack}

**ID de comprobación:** `protocol_handler_hijack` · **ATT&CK del catálogo:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Qué comprueba Juntei:** lee los comandos de apertura de URL de http, https, mailto, ms-msdt y ms-officecmd, además de los registros `URL Protocol` descubiertos y las invalidaciones de usuario aplicables.

**Evidencias examinadas:**

- Almacenes de clases `HKLM\SOFTWARE\Classes` del equipo y UsrClass.dat del usuario: marcadores `URL Protocol` y valores predeterminados de `<scheme>\shell\open\command`.

**Cómo investigar:** los navegadores y los instaladores de aplicaciones registran esquemas de forma legítima. Siga el comando y los argumentos, evalúe la finalidad esperada del esquema y correlacione con evidencias de apertura de URL y de procesos.

### Controlador de extensión COM de la Papelera de reciclaje {#recycle_bin_com_extension}

**ID de comprobación:** `recycle_bin_com_extension` · **ATT&CK del catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Qué comprueba Juntei:** lee las invalidaciones de comandos open/empty/explore del CLSID de la Papelera de reciclaje indicadas y los registros inmediatos de controladores del menú contextual y de arrastrar y colocar.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**Cómo investigar:** compare con las extensiones del shell aprobadas. Algunos valores notificados son identificadores COM y no una biblioteca resuelta directamente; investigue la clase referenciada y la acción del usuario antes de afirmar que hubo ejecución de código.

### Secuestro del controlador del protocolo de búsqueda {#search_protocol_handler}

**ID de comprobación:** `search_protocol_handler` · **ATT&CK del catálogo:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Qué comprueba Juntei:** lee el controlador predeterminado `search-ms\shell\open\command` del equipo y de los usuarios.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**Cómo investigar:** un cambio en el controlador puede alterar lo que ocurre al abrir una URL search-ms, pero su presencia no demuestra la entrega de un exploit. Compare con los valores predeterminados de Windows e investigue la actividad de URL y de procesos que lo utilizaría.

### SharedTaskScheduler {#shared_task_scheduler}

**ID de comprobación:** `shared_task_scheduler` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** enumera los valores `SharedTaskScheduler` no vacíos del Explorador como componentes del shell registrados.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | Todos los valores no vacíos |

**Cómo investigar:** los valores pueden hacer referencia a clases COM en lugar de a una ruta de ejecutable. Siga sus registros de servidor y compruebe el comportamiento del sistema operativo, la procedencia de la extensión y las evidencias pertinentes de carga en el shell.

### ShellExecuteHooks {#shell_execute_hooks}

**ID de comprobación:** `shell_execute_hooks` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** enumera los valores `ShellExecuteHooks` no vacíos del Explorador, que abarcan los registros heredados de enlaces (hooks) de ejecución del shell.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | Todos los valores no vacíos |

**Cómo investigar:** resuelva cualquier CLSID referenciado y verifique si la compilación de Windows y las directivas permiten el enlace. Un registro heredado puede no utilizarse; el valor en sí no es una ejecución del shell observada.

### Persistencia COM mediante bibliotecas y carpetas CLSID del menú Inicio {#start_menu_com}

**ID de comprobación:** `start_menu_com` · **ATT&CK del catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Qué comprueba Juntei:** busca archivos XML de biblioteca `.library-ms` cuya URL `simpleLocation` hace referencia
a un CLSID de COM, así como directorios con el nombre `<name>.{CLSID}`, bajo
`Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu`. La interacción del Explorador con estos
objetos puede activar código COM. La comprobación incluye los directorios capturados vacíos; no sigue
los vínculos simbólicos. Esta comprobación no notifica los archivos de biblioteca corrientes situados fuera del menú Inicio ni los GUID
que aparecen en comentarios XML.

**Evidencias examinadas:**

- Archivos `.library-ms` y entradas de directorio con nombre de CLSID del menú Inicio de cada usuario.
- Los registros de clases capturados del usuario correspondiente y SOFTWARE del equipo, por separado según la arquitectura.
- La configuración administrada de CodeBase, AppID y servicios, mediante el mismo resolvedor que se usa para las tareas programadas.

**Cómo investigar:** `Path` y `SourceFile` identifican el artefacto desencadenante de la activación. `Value`
contiene el comando del servidor candidato o, si la resolución no está disponible, el CLSID. `Description`
vincula el desencadenador con el CLSID y con el origen del Registro capturado. `ResolvedPath`/`Sha256` describen únicamente
una carga útil resuelta y recopilada. Una asociación de AppID que solo contiene el nombre del ejecutable queda sin resolver.
Un desencadenador puede ser legítimo; su existencia no demuestra un inicio de sesión, una activación, una ejecución ni un ataque.
Inspeccione la URL de la biblioteca, el registro de la clase, la carga útil, el contexto del usuario y registros de ejecución independientes.

**Precauciones de recopilación:** las recopilaciones que solo contienen archivos pueden omitir los directorios vacíos. Se examinan las rutas estándar del menú Inicio
de los usuarios; esta comprobación no abarca los menús Inicio redirigidos ni otras ubicaciones del espacio de nombres del Explorador.
Los hives ausentes o los registros sin resolver producen una cobertura parcial, pero se conserva
el candidato. La arquitectura de la tarea, la elevación y las directivas pueden cambiar el comportamiento efectivo de la búsqueda COM.

**Más detalles:** [Propuesta original y reconocimiento al investigador](https://github.com/last-byte/PersistenceSniper/pull/32),
[vistas combinadas de COM por usuario y por equipo](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root),
[secuestro de COM en MITRE](https://attack.mitre.org/techniques/T1546/015/).

### Secuestro de COM mediante TypeLib {#typelib_hijack}

**ID de comprobación:** `typelib_hijack` · **ATT&CK del catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Qué comprueba Juntei:** lee los registros win32/win64 de GUID, versión y configuración regional de TypeLib en el UsrClass.dat de cada usuario, y conserva las rutas y los valores de monikers de script. Los índices numéricos de recursos finales solo se eliminan para resolver el archivo de destino.

**Evidencias examinadas:**

- Valores predeterminados de `TypeLib\<GUID>\<version>\<locale>\win32` y `win64` en cada UsrClass.dat capturado.

**Cómo investigar:** una biblioteca de tipos puede apuntar a una DLL legítima, a un archivo alternativo o a un moniker de script. La comprobación inventaría estos registros; evalúe la aplicación afectada y corrobore la carga de la biblioteca de tipos en lugar de suponer que toda ruta no predeterminada es un ataque.

## Aplicaciones y complementos de Office { #office-applications-and-add-ins }

Office admite complementos, plantillas y extensiones específicas de cada aplicación de forma legítima. Un registro sospechoso puede depender de una versión concreta de Office, de la aplicación, del ajuste de carga o de la interacción del usuario. La configuración por sí sola no establece la ejecución de macros.

### Registro de complementos de Office {#office_addins}

**ID de comprobación:** `office_addins` · **ATT&CK del catálogo:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Qué comprueba Juntei:** enumera los complementos de Word, Excel, PowerPoint, Outlook y Access en las raíces de Office sin versión y de las versiones 14.0/15.0/16.0, incluidos el almacén Click-to-Run del equipo implementado y las raíces de usuario. Conserva los valores explícitos Manifest/FileName/Path o resuelve las entradas aptas respaldadas por COM, con el contexto de LoadBehavior.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` y el equivalente Click-to-Run del equipo implementado.
- `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` en NTUSER.DAT; las clases COM del equipo y de los usuarios correspondientes de SOFTWARE, NTUSER.DAT y UsrClass.dat.

**Cómo investigar:** los complementos son software empresarial habitual. Revise LoadBehavior, la aplicación y la versión, los manifiestos, las cargas útiles reales y el proveedor esperado. La comprobación no garantiza que se hayan recopilado o cargado todos los formatos de complementos instalados ni todas las vistas de registro.

### Secuestro de complementos de IA de Office {#office_ai_hijack}

**ID de comprobación:** `office_ai_hijack` · **ATT&CK del catálogo:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Qué comprueba Juntei:** enumera los valores de la ubicación de registro Common AI de Click-to-Run de Office 16.0.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: todos los valores.

**Cómo investigar:** compare los valores y las rutas inusuales con la compilación de Office instalada. La presencia de un valor no muestra qué función de Office lo utilizó ni demuestra que seleccionara código ejecutable.

### Invalidación de DLL de Office {#office_dll_override}

**ID de comprobación:** `office_dll_override` · **ATT&CK del catálogo:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Qué comprueba Juntei:** lee `WwlibtDll` y `PPCoreTDLL` bajo las raíces de Word y PowerPoint para la configuración sin versión y de las versiones 14.0/15.0/16.0, incluidas las raíces Click-to-Run y de usuario implementadas.

**Evidencias examinadas:**

- La configuración `Microsoft\Office\[<version>\]Word` y `PowerPoint` del equipo y de los usuarios, además del árbol de Office Click-to-Run del equipo implementado: `WwlibtDll` y `PPCoreTDLL`.

**Cómo investigar:** determine qué aplicación o compilación reconoce la invalidación, inspeccione la biblioteca y su procedencia, y correlacione con un inicio de Office o una carga de imagen. Una invalidación configurada no es por sí sola una prueba de ejecución.

### Plantillas predeterminadas de Office {#office_templates}

**ID de comprobación:** `office_templates` · **ATT&CK del catálogo:** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**Qué comprueba Juntei:** inventaría los archivos Normal.dotm/NormalEmail.dotm recopilados y los archivos situados directamente en las carpetas Startup de Word y XLSTART de Excel reconocidas, incluidas las carpetas de inicio de la instalación de Office.

**Evidencias examinadas:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` y `NormalEmail.dotm`.
- Archivos situados directamente en `Microsoft\Word\STARTUP` y `Microsoft\Excel\XLSTART` del usuario, bajo AppData de Roaming.
- Archivos situados directamente en las carpetas de Office reconocidas `<version>\STARTUP` / `XLSTART` y `root\<version>\STARTUP` / `XLSTART` bajo Program Files y Program Files (x86).

**Cómo investigar:** una plantilla puede ser legítima o no contener contenido ejecutable. Juntei no extrae ni ejecuta macros en esta comprobación. Inspeccione la estructura interna del documento, la directiva de macros y la actividad de la aplicación; una ruta de instalación del equipo no implica un token de proceso de SYSTEM.

### DLL de Office Test {#office_test_dll}

**ID de comprobación:** `office_test_dll` · **ATT&CK del catálogo:** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**Qué comprueba Juntei:** enumera los valores de la clave Office Test Special Perf en las vistas indicadas del equipo, de los usuarios y del equipo de 32 bits.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | Todos los valores no vacíos |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | Todos los valores no vacíos |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | Todos los valores no vacíos |

**Cómo investigar:** trate las invalidaciones inesperadas como una pista que requiere validar la versión de Office y la carga útil. La comprobación informa de la configuración, no prueba si una instalación concreta de Office carga el valor.

### Ataque mediante la página principal de Outlook {#outlook_home_page}

**ID de comprobación:** `outlook_home_page` · **ATT&CK del catálogo:** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**Qué comprueba Juntei:** lee `URL` bajo los elementos secundarios de WebView de Outlook del usuario para las versiones de Office 14.0, 15.0 y 16.0.

**Evidencias examinadas:**

- `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL` en NTUSER.DAT.

**Cómo investigar:** la página principal de una carpeta puede ser legítima, y el comportamiento varía según la versión de Outlook y las actualizaciones de seguridad. Inspeccione la URL y su contenido y la directiva efectiva y, después, correlacione con evidencias de Outlook, web y procesos.

### Secuestro de DLL de monitores de VBA {#vba_monitors}

**ID de comprobación:** `vba_monitors` · **ATT&CK del catálogo:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Qué comprueba Juntei:** lee los valores InprocServer32 de los dos CLSID relacionados con los monitores de VBA implementados en los almacenes de clases del equipo y por usuario.

**Evidencias examinadas:**

- `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` y `{0002E157-0000-0000-C000-000000000046}` en SOFTWARE del equipo, además de los registros equivalentes en UsrClass.dat del usuario.

**Cómo investigar:** compare las bibliotecas seleccionadas con el entorno de Office y VBA instalado. Se trata de una comprobación dirigida a determinados CLSID, no de un inventario de todas las extensiones de VBA ni de una prueba de que se ejecutara una macro.

## Depuradores, controladores de bloqueos y shims de compatibilidad { #debuggers-crash-handlers-and-compatibility-shims }

Un depurador o un controlador de errores puede ejecutarse cuando una aplicación se inicia, se bloquea, deja de responder o finaliza. Las bases de datos de compatibilidad de aplicaciones también pueden afectar al inicio. Investigue los ajustes necesarios y el programa desencadenante junto con el comando notificado.

### Secuestro del depurador AeDebug {#ae_debug}

**ID de comprobación:** `ae_debug` · **ATT&CK del catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Qué comprueba Juntei:** lee los comandos `Debugger` de AeDebug nativos y de 32 bits que utiliza la configuración de depuración Just-In-Time de bloqueos.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**Cómo investigar:** el software de desarrollo y depuración puede configurarlo de forma legítima. Examine el comando completo del depurador y los requisitos previos para que un bloqueo lo desencadene y, después, correlacione con los eventos de errores de aplicaciones y de procesos.

### Depurador de procesos protegidos AeDebug {#ae_debug_protected}

**ID de comprobación:** `ae_debug_protected` · **ATT&CK del catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Qué comprueba Juntei:** lee los comandos `Debugger` de AeDebugProtected nativos y de 32 bits, correspondientes a la superficie de depuración de procesos protegidos.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**Cómo investigar:** determine si la compilación del sistema operativo y la directiva de protección pueden utilizar el comando configurado. Un depurador guardado no demuestra que un proceso protegido se bloqueara ni que el comando se ejecutara.

### Depurador de Image File Execution Options {#ifeo_debugger}

**ID de comprobación:** `ifeo_debugger` · **ATT&CK del catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Qué comprueba Juntei:** busca valores `Debugger` en los registros de Image File Execution Options nativos y de 32 bits, incluidas las claves de filtro anidadas.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`, incluidas las claves de filtro anidadas y la vista `Wow6432Node` del equipo.

**Cómo investigar:** puede seleccionarse un depurador cuando se inicia la imagen nombrada. Inspeccione el alcance de la imagen o del filtro y los argumentos del depurador; las herramientas de desarrollo también pueden utilizar IFEO. Rastree el inicio del destino para establecer la ejecución.

### NTDLL delegada de IFEO {#ifeo_delegated_ntdll}

**ID de comprobación:** `ifeo_delegated_ntdll` · **ATT&CK del catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Qué comprueba Juntei:** busca invalidaciones `VerifierDlls` en los registros de Image File Execution Options nativos y de 32 bits, incluidas las claves anidadas.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`, incluidas las claves anidadas y la vista `Wow6432Node` del equipo.

**Cómo investigar:** el predicado de la comprobación es el valor VerifierDlls configurado; no demuestra que estén habilitados todos los requisitos previos del comprobador o de la delegación. Revise los indicadores aplicables, la versión de Windows y la carga útil antes de deducir una vía de inyección funcional.

### Monitor de salida silenciosa de procesos {#ifeo_silent_process_exit}

**ID de comprobación:** `ifeo_silent_process_exit` · **ATT&CK del catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Qué comprueba Juntei:** busca `MonitorProcess` en los registros SilentProcessExit nativos y de 32 bits, incluidas las claves anidadas.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`, incluidas las claves anidadas y la vista `Wow6432Node` del equipo.

**Cómo investigar:** la supervisión de salidas silenciosas también requiere la configuración de informes o de IFEO correspondiente. Inspeccione esos ajustes y los eventos de finalización de la imagen supervisada; las herramientas de diagnóstico legítimas pueden explicar el monitor.

### Shims de aplicaciones {#installed_sdb}

**ID de comprobación:** `installed_sdb` · **ATT&CK del catálogo:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Qué comprueba Juntei:** lee `DatabasePath` de los registros inmediatos InstalledSDB de las bases de datos de compatibilidad de aplicaciones instaladas.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**Cómo investigar:** las correcciones de compatibilidad pueden ser legítimas. Inspeccione el contenido del SDB, los ejecutables a los que se dirige y el historial de instalación. El registro de una base de datos no identifica sus shims ni demuestra que se ejecutara una aplicación coincidente.

### Secuestro del depurador de LSM {#lsm_debugger}

**ID de comprobación:** `lsm_debugger` · **ATT&CK del catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Qué comprueba Juntei:** lee `MonitorProcess` específicamente bajo SilentProcessExit para `lsm.exe` en las vistas del equipo nativa y de 32 bits.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**Cómo investigar:** se solapa con el inventario más amplio de SilentProcessExit. Confirme los indicadores necesarios y una finalización relevante de LSM antes de tratar la configuración como persistencia ejecutada; cuente la misma entrada una sola vez al evaluar los hallazgos distintos.

### DLL auxiliares de MiniDump {#minidump_auxiliary_dlls}

**ID de comprobación:** `minidump_auxiliary_dlls` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** enumera los valores con nombre no vacíos de MiniDumpAuxiliaryDlls, y conserva el nombre del valor como contexto del módulo desencadenante y los datos del valor como destino seleccionado.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: nombres de valor de los módulos desencadenantes y datos de destino.

**Cómo investigar:** inspeccione tanto el módulo desencadenante como la DLL auxiliar. Las herramientas de bloqueos y minivolcados pueden explicar una entrada, y un registro no muestra que una operación de volcado cargara la biblioteca.

### Shims de aplicaciones {#shim_custom}

**ID de comprobación:** `shim_custom` · **ATT&CK del catálogo:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Qué comprueba Juntei:** lee los valores con nombre `.sdb` bajo las claves inmediatas de aplicaciones Custom de AppCompatFlags, y conserva la asociación entre la aplicación y la base de datos.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: nombres de valor terminados en `.sdb`.

**Cómo investigar:** empareje estas asociaciones con InstalledSDB y con la base de datos de compatibilidad real. Una asociación de nombre de archivo por sí sola no revela el comportamiento de la base de datos ni demuestra el inicio de una aplicación coincidente.

### Secuestro del depurador de WER {#wer_debugger}

**ID de comprobación:** `wer_debugger` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** lee los valores `Debugger` de Informe de errores de Windows nativos y de 32 bits.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**Cómo investigar:** el software de diagnóstico legítimo puede configurar un depurador. Investigue el comando completo y los eventos de bloqueo pertinentes; el registro no demuestra que Informe de errores de Windows lo iniciara.

### Secuestro del depurador Hangs de WER {#wer_hangs}

**ID de comprobación:** `wer_hangs` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** lee los valores `Debugger` de Hangs de Informe de errores de Windows nativos y de 32 bits.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**Cómo investigar:** compruebe si un flujo de tratamiento de aplicaciones que no responden invocaría este ajuste e inspeccione el programa de destino. Correlacione con los registros de aplicaciones que no responden y de procesos en lugar de tratar una escritura en el Registro como la ejecución de un depurador.

### Secuestro de ReflectDebugger de WER {#wer_reflect_debugger}

**ID de comprobación:** `wer_reflect_debugger` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** lee los valores `ReflectDebugger` de Informe de errores de Windows nativos y de 32 bits.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**Cómo investigar:** compare con la configuración de diagnóstico aprobada y determine la aplicabilidad según la versión de Windows y el desencadenador. Inspeccione los argumentos del comando y busque corroboración en bloqueos y procesos antes de atribuir una ejecución.

### Módulos de excepciones en tiempo de ejecución de WER {#wer_runtime_exception}

**ID de comprobación:** `wer_runtime_exception` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** enumera los nombres de valor bajo RuntimeExceptionHelperModules nativo y de 32 bits; aquí la ruta de la biblioteca figura en el nombre del valor, no necesariamente en sus datos.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` y la vista `Wow6432Node` del equipo: nombres de valor.

**Cómo investigar:** inspeccione el módulo nombrado y el contexto de la aplicación o del runtime. Un valor numérico pequeño en los datos no es la ruta de la carga útil, y el registro por sí solo no demuestra que se cargara el módulo auxiliar.

## Configuración del runtime y del generador de perfiles de .NET { #net-runtime-and-profiler-settings }

Los runtimes administrados pueden cargar generadores de perfiles, ensamblados de inicio o bibliotecas de depuración antes que el código de la aplicación. Los desarrolladores y los productos de supervisión también utilizan esta configuración. Compruebe el runtime, la arquitectura del proceso, los indicadores de habilitación y si un proceso recién iniciado podría heredar el ajuste.

### Inyección mediante AppDomainManager {#appdomain_manager}

**ID de comprobación:** `appdomain_manager` · **ATT&CK del catálogo:** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**Qué comprueba Juntei:** lee las variables de entorno `APPDOMAIN_MANAGER_ASM` y `APPDOMAIN_MANAGER_TYPE` del equipo y de los usuarios, que abarcan la selección del ensamblado y del tipo para un administrador de AppDomain de .NET personalizado.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**Cómo investigar:** las aplicaciones pueden utilizar administradores personalizados de forma intencionada. Resuelva conjuntamente el ensamblado y el tipo, y compruebe la compatibilidad del runtime y la herencia entre procesos; ni una sola variable ni la marca de tiempo de su clave demuestran que una aplicación administrada lo cargara.

### Secuestro del generador de perfiles CLR de .NET {#cor_profiler}

**ID de comprobación:** `cor_profiler` · **ATT&CK del catálogo:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Qué comprueba Juntei:** lee la configuración de entorno de generación de perfiles COR del equipo y de los usuarios, así como las entradas con nombre coincidentes en las cadenas múltiples Environment de cada servicio. Conserva los indicadores de habilitación, los identificadores del generador de perfiles y las rutas específicas de cada arquitectura.

**Evidencias examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` y el `Environment` de cada NTUSER.DAT de usuario capturado.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: ajustes COR con nombre en la cadena múltiple.

**Cómo investigar:** los generadores de perfiles dan soporte a supervisiones legítimas. Un indicador de habilitación, un CLSID y una ruta de DLL son elementos de configuración distintos; confirme la habilitación efectiva y el proceso .NET pertinente en lugar de tratar cada fila como un generador de perfiles ejecutado.

### Secuestro del generador de perfiles CLR de .NET Core {#coreclr_profiler}

**ID de comprobación:** `coreclr_profiler` · **ATT&CK del catálogo:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Qué comprueba Juntei:** lee la configuración de entorno de generación de perfiles CORECLR correspondiente del equipo y de los usuarios, así como las entradas Environment de servicios coincidentes, incluidas las rutas específicas de cada arquitectura.

**Evidencias examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` y el `Environment` de cada NTUSER.DAT de usuario capturado.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: ajustes CORECLR con nombre en la cadena múltiple.

**Cómo investigar:** valide el runtime de .NET, la arquitectura, el indicador de habilitación y la biblioteca. Los productos de observabilidad aprobados pueden configurar estos ajustes, y un proceso existente puede no heredar un entorno modificado recientemente.

### Secuestro de DbgManagedDebugger de .NET {#dotnet_dbg_managed_debugger}

**ID de comprobación:** `dotnet_dbg_managed_debugger` · **ATT&CK del catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Qué comprueba Juntei:** lee los ajustes `DbgManagedDebugger` de `.NETFramework` del equipo y de los usuarios indicados, incluida la ruta explícita de 32 bits del equipo.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**Cómo investigar:** determine si el runtime de .NET o la directiva de depuración aplicables utilizan este comando. Las herramientas de desarrollo pueden explicarlo; correlacione con los bloqueos de código administrado y la creación de procesos para establecer la ejecución.

### Clave del Registro COR_PROFILER de .NET Framework {#dotnet_framework_profiler}

**ID de comprobación:** `dotnet_framework_profiler` · **ATT&CK del catálogo:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Qué comprueba Juntei:** lee `COR_PROFILER`, `COR_PROFILER_PATH` y `COR_ENABLE_PROFILING` de las vistas del Registro de `.NETFramework` indicadas. Complementa las comprobaciones del generador de perfiles basadas en el entorno.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**Cómo investigar:** evalúe los valores conjuntamente y determine si el runtime respeta esa ubicación del Registro. Una herramienta de generación de perfiles configurada puede ser legítima; no interprete un indicador de habilitación aislado como la carga de una DLL identificada.

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**ID de comprobación:** `dotnet_startup_hooks` · **ATT&CK del catálogo:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Qué comprueba Juntei:** lee los valores de entorno `DOTNET_STARTUP_HOOKS` del equipo y de los usuarios, que pueden seleccionar ensamblados de enlace de inicio para los runtimes de .NET compatibles.

**Evidencias examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` y el `Environment\DOTNET_STARTUP_HOOKS` de cada NTUSER.DAT de usuario.

**Cómo investigar:** inspeccione el valor completo, ya que puede contener varios ensamblados, y verifique la compatibilidad del runtime. Esta comprobación no enumera los entornos de enlaces de inicio específicos de los servicios ni demuestra que algún proceso heredara y utilizara el valor.

### DLL de depuración administrada conocidas {#known_managed_debugging_dlls}

**ID de comprobación:** `known_managed_debugging_dlls` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee los registros `KnownManagedDebuggingDlls` de `.NETFramework` como posibles invalidaciones de bibliotecas del depurador administrado.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**Cómo investigar:** compare con las herramientas de desarrollo y de runtime instaladas y determine el depurador desencadenante. La entrada del Registro no demuestra una sesión de depuración ni la carga de la biblioteca.

## Invalidaciones de la búsqueda de DLL y ejecutables { #dll-and-executable-lookup-overrides }

Estas comprobaciones examinan ubicaciones explícitas de bibliotecas o ajustes que influyen en la forma en que un componente encuentra el código. Una invalidación de ruta o una directiva de búsqueda debilitada es una pista para la investigación; para que haya ejecución siguen siendo necesarios una carga útil y un componente desencadenante compatible.

### AppCert DLLs {#appcert_dlls}

**ID de comprobación:** `appcert_dlls` · **ATT&CK del catálogo:** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**Qué comprueba Juntei:** enumera los registros AppCertDlls no vacíos de Session Manager, una superficie de extensión de bibliotecas asociada a las API de creación de procesos.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | Todos los valores no vacíos |

**Cómo investigar:** inspeccione cada biblioteca y la procedencia de su instalación. Determine por separado la aplicabilidad y la carga en tiempo de ejecución; la creación normal de procesos en una línea de tiempo no demuestra por sí misma que se cargara la DLL configurada.

### AppInit DLLs {#appinit_dlls}

**ID de comprobación:** `appinit_dlls` · **ATT&CK del catálogo:** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**Qué comprueba Juntei:** lee `AppInit_DLLs` nativo y de 32 bits, divide su lista de bibliotecas y añade a cada candidato el contexto disponible de `LoadAppInit_DLLs` y `RequireSignedAppInit_DLLs`.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs`, `LoadAppInit_DLLs` y `RequireSignedAppInit_DLLs`; también la vista `Wow6432Node` del equipo.

**Cómo investigar:** un indicador LoadAppInit_DLLs con valor cero se marca explícitamente como inactivo. La versión de Windows, el arranque seguro y los requisitos de firma también afectan a la carga. Revise la configuración efectiva antes de afirmar que se ejecutó una DLL de AppInit.

**Más detalles:** [documentación de Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls).

### Invalidación de AutodialDLL {#autodial_dll}

**ID de comprobación:** `autodial_dll` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee la invalidación `AutodialDLL` de Parameters de WinSock2.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**Cómo investigar:** compruebe la función de red y la versión de Windows de origen que utilizan este ajuste. Revise la DLL real y la actividad de red y de procesos; una invalidación por sí sola no establece una carga activa o maliciosa.

### DLL auxiliar de CHM {#chm_helper_dll}

**ID de comprobación:** `chm_helper_dll` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee `Location` bajo HtmlHelp Author, una superficie de invalidación de bibliotecas o ubicaciones relacionada con la ayuda.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**Cómo investigar:** determine si el componente de ayuda pertinente utiliza el valor en este sistema e inspeccione el archivo seleccionado. Una invalidación en el Registro no demuestra que se abriera un documento CHM.

### Invalidación de DLL de ContentIndex NL6 {#content_index_dll}

**ID de comprobación:** `content_index_dll` · **ATT&CK del catálogo:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Qué comprueba Juntei:** lee `DLLOverridePath` de los registros inmediatos de ContentIndex Language del conjunto de controles activo.

**Evidencias examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`.

**Cómo investigar:** los componentes de idioma y de búsqueda pueden variar según la compilación de Windows. Inspeccione la biblioteca y el componente instalado correspondiente; siga el proceso de indexación real para corroborar la carga.

### DLL ExpoOffload de criptografía {#crypto_expo_offload}

**ID de comprobación:** `crypto_expo_offload` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee el valor `ExpoOffload` de Cryptography Offload como candidato de invalidación de una biblioteca criptográfica.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**Cómo investigar:** confirme la aplicabilidad según la función y la compilación e investigue el origen de la biblioteca seleccionada. No trate la presencia de un valor poco común como prueba de robo de credenciales o de ejecución de código.

### DLL del rasterizador por software de Direct3D {#direct3d_dll}

**ID de comprobación:** `direct3d_dll` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee la invalidación de biblioteca `SoftwareRasterizer` de Direct3D Drivers.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**Cómo investigar:** el software de gráficos y de compatibilidad puede influir en la configuración del rasterizador. Inspeccione la DLL y la actividad de una aplicación Direct3D compatible; guardar la ruta no demuestra que se cargara.

### Debilitamiento del orden de búsqueda de DLL {#dll_search_mode}

**ID de comprobación:** `dll_search_mode` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** informa de `SafeDllSearchMode` o `CWDIllegalInDllSearch` solo cuando el valor activo de Session Manager se interpreta como cero. Se trata de condiciones de la directiva de búsqueda, no de cargas útiles seleccionadas.

**Evidencias examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` y `CWDIllegalInDllSearch`.

**Cómo investigar:** una directiva de búsqueda debilitada puede facilitar un ataque independiente basado en el orden de búsqueda. No identifica una DLL plantada ni demuestra una ejecución. Esta comprobación no emite los valores distintos de cero ni los ausentes; evalúe por separado los valores predeterminados efectivos y las directivas.

**Más detalles:** [documentación de Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security).

### Exclusión de KnownDLLs {#exclude_from_known_dlls}

**ID de comprobación:** `exclude_from_known_dlls` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee las entradas `ExcludeFromKnownDlls` no vacías de Session Manager como cambios en el comportamiento de búsqueda de las DLL protegidas o conocidas.

**Evidencias examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`.

**Cómo investigar:** una exclusión identifica una condición de búsqueda, no la propia biblioteca de sustitución. Busque la aplicación afectada y las posibles ubicaciones de búsqueda de DLL y, después, corrobore una carga de imagen no estándar.

**Más detalles:** [documentación de Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order).

### Invalidación de la DLL hhctrl.ocx {#hhctrl_ocx_dll}

**ID de comprobación:** `hhctrl_ocx_dll` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee las invalidaciones InprocServer32 del equipo nativas y de 32 bits y de los almacenes de clases de usuario compatibles para el CLSID del control de Ayuda HTML implementado.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**Cómo investigar:** determine qué aplicación y qué arquitectura activan el control. Inspeccione la DLL registrada y la actividad relacionada con la ayuda; un registro COM no demuestra que la Ayuda HTML mostrara contenido.

### Known DLLs {#known_dlls}

**ID de comprobación:** `known_dlls` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** enumera los valores con nombre de la clave KnownDLLs de Session Manager activa.

**Evidencias examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: valores con nombre.

**Cómo investigar:** muchas entradas son bibliotecas de Windows esperadas, y algunos valores son configuración complementaria y no una ruta de carga útil. Compare la clave completa con una compilación equivalente de confianza; esta comprobación no analiza todos los árboles KnownDLLs alternativos específicos de cada arquitectura.

### DLL de extensiones de LSA {#lsa_extensions}

**ID de comprobación:** `lsa_extensions` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee `Extensions` de la configuración LsaSrv de LsaExtensionConfig.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**Cómo investigar:** el software de seguridad puede ampliar los componentes de autenticación. Inspeccione las bibliotecas reales y su compatibilidad con la versión de Windows de origen y, después, busque evidencias de carga de imágenes en LSASS u otras evidencias independientes de uso.

### Invalidación de la ruta de la DLL MAPI32 {#mapi32_dll_path}

**ID de comprobación:** `mapi32_dll_path` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee `DLLPathEx` y `DLLPath` bajo los registros inmediatos de clientes de correo en las vistas del equipo nativa y de 32 bits.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**Cómo investigar:** los clientes de correo registran proveedores MAPI legítimos. Determine qué cliente o proveedor estaba seleccionado e inspeccione la biblioteca real; un proveedor instalado no demuestra que una aplicación utilizara MAPI durante el incidente.

### DLL XA de MSDTC {#msdtc_xa_dll}

**ID de comprobación:** `msdtc_xa_dll` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee `OracleXaLib` y `OracleOciLib` de la configuración MTxOCI de MSDTC.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**Cómo investigar:** la integración con bases de datos puede requerir legítimamente estas bibliotecas. Verifique la configuración del coordinador de transacciones y el software de base de datos instalado, y correlacione con el uso de servicios o módulos en lugar de suponer que una ruta configurada se ejecutó.

### Invalidación de DLL de NLDP {#nldp_dll}

**ID de comprobación:** `nldp_dll` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee el valor de configuración `DllOverridePath` de NlsData.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**Cómo investigar:** revise la biblioteca de destino, la compilación de Windows y el componente de lenguaje natural que podría utilizar la invalidación. El valor no muestra que se produjera una indexación ni un procesamiento lingüístico.

### DLL TestDVCPlugin de RDP {#rdp_test_dvc_plugin}

**ID de comprobación:** `rdp_test_dvc_plugin` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee el ajuste de biblioteca `TestDVCPlugin` de Terminal Server Client del equipo.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**Cómo investigar:** investigue un complemento de prueba desconocido y el comportamiento específico de la versión. Para establecer la ejecución se necesitan una sesión de cliente compatible y la carga de la biblioteca; el valor del Registro por sí solo no muestra ninguna de las dos.

### Invalidación de DLL del indizador de búsqueda {#search_indexer_dll}

**ID de comprobación:** `search_indexer_dll` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee la invalidación `DllPath` de Windows Search en SOFTWARE.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**Cómo investigar:** determine el componente y la versión de Search correspondientes e inspeccione el archivo de destino. Un servicio Windows Search inactivo o un ajuste no utilizado no equivalen a una carga observada de una biblioteca maliciosa.

### DLL AutoProxy de Winsock {#winsock_auto_proxy}

**ID de comprobación:** `winsock_auto_proxy` · **ATT&CK del catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Qué comprueba Juntei:** lee `LibraryPath` bajo los registros inmediatos Catalog_Entries y Catalog_Entries64 de NameSpace_Catalog5.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**Cómo investigar:** los proveedores de red pueden ser legítimos. Revise ambas arquitecturas, la procedencia del proveedor y las evidencias pertinentes de aplicaciones de red o de carga de imágenes. Esta búsqueda dirigida de proveedores de espacios de nombres no abarca todas las extensiones posibles de Winsock.

## Cuentas y visibilidad en el inicio de sesión { #accounts-and-sign-in-visibility }

Los registros de cuentas locales y los ajustes de visibilidad en el inicio de sesión pueden revelar persistencia basada en identidades. Estas comprobaciones inspeccionan los hives SAM y SOFTWARE capturados; no constituyen un inventario de las cuentas de Active Directory, de la pertenencia a grupos del dominio ni de los inicios de sesión anteriores.

### Pertenencia al grupo Administradores {#admin_group_membership}

**ID de comprobación:** `admin_group_membership` · **ATT&CK del catálogo:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Qué comprueba Juntei:** decodifica la lista de miembros del alias local Administradores en SAM. Informa de los SID del equipo local que designan una cuenta oculta o que no corresponden a ninguna cuenta o grupo en el SAM capturado.

**Evidencias examinadas:**

- `SAM\Domains\Builtin\Aliases\00000220\C` y los registros de SID del equipo, usuarios y grupos bajo `SAM\Domains\Account` en el hive SAM.
- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList` para la correlación con nombres ocultos.

**Cómo investigar:** se trata de una comprobación dirigida de anomalías, no de una lista de todos los administradores. Los SID externos o del dominio no se evalúan como usuarios locales ausentes. Las cuentas eliminadas, el momento de la captura o los hives incompletos requieren investigación antes de concluir que la pertenencia es maliciosa.

### Cuenta local oculta {#hidden_account}

**ID de comprobación:** `hidden_account` · **ATT&CK del catálogo:** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**Qué comprueba Juntei:** informa de los valores numéricos cero en UserList de SpecialAccounts de Winlogon, que identifican los nombres ocultos en la pantalla de inicio de sesión ordinaria.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: valores con nombres de cuenta.

**Cómo investigar:** las cuentas de servicio o de mantenimiento pueden ocultarse de forma intencionada, y el valor no establece que la cuenta nombrada exista ni que sea administradora. Combínelo con los registros de cuentas de SAM y con la actividad de inicio de sesión.

### Secuestro de RID {#rid_hijacking}

**ID de comprobación:** `rid_hijacking` · **ATT&CK del catálogo:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Qué comprueba Juntei:** compara el RID hexadecimal de una subclave de cuenta de SAM con el RID almacenado en el desplazamiento 0x30 de su valor F e informa de las discrepancias.

**Evidencias examinadas:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` en el hive SAM.

**Cómo investigar:** una discrepancia es una anomalía grave del registro de la cuenta, pero también puede requerir un análisis de corrupción o de la captura. Inspeccione los bytes de origen, el estado de los registros de transacciones y los registros de cuentas e inicios de sesión; no demuestra qué actor modificó el registro.

### RID Suborner (cuenta de administrador oculta) {#rid_suborner}

**ID de comprobación:** `rid_suborner` · **ATT&CK del catálogo:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Qué comprueba Juntei:** informa de las cuentas de SAM cuyo RID de subclave no es 500 pero cuyo RID en el valor F es 500, un patrón más específico de sustitución de la identidad del administrador.

**Evidencias examinadas:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` en el hive SAM.

**Cómo investigar:** se solapa con la comprobación general de discrepancias de RID. A pesar del título del catálogo, no requiere de forma independiente un ajuste de ocultación en el inicio de sesión; confirme por separado la visibilidad de la cuenta, los privilegios y el uso real de la cuenta.

## Componentes de autenticación y protección de credenciales { #authentication-components-and-credential-protection }

La Autoridad de seguridad local (LSA) se encarga de las funciones de seguridad a través de LSASS y de los componentes relacionados. Los paquetes de autenticación, los filtros de contraseñas y los proveedores de red pueden ser software de seguridad legítimo o una vía de acceso persistente a las credenciales. Dos comprobaciones inventarían ajustes de protección en lugar de elementos ejecutables de inicio automático. Interprételas teniendo en cuenta la función del host, la versión de Windows y la directiva efectiva.

### DLL de proveedor de AMSI {#amsi_providers}

**ID de comprobación:** `amsi_providers` · **ATT&CK del catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Qué comprueba Juntei:** enumera los CLSID de los proveedores de AMSI del equipo y resuelve sus bibliotecas InprocServer32 nativas y de 32 bits cuando están disponibles. Los registros sin resolver conservan el identificador de clase.

**Evidencias examinadas:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` y los registros InprocServer32 de `Classes\CLSID` / `Classes\Wow6432Node\CLSID` que se puedan resolver.

**Cómo investigar:** los antivirus y los productos de seguridad registran proveedores de AMSI de forma legítima. Revise la procedencia del proveedor y de la biblioteca y su carga real; un registro no demuestra que se eludiera AMSI ni que el proveedor fuera malicioso.

### Paquetes de autenticación {#authentication_packages}

**ID de comprobación:** `authentication_packages` · **ATT&CK del catálogo:** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**Qué comprueba Juntei:** lee cada entrada no vacía de `Authentication Packages` de LSA en el conjunto de controles activo.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**Cómo investigar:** compare con la compilación de Windows de origen y con el software de autenticación instalado. Un paquete no predeterminado puede ser legítimo. Resuelva la DLL y corrobore la carga y la actividad relacionada con credenciales antes de concluir que hubo interceptación.

### Comportamiento de inicio de sesión del administrador de DSRM {#dsrm_backdoor}

**ID de comprobación:** `dsrm_backdoor` · **ATT&CK del catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Qué comprueba Juntei:** informa del valor numérico `DsrmAdminLogonBehavior=2` en las ubicaciones implementadas ControlSet001, ControlSet002 o CurrentControlSet. Este ajuste puede permitir el uso de la cuenta de DSRM fuera de su contexto de recuperación habitual.

**Evidencias examinadas:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior`, además de ControlSet002 y CurrentControlSet cuando existen.

**Cómo investigar:** es relevante principalmente en los controladores de dominio. Determine qué conjunto de controles está activo mediante Select de SYSTEM; una coincidencia en un conjunto de controles histórico no es necesariamente efectiva. El valor no demuestra que un atacante conociera la contraseña de DSRM ni que iniciara sesión.

### Configuración de Credential Guard {#lsa_cfg_flags}

**ID de comprobación:** `lsa_cfg_flags` · **ATT&CK del catálogo:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Qué comprueba Juntei:** lee el valor `LsaCfgFlags` de LSA configurado, incluidos tanto los valores de habilitación ordinarios como el cero. Inventaría la configuración de Credential Guard, no un elemento ejecutable de inicio automático.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**Cómo investigar:** interprete el valor teniendo en cuenta la versión de Windows, los requisitos previos de firmware y seguridad y la directiva efectiva. Un cero almacenado o un valor ausente no establecen por sí solos un cambio realizado por un atacante, el estado de protección efectivo ni un robo de credenciales.

### Filtro de contraseñas de LSA {#lsa_password_filter}

**ID de comprobación:** `lsa_password_filter` · **ATT&CK del catálogo:** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**Qué comprueba Juntei:** lee `Notification Packages` de LSA. Los nombres de biblioteca sin ruta se resuelven en el directorio System32 de Windows capturado, añadiendo `.dll` cuando corresponde.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**Cómo investigar:** los productos de directivas de contraseñas suelen instalar filtros. Inspeccione la biblioteca y las evidencias de carga, y distinga los filtros configurados de una prueba de que se produjo un cambio de contraseña o una interceptación de credenciales.

### Estado de la protección PPL de LSASS {#lsa_run_as_ppl}

**ID de comprobación:** `lsa_run_as_ppl` · **ATT&CK del catálogo:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Qué comprueba Juntei:** lee el valor `RunAsPPL` de LSA configurado, incluidos los valores distintos de cero. Es un inventario de ajustes de protección, no una prueba de que LSASS se ejecutara sin protección.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**Cómo investigar:** la protección efectiva de LSA depende de la versión de Windows, de las directivas, del bloqueo por firmware y del estado de reinicio. La ausencia de una fila por un valor inexistente no es un resultado de protección medido. Utilice evidencias independientes del sistema y del tiempo de ejecución para evaluar la protección efectiva.

### DLL de proveedor de red {#network_provider_dll}

**ID de comprobación:** `network_provider_dll` · **ATT&CK del catálogo:** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**Qué comprueba Juntei:** lee `NetworkProvider\ProviderPath` bajo los servicios del conjunto de controles activo de SYSTEM.

**Evidencias examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`.

**Cómo investigar:** los proveedores de inicio de sesión de red pueden ser legítimos. Compruebe el orden y la aplicabilidad de los proveedores, el software de red y de seguridad instalado y la carga real de la biblioteca; un valor ProviderPath por sí solo no demuestra una interceptación de credenciales.

### Paquetes de seguridad {#security_packages}

**ID de comprobación:** `security_packages` · **ATT&CK del catálogo:** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**Qué comprueba Juntei:** lee cada entrada no vacía de `Security Packages` de LSA, que abarca los proveedores de compatibilidad para seguridad configurados.

**Evidencias examinadas:**

| Clave del Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**Cómo investigar:** compare con la referencia esperada de Windows y de los productos de seguridad e inspeccione cada biblioteca. La configuración no demuestra que un proveedor se cargara, capturara credenciales o se añadiera durante el incidente.

## Del candidato a la conclusión { #work-from-a-candidate-to-a-conclusion }

1. Abra el origen original identificado por `SourceFile`. Confirme el valor completo, la acción XML,
   el script o el registro de base de datos recuperado, así como el usuario y el host exactos a los que pertenece.
2. Determine si el mecanismo podría funcionar: compruebe los indicadores de habilitación, el runtime o la versión de Windows,
   el estado del servicio o de la tarea, el desencadenador necesario, el usuario, la arquitectura y el conjunto de controles activo.
3. Inspeccione el programa, la DLL, el script o el ensamblado seleccionados. Calcule el hash de los bytes recopilados y revise los argumentos
   y el contenido. Un iniciador de Windows reconocible o una cadena de editor no zanjan la cuestión de la legitimidad.
4. Establezca lo que ocurrió realmente utilizando evidencias independientes. Algunos ejemplos son la creación de procesos,
   las cargas de imágenes de Sysmon, los registros de ejecución del Programador de tareas, la actividad de servicios, Prefetch y los archivos relacionados.
   Un evento de cambio de configuración puede demostrar que el ajuste cambió sin demostrar que su carga útil se ejecutara.
5. Construya una línea de tiempo con las marcas de tiempo del origen original y la procedencia de la correlación registrada.
   Mantenga como hechos separados las escrituras en claves, los cambios de instalación, las escrituras en archivos o contenedores y las ejecuciones.
6. Registre las limitaciones además de los hallazgos: hives ausentes, archivos de destino ausentes, entidades de seguridad de tareas desconocidas,
   metadatos de firmante no verificados, registros históricos recuperados mediante carving o un desencadenador que no se puede corroborar.

Para la sintaxis del comando, los perfiles, todos los campos de salida y los estados de cobertura, vuelva a la
[guía del comando Persistence Hunter](../commands/persistence-hunter.md). Las
[guías de artefactos](../artifacts/index.md) explican las fuentes subyacentes, y los
[agradecimientos](acknowledgements.md) identifican las herramientas de persistencia originales.
