# Caché de mapas de bits de RDP { #rdp-bitmap-cache }

## Qué registra la caché de mapas de bits de RDP { #what-the-rdp-bitmap-cache-records }

Cuando el cliente de Escritorio remoto (`mstsc.exe`) se conecta con la caché persistente de mapas de bits activada, que
es la opción predeterminada, guarda en el disco partes de la pantalla remota para que las sesiones posteriores no tengan
que volver a recibirlas. Cada parte es una tesela de 64 × 64 píxeles como máximo. En conjunto, las teselas son pequeños
fragmentos de lo que el usuario vio en los equipos remotos: ventanas, texto, nombres de archivo, comandos y escritorios.

| Cliente | Archivos de caché | Teselas |
|---|---|---|
| Windows 7 y posteriores | `Cache0000.bin`, `Cache0001.bin`, …, de hasta 100 MB cada uno | De 32 bits, una tras otra |
| Windows XP, Server 2003 y anteriores | `bcache2.bmc` (8 bits por píxel), `bcache22.bmc` (16), `bcache23.bmc` (24) y `bcache24.bmc` (32), de hasta 10 MB cada uno | Una por cada ranura de 64 × 64; algunas comprimidas |

Windows 10 y posteriores también dejan un `bcache24.bmc` vacío junto a los archivos `.bin`.

<span class="evidence-strength">Mejor conclusión:</span> el usuario en cuyo perfil está la caché se conectó con el
cliente de Escritorio remoto a un equipo que mostraba el contenido almacenado en la caché. La caché no indica qué equipo
era ni cuándo.

## Ubicaciones y recopilación { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

El valor `BitmapPersistCacheLocation` de `HKCU\Software\Microsoft\Terminal Server Client` puede trasladar la caché a
otra ubicación. Recopile la carpeta `Cache` completa de cada perfil, también desde las instantáneas de volumen: una copia
más antigua de un archivo de caché puede contener teselas que el archivo actual ya no tiene.

## Precauciones para el analista { #analyst-cautions }

- Las teselas no tienen horas, y la caché no nombra los equipos de los que proceden. Relaciónelas con conexiones RDP
  mediante otras evidencias: las claves de registro `Terminal Server Client` del cliente
  (`parse-registry remote-access`), `Default.rdp` y otros archivos `.rdp`, las Jump Lists de `mstsc.exe`, las horas
  de los archivos de caché en `$MFT` y los eventos de inicio de sesión de los equipos remotos.
- Un archivo de caché mezcla teselas de muchas sesiones y de varios equipos remotos. El cliente sustituye las teselas
  antiguas a medida que se llena un archivo, por lo que una tesela puede tener días o meses de antigüedad.
- Las teselas son fragmentos de pantallas, no pantallas completas. Las teselas que estaban juntas en la pantalla suelen
  estar juntas también en el archivo, pero no siempre. Reconstruya las pantallas con
  [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher) o a simple vista a partir de los collages.
- La mayoría de las teselas miden 64 × 64 píxeles. Las teselas de los bordes derecho e inferior de una pantalla son más
  estrechas o más bajas, según la resolución: por ejemplo, de 58 o 62 píxeles de ancho, o de 28 o 35 de alto. Juntei lee
  cada tesela con su propio tamaño. Las herramientas que leen todas las teselas como si tuvieran 64 píxeles de ancho
  muestran deformadas las estrechas.
- La mayoría de las teselas de un solo color (`SingleColor`) son fondo. Fíltrelas para encontrar contenido.
- En los archivos `.bmc`, las teselas de 8 bits son colores de la paleta de la sesión, que la caché no guarda. Juntei
  usa la paleta predeterminada de Windows, por lo que sus colores pueden ser incorrectos. Las teselas comprimidas usan
  el RLE entrelazado del protocolo de Escritorio remoto (MS-RDPBCGR 3.1.9), que Juntei descomprime tal como lo define
  la especificación.
- Una tesela `.bmc` más pequeña que su ranura de 64 × 64 deja detrás el final de una tesela anterior. Juntei guarda esas
  filas sobrantes como una imagen aparte (`RemnantImage`). Pertenecen a una tesela más antigua, no a la tesela de la
  ranura.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```

```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```

Con `-d`, Juntei lee los archivos cuyo nombre se compone de `Cache`, dígitos y `.bin`, o de `bcache`, dígitos y
`.bmc`. Un archivo `Cache????.bin` que no empieza por la firma `RDP8bmp` pertenece a otro programa: se cuenta en el
resumen y se omite. Con `-f`, Juntei lee el archivo sea cual sea su nombre.

### Imágenes { #images }

`--extract` crea una carpeta para cada archivo de caché, `<DIR>/<user>/<cache file>/`. Cuando dos archivos de caché
compartirían la misma carpeta, como un archivo de caché y su copia procedente de una instantánea de volumen, la segunda
carpeta se numera (`Cache0001.bin_2`). La carpeta contiene:

- `<cache file>_<index>.bmp`: cada tesela, con su propio tamaño. Los nombres son los que usa bmc-tools, de modo que
  RdpCacheStitcher puede abrir la carpeta.
- `<cache file>_old_<index>.bmp`: solo en `.bmc`, el final de una tesela anterior que quedó detrás de una más pequeña.
- `<cache file>_collage.png`: todas las teselas en el orden del archivo, 64 por fila (`--collage-width`), cada una en
  una celda de 64 × 64 a partir de su esquina superior izquierda. La tesela con índice *i* está en la fila *i* / 64 y en
  la columna *i* % 64, contando desde 0. El magenta rellena las partes vacías de las celdas: junto a una tesela
  estrecha, debajo de una baja y donde no se pudo leer una tesela.

`--collage-only` guarda solo los collages. Una imagen que ya existe no se sustituye sin `-C`, y el campo `Problems` de
su tesela lo indica. Los archivos de caché en sí nunca se sustituyen, ni siquiera con `-C`.

## Campos de salida CSV { #csv-output-fields }

| Campo | Significado |
|---|---|
| `SourceFile` | El archivo de caché. |
| `User` | El usuario en cuya carpeta de perfil está el archivo de caché, según su ruta. |
| `TileIndex` | El número de la tesela en el archivo, contando desde 0, como en el nombre de su imagen. |
| `Width`, `Height` | El tamaño de la tesela en píxeles. |
| `SingleColor` | `true` cuando todos los píxeles de la tesela tienen el mismo color. |
| `TileImage` | Con `--extract`: el archivo BMP de la tesela. |
| `Collage` | Con `--extract`: el collage del archivo de caché. |
| `RemnantImage` | `.bmc` con `--extract`: el final de una tesela anterior que quedó en la ranura. |
| `Key` | La clave de 64 bits de la tesela en el protocolo de Escritorio remoto, en hexadecimal. |
| `Format`, `FormatVersion` | `bin` con la versión que sigue a su firma (6 en Windows 10 y posteriores), o `bmc`. |
| `BitsPerPixel` | 32 en `.bin`; 8, 16, 24 o 32 en `.bmc`. |
| `Compressed`, `Flags` | `.bmc`: si la tesela estaba comprimida y los indicadores de su encabezado. |
| `Offset` | Dónde empieza el encabezado de la tesela en el archivo. |
| `Problems` | Daños: un archivo que termina en medio de una tesela, un encabezado de tesela que ninguna tesela podría tener, datos que no encajan con el tamaño de la tesela, datos comprimidos que no se pueden descomprimir, una imagen que no se pudo guardar. |

JSON y JSONL tienen los mismos campos, con los números y los booleanos tipados, `null` para los valores que una tesela
no tiene y `Problems` como lista. Los daños que no afectan a ninguna tesela concreta, como datos posteriores a la última
tesela, se notifican en los mensajes de la ejecución.

## Análisis automático { #automatic-parsing }

`auto-parse` escribe `LateralMovement/RdpCache.csv` (`RdpCache.json` o `RdpCache.jsonl` con
`-t json` o `-t jsonl`), junto con `LateralMovement/RdpCache-logs.txt`, y guarda el collage de cada archivo de caché
en una carpeta `LateralMovement/RdpCache/`. Añada `--extract-rdp-tiles` para guardar también cada tesela, lo que puede
suponer decenas de miles de archivos para un solo usuario. Desde imágenes de disco se copian los archivos de caché. La
línea de tiempo combinada `-j` incluye todas las filas, con el identificador de `Artifact` `rdp-cache`. Las teselas no
tienen horas, por lo que [`super-timeline`](../commands/super-timeline.md) no genera eventos a partir de ellas.

## Estrategia de investigación { #investigation-strategy }

1. Abra los collages y busque texto: nombres de equipos y de usuarios, rutas de archivo, comandos, ventanas de consola,
   barras de direcciones de navegadores y cuadros de diálogo.
2. Reconstruya las pantallas con RdpCacheStitcher a partir de una carpeta de teselas guardadas con `--extract`.
3. Averigüe a qué equipos se conectó el usuario a partir del registro del cliente (`parse-registry remote-access`), los
   archivos `.rdp` y las Jump Lists de `mstsc.exe`. En esos equipos, busque los inicios de sesión RDP correspondientes y
   las entradas del registro de acceso de usuarios.
4. Use las horas de `$MFT` de los archivos de caché, y las de sus copias en instantáneas de volumen, para acotar cuándo
   se escribieron las teselas.

## Más información { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: la caché de mapas de bits de RDP (en francés)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — plugin de caché de RDP](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — caché persistente de mapas de bits](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
