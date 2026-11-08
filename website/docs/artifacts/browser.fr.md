# Navigateurs web { #web-browsers }

## Que sont les bases de données des navigateurs ? { #what-the-browser-databases-are }

Les navigateurs conservent leur historique, leurs téléchargements, leurs cookies et leurs entrées de formulaires dans des
bases de données SQLite, dans le profil de chaque utilisateur. Les navigateurs basés sur Chromium (Chrome, Edge, Brave,
Opera, Vivaldi) et ceux basés sur Firefox utilisent chacun leurs propres fichiers et tables, mais enregistrent les mêmes
types d'activité :

- **Historique** : les pages visitées, chaque visite avec son heure, la manière dont le navigateur est arrivé sur la page
  (saisie, lien, redirection, favori) et, dans Chromium, la durée pendant laquelle la page était au premier plan.
- **Téléchargements** : l'URL d'où provient un fichier (et les redirections en chemin), l'emplacement où il a été
  enregistré, sa taille, le début et la fin du téléchargement, et si celui-ci s'est terminé.
- **Cookies** : les sites qui les ont déposés, quand ils ont été déposés et envoyés pour la dernière fois, et quand ils
  expirent.
- **Historique des formulaires** : les valeurs saisies dans les formulaires web et les champs de recherche, combien de
  fois, la première et la dernière fois.

L'activité du navigateur est souvent la trace la plus directe de ce qu'a fait un utilisateur : la page d'hameçonnage qu'il
a suivie, l'outil qu'il a téléchargé, le webmail qu'il a utilisé, les termes de recherche qu'il a saisis.

<span class="evidence-strength">Meilleure conclusion :</span> le profil du navigateur a consigné une visite de l'URL à
l'heure indiquée, ou a enregistré le téléchargement sous le chemin indiqué ; la base de données ne dit pas qui était au
clavier.

## Emplacement et collecte { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` vaut `Default`, `Profile 1`, `Profile 2`, etc. Chromium a déplacé `Cookies` dans `Network` à la
version 96 ; les versions antérieures le conservent à côté de `History`. Jusqu'à la version 25, Firefox conservait ses
téléchargements dans `downloads.sqlite` ; les versions ultérieures les conservent dans `places.sqlite`.

Collectez tous les fichiers du profil, et chaque base de données avec les fichiers voisins qui partagent son nom :
`places.sqlite-wal`, `cookies.sqlite-wal`, `formhistory.sqlite-wal`, `History-journal`.

!!! warning "Journaux à écriture anticipée (WAL)"
    Firefox utilise ses bases de données en mode journal à écriture anticipée (WAL) : les modifications les plus
    récentes ne se trouvent que dans le fichier `-wal` jusqu'à ce que le navigateur les recopie dans la base de données
    (un point de contrôle, ou checkpoint). Juntei lit chaque base de données avec son fichier `-wal`, comme le ferait le
    navigateur. Sans lui, des visites, des cookies et des entrées de formulaires récents manquent, et les entrées
    supprimées dans le journal apparaissent encore comme actives.

## Entrées supprimées { #deleted-entries }

Lorsqu'un navigateur supprime l'historique, les lignes quittent les tables, mais leurs enregistrements peuvent subsister
dans l'espace libre de la base de données, dans sa liste de pages libres (freelist) ou dans des versions plus anciennes
de ses pages : dans le fichier `-wal`, ou dans le fichier de base de données lorsqu'un `-wal` a remplacé ces pages.
Juntei y récupère les visites, pages, téléchargements et entrées de formulaires supprimés, sous forme de lignes dont le
`Status` vaut `Deleted` :

- `Confidence` indique le degré de certitude de la correspondance entre l'enregistrement et sa table : `High` pour un
  enregistrement complet dont chaque valeur a le type de sa colonne, `Medium` ou `Low` pour des enregistrements
  endommagés ou des enregistrements compatibles avec plusieurs tables.
- `RecoveredFrom` indique où se trouvait l'enregistrement : la zone de la page (une cellule, un freeblock, de l'espace
  non alloué, une page libre), quelle page et quelle version de celle-ci, ainsi que le décalage de l'enregistrement dans
  la page. Les trames WAL sont numérotées à partir de 0.
- `LostColumns` liste les colonnes qu'un enregistrement endommagé ne contient plus : leurs champs sont vides.

Les navigateurs suppriment avec l'option `secure_delete` de SQLite : l'espace libéré dans la base de données est mis à
zéro. Ce qui peut être récupéré se trouve alors surtout dans des versions plus anciennes des pages : celles que le
fichier `-wal` contient encore, et les copies que le fichier de base de données garde des pages qu'un `-wal` a
remplacées, jusqu'au prochain point de contrôle. Copiez le fichier `-wal` avec sa base de données, et faites-le avant
que le navigateur ne soit relancé.

## Précautions d'analyse { #analyst-cautions }

- Les heures sont en UTC. Chromium et Firefox les enregistrent à la microseconde près ; l'historique des formulaires de
  Chromium, à la seconde près.
- Une visite ne signifie pas que l'utilisateur a saisi quelque chose ou cliqué : les redirections, les cadres (frames)
  et les pages ouvertes par d'autres programmes sont aussi des visites. Lisez `Transition`, `Typed` et `Hidden`.
- Chromium chiffre les valeurs des cookies : `Value` est vide pour celles-ci. Juntei ne les déchiffre pas.
- Une ligne `Removed` de l'historique des formulaires provient de la table `moz_deleted_formhistory` de Firefox, que
  Firefox pour Android remplit pour Sync. Firefox pour ordinateur la laisse normalement vide.
- Une entrée supprimée de confiance `Low` peut être un fragment d'un autre enregistrement : confrontez-la aux lignes
  actives et aux autres preuves.
- L'absence de ligne supprimée ne signifie pas que rien n'a été supprimé : un espace mis à zéro et un point de contrôle
  ne laissent rien à récupérer. Les cookies supprimés ne sont pas encore récupérés.
- Le contenu détermine la famille du navigateur. Un chemin de profil compatible précise `Browser` (Chrome, Firefox, etc.) ; un chemin contradictoire ne remplace pas la famille. Sinon, le libellé est `Chromium-based`, `Firefox-based` ou `Safari` (`Unknown` si indéterminé). `User` et `Profile` proviennent du chemin et peuvent être perdus lors de la copie.
- Les journaux de rollback (`-journal`) ne sont pas lus, pas plus que le `WebCacheV01.dat` d'Internet Explorer et de
  l'ancien Edge.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` recherche les fichiers par leur nom et identifie chacun par ses tables : un fichier `History` appartenant à un autre
programme est ignoré. `-f` lit une base de données quel que soit son nom.

## Tables de sortie { #output-tables }

Le format CSV crée quatre fichiers, chacun ne contenant que l'en-tête lorsqu'il n'a aucune ligne. Les lignes JSON et
JSONL comportent `Table` (`History`, `Downloads`, `Cookies` ou `FormHistory`) après `SourceFile`.

| Fichier CSV | Contenu |
|---|---|
| `Browser-History.csv` | Visites, actives et supprimées ; pages supprimées dont aucune page active n'a l'URL. |
| `Browser-Downloads.csv` | Téléchargements, actifs et supprimés. |
| `Browser-Cookies.csv` | Cookies. |
| `Browser-FormHistory.csv` | Entrées de formulaires, actives et supprimées ; enregistrements de suppression de Firefox. |

## Champs { #fields }

Chaque table commence par ces champs :

| Champ | Interprétation |
|---|---|
| `SourceFile` | La base de données dans laquelle la ligne a été lue. |
| `User` | L'utilisateur Windows dans le profil duquel se trouve le chemin (le dossier qui suit `Users`). |
| `Browser` | Famille déterminée par le contenu, précisée par un chemin de profil compatible. |
| `Profile` | Le profil du navigateur : le dossier de la base de données. |
| `Status` | `Live`, `Deleted` (récupéré) ou `Removed` (un enregistrement de suppression de Firefox). |

### Historique { #history }

| Champ | Interprétation |
|---|---|
| `RecordType` | `Visit`, ou `Page` pour une page supprimée dont les visites ont pu disparaître elles aussi. |
| `VisitTime` | Le début de la visite (UTC). |
| `URL`, `Title` | La page et son titre, tels que vus en dernier. |
| `Transition` | La manière dont le navigateur est arrivé sur la page. Chromium : le type principal, puis ses qualificatifs (`LINK`, `TYPED`, `AUTO_BOOKMARK`, `FORM_SUBMIT`, `RELOAD`, … avec `CHAIN_START`, `SERVER_REDIRECT`, `FROM_ADDRESS_BAR`, …) ; Firefox : `LINK`, `TYPED`, `BOOKMARK`, `EMBED`, `REDIRECT_PERMANENT`, `REDIRECT_TEMPORARY`, `DOWNLOAD`, `FRAMED_LINK`, `RELOAD`. |
| `VisitDuration` | Le nombre de secondes pendant lesquelles la page était au premier plan (Chromium ; 0 lorsque la durée n'a pas été mesurée). |
| `VisitCount` | Le nombre total de visites de la page. |
| `Typed`, `TypedCount` | Si l'URL a déjà été saisie, et combien de fois (Chromium). |
| `Hidden` | Masquée dans la liste de l'historique : sous-cadres, sources de redirection, erreurs. |
| `Frecency` | Le classement de la page par Firefox pour les suggestions de la barre d'adresse. |
| `VisitID`, `FromVisitID` | L'identifiant de ligne de la visite, et celui de la visite dont elle provient (la page référente, la source de la redirection). |
| `LastVisitTime` | La dernière visite d'une page supprimée (UTC). |
| `PageSource` | L'origine de l'URL et du titre d'une visite supprimée : `Live` (une page active), `Recovered` (une page supprimée) ou `NotFound`. |
| `Confidence`, `RecoveredFrom`, `LostColumns` | La manière dont une entrée supprimée a été récupérée ; voir [Entrées supprimées](#deleted-entries). |

### Téléchargements { #downloads }

| Champ | Interprétation |
|---|---|
| `StartTime`, `EndTime` | Le début et la fin du téléchargement (UTC). |
| `URL`, `RedirectChain` | L'origine du fichier, et chaque URL rencontrée en chemin (Chromium ; séparées par ` -> ` en CSV). |
| `TargetPath`, `CurrentPath` | L'emplacement où le fichier a été enregistré (un URI `file://` dans Firefox), et celui où il a été écrit pendant le téléchargement. |
| `ReceivedBytes`, `TotalBytes` | Les octets reçus, et la taille du fichier. |
| `State` | `Complete`, `InProgress`, `Cancelled`, `Interrupted`, `Paused`, `Blocked` ou `Dirty` (bloqué comme logiciel malveillant). |
| `DangerType` | Le verdict de Chromium sur le fichier : 0 non dangereux, 1 type de fichier dangereux, 2 URL dangereuse, 3 contenu dangereux, 4 peut-être dangereux, 5 peu courant, 6 conservé par l'utilisateur, 7 hôte dangereux, 8 potentiellement indésirable, … |
| `InterruptReason` | La raison de l'arrêt d'un téléchargement interrompu (Chromium ; 0 si aucune). |
| `Referrer`, `TabURL` | La page référente, et la page de l'onglet qui a lancé le téléchargement. |
| `MimeType`, `Opened` | Le type du fichier tel que servi, et s'il a été ouvert depuis le navigateur (Chromium). |
| `FileDeletedByUser` | Si l'utilisateur a supprimé du disque le fichier téléchargé depuis la liste des téléchargements du navigateur (Firefox). |
| `DownloadID` | L'identifiant de ligne du téléchargement. |

### Cookies { #cookies }

| Champ | Interprétation |
|---|---|
| `Created`, `LastAccessed`, `Expires` | Quand le cookie a été déposé et envoyé pour la dernière fois, et quand il expire (UTC ; `Expires` vide pour un cookie de session). |
| `Host`, `Name`, `Path` | Le site auquel le cookie est destiné, son nom et son chemin. |
| `Value` | La valeur telle qu'elle est stockée : vide lorsque Chromium l'a chiffrée. |
| `Secure`, `HttpOnly`, `Persistent` | Envoyé uniquement via HTTPS, masqué aux scripts, conservé d'une session à l'autre (Chromium). |
| `GoogleAnalytics` | Les champs décodés d'un cookie Google Analytics (`__utma`, `__utmb`, `__utmt`, `__utmz`) : heures de visite, sessions, ainsi que la source de la campagne et les termes de recherche. |
| `RowID` | L'identifiant de ligne du cookie. |

### Historique des formulaires { #form-history }

| Champ | Interprétation |
|---|---|
| `FirstUsed`, `LastUsed` | Quand la valeur a été saisie pour la première et pour la dernière fois (UTC). |
| `RemovedTime` | Le moment où l'entrée a été supprimée selon un enregistrement de suppression de Firefox (UTC). |
| `FieldName`, `Value` | Le champ du formulaire (`email`, `q`, la barre de recherche de Firefox : `searchbar-history`) et ce qui y a été saisi. |
| `Count` | Le nombre de fois où la valeur a été saisie. |
| `GUID` | L'identifiant de l'entrée chez Firefox, que nomment les enregistrements de suppression. |
| `Sources` | La provenance à partir de laquelle Firefox a enregistré l'entrée, dans les versions qui la conservent (séparées par une barre verticale en CSV). |
| `RowID` | Identifiant de ligne ; vide/null si perdu lors de la récupération. Un zéro réellement enregistré reste `0`. |

## Analyse automatique { #automatic-parsing }

`auto-parse` écrit les tables dans `BrowserActivity` sous les noms `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` et `Browser-FormHistory.csv` (un seul fichier `Browser.json` ou `Browser.jsonl` avec `-t json` ou
`-t jsonl`), accompagnées de `BrowserActivity/Browser-logs.txt`. La chronologie combinée `-j` inclut chaque ligne, avec
`Comp` et l'identifiant d'`Artifact` `browser`. `-Z` convertit `VisitTime`, `LastVisitTime`, `StartTime`, `EndTime`,
`Created`, `LastAccessed`, `Expires`, `FirstUsed`, `LastUsed` et `RemovedTime`.

## Plus de détails { #further-details }

- [Chromium — types de transition de page](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — types de danger des téléchargements](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — base de données Places](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — journalisation à écriture anticipée](https://www.sqlite.org/wal.html)
- [Plaso — parseurs SQLite des navigateurs](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
