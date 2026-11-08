# Outlook { #outlook }

## Que sont les fichiers d'Outlook ? { #what-outlooks-files-are }

Outlook pour Windows conserve les e-mails, les contacts, les calendriers et les tâches dans ses propres fichiers :

- **OST** (`.ost`) : la copie hors connexion d'une boîte aux lettres, qu'Outlook tient à jour pour les comptes
  Exchange et Microsoft 365, et depuis Outlook 2013 pour les comptes IMAP aussi. Ceux d'Outlook 2013 et versions
  ultérieures ont des pages de 4 Kio et des blocs compressés.
- **PST** (`.pst`) : une banque de messages personnelle : comptes POP3, archives (`archive.pst`), exportations et
  banques que créent les utilisateurs. Les PST ANSI sont ceux d'Outlook 97 à 2002 ; les PST Unicode ceux d'Outlook
  2003 et versions ultérieures.
- **MSG** (`.msg`) : un élément enregistré dans un fichier, lorsqu'il est glissé hors d'Outlook, enregistré avec
  **Enregistrer sous**, ou exporté par un autre programme. Les messages joints sont aussi des fichiers MSG.
- **OFT** (`.oft`) : un modèle Outlook : un fichier MSG à partir duquel de nouveaux éléments sont créés.

Chaque banque de messages contient des dossiers, et chaque dossier ses éléments : e-mails, demandes de réunion et
réponses, contacts et listes de distribution, éléments de calendrier, tâches, notes, entrées du journal et rapports.
Chaque dossier conserve aussi des éléments masqués, ses informations associées au dossier (FAI, folder associated
information) : affichages, règles, formulaires et paramètres d'Outlook (éléments `IPM.Configuration.*`, comme la liste
de saisie semi-automatique d'une boîte aux lettres Exchange). Les pièces jointes d'un élément peuvent être des
fichiers, des références à des fichiers, des objets OLE ou d'autres éléments (un message transféré en tant que pièce
jointe, les exceptions d'un rendez-vous périodique), qui ont eux-mêmes leurs propres pièces jointes.

<span class="evidence-strength">Meilleure conclusion :</span> la banque de messages ou le fichier contenait l'élément,
avec ces expéditeurs, destinataires, heures et pièces jointes, au moment de la collecte. Cela ne dit pas que
l'utilisateur l'a lu, écrit ou envoyé, ni que son expéditeur est bien celui qu'indique l'élément.

## Emplacement et collecte { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

Les PST peuvent se trouver n'importe où : les utilisateurs déplacent les archives vers d'autres dossiers, lecteurs et
partages de fichiers. Outlook conserve les chemins des banques de messages de chaque profil dans le Registre de
l'utilisateur. Les pièces jointes qu'un utilisateur a ouvertes depuis Outlook sont copiées dans un dossier de
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` (`Temporary Internet Files\Content.Outlook` avant
Windows 8.1), où elles peuvent subsister.

Outlook garde ses banques de messages ouvertes tant qu'il s'exécute, et un OST peut atteindre des dizaines de
gigaoctets. Collectez-les avec Outlook fermé, ou à partir d'une image disque ou d'un cliché instantané de volume.
Juntei lit chaque banque de messages en flux : il ne garde jamais une banque entière en mémoire.

## Précautions d'analyse { #analyst-cautions }

- Juntei lit de manière tolérante : il passe outre ce qui est endommagé, lit ce qui peut l'être et signale le reste
  comme des problèmes. Ceux-ci sont affichés (les 10 premiers de chaque fichier), comptés dans le résumé et dans la
  table des fichiers, et listés dans le champ `Problems` des éléments et pièces jointes qu'ils concernent. Un élément
  totalement illisible a tout de même une ligne dans la table des éléments, avec son `ItemID` et la raison.
- Les heures sont en UTC, à 100 nanosecondes près. `SubmitTime` est le moment où le client de l'expéditeur a soumis le
  message, selon l'horloge de cet ordinateur ; `DeliveryTime` celui où il est arrivé dans cette boîte aux lettres.
  `CreationTime` est le moment où cette copie de l'élément a été créée : remise dans la boîte aux lettres ou créée
  dans celle-ci, synchronisée dans l'OST, copiée ou importée dans le PST, ou enregistrée en tant que fichier MSG. Ce
  n'est pas le moment où le message a été rédigé. `LastModifiedTime` est la dernière modification de l'élément dans
  la banque de messages.
- Les `StartTime` et `DueTime` d'une tâche sont des dates dans le fuseau horaire de l'utilisateur, qu'Outlook stocke
  à minuit. Juntei les écrit sans décalage (`2017-02-19T00:00:00`), et `-Z` les laisse telles quelles.
- Outlook écrit minuit le 1er janvier 4501 pour **Aucune** dans certaines dates, comme l'échéance d'une tâche ou
  l'heure d'un rappel. Juntei l'écrit tel qu'il est stocké : `4501-01-01T00:00:00Z`, sans le `Z` dans les
  `StartTime` et `DueTime` d'une tâche.
- `Read` est l'indicateur de lecture au moment de la collecte de la banque de messages : quelqu'un, ou une règle, a
  marqué l'élément comme lu. Il ne dit ni qui ni quand.
- Les noms et adresses d'un expéditeur et d'un destinataire sont ce qu'ont enregistré le programme d'envoi et les
  serveurs de messagerie, et peuvent être falsifiés. Les adresses `EX` sont des noms uniques (distinguished names)
  Exchange (`/o=…/ou=…/cn=Recipients/cn=…`) ; l'adresse SMTP se trouve dans `SenderSMTP-Address` et `SMTP-Address`
  lorsque l'élément la stocke en plus. Pour vérifier d'où provient un message, lisez ses en-têtes Internet : le
  `0x007D001F` ou le `0x007D001E` (`PidTagTransportMessageHeaders`) de la table des propriétés.
- Les éléments supprimés dans Outlook se trouvent dans **Éléments supprimés** jusqu'à ce que ce dossier soit vidé, et
  sont lus comme les autres. Les éléments qui ne figurent plus dans la table d'aucun dossier, qu'une banque de
  messages peut conserver jusqu'à ce qu'elle réutilise leur espace, ne sont pas encore lus.
- `ItemID` est l'identifiant de nœud du message dans son PST ou son OST : le même élément peut en avoir un autre dans
  une autre copie de la banque de messages. Celui d'un élément incorporé est celui de son parent suivi de l'index de
  la pièce jointe (`2097348/0`, `0/0/1`) ; l'élément d'un fichier `.msg` ou `.oft` a l'identifiant `0`.
- `User` provient uniquement du chemin (le dossier qui suit `Users`), et se perd lorsqu'un fichier est copié ailleurs.
  Un PST peut appartenir à n'importe quel utilisateur.
- Le texte qui n'est pas en Unicode est lu dans la page de codes de l'élément (la sienne, celle de ses paramètres
  régionaux ou sa page de codes Internet). `--code-page` donne la page de codes des éléments qui n'en indiquent
  aucune : la page de codes ANSI du système qui les a écrits, `932` pour un système japonais.
- Le mot de passe d'un PST ne le chiffre pas : Juntei lit les PST protégés par mot de passe comme les autres.
  L'`Encryption` d'une banque de messages (`Permute`, le « chiffrement compressible » d'Outlook, ou `Cyclic`) ne fait
  qu'obscurcir ses blocs. Les messages chiffrés avec S/MIME conservent leur corps et leurs pièces jointes dans une
  pièce jointe chiffrée (`smime.p7m`), qui n'est pas déchiffrée.
- Les pièces jointes sont souvent des logiciels malveillants. `--extract-attachments` écrit leurs données dans des
  fichiers nommés d'après leur SHA-256, sans autorisation d'exécution ; ne les exécutez jamais, et conservez-les sur
  un système d'analyse isolé.

### Pas encore lus { #not-read-yet }

Les carnets d'adresses en mode hors connexion (OAB), le flux de saisie semi-automatique
(`Stream_Autocomplete_*.dat`) et les fichiers `.nk2`, les paramètres d'envoi/réception (`.srs`), les caches de
paramètres d'Outlook (RoamCache, Autodiscover), les bases de données SQLite d'Outlook, et les messages qui ne figurent
plus dans la table d'aucun dossier.

## Analyser avec Juntei { #parse-with-juntei }

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

`-d` recherche les fichiers se terminant par `.pst`, `.ost`, `.msg` et `.oft`, sans tenir compte de la casse. Quelle
que soit la façon dont un fichier est donné, ses premiers octets indiquent ce qu'il est : `!BDN` pour un PST ou un OST,
la signature de fichier composé pour un MSG ou un OFT. Tout le reste est signalé comme un fichier qui n'a pas pu être
analysé. Le code de sortie est 1 lorsqu'aucun fichier n'a pu être lu.

`--extract-attachments` écrit les données de chaque pièce jointe une seule fois, sous le nom `<SHA-256>` suivi de
l'extension de la pièce jointe lorsque celle-ci compte de 1 à 10 lettres et chiffres ASCII (`0c7d…1311.jpg`) ; aucun
nom provenant d'un élément n'est utilisé dans un chemin. Le répertoire doit être vide sauf si `-C` est indiqué ; avec
`-C`, un fichier du même nom dont le contenu est différent est remplacé.

## Tables de sortie { #output-tables }

Le format CSV crée huit fichiers, neuf avec `--properties`, chacun ne contenant que l'en-tête lorsqu'il n'a aucune
ligne. Les lignes JSON et JSONL comportent `Table` (`Files`, `Folders`, `Items`, `Recipients`, `Attachments`,
`Contacts`, `Calendar`, `Tasks` ou `Properties`) après `SourceFile`. Les lignes d'un élément le suivent, et celles de
ses éléments incorporés les suivent ; la ligne d'un fichier dans la table des fichiers vient après toutes ses autres
lignes.

| Fichier CSV | Contenu |
|---|---|
| `Outlook-Files.csv` | Une ligne par fichier lu : la banque de messages ou le fichier, et ce qu'il contient. |
| `Outlook-Folders.csv` | Les dossiers des PST et des OST. |
| `Outlook-Items.csv` | Tous les éléments : les messages des dossiers, leurs éléments masqués, les éléments des fichiers `.msg` et `.oft`, et les éléments incorporés dans leurs pièces jointes. |
| `Outlook-Recipients.csv` | Les destinataires des éléments. |
| `Outlook-Attachments.csv` | Les pièces jointes des éléments. |
| `Outlook-Contacts.csv` | Les champs des contacts et des listes de distribution. |
| `Outlook-Calendar.csv` | Les champs des éléments de calendrier et des messages de réunion. |
| `Outlook-Tasks.csv` | Les champs des tâches et des demandes de tâche. |
| `Outlook-Properties.csv` | Avec `--properties` : toutes les propriétés des banques de messages, dossiers, éléments, destinataires et pièces jointes, telles qu'elles sont stockées. |

## Champs { #fields }

Chaque table commence par ces champs :

| Champ | Interprétation |
|---|---|
| `SourceFile` | Le fichier dans lequel la ligne a été lue. |
| `User` | L'utilisateur Windows dans le profil duquel se trouve le chemin (le dossier qui suit `Users`). |

### Fichiers { #files }

| Champ | Interprétation |
|---|---|
| `FileType` | `PST`, `OST`, `MSG`, `OFT`, ou `CFB` pour un fichier composé lu comme un élément dont l'identifiant de classe n'est ni celui d'un message ni celui d'un modèle (de nombreux programmes autres qu'Outlook le laissent vide). |
| `Format` | La structure d'une banque de messages : `ANSI` (Outlook 97 à 2002), `Unicode` (Outlook 2003 et versions ultérieures) ou `Unicode 4K` (les OST d'Outlook 2013 et versions ultérieures) ; la version de fichier composé d'un fichier MSG ou OFT (`CFB v3`, `CFB v4`). |
| `Version`, `ClientVersion` | Les `wVer` et `wVerClient` de l'en-tête d'une banque de messages. |
| `Encryption` | La façon dont les blocs d'une banque de messages sont encodés : `None`, `Permute` ou `Cyclic`. |
| `DisplayName` | Le nom de la banque de messages (`Personal Folders`, une adresse e-mail). |
| `RecordKey` | L'identité de la banque de messages (hexadécimal), que contiennent les identifiants d'entrée de ses éléments. |
| `Folders` | Le nombre de dossiers de la banque de messages. |
| `Items` | Le nombre de messages de ses dossiers, ou 1 pour un fichier MSG ou OFT. |
| `HiddenItems` | Le nombre d'éléments masqués (informations associées au dossier) de ses dossiers. |
| `EmbeddedItems` | Le nombre d'éléments incorporés dans des pièces jointes, à tous les niveaux. |
| `Attachments` | Le nombre de pièces jointes de tous ces éléments. |
| `Problems` | Le nombre de problèmes trouvés dans le fichier. |

### Dossiers { #folders }

| Champ | Interprétation |
|---|---|
| `FolderID`, `ParentID` | L'identifiant de nœud du dossier, et celui de son dossier parent (vide pour le dossier racine). |
| `FolderPath` | Les noms des dossiers depuis la racine : `/Top of Personal Folders/Inbox`, `/Root - Mailbox/IPM_SUBTREE/Sent Items`. |
| `Name` | Le nom du dossier. |
| `ContainerClass` | Ce que contient le dossier : `IPF.Note` (e-mails), `IPF.Appointment`, `IPF.Contact`, `IPF.Task`, `IPF.Imap`, …. |
| `ContentCount`, `UnreadCount` | Le nombre de messages, et de messages non lus, que le dossier indique contenir. |
| `Messages`, `HiddenItems`, `Subfolders` | Les lignes de sa table des contenus, de sa table des contenus associés et de sa table de hiérarchie : ce que Juntei lit. Un écart avec `ContentCount` mérite un examen. |
| `CreationTime`, `LastModifiedTime` | La création du dossier et sa dernière modification (UTC), lorsque la banque de messages les conserve. |
| `Problem` | La raison pour laquelle le dossier n'a pas pu être ouvert ; ses sous-dossiers ne sont alors pas lus. |

### Éléments { #items }

| Champ | Interprétation |
|---|---|
| `FolderPath` | Le dossier de l'élément ; vide dans un fichier MSG ou OFT. |
| `ItemID` | L'identifiant de l'élément : son identifiant de nœud dans un PST ou un OST, `0` dans un fichier MSG ou OFT, l'identifiant de son parent suivi de l'index de la pièce jointe pour un élément incorporé (`2097348/0`). |
| `Hidden` | Si l'élément fait partie des informations associées au dossier : un élément masqué. |
| `EmbeddedIn` | L'identifiant de l'élément auquel il est joint ; vide pour les autres. |
| `ItemType` | Ce qu'est l'élément d'après sa classe de message : `Mail`, `MeetingMessage`, `Appointment`, `Contact`, `DistributionList`, `Task`, `TaskRequest`, `StickyNote`, `Journal`, `Post`, `Report`, `Configuration` ou `Other`. |
| `MessageClass` | La classe de message telle qu'elle est stockée (`IPM.Note`, `IPM.Note.SMIME`, `IPM.Schedule.Meeting.Request`, `REPORT.IPM.Note.NDR`, …). |
| `SubmitTime` | Le moment où le client de l'expéditeur a soumis le message (UTC, selon l'horloge de l'expéditeur). |
| `DeliveryTime` | Le moment où il a été remis dans cette boîte aux lettres (UTC). |
| `CreationTime`, `LastModifiedTime` | La création de cette copie dans la banque de messages ou le fichier, et sa dernière modification (UTC). |
| `Subject` | L'objet, avec son préfixe (`RE: `). |
| `SenderName`, `SenderAddressType`, `SenderAddress` | Qui l'a envoyé : le nom, le type d'adresse (`SMTP`, `EX`) et l'adresse de ce type. |
| `SenderSMTP-Address` | L'adresse SMTP de l'expéditeur, lorsqu'elle est stockée en plus d'une adresse d'un autre type. |
| `SentRepresentingName`, `SentRepresentingAddress` | Pour le compte de qui il a été envoyé (un délégué envoie pour le compte d'un responsable ; une boîte aux lettres partagée), avec son adresse SMTP lorsqu'elle est stockée, sinon l'adresse de son type. |
| `To`, `Cc`, `Bcc` | Les noms des destinataires tels que l'élément les affiche (voir la table des destinataires pour leurs adresses). `Bcc` ne figure que dans la copie de l'expéditeur. |
| `InternetMessageID`, `InReplyTo` | Le `Message-ID` du message, et celui du message auquel il répond. |
| `ConversationTopic` | L'objet sans son préfixe, que partagent les messages d'une conversation. |
| `Importance`, `Sensitivity` | `Low`, `Normal` ou `High` ; `Normal`, `Personal`, `Private` ou `Confidential`. |
| `Read` | Si l'élément est marqué comme lu. |
| `Flags` | Les indicateurs du message (`PidTagMessageFlags`, hexadécimal) : `0x1` lu, `0x2` non modifié, `0x4` soumis, `0x8` non envoyé (un brouillon), `0x10` contient des pièces jointes, `0x20` de moi, `0x40` masqué, `0x80` renvoyé, `0x400` déjà lu, `0x2000` provenant d'Internet. |
| `Size` | La taille de l'élément dans la banque de messages, pièces jointes comprises. |
| `Recipients`, `Attachments` | Le nombre de ses destinataires et de ses pièces jointes. |
| `Categories` | Ses catégories, séparées par un `;`. |
| `BodyPreview` | Le début de son corps en texte brut, espaces blancs réduits, 256 caractères au plus ; vide lorsqu'il n'a qu'un corps HTML ou RTF. |
| `Problems` | Ce qui n'a pas pu être lu dans l'élément (10 au plus, puis le nombre restant). |

### Destinataires { #recipients }

| Champ | Interprétation |
|---|---|
| `ItemID` | L'identifiant de l'élément. |
| `Index` | La ligne du destinataire dans la table des destinataires de l'élément, à partir de 0. |
| `RecipientType` | `To`, `Cc`, `Bcc`, `Originator`, ou la valeur telle qu'elle est stockée (hexadécimal). |
| `Name`, `AddressType`, `Address` | Le nom, le type d'adresse et l'adresse du destinataire. |
| `SMTP-Address` | L'adresse SMTP, lorsqu'elle est stockée en plus d'une adresse d'un autre type. |

### Pièces jointes { #attachments }

| Champ | Interprétation |
|---|---|
| `ItemID`, `Index` | L'identifiant de l'élément, et la ligne de la pièce jointe dans sa table des pièces jointes, à partir de 0. |
| `Method` | `ByValue` (un fichier), `ByReference`, `ByReferenceResolve`, `ByReferenceOnly` (un chemin vers un fichier), `EmbeddedMessage` (un élément), `OLE` (un objet OLE), `ByWebReference`, ou le nombre. |
| `Filename` | Le nom de la pièce jointe : son nom de fichier long, son nom court ou son nom d'affichage. |
| `Extension` | Son extension telle qu'elle est stockée (`.pdf`), sinon celle de son nom. |
| `MimeType`, `ContentID` | Son type MIME, et l'identifiant de contenu par lequel un corps HTML y fait référence. |
| `Inline` | Si le corps l'affiche (une image du corps HTML) au lieu de la lister. |
| `Hidden` | Si Outlook la masque dans la liste des pièces jointes. |
| `Size` | Sa taille telle que la donne la banque de messages, propriétés comprises. |
| `DataSize`, `SHA256` | La taille et le SHA-256 de ses données : le fichier, ou le fichier composé d'un objet OLE dans un PST ou un OST. Vide pour un élément incorporé et une pièce jointe par référence. |
| `Pathname` | Le chemin d'une pièce jointe par référence. |
| `CreationTime`, `LastModifiedTime` | Les heures de la pièce jointe (UTC) : souvent celles du fichier à partir duquel elle a été créée. |
| `EmbeddedSubject` | L'objet d'un élément incorporé, dont les propres lignes ont l'`ItemID` `<ItemID>/<Index>`. |
| `ExtractedFile` | Avec `--extract-attachments`, le fichier dans lequel ses données ont été écrites, dans ce répertoire. |
| `Problems` | Ce qui n'a pas pu être lu dans la pièce jointe, ou écrit. |

### Contacts { #contacts }

| Champ | Interprétation |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | Comme dans la table des éléments : `Contact` ou `DistributionList`. |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | Les noms du contact. |
| `FileUnder` | Le nom sous lequel le contact est classé. |
| `Company`, `Title`, `Department` | L'endroit où travaille le contact. |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | Les trois adresses e-mail du contact, chacune avec son nom d'affichage. |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | Les numéros de téléphone du contact. |
| `InstantMessaging` | L'adresse de messagerie instantanée du contact. |
| `CreationTime`, `LastModifiedTime` | Comme dans la table des éléments. |

### Calendrier { #calendar }

| Champ | Interprétation |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Comme dans la table des éléments : `Appointment` ou `MeetingMessage`. |
| `StartTime`, `EndTime` | Le début et la fin du rendez-vous (UTC) ; ceux de la première occurrence pour un rendez-vous périodique. |
| `DurationMinutes` | Sa durée en minutes. |
| `Location` | Son lieu. |
| `AllDay`, `Recurring` | S'il s'agit d'un événement d'une journée entière, et s'il est périodique (ses exceptions sont des éléments incorporés). |
| `BusyStatus` | `Free`, `Tentative`, `Busy`, `OutOfOffice` ou `WorkingElsewhere`. |
| `ResponseStatus` | La réponse de l'utilisateur : `None`, `Organized`, `Tentative`, `Accepted`, `Declined` ou `NotResponded`. |
| `Organizer` | Le nom de l'expéditeur de l'élément. |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | Les participants, tels que l'élément les liste. |
| `GlobalObjectID` | L'identité de la réunion d'une boîte aux lettres à l'autre (hexadécimal) : la même réunion dans les calendriers de l'organisateur et des participants. |
| `ReminderSet`, `ReminderTime` | Si un rappel est défini, et l'heure pour laquelle il l'est (UTC). |
| `CreationTime`, `LastModifiedTime` | Comme dans la table des éléments. |

### Tâches { #tasks }

| Champ | Interprétation |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Comme dans la table des éléments : `Task` ou `TaskRequest`. |
| `Status` | `NotStarted`, `InProgress`, `Complete`, `Waiting` ou `Deferred`. |
| `PercentComplete` | La part accomplie, de 0 à 100. |
| `StartTime`, `DueTime` | Son début et son échéance : des dates dans le fuseau horaire de l'utilisateur, sans décalage. |
| `CompletedTime` | Le moment où elle a été terminée (UTC). |
| `Complete` | Si elle est terminée. |
| `Owner`, `Assigner` | Son propriétaire, et qui l'a attribuée. |
| `ReminderSet`, `ReminderTime` | Comme dans la table du calendrier. |
| `CreationTime`, `LastModifiedTime` | Comme dans la table des éléments. |

### Propriétés { #properties }

| Champ | Interprétation |
|---|---|
| `ItemID` | L'identifiant de l'élément ; `store` pour les propriétés propres de la banque de messages ; `folder:<FolderID>` pour celles d'un dossier. |
| `Scope` | À qui appartient la propriété : `Store`, `Folder`, `Item`, `Recipient` ou `Attachment`. |
| `Index` | L'index du destinataire ou de la pièce jointe ; vide pour les autres. |
| `PropertyTag` | L'identifiant et le type de la propriété, sous la forme `0x0037001F` (`PidTagSubject`, une chaîne). |
| `PropertyName` | Pour une propriété nommée (identifiants à partir de `0x8000`), son jeu de propriétés et son nom : `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords`, ou le GUID du jeu lorsqu'il n'est pas un jeu bien connu. |
| `Type` | Le type de propriété : `PtypString`, `PtypInteger32`, `PtypTime`, `PtypBinary`, `PtypMultipleString`, …. |
| `Value` | La valeur : du texte pour les chaînes, ISO 8601 en UTC pour les heures (que `-Z` ne convertit pas), de l'hexadécimal pour les valeurs binaires (les 4 096 premiers octets, puis `…(N bytes)`), les valeurs d'une propriété à valeurs multiples sous forme de liste (séparées par `, ` en CSV), `(object)` pour un élément incorporé ou les données d'une pièce jointe, les octets en hexadécimal accompagnés d'une note pour une valeur d'un type qui n'est pas décodé, et, pour une valeur endommagée, ce qui a pu en être récupéré, suivi d'une note avec ses octets en hexadécimal (seulement les octets et la note si rien n'a pu être récupéré). |

## Analyse automatique { #automatic-parsing }

`auto-parse` écrit les tables dans la nouvelle catégorie `Communication` sous les noms `Outlook-Files.csv`,
`Outlook-Folders.csv`, `Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`,
`Outlook-Contacts.csv`, `Outlook-Calendar.csv` et `Outlook-Tasks.csv` (un seul fichier `Outlook.json` ou
`Outlook.jsonl` avec `-t json` ou `-t jsonl`), accompagnées de `Communication/Outlook-logs.txt`. Il transmet à
l'analyseur sa `--code-page`, et n'ajoute pas la table des propriétés ni n'extrait les pièces jointes : exécutez
`parse-file outlook` pour cela. La chronologie combinée `-j` inclut chaque ligne, avec `Comp` et l'identifiant
d'`Artifact` `outlook`. `-Z` convertit tous les champs se terminant par `Time`, sauf les `StartTime` et `DueTime`
d'une tâche.

## Plus de détails { #further-details }

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — format de fichier Personal Folder File (PFF)](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
