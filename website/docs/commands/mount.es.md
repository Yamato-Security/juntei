# Montar imágenes de disco { #mounting-disk-images }

`juntei mount` permite leer los archivos de una imagen de disco, VHDX o E01, desde un directorio, en modo de solo lectura, mientras se ejecuta. Herramientas como Hayabusa y Juntei pueden analizarlos sin extraer archivos ni requerir permisos de root. Utiliza FUSE y funciona en Linux. El lector de VHDX y la biblioteca de montaje se mantienen en [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser), y el lector de E01, en [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser).

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

Una imagen E01 (el formato Expert Witness de EnCase, también `.Ex01`) se monta de la misma manera. Una imagen dividida se indica por su primer segmento, y Juntei encuentra los demás (`host1.E02`, `host1.E03`, …) junto a él:

```bash
juntei mount host1.E01 /mnt/host1
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
- Una imagen de un único volumen, sin tabla de particiones, corresponde a ese volumen.
- Los nombres se enumeran tal como los almacena NTFS y se buscan de forma exacta. No se muestran los nombres
  DOS 8.3 ni los flujos de datos alternativos.
- Los archivos comprimidos se descomprimen (hasta 1 GiB cada uno). Los archivos cifrados con EFS no se pueden
  leer.

## Las evidencias no se modifican { #the-evidence-is-not-changed }

La imagen se abre en modo de solo lectura y el montaje rechaza escrituras. Si un VHDX no se cerró correctamente, su registro pendiente se aplica en memoria, sin escribirlo en la imagen. Algunos componentes DiscUtils dejan a cero la instrucción de salto y la firma `55 AA` del sector de arranque NTFS; Juntei lee estos volúmenes tal como están.

Los segmentos de una imagen E01 también se abren en modo de solo lectura. Cada bloque se comprueba con su suma de comprobación al leerse: un bloque dañado no se puede leer, pero el resto de la imagen sí. Los sectores que la adquisición no pudo leer y una adquisición que no llegó a terminar se notifican como advertencias. Un archivo de evidencia lógica (`.L01`, `.Lx01`) contiene archivos, no un disco, y no se monta.

## Uso { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Analizar todas las imágenes con auto-parse { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` analiza los sistemas Windows contenidos en cada imagen de disco VHDX del directorio de
evidencias (o en la única imagen indicada con `-d`), y `auto-parse --e01` los de cada imagen E01, sin un
montaje aparte. Las dos opciones pueden indicarse juntas:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d /cases/e01 --e01 -o parsed
```

- Cada imagen se monta en modo de solo lectura en un nuevo directorio temporal, en una carpeta con el nombre de
  la imagen (`host1-triage`), y los sistemas que se encuentran allí se analizan como en una recopilación en
  disco. Las imágenes se desmontan al final y también al pulsar Ctrl+C.
- La terminal enumera las imágenes y la imagen de cada sistema:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- `--e01` toma el primer segmento de cada imagen E01 (`*.E01`, `*.Ex01`); los demás segmentos se leen
  con él.
- En el manifiesto, `EvidenceDirectory` es el directorio indicado y el campo `Image` de cada sistema nombra su
  imagen. `EvidenceRoot` y los comandos de los analizadores utilizan la carpeta de montaje
  (`./host1-triage/C`).
- Si la imagen contiene archivos de Volume Shadow Copies en carpetas como `VSS32`, cada carpeta se detecta y analiza como otro sistema (`HOST1_2`, …).
- Si una imagen no se puede montar, se informa de ello; las demás se siguen analizando y la ejecución termina
  con el código de salida 1.

Si no se puede abrir un volumen NTFS, Juntei informa del problema y conserva los volúmenes legibles. Con imágenes parcialmente legibles, `auto-parse --vhdx` y `--e01` devuelven el código 1; `mount` también devuelve 1 después de desmontar.

## Requisitos { #requirements }

- Linux: FUSE 3 (`fusermount3`, el paquete `fuse3`) y acceso a `/dev/fuse`, que los usuarios normales suelen
  tener. No se necesita root.
- Windows y macOS no son compatibles. En Windows, conecte en su lugar una imagen VHDX en modo de solo lectura
  desde un PowerShell con privilegios elevados e indique a las herramientas su letra de unidad. Windows no
  puede conectar una imagen E01: móntela en modo de solo lectura con una herramienta que sí pueda hacerlo e
  indique esa unidad a las herramientas.

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
