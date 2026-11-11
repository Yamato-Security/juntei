# Cuentas locales SAM { #sam-local-accounts }

## Qué contiene el SAM { #what-the-sam-contains }

El hive Security Account Manager (`SAM`) almacena los registros de las cuentas locales. Juntei
decodifica los nombres de cuenta, los RID, los campos de perfil y comentario, los indicadores de
control de la cuenta, las marcas de tiempo del último inicio de sesión y de la contraseña, los
contadores, las pistas de contraseña, la vinculación con cuentas Microsoft, las rutas de perfil y
si Windows oculta la cuenta en la interfaz de inicio de sesión.

Resulta útil para encontrar administradores locales creados recientemente, cuentas inactivas que se
han habilitado, cambios de contraseña, cuentas ocultas y cuentas preparadas para el movimiento
lateral.

<span class="evidence-strength">Mejor conclusión:</span> el SAM local representaba la cuenta con el
estado registrado. No es un historial de autenticación completo y no describe cuentas de dominio.

## Ubicación y recopilación { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

Recopile el hive y los registros de transacciones con privilegios suficientes o acceso sin procesar
(raw). La identidad del host puede enriquecerse a partir del hive `SYSTEM` correspondiente de la
recopilación.

## Precauciones para el analista { #analyst-cautions }

- Este comando analiza cuentas **locales**, no usuarios de dominio de Active Directory.
- Juntei no extrae intencionadamente los hashes de contraseñas.
- El RID 500 es la cuenta Administrator integrada aunque se haya renombrado; el nombre por sí solo
  no basta.
- `LastLogon` y los contadores son estado del Registro, no un registro de auditoría completo por
  sesión.
- `HiddenFromLogonUI` oculta una cuenta en la pantalla de inicio de sesión; no la deshabilita.
- Las cuentas integradas `DefaultAccount` y `WDAGUtilityAccount` pueden estar ocultas de forma
  legítima.
- Los indicadores de cuenta nulos procedentes de un valor `F` dañado o ausente no deben
  interpretarse como cuenta habilitada.
- `NamesKeyLastWrittenUTC` aproxima el contexto de registro del nombre de la cuenta, pero no es una
  hora de creación garantizada.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `SourcePath` | Contexto del host y hive SAM exacto. |
| `RID` | Relative Identifier. Pivote sólido de identidad de la cuenta; 500 es el Administrator integrado. |
| `User` | Nombre actual de la cuenta local. Puede cambiarse; correlacione por RID. |
| `FullName`, `Comment`, `UserComment` | Texto descriptivo; lo controla el atacante y es útil para revisar suplantaciones. |
| `AccountFlags` | Estados decodificados como deshabilitada, bloqueada, contraseña no requerida, cuenta normal o contraseña que nunca caduca. |
| `AccountFlagsValue` | Indicadores numéricos sin procesar para la validación y para bits no admitidos. |
| `CountryCode`, `CodePage` | Metadatos de configuración regional y página de códigos de la cuenta; normalmente contexto, no atribución. |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | El mismo estado del último inicio de sesión con precisión completa y en segundos enteros por compatibilidad. No es un historial completo. |
| `PasswordLastSetUTC` | Hora registrada del cambio de contraseña. Investigue valores inesperadamente recientes o antiguos. |
| `AccountExpirationUTC` | Caducidad configurada cuando es finita. Un valor nulo puede significar nunca o no establecida. |
| `LastFailedLogonUTC` | Último inicio de sesión local fallido registrado en el estado del SAM; corrobórelo con los eventos de Security. |
| `FailedLogonCount`, `LogonCount` | Contadores acumulados o de estado con un comportamiento de actualización propio de Windows; úselos como contexto, no como recuento exacto de eventos. |
| `PasswordHint` | Pista sensible proporcionada por el usuario; puede revelar patrones de contraseña. Trátela adecuadamente. |
| `InternetUserName` | Identidad de la cuenta Microsoft vinculada, si existe. |
| `HiddenFromLogonUI` | Estado de ocultación en la interfaz. Revise los RID altos y las cuentas personalizadas inesperadas; no significa que esté deshabilitada. |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | Configuración del entorno y del inicio de sesión de la cuenta local; los scripts o rutas UNC inusuales pueden indicar persistencia. |
| `KeyLastWrittenUTC` | Hora de cambio de la clave de la cuenta del RID. A nivel de clave, no específica de un campo. |
| `NamesKeyLastWrittenUTC` | Hora de cambio de la clave de asignación del nombre de usuario; útil como aproximación del ciclo de vida de la cuenta. |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | Valores FILETIME de 64 bits sin procesar que se conservan para la validación y el tratamiento de valores centinela especiales. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la reproducción de los registros de transacciones del Registro. |
| `Problems` | Discrepancias entre F/V/Names, entradas ausentes, desplazamientos mal formados o daños en la decodificación. |

## Estrategia de investigación { #investigation-strategy }

1. Compare el RID, el nombre, los indicadores, la ruta del perfil y `NamesKeyLastWrittenUTC` con la
   línea base del host.
2. Revise las cuentas habilitadas con RID 1000 o superior, las que no requieren contraseña, las que
   nunca caducan, las ocultas o las modificadas recientemente.
3. Correlacione con los eventos 4720/4722/4724/4732 de Security, los eventos de inicio de sesión, la
   creación de perfiles y la actividad remota.
4. Busque en el historial de recuperación del Registro si una cuenta se eliminó antes de la
   recopilación.
5. Distinga las cuentas legítimas de aprovisionamiento o predeterminadas de las cuentas
   interactivas creadas por un atacante.

## Más información { #further-details }

- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Plugin SAM de dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
