# FeatureUsage, RecentApps y elementos anclados a la barra de tareas { #featureusage-recentapps-and-taskbar-pins }

## Qué son { #what-they-are }

El Explorador conserva en el `NTUSER.DAT` del usuario varios registros de cómo utiliza este las aplicaciones:

- **FeatureUsage** (Windows 10 y posteriores): contadores por aplicación, según el tipo de uso.
  - `AppSwitched`: cuántas veces cambió el usuario a la ventana de la aplicación (clic en la barra de tareas,
    Alt+Tab).
  - `AppLaunch`: cuántas veces se inició desde la barra de tareas (un elemento anclado).
  - `ShowJumpView`: cuántas veces se abrió su Jump List (clic con el botón derecho en la barra de tareas).
  - `AppBadgeUpdated`: cuántas veces cambió su distintivo en la barra de tareas (un recuento de elementos no leídos).
  - `TrayButtonClicked`: cuántas veces se hizo clic en cada botón de la barra de tareas (`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …).

  Una aplicación es un identificador de modelo de usuario de aplicación (`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`), una ruta cuya primera parte puede ser el GUID de una
  carpeta conocida (`{1AC14E77-…}\cmd.exe` es `%windir%\system32\cmd.exe`) o un identificador de proceso
  (`*PID00001d74`) para las ventanas que el Explorador no pudo asociar a una aplicación.
- **RecentApps** (de Windows 10 hasta la versión 1709): las aplicaciones que un usuario ejecutó recientemente, con un recuento
  de inicios y la última hora; debajo de cada una, los archivos que abrió (`RecentItems`).
- **Elementos anclados a la barra de tareas** (`Taskband`): los programas anclados a la barra de tareas, como elementos del shell
  (`Favorites`) y como el vínculo de cada acceso directo anclado (`FavoritesResolve`). En Windows 10 y
  posteriores, cada elemento anclado indica si lo ancló Windows (`SystemPinned`) o el usuario (`UserPinned`).
- **ProgramsCache** (`StartPage`, `StartPage2`; de Windows XP a 7): los programas en caché del menú Inicio
  (`ProgramsCache`) y, en Windows 7, los anclados al menú Inicio y a la barra de tareas
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`).

<span class="evidence-strength">Mejor conclusión:</span> este usuario utilizó esta aplicación este
número de veces de esta forma (FeatureUsage), la ejecutó a esta hora (RecentApps) o la ancló. Los
contadores no tienen hora propia; solo la última escritura de la clave y las horas de RecentApps permiten fechar
algo.

## Ubicación { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

El `KeyCreationTime` de `FeatureUsage` indica cuándo empezó a contar; el resumen lo muestra. Recopile
`NTUSER.DAT` con sus registros de transacciones para cada perfil, así como los hives `SYSTEM` y
`SOFTWARE` del sistema para obtener el host y el SID del usuario.

## Precauciones para el analista { #analyst-cautions }

- **Recuentos sin horas.** Un recuento de FeatureUsage indica con qué frecuencia, no cuándo. La hora de última escritura
  de la clave del tipo (`KeyLastWrittenUTC`) corresponde al cambio más reciente de cualquiera de sus recuentos.
- **No todas las ejecuciones.** `AppSwitched` cuenta los cambios de ventana, no los inicios; una aplicación que se ejecutó
  sin ventana, o a la que el usuario nunca cambió, no se cuenta. `AppLaunch` solo cuenta los
  inicios desde la barra de tareas.
- **Aplicaciones empaquetadas y rutas.** Una aplicación recibe el nombre con el que Windows agrupó sus ventanas: el identificador de una
  aplicación empaquetada, un identificador que el propio programa estableció (`Chrome`, `VMware.Workstation.vmui`) o una
  ruta. `AppPath` proporciona la ruta con el GUID de la carpeta conocida sustituido por la ruta
  predeterminada de la carpeta.
- **Elementos anclados predeterminados.** Los perfiles nuevos reciben elementos anclados predeterminados (Explorador de archivos, Edge, la Store); en Windows 10
  y posteriores indican `SystemPinned`. Un elemento anclado `UserPinned` es una elección del usuario.
- **Elementos anclados en dos listas.** `Favorites` y `FavoritesResolve` describen los mismos elementos anclados en el mismo
  orden (`Index`), pero se proporcionan como registros independientes.
- **Las rutas de ProgramsCache** son relativas: una carpeta (`Programs\Accessories`) va seguida de los
  accesos directos que contiene (`Notepad.lnk`).
- **Valores dañados.** Un recuento que no es `REG_DWORD`, una hora que no es `REG_QWORD`, un elemento anclado
  que no se entiende o un vínculo que no lo es conserva sus bytes (`DataRaw`) y se notifica en
  `Problems`.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

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
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

El resumen enumera los registros por origen, cuándo empezó a contar FeatureUsage en cada hive, las
aplicaciones a las que más se cambió, las aplicaciones y elementos recientes (los más nuevos primero) y los elementos anclados a la barra de tareas.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, nombre de la carpeta del perfil, SID del usuario según el `ProfileList` del sistema y el hive. |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked` (FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` o `ProgramsCache`. |
| `App` | Nombre de un valor de FeatureUsage, `AppId` de una aplicación reciente o identificador de modelo de usuario de aplicación de un elemento anclado. |
| `AppPath` | `App` como ruta, con el GUID de una carpeta conocida sustituido por su ruta predeterminada; vacío para los identificadores. |
| `Count` | Un recuento de FeatureUsage o el `LaunchCount` de una aplicación reciente. |
| `LastAccessedUTC` | El `LastAccessedTime` de una aplicación o un elemento recientes. |
| `ItemPath` | El `Path` de un elemento reciente (o el `AppPath` de una aplicación reciente); la ruta del elemento del shell de un elemento anclado o de un programa en caché (`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`); la ruta de un vínculo de `FavoritesResolve`. |
| `DisplayName` | El `DisplayName` de un elemento reciente. |
| `PinType` | `SystemPinned` o `UserPinned`, para un elemento anclado de `Favorites`. |
| `Index` | Posición de un elemento anclado o de un programa en caché en su lista. |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | El valor y la clave de los que procede, y la última escritura de la clave. |
| `DataRaw` | Bytes (hexadecimal) que no se pueden decodificar. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Valores del tipo incorrecto, elementos anclados o listas que no se entienden, vínculos que no son vínculos. |

## Estrategia de investigación { #investigation-strategy }

1. Ordene `AppSwitched` por `Count` para conocer las aplicaciones con las que más trabajó un usuario y busque
   herramientas de administración, acceso remoto, compresión y transferencia, así como rutas en ubicaciones con permisos de escritura para el usuario
   (`\Users\`, `Downloads`, `Temp`).
2. Revise los elementos anclados: una herramienta `UserPinned` o un elemento anclado cuyo acceso directo se encuentre en una ubicación inusual.
3. Utilice las horas de RecentApps, junto con UserAssist, Prefetch, BAM y Jump Lists, para fechar el uso.
4. Compare `KeyCreationTime` con la creación del perfil: los recuentos comienzan entonces.
5. Utilice `parse-registry read --recover-deleted` para los contadores y elementos anclados eliminados.

## Más información { #further-details }

- [Microsoft: identificadores de modelo de usuario de aplicación](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: caché de programas](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
