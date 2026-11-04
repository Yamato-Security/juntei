# Caché del Programador de tareas { #task-scheduler-cache }

## Qué es TaskCache { #what-taskcache-is }

El Programador de tareas de Windows mantiene en `SOFTWARE` una caché del Registro que vincula las rutas de las tareas con GUID,
desencadenadores y listas, entidades de seguridad, acciones, metadatos de ejecución y descriptores de seguridad. Juntei decodifica
las acciones de ejecución y de controlador COM, además de las acciones heredadas de correo electrónico y de mensaje, las marcas de tiempo de las tareas y los indicios de
tareas ocultas en la interfaz normal del Programador de tareas.

Las tareas son un mecanismo habitual de persistencia y de ejecución remota. La caché del Registro también es valiosa
cuando un atacante elimina o manipula el archivo XML de la tarea.

<span class="evidence-strength">Mejor conclusión:</span> TaskCache configuró o conservó la
tarea o la acción. Se necesitan los campos de ejecución y registros independientes para probar una ejecución concreta.

## Ubicaciones { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

Recopile `SOFTWARE` con sus registros de transacciones y todo el árbol `System32\Tasks`. Los registros de eventos Operational del Programador de tareas
y Security aportan evidencias de creación, actualización y ejecución.

## Precauciones para el analista { #analyst-cautions }

- La caché del Registro y el XML de la tarea pueden no coincidir tras una manipulación, una eliminación, un registro incompleto o daños.
- `Hidden` de Juntei identifica una ocultación estructural (ausencia del vínculo con Tree o del descriptor de seguridad en
  las condiciones pertinentes), no simplemente una tarea configurada con el indicador normal “Oculta” de la interfaz.
- Windows 7 almacena las acciones de las tareas en el XML en lugar de en el valor `Actions` de la caché; en ese caso, una acción vacía
  no es automáticamente sospechosa.
- `LastRun` y `LastResult` son útiles, pero no constituyen un historial de ejecución completo ni a prueba de manipulaciones.
- `Context` nombra la entidad de seguridad o el contexto configurados, no el usuario que registró la tarea.
- Los CLSID de los controladores COM requieren una resolución aparte en el Registro o en los archivos.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath` | Contexto del host y hive `SOFTWARE` de origen. |
| `TaskName` | Ruta completa en el programador. Pivote principal hacia el XML y los registros de eventos. |
| `TaskGUID` | GUID que une las entradas de caché de Tree y Tasks. Clave de correlación sólida. |
| `TaskIndex` | Metadatos de Tree o de índice utilizados por el programador; contextual, no es un recuento de ejecuciones. |
| `TaskLists` | Listas de la caché (`Boot`, `Logon`, `Plain`, `Maintenance`) que clasifican el desencadenamiento o el tratamiento. |
| `Hidden` | Indicador estructural de tarea oculta. Revise `InTree`, `InTasks` y el estado del descriptor antes de concluir que hubo manipulación. |
| `InTree`, `InTasks` | Si existen ambos lados de la caché. La falta del vínculo puede indicar eliminación, manipulación o un registro incompleto. |
| `HasSecurityDescriptor` | Si la entrada de Tree tiene `SD`; su ausencia puede contribuir al comportamiento de tarea oculta en las versiones pertinentes. |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | Cambios a nivel de clave para cada lado de la caché; no son horas exactas de creación ni de ejecución. |
| `ActionIndex` | Acción dentro de la tarea, empezando por cero. Una tarea puede generar varias filas. |
| `ActionType` | `exec`, `com_handler`, `send_email` o `show_message`. |
| `Executable`, `Args`, `WorkingDirectory` | Acción de ejecución efectiva. Interprete las comillas, las variables, las rutas relativas y los LOLBins. |
| `COM-ClassID`, `ActionData` | CLSID y datos del controlador COM, o carga útil heredada de correo electrónico o mensaje. Resuelva el CLSID por separado. |
| `Context` | Contexto de la entidad de seguridad codificado con las acciones (`LocalSystem`, `Author`, etc.); identidad configurada, no quien registró la tarea. |
| `Author`, `URI`, `Description` | Metadatos de la tarea. Fácilmente controlables por el atacante y no constituyen evidencia de confianza. |
| `LastRegisteredUTC` | Hora de registro o actualización de DynamicInfo, cuando está presente. |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | Metadatos de ejecución y resultado en caché. Correlaciónelos con los registros operativos y las evidencias de procesos. |
| `ActionsRaw` | Bytes originales de las acciones si la decodificación es incompleta o es necesaria una validación. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Discrepancia entre ruta y GUID, tipo de valor erróneo, estructura de acción truncada o elementos ausentes. |

## Estrategia de investigación { #investigation-strategy }

1. Revise primero las tareas ocultas o incoherentes y las rutas que no son de Microsoft.
2. Reconstruya los comandos efectivos, incluidos los argumentos, el directorio de trabajo, la expansión de variables de entorno y la resolución COM.
3. Compare el registro del Registro con el XML de la tarea y calcule el hash y las marcas de tiempo del ejecutable referenciado.
4. Correlacione `TaskName`/GUID/hora con los eventos del Programador de tareas, los eventos 4698/4702 de Security, los registros de procesos, Prefetch y MFT/USN.
5. Busque en los datos del Registro eliminados o recuperados cuando un atacante haya quitado la tarea.

## Más información { #further-details }

- [Microsoft: esquema del Programador de tareas](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: registro de eventos del Programador de tareas](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
