# Secretos LSA { #lsa-secrets }

## Qué son { #what-they-are }

La Autoridad de seguridad local (LSA) guarda secretos en el hive `SECURITY`. Entre ellos se encuentran la
contraseña de inicio de sesión automático, las contraseñas de las cuentas con las que se ejecutan los servicios, la contraseña de dominio
de la cuenta de equipo, las claves del sistema de DPAPI y la clave con la que se cifran los inicios de sesión de dominio en caché.
También guarda los inicios de sesión de dominio en caché, para que los usuarios del dominio puedan iniciar sesión sin un controlador de dominio.

`parse-registry lsa-secrets` enumera qué secretos contiene un sistema y cuándo se establecieron, así como cuántos
inicios de sesión en caché conserva y utiliza. **Solo lee nombres y horas: no se descifra nada y
no se leen ni escriben los bytes de ningún secreto.** Juntei lee evidencias; no extrae credenciales.

<span class="evidence-strength">Mejor conclusión:</span> el sistema contenía un secreto con este nombre, establecido
en esta hora. El secreto de un servicio (`_SC_<service>`) significa que LSA almacena la contraseña de la cuenta
con la que se ejecuta ese servicio.

## Ubicación { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| Nombre | Tipo | Qué es |
|---|---|---|
| `DefaultPassword` | `autologon` | La contraseña con la que Winlogon inicia sesión automáticamente. |
| `_SC_<service>` | `service_account` | La contraseña de la cuenta con la que se ejecuta `<service>`. |
| `$MACHINE.ACC` | `machine_account` | La contraseña de la cuenta de equipo en su dominio. |
| `DPAPI_SYSTEM` | `dpapi_system` | Las claves DPAPI del sistema. |
| `NL$KM` | `cached_logon_key` | La clave con la que se cifran los inicios de sesión en caché. |
| `L$_SQSA_<SID>` | `security_questions` | Las preguntas de seguridad de una cuenta local (Windows 10 y posteriores). |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | Credenciales de acceso telefónico y de VPN. |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | Otros datos privados, según el prefijo que les asigna `LsaStorePrivateData`. |

Recopile `SECURITY` con sus registros de transacciones, así como el hive `SYSTEM` del sistema, que proporciona el nombre del
host.

## Precauciones para el analista { #analyst-cautions }

- **Nombres y horas, no valores.** Si un valor contiene datos se deduce de su tamaño. Su contenido
  no se lee.
- **`DefaultPassword` no implica inicio de sesión automático.** El secreto existe en la mayoría de los sistemas, incluidos
  los que nunca inician sesión automáticamente. La clave
  `Microsoft\Windows NT\CurrentVersion\Winlogon` de `SOFTWARE` (`AutoAdminLogon`, `DefaultUserName`) indica
  si el inicio de sesión automático está configurado. Un `CurrentSetUTC` reciente es un cambio que conviene explicar.
- **Secretos de servicios:** `_SC_<service>` aparece cuando un servicio está configurado para ejecutarse con una cuenta con
  contraseña. Compárelo con `parse-registry services` (`ObjectName`) y busque servicios
  instalados recientemente o desconocidos. El secreto puede permanecer después de que el servicio desaparezca.
- **Horas:** `CurrentSetUTC` y `OldSetUTC` indican cuándo estableció Windows los valores actual y
  anterior. De forma predeterminada, la contraseña de `$MACHINE.ACC` cambia cada 30 días, por lo que sus horas varían.
  `KeyLastWrittenUTC` y las horas de las claves `CurrVal`/`OldVal` indican aproximadamente cuándo cambió el secreto
  por última vez.
- **Inicios de sesión en caché:** una ranura está en uso cuando su registro nombra a un usuario. El encabezado del registro no está
  cifrado; solo se lee la longitud del nombre de usuario. `CachedLogonsKept` es el recuento de `NL$Control`,
  que sigue a `CachedLogonsCount` (10 de forma predeterminada). Las ranuras en uso significan que usuarios del dominio iniciaron sesión
  de forma interactiva, y sus credenciales pueden descifrarse sin conexión a partir del hive. Los nombres de los usuarios
  están cifrados y no se decodifican. Solo los registros binarios validados con un encabezado completo y una longitud de nombre de usuario coherente cuentan como en uso. Si `Problems` informa de ranuras dañadas, el recuento de ranuras en uso abarca solo las ranuras validadas; no interprete las ranuras dañadas como vacías.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

Para cada hive, el resumen enumera cada secreto con su tipo y cuándo se estableció. A continuación, nombra
los servicios cuyas contraseñas de cuenta almacena LSA y cuenta las ranuras de inicio de sesión en caché que están en uso.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath` | Host (según el hive `SYSTEM` del sistema) y el hive `SECURITY`. |
| `Secret`, `Kind` | El nombre del secreto y qué es (consulte la tabla anterior); `Cache` y `cached_logons` para el registro de inicios de sesión en caché. |
| `ServiceName`, `UserSID` | El servicio al que corresponde un secreto `_SC_` y el SID de usuario que incluye un nombre (`L$_SQSA_`, `RasDialParams`). |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` y `OupdTime`: cuándo se establecieron los valores actual y anterior. |
| `HasCurrentValue`, `HasOldValue` | Si `CurrVal` y `OldVal` contienen datos. |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | Últimas escrituras de las claves `CurrVal`, `OldVal` y del secreto. |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | Registro de inicios de sesión en caché: ranuras, ranuras en uso y el recuento de `NL$Control`. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Horas con formato incorrecto o fuera de rango (solo se muestran los metadatos de hora), encabezados de caché dañados, tipos de valor de caché erróneos y claves que no se pueden leer. |

## Estrategia de investigación { #investigation-strategy }

1. Busque secretos `_SC_` de servicios desconocidos o instalados recientemente y emparéjelos con
   `parse-registry services` y con el evento 7045 (se instaló un servicio).
2. Compare las horas de `DefaultPassword` con la configuración de inicio de sesión automático de `Winlogon`.
3. En los miembros de un dominio, anote las ranuras de inicio de sesión en caché en uso y cuándo se escribió `Cache` por última vez.
4. Compare las horas de los secretos con la cronología: un secreto establecido durante el periodo de una intrusión puede
   indicar una nueva cuenta de servicio, un inicio de sesión automático o un cambio en la cuenta de equipo.

## Más información { #further-details }

- [Microsoft: objeto de datos privados](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: información de inicio de sesión de dominio en caché](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: credenciales en caché](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
