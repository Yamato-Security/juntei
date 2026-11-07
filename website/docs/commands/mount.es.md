# Montar imágenes de disco { #mounting-disk-images }

`juntei mount` permite leer los archivos de una imagen de disco VHDX desde un directorio, en modo de solo lectura, mientras se ejecuta. Herramientas como Hayabusa y Juntei pueden analizarlos sin extraer archivos ni requerir permisos de root. Utiliza FUSE y funciona en Linux. El lector y la biblioteca de montaje se mantienen en [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

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

- Los archivos del volumen NTFS aparecen en el punto de montaje. Una imagen recopilada puede contener directorios como `C` y `LongFileNames`, junto con metadatos del volumen como `$MFT`. Varios volúmenes NTFS aparecen como `Volume1`, `Volume2`, etc.
- Todos los archivos son de solo lectura y pertenecen al propietario del punto de montaje. Las marcas de tiempo
  de los archivos son las de `$STANDARD_INFORMATION` de NTFS.
- Los nombres se enumeran tal como los almacena NTFS y se buscan de forma exacta. No se muestran los nombres
  DOS 8.3 ni los flujos de datos alternativos.
- Los archivos comprimidos se descomprimen (hasta 1 GiB cada uno). Los archivos cifrados con EFS no se pueden
  leer.

## Las evidencias no se modifican { #the-evidence-is-not-changed }

La imagen se abre en modo de solo lectura y el montaje rechaza escrituras. Si un VHDX no se cerró correctamente, su registro pendiente se aplica en memoria, sin escribirlo en la imagen. Algunos componentes DiscUtils dejan a cero la instrucción de salto y la firma `55 AA` del sector de arranque NTFS; Juntei lee estos volúmenes tal como están.

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
- Si la imagen contiene archivos de Volume Shadow Copies en carpetas como `VSS32`, cada carpeta se detecta y analiza como otro sistema (`HOST1_2`, …).
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
