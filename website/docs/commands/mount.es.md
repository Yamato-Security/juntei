# Montar imágenes de disco { #mounting-disk-images }

`juntei mount` hace que los archivos de una imagen de disco VHDX se puedan leer en un directorio, en modo de
solo lectura, mientras se esté ejecutando. Así, las recopilaciones `--vhdx` de KAPE y los discos de Hyper-V
pueden examinarse con cualquier herramienta, como Hayabusa o los propios comandos de Juntei, sin extraer nada,
sin root y sin que el sistema operativo admita VHDX ni NTFS. Utiliza FUSE y funciona en Linux. El lector y la biblioteca de montaje se mantienen en [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

En otra terminal:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

Pulse Ctrl+C en la primera terminal para desmontar, o ejecute `fusermount3 -u /mnt/host1`.

## Qué muestra el montaje { #what-the-mount-shows }

- Los archivos del volumen NTFS en el punto de montaje: en una recopilación de KAPE, la carpeta `C` que contiene
  los archivos recopilados, la carpeta `LongFileNames` de KAPE y los propios archivos de metadatos del volumen,
  como la `$MFT`. Una imagen con varios volúmenes NTFS los muestra como `Volume1`, `Volume2`, etc.
- Todos los archivos son de solo lectura y pertenecen al propietario del punto de montaje. Las marcas de tiempo
  de los archivos son las de `$STANDARD_INFORMATION` de NTFS.
- Los nombres se enumeran tal como los almacena NTFS y se buscan de forma exacta. No se muestran los nombres
  DOS 8.3 ni los flujos de datos alternativos.
- Los archivos comprimidos se descomprimen (hasta 1 GiB cada uno). Los archivos cifrados con EFS no se pueden
  leer.

## Las evidencias no se modifican { #the-evidence-is-not-changed }

La imagen se abre en modo de solo lectura y el montaje rechaza las escrituras. Un VHDX que no se cerró
correctamente se lee aplicando en memoria las entradas pendientes de su registro, sin volver a escribir nunca
nada en la imagen. KAPE escribe sus archivos VHDX con DiscUtils, que deja el sector de arranque NTFS sin su
instrucción de salto ni su firma `55 AA`; Juntei lee estos volúmenes tal como están.

## Uso { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Analizar todas las imágenes con auto-parse { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` analiza los sistemas Windows contenidos en cada imagen de disco VHDX del directorio de
evidencias (o en la única imagen indicada con `-d`), sin un montaje aparte:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- Cada imagen se monta en modo de solo lectura en un nuevo directorio temporal, en una carpeta con el nombre de
  la imagen (`host1-triage`), y los sistemas que se encuentran allí se analizan como en una recopilación en
  disco. Las imágenes se desmontan al final y también al pulsar Ctrl+C.
- La terminal enumera las imágenes y la imagen de cada sistema:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- En el manifiesto, `EvidenceDirectory` es el directorio indicado y el campo `Image` de cada sistema nombra su
  imagen. `EvidenceRoot` y los comandos de los analizadores utilizan la carpeta de montaje
  (`./host1-triage/C`).
- KAPE guarda los archivos que copia de las instantáneas de volumen (Volume Shadow Copies) en carpetas como
  `VSS32`. Cada una de ellas se detecta y se analiza como un sistema más (`HOST1_2`, …).
- Si una imagen no se puede montar, se informa de ello; las demás se siguen analizando y la ejecución termina
  con el código de salida 1.

Si no se puede abrir un volumen NTFS, Juntei informa del problema y conserva los volúmenes legibles. Con imágenes parcialmente legibles, `auto-parse --vhdx` devuelve el código 1; `mount` también devuelve 1 después de desmontar.

## Requisitos { #requirements }

- Linux: FUSE 3 (`fusermount3`, el paquete `fuse3`) y acceso a `/dev/fuse`, que los usuarios normales suelen
  tener. No se necesita root.
- Windows y macOS no son compatibles. En Windows, conecte en su lugar la imagen en modo de solo lectura desde
  un PowerShell con privilegios elevados e indique a las herramientas su letra de unidad:

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
