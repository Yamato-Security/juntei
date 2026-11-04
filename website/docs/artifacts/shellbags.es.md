# ShellBags { #shellbags }

## Qué son los ShellBags { #what-shellbags-are }

El Explorador recuerda cómo se muestra cada carpeta que abre un usuario: su vista, el tamaño de los iconos, el criterio de ordenación y
la posición de la ventana. Para volver a encontrar la configuración de una carpeta, guarda la identidad de la carpeta en el
Registro del usuario como un árbol de elementos del shell, los mismos elementos que contiene la lista de ID de destino de un vínculo: una unidad y, después, cada
carpeta hasta llegar a la abierta. Estos árboles son los ShellBags.

Abarcan las carpetas abiertas en el Explorador y en los cuadros de diálogo Abrir y Guardar, incluidas las carpetas de unidades
extraíbles, recursos compartidos de red, dispositivos portátiles (teléfonos, cámaras), archivos ZIP y servidores FTP, y
las páginas del Panel de control. Permanecen después de que la carpeta, la unidad o el recurso compartido de red hayan desaparecido.

<span class="evidence-strength">Mejor conclusión:</span> el Explorador de este usuario mostró esta carpeta
(o una carpeta situada bajo ella). Un ShellBag no muestra qué se hizo en la carpeta ni fecha
cada visita: consulte las horas más abajo.

## Ubicación { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

Una clave `BagMRU` contiene valores numerados, cada uno de ellos un elemento del shell, y un valor `MRUListEx` que indica su
orden, con el abierto más recientemente en primer lugar. Cada valor tiene una subclave con el mismo nombre para los elementos
situados bajo él, de modo que una ruta son los elementos desde `BagMRU` hasta un valor. El `NodeSlot` de una subclave es el
número de su clave `Bags\<N>`, donde se encuentra la configuración de vista de la carpeta.

Recopile `UsrClass.dat` y `NTUSER.DAT` junto con sus registros de transacciones (`.LOG1`, `.LOG2`) para cada
perfil, y el hive `SOFTWARE` del sistema, que proporciona el SID del usuario de un `NTUSER.DAT`.

## Qué indica cada hora { #which-times-say-what }

- **Las horas del elemento del shell** (`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`) son las
  horas propias del sistema de archivos de la carpeta (precisión FAT: 2 segundos) cuando el Explorador escribió el elemento. Fechan
  la carpeta, no la visita, y no se actualizan en las visitas posteriores.
  Los relojes FAT no tienen un desplazamiento UTC almacenado. Los encabezados heredados `UTC` y el sufijo `Z` mostrado son
  convenciones de visualización, no una prueba de la zona horaria de origen. `auto-parse --timezone` conserva
  estos campos; interprételos con el contexto del sistema de origen.
- **La hora de última escritura de una clave** cambia cuando se reescribe su `MRUListEx`, es decir, cuando se abre uno de sus
  elementos y pasa a la primera posición. Por tanto, la hora de la clave en la que está el valor de un elemento
  (`ParentKeyLastWrittenUTC`) es la de la última apertura del elemento **solo cuando es el elemento más reciente
  de esa clave** (`IsMostRecentInParent` verdadero, `MruPosition` 0). Para los demás elementos, es una
  hora posterior, en la que se abrió un elemento hermano.
- La clave propia de un elemento (`KeyLastWrittenUTC`) fecha de la misma forma la apertura de su elemento secundario más
  reciente.

## Precauciones para el analista { #analyst-cautions }

- **La ausencia demuestra poco.** El Explorador escribe los ShellBags cuando se guarda la vista de una carpeta; una carpeta
  vista solo brevemente, o abierta de otro modo (una línea de comandos, un script, un programa), puede no tener ninguno.
  Un usuario puede eliminar las claves.
- **Una referencia NTFS es la de la carpeta** (`MftEntry`, `MftSequence`) cuando se escribió el elemento.
  Búsquela en la `$MFT` para ver si la misma carpeta sigue existiendo o si otro archivo reutilizó la
  entrada.
- **Las unidades extraíbles** se nombran por su letra de unidad. La letra es la que tenía la unidad en ese momento;
  hágala coincidir con el historial de dispositivos USB y de dispositivos montados para identificar el dispositivo.
- **Los nombres proceden del elemento.** Una carpeta renombrada después conserva aquí su nombre antiguo. Las carpetas raíz y las carpetas
  conocidas se nombran como suelen nombrarlas las herramientas forenses (`This PC`, `Quick Access`, `Downloads`);
  `Guid` proporciona su identificador.
- **Los resultados de búsqueda, las bibliotecas y las páginas del Panel de control** son vistas, no carpetas en disco
  (`property_view`, `control_panel_item`, `control_panel_category`).
- **Un elemento que ningún analizador conoce** conserva sus bytes en `DataRaw`, con `ItemType` `unknown` y su
  byte de clase en `ItemClass`. Los elementos dañados o inusuales se notifican en `Problems`, y también se conservan
  sus bytes.
- **Un hive con cambios pendientes** puede carecer de sus elementos más recientes si no se dispone de sus registros. Juntei reproduce los registros situados junto al
  hive salvo que se indique `--no-logs`; `TransactionLogStatus` indica si un registro procede de ellos.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

Los elementos del shell se decodifican con ys-lnk-parser, el analizador de elementos del shell de `parse-file lnk`. El
resumen enumera las últimas carpetas abiertas (las más recientes primero), según la hora de última escritura de su clave principal,
con sus usuarios.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | Host, nombre de la carpeta del perfil, SID del usuario (lo indica la clave raíz de un `UsrClass.dat`; para `NTUSER.DAT`, el `ProfileList` del sistema), el hive y su tipo. |
| `BagKey` | Clave del elemento bajo la raíz del hive (`…\BagMRU\4\3\0`): su valor está en la clave superior. |
| `MRU-Position`, `IsMostRecentInParent` | Su posición en el `MRUListEx` de su clave principal (0: el abierto más recientemente); vacío cuando la lista no lo contiene. |
| `NodeSlot` | Número de su clave `Bags\<N>` (la configuración de vista de la carpeta). |
| `Path` | Los elementos desde la parte superior de su árbol hasta él, unidos con `\`. `[unknown]` representa un elemento cuyo valor falta. |
| `ItemType`, `ItemClass` | Qué es el elemento (`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`) y su byte de clase. |
| `ItemName`, `GUID` | Su nombre (el nombre largo de una carpeta) y el identificador de clase de una carpeta raíz, un elemento del Panel de control o una carpeta delegada. |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **Del elemento del shell:** las horas de la carpeta cuando se escribió el elemento (FAT, precisión de 2 segundos). |
| `MFT-Entry`, `MFT-Sequence` | **Del elemento del shell:** la referencia de archivo NTFS de la carpeta. |
| `ExtensionBlocks` | Cuántos bloques de extensión tiene el elemento. |
| `KeyLastWrittenUTC` | **Del Registro:** la última escritura de la clave propia del elemento: cuándo se abrió su elemento secundario más reciente. |
| `ParentKeyLastWrittenUTC` | **Del Registro:** la última escritura de la clave en la que está su valor: cuándo se abrió por última vez, si `IsMostRecentInParent` es verdadero. |
| `DataRaw` | Bytes del valor (en hexadecimal), para un elemento que no se entiende por completo o que tiene problemas. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Un valor que nombra el `MRUListEx` de su clave pero que la clave no tiene, un valor que la lista no nombra, una subclave sin su valor, un valor que no es binario, un `NodeSlot` de tipo incorrecto, una lista de elementos del shell dañada. |

## Estrategia de investigación { #investigation-strategy }

1. Enumere las unidades extraíbles (`volume` con el `Guid` de Unidades extraíbles), las ubicaciones de red, las carpetas
   FTP y los dispositivos portátiles, y las carpetas situadas bajo ellos.
2. Busque carpetas de interés: ubicaciones de preparación con permisos de escritura para el usuario (`Temp`, `ProgramData`, `Public`),
   carpetas de herramientas y de archivos comprimidos, perfiles de otros usuarios y carpetas que ya no existen (compruebe la
   referencia NTFS en la `$MFT`).
3. Feche las visitas solo con el `ParentKeyLastWrittenUTC` de los elementos cuyo `IsMostRecentInParent` es verdadero;
   correlacione el resto con los archivos LNK, las Jump Lists y RecentDocs, que fechan las aperturas de archivos.
4. Utilice `parse-registry read --recover-deleted` para las claves y los valores de `BagMRU` eliminados.

## Más información { #further-details }

- [libyal libfwsi: formato Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

La búsqueda del SID en `ProfileList` requiere un único SID sintácticamente válido. Las correspondencias ambiguas de carpetas de perfil y las claves de copia de seguridad como `S-1-...bak` dejan `UserSid` como desconocido.
