# RecentDocs { #recentdocs }

## Qué es RecentDocs { #what-recentdocs-is }

El Explorador de Windows almacena listas de archivos y carpetas usados más recientemente en el `NTUSER.DAT` de cada usuario.
La lista raíz abarca los elementos recientes de todos los tipos; las subclaves agrupan extensiones como `.docx`, además de una
lista `Folder`. `MRUListEx` especifica el orden de los valores numerados.

RecentDocs resulta útil para identificar documentos con los que interactuó un usuario, incluidos archivos que
posteriormente se eliminaron o movieron. Los valores también pueden contener el nombre del acceso directo que Windows utilizó en la carpeta
Recent del usuario, que puede correlacionarse con las evidencias LNK.

<span class="evidence-strength">Mejor conclusión:</span> el Explorador registró el elemento en la lista
RecentDocs de ese usuario. La hora de la clave fecha únicamente el elemento más reciente de esa lista concreta.

## Ubicación { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

Recopile `NTUSER.DAT` y sus registros de transacciones para cada perfil relevante, además del directorio
de archivos LNK Recent del usuario.

## Precauciones para el analista { #analyst-cautions }

- `MRUListEx` proporciona el orden, no una marca de tiempo para cada elemento.
- La hora de última escritura de la clave corresponde a la actualización más reciente; solo la posición 0 de la MRU obtiene un
  `LastOpenedUTC` defendible derivado de ella.
- Un elemento puede aparecer en la lista raíz y en una lista de extensión. No lo cuente dos veces como dos aperturas.
- El comportamiento de las aplicaciones y del Explorador puede variar; algunos métodos de acceso pueden no actualizar RecentDocs.
- Las herramientas de limpieza o privacidad y los límites de tamaño de la lista eliminan las evidencias más antiguas.
- Un nombre de archivo puede estar controlado por el atacante y no prueba el tipo de contenido.
- La inferencia del usuario procede de las rutas del perfil o de los datos de las carpetas del shell; revise los nombres de usuario vacíos o ambiguos.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host` | Host derivado del contexto de la recopilación o del sistema, cuando está disponible. |
| `User` | Propietario del perfil inferido a partir de la ruta o de las rutas de las carpetas del shell; mantenga la incertidumbre cuando esté vacío. |
| `SourcePath` | `NTUSER.DAT` de entrada. Imprescindible al combinar usuarios. |
| `Extension` | Sublista (`.docx`, `Folder`, etc.); nulo para la lista raíz de RecentDocs. |
| `MRU-Position` | Cero es el más reciente dentro de esta lista. Es un orden relativo, no un tiempo transcurrido. |
| `ValueIndex` | Valor numerado del Registro al que hace referencia `MRUListEx`. Solo es un pivote estructural. |
| `Filename` | Nombre del elemento decodificado. Correlacione la ruta completa mediante LNK, Jump List, MFT o datos de aplicaciones. |
| `LinkName` | Nombre del acceso directo codificado en el elemento del shell; a menudo se encuentra en el directorio Recent del usuario. |
| `KeyLastWrittenUTC` | Hora en que cambió por última vez la clave de la lista. Se aplica a la lista, no a cada fila. |
| `LastOpenedUTC` | Solo se rellena para la posición 0 de la MRU, donde la hora de la clave respalda esa interpretación. |
| `ValueDataRaw` | Bytes sin procesar cuando la decodificación falla o es necesario validar la evidencia. |
| `TransactionLogStatus`, `LogsApplied` | Si se reprodujeron registros de transacciones del Registro y cuáles. |
| `Problems` | Valores MRU ausentes, valores no incluidos en la lista, tipos erróneos o daños en la decodificación. |

## Estrategia de investigación { #investigation-strategy }

1. Separe las listas raíz y de extensión de cada usuario; elimine con cuidado los duplicados del mismo elemento.
2. Utilice la posición en la MRU para ordenar y solo `LastOpenedUTC` para una afirmación con marca de tiempo.
3. Empareje `LinkName`/`Filename` con los datos de LNK de Recent, Jump List, `$MFT` y USN.
4. Revise los nombres señuelo, las extensiones dobles, los destinos de red o extraíbles y los documentos cercanos al acceso inicial.
5. Correlacione con Prefetch/SRUM de las aplicaciones y los registros de eventos cuando la ejecución sea relevante.

## Más información { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libfwsi de libyal](https://github.com/libyal/libfwsi)
- [Póster de SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/)
