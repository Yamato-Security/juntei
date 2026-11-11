# Accesos directos LNK { #lnk-shortcuts }

## Qué son los archivos LNK { #what-lnk-files-are }

Los archivos Shell Link (`.lnk`) de Windows describen un destino junto con los argumentos de inicio, el directorio de trabajo, el icono,
el estado de la ventana, los metadatos del destino, información del volumen y de red, elementos del espacio de nombres del shell e identificadores
opcionales de seguimiento de vínculos distribuidos. Windows y las aplicaciones los crean en Elementos recientes,
el Escritorio, el menú Inicio y otras ubicaciones; los atacantes también distribuyen directamente accesos directos maliciosos.

Un LNK de la carpeta Recent puede respaldar la interacción del usuario con un destino que ya no existe, incluida una
unidad extraíble o un recurso compartido de red. Un LNK cualquiera puede existir sin que se haya hecho clic en él nunca.

<span class="evidence-strength">Mejor conclusión:</span> el acceso directo se creó o actualizó con el
contexto de destino registrado. Su ubicación y los artefactos circundantes determinan si respalda un uso.

## Ubicaciones y recopilación { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

Busque en todo el perfil de usuario y en los archivos comprimidos y el contenido de correo electrónico recopilados, no solo en `Recent`. Conserve
las horas y la ruta propias del archivo de origen, ya que la ubicación cambia la interpretación.

## Precauciones para el analista { #analyst-cautions }

- Las horas del destino, el tamaño, la referencia MFT y los datos del volumen son instantáneas almacenadas en el LNK; no son
  los metadatos propios del archivo de acceso directo y pueden estar desactualizados.
- `MachineID`, la dirección MAC y la hora de creación del rastreador describen el contexto de seguimiento de vínculos distribuidos, normalmente
  el equipo que creó o actualizó el vínculo, y no necesariamente el equipo donde se encontró.
- Un nodo UUID con aspecto de MAC puede proceder de la virtualización, de la aleatorización o de identificadores no relacionados con la red.
- `Args` puede convertir en peligroso un ejecutable benigno (`powershell.exe`, `rundll32.exe`, `mshta.exe`).
- Las cadenas ANSI requieren la página de códigos del sistema de origen; utilice `--code-page 932` para muchos vínculos japoneses.
- Un acceso directo puede fabricarse o copiarse. No equipare sus campos con una ejecución sin el respaldo de Prefetch,
  Jump List, RecentDocs, registros de eventos u otras evidencias.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## Campos de salida CSV { #csv-output-fields }

| Grupo de campos | Campos e interpretación |
|---|---|
| Origen | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: ruta del acceso directo y horas del sistema de archivos del host. Son independientes de las horas del destino. |
| Horas del destino | `TargetCreated`, `TargetModified`, `TargetAccessed`: instantáneas registradas en el LNK; pistas históricas útiles, no metadatos actuales del destino. |
| Datos básicos del destino | `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`: contexto registrado del destino y del inicio. Los argumentos pueden cambiar el significado. |
| Volumen | `DriveType`, `VolumeSerialNumber`, `VolumeLabel`: identifican el contexto fijo, extraíble o de red y permiten correlacionar otros accesos directos. |
| Rutas | `LocalPath`, `NetworkPath`, `CommonPath`, `TargetIDAbsolutePath`: representaciones complementarias de la ruta. Revise las discrepancias y las rutas del espacio de nombres del shell. |
| Comportamiento de inicio | `Args`, `ShowWindow`, `HotKey`, `ShimLayer`, `DarwinId`: cómo inicia o configura el shell el destino. `Args` es fundamental. |
| Identidad NTFS | `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`: referencia del destino en el momento de crear el vínculo; haga coincidir ambos con `$MFT`. |
| Seguimiento | `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `VolumeDroid`, `FileDroid`, `VolumeDroidBirth`, `FileDroidBirth`: seguimiento de vínculos distribuidos. Utilícelos para correlacionar, no como única base de atribución. |
| Bloques adicionales | `ExtraBlocksPresent`: lista de las estructuras opcionales disponibles; explica qué otros campos están respaldados. |
| Presentación | `Name`, `IconLocation`, `IconIndex`, `IconEnvironment`, `EnvironmentVariables`: datos de visualización y del destino con las variables de entorno expandidas. Los atacantes pueden usar nombres o iconos engañosos. |
| Carpetas del shell | `KnownFolder`, `SpecialFolder`: identificadores decodificados de carpetas del shell de Windows. |
| Red | `NetworkDevice`, `NetworkProvider`: contexto del recurso compartido o dispositivo de red y del proveedor. |
| Codificación | `ConsoleCodePage`: página de códigos de la consola registrada, si existe; es independiente de la opción `--code-page` del analizador. |
| Datos finales | `AppendedBytes`: bytes posteriores al final lógico. Los datos inesperados pueden indicar contrabando de contenido o archivos con formato incorrecto. |
| Validación | `Problems`: daños, truncamiento, bloques incoherentes o limitaciones de decodificación. Inclúyalos en la revisión. |

Además, JSON/JSONL conserva cada elemento de ID de destino (tipo, nombre, GUID, horas FAT y referencia
MFT), todos los bloques de datos adicionales, las propiedades del almacén de propiedades, la configuración de la consola y las relaciones
estructurales sin procesar. Utilícelo para un acceso directo sospechoso en lugar de basarse únicamente en la fila CSV plana.
Las horas anidadas de `TargetItems` proceden de relojes FAT de los elementos del shell sin un desplazamiento almacenado.
Su representación heredada con `Z` no demuestra que estén en UTC; `auto-parse --timezone` las conserva.
Los FILETIME del destino del encabezado LNK y las horas del rastreador siguen siendo marcas de tiempo absolutas convertibles.

## Estrategia de investigación { #investigation-strategy }

1. Examine la ubicación y las horas del origen antes de interpretar los campos del destino.
2. Reconstruya el inicio efectivo a partir de la ruta de destino, `Args`, `WorkingDirectory` y los bloques de entorno.
3. Pivote sobre la referencia MFT del destino, el número de serie del volumen, la ruta de red, los droids, MachineID y la MAC en todas las evidencias.
4. Compruebe Prefetch y los registros de eventos para la ejecución; compruebe Jump Lists y RecentDocs para la interacción del usuario.
5. En el caso de adjuntos maliciosos, conserve los datos anexados y calcule el hash y analice el propio LNK.

## Más información { #further-details }

- [Microsoft Open Specifications: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
