# Evidencia e interpretación { #evidence-and-interpretation }

La ciencia forense digital no consiste solo en leer campos. Consiste en decidir qué observación respalda el campo,
con qué solidez la respalda y qué podría producir la misma observación de forma inocente.

## Cuatro categorías útiles de evidencia { #four-useful-evidence-categories }

| Categoría | Significado | Ejemplos |
|---|---|---|
| Presencia | Windows conocía una ruta o un archivo | Registro de archivo de Amcache, entrada de ShimCache, registro de `$MFT` |
| Ejecución | Windows registró el inicio de un programa o un estado de ejecución | Hora de ejecución de Prefetch; indicador de ejecución de ShimCache compatible |
| Interacción del usuario | Un usuario o una aplicación hizo referencia a un elemento o lo abrió | RecentDocs, Jump List, LNK de la carpeta Recent |
| Actividad del sistema | Windows registró el uso de recursos o la configuración | Contadores de SRUM, configuración de servicios, acción de TaskCache |

Un artefacto puede respaldar más de una categoría, pero nunca debe promoverse automáticamente. Por ejemplo,
una entrada de Amcache con una ruta sospechosa respalda la **presencia**. Una hora de ejecución de Prefetch coincidente aporta
una evidencia de **ejecución** más sólida. Un registro de creación del USN y la ruta de `$MFT` pueden situar el archivo en el disco, y
la cuarentena de Defender puede explicar por qué desapareció.

## Las marcas de tiempo son afirmaciones, no verdades universales { #timestamps-are-statements-not-universal-truth }

Pregúntese qué originó cada marca de tiempo:

- Las horas de `$STANDARD_INFORMATION` y `$FILE_NAME` describen metadatos de NTFS y pueden divergir.
- Una hora de ejecución de Prefetch describe un inicio registrado, pero solo se conservan las horas de ejecución recientes.
- Una hora de ShimCache suele ser la hora de última modificación del archivo almacenado en caché, no la hora de ejecución.
- La hora de última escritura de una clave del Registro se aplica a la clave en su conjunto, no a un valor individual.
- RecentDocs asigna la hora de la clave al elemento más reciente; los elementos más antiguos tienen un orden, pero normalmente no tienen hora de apertura.
- Las marcas de tiempo de SRUM suelen identificar un intervalo de agregación, no la operación de red exacta.

Utilice UTC durante el análisis, conserve el valor original cuando esté disponible y documente cualquier conversión.

## Evidencia positiva y negativa { #positive-and-negative-evidence }

La presencia de un registro bien comprendido puede ser una evidencia positiva sólida. La ausencia suele ser
más débil, porque los artefactos se sobrescriben de forma cíclica, las cachés tienen un límite, la recopilación puede estar incompleta, la configuración varía
y existen herramientas de limpieza. Diga «no se observó en el artefacto recopilado» en lugar de «no ocurrió».

## Escalera de corroboración { #corroboration-ladder }

1. Valide el archivo de origen y la ruta de recopilación.
2. Confirme que el analizador no informó de daños relevantes ni de registros de transacciones sin aplicar.
3. Interprete el campo según la página del artefacto.
4. Busque un artefacto independiente que respalde la misma acción.
5. Compruebe los registros de eventos, la telemetría de red, la memoria o el contenido del archivo cuando estén disponibles.
6. Registre la versión exacta de Juntei, el comando, los filtros y el hash del origen.

!!! tip "Redacte las conclusiones según la solidez de la evidencia"
    Es preferible «Prefetch registra la ejecución de `tool.exe` a las 10:02:11Z» a «el atacante ejecutó el
    malware exactamente a las 10:02:11Z». La segunda afirmación añade identidad e intención que Prefetch
    por sí solo no puede establecer.
