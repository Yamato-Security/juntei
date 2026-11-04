# Sector de arranque (`$Boot`) { #boot-sector-boot }

## Qué es `$Boot` { #what-boot-is }

`$Boot` es la entrada 7 de la MFT de todo volumen NTFS: los primeros 8 KiB del volumen. Su primer sector, el
**registro de arranque del volumen** (VBR), contiene el bloque de parámetros del BIOS que describe el sistema de archivos:

- los tamaños de sector y de clúster, el número de sectores del volumen y los **sectores ocultos**
  (los sectores del disco anteriores al volumen, es decir, dónde empieza su partición);
- dónde empiezan la `$MFT` y la `$MFTMirr`, y los tamaños de los registros FILE e INDX;
- el **número de serie del volumen** de 64 bits, establecido al formatear el volumen.

El resto del primer sector y los 15 sectores siguientes contienen el **código de arranque** que carga
Windows.

Los 32 bits inferiores del número de serie son lo que Windows muestra como `XXXX-XXXX` (`vol`, `dir`) y lo que
los [accesos directos LNK](lnk.md), las [Jump Lists](jump-lists.md) y [Prefetch](prefetch.md) registran del
volumen en el que estaba un archivo. `$Boot` vincula esos registros con un volumen de un host recopilado.

<span class="evidence-strength">Mejor conclusión:</span> un número de serie de volumen de LNK, Jump List o Prefetch
igual al `VolumeSerialNumber` de un `$Boot` apunta a ese volumen, o a otro volumen al que se
asignó el mismo número de serie o que se copió con él. Los números de serie se pueden cambiar y no son
únicos entre hosts.

## Ubicación y recopilación { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

Recopílelo con una herramienta forense con acceso sin procesar (raw), o lea el primer sector de una imagen de volumen: ambos son
los mismos bytes. Juntei lee cualquier archivo que empiece por un sector de arranque NTFS, por lo que una imagen de volumen sirve
como entrada. Una imagen de disco, en cambio, empieza por la tabla de particiones; proporcione a Juntei el volumen de la partición.

## Precauciones para el analista { #analyst-cautions }

- El número de serie cambia al formatear el volumen. Un archivo LNK cuyo número de serie no coincide
  con ningún volumen actual puede apuntar a un medio extraíble, al servidor de un recurso compartido de red o a un volumen formateado
  después.
- El código de arranque es el mismo en los volúmenes formateados por la misma versión de Windows: es idéntico en los
  archivos `$Boot` de Windows 10, 11 y Server 2012 R2 del DFIR Artifact Museum. Un bootkit del registro de arranque
  del volumen lo modifica, o modifica el campo de sectores ocultos, que el código de arranque utiliza para encontrar el resto de
  `$Boot`. Compare `BootCodeSHA256`, `JumpInstruction` y `HiddenSectors` con otros hosts y con la tabla de
  particiones antes de concluir nada: un gestor de arranque o un software de cifrado también pueden modificar el código
  de arranque.
- Una imagen de un volumen cifrado con BitLocker empieza por el propio sector de arranque de BitLocker (OEM ID
  `-FVE-FS-`). Si se proporciona con `-f`, Juntei lo informa, y `Problems` indica que no es NTFS; el sector de arranque
  NTFS está en los datos cifrados. Un `$Boot` recopilado del volumen desbloqueado es el de NTFS.
- `Problems` enumera las incoherencias estructurales y los truncamientos reconocidos, en lugar de ofrecer una prueba completa de si Windows podría montar el volumen. Utilícelo para evaluar sectores recuperados (carving) y copias, no como un hallazgo de ataque por sí mismo. El hash del código de arranque excluye el salto inicial, el bloque de parámetros del BIOS y la firma final; inspeccione también esos campos.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

El resumen enumera el número de serie del volumen de cada sector de arranque (tal como lo muestra Windows y tal como lo muestran los campos de LNK y
Prefetch), el tamaño, el tamaño de clúster y el clúster de la `$MFT`. Cuando se analizan varios hosts, los
agrupa por hash del código de arranque, de modo que destaque un volumen cuyo código de arranque difiera del de los demás.

## Campos de salida CSV { #csv-output-fields }

| Campo | Interpretación |
|---|---|
| `SourcePath` | Archivo de entrada. |
| `VolumeSerialNumber` | Número de serie de 32 bits, tal como lo muestra la salida de LNK, Jump List y Prefetch (`180B1E67`). Windows lo muestra como `180B-1E67`. |
| `VolumeSerialNumber64` | Número de serie completo de 64 bits. |
| `OEM-ID` | `NTFS` en un volumen NTFS. |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | Geometría del volumen. Los valores de sectores por clúster superiores a 128 se registran como potencia de dos y aquí se decodifican. Vacío cuando no es válido. |
| `TotalSectors`, `VolumeSize` | Sectores del volumen y su tamaño en bytes, sin la copia del sector de arranque situada al final. |
| `HiddenSectors` | Sectores del disco anteriores al volumen: dónde empieza su partición. |
| `MFT-Cluster`, `MFT-Offset` | Dónde empieza la `$MFT`, en clústeres y en bytes desde el inicio del volumen. |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | Dónde empieza la `$MFTMirr`. |
| `FileRecordSize`, `IndexRecordSize` | Bytes por registro FILE (normalmente 1024) y por registro INDX (normalmente 4096). |
| `MediaDescriptor` | `F8` para un disco duro. |
| `JumpInstruction` | Los tres primeros bytes (`EB 52 90`): un salto por encima del bloque de parámetros del BIOS hasta el código de arranque. |
| `EndSignature` | `55 AA` al final de un sector de arranque válido. |
| `BootCodeSHA256` | SHA-256 del código de arranque: los bytes 0x54 a 0x1FD del primer sector y el resto de los 8 KiB. |
| `Problems` | Lo que impediría que Windows montara el volumen: un OEM ID distinto de `NTFS`, la ausencia de `55 AA`, tamaños no válidos, campos FAT que NTFS mantiene a cero con valor, la `$MFT` más allá del final del volumen; y un archivo de menos de 8 KiB. |

JSON/JSONL contienen los mismos campos con los mismos nombres, con los números como números, `null` para los valores desconocidos y
`Problems` como lista.

## Estrategia de investigación { #investigation-strategy }

1. Coloque el `VolumeSerialNumber` de cada host junto a los números de serie de volumen de sus archivos LNK, Jump
   Lists y archivos Prefetch: los que no coinciden con ningún volumen recopilado son otros volúmenes (unidades
   USB, recursos compartidos, volúmenes formateados después).
2. Busque los demás números de serie en la salida de [dispositivos USB](usb.md), que enumera los números de serie
   de los volúmenes extraíbles montados.
3. Compare `BootCodeSHA256` y `HiddenSectors` entre hosts. Investigue un código de arranque distinto
   con un desensamblador y la tabla de particiones del disco, y busque entradas de los registros de eventos relacionadas con el arranque.
4. Utilice `MFT-Offset` y `ClusterSize` para encontrar la `$MFT` en una imagen de volumen.

## Más información { #further-details }

- [Microsoft: información general sobre NTFS](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: documentación del formato NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (el encabezado del volumen)
- [The Sleuth Kit: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (su crate `boot`)
