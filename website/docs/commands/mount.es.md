# Montar imágenes de disco { #mounting-disk-images }

`juntei mount` permite leer los archivos de una imagen de disco VHDX / E01 / Ex01 desde un directorio, en modo de solo lectura, mientras se ejecuta. Herramientas como Hayabusa y Juntei pueden analizarlos sin extraer archivos ni requerir permisos de root. Utiliza FUSE y funciona en Linux. El lector y la biblioteca de montaje se mantienen en [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

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
Version: 1.1.0 (CODE BLUE Release)
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

## Análisis nativo de imágenes { #parsing-every-image-with-auto-parse }

Con `--vhdx`, `auto-parse` lee VHDX de forma nativa en **Linux, macOS y Windows** mediante [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser). No requiere montaje del sistema, FUSE ni privilegios de administrador.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Los parsers necesitan rutas: Juntei copia los archivos que leen los parsers a un espacio temporal privado con búferes limitados: hives del registro y sus registros de transacciones, archivos de metadatos NTFS, bases de datos, archivos de prefetch, accesos directos, Jump Lists, archivos de Outlook y carpetas de cuarentena de Defender. Cualquier otro archivo queda como un marcador vacío con su fecha de modificación original, de modo que directorios y rutas se ven como en el disco. persistence-hunter lee el contenido de un marcador desde la imagen solo cuando lo necesita, por ejemplo un ejecutable al que apunta una entrada de persistencia. **Reserve espacio para los archivos copiados**, además de los resultados. Las regiones nulas se escriben como huecos cuando se admite. Se conservan nombres, estructura, enlaces físicos y fechas de modificación. Las fechas de origen de LNK y Jump Lists provienen de los metadatos NTFS originales, nunca de las copias temporales. La finalización normal, incluso con errores de parser, elimina las copias; una terminación forzada puede dejarlas. La fuente es solo lectura y los registros VHDX pendientes se aplican únicamente en memoria.

La detección de hosts, CSV/JSON/JSONL y `-j` usan el flujo habitual. `Image` en `manifest.json` identifica la imagen original; `EvidenceRoot`, la ruta relativa interna. Los errores de lectura y nombres inseguros se registran en `ImageErrors` y filas fallidas de `manifest.csv`. Se conservan los resultados legibles; una entrada incompleta devuelve 1. No se renombran nombres peligrosos o ambiguos. Los flujos NTFS con nombre no se extraen como archivos adicionales; los artefactos recogidos como archivos normales siguen disponibles.

El [`mount` independiente](mount.md) expone un sistema de archivos a otros programas y sigue necesitando Linux FUSE. `auto-parse --vhdx` no lo utiliza.

La preparación usa de forma predeterminada un directorio privado en el sistema de archivos de salida. Use `--temp-dir` para elegir un directorio existente en otro disco. Los metadatos originales se guardan aparte en el volumen de salida, por lo que un disco de preparación lleno no impide analizar los archivos ya legibles. Reserve espacio para copias y resultados; si el volumen de salida está lleno, no se pueden guardar resultados. Ctrl+C detiene los analizadores, limpia el espacio privado y sale con código 130. SIGKILL o un corte de energía pueden dejar copias. Se omiten los subdirectorios inaccesibles al buscar imágenes; una raíz de entrada ilegible sigue siendo un error.

El directorio de preparación debe estar fuera del directorio de salida. En Linux y macOS, SIGTERM y SIGHUP también detienen los procesos de análisis y limpian el espacio de trabajo.

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## Imágenes de disco E01 y Ex01 { #e01-and-ex01-disk-images }

`--e01` lee imágenes físicas EWF1 (`.E01`) y EWF2 (`.Ex01`) de forma nativa en **Linux, macOS y Windows**, sin FUSE, montaje del sistema ni privilegios de administrador. Utiliza [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) y el mismo procesamiento NTFS descrito arriba.

Indique el primer segmento de una imagen dividida. Mantenga todos los archivos asociados, incluido `.E02`, junto a él con sus nombres originales. Se admiten discos completos (MBR/GPT) y volúmenes NTFS individuales. Use `--vhdx --e01` para directorios mixtos.

Se aplican los mismos requisitos de espacio temporal y metadatos de origen. Las imágenes E01 suelen contener discos completos y todas las imágenes se preparan antes de empezar el análisis, pero solo se copian los archivos que leen los parsers, normalmente una pequeña parte del espacio usado. Las imágenes son de solo lectura. Los huecos de adquisición y los daños detectados al abrir o leer se indican en `ImageErrors` y `manifest.csv`; se conservan los resultados legibles y se devuelve el código 1. Se comprueban los datos consultados, no todos los bloques ni el hash del medio completo. Se rechazan los archivos lógicos `.L01` y `.Lx01`; no hay opción de contraseña.

El comando independiente `mount` sigue necesitando Linux FUSE; el análisis nativo automático no.

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## Requisitos del montaje visible para el sistema { #os-visible-mount-requirements }

`juntei mount` necesita Linux FUSE 3 (`fusermount3`) y acceso a `/dev/fuse`. En Windows y macOS, use el análisis nativo anterior.
