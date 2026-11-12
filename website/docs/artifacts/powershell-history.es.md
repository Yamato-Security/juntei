# Historial de PowerShell { #powershell-history }

## Qué registra el historial { #what-the-history-records }

PSReadLine guarda las líneas de comandos introducidas en un host interactivo de PowerShell. El archivo contiene texto en el orden guardado, sin marcas de tiempo por comando. El historial puede editarse, borrarse, desactivarse o filtrarse y no demuestra que un comando terminara correctamente. Corrobore la ejecución y sus tiempos con los registros de eventos de PowerShell, la telemetría de procesos y otros artefactos de ejecución.

## Ubicación y adquisición { #location-and-collection }

La ubicación predeterminada en Windows para Windows PowerShell 5.1 y PowerShell 7+ es:

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

Otros hosts usan nombres como `Visual Studio Code Host_history.txt` en ese directorio. PowerShell en Linux y macOS suele usar `~/.local/share/powershell/PSReadLine/<Host>_history.txt`. `HistorySavePath` de PSReadLine permite redirigir el archivo; estas rutas predeterminadas son supuestos para la búsqueda, no una lista exhaustiva de ubicaciones.

La entrada de directorio y `auto-parse` seleccionan recursivamente archivos `<Host>_history.txt` directamente dentro de una carpeta `PSReadLine`, sin distinguir mayúsculas y minúsculas ASCII. No seleccionan archivos `*_history.txt` arbitrarios en otros lugares. Use explícitamente `-f` para un archivo renombrado o una copia fuera de un directorio `PSReadLine`. Preserve la estructura original del perfil cuando sea posible para atribuir el usuario.

## Analizar con Juntei { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV es el formato de archivo predeterminado. Con `-t csv`, `-t json` o `-t jsonl` y sin `-o`, los registros se escriben en stdout. Sin ninguna de las dos opciones, solo se muestra un resumen. JSON consta de objetos separados con formato legible; JSONL contiene un objeto por línea. Añada `-C` para sobrescribir una salida existente; los alias del archivo de origen están protegidos contra la sustitución.

`auto-parse` guarda `ShellHistory/PowerShellHistory.csv` (o el formato JSON elegido), también para imágenes VHDX/E01. El historial no aporta eventos a `auto-parse -j` ni a `super-timeline`.

## Codificación y tratamiento de líneas { #encoding-and-line-handling }

- Una marca de orden de bytes UTF-8, UTF-16LE o UTF-16BE determina la codificación y se elimina al principio del archivo. Sin BOM, la entrada siempre se trata como UTF-8. No se intenta adivinar UTF-16 sin BOM; los caracteres NUL se conservan y se señalan como posibles daños de codificación.
- Las secuencias UTF-8 no válidas y los sustitutos UTF-16 sin pareja se convierten en U+FFFD, y el daño se indica en `Problems`. También se notifican un byte UTF-16 sobrante al final y los errores de lectura; las líneas legibles se conservan. Revise `Problems` y el registro del parser antes de confiar en la integridad del resultado.
- Los archivos se procesan en flujo. Se conserva un máximo de 1,048,576 bytes de entrada por línea física, sin contar el BOM ni el delimitador LF (un CR antes de LF sí cuenta). Las líneas más largas se truncan en un límite de carácter completo, se consumen los bytes restantes y `Problems` indica cuántos bytes se omitieron. El límite se aplica a los bytes de entrada codificados, no al número de caracteres.
- Se eliminan los finales de línea LF y CRLF, así como un CR final de una línea sin terminar. Se conservan los demás espacios y tabuladores. Las líneas vacías no generan registros, pero cuentan para los números de línea y terminan una cadena de continuación; las líneas que solo contienen espacios en blanco siguen siendo registros. Los errores de lectura pueden producir una fila de diagnóstico vacía que marca dónde se detuvo la lectura.
- PSReadLine almacena entradas multilínea con un acento grave al final de cada línea física que continúa. Las líneas permanecen separadas, conservando los acentos graves y la sangría. `EntryStartLine` apunta a la primera línea de una cadena consecutiva cuyas líneas anteriores terminan en acento grave. No se interpreta sintaxis del shell ni se reconstruyen comandos. El truncamiento puede afectar a la atribución de continuaciones; revise `Problems` antes de agrupar esas filas.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `SourceFile` | Ruta relativa al directorio de entrada, usando `/`; con `-f`, solo el nombre del archivo. No se exporta ninguna ruta de origen absoluta. |
| `User` | Nombre del perfil inferido de la ruta conservada. La entrada de directorio también reconoce estructuras `Users`/`Documents and Settings` bajo la raíz de entrada; `-f` requiere el sufijo habitual de la ruta de perfil de PSReadLine. Los valores desconocidos están vacíos en CSV y son `null` en JSON. |
| `ShellHost` | Prefijo del nombre de archivo anterior a `_history.txt`, como `ConsoleHost`; desconocido para archivos renombrados sin ese sufijo. |
| `LineNumber` | Número de línea física desde 1, incluidas las líneas vacías omitidas. |
| `EntryStartLine` | Línea física inicial de la cadena de continuación, o el número de esta misma línea. |
| `Command` | Línea física decodificada sin su final de línea, sujeta al límite documentado y a la protección CSV. |
| `FileLastWriteTime` | Fecha de modificación del archivo de origen, repetida en cada fila. **Metadatos del archivo, nunca la hora de ejecución del comando.** El análisis de imágenes usa los metadatos originales, no la hora de staging. |
| `FileSize` | Tamaño del archivo de origen en bytes. |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE` o `UTF-16BE`. |
| `Problems` | Diagnósticos de daños y truncamiento de líneas; separados por punto y coma en CSV y como matriz en JSON/JSONL. Los diagnósticos del archivo también aparecen en el registro. |

`FileLastWriteTime` usa UTC en formato ISO 8601 de forma predeterminada, por ejemplo `2026-10-20T08:46:38.5Z`. Solo `auto-parse` acepta `-Z, --timezone` para cambiar el desfase mostrado. El comando independiente `parse-file powershell-history` no tiene opción de zona horaria.

Se comprueba cada celda de texto CSV en busca de prefijos de fórmulas de hojas de cálculo (`=`, `+`, `-`, `@`, incluidas las variantes de ancho completo y los prefijos tras espacios en blanco o ciertos caracteres invisibles/de control). Las celdas afectadas y las que empiezan por caracteres de control reciben un apóstrofo inicial. Esto puede modificar los comandos y las rutas en CSV. JSON y JSONL conservan el texto decodificado sin el apóstrofo CSV; las sustituciones de decodificación y el truncamiento siguen aplicándose. Use JSONL para comparar el texto guardado de los comandos y conserve siempre los archivos de historial originales.
