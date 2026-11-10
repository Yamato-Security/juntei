# Recopilación de evidencias { #collecting-evidence }

Juntei analiza archivos ya recopilados de Windows. Use una herramienta de adquisición fiable y preserve los metadatos originales cuando sea posible.

[auto-parse --vhdx](../commands/auto-parse.md#vhdx-disk-images): Linux / macOS / Windows.

## Ubicaciones habituales { #common-locations }

| Artefacto | Ubicación típica |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | Archivo de metadatos del volumen NTFS `C:\$MFT` |
| `$LogFile` | Archivo de metadatos del volumen NTFS `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| Diario USN | Flujo de metadatos NTFS `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| Registro de acceso de usuarios (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| Internet Explorer y Edge heredado (WebCache) | `%LOCALAPPDATA%\Microsoft\Windows\WebCache\` |
| Outlook | `%LOCALAPPDATA%\Microsoft\Outlook\*.ost`, `%USERPROFILE%\Documents\Outlook Files\*.pst` (`%LOCALAPPDATA%\Microsoft\Outlook\*.pst` antes de Outlook 2010), `%APPDATA%\Microsoft\Templates\*.oft`, y los archivos `.msg` dondequiera que se hayan guardado |
| Navegadores web | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\` y las carpetas de perfil de los demás navegadores |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` y otras ubicaciones de usuario/sistema |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` y `CustomDestinations` |
| ShimCache | Hive `SYSTEM`: `...\Session Manager\AppCompatCache\AppCompatCache` |
| Cuarentena de Defender | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| Registro del equipo | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| Registro del usuario | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## Archivos que deben recopilarse juntos { #files-that-must-travel-together }

- Recopile los hives del Registro junto con los archivos `.LOG`, `.LOG1` y `.LOG2` adyacentes.
  Juntei puede reproducir los registros de transacciones para los comandos del Registro y para
  ShimCache cuando el hive está sucio (dirty).
- Recopile el `SRUDB.dat` de SRUM de forma coherente. Actualmente, Juntei informa de que una base de
  datos ESE está sucia, pero no reproduce `SRU*.log`; los registros que solo existen en esos
  archivos de registro no están disponibles.
- Recopile `$MFT` junto con `$Extend/$J` siempre que sea posible. El registro USN almacena
  referencias al archivo principal, no una ruta completa; la MFT mejora la resolución de rutas.
- Recopile `$LogFile` con la `$MFT` del mismo volumen, lo más próximas en el tiempo que sea
  posible. `$LogFile` se sobrescribe rápidamente; una recopilación posterior puede perder por
  completo los registros redo/undo anteriores.
- Recopile SRUM con el hive `SOFTWARE` correspondiente para resolver los nombres de cuenta y de
  perfil Wi-Fi.
- Recopile todo el directorio `Sum` del registro de acceso de usuarios: `SystemIdentity.mdb` proporciona
  los nombres de los roles y enumera las bases de datos anuales `{GUID}.mdb`. Al igual que con SRUM,
  Juntei no reproduce los registros de transacciones `Svc*.log` de UAL.
- Recopile las bases de datos de los navegadores con los archivos `-wal` que están junto a ellas
  (`places.sqlite-wal`, `cookies.sqlite-wal`): los cambios más recientes, y las versiones anteriores de las páginas
  de las que se recuperan las entradas eliminadas, solo están ahí hasta que el navegador ejecuta un punto de
  control.
- Recopile toda la carpeta `WebCache`: Windows mantiene abierto `WebCacheV01.dat` mientras el usuario tiene la
  sesión iniciada, y sus cambios más recientes solo están en los registros de transacciones `V01*.log`, que Juntei
  no reproduce.
- Recopile los archivos OST y PST de Outlook con Outlook cerrado, o desde una imagen de disco o una instantánea de
  volumen: Outlook los mantiene abiertos mientras se ejecuta. Busque también archivos PST fuera de las carpetas
  predeterminadas: los usuarios mueven los PST archivados a otras carpetas, unidades y recursos compartidos de red.
- Recopile todo el árbol `Quarantine` de Defender para poder relacionar Entries, ResourceData y
  Resources.

## Precauciones en sistemas en vivo { #live-system-cautions }

Los archivos abiertos pueden cambiar durante una copia. Los hives del Registro y las bases de datos
ESE pueden estar internamente sucios. Es posible que ShimCache no vuelque sus entradas más
recientes en memoria hasta el apagado. Es preferible una herramienta de recopilación forense que
utilice instantáneas de volumen o acceso sin procesar (raw) a una copia de archivos convencional.

## Seguridad { #safety }

Las evidencias pueden contener malware, muestras de exploits, archivos LNK maliciosos y texto con
fórmulas de hoja de cálculo. Realice el análisis en un sistema aislado, no ejecute los flujos
recuperados y utilice `--extract` solo cuando tenga intención de manipular contenido peligroso.
Conserve por separado la fuente original sin modificar.
