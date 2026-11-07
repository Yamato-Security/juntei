# Recopilación de evidencias { #collecting-evidence }

Juntei analiza archivos ya recopilados de Windows. Utilice una herramienta de adquisición de
confianza y conserve los metadatos originales siempre que sea posible.
La salida `--vhdx` de KAPE puede analizarse sin extraerla: consulte
[Montar imágenes de disco](../commands/mount.md).

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
