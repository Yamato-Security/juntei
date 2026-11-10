# Primera investigación { #first-investigation }

Este flujo de trabajo supone que ha recibido una recopilación de triaje offline en `evidence/`.

## 1. Preservar el origen { #1-preserve-the-source }

Trabaje a partir de una copia verificada. Registre los hashes, la hora de la recopilación, la zona horaria, el host, el volumen y la herramienta
de recopilación. Juntei lee las evidencias, pero opciones como la extracción de flujos residentes o de la cuarentena crean
nuevos archivos de salida; diríjalos a un directorio de análisis independiente.

## 2. Inventariar lo que tiene { #2-inventory-what-you-have }

Busque las rutas indicadas en [Recopilación de evidencias](collection.md). Es normal que falten artefactos. No
suponga que una recopilación contiene registros de transacciones solo porque contenga un hive del Registro.

## 3. Ejecutar primero los resúmenes { #3-run-summaries-first }

Muchos comandos muestran un resumen conciso cuando no se solicita ningún archivo de salida:

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

Lea las advertencias de daños, de bases de datos con cambios pendientes y de registros no aplicados antes de confiar en los registros.

Para analizar en una sola operación todos los artefactos compatibles de una recopilación de varios equipos, utilice:

```bash
juntei auto-parse -d evidence/ -o analysis/
```

Consulte [Análisis automático de recopilaciones](../commands/auto-parse.md) para conocer la detección de hosts, el paralelismo,
la estructura de la salida y el JSONL consolidado para la ingesta en un SIEM.

## 4. Guardar la salida estructurada { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

Dé preferencia a JSONL para los scripts y los valores anidados, y a CSV para una hoja de cálculo o un visor de líneas de tiempo.

## 5. Pivotar desde un identificador fiable { #5-pivot-from-a-reliable-identifier }

Empiece por un nombre de archivo, una ruta completa, un SHA-1/SHA-256, un SID de usuario, una entrada y secuencia de la MFT, un GUID de tarea,
un nombre de servicio o una ventana temporal. Normalice las mayúsculas y minúsculas solo en los campos que Windows trata sin distinguirlas.

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. Correlacionar { #6-correlate }

Utilice las [investigaciones entre artefactos](../resources/workflows.md). Una secuencia práctica es:

1. Prefetch para la ejecución y las horas de las ejecuciones recientes.
2. `$MFT` y el Diario USN para la creación, las rutas, los cambios de nombre y la eliminación.
3. Amcache y ShimCache para la presencia histórica y los metadatos.
4. LNK, Jump Lists, RecentDocs y SRUM para el contexto del usuario y de la aplicación.
5. Los servicios del Registro y TaskCache para la persistencia.
6. La cuarentena de Defender para la detección, la eliminación, el origen y los hashes.

## 7. Documentar la reproducibilidad { #7-document-reproducibility }

Registre la ruta de origen, el hash del origen, la versión de Juntei, el comando exacto, el hash de la salida, la zona horaria del sistema
y cualquier filtrado posterior. No edite nunca en silencio la salida de evidencias generada.
