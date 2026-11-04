# Información del sistema { #system-information }

## Qué es { #what-it-is }

Los cuatro hives del equipo de un sistema indican lo que un analista necesita saber antes de leer
su línea de tiempo:

- sus nombres (NetBIOS, nombre de host DNS, dominio DNS) y el dominio o grupo de trabajo al que
  pertenece;
- su versión de Windows, su compilación y su revisión de actualización, y cuándo se instaló o se
  actualizó Windows por última vez;
- su zona horaria, que indica las horas locales que mantenía el sistema;
- cuándo se apagó correctamente por última vez;
- su arquitectura de procesador y sus páginas de códigos;
- el último usuario que inició sesión y los perfiles de todos los usuarios que han iniciado sesión;
- sus SID: el del equipo (las cuentas locales son ese SID más un RID) y el de su dominio;
- las versiones de Windows desde las que se actualizó.

`parse-registry system-info` escribe un registro por sistema con todos estos datos.

<span class="evidence-strength">Mejor conclusión:</span> la configuración tal como estaba cuando se
recopilaron los hives (o cuando se escribieron por última vez). Las horas de las claves indican
cuándo cambiaron por última vez partes de ella.

## Ubicación { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

Recopile los cuatro hives con sus registros de transacciones. Cualquiera de ellos por separado
proporciona lo que contiene.

## Precauciones para el analista { #analyst-cautions }

- **Las horas del Registro están en UTC; la zona horaria sirve para todo lo demás.** Las horas de
  las claves, los FILETIME y la mayoría de las horas de los eventos están en UTC. Los registros de
  texto, algunos registros de aplicaciones y las fechas escritas en nombres o cadenas (como
  `Source OS (Updated on …)`) pueden estar en hora local. `Bias` es el número de minutos desde la
  hora estándar local hasta UTC (`UTC = local + Bias`). `UtcOffset` y `DaylightUtcOffset`
  proporcionan los dos desplazamientos, y `TimeZoneDescription` indica cuándo empieza y termina el
  horario de verano.
- **La zona horaria puede cambiar.** `TimeZoneLastWrittenUTC` es la última escritura de la clave.
  Windows también reescribe la clave por sí mismo cuando actualiza `ActiveTimeBias` (el
  desplazamiento vigente en ese momento), por lo que una hora reciente no prueba que alguien
  cambiara la zona. Compárela con los registros de eventos: el 4616 de Security (cambio de la hora
  del sistema) y los eventos `Microsoft-Windows-Kernel-General` del registro System.
- **`StandardStart` y `DaylightStart` son `TIME_FIELDS` del kernel, no `SYSTEMTIME`.** El día de la
  semana es su último campo, no el tercero (como en el valor `TZI` de la zona en `SOFTWARE`). En las
  zonas de EE. UU. ambas disposiciones se leen igual, porque el día de la semana es domingo (0).
  Juntei los decodifica como `TIME_FIELDS`: un sistema público de la Isla de Pascua (primer sábado
  a las 22:00) muestra la diferencia. Un mes 0 significa que no hay horario de verano.
- **Windows 11 dice "Windows 10".** `ProductName` mantiene `Windows 10 …` en Windows 11. Las
  compilaciones 22000 y posteriores son Windows 11, y el resumen lo indica. `CurrentVersion` es
  `6.3` en Windows 8.1, 10 y 11; `Version` utiliza `CurrentMajorVersionNumber` (10) cuando existe.
- **`InstallDate` cambia con las actualizaciones de características.** Una actualización en el
  sitio a una nueva versión (de 1809 a 21H2, por ejemplo) lo restablece. La versión que sustituyó
  cada actualización, con su propio `InstallDate`, está en `PreviousVersions` (`Setup\Source OS`).
  El `InstallDate` más antiguo de todos ellos es el más próximo a la instalación original. La fecha
  del nombre de una clave `Source OS` no indica su zona horaria.
- **`ShutdownTime` es el último apagado correcto.** Un bloqueo o un corte de alimentación no lo
  actualiza. Compárelo con los registros de eventos (6006, 6008, 1074) y con las horas de las
  últimas escrituras en los hives.
- **`LastLoggedOnUser` es el último usuario de la pantalla de inicio de sesión,** no todos los
  inicios de sesión. Escritorio remoto, `runas`, los servicios y las tareas programadas no siempre
  lo establecen. `LogonUiLastWrittenUTC` indica aproximadamente cuándo se estableció por última
  vez.
- **Perfiles:** `ProfileList` enumera todos los usuarios (y servicios) que han tenido un perfil en
  el sistema, incluidas las cuentas eliminadas hasta que se quita su perfil. Windows 10 y versiones
  posteriores escriben `LocalProfileLoadTime` y `LocalProfileUnloadTime`: aproximadamente el último
  inicio y cierre de sesión.
- **SID:** las cuentas locales son `MachineSid-<RID>` y las cuentas de dominio `DomainSid-<RID>`.
  `MachineSid` procede del `SAM` (o, en su defecto, del `PolAcDmS` de `SECURITY`). Ambos deberían
  coincidir; cuando no es así, `Problems` lo indica, como ocurriría con hives de sistemas
  distintos. `DomainSid` está vacío en un grupo de trabajo.
- **Un directorio, un sistema.** Con `-d`, los hives de un directorio pertenecen a un único
  sistema. Las copias de `RegBack` y un segundo hive del mismo tipo (`SYSTEM.old`) son registros
  independientes. Con `-f`, los hives indicados pertenecen a un único sistema, y los demás se
  buscan junto a ellos.
- **Conjuntos de control:** solo se lee el conjunto de control que Windows usó por última vez.
  `parse-registry read` muestra los demás.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

El resumen indica, para cada sistema: su versión de Windows, cuándo se instaló, su zona horaria
descrita en palabras, el desplazamiento vigente cuando se escribió la clave por última vez, su
último apagado, su dominio o grupo de trabajo, el último usuario que inició sesión, su SID, cuántos
perfiles tiene y las versiones desde las que se actualizó. `auto-parse` también muestra la zona
horaria de cada sistema cuando enumera los sistemas que ha encontrado.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | Host y los hives leídos (`SourcePath` es el hive `SYSTEM` o, en su defecto, el primero que se leyó). |
| `CompName`, `Hostname`, `Domain` | El nombre NetBIOS (`SYSTEM` o, en su defecto, el `PolAcDmN` de `SECURITY`), el nombre de host DNS y el dominio DNS (`Tcpip\Parameters`). |
| `PrimaryDomain`, `DNS-Domain` | El dominio al que está unido el equipo, o su grupo de trabajo (`PolPrDmN`), y el nombre DNS del dominio (`PolDnDDN`). |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: el nombre del producto tal como está escrito, `EditionID`, `DisplayVersion` (o, en su defecto, `ReleaseId`), la versión principal y secundaria (o, en su defecto, `CurrentVersion`), la compilación, la revisión de compilación de actualización, `CSDVersion` y `Client`, `Server` o `Server Core`. |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (con precisión de segundos) e `InstallTime` (con precisión de 100 ns, Windows 10 y posteriores): la instalación o la última actualización de características. |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | Tal como están escritos. |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (o, en su defecto, `StandardName`) y la zona descrita en palabras con sus desplazamientos y las fechas del horario de verano. |
| `UTC-Offset`, `DaylightUTC-Offset` | El desplazamiento de la hora estándar local respecto a UTC y el del horario de verano (vacío si no hay horario de verano). |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`, `ActiveTimeBias` (vigente cuando se escribió la clave), `StandardBias`, `DaylightBias`. |
| `StandardName`, `DaylightName` | Tal como están escritos, a menudo una referencia a un recurso (`@tzres.dll,-112`). |
| `StandardStart`, `DaylightStart` | Cuándo empiezan la hora estándar y el horario de verano, en palabras (`first Sunday of November, 02:00`). |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | El horario de verano dinámico está desactivado; el reloj del hardware mantiene la hora UTC. |
| `TimeZoneLastWrittenUTC` | La última escritura de la clave de zona horaria. |
| `LastShutdownUTC` | `ShutdownTime`: el último apagado correcto. |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | El conjunto de control leído, `PROCESSOR_ARCHITECTURE`, `ACP` y `OEMCP`. |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | El último usuario de la pantalla de inicio de sesión y la última escritura de su clave. |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | El SID del equipo, el SID de su dominio (`PolPrDmS`) y el SID de la cuenta de equipo en el dominio (`PolMachineAccountS`). |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | Aproximadamente cuándo se asignó nombre al equipo por última vez, cuándo se unió por última vez a un dominio o grupo de trabajo y cuándo se actualizó por última vez (la clave de `CurrentVersion`). |
| `PreviousVersions` | Cada clave `Source OS`: la versión, su fecha de instalación y la fecha del nombre de la clave (JSON: un objeto por cada una). |
| `Profiles` | Cada perfil: SID, carpeta, hora de la clave, última carga y última descarga (JSON: un objeto por cada uno). |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro, para los cuatro hives. |
| `Problems` | Valores de tipo o tamaño incorrecto (se conservan sus bytes) y un `SAM` y un `SECURITY` cuyos SID de equipo difieren. |

## Estrategia de investigación { #investigation-strategy }

1. Lea primero la zona horaria: anote `UtcOffset`, las fechas del horario de verano y si la zona se
   cambió recientemente. Úsela para convertir los registros en hora local.
2. Anote la versión, la compilación y la fecha de instalación. Revise `PreviousVersions` para ver
   las actualizaciones y compare la fecha de instalación con las horas más antiguas del sistema de
   archivos y de los registros de eventos.
3. Anote `LastShutdownUTC`: la actividad posterior sin una nueva hora de apagado sugiere un
   bloqueo, un corte de alimentación o una recopilación en vivo.
4. Utilice `MachineSid` y `DomainSid` para distinguir las cuentas locales de las de dominio en los
   SID de otros artefactos (BAM, `ProfileList`, registros de eventos), y las horas de carga y
   descarga de los perfiles para los últimos inicios de sesión.
5. Correlacione los nombres del equipo y del dominio con los registros de eventos, y con
   `CompNameLastWrittenUTC` y `PrimaryDomainLastWrittenUTC` para los cambios de nombre y las
   uniones a dominios.

## Más información { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: identificadores de seguridad](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: zonas horarias](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: información del producto Windows](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
