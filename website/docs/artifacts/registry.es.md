# Hives del Registro de Windows { #windows-registry-hives }

## Qué son los hives del Registro { #what-registry-hives-are }

El Registro de Windows almacena la configuración del sistema y de los usuarios en archivos de hive
binarios. Las claves forman un árbol; los valores de una clave contienen datos con tipo. Los
artefactos del Registro describen la persistencia, el software instalado, los servicios, las
cuentas, la configuración de red, la actividad de los usuarios, la configuración de seguridad y
prácticamente cualquier aspecto del estado de Windows.

El lector genérico de Juntei decodifica cualquier hive principal, reproduce los registros de
transacciones, informa de los daños y puede recuperar registros eliminados o de celdas libres, del
espacio residual de celdas, de restos de datos, anteriores a la reproducción y presentes solo en
los registros de transacciones.

<span class="evidence-strength">Mejor conclusión:</span> el hive contenía, o conservaba de forma
recuperable, el estado de la clave o del valor. La hora de última escritura de una clave indica que
la clave cambió, no qué valor cambió ni por qué.

## Hives habituales y ubicaciones { #common-hives-and-locations }

| Hive | Ruta típica | Ejemplos de evidencia |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | Servicios, controladores, conjuntos de control, nombre de host, dispositivos montados, ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | Software instalado, TaskCache, perfiles, configuración de Windows |
| `SAM` | `C:\Windows\System32\config\SAM` | Cuentas locales y metadatos de cuentas |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | Directiva de seguridad local y metadatos de secretos (según el acceso) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | Configuración del perfil predeterminado del sistema |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | Preferencias del usuario, claves Run, RecentDocs, actividad de aplicaciones |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | Clases del shell, actividad COM y del shell del usuario |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | Inventario de aplicaciones (utilice `parse-registry amcache` para obtener una salida semántica) |

Recopile los archivos `.LOG`, `.LOG1` y `.LOG2` adyacentes. Un hive copiado mientras Windows está en
ejecución puede estar sucio (dirty); los registros de transacciones contienen estado confirmado que
el archivo principal por sí solo aún no refleja.

## Precauciones para el analista { #analyst-cautions }

- La marca de tiempo de última escritura de una clave se aplica a la **clave**, no a un valor
  concreto.
- Los valores del Registro normalmente no tienen una marca de tiempo propia.
- `CurrentControlSet` es un alias de Windows y de los analistas que se resuelve mediante
  `SYSTEM\Select`; no es una clave almacenada literalmente.
- Los registros eliminados pueden ser parciales, obsoletos, estar duplicados entre fuentes de
  recuperación o haberse sobrescrito.
- Los registros presentes solo en los registros de transacciones muestran un estado histórico o
  intermedio. Explique `recovered_from`.
- Un resultado `absent_value` significa que la clave consultada existe pero carece de ese valor en
  el estado analizado; no prueba que el valor nunca haya existido.
- Las cadenas, rutas y líneas de comandos son configuración, no una prueba de que la acción
  configurada se ejecutara.
- Un hive sucio sin registros de transacciones aplicables puede omitir estado o mostrar un estado
  obsoleto.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host` | Nombre de host derivado del contexto del sistema disponible; vacío cuando Juntei no puede identificarlo con seguridad. |
| `HiveType` | Categoría de hive detectada (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT`, etc.). Úsela para interpretar las rutas. |
| `SourcePath` | Hive de entrada exacto. Esencial cuando el modo de directorio combina hosts o usuarios. |
| `RecordType` | `key`, `value`, `default_value` o `absent_value`. La ausencia se aplica solo a la consulta o al estado. |
| `KeyPath` | Ruta bajo la raíz del hive. `[unknown]` marca una ascendencia eliminada sin resolver. |
| `KeyLastWrittenUTC` | Última modificación de la clave en su conjunto. No permite identificar qué valor cambió. |
| `ValueName` | Valor con nombre; vacío para un valor predeterminado o sin nombre. |
| `ValueType` | Tipo del Registro (`REG_SZ`, `REG_DWORD`, `REG_BINARY`, etc.). El tipo influye en la interpretación. |
| `ValueData` | Valor representado según su tipo. Las cadenas múltiples siguen siendo listas en JSON. |
| `DataSize` | Tamaño registrado por el valor. Compárelo con los bytes sin procesar y con `Problems`. |
| `ValueDataRaw` | Bytes exactos cuando la decodificación falla o se utiliza `--raw-data`. Consérvelos para la validación. |
| `ValueU64` | Representación numérica entera para DWORD/QWORD sin pérdida de precisión en JavaScript. |
| `ValueFiletimeUTC` | Representación opcional de QWORD como FILETIME. Solo tiene sentido cuando el esquema de la clave define un FILETIME. |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only` o `transaction_log_recovered`. Determina el grado de confianza. |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay` o una entrada de registro de transacciones como `SYSTEM.LOG1#1234`. |
| `TransactionLogStatus` | `clean`, `replayed`, `unchanged` o `not_applied` para el contexto de este registro o hive. |
| `LogsApplied` | Nombres de los archivos de registro de transacciones que Juntei reprodujo. Indíquelos en el informe. |
| `Problems` | Daños, celdas ausentes, discrepancias de tipo, listas o claves principales sin resolver u otras limitaciones. |

## Interpretación del estado de recuperación { #recovery-status-interpretation }

| Estado | Significado |
|---|---|
| `deleted_complete` | La clave o el valor eliminado se recuperó con la ruta y los datos conocidos completos. Sigue siendo histórico, no un estado activo. |
| `deleted_partial` | Parte de la ruta, la lista de valores, el valor o los datos se sobrescribió o es ilegible. No rellene los huecos con celdas reutilizadas. |
| `residual_value_only` | El valor se conservó sin suficiente contexto de clave; puede faltar `KeyPath`. |
| `transaction_log_recovered` | El estado solo existe en versiones anteriores a la reproducción o en los registros de transacciones, y no en los registros asignados o libres actuales. |

## Estrategia de investigación { #investigation-strategy }

1. Empiece por una clave semántica exacta cuando la conozca; utilice búsquedas con expresiones
   regulares o en los datos para descubrir.
2. Compruebe `TransactionLogStatus`, `AllocationStatus`, `RecoveredFrom` y `Problems` antes que el
   contenido.
3. Interprete los datos según el esquema documentado de la clave; `REG_BINARY` no se describe a sí
   mismo.
4. Correlacione la configuración con evidencias de ejecución (registros de eventos, Prefetch, SRUM,
   servicios/tareas).
5. Conserve los datos sin procesar para los campos nuevos o controvertidos.

## Más información { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Especificación del formato del Registro de Maxim Suhanov](https://github.com/msuhanov/regf)
- [Analizador del Registro yarp](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Registry de Eric Zimmerman](https://github.com/EricZimmerman/Registry)
- [Microsoft: tipos de valores del Registro](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
