# UserAssist { #userassist }

## Qué es UserAssist { #what-userassist-is }

El Explorador de Windows cuenta los programas y los accesos directos que un usuario inicia desde el shell del Explorador: el
menú Inicio, la barra de tareas, el escritorio y los dobles clics en carpetas. Guarda los recuentos en el
`NTUSER.DAT` del usuario. Cada elemento tiene:

- un recuento de ejecuciones;
- en Windows 7 y posteriores, cuántas veces y durante cuánto tiempo (en milisegundos) tuvo el foco;
- cuándo se ejecutó por última vez.

Los nombres de los valores están codificados con ROT13. El nombre de un programa suele empezar por el GUID de una carpeta conocida
(`{1AC14E77-…}` es `System32`).

UserAssist muestra lo que un usuario concreto inició de forma interactiva, algo que la mayoría de las demás evidencias
de ejecución no atribuyen a un usuario.

<span class="evidence-strength">Mejor conclusión:</span> el usuario inició este programa o este acceso directo
a través del Explorador, este número de veces, la última a esta hora. Los programas iniciados de otro modo (una línea
de comandos, un servicio, una tarea programada, otro programa) no se cuentan.

## Ubicación { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | Contiene |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | Programas (`executable`), Windows 7 y posteriores |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | Accesos directos (`shortcut`), Windows 7 y posteriores |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | Programas y accesos directos (`active_desktop`), de Windows 2000 a Vista |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | Barra de herramientas de Internet (`internet_toolbar`), de Windows 2000 a Vista |

Los demás GUID se notifican como `other`. Recopile `NTUSER.DAT` junto con sus registros de transacciones para cada
perfil, y el hive `SOFTWARE` del sistema, que proporciona el SID del usuario.

## Precauciones para el analista { #analyst-cautions }

- **Los perfiles nuevos vienen precargados.** En Windows 7 y posteriores, un perfil nuevo empieza con elementos predeterminados
  (Introducción, Calculadora, Paint, Notas rápidas, …). Sus recuentos de ejecuciones descienden desde 14,
  y todos comparten una misma hora de última ejecución, en torno al momento en que se creó el perfil. No son actividad
  del usuario.
- **La versión 3 cuenta a partir de 5** (de Windows 2000 a Vista). El recuento almacenado empieza en 5, por lo que se le
  resta 5. Un recuento almacenado inferior a 5 se notifica en `Problems`.
- **Las rutas usan la ruta predeterminada de la carpeta conocida** (`%windir%\system32`, `%ProgramFiles%`,
  `%APPDATA%\…`). Una carpeta redirigida a otro lugar sigue mostrando su ruta predeterminada. Los identificadores de aplicación
  (`Microsoft.Windows.Explorer`, `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) y los elementos del Panel de control
  (`::{…}`) no tienen ruta.
- **El seguimiento se puede desactivar** con `Explorer\Advanced` `Start_TrackProgs` = 0,
  `Policies\Explorer` `NoInstrumentation` = 1 o `UserAssist\Settings` `NoLog` = 1. El resumen
  enumera los hives en los que está desactivado. Un UserAssist vacío o ausente en un perfil activo puede indicar
  técnicas antiforenses.
- **`NoEncrypt` = 1** hace que Windows escriba los nombres sin ROT13. Esos nombres se decodifican mal, por lo que
  todos los elementos de ese hive lo indican en `Problems` (y en stderr cuando los registros se envían a stdout).
- **La hora de última ejecución y los recuentos se guardan por elemento,** pero la hora de última escritura de la clave
  (`KeyLastWrittenUTC`) es solo la hora de la actualización más reciente de cualquier elemento.
- **Los recuentos de ejecuciones pueden ser 0** en los elementos que solo tuvieron el foco. Los recuentos y las horas de foco solo existen en
  la versión 5 (Windows 7 y posteriores).

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

El resumen enumera los últimos elementos ejecutados (los más recientes primero) con sus usuarios, y los hives en los que
UserAssist está desactivado.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, nombre de la carpeta del perfil, SID del usuario según el `ProfileList` del sistema (con el hive `SOFTWARE` del sistema) y el hive. |
| `GUID`, `Kind` | La clave `{GUID}` y lo que contiene (`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`). |
| `NameRot13`, `Name` | Nombre del valor tal como se almacena, y decodificado. |
| `Path` | El nombre decodificado como ruta, con el GUID de una carpeta conocida sustituido por su ruta predeterminada; vacío para los identificadores de aplicación y los elementos del shell. |
| `RunCount` | Cuántas veces se inició desde el Explorador (versión 3: el recuento almacenado menos 5). |
| `FocusCount`, `FocusTimeMs` | Cuántas veces tuvo el foco y durante cuánto tiempo, en milisegundos (versión 5). |
| `LastRunUTC` | Cuándo se inició por última vez. Vacío cuando Windows almacenó 0 o cuando la hora almacenada es posterior a 9999 (en ese caso `Problems` la indica y `DataRaw` conserva los bytes). |
| `Format`, `SessionID` | Estructura de los datos (5: 72 bytes, 3: 16 bytes) y número de la sesión en la que se escribió por última vez. |
| `KeyLastWrittenUTC` | Última escritura de la clave `Count`: la actualización más reciente de cualquiera de sus elementos. |
| `DataRaw` | Bytes del valor (en hexadecimal) cuando no se pueden decodificar. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Datos de tamaño desconocido, un tamaño que no concuerda con la `Version` de la clave, un recuento de versión 3 inferior a 5, una última ejecución posterior a 9999, `NoEncrypt`. |

## Estrategia de investigación { #investigation-strategy }

1. Ordene por `LastRunUTC` y aparte los elementos precargados: comparten una misma hora, en torno a la creación
   del perfil.
2. Busque programas situados en ubicaciones con permisos de escritura para el usuario (`%USERPROFILE%`, `Downloads`, `Temp`,
   `ProgramData`), en unidades extraíbles y rutas de red, y herramientas de administración y de acceso
   remoto.
3. Correlacione con Prefetch (ejecución con horas de ejecución), Amcache y ShimCache (presencia),
   las Jump Lists y los archivos LNK (lo que se abrió con ellos) y la `$MFT` para los archivos de los programas.
4. Revise los hives en los que el seguimiento está desactivado, y los perfiles cuyo UserAssist está vacío, en busca de borrados.
5. Utilice `parse-registry read --recover-deleted` para los valores de UserAssist eliminados.

## Más información { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: User Assist keys](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
