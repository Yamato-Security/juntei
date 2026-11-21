# Outlook { #outlook }

## Qué son los archivos de Outlook { #what-outlooks-files-are }

Outlook para Windows guarda el correo, los contactos, los calendarios y las tareas en archivos propios:

- **OST** (`.ost`): la copia sin conexión de un buzón, que Outlook mantiene para las cuentas de Exchange y
  Microsoft 365, y desde Outlook 2013 también para las cuentas IMAP. Los de Outlook 2013 y versiones posteriores
  tienen páginas de 4 KiB y bloques comprimidos.
- **PST** (`.pst`): un almacén personal: cuentas POP3, elementos archivados (`archive.pst`), exportaciones y los almacenes
  que crean los usuarios. Los PST ANSI son los de Outlook 97 a 2002; los PST Unicode, los de Outlook 2003 y
  versiones posteriores.
- **MSG** (`.msg`): un elemento guardado como archivo, cuando se arrastra fuera de Outlook, se guarda con
  **Guardar como** o lo exporta otro programa. Los mensajes adjuntos también son archivos MSG.
- **OFT** (`.oft`): una plantilla de Outlook: un archivo MSG a partir del cual se crean elementos nuevos.

Cada almacén contiene carpetas, y cada carpeta sus elementos: correos electrónicos, convocatorias de reunión y
respuestas, contactos y listas de distribución, elementos de calendario, tareas, notas, entradas del diario e
informes. Cada carpeta guarda también elementos ocultos, su información asociada a carpetas (FAI, folder associated
information): vistas, reglas, formularios y la configuración de Outlook (elementos `IPM.Configuration.*`, como la
lista de Autocompletar de un buzón de Exchange). Los adjuntos de un elemento pueden ser archivos, referencias a
archivos, objetos OLE u otros elementos (un mensaje reenviado como adjunto, las excepciones de una cita periódica),
que a su vez tienen sus propios adjuntos.

<span class="evidence-strength">Mejor conclusión:</span> el almacén o el archivo contenía el elemento, con estos
remitentes, destinatarios, horas y adjuntos, cuando se recopiló. No indica que el usuario lo leyera, lo escribiera o
lo enviara, ni que su remitente sea quien el elemento dice.

## Ubicación y recopilación { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

Los PST pueden estar en cualquier lugar: los usuarios mueven los PST archivados a otras carpetas, unidades y recursos
compartidos de red. Outlook guarda las rutas de los almacenes de cada perfil en el registro del usuario.
Los adjuntos que un usuario abrió desde Outlook se copian en una carpeta de
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` (`Temporary Internet Files\Content.Outlook` antes de
Windows 8.1), donde pueden permanecer.

Outlook mantiene abiertos sus almacenes mientras se ejecuta, y un OST puede ocupar decenas de gigabytes. Recopílelos
con Outlook cerrado, o desde una imagen de disco o una instantánea de volumen. Juntei lee cada almacén como un flujo:
nunca mantiene un almacén completo en memoria.

## Precauciones para el analista { #analyst-cautions }

- Juntei lee de forma tolerante: pasa por alto lo que está dañado, lee lo que se puede leer y notifica el resto como
  problemas. Estos se muestran (los 10 primeros de cada archivo), se cuentan en el resumen y en la tabla de archivos,
  y se enumeran en el campo `Problems` de los elementos y adjuntos a los que afectan. Un elemento que no se puede leer
  en absoluto tiene igualmente una fila en la tabla de elementos, con su `ItemID` y el motivo.
- Las horas están en UTC, con precisión de 100 nanosegundos. `SubmitTime` es cuándo el cliente del remitente envió el
  mensaje, según el reloj de ese equipo; `DeliveryTime`, cuándo llegó a este buzón. `CreationTime` es cuándo se creó
  esta copia del elemento: al entregarse o crearse en el buzón, sincronizarse con el OST, copiarse o importarse en el
  PST, o guardarse como archivo MSG. No es cuándo se escribió el mensaje. `LastModifiedTime` es cuándo se modificó el
  elemento por última vez en el almacén.
- `StartTime` y `DueTime` de una tarea son fechas en la zona horaria del usuario, que Outlook almacena como
  medianoche. Juntei las escribe sin desplazamiento (`2017-02-19T00:00:00`), y `-Z` las deja tal cual.
- Outlook escribe la medianoche del 1 de enero de 4501 para **Ninguna** en algunas fechas, como la fecha de
  vencimiento de una tarea o la hora de un aviso. Juntei la escribe tal como está almacenada:
  `4501-01-01T00:00:00Z`, sin la `Z` en `StartTime` y `DueTime` de una tarea.
- `Read` es la marca de leído en el momento en que se recopiló el almacén: alguien, o una regla, marcó el elemento
  como leído. No indica quién ni cuándo.
- Los nombres y direcciones de un remitente y de un destinatario son lo que registraron el programa que lo envió y
  los servidores de correo, y pueden estar falsificados. Las direcciones `EX` son nombres distintivos (DN) de
  Exchange (`/o=…/ou=…/cn=Recipients/cn=…`); la dirección SMTP está en `SenderSMTP-Address` y `SMTP-Address` cuando
  el elemento la almacena además. Para comprobar de dónde procede un mensaje, lea sus encabezados de Internet: el
  `0x007D001F` o el `0x007D001E` (`PidTagTransportMessageHeaders`) de la tabla de propiedades.
- Los elementos eliminados en Outlook están en **Elementos eliminados** hasta que se vacía esa carpeta, y se leen
  como cualquier otro. Los elementos que ya no están en la tabla de ninguna carpeta, que un almacén puede conservar
  hasta que reutiliza su espacio, todavía no se leen.
- `ItemID` es el identificador de nodo del mensaje en su PST u OST: el mismo elemento puede tener otro en otra copia
  del almacén. El de un elemento incrustado es el de su elemento primario seguido del índice del adjunto
  (`2097348/0`, `0/0/1`); el elemento de un archivo `.msg` u `.oft` tiene `0`.
- `User` procede únicamente de la ruta (la carpeta que sigue a `Users`), y se pierde cuando un archivo se copia a
  otro lugar. Un PST puede ser de cualquier usuario.
- El texto que no está en Unicode se lee en la página de códigos del elemento (la suya, la de su configuración
  regional o su página de códigos de Internet). `--code-page` indica la página de códigos de los elementos que no
  indican ninguna: la página de códigos ANSI del sistema que los escribió, `932` para un sistema japonés.
- La contraseña de un PST no lo cifra: Juntei lee los PST protegidos con contraseña como cualquier otro. El valor
  `Encryption` de un almacén (`Permute`, el «cifrado comprimible» de Outlook, o `Cyclic`) solo ofusca sus bloques.
  Los mensajes cifrados con S/MIME guardan su cuerpo y sus adjuntos en un adjunto cifrado (`smime.p7m`), que no se
  descifra.
- Los adjuntos suelen ser malware. `--extract-attachments` escribe sus datos en archivos con el nombre de su
  SHA-256, con modo `0600` para archivos nuevos en Unix; no los ejecute nunca y consérvelos en un sistema de análisis aislado.

### Todavía no se leen { #not-read-yet }

Las libretas de direcciones sin conexión (OAB), el flujo de Autocompletar (`Stream_Autocomplete_*.dat`) y los
archivos `.nk2`, la configuración de envío y recepción (`.srs`), las cachés de configuración de Outlook (RoamCache,
Autodiscover), las bases de datos SQLite de Outlook y los mensajes que ya no están en la tabla de ninguna carpeta.

## Análisis con Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pst, *.ost, *.msg and *.oft
  -f, --file <FILE>      Parse one PST, OST, MSG or OFT file, whatever its name
      --code-page <CP>   Code page of items that don't give theirs (default 1252)

Output:
  -o, --output <PATH>              Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>         csv, json, or jsonl
      --properties                 Add the Properties table: every property as stored
      --extract-attachments <DIR>  Write each attachment's data once, named by its SHA-256
  -C, --clobber                    Overwrite files when saving or extracting
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl \
  | jq 'select(.Table == "Items" and .Hidden == false) | {DeliveryTime, SenderAddress, Subject}'
juntei parse-file outlook -f suspicious.msg -o msg-csv --properties --extract-attachments isolated/
```

`-d` busca nombres `.pst`, `.ost`, `.msg` y `.oft` sin distinguir mayúsculas y comprueba sus cabeceras: `!BDN` para PST u OST, o la firma de archivo compuesto para MSG u OFT. Se omiten archivos ajenos con esas extensiones, como los catálogos de mensajes Tcl. Los archivos ilegibles y los contenedores dañados con firma reconocible siguen generando errores. `-f` siempre valida el archivo indicado, independientemente del nombre, y comunica las entradas inválidas como errores. El estado de salida es 1 si no se puede leer ningún archivo.

`--extract-attachments` escribe los datos de cada adjunto una sola vez, como `<SHA-256>` con la extensión del
adjunto cuando esta tiene de 1 a 10 letras y dígitos ASCII (`0c7d…1311.jpg`); no se usa en una ruta ningún nombre
procedente de un elemento. El directorio debe estar vacío salvo que se indique `-C`; con `-C`, se sustituye un
archivo con el mismo nombre que contenga otra cosa.

## Tablas de salida { #output-tables }

CSV crea ocho archivos, nueve con `--properties`, cada uno solo con su encabezado cuando no tiene filas. Las filas
JSON y JSONL incluyen `Table` (`Files`, `Folders`, `Items`, `Recipients`, `Attachments`, `Contacts`, `Calendar`,
`Tasks` o `Properties`) después de `SourceFile`. Las filas de un elemento van a continuación de él, y las de sus
elementos incrustados a continuación de estas; la fila de un archivo en la tabla de archivos va después del resto de
sus filas.

| Archivo CSV | Contenido |
|---|---|
| `Outlook-Files.csv` | Una fila por archivo leído: el almacén o el archivo y lo que contiene. |
| `Outlook-Folders.csv` | Las carpetas de los PST y OST. |
| `Outlook-Items.csv` | Todos los elementos: los mensajes de las carpetas, sus elementos ocultos, los elementos de los archivos `.msg` y `.oft`, y los elementos incrustados en sus adjuntos. |
| `Outlook-Recipients.csv` | Los destinatarios de los elementos. |
| `Outlook-Attachments.csv` | Los adjuntos de los elementos. |
| `Outlook-Contacts.csv` | Los campos de los contactos y las listas de distribución. |
| `Outlook-Calendar.csv` | Los campos de los elementos de calendario y los mensajes de reunión. |
| `Outlook-Tasks.csv` | Los campos de las tareas y las solicitudes de tarea. |
| `Outlook-Properties.csv` | Con `--properties`: todas las propiedades de los almacenes, carpetas, elementos, destinatarios y adjuntos, tal como están almacenadas. |

## Campos { #fields }

Todas las tablas empiezan con estos campos:

| Campo | Interpretación |
|---|---|
| `SourceFile` | El archivo del que se leyó la fila. |
| `User` | El usuario de Windows en cuyo perfil está la ruta (la carpeta que sigue a `Users`). |

### Archivos { #files }

| Campo | Interpretación |
|---|---|
| `FileType` | `PST`, `OST`, `MSG`, `OFT`, o `CFB` para un archivo compuesto leído como elemento cuyo identificador de clase no es el de un mensaje ni el de una plantilla (muchos programas distintos de Outlook lo dejan vacío). |
| `Format` | La estructura de un almacén: `ANSI` (Outlook 97 a 2002), `Unicode` (Outlook 2003 y versiones posteriores) o `Unicode 4K` (los OST de Outlook 2013 y versiones posteriores); la versión de archivo compuesto de un archivo MSG u OFT (`CFB v3`, `CFB v4`). |
| `Version`, `ClientVersion` | Los campos `wVer` y `wVerClient` del encabezado de un almacén. |
| `Encryption` | Cómo están codificados los bloques de un almacén: `None`, `Permute` o `Cyclic`. |
| `DisplayName` | El nombre del almacén (`Personal Folders`, una dirección de correo electrónico). |
| `RecordKey` | La identidad del almacén (en hexadecimal), que contienen los identificadores de entrada de sus elementos. |
| `Folders` | El número de carpetas del almacén. |
| `Items` | El número de mensajes de sus carpetas, o 1 para un archivo MSG u OFT. |
| `HiddenItems` | El número de elementos ocultos (información asociada a carpetas) de sus carpetas. |
| `EmbeddedItems` | El número de elementos incrustados en adjuntos, en todos los niveles. |
| `Attachments` | El número de adjuntos de todos esos elementos. |
| `Problems` | El número de problemas encontrados en el archivo. |

### Carpetas { #folders }

| Campo | Interpretación |
|---|---|
| `FolderID`, `ParentID` | El identificador de nodo de la carpeta y el de su carpeta primaria (vacío para la carpeta raíz). |
| `FolderPath` | Los nombres de las carpetas desde la raíz hacia abajo: `/Top of Personal Folders/Inbox`, `/Root - Mailbox/IPM_SUBTREE/Sent Items`. |
| `Name` | El nombre de la carpeta. |
| `ContainerClass` | Lo que contiene la carpeta: `IPF.Note` (correo), `IPF.Appointment`, `IPF.Contact`, `IPF.Task`, `IPF.Imap`, …. |
| `ContentCount`, `UnreadCount` | Los mensajes, y los mensajes no leídos, que la carpeta dice tener. |
| `Messages`, `HiddenItems`, `Subfolders` | Las filas de su tabla de contenido, de su tabla de contenido asociado y de su tabla de jerarquía: lo que lee Juntei. Conviene revisar cualquier diferencia con `ContentCount`. |
| `CreationTime`, `LastModifiedTime` | Cuándo se creó la carpeta y cuándo se modificó por última vez (UTC), cuando el almacén los guarda. |
| `Problem` | Por qué no se pudo abrir la carpeta; en ese caso, sus subcarpetas no se leen. |

### Elementos { #items }

| Campo | Interpretación |
|---|---|
| `FolderPath` | La carpeta del elemento; vacío en un archivo MSG u OFT. |
| `ItemID` | El identificador del elemento: su identificador de nodo en un PST u OST, `0` en un archivo MSG u OFT, el identificador de su elemento primario y el índice del adjunto para un elemento incrustado (`2097348/0`). |
| `Hidden` | Si el elemento es información asociada a carpetas: un elemento oculto. |
| `EmbeddedIn` | El identificador del elemento al que está adjunto; vacío para los demás. |
| `ItemType` | Lo que es el elemento según su clase de mensaje: `Mail`, `MeetingMessage`, `Appointment`, `Contact`, `DistributionList`, `Task`, `TaskRequest`, `StickyNote`, `Journal`, `Post`, `Report`, `Configuration` u `Other`. |
| `MessageClass` | La clase de mensaje tal como está almacenada (`IPM.Note`, `IPM.Note.SMIME`, `IPM.Schedule.Meeting.Request`, `REPORT.IPM.Note.NDR`, …). |
| `SubmitTime` | Cuándo el cliente del remitente envió el mensaje (UTC, según el reloj del remitente). |
| `DeliveryTime` | Cuándo se entregó en este buzón (UTC). |
| `CreationTime`, `LastModifiedTime` | Cuándo se creó esta copia en el almacén o el archivo, y cuándo se modificó por última vez (UTC). |
| `Subject` | El asunto, con su prefijo (`RE: `). |
| `SenderName`, `SenderAddressType`, `SenderAddress` | Quién lo envió: el nombre, el tipo de dirección (`SMTP`, `EX`) y la dirección de ese tipo. |
| `SenderSMTP-Address` | La dirección SMTP del remitente, cuando está almacenada además de una dirección de otro tipo. |
| `SentRepresentingName`, `SentRepresentingAddress` | En nombre de quién se envió (un delegado envía en nombre de un responsable; un buzón compartido), con su dirección SMTP cuando está almacenada y, si no, la dirección de su tipo. |
| `To`, `Cc`, `Bcc` | Los nombres de los destinatarios tal como los muestra el elemento (sus direcciones están en la tabla de destinatarios). `Bcc` solo está en la copia del remitente. |
| `InternetMessageID`, `InReplyTo` | El `Message-ID` del mensaje, y el del mensaje al que responde. |
| `ConversationTopic` | El asunto sin su prefijo, que comparten los mensajes de una conversación. |
| `Importance`, `Sensitivity` | `Low`, `Normal` o `High`; `Normal`, `Personal`, `Private` o `Confidential`. |
| `Read` | Si el elemento está marcado como leído. |
| `Flags` | Las marcas del mensaje (`PidTagMessageFlags`, en hexadecimal): `0x1` leído, `0x2` sin modificar, `0x4` enviado, `0x8` sin enviar (un borrador), `0x10` tiene adjuntos, `0x20` de mí, `0x40` oculto, `0x80` vuelto a enviar, `0x400` leído alguna vez, `0x2000` procedente de Internet. |
| `Size` | El tamaño del elemento en el almacén, con sus adjuntos. |
| `Recipients`, `Attachments` | Cuántos destinatarios y adjuntos tiene. |
| `Categories` | Sus categorías, separadas por `;`. |
| `BodyPreview` | El comienzo de su cuerpo en texto sin formato, con los espacios en blanco contraídos, como máximo 256 caracteres; vacío cuando solo tiene un cuerpo HTML o RTF. |
| `Problems` | Lo que no se pudo leer en el elemento (como máximo 10 y, después, cuántos más). |

### Destinatarios { #recipients }

| Campo | Interpretación |
|---|---|
| `ItemID` | El identificador del elemento. |
| `Index` | La fila del destinatario en la tabla de destinatarios del elemento, desde 0. |
| `RecipientType` | `To`, `Cc`, `Bcc`, `Originator` o el valor tal como está almacenado (en hexadecimal). |
| `Name`, `AddressType`, `Address` | El nombre, el tipo de dirección y la dirección del destinatario. |
| `SMTP-Address` | La dirección SMTP, cuando está almacenada además de una dirección de otro tipo. |

### Adjuntos { #attachments }

| Campo | Interpretación |
|---|---|
| `ItemID`, `Index` | El identificador del elemento y la fila del adjunto en su tabla de adjuntos, desde 0. |
| `Method` | `ByValue` (un archivo), `ByReference`, `ByReferenceResolve`, `ByReferenceOnly` (una ruta a un archivo), `EmbeddedMessage` (un elemento), `OLE` (un objeto OLE), `ByWebReference` o el número. |
| `Filename` | El nombre del adjunto: su nombre de archivo largo, el corto o su nombre para mostrar. |
| `Extension` | Su extensión tal como está almacenada (`.pdf`) o, si no, la de su nombre. |
| `MimeType`, `ContentID` | Su tipo MIME y el identificador de contenido con el que un cuerpo HTML hace referencia a él. |
| `Inline` | Si el cuerpo lo muestra (una imagen del cuerpo HTML) en lugar de enumerarlo. |
| `Hidden` | Si Outlook lo oculta en la lista de adjuntos. |
| `Size` | Su tamaño según el almacén, con sus propiedades. |
| `DataSize`, `SHA256` | El tamaño y el SHA-256 de sus datos: el archivo, o el archivo compuesto de un objeto OLE en un PST u OST. Vacío para un elemento incrustado y para un adjunto por referencia. |
| `Pathname` | La ruta de un adjunto por referencia. |
| `CreationTime`, `LastModifiedTime` | Las horas del adjunto (UTC): a menudo las del archivo a partir del cual se creó. |
| `EmbeddedSubject` | El asunto de un elemento incrustado, cuyas propias filas tienen el `ItemID` `<ItemID>/<Index>`. |
| `ExtractedFile` | Con `--extract-attachments`, el archivo de ese directorio en el que se escribieron sus datos. |
| `Problems` | Lo que no se pudo leer en el adjunto, o escribir. |

### Contactos { #contacts }

| Campo | Interpretación |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | Como en la tabla de elementos: `Contact` o `DistributionList`. |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | Los nombres del contacto. |
| `FileUnder` | El nombre con el que se archiva el contacto. |
| `Company`, `Title`, `Department` | Dónde trabaja el contacto. |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | Las tres direcciones de correo electrónico del contacto, cada una con su nombre para mostrar. |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | Los números de teléfono del contacto. |
| `InstantMessaging` | La dirección de mensajería instantánea del contacto. |
| `CreationTime`, `LastModifiedTime` | Como en la tabla de elementos. |

### Calendario { #calendar }

| Campo | Interpretación |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Como en la tabla de elementos: `Appointment` o `MeetingMessage`. |
| `StartTime`, `EndTime` | Cuándo empieza y termina la cita (UTC); las de la primera repetición para una cita periódica. |
| `DurationMinutes` | Su duración en minutos. |
| `Location` | Dónde tiene lugar. |
| `AllDay`, `Recurring` | Si es un evento de todo el día y si es periódica (sus excepciones son elementos incrustados). |
| `BusyStatus` | `Free`, `Tentative`, `Busy`, `OutOfOffice` o `WorkingElsewhere`. |
| `ResponseStatus` | La respuesta del usuario: `None`, `Organized`, `Tentative`, `Accepted`, `Declined` o `NotResponded`. |
| `Organizer` | El nombre del remitente del elemento. |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | Los asistentes, tal como los enumera el elemento. |
| `GlobalObjectID` | La identidad de la reunión entre buzones (en hexadecimal): la misma reunión en los calendarios del organizador y de los asistentes. |
| `ReminderSet`, `ReminderTime` | Si hay un aviso establecido, y la hora para la que está (UTC). |
| `CreationTime`, `LastModifiedTime` | Como en la tabla de elementos. |

### Tareas { #tasks }

| Campo | Interpretación |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Como en la tabla de elementos: `Task` o `TaskRequest`. |
| `Status` | `NotStarted`, `InProgress`, `Complete`, `Waiting` o `Deferred`. |
| `PercentComplete` | Cuánto se ha completado, de 0 a 100. |
| `StartTime`, `DueTime` | Cuándo empieza y cuándo vence: fechas en la zona horaria del usuario, sin desplazamiento. |
| `CompletedTime` | Cuándo se completó (UTC). |
| `Complete` | Si está completada. |
| `Owner`, `Assigner` | Quién es su propietario y quién la asignó. |
| `ReminderSet`, `ReminderTime` | Como en la tabla del calendario. |
| `CreationTime`, `LastModifiedTime` | Como en la tabla de elementos. |

### Propiedades { #properties }

| Campo | Interpretación |
|---|---|
| `ItemID` | El identificador del elemento; `store` para las propiedades del propio almacén; `folder:<FolderID>` para las de una carpeta. |
| `Scope` | De quién es la propiedad: `Store`, `Folder`, `Item`, `Recipient` o `Attachment`. |
| `Index` | El índice del destinatario o del adjunto; vacío para los demás. |
| `PropertyTag` | El identificador y el tipo de la propiedad, como `0x0037001F` (`PidTagSubject`, una cadena). |
| `PropertyName` | Para una propiedad con nombre (identificadores a partir de `0x8000`), su conjunto de propiedades y su nombre: `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords`, o el GUID del conjunto cuando no es uno conocido. |
| `Type` | El tipo de propiedad: `PtypString`, `PtypInteger32`, `PtypTime`, `PtypBinary`, `PtypMultipleString`, …. |
| `Value` | El valor: texto para las cadenas, ISO 8601 en UTC para las horas (que `-Z` no convierte), hexadecimal para los valores binarios (los primeros 4096 bytes y, después, `…(N bytes)`), los valores de una propiedad multivalor como una lista (separados por `, ` en CSV), `(object)` para un elemento incrustado o los datos de un adjunto, los bytes en hexadecimal con una nota para un valor de un tipo que no se decodifica y, para un valor dañado, lo que se pudo recuperar de él seguido de una nota con sus bytes en hexadecimal (solo los bytes y la nota si no se pudo recuperar nada). |

## Análisis automático { #automatic-parsing }

`auto-parse` escribe las tablas en la nueva categoría `Communication` como `Outlook-Files.csv`, `Outlook-Folders.csv`,
`Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`, `Outlook-Contacts.csv`,
`Outlook-Calendar.csv` y `Outlook-Tasks.csv` (un único archivo `Outlook.json` o `Outlook.jsonl` con `-t json` o
`-t jsonl`), junto con `Communication/Outlook-logs.txt`. Pasa al analizador su `--code-page`, y no añade la tabla de
propiedades ni extrae los adjuntos: ejecute `parse-file outlook` para ello. La línea de tiempo combinada `-j` incluye
todas las filas, con `Comp` y el identificador de `Artifact` `outlook`. `-Z` convierte todos los campos que terminan
en `Time`, salvo `StartTime` y `DueTime` de una tarea.

## Más información { #further-details }

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — formato de archivo Personal Folder File (PFF)](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
