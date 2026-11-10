# Compatibilidad de aplicaciones { #application-compatibility }

## Qué conserva Windows { #what-windows-keeps }

Además de ShimCache (`parse-registry shimcache`) y Amcache (`parse-registry amcache`),
las funciones de compatibilidad de aplicaciones de Windows conservan lo siguiente en
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags`:

| Origen | Hive y clave | Qué muestra |
|---|---|---|
| `layers` | `SOFTWARE` (y `Wow6432Node`) y `NTUSER.DAT` `Layers` | Programas configurados para ejecutarse con capas de compatibilidad: `RUNASADMIN` (siempre como administrador), `WIN7RTM`, `HIGHDPIAWARE`, … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 y posteriores) | Programas que el Asistente para la compatibilidad de programas vio ejecutar al usuario |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | Lo mismo, en Windows 7 |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | Programas que cargaron un módulo supervisado y cuándo lo hicieron por última vez |
| `cit_system` | `SOFTWARE` `CIT\System` (de Windows 7 a las primeras versiones de Windows 10) | Rutas de programas y líneas de comandos en las bases de datos CIT comprimidas |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | Bases de datos de shims personalizadas instaladas con `sdbinst`: archivo, descripción, hora de instalación |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | Las bases de datos de shims personalizadas aplicadas a cada programa, con sus horas de instalación |

<span class="evidence-strength">Mejor conclusión:</span> las capas y las bases de datos de shims son
configuración que cambia la forma en que se ejecuta un programa; las entradas del Asistente para la compatibilidad de programas y de CIT
muestran que un programa se ejecutó (o cargó un módulo) como este usuario o en este sistema.

Las bases de datos de shims personalizadas son una técnica de persistencia e inyección (MITRE ATT&CK T1546.011): un shim
puede cargar una DLL en un programa o iniciar otro programa cada vez que se ejecuta.

## Ubicación { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

Recopile `SOFTWARE`, todos los `NTUSER.DAT` y sus registros de transacciones, así como los archivos `.sdb` que indica la ruta
de cada base de datos de shims (normalmente `C:\Windows\AppPatch\Custom\`).

## Precauciones para el analista { #analyst-cautions }

- **La expansión de CIT está limitada.** Los valores que declaran más de 64 MiB sin comprimir se conservan
  como bytes sin procesar con un diagnóstico. Se informa de la compresión con formato incorrecto y de las discrepancias de tamaño.


- **Las entradas del Asistente para la compatibilidad de programas no tienen hora de ejecución.** Los datos de `Store` (`SACP`, conservados en
  `DataRaw`) tienen una hora, pero en Windows 8.1, 10 y 11 es la misma para todos los programas de un
  hive, y en los demás sistemas tampoco es el momento en que se ejecutó el programa. Solo la última escritura de la clave `Store`
  fecha el cambio más reciente.
- **Los programas en medios extraíbles** se nombran `SIGN.MEDIA=<id> <path>`: `Path` es la ruta en el
  medio y `Details` contiene el identificador del medio.
- **Las bases de datos CIT** están comprimidas con LZNT1 y su estructura no está documentada públicamente: Juntei
  enumera las rutas de programas y las líneas de comandos encontradas en cada una, una vez por base de datos, sin recuentos ni
  horas. Una línea de comandos es un registro propio, no vinculado a su programa. Windows 10 y posteriores conservan
  poca información aquí.
- **Bases de datos de shims.** Revise todos los registros `custom_sdb` e `installed_sdb`: el software legítimo
  rara vez instala una. El propio archivo `.sdb` indica lo que hace el shim.
- **Las capas** establecidas por un usuario se aplican solo a ese usuario; las de `SOFTWARE`, a todos. `RUNASADMIN` hace que un
  programa solicite elevación cada vez.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

El resumen enumera las bases de datos de shims personalizadas, los programas con capas y los últimos usos
de los módulos CIT.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, archivo del hive y `SOFTWARE` o `NTUSER.DAT`. |
| `User`, `UserSID` | Nombre de la carpeta de perfil de un hive de usuario y su SID según el `ProfileList` del sistema. |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb`; `machine` o `user`. |
| `Path` | El programa (el nombre del valor), una ruta o línea de comandos de una base de datos CIT, o el programa al que se aplica una base de datos de shims. |
| `Layers` | Las capas de compatibilidad, tal como están escritas (`~ RUNASADMIN`). |
| `LastSeenUTC` | Cuándo cargó un programa un módulo CIT por última vez. |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | GUID, archivo y hora de instalación de una base de datos de shims. |
| `Details` | Nombre de un módulo CIT, nombre del valor de una base de datos CIT, identificador del medio extraíble de un programa (`SignMedia`), `DatabaseDescription` y `DatabaseType` de una base de datos de shims. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | De dónde procede el registro, cuándo cambió esa clave por última vez y si se encuentra en `Wow6432Node`. |
| `DataRaw` | Los datos `SACP` del valor `Store`, o datos que no tienen la forma esperada (hexadecimal). |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Datos que no se pudieron decodificar ni leer. |

## Estrategia de investigación { #investigation-strategy }

1. Revise todas las bases de datos de shims personalizadas: su archivo `.sdb`, la hora de instalación y los programas a los que
   se aplica; correlaciónelas con `sdbinst.exe` en Prefetch, Amcache y los eventos de creación de procesos.
2. Busque programas en carpetas con permisos de escritura para el usuario entre las entradas del Asistente para la compatibilidad de programas, así
   como programas procedentes de medios extraíbles.
3. Revise `RUNASADMIN` y otras capas en programas inusuales.
4. Correlacione con ShimCache, Amcache, BAM, Prefetch y UserAssist para obtener las horas de ejecución.

## Más información { #further-details }

- [Microsoft: base de datos de compatibilidad de aplicaciones](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
