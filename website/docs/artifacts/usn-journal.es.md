# Diario USN (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## Qué es el Diario USN { #what-the-usn-journal-is }

NTFS registra los cambios de archivos y directorios en el diario de cambios Update Sequence Number
(USN). Un registro puede describir la creación, la ampliación o sobrescritura de datos, cambios de
atributos o de seguridad, el cambio de nombre, el cierre y la eliminación. Almacena una referencia
de archivo, una referencia al directorio principal, el nombre, los motivos, los atributos, una
marca de tiempo y un USN que aumenta de forma monótona.

El diario es excelente para reconstruir una ráfaga de operaciones de archivos de un atacante,
especialmente cuando los archivos se eliminaron. Registra **que los metadatos o el contenido
cambiaron**, no los bytes modificados.

<span class="evidence-strength">Mejor conclusión:</span> NTFS registró los motivos de cambio
indicados para el archivo referenciado en la marca de tiempo. Los motivos suelen acumularse hasta
un registro `CLOSE`.

## Ubicación y recopilación { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` es un flujo con nombre de los metadatos de NTFS. Recopílelo con una herramienta forense con
acceso sin procesar (raw). Recopile también la `$MFT` del mismo volumen, porque los registros USN
almacenan referencias al archivo principal en lugar de rutas completas.

## Precauciones para el analista { #analyst-cautions }

- El diario es circular y de tamaño limitado. Los registros antiguos se sobrescriben.
- `FILE_DELETE` indica que NTFS procesó la eliminación; el contenido puede seguir siendo
  recuperable en otro lugar.
- Un motivo como `DATA_OVERWRITE` no revela los bytes ni el proceso o usuario responsable.
- Se pueden combinar varios motivos en un mismo registro; `CLOSE` suele marcar el final de los
  cambios acumulados.
- Las rutas se reconstruyen. Los directorios principales pueden haberse renombrado, eliminado o
  reutilizado.
- Los números de secuencia de las referencias de archivo importan. El número de entrada por sí solo
  puede vincularse a la generación equivocada.
- El carving puede recuperar registros válidos sin su contexto original en el diario; indique esa
  distinción en el informe.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## Campos de salida CSV { #csv-output-fields }

| Campo | Interpretación |
|---|---|
| `Name` | Nombre del archivo o directorio almacenado en este registro USN. No es una ruta completa. |
| `Extension` | Extensión de conveniencia extraída de `Name`; no la utilice como un tipo de contenido fiable. |
| `EntryNumber`, `SequenceNumber` | Referencia NTFS del archivo. Utilice ambos para identificar la generación del registro. |
| `ParentEntryNumber`, `ParentSequenceNumber` | Referencia NTFS del directorio principal en el momento del cambio. |
| `ParentPath` | Ruta del directorio principal reconstruida por Juntei teniendo en cuenta el tiempo. Los marcadores de desconocido indican que no se pudo resolver la ascendencia. |
| `UpdateSequenceNumber` | Valor de orden USN. Útil para una ordenación estable y para pivotar con Amcache/MFT. |
| `UpdateTimestamp` | Marca de tiempo almacenada en los registros V2/V3. Valide las marcas de tiempo en contexto recuperado por carving igual que las de cualquier registro. |
| `UpdateReasons` | Uno o varios indicadores de cambio con nombre, unidos por `|`. Interprete las combinaciones y `CLOSE` en conjunto. |
| `FileAttributes` | Atributos NTFS registrados en el momento del cambio; los bits desconocidos permanecen en hexadecimal. |
| `OffsetToData` | Desplazamiento en bytes dentro de la entrada. Esencial para la validación y para los registros recuperados por carving. |
| `SourceFile` | Diario o imagen de entrada; distingue hosts y volúmenes. |
| `SourceInfo` | Indicadores de origen que describen cómo entró el cambio en el diario. Normalmente aportan contexto, no atribución. |
| `SecurityID` | Índice del identificador de seguridad NTFS del registro, no un SID de usuario. |
| `Version` | Versión del registro USN. Juntei escribe V2/V3 en CSV; JSON conserva las extensiones de seguimiento de rangos de V4. |

JSON/JSONL representa correctamente las referencias de 128 bits de V3 y conserva los registros de
seguimiento de rangos de V4 y sus extensiones. Utilícelo cuando importen los campos de bajo nivel
específicos de cada versión.

## Estrategia de investigación { #investigation-strategy }

1. Filtre por hora, nombre, extensión, ruta del directorio principal o referencia de archivo.
2. Ordene por `UpdateSequenceNumber` además de por hora para ver el orden exacto del diario.
3. Empareje `RENAME_OLD_NAME` con `RENAME_NEW_NAME` para los movimientos y cambios de nombre.
4. Utilice `$MFT` para validar el estado de los registros, las rutas, los tamaños y las marcas de
   tiempo.
5. Correlacione la atribución de procesos con los registros de eventos, EDR, Prefetch,
   servicios/tareas o Defender.

## Más información { #further-details }

- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: indicadores de motivo del diario de cambios](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [Crate usn de ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
