# Historial de detecciones de Microsoft Defender { #microsoft-defender-detection-history }

## Qué registra el historial { #what-the-history-records }

Microsoft Defender escribe un archivo de historial de detecciones por cada detección. Cada archivo registra:

- el identificador, el nombre, la gravedad y la categoría de la amenaza;
- qué hizo Defender y si tuvo éxito;
- si la amenaza se estaba ejecutando;
- el usuario y el proceso implicados, y qué detectó la amenaza;
- la entrada de cuarentena, si existe;
- cada recurso en el que se encontró la amenaza: archivos, miembros de archivos comprimidos, descargas, procesos, valores del Registro,
  servicios, líneas de comandos y comportamientos.

Defender también escribe estos archivos para amenazas que nunca puso en cuarentena: las que solo detectó, las que
se permitieron y las que no logró eliminar. El historial de protección de Seguridad de Windows y la clase WMI
`MSFT_MpThreatDetection` (`Get-MpThreatDetection`) muestran estos registros. Cuando Orange
Cyberdefense modificó valores en un archivo, WMI informó de los valores modificados.

<span class="evidence-strength">Mejor conclusión:</span> Defender registró una detección de la amenaza indicada
en los recursos enumerados, con el estado y las horas registrados. El nombre de la amenaza es el veredicto del
producto. Un registro de detección por sí solo no demuestra que la amenaza se ejecutara ni que se eliminara
por completo.

## Ubicación y recopilación { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

Cada archivo lleva el nombre de su `DetectionID`. Recopile toda la carpeta `DetectionHistory`. La entrada de directorio
y `auto-parse` leen cada archivo situado dentro de una carpeta `DetectionHistory`, a cualquier profundidad, que empiece
como un archivo de historial de detecciones. Utilice `-f` para una copia guardada fuera de una carpeta así. Defender elimina
elementos de su historial de análisis tras un número configurable de días (`ScanPurgeItemsAfterDelay` en
`Set-MpPreference`), por lo que es posible que las detecciones más antiguas ya no estén.

La cuarentena es un artefacto independiente: consulte [Cuarentena de Defender](defender-quarantine.md). El
`QuarantineID` de una detección usa la misma grafía que el `EntryID` de la salida de la cuarentena (en mayúsculas,
sin llaves), por lo que ambos pueden vincularse.

## Precauciones para el analista { #analyst-cautions }

- La detección, un intento de corrección y una eliminación correcta son estados distintos. Lea
  `ThreatStatus` (`Detected`, `Quarantined`, `Removed`, `Allowed`, `RemoveFailed`, …) junto con
  `ThreatStatusErrorCode`, que vale 0 cuando el último cambio de estado tuvo éxito.
- `ExecutionStatus` es la visión de Defender en el momento de la detección (`Blocked`, `Allowed`, `Executing`,
  `NotExecuting`, `Unknown`). No demuestra que la amenaza se ejecutara, ni que nunca se ejecutara.
  Corrobórelo con Prefetch, Amcache, los registros de eventos y la telemetría de procesos.
- `ThreatName` es el nombre de la firma en ese momento. Son posibles los falsos positivos y las reclasificaciones
  posteriores. Los archivos de prueba como EICAR se notifican como amenazas reales.
- `User` es el valor `DomainUser` que registró Defender (`UserSource` es `DomainUser`). Puede estar
  vacío, ser una cuenta de servicio o ser un usuario distinto del que creó el archivo.
- Defender puede escribir varios archivos para una misma amenaza, y Juntei conserva cada uno de ellos. Dos archivos pueden
  incluso contener el mismo `DetectionID`, como ocurre con un archivo copiado o restaurado. Agrupe por `DetectionID` y
  `SourceFile` a la vez.
- Los valores cuyo significado no se conoce se conservan, no se interpretan: su sección y su posición están en
  `UnknownValues`.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

CSV es el formato de archivo predeterminado. Con `-t csv`, `-t json` o `-t jsonl` y sin `-o`, los registros se
escriben en stdout. Sin ninguna de las dos opciones, solo se muestra un resumen. Añada `-C` para sobrescribir una salida
existente. La salida nunca se escribe sobre uno de los archivos de entrada, ni siquiera con `-C`.

Juntei escribe un registro por cada recurso de cada detección. Una detección sin recursos obtiene un registro
con los campos de recurso vacíos. Los registros de una detección comparten su `DetectionID` y se numeran con
`ResourceIndex` en el orden almacenado.

Los daños no detienen la ejecución. Un archivo truncado conserva los valores leídos antes del daño, y
`Problems` indica lo que falta. Un archivo de una carpeta `DetectionHistory` que no es un archivo de historial
de detecciones se notifica y se omite. Los demás archivos se siguen leyendo.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Record` | `resource`, o `detection` para una detección sin recursos. |
| `InitialDetectionTime` | Cuándo detectó Defender la amenaza por primera vez (UTC). Vacío cuando el archivo contiene 0; un valor que no es una hora se conserva en `UnknownValues`. |
| `LastThreatStatusChangeTime` | Cuándo cambió por última vez el estado de la amenaza (UTC). |
| `RemediationTime` | Cuándo corrigió Defender la amenaza (UTC). Es la hora de un intento, no una prueba de eliminación; compruebe `ThreatStatus`. |
| `ThreatName`, `ThreatID` | Nombre de firma e identificador numérico de Defender (`Get-MpThreatCatalog`). |
| `Severity`, `SeverityID` | `Low`, `Moderate`, `High`, `Severe` o `Unknown`, con el número almacenado. |
| `Category`, `CategoryID` | Categoría de la amenaza, como `TROJAN`, `TOOL` o `BEHAVIOR`, con el número almacenado. |
| `ThreatStatus`, `ThreatStatusID` | Estado de la amenaza según Defender. Un número sin nombre documentado se conserva con un `ThreatStatus` vacío. |
| `ThreatStatusErrorCode` | Código de error del último cambio de estado; 0 significa que tuvo éxito. |
| `ExecutionStatus`, `ExecutionStatusID` | Si Defender vio la amenaza en ejecución cuando la detectó. |
| `DetectionSource`, `DetectionSourceTypeID` | Qué detectó la amenaza: `Real-time`, `IOAV` (descargas y archivos adjuntos), `User`, `System`, `AMSI` y otros. |
| `User`, `UserSource` | El valor `DomainUser` que registró Defender y su base (`DomainUser`). Vacío cuando el archivo no nombra a ningún usuario. |
| `Process` | El proceso implicado (`ProcessName`); puede ser `Unknown`. |
| `ResourceIndex`, `ResourceCount` | Posición de este recurso y número de recursos de la detección. |
| `ResourceType` | `file`, `containerfile`, `webfile`, `process`, `regkeyvalue`, `service`, `CmdLine`, `behavior`, `amsi` y otros, tal como los nombra Defender. |
| `Path` | El recurso: una ruta, un miembro de un archivo comprimido (`archive.zip->member`), un valor del Registro, una línea de comandos o un proceso. Un `webfile` también indica su URL y el proceso que lo descargó (`pid:…,ProcessStart:…`). Conserve la ortografía exacta. |
| `ResourceFlags` | Indicadores del recurso en hexadecimal; su significado no está documentado. |
| `TrackingStartTime` | El `ThreatTrackingStartTime` del recurso, un entero leído como FILETIME (UTC), igual que plaso fecha una detección. |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | Hashes y tamaño de los datos de seguimiento del recurso. Describen el archivo en el momento en que Defender lo analizó. |
| `TrackingID` | El `ThreatTrackingId` del recurso. |
| `OtherTracking` | Todos los demás valores de seguimiento, como `Key=Value` en el orden almacenado, por ejemplo `ThreatTrackingScanSource` o `ThreatTrackingBMContextRich` en las detecciones de comportamiento. |
| `QuarantineID` | La entrada de cuarentena (`Quarantine\Entries\{GUID}`); vacío cuando el archivo contiene un GUID cero. |
| `DetectionID` | El GUID de la detección; también es el nombre del archivo. Una discrepancia se notifica en `Problems`. |
| `MagicVersion` | El `Magic.Version:1.2` del encabezado; vacío en los archivos escritos sin él, como algunos de Windows Server 2016. |
| `UnknownValues` | Valores cuyo significado no se conoce, como `position=value` (por ejemplo, `6.13=NT AUTHORITY\SYSTEM`), con la lista contada que sigue al estado como `2.list`. |
| `SourceFile` | El archivo leído, relativo al directorio de entrada, o su nombre con `-f`. |
| `Problems` | Daños, valores ausentes, tipos inesperados o un nombre de archivo que no coincide. No lo suprima. |

JSON y JSONL conservan los mismos campos con valores tipados: los números siguen siendo números y los valores ausentes son
`null`. En lugar de `OtherTracking` tienen `Tracking`, cada valor de seguimiento con la clave de Defender,
su tipo y su valor. `UnknownValues` se convierte en una lista de posiciones con tipos. `CountedList`
contiene la lista contada. Un FILETIME almacenado conserva su número bruto de ticks junto a la hora que representa.

## Comportamiento en la línea de tiempo { #timeline-behavior }

`auto-parse` guarda los registros como `SecurityProducts/DefenderHistory.csv` (o `.json`/`.jsonl`) y
los añade a la línea de tiempo combinada de `-j`. `super-timeline` genera eventos a partir de
`InitialDetectionTime`, `LastThreatStatusChangeTime` y `RemediationTime` en el registro de cada recurso,
con el estado, el código de error, el estado de ejecución y el origen de la detección en los detalles del
evento. `TrackingStartTime` se almacena como un entero, por lo que solo se convierte en evento con
`--include-interpreted-times`. Nunca se usan las horas propias del archivo. Un registro sin ninguna hora utilizable
permanece en la salida analizada y se cuenta como sin hora.

## Estrategia de investigación { #investigation-strategy }

1. Empiece por `ThreatName`, `Path`, `SHA256`, `InitialDetectionTime` y `ThreatStatus`.
2. Compruebe `ThreatStatusErrorCode` y `Problems` antes de concluir que se eliminó una amenaza.
3. Vincule `QuarantineID` con el `EntryID` de la cuarentena para recuperar los bytes en cuarentena.
4. Pivote con los hashes y las rutas hacia `$MFT`, el Diario USN, Prefetch, Amcache, el historial del navegador y los registros
   de eventos (el registro operativo de Defender: identificadores de evento 1116, 1117, 1118 y 1119).
5. Para los recursos `webfile`, utilice la URL y el inicio del proceso para investigar el acceso inicial, y
   valídelos con evidencias del navegador, el proxy y el DNS.

## Versiones probadas { #tested-versions }

Las pruebas usan archivos públicos con resultados que otras herramientas leyeron de forma independiente:

- La muestra de Windows 10 de Orange Cyberdefense (Defender 4.18.24090.11), cotejada con lo que informó
  WMI.
- Los dos archivos de prueba de Windows de plaso, cotejados con el analizador `windefender_history` de plaso.
- 14 archivos de la máquina virtual del simulador de APT de DFIRArtifactMuseum.
- Una copia escrita sin la versión del encabezado, como algunos archivos de Windows Server 2016.
- Archivos truncados, vacíos o de otro tipo.

## Más información { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [defender de Sootmark](https://github.com/Sootmark/defender)
- [wddh-parser de Orange Cyberdefense](https://github.com/cert-orangecyberdefense/wddh-parser)
- [Analizador del historial de Defender de plaso](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: clase MSFT_MpThreatDetection](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: identificadores de eventos del Antivirus de Microsoft Defender](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
