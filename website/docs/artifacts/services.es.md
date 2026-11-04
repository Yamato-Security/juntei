# Servicios de Windows { #windows-services }

## Qué registra la configuración de servicios { #what-service-configuration-records }

Los servicios y los controladores de kernel de Windows se configuran en cada conjunto de controles de `SYSTEM`. Los registros indican
el ejecutable, el controlador o la DLL del servicio, el modo de inicio, el tipo de servicio, la cuenta, el grupo de orden de carga, el inicio
retrasado, el comportamiento ante errores y las acciones que realiza Windows tras un error.

Los atacantes crean o modifican servicios para lograr persistencia, escalada de privilegios, carga de controladores y
movimiento lateral. El software legítimo utiliza exactamente el mismo mecanismo, por lo que importan las rutas, las cuentas, la firma,
el contexto de creación y las evidencias de ejecución.

<span class="evidence-strength">Mejor conclusión:</span> el conjunto de controles analizado configuraba el
servicio o controlador tal como se muestra. La configuración por sí sola no demuestra que el servicio se iniciara correctamente.

## Ubicación { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

Recopile `SYSTEM` junto con sus registros de transacciones. Los identificadores de evento como el 7045 de System y el 4697 de Security pueden
registrar la instalación; los eventos de control de servicios y de procesos pueden respaldar el inicio o la ejecución real.

## Precauciones para el analista { #analyst-cautions }

- `ImagePath` puede contener variables de entorno, comillas, argumentos, rutas de dispositivo o una ruta
  sin comillas que incluya espacios. Consérvela exactamente antes de normalizarla.
- `ServiceDll` es fundamental para los servicios alojados en `svchost.exe`; `ImagePath` por sí solo puede nombrar únicamente el proceso host.
- `StartType=automatic` no demuestra que el servicio se iniciara ni que siguiera habilitado.
- `ObjectName` es la cuenta configurada, no necesariamente el usuario que instaló el servicio.
- Los controladores y los servicios en modo de usuario utilizan indicadores `ServiceType` y convenciones de ruta diferentes.
- Las acciones de error pueden ejecutar un comando con la cuenta del servicio tras un error; se trata de una superficie de persistencia
  que es fácil pasar por alto.
- Otros conjuntos de controles pueden conservar la configuración anterior; utilice `IsCurrentControlSet`.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath` | Identidad del sistema y hive de entrada. Consérvelos entre recopilaciones. |
| `ControlSet`, `IsCurrentControlSet` | Conjunto de controles de origen y si `Select\Current` lo designa. Dé preferencia al actual para afirmaciones sobre el estado activo. |
| `ServiceName` | Nombre de la clave del Registro o nombre interno del servicio. Pivote sólido hacia los eventos y los comandos del Administrador de control de servicios. |
| `DisplayName`, `Description` | Texto visible para el usuario; puede ser una referencia a recursos o estar controlado por el atacante. |
| `ImagePath` | Comando del ejecutable o del controlador tal como está configurado. Analice las comillas, los argumentos, las variables de entorno y las rutas de dispositivo. |
| `ServiceDLL` | DLL cargada por un servicio host. A menudo es la carga útil real en los servicios de proceso compartido. |
| `StartType`, `StartValue` | Modo de inicio legible y sin procesar: arranque (0), sistema (1), automático (2), manual (3), deshabilitado (4). |
| `DelayedAutostart` | El servicio automático se inicia después de la fase inicial de arranque; sigue siendo apto para la persistencia. |
| `ServiceType`, `TypeValue` | Indicadores decodificados y sin procesar: controlador de kernel, controlador de sistema de archivos, proceso propio o compartido, servicio de usuario, etc. |
| `ErrorControl` | Nivel de tratamiento de errores de arranque o inicio. Principalmente, contexto operativo. |
| `ObjectName` | Cuenta de servicio configurada o nombre del objeto de controlador. No es la identidad de quien lo instaló. |
| `Group` | Grupo de orden de carga, especialmente importante para los controladores de arranque y de sistema. |
| `FailureResetPeriod` | Segundos antes de que se restablezca el recuento de errores; `4294967295` significa nunca. |
| `FailureCommand` | Comando configurado para una acción de error `run_command`. Campo de persistencia de gran valor. |
| `FailureActions` | Lista completa decodificada de acciones (`restart`, `reboot`, `run_command`, `none`) y retrasos. |
| `FailureActionType`, `FailureActionDelayMs` | Primera acción aplanada para facilitar el filtrado; inspeccione también la lista completa. |
| `FailureActionsRaw` | Bytes originales cuando la decodificación está incompleta o dañada. |
| `KeyLastWrittenUTC` | Último cambio en la clave del servicio, no necesariamente la hora de instalación o de inicio. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la reproducción del Registro. |
| `Problems` | Discrepancias de tipo, estructura de errores con formato incorrecto o campos ausentes o ilegibles. |

## Estrategia de investigación { #investigation-strategy }

1. Revise las rutas no estándar, los directorios con permisos de escritura, las cuentas inusuales, las horas de clave recientes y los comandos de error.
2. Expanda las variables de entorno utilizando la configuración del host afectado, no la del equipo del analista.
3. Calcule el hash y compruebe la firma del ejecutable o la DLL reales si se recopilaron.
4. Correlacione el nombre y la ruta del servicio con los eventos 7045/4697, la creación de procesos, Prefetch, `$MFT` y USN.
5. Compare los conjuntos de controles actuales y no actuales para detectar persistencia modificada o eliminada.

## Más información { #further-details }

- [Microsoft: árbol del Registro para servicios de dispositivos y controladores](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / tipos y estados de servicios](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: acciones de error de ChangeServiceConfig2](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
