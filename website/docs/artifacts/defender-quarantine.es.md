# Cuarentena de Microsoft Defender { #microsoft-defender-quarantine }

## Qué contiene la cuarentena { #what-the-quarantine-contains }

La cuarentena de Microsoft Defender conserva metadatos sobre una detección y el contenido cifrado de los
recursos que puso en cuarentena. Juntei asocia los registros de Entries, ResourceData y Resources para informar de
la etiqueta de la amenaza, la hora de la cuarentena, las rutas lógica y física originales, los metadatos del archivo, el propietario,
el `Zone.Identifier` de origen de la descarga, los hashes y los problemas de integridad.

Este artefacto puede sobrevivir a los registros de eventos que registraron originalmente la detección o la corrección. También puede
conservar los bytes sospechosos exactos para realizar ingeniería inversa en un entorno aislado.

<span class="evidence-strength">Mejor conclusión:</span> Defender creó el registro de cuarentena de
la amenaza o el recurso indicados en la hora registrada. La etiqueta del producto no es una prueba independiente de malware,
y la cuarentena por sí sola no demuestra una ejecución previa.

## Ubicación y recopilación { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

Recopile todo el directorio `Quarantine`. Una entrada sin archivos de contenido o de registro coincidentes sigue siendo
un metadato útil, pero no permite obtener los bytes del archivo ni los hashes calculados.

## Seguridad y precauciones para el analista { #safety-and-analyst-cautions }

- Es probable que el contenido extraído sea malicioso. Utilice un sistema de análisis aislado y nunca lo ejecute.
- `ThreatName` es la firma o clasificación de Defender en ese momento. Son posibles los falsos positivos y las
  reclasificaciones posteriores.
- La hora de la cuarentena es la hora de corrección o de cuarentena, no necesariamente la de descarga, creación o ejecución.
- Un archivo en cuarentena puede haberse detectado antes de ejecutarse o después de haberse ejecutado.
- La ruta original y Zone.Identifier son valiosos, pero pueden estar ausentes o estar controlados por el atacante.
- Los registros huérfanos pueden deberse a la retención o la limpieza, a una recopilación incompleta o a daños.
- Juntei verifica los tamaños, los CRC, los identificadores y los hashes; revise `Problems` antes de confiar en los bytes recuperados.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

Por ejemplo, `tool.exe` se convierte en `tool.exe.quarantined`. Los flujos auxiliares incluyen su tipo en
el nombre, como `tool.exe.Zone.Identifier.quarantined`. Juntei también elimina el permiso de ejecución.
La extensión y los permisos reducen las ejecuciones accidentales, pero los bytes recuperados deben seguir
tratándose como malware.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Record` | `resource`, entrada sin recursos, `orphan-content` u `orphan-record`. Los huérfanos revelan vínculos o recopilaciones incompletos. |
| `QuarantineTime` | Hora de cuarentena o corrección de Defender en UTC, no hora de ejecución. |
| `ThreatName`, `ThreatID` | Clasificación de Defender e identificador numérico. Trátelos como veredicto o contexto del fabricante. |
| `ResourceType` | Archivo, clave del Registro, servicio, elemento de inicio, tarea programada u otra categoría de recurso. |
| `Path`, `PhysicalPath` | Ubicación lógica o física original. Pivote principal de correlación; conserve la ortografía exacta. |
| `FileSize` | Tamaño registrado por los metadatos del recurso. Compárelo con `ContentSize` y la MFT. |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | Metadatos capturados para el recurso en cuarentena; independientes de la hora de cuarentena y vulnerables al timestomping. |
| `Attr` | Atributos de archivo registrados. El estado oculto o de sistema puede aportar contexto útil. |
| `Owner` | SID del descriptor de seguridad recuperado; el propietario no es necesariamente el usuario que lo ejecutó. |
| `ZoneID`, `ReferrerURL`, `HostURL` | Valores de zona de descarga y de URL de `Zone.Identifier`. Pistas sólidas sobre el origen cuando están presentes; el ADS puede estar ausente o haberse eliminado. |
| `ContentSize` | Tamaño real del contenido principal recuperado. Compárelo con el `FileSize` registrado. |
| `MD5`, `SHA1`, `SHA256` | Calculados a partir de los bytes recuperados. Utilice SHA-256 para la identificación moderna del contenido; conserve todos por interoperabilidad. |
| `Streams` | Tipos, nombres y tamaños de los flujos recuperados; pueden incluir ADS, datos de seguridad o un hive del Registro. |
| `ResourceID` | Identificador que vincula los registros de metadatos y de contenido del recurso. |
| `EntryID` | GUID de la entrada de cuarentena; también vincula con el historial de Defender y las evidencias de eventos. |
| `ScanID` | Identificador de análisis de Defender para agrupar recursos y acciones. |
| `QuarantineFolder` | Carpeta raíz de origen y distinción del host. |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | Archivos de origen exactos utilizados para generar la fila. Imprescindibles para la validación. |
| `ExtractedFile` | Archivo principal en cuarentena escrito por `--extract`; vacío si no se produjo ninguna extracción principal. Los flujos auxiliares se escriben junto a él. `auto-parse -x` lo indica a partir de su directorio de salida (`./HOST1/SecurityProducts/DefenderQuarantined/…`). |
| `Problems` | Discrepancia de integridad, contenido ausente, desacuerdo de identificadores, daños o contexto de huérfano. No lo suprima. |

JSON/JSONL conserva el recurso estructurado, la lista de flujos, las entradas de registro y todos los detalles
de las relaciones, en lugar del texto concatenado del CSV.

## Comportamiento de la extracción { #extraction-behavior }

Juntei utiliza el nombre de archivo original con una extensión final `.quarantined` cuando la entrada de cuarentena
lo registra. Utiliza el identificador del contenido para los recursos huérfanos o que no son archivos, y añade el
identificador para resolver colisiones de nombres de archivo. Se sustituyen los caracteres no válidos en nombres de archivo de Windows. Los flujos de seguridad
y alternativos se escriben junto al archivo principal. Cada flujo de seguridad obtiene también un informe de texto
legible. Juntei elimina el permiso de ejecución;
el contenido existente distinto no se sobrescribe sin `-C`. Estos controles reducen los accidentes, pero no
hacen que los bytes sean seguros.

### Lectura del descriptor de seguridad recuperado { #reading-the-recovered-security-descriptor }

Un archivo `.security.quarantined` contiene metadatos de seguridad binarios de Windows, por lo que no se puede leer en un
editor de texto. Es un flujo de copia de seguridad independiente, no otra copia del ejecutable. Juntei lo decodifica
automáticamente al extraer, tanto con `parse-file defender-quarantine --extract` como con `auto-parse -x`:

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

El nombre del archivo de texto sigue el nombre real del binario, incluido cualquier identificador de colisión. El texto
generado es UTF-8 normal y utiliza las mismas protecciones contra sobrescritura que los demás archivos extraídos. Incluye
el nombre del archivo binario, el recuento de bytes y el SHA-256 para que el informe pueda emparejarse con su origen.

| Campo del informe | Cómo interpretarlo |
|---|---|
| Propietario y grupo principal | SID registrados en el descriptor. Los SID conocidos reciben nombres como `BUILTIN\Administrators` o `NT AUTHORITY\SYSTEM`. Los demás SID quedan sin resolver; un RID conocido por sí solo no basta para nombrar una cuenta local o de dominio. El propietario no demuestra quién creó o ejecutó un archivo. |
| Indicadores de control | Indicadores numéricos originales y sus nombres, incluidos la presencia de ACL, la herencia y la protección. |
| DACL | Entradas de control de acceso (ACE) ordenadas: tipo de permiso o denegación, SID de la cuenta, máscara de permisos, derechos e indicadores de herencia. Son entradas registradas, no un cálculo del acceso efectivo para el token de un usuario. |
| SACL | Entradas de auditoría o de integridad obligatoria registradas. Los indicadores de auditoría de éxito o error describen la configuración, no un historial de accesos. La ausencia de SACL no establece que toda la auditoría estuviera deshabilitada. |
| GUID de objeto | GUID opcionales para las ACE específicas de objeto, con los derechos específicos de objeto conservados en forma numérica en lugar de etiquetarse erróneamente como derechos de archivo. |
| SDDL | Representación en Security Descriptor Definition Language del propietario, el grupo y las entradas de ACL admitidas. Consulte los indicadores de control por separado. Juntei no emite una cadena SDDL parcial si algún componente está dañado o no se admite. |
| Estado de decodificación y problemas | `complete` o `incomplete`, con los motivos. Las ACE no admitidas conservan su hexadecimal sin procesar en el informe. El binario original se conserva siempre que la extracción se realiza correctamente. Los problemas de decodificación también aparecen en el campo `Problems` del registro de cuarentena. |

Una DACL ausente, una DACL NULL y una DACL vacía se notifican por separado. Una DACL ausente o NULL no proporciona
ninguna restricción de acceso discrecional; una DACL vacía no tiene ninguna ACE que conceda acceso. Las demás comprobaciones
de acceso y los privilegios de Windows siguen siendo relevantes. Las ACE heredadas y de solo herencia siguen marcadas explícitamente.

Por ejemplo, una entrada que concede control total heredado a Administrators puede aparecer así:

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

El decodificador funciona sin conexión en Windows, macOS y Linux sin API de Windows, Python ni búsqueda de cuentas.
Decodifica ACE básicas de permiso, denegación, auditoría y alarma, ACE de objeto y etiquetas de integridad obligatoria. Las ACE condicionales,
de devolución de llamada, de atributos de recursos y otras no admitidas se identifican explícitamente y se conservan en
forma sin procesar; sus condiciones de acceso no se evalúan. Un descriptor dañado no detiene la extracción
de los demás flujos.

## Estrategia de investigación { #investigation-strategy }

1. Empiece por `ThreatName`, `Path`, `QuarantineTime`, `EntryId` y los hashes.
2. Valide `Problems` y compare el tamaño registrado con el contenido recuperado.
3. Pivote con el SHA-256 hacia otros hosts y los datos de EDR, sandbox e inteligencia de amenazas.
4. Correlacione la ruta y la hora originales con `$MFT`, el Diario USN, Prefetch, Amcache y los registros de eventos.
5. Utilice los campos de URL y zona para investigar el acceso inicial, pero valídelos con evidencias del navegador, el proxy o el DNS.

## Más información { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [defender de Sootmark](https://github.com/Sootmark/defender)
- [Complemento Defender de dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: corrección y cuarentena del Antivirus de Microsoft Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft: descriptores de seguridad](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: formato de cadena del descriptor de seguridad](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: cadenas ACE](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
