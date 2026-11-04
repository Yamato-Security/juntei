# BAM / DAM { #bam-and-dam }

## Qué son BAM y DAM { #what-bam-and-dam-are }

El moderador de actividad en segundo plano (BAM, Background Activity Moderator) y el moderador de actividad de escritorio (DAM, Desktop Activity Moderator) son servicios
de Windows (Windows 10 1709 y posteriores) que controlan lo que las aplicaciones pueden hacer en segundo plano.
Guardan, por usuario, la última vez que se ejecutó cada programa, en el hive `SYSTEM`:

- una clave por usuario, cuyo nombre es el SID del usuario;
- un valor por programa: su nombre es la ruta del programa en su dispositivo
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) o el identificador de una aplicación empaquetada
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`);
- los primeros 8 bytes de los datos del valor son la hora de la última ejecución (FILETIME).

BAM vincula un programa con un usuario y una hora sin necesidad del hive propio del usuario, y enumera los programas ejecutados
desde dispositivos extraíbles y de red.

<span class="evidence-strength">Mejor conclusión:</span> un proceso de este programa se ejecutó en la sesión de este
usuario; su última ejecución fue a esta hora. BAM no cuenta las ejecuciones, y una entrada puede permanecer después de que
el programa se haya eliminado.

## Ubicación { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

La clave de cada usuario también tiene los valores `Version` y `SequenceNumber`; no son programas y se
omiten. Un sistema actualizado desde 1709 o 1803 puede tener ambas estructuras.

Recopile `SYSTEM` junto con sus registros de transacciones, y los hives `SAM` y `SOFTWARE` del sistema: indican los nombres de los
usuarios.

## Precauciones para el analista { #analyst-cautions }

- **Una hora por programa y usuario.** Solo se conserva la ejecución más reciente. Prefetch, Amcache y
  los registros de eventos proporcionan las ejecuciones anteriores.
- **Actividad reciente.** Según se ha documentado, las entradas de programas que llevan un tiempo sin ejecutarse se
  eliminan al iniciarse Windows, por lo que BAM muestra sobre todo actividad reciente: un programa ausente
  puede haberse ejecutado hace mucho tiempo.
- **Rutas de dispositivo, no letras de unidad.** `\Device\HarddiskVolume3` es el tercer volumen que numeró Windows,
  no `C:`. `DevicePath` y `Path` separan el dispositivo de la ruta dentro de él; haga coincidir el
  volumen con `MountedDevices`, la `$MFT` u otros artefactos antes de indicar una letra de unidad.
- **No todos los programas.** BAM y DAM registran programas en las sesiones de los usuarios; los servicios no suelen
  aparecer. Las entradas `S-1-5-18` suelen ser procesos de la instalación y de la pantalla de inicio de sesión
  (`FirstLogonAnim.exe`), y las `S-1-5-90-0-<n>`, del Administrador de ventanas del escritorio (`DWM-<n>`).
- **Conjuntos de controles.** De forma predeterminada se lee el último conjunto de controles que utilizó Windows. `--all-control-sets`
  los lee todos: un conjunto de controles anterior puede conservar una entrada que el actual ya no tiene.
- **Nombres de usuario.** El nombre de una cuenta local procede del `SAM` (el SID del equipo y el
  RID de la cuenta); el de una cuenta de dominio, de la carpeta de su perfil en `ProfileList`. `UserSource`
  indica cuál (`sam`, `profile_list`, `well_known`). Sin ellos, solo se proporciona el SID. Un RID
  que el `SAM` ya no tiene (una cuenta eliminada) se queda sin nombre.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

El resumen enumera cada usuario con su SID, su nombre y su número de entradas, y los últimos programas ejecutados
(los más recientes primero).

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath` | Host y el hive `SYSTEM`. |
| `ControlSet`, `IsCurrentControlSet` | Conjunto de controles en el que está la entrada y si es el último que utilizó Windows. |
| `Moderator` | `bam` o `dam`. |
| `UserSID`, `User`, `UserSource` | SID que da nombre a la clave, nombre del usuario y su procedencia (`sam`, `profile_list`, `well_known`). |
| `Program` | Nombre del valor tal como lo escribió Windows: una ruta de dispositivo o el identificador de una aplicación empaquetada. |
| `DevicePath`, `Path` | Para una ruta de dispositivo, el dispositivo (`\Device\HarddiskVolume3`) y la ruta dentro de él (`\Windows\System32\cmd.exe`); vacío para una aplicación empaquetada. |
| `LastRunUTC` | Cuándo se ejecutó el programa por última vez (con precisión de 100 ns). |
| `KeyPath`, `KeyLastWrittenUTC` | La clave del usuario y su última escritura: la actualización más reciente de cualquiera de sus entradas. |
| `DataRaw` | Bytes del valor (en hexadecimal) cuando no contienen una hora: no es `REG_BINARY` o tiene menos de 8 bytes. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Datos que no contienen una hora y valores que no se pueden leer. |

## Estrategia de investigación { #investigation-strategy }

1. Ordene por `LastRunUTC` y examine los programas situados en ubicaciones con permisos de escritura para el usuario (`\Users\`, `Downloads`,
   `Temp`, `ProgramData`), en dispositivos distintos del volumen del sistema, y las herramientas de administración y
   de acceso remoto.
2. Compruebe qué usuario los ejecutó y las cuentas que no coinciden con un perfil actual.
3. Correlacione con Prefetch (recuentos de ejecuciones y ejecuciones anteriores), UserAssist (inicios desde el Explorador), Amcache
   y ShimCache (presencia), la `$MFT` (el archivo del programa) y los registros de eventos.
4. Utilice `--all-control-sets`, y `parse-registry read --recover-deleted` para los valores de BAM eliminados.

## Más información { #further-details }

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: identificadores de seguridad (SID conocidos)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

Los FILETIME de última ejecución fuera de rango dejan `LastRunUTC` vacío y conservan el valor original en `DataRaw`, con una explicación en `Problems`. El resumen de actividad reciente identifica cada host y mantiene separadas las observaciones de cada hive y usuario.
