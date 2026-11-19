# Index de Windows Search { #windows-search-index }

## Ce que l'index enregistre { #what-the-index-records }

Windows Search (le service `WSearch`) parcourt les profils utilisateur, le menu Démarrer, les boîtes aux lettres
d'Outlook, l'historique d'Internet Explorer, les fichiers hors connexion et tout ce qui entre dans son périmètre, et
conserve ce qu'il a lu dans `Windows.edb`, une base de données ESE. Deux de ses parties comptent pour une
investigation :

- le **magasin de propriétés** (property store ; `SystemIndex_PropertyStore`, ou `SystemIndex_0A` sous Windows 7),
  avec une ligne par élément indexé : son chemin, son nom, son type et sa taille, son propriétaire, les dates de
  création, de modification et d'accès que l'indexeur a copiées depuis le fichier, le moment où l'indexeur l'a lu,
  le titre et l'auteur d'un document, le début de son texte, le nom de l'ordinateur et des centaines d'autres
  propriétés ;
- les tables du **collecteur** (gatherer ; `SystemIndex_Gthr`, `SystemIndex_GthrPth`), avec le propre
  enregistrement du robot d'indexation pour chaque élément : son chemin URL, sa date de modification au moment de
  sa mise en file d'attente, le numéro et les indicateurs d'analyse, et un compteur de suppressions.

Un élément reste dans l'index après la disparition du fichier, du message ou de la page, jusqu'à ce que le robot
d'indexation remarque la suppression. L'index nomme donc des fichiers qui n'existent plus, avec leurs métadonnées et
un extrait de leur texte tels qu'ils étaient lors de leur indexation.

<span class="evidence-strength">Meilleure conclusion :</span> l'indexeur a lu l'élément au chemin enregistré,
avec les métadonnées enregistrées, à `GatherTime`, sur l'ordinateur nommé. L'index montre que le fichier existait
et ce qu'il contenait. Il ne montre pas qu'un utilisateur l'a ouvert.

## Emplacement et collecte { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

De Windows 7 à Windows 11 21H2, ainsi que dans les versions de Windows Server des mêmes années, l'index est
conservé dans `Windows.edb`. Windows 11 22H2 et les versions ultérieures le conservent dans des bases de données
SQLite du même dossier (`Windows.db`, `Windows-gather.db`, `Windows-usn.db`) ; Juntei les trouve, les nomme et les
signale comme le format plus récent non pris en charge.

Collectez l'intégralité du dossier `Applications\Windows` à partir d'une image disque, d'un cliché instantané de
volume, ou après avoir arrêté le service `WSearch` : la base de données est ouverte tant que le service s'exécute,
et ses modifications les plus récentes ne se trouvent que dans les journaux de transactions `MSS*.log`, que Juntei
ne rejoue pas. Un index copié depuis un système en cours d'exécution reste dans un état « dirty » ; la sortie le
signale et lit ce qui se trouve sur le disque. L'entrée par répertoire et `auto-parse` lisent chaque `Windows.edb`
à n'importe quelle profondeur ; utilisez `-f` pour un seul fichier.

## Précautions d'analyse { #analyst-cautions }

- `GatherTime` est le moment où l'indexeur a lu l'élément, pas celui où un utilisateur l'a créé, modifié ou ouvert.
  L'indexation s'exécute en arrière-plan, souvent de quelques minutes à quelques heures après une modification, et
  se met en pause lorsque l'ordinateur est occupé ou sur batterie.
- `DateCreated`, `DateModified` et `DateAccessed` sont des copies que l'indexeur a prises depuis le fichier. Elles
  sont rafraîchies lorsque l'élément est indexé de nouveau ; elles décrivent donc le fichier tel qu'il était à
  `GatherTime`.
- `ItemPath` est le chemin d'affichage. `ItemURL` et `GatherPath` sont les URL du robot d'indexation : `file:`
  pour les fichiers (avec le volume sous la forme `[GUID]` à partir de Windows 8), `iehistory://{SID}/` pour
  l'historique d'Internet Explorer, `mapi://` pour les éléments Outlook, `csc://{SID}/` pour les fichiers hors
  connexion et `winrt://{SID}/` pour le contenu des applications Windows.
- `User` est le profil nommé dans le chemin de l'élément (`C:\Users\<name>\…`). Il indique où se trouvait
  l'élément, pas qui l'a créé ; `FileOwner` est le propriétaire du fichier tel que l'indexeur l'a vu.
- `AutoSummary` est le début du texte de l'élément. Il peut citer un document supprimé ou chiffré depuis, et c'est
  un extrait tronqué, pas le document.
- Une base de données « dirty » peut contenir des pages que ses pages de branche n'atteignent pas encore. Juntei
  suit le chaînage propre des pages feuilles et indique dans le résumé combien il en a lu de cette façon. Ses
  modifications les plus récentes se trouvent toujours dans les journaux de transactions.
- Un élément présent dans les tables du collecteur sans ligne dans le magasin de propriétés (`InTables` vaut
  `Gthr`) a été mis en file d'attente ou analysé, et ses propriétés n'ont pas encore été écrites ou ont été
  retirées. Son chemin, son nom et ses dates de collecte sont toujours là.
- Windows 7 stocke les chaînes dans son propre encodage compressé et obscurci. Juntei le décode ; une valeur qui
  ne se décode pas reste en hexadécimal, avec une note dans `Problems`.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

CSV est le format de fichier par défaut. Avec `-t csv`, `-t json` ou `-t jsonl` sans `-o`, les enregistrements sont
écrits sur stdout. Sans aucune de ces options, seul un résumé s'affiche. Ajoutez `-C` pour écraser une sortie
existante. La sortie n'est jamais écrite par-dessus l'un des fichiers d'entrée, même avec `-C`.

Juntei écrit un enregistrement par élément indexé, en joignant le magasin de propriétés et les tables du collecteur
sur l'identifiant de l'élément. Le résumé compte les bases de données par schéma, celles qui sont « dirty », les
éléments présents dans les deux tables, dans le collecteur seulement et dans le magasin de propriétés seulement,
les dossiers, les éléments marqués comme supprimés, les éléments par magasin, les éléments sans date
stockée et les problèmes.

Les dommages n'interrompent pas l'exécution. Une base de données tronquée conserve les éléments que contiennent ses
pages restantes, et le résumé indique ce qui n'a pas pu être lu. Un fichier nommé `Windows.edb` qui n'est pas un
index Windows Search est signalé et ignoré. Les autres bases de données sont tout de même lues.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `DocumentID` | L'identifiant de l'élément dans l'index : le `WorkID` du magasin de propriétés (`DocID` sous Windows 7) et le `DocumentID` de la ligne du collecteur. |
| `InTables` | `Gthr+PropertyStore`, `Gthr` (un élément mis en file d'attente ou analysé, sans propriétés) ou `PropertyStore`. |
| `ItemPath` | `System_ItemPathDisplay` : le chemin tel que Windows l'affiche. Vide lorsque le magasin de propriétés n'a pas de ligne. |
| `ItemURL` | `System_ItemUrl` : l'URL du robot d'indexation pour l'élément (`file:`, `iehistory://`, `mapi://`, `csc://`, `winrt://`). |
| `GatherPath` | Le chemin du collecteur, assemblé à partir de l'arborescence des périmètres et du nom de fichier de la ligne. |
| `Name` | `System_ItemName`, ou le nom de fichier de la ligne du collecteur. |
| `ItemType`, `ItemTypeText` | `System_ItemType` (l'extension, ou `Directory`) et son nom d'affichage. |
| `Kind` | `System_Kind` : `document`, `folder`, `link`, `program`, `email`, `picture`, … ; plusieurs valeurs sont jointes par `\|`. |
| `MimeType` | `System_MIMEType`. |
| `Store` | `System_Search_Store` : `file`, `mapi` (Outlook), `iehistory`, `csc` (fichiers hors connexion) ou `winrt` (applications Windows). |
| `IsFolder`, `IsDeleted` | `System_IsFolder` et `System_IsDeleted`, tels que stockés. |
| `DeletedCount` | Le compteur de suppressions de la ligne du collecteur. |
| `Size` | `System_Size` en octets. Vide pour les dossiers, qui contiennent une valeur de substitution. |
| `FileAttributes` | `System_FileAttributes` sous forme de nombre (16 répertoire, 32 archive, 2 caché, …). |
| `FileOwner` | `System_FileOwner` (`DOMAIN\user`) tel que l'indexeur l'a lu. |
| `DateCreated`, `DateModified`, `DateAccessed` | Les dates du fichier telles que l'indexeur les a copiées (UTC). |
| `ItemDate` | `System_ItemDate` : la date propre de l'élément, comme la date d'envoi d'un message ou la date de prise de vue d'une photo ; pour un fichier, généralement sa date de modification à la seconde près. |
| `GatherTime` | `System_Search_GatherTime` : le moment où l'indexeur a lu l'élément (UTC). |
| `GatherLastModified` | Le `LastModified` de la ligne du collecteur : la date de modification de l'élément lorsque le robot d'indexation l'a mis en file d'attente (UTC). |
| `Title`, `Author` | `System_Title` et `System_Author`. |
| `AutoSummary` | `System_Search_AutoSummary` : le début du texte de l'élément. |
| `CompName` | `System_ComputerName` : l'ordinateur auquel l'index appartient. |
| `TargetURL`, `DateVisited` | L'URL et la date de visite d'un élément de l'historique d'Internet Explorer (`System_Link_TargetUrl`, `System_Link_DateVisited`). |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | Un élément de l'historique des activités de Windows 10 (`System_Activity_*`, `System_ActivityHistory_*`). |
| `User`, `UserSource` | Le profil nommé dans le chemin de l'élément, et le chemin dont il a été lu (`ItemPath`, `ItemURL` ou `GatherPath`). Vide en dehors d'un profil. |
| `ScopeID`, `SDID` | Les identifiants de périmètre et de descripteur de sécurité de la ligne du collecteur. |
| `CrawlNumber` | Le `CrawlNumberCrawled` de la ligne du collecteur. |
| `GatherFlags` | Les `TransactionFlags` de la ligne du collecteur. |
| `Priority` | La priorité d'analyse d'un élément encore en file d'attente. ESE la marque NULL une fois l'élément analysé ; elle est donc vide pour la plupart des éléments. |
| `PropertyCount` | Le nombre de propriétés que le magasin de propriétés contient pour l'élément. |
| `OtherProperties` | Toutes les autres propriétés, sous la forme `Name=Value` séparées par `; `. |
| `SourceFile` | La base de données lue, relative au répertoire d'entrée, ou son nom avec `-f`. |
| `Problems` | Valeurs qui n'ont pas pu être décodées, dates qui n'en sont pas, lignes en double et dommages. Ne les masquez pas. |

JSON et JSONL conservent les mêmes champs avec des valeurs typées : les nombres restent des nombres et les valeurs
manquantes valent `null`. À la place de `OtherProperties`, ils contiennent `Properties`, chaque propriété de
l'élément avec son nom, son identifiant numérique, son type de colonne et sa valeur, et, pour une date, son nombre
brut de ticks FILETIME à côté de l'heure qu'il représente. `Gather` contient les colonnes de la ligne du
collecteur.

## Comportement dans la chronologie { #timeline-behavior }

`auto-parse` enregistre les données dans `FileExistence/WindowsSearch.csv` (ou `.json`/`.jsonl`) et les ajoute à
la chronologie combinée `-j`. `super-timeline` crée des événements à partir de `DateCreated`, `DateModified` et
`DateAccessed` (les dates du fichier telles que Windows Search les a enregistrées), de `GatherTime` (le moment où
l'élément a été indexé), du `DateVisited` d'un élément de l'historique d'Internet Explorer et des
`ActivityStartTime` et `ActivityEndTime` d'une activité. `GatherLastModified` ne devient un événement que pour un
élément dont le magasin de propriétés ne contient aucune date de modification. Un élément sans aucune de ces dates
reste dans la sortie analysée et est compté comme non daté.

## Stratégie d'investigation { #investigation-strategy }

1. Filtrez `Store` sur `file` et cherchez dans `ItemPath`, `Name` et `AutoSummary` les fichiers, noms et mots de
   l'affaire. L'index nomme des fichiers supprimés depuis.
2. Lisez `GatherTime` comme le dernier moment où le fichier existait avec les métadonnées enregistrées, et
   `DateModified` comme sa date de modification à ce moment-là.
3. Utilisez `FileOwner` et `User` pour rattacher un fichier à un compte, puis confirmez avec la `$MFT`, le journal
   USN et `$Secure:$SDS`.
4. Pour les e-mails (`Store` `mapi`) et l'historique d'Internet Explorer (`iehistory`), pivotez vers les fichiers
   d'Outlook et le WebCache pour obtenir les enregistrements complets.
5. Comparez l'index entre les hôtes : `CompName` nomme l'ordinateur auquel l'index appartient, et un index
   collecté sur une image nomme les fichiers de cette image.

## Versions testées { #tested-versions }

Les tests utilisent des index publics dont d'autres outils ont lu les résultats de manière indépendante :

- L'index Windows 7 de plaso (`SystemIndex_0A`), vérifié par rapport à l'`esedbexport` de libyal, y compris les
  encodages de chaînes de Windows 7.
- L'index Windows 10 de DFIRArtifactMuseum, laissé dans un état « dirty », vérifié par rapport au SIDR de Stroz
  Friedberg, et son index Windows 11 (révision 200 du format ESE), vérifié par rapport à `esedbexport`.
- L'index SQLite de Windows 11 22H2 de DFIRArtifactMuseum, signalé comme le format plus récent non pris en charge.
- Des bases de données tronquées, vides ou d'un autre type.

## Plus de détails { #further-details }

- [Microsoft : vue d'ensemble de Windows Search](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft : Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb : format de fichier des bases de données ESE](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb : Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg : SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft : code source de l'Extensible Storage Engine](https://github.com/microsoft/Extensible-Storage-Engine)
