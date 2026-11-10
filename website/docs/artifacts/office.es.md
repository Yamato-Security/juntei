# Microsoft Office (archivos recientes, documentos de confianza, configuración de macros) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## Qué registra Office { #what-office-records }

Microsoft Office guarda, por usuario en `NTUSER.DAT` y por versión y aplicación:

- **Archivos y ubicaciones recientes:** `File MRU`, los documentos que abrió una aplicación, y
  `Place MRU`, las carpetas desde las que los abrió, cada elemento con la hora en que se abrió por
  última vez. Microsoft 365 también mantiene una lista por cada cuenta con sesión iniciada
  (`User MRU\<account>`).
- **Documentos de confianza:** cuando un usuario hace clic en *Habilitar edición* (*Enable
  Editing*) o *Habilitar contenido* (*Enable Content*) en un documento, Office lo recuerda en
  `TrustRecords`: la ruta del documento, cuándo se consideró de confianza y si se habilitaron sus
  macros.
- **Configuración de macros:** `VBAWarnings` (qué macros se ejecutan),
  `BlockContentExecutionFromInternet` (si se bloquean las macros de archivos procedentes de
  Internet), `AccessVBOM` (si los programas pueden modificar proyectos VBA) y las **ubicaciones de
  confianza**, carpetas cuyos documentos ejecutan macros sin preguntar. La directiva de grupo puede
  establecer lo mismo en `Software\Policies`.

Un documento malicioso normalmente necesita que el usuario habilite sus macros: un registro de
confianza con las macros habilitadas vincula el documento, el usuario y una hora.

<span class="evidence-strength">Mejor conclusión:</span> el usuario abrió este documento en esta
aplicación (por última vez a esta hora); el usuario habilitó la edición o las macros de este
documento a esta hora; Office estaba configurado para ejecutar macros de esta manera.

## Ubicación { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

Versiones: `16.0` es Office 2016 y posteriores (2019, 2021, 2024, Microsoft 365), `15.0` Office
2013, `14.0` Office 2010 y `12.0` Office 2007.

Un elemento tiene el aspecto
`[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx`: `T` es la hora de la
última apertura (un FILETIME en hexadecimal) y la ruta sigue a `*`. Un registro de confianza ocupa
24 bytes: el FILETIME en que el documento se consideró de confianza y, a continuación, sus
indicadores en los últimos 4 bytes: `0x7FFFFFFF` cuando se habilitaron sus macros y `0x00000001`
cuando solo se habilitó su edición.

## Precauciones para el analista { #analyst-cautions }

- **Horas.** `LastOpenedUTC` es la hora propia de cada elemento. `TrustedAtUTC` es el momento en
  que el usuario confió en el documento, cuando hizo clic en *Habilitar edición* o *Habilitar
  contenido*, que puede ser anterior a la apertura más reciente.
- **Macros habilitadas, no ejecutadas.** `MacrosEnabled` indica que el usuario habilitó las macros
  del documento; no prueba que se ejecutara una macro ni lo que hizo. Correlacione con la creación
  de procesos, Prefetch, los registros de PowerShell y el propio documento.
- **Rutas tal como las almacena Office.** Los registros de confianza designan el documento mediante
  una ruta que puede usar `%USERPROFILE%`, barras diagonales, una URL `file:///` y codificación URL
  (`%20`). Los elementos MRU de documentos en la nube son URL.
- **Valores predeterminados y directiva efectiva.** Office crea ubicaciones de confianza
  predeterminadas (`Templates`, `XLSTART`, `STARTUP`, …). El `Date` de una ubicación es una pista
  útil, no una prueba de quién la creó: los administradores, los scripts, las directivas y las
  configuraciones del Registro copiadas también pueden rellenarla. La ausencia de `VBAWarnings` por
  sí sola no determina la directiva de macros efectiva. También influyen la versión de Office, el
  marcado de archivos procedentes de Internet y otras opciones de administración.
  [Microsoft explica el bloqueo de macros de Internet](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked).
- **Documentos eliminados.** Las listas sobreviven a los documentos; un archivo que ya no existe
  puede seguir apareciendo aquí. Las listas son cortas y se vacían con la opción *Borrar elementos
  no anclados* (*Clear unpinned items*) de Office y con las herramientas de limpieza.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

El resumen enumera los documentos cuyas macros se habilitaron, las opciones que permiten ejecutar
macros con más libertad (`VBAWarnings` 1, `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1,
`AllowNetworkLocations` 1), las ubicaciones de confianza que añadieron los usuarios o que
estableció una directiva, y los últimos documentos abiertos.

## Campos de salida { #output-fields }

| Campo | Interpretación |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, el propietario del perfil (a partir de la ruta o de las carpetas del shell), el SID del `ProfileList` del sistema y el hive. |
| `OfficeVersion`, `App` | La clave de la versión (`16.0`) y la aplicación (`Word`, `Excel`, …). |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security` o `Trusted Locations`. |
| `Account` | La cuenta de Microsoft 365 de una lista `User MRU`. |
| `Policy` | Si la opción o la ubicación la establece una directiva. |
| `MRU-Position`, `IsMostRecent` | La posición de un elemento en su lista (`Item 1` es 0, el más reciente). |
| `Path` | El documento o la carpeta; la ruta de un documento de confianza tal como se almacena; el `Path` de una ubicación de confianza. |
| `LastOpenedUTC` | Cuándo se abrió el elemento por última vez. |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | Cuándo se consideró de confianza el documento, si se habilitaron sus macros y los indicadores del registro. Los indicadores desconocidos dejan `MacrosEnabled` vacío y generan un diagnóstico. |
| `Setting`, `Value` | Una opción de macros y su valor con su significado (`1 (all macros enabled)`); la clave de una ubicación de confianza (`Location3`) y sus demás valores (`AllowSubFolders=1; Date=…; Description=…`). |
| `KeyPath`, `KeyLastWrittenUTC` | La clave y su última escritura. |
| `DataRaw` | Los bytes del valor (en hexadecimal) cuando no se decodifican. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de la recuperación del Registro. |
| `Problems` | Elementos sin `*` o con una hora que no es válida, registros de confianza que no ocupan 24 bytes, ubicaciones sin `Path` y tipos incorrectos. |

## Estrategia de investigación { #investigation-strategy }

1. Empiece por `MacrosEnabled` verdadero: los documentos, cuándo y quién. Localice los archivos
   (`$MFT`, Diario USN, `Zone.Identifier`), su origen (historial del navegador y del correo) y lo
   que se ejecutó a continuación.
2. Revise la configuración de macros y las ubicaciones de confianza: todas las macros habilitadas,
   macros de Internet sin bloquear, acceso a proyectos VBA o una ubicación añadida por el usuario
   (en `Downloads`, `Temp`, un recurso compartido de red) debilitan los valores predeterminados.
3. Sitúe `LastOpenedUTC` en la línea de tiempo junto con los archivos LNK, las Jump Lists y
   RecentDocs para las rutas de los documentos, incluidas las ubicaciones extraíbles y de red.

## Más información { #further-details }

- [Microsoft: habilitar o deshabilitar macros en archivos de Microsoft 365](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: documentos de confianza](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: agregar, quitar o cambiar una ubicación de confianza](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
