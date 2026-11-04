# Inicio automático en el Registro { #registry-autostarts }

## Qué son { #what-they-are }

Windows inicia programas y carga DLL desde muchas ubicaciones del Registro: al arrancar, al iniciar sesión, cuando un
programa se inicia o termina, cuando se abre `cmd.exe`, cuando se crea una clase COM, o cuando el administrador de trabajos de impresión,
LSA, netsh o el servicio de hora cargan sus complementos. Los atacantes utilizan estas ubicaciones para lograr persistencia,
escalada de privilegios e inyección de DLL, y el software legítimo también las utiliza. Juntei las lee
de `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` y `UsrClass.dat` en una sola tabla. Escribe un registro por
programa, DLL o paquete, y etiqueta cada uno con su técnica de MITRE ATT&CK.

<span class="evidence-strength">Mejor conclusión:</span> el hive configuraba este programa o esta DLL
para iniciarse o cargarse. No demuestra que el programa se ejecutara. Su ejecución requiere las condiciones que indica el
mecanismo (un inicio de sesión, el inicio del programa depurado, la creación de la clase) y otras
evidencias de ejecución.

## Ubicaciones { #locations }

| Hive | Qué se lee |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce` (+`Setup`), `RunOnceEx`, `RunServices(Once)` y `Policies\Explorer\Run`, con sus copias de `Wow6432Node` y del modo de instalación de Terminal Server; `Explorer\StartupApproved\Run`, `Run32` y `StartupFolder`; `Winlogon` (`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … y `Notify`); `Windows` `AppInit_DLLs` con `LoadAppInit_DLLs`; `Image File Execution Options\*` `Debugger` (con `UseFilter`) y `VerifierDlls`; `SilentProcessExit\*` `MonitorProcess`; `Active Setup\Installed Components\*` `StubPath`; `NetSh`; `Command Processor` `AutoRun` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, variables del generador de perfiles de .NET de `Environment`); `Control\Lsa` `Authentication`, `Security` y `Notification Packages` (+`OSConfig`); `SecurityProviders`; monitores, proveedores y procesadores de impresión de `Control\Print`; `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | las claves `Run` y `StartupApproved` del usuario; `Windows NT\CurrentVersion\Windows` `Load` y `Run`; `Winlogon` `Shell`; `Environment` (`UserInitMprLogonScript`, generador de perfiles de .NET); `Control Panel\Desktop` `SCRNSAVE.EXE`; `PackagedAppXDebug`; `Command Processor` `AutoRun`; `Software\Classes` (como `UsrClass.dat`) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` y `LocalServer32` (y los de `Wow6432Node`); `shell\open\command` de los tipos de programa (`exefile`, `batfile`, `scrfile`, …) y de lo que inician los programas con elevación automática (`ms-settings`, `mscfile`, `Folder`, …); `ActivatableClasses\…\DebugInformation` |

Recopile cada hive junto con sus registros de transacciones. Los hives de usuario están en `Users\<name>\NTUSER.DAT` y
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat`.

## Precauciones para el analista { #analyst-cautions }

- **También se informa de los valores predeterminados.** La mayoría de los registros de un sistema limpio son valores predeterminados de Windows:
  - `Userinit` es `userinit.exe`;
  - `Shell` es `explorer.exe`;
  - `KnownDLLs`, los paquetes de LSA, los monitores de impresión, los asistentes de netsh y los componentes de Active Setup tienen todos entradas de serie.

  Busque lo que difiere de una instalación limpia de la misma versión.
- **`StartupApproved` solo abarca la lista Inicio del Administrador de tareas.** Indica si una entrada `Run`
  o un archivo de la carpeta Inicio están deshabilitados allí, y cuándo. Se informa de una entrada sin valor `Run`
  correspondiente, con el detalle "only in StartupApproved". Puede mostrar un inicio automático que
  se eliminó.
- **`OverridesMachineRegistration` necesita el hive `SOFTWARE` del sistema.** La clase COM o el
  comando de un usuario se comparan con él, y se localiza de la misma forma que el hive `SYSTEM` del host. Una
  invalidación corresponde al patrón de secuestro de COM (T1546.015). OneDrive y otras instalaciones por usuario también
  registran clases propias que no invalidan nada.
- **`ImagePath` y `Args` solo se separan de la línea de comandos cuando es posible determinarlo.**
  - Un programa entre comillas se separa en la comilla de cierre.
  - Una primera palabra sin carpeta se toma como el programa (`cmd /c …`).
  - En los demás casos, la separación se hace en el inicio más corto que termina con la extensión de un programa.

  Una ruta sin comillas que contenga espacios puede ejecutar otro programa (`C:\Program.exe`) si existe.
- **`GlobalFlag` determina `Enabled` para `VerifierDlls` y `MonitorProcess`.** `VerifierDlls` se carga
  solo con `0x100`. `MonitorProcess` se ejecuta solo con `0x200` y el `0x1` de `ReportingMode` de
  `SilentProcessExit\<program>`. Un modo `2` (un volcado) o `4` (una notificación) no ejecuta nada, y
  `Details` indica los indicadores del modo.
- **`KeyLastWrittenUTC` es la última escritura de la clave**, no la del valor. Varios valores la comparten.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

El resumen cuenta los registros por mecanismo. También enumera las entradas `StartupApproved\Run` y `Run32`
que se han quedado sin su valor `Run` (los archivos de la carpeta Inicio no tienen ninguno que perder), y las clases COM
y los comandos de usuario que invalidan los del equipo.

Como todos los comandos `parse-registry`, rechaza un `-o` que sea uno de los archivos que lee, incluso
con `-C`. Esto abarca el hive, sus registros de transacciones y los hives SYSTEM y SOFTWARE del sistema,
por ruta, vínculo simbólico o vínculo físico. Cuando no puede determinarlo, no escribe.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Contexto del host, hive de origen y su tipo. |
| `User`, `UserSID` | En los hives de usuario: la carpeta del perfil y el SID. El SID procede del nombre de la clave raíz de `UsrClass.dat` o, si no, del `ProfileList` del sistema. |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | Técnica de MITRE ATT&CK, cuando hay una que encaje (`T1547.001`, `T1546.012`, …). |
| `KeyPath`, `ValueName` | La clave bajo la raíz del hive (nombres tal como se escribieron) y el valor (vacío: el valor predeterminado). |
| `Data` | Texto del valor, o un elemento de una lista. Vacío cuando no se puede leer o cuando solo `StartupApproved` nombra la entrada. |
| `ImagePath`, `Args` | El programa o la DLL, y los argumentos, cuando se pueden distinguir (consulte más arriba). |
| `Enabled`, `DisabledAtUTC` | Lo que indican `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag` (con `ReportingMode`), `UseFilter`, `IsInstalled`, el `Enabled` del proveedor de hora o `ScreenSaveActive`. Vacío cuando nada lo indica. |
| `Details` | Más información: qué hace un cambio de nombre pendiente, el nombre de la clase COM, `ReportingMode`, `FilterFullPath`, un `ShellComponent` de Active Setup, … |
| `OverridesMachineRegistration` | Para la clase COM o el comando de un usuario: si el equipo registra el mismo. Vacío sin el hive `SOFTWARE` del sistema. |
| `WOW64` | Bajo `Wow6432Node`: para programas de 32 bits. |
| `KeyLastWrittenUTC` | Última escritura de la clave. |
| `DataRaw` | Bytes del valor (en hexadecimal) cuando no son texto o no son UTF-16 válido. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Tipos de valor incorrectos, datos que no se pueden leer, entradas de `StartupApproved` con un tamaño incorrecto. |

## Estrategia de investigación { #investigation-strategy }

1. Empiece por el resumen:
   - las clases COM y los comandos que invalidan los del equipo;
   - las entradas de `StartupApproved` sin su valor `Run`;
   - los mecanismos que rara vez tienen entradas (`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`).
2. Compare los paquetes de LSA, `BootExecute`, `KnownDLLs`, los monitores de impresión y los asistentes de netsh con una instalación
   limpia de la misma versión de Windows.
3. Resuelva `ImagePath` en el sistema de archivos ($MFT, hashes y firmas), y compruebe las rutas del perfil
   de usuario, `ProgramData` y `Temp`.
4. Correlacione `KeyLastWrittenUTC` con los eventos del Registro de Security y Sysmon, la creación de procesos,
   Prefetch y Amcache.
5. Utilice `parse-registry read --recover-deleted` para encontrar inicios automáticos eliminados de estas claves.

## Más información { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: claves del Registro Run y RunOnce](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
