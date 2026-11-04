# Listas de elementos usados recientemente (RunMRU, TypedPaths, WordWheelQuery, TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## Qué son estas listas { #what-the-lists-are }

Windows y sus programas mantienen listas breves de lo último que un usuario escribió o abrió, en el
`NTUSER.DAT` del usuario:

- **RunMRU:** comandos escritos en el cuadro de diálogo Ejecutar (Win+R): `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`.
- **TypedPaths:** rutas escritas en la barra de direcciones del Explorador.
- **WordWheelQuery:** términos buscados en el cuadro de búsqueda del Explorador.
- **TypedURLs:** direcciones escritas en la barra de direcciones de Internet Explorer y, desde Windows 8,
  `TypedURLsTime`, que indica cuándo se escribió cada una.
- **Paint y WordPad:** los últimos archivos que abrió cada uno (`Recent File List`).
- **ACMru (Windows XP):** los términos del Asistente para búsqueda, por tipo de búsqueda.

<span class="evidence-strength">Mejor conclusión:</span> el usuario escribió este comando, ruta,
término de búsqueda o dirección, o el programa abrió este archivo. Solo la entrada más reciente de cada lista
tiene fecha, dada por la hora de última escritura de su clave, salvo las direcciones escritas, cuya fecha proporciona `TypedURLsTime`.

## Ubicación { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

Las subclaves de `ACMru` son los tipos de búsqueda: `5603` nombres de archivos y carpetas, `5604` palabras o frases en
archivos, `5647` equipos y personas, `5001` Internet.

Recopile `NTUSER.DAT` junto con sus registros de transacciones para cada perfil.

## Precauciones para el analista { #analyst-cautions }

- **Una hora por lista.** La hora de última escritura de una clave cambia cuando cambia su lista, por lo que solo fecha la
  entrada más reciente (`IsMostRecent`, `LastUsedUTC`). Las demás están ordenadas, pero sin fecha.
  `TypedURLsTime` fecha cada dirección escrita (`TypedTimeUTC`).
- **Escrito, no ejecutado.** RunMRU registra lo que se escribió en el cuadro de diálogo Ejecutar, tanto si se ejecutó
  correctamente como si no. Windows almacena cada comando con un `\1` final; Juntei lo elimina. Correlacione
  con Prefetch, BAM, UserAssist y los eventos de creación de procesos.
- **Las listas son breves.** Cada una conserva solo sus entradas más recientes, y la configuración de privacidad y las herramientas
  de limpieza las borran. Utilice `parse-registry read --recover-deleted` para los valores eliminados.
- **Los términos son lo que se escribió.** WordWheelQuery y ACMru conservan el texto escrito, no lo que se encontró.
- **El orden lo da la lista.** `MRUList` de RunMRU y `MRUListEx` de WordWheelQuery indican el
  orden; en las demás, lo indica el número del valor (`url1`, `File1` y `000` son los más recientes).

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

El resumen muestra las entradas de cada lista y las entradas con fecha más recientes (las direcciones escritas y la
entrada más reciente de cada lista), empezando por la más nueva.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | El host, el propietario del perfil (obtenido de la ruta o de las carpetas del shell), el SID procedente de `ProfileList` del sistema y el hive. |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad` o `ACMru`. |
| `Subkey` | La subclave de `ACMru`: el tipo de búsqueda (`5603`, `5604`, `5647`, `5001`). |
| `MRU-Position`, `IsMostRecent` | La posición en la lista (0 es la más reciente); vacío cuando la lista no nombra el valor. |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | El comando (sin su `\1`), la ruta, el término de búsqueda, la dirección o el archivo. |
| `TypedTimeUTC` | Cuándo se escribió la dirección, según `TypedURLsTime`. |
| `LastUsedUTC` | La hora de última escritura de la clave, solo para la entrada más reciente. |
| `KeyPath`, `KeyLastWrittenUTC` | La clave y su última escritura: la hora de su entrada más reciente, no la de cada fila. |
| `DataRaw` | Los bytes del valor (en hexadecimal) cuando no se pueden decodificar. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperación del Registro. |
| `Problems` | Valores que la lista nombra pero que la clave no contiene, valores que la lista no nombra, valores con nombres incorrectos, tipos erróneos, términos truncados y horas que no son FILETIME. |

## Estrategia de investigación { #investigation-strategy }

1. Lea RunMRU para encontrar comandos, scripts, recursos compartidos de red y herramientas de administración escritos por el usuario.
2. Lea TypedPaths y WordWheelQuery para encontrar las carpetas y los archivos que buscó el usuario:
   credenciales, exportaciones, archivos comprimidos, ubicaciones extraíbles y de red.
3. Sitúe las direcciones escritas en la línea de tiempo con `TypedTimeUTC`.
4. Correlacione con Prefetch, BAM, UserAssist, ShellBags, archivos LNK, Jump Lists y el historial del navegador.

## Más información { #further-details }

- [libyal winreg-kb: claves de elementos usados recientemente (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
