# Análisis YARA { #yara-scan }

Analiza archivos recopilados con un motor Rust portátil. Compatible con Windows, macOS y Linux. Los archivos se leen como datos y nunca se ejecutan. Este comando es independiente del análisis automático de artefactos.

## Elegir reglas { #choose-rules }

Core es la opción predeterminada y prioriza precisión y rendimiento. Extended amplía la búsqueda. Full ofrece la mayor cobertura, pero puede generar más falsos positivos.

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
```

`--rules <FILE_OR_DIRECTORY>`: Usa un archivo o directorio local de reglas UTF-8, sin descargar nada. Los directorios se recorren recursivamente. Las inclusiones permanecen en el directorio seleccionado y se expanden una vez, en orden de dependencia. Las reglas inválidas o no compatibles detienen la compilación en lugar de descartarse silenciosamente.

## Límites y alcance { #limits-and-scope }

Se admiten todos los archivos normales, incluidos los recuperados de cuarentena. Se omiten enlaces simbólicos y archivos especiales. No se descomprimen archivos ni imágenes de disco ni se analiza memoria en vivo. Usa archivos extraídos estables y un directorio de salida nuevo fuera de las pruebas.

- `-T, --threads <N>`: Análisis paralelos; por defecto, número de CPU limitado a 8.
- `--timeout <SECONDS>`: Tiempo máximo de evaluación por archivo en segundos; predeterminado: 60.
- `--max-file-mib <MIB>`: Tamaño máximo en MiB; predeterminado: 64. Los archivos mayores se registran como omitidos. Más trabajadores y búferes mayores aumentan el consumo de memoria.
- `--rules-release YYYYMMDD`: Fecha de la versión Forge o latest por defecto. Se guardan las fuentes descargadas y sus hashes para reproducir el análisis.

## Resultados { #results }

- `YaraDetections.csv` / `YaraDetections.jsonl`: Coincidencias de regla y archivo, con rutas relativas, nombres de los equipos de origen, hashes, metadatos y posiciones limitadas. No se exportan los bytes coincidentes.
- `YaraFiles.csv` / `YaraFiles.jsonl`: Cobertura por archivo, incluidas omisiones, errores de lectura, tiempos agotados y resultados parciales.
- `Summary.json`: Estado final, opciones, cantidades, versión del motor y hashes de las fuentes.
- `Rules.yar`: Reglas descargadas exactas o una instantánea de las reglas locales con inclusiones resueltas.
- `CompilationWarnings.txt` / `CompilationError.txt`: Advertencias de compilación o error si falla la compilación.

Códigos de salida: 0 indica finalización, 3 cobertura incompleta, 2 fallo de configuración/salida y 130 cancelación. Una coincidencia es una pista, no prueba de compromiso; ninguna coincidencia no garantiza seguridad. Revisa el título de la regla y el contexto del artefacto. Descifra la cuarentena antes de identificar su contenido.

## Coincidencias MFT y recuperación { #mft-recovery }

Las MFT se analizan una a una como bytes sin procesar, con un límite propio (512 MiB por defecto). Se interpretan los registros en las posiciones coincidentes y se vuelven a analizar sus flujos DATA residentes por separado. Solo se recuperan los flujos que coinciden por sí mismos. Las coincidencias de metadatos, entre registros o no residentes se informan sin exportar contenido.

`--max-mft-mib <MIB>` · `--max-mft-recoveries <N>`

La recuperación se limita por defecto a 1000 flujos por ejecución. Se indican los límites, registros dañados y tiempos agotados. Los nombres guardados llevan una extensión de cuarentena; no se recrean los directorios de las pruebas.

`YaraMFTResults.csv` / `YaraMFTResults.jsonl` · `RecoveredMFT/` · `.quarantined`

## Más detalles { #further-details }

- [Motor: Boreal](https://github.com/vthib/boreal)
- [Paquetes de reglas: YARA Forge](https://github.com/YARAHQ/yara-forge)
- [Uso detallado y referencia de campos (inglés)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.md)
- [Uso detallado y referencia de campos (japonés)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.ja.md)
