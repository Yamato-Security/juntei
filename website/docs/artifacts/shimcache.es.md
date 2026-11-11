# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## Qué es ShimCache { #what-shimcache-is }

El subsistema de compatibilidad de aplicaciones de Windows almacena en caché rutas de ejecutables y metadatos mientras evalúa
el comportamiento de compatibilidad o de los shims. La caché sobrevive a que los archivos cambien de nombre, se eliminen o se desconecten junto con
un volumen extraíble. Su estructura y los campos disponibles difieren de Windows XP a Windows 11.

ShimCache es valioso para descubrir rutas históricas y, cuando el formato tiene un indicador de ejecución
fiable, para respaldar adicionalmente la ejecución. La hora almacenada en caché suele ser la **hora de última modificación del archivo**,
no la hora en que Windows lo vio o lo ejecutó.

<span class="evidence-strength">Mejor conclusión:</span> Windows observó la ruta durante el procesamiento
de compatibilidad. Una entrada por sí sola es evidencia de presencia u observación; utilice `Executed` solo cuando sea compatible.

## Ubicación y recopilación { #location-and-collection }

Los bytes se encuentran en el hive del Registro `SYSTEM`, normalmente en:

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

Recopile `SYSTEM`, `SYSTEM.LOG1` y `SYSTEM.LOG2`. Juntei lee todos los conjuntos de controles e identifica
`Select\Current` cuando es posible.

!!! warning "Las entradas activas más recientes pueden no estar en el hive"
    Windows puede mantener las actualizaciones de ShimCache en memoria y volcarlas durante el apagado o el reinicio. Un hive copiado
    de un sistema en ejecución puede no contener las observaciones más recientes. La hora de última escritura de la caché en el Registro
    ayuda a describir cuándo se guardó, pero no es una hora de ejecución por entrada.

## Precauciones para el analista { #analyst-cautions }

- Una entrada de la caché por sí sola **no es prueba de ejecución**.
- `Executed` puede ser `Yes`, `No` o `NA`; la semántica disponible depende del formato o la versión.
- `LastModifiedTimeUTC` pertenece al archivo referenciado, no a la observación ni a la ejecución.
- El orden es útil, pero no constituye una cronología precisa universal; se producen duplicados entre conjuntos de controles y el espacio residual.
- Las entradas `FromSlack` se recuperaron fuera de la caché activa y requieren una precaución adicional.
- Un `SYSTEM` inconsistente sin registros de transacciones utilizables puede omitir registros actuales. Lea el resumen de la recuperación.
- Los hallazgos indican patrones sospechosos de nombres o rutas, no veredictos de malware.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `ControlSet` | Conjunto de controles numérico que contiene la caché. Compare `IsCurrent` antes de elegir uno. |
| `CacheEntryPosition` | Posición u orden dentro de la caché decodificada. Útil para el orden relativo, no es una hora. |
| `Path` | Ruta del ejecutable o paquete en caché. Pivote principal de búsqueda. |
| `LastModifiedTimeUTC` | Hora de modificación del archivo almacenada en la entrada, no la hora de ejecución ni de observación. |
| `Executed` | `Yes`, `No` o `NA` según los indicadores de formato admitidos. Utilícelo solo con el contexto de la versión. |
| `Duplicate` | La misma entrada ya se había visto en un conjunto de controles anterior o en el espacio residual; ayuda a evitar contarla dos veces. |
| `SourceFile` | Hive o valor sin procesar de entrada. Imprescindible cuando se trabaja con varios hosts. |
| `FromSlack` | Entrada recuperada del espacio residual del valor de la caché en lugar de la lista activa. Menor confianza y posiblemente más antigua. |
| `CacheFormat` | Estructura de Windows o de la caché decodificada. Determina la disponibilidad y la semántica de los campos. |
| `CacheLastWrittenUTC` | Hora de guardado de la clave del Registro o de la caché. Es contexto compartido, no la hora de ejecución de cada entrada. |
| `InsertFlags`, `ShimFlags` | Indicadores de compatibilidad sin procesar; `Executed` es el resultado interpretado por Juntei cuando es compatible. |
| `DataSize` | Tamaño de los datos de esta entrada; útil para la validación estructural. |
| `Machine` | Arquitectura registrada por los formatos más recientes (`x64`, `x86`, `ARM64`). |
| `FileSize`, `LastUpdateTimeUTC` | Metadatos en caché específicos de Windows XP, cuando están presentes. |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | Identidad de la aplicación empaquetada en las entradas compatibles. El identificador del editor es contexto, no prueba de firma. |
| `Findings` | Códigos de anomalía de Juntei, como un binario del sistema reubicado o una ruta de preparación sospechosa. Solo son pistas. |
| `RegistryPath`, `RegistryValueName` | Clave y valor exactos de origen en el hive. Consérvelos para la validación. |
| `CacheValueSize` | Tamaño de todo el valor del Registro, no el `DataSize` de la entrada individual. |
| `IsCurrent` | Si el conjunto de controles de esta entrada coincide con `SYSTEM\Select\Current`; vacío si no se conoce con seguridad. |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | Números de conjunto de controles sin procesar de `SYSTEM\Select`, para el contexto y el análisis de la recuperación. |

Además, JSON/JSONL conserva los datos de las entradas, las etiquetas, los subregistros de aplicaciones empaquetadas, los hallazgos estructurados,
los problemas de la caché y los detalles de la recuperación.

## Estrategia de investigación { #investigation-strategy }

1. Dé preferencia al conjunto de controles actual, pero conserve los demás conjuntos y el espacio residual como evidencia histórica.
2. Busque en las rutas y los hallazgos; no utilice `LastModifiedTimeUTC` como hora de ejecución.
3. Cuando `Executed == Yes`, corrobórelo con Prefetch, SRUM, los registros de eventos u otros rastros de ejecución.
4. Empareje las rutas con `$MFT`/USN para determinar la creación, la eliminación y el historial de metadatos.
5. Indique si se reprodujeron los registros de transacciones y si es posible que la caché activa no se hubiera volcado.

## Más información { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [shimcache-forensic de Security Ronin](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: aprovechamiento de la caché de compatibilidad de aplicaciones en investigaciones forenses](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Especificación del formato del Registro de Windows de Maxim Suhanov](https://github.com/msuhanov/regf)
