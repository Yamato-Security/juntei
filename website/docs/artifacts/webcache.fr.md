# Internet Explorer et l'ancien Edge { #internet-explorer-and-legacy-edge }

## Qu'est-ce que la base de données WebCache ? { #what-the-webcache-database-is }

`WebCacheV01.dat` est la base de données des fonctions Internet de Windows (WinINet) dans le profil de chaque
utilisateur, une base de données ESE. Internet Explorer 10 et 11 y conservent leur historique, leurs téléchargements,
leur cache et leurs cookies, tout comme l'ancien Edge, l'Edge de Windows 10 d'avant le passage d'Edge à Chromium. Chaque
type d'enregistrement est un conteneur, une table à part entière :

- **Historique** (`History`, et `MSHist01…` pour un jour ou une semaine) : les URL visitées, chacune avec le titre de la
  page, le compte pour lequel elle a été enregistrée, la date de sa dernière visite et le nombre de visites. L'Explorateur de fichiers y
  enregistre aussi les fichiers locaux qu'il ouvre, sous forme d'URL `file:///`.
- **Téléchargements** (`iedownload`) : l'URL d'où provient un fichier, l'emplacement où il a été enregistré et celui où
  il a été écrit pendant le téléchargement, sa taille, le début du téléchargement, la page référente et le type MIME.
- **Cookies** : à partir de Windows 10, les cookies eux-mêmes (tables `CookieEntryEx`), avec leur hôte, leur nom, leur
  valeur et leurs heures ; auparavant, les fichiers de cookies qui les contenaient.
- **Cache** (`Content`) : les fichiers téléchargés pour afficher les pages, chacun avec son URL, l'emplacement de la
  copie sur le disque et les en-têtes de la réponse HTTP. D'autres programmes qui utilisent WinINet mettent aussi leurs
  téléchargements en cache ici.
- Le stockage web (`DOMStore`), les listes de compatibilité (`iecompat`) et d'autres.

La base de données survit aux navigateurs : Internet Explorer est retiré et l'Edge basé sur Chromium conserve ses propres
bases de données SQLite (voir [Navigateurs web](browser.md)), mais la base de données est toujours présente dans chaque
profil, avec ce qui a été enregistré auparavant, et ce que l'Explorateur de fichiers et d'autres programmes enregistrent
encore.

<span class="evidence-strength">Meilleure conclusion :</span> le profil de l'utilisateur a consigné l'URL, le
téléchargement ou le fichier mis en cache à l'heure indiquée ; la base de données ne dit pas qui était au clavier, ni
qu'un fichier mis en cache a été ouvert.

## Emplacement et collecte { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

Les fichiers que nomment les conteneurs (fichiers mis en cache, fichiers de cookies) se trouvent dans leurs propres
dossiers : `%LOCALAPPDATA%\Microsoft\Windows\INetCache\`, `INetCookies\`, `History\`, et sous
`%LOCALAPPDATA%\Packages\` pour les applications.

Collectez l'intégralité du dossier `WebCache`. Windows garde la base de données ouverte tant que l'utilisateur a une
session ouverte ; une copie prise sur un système en cours d'exécution est donc généralement dans un état « dirty » : ses
modifications les plus récentes ne se trouvent que dans les journaux de transactions `V01*.log`.

!!! warning "Journaux de transactions"
    Juntei ne rejoue pas les journaux de transactions ESE. Une base de données qui n'a pas été fermée proprement est
    comptée dans le résumé comme laissée dans un état « dirty » et signalée comme un problème : les modifications qui ne
    se trouvent encore que dans ses journaux sont absentes. Pour les inclure, rejouez d'abord les journaux dans une copie
    du dossier, avec la commande Windows `esentutl /r V01 /d` exécutée dans cette copie.

## Précautions d'analyse { #analyst-cautions }

- Les heures sont en UTC, à 100 nanosecondes près, sauf le `ModifiedTime` du conteneur d'historique d'un jour ou
  d'une semaine : il s'agit de l'heure locale du système, écrite sans décalage, et `-Z` la laisse telle quelle.
  Convertissez-la avec le fuseau horaire qu'avait le système source à ce moment-là. Son écart avec `AccessedTime` est
  souvent le décalage de ce fuseau horaire, mais pas toujours : les deux peuvent correspondre à des moments
  différents.
- Une entrée d'historique est une URL, et non une visite : `AccessedTime` est sa dernière visite et `AccessCount` le
  nombre de visites. Le conteneur `History` conserve une entrée par URL ; le conteneur d'un jour ou d'une semaine
  conserve celles de cette période, son nom donnant son premier jour et le jour qui suit son dernier
  (`MSHist012026110120261102` : 1er novembre 2026).
- Les URL `file:///` de l'historique sont des fichiers locaux ouverts depuis l'Explorateur de fichiers et d'autres
  programmes, et non des pages web.
- `Account` est le compte pour lequel une entrée a été enregistrée (`Visited: alice@…`) ; `User` est le dossier de
  profil dans lequel se trouve le chemin de la base de données. Ils peuvent différer : un dossier de profil ne porte pas
  toujours le nom de son compte, et le chemin d'une base de données copiée ailleurs ne contient pas de dossier de profil.
- `Package` indique à qui appartient une entrée : à l'ancien Edge (`microsoft.microsoftedge_8wekyb3d8bbwe`), à Internet
  Explorer en mode protégé amélioré (`windows_ie_ac_001`) ou à une autre application. Vide, l'entrée appartient à
  l'utilisateur lui-même : Internet Explorer, l'Explorateur de fichiers et d'autres programmes.
- Microsoft ne documente pas l'enregistrement de téléchargement. Juntei n'en lit l'heure de début et la taille que
  lorsque son en-tête tient dans l'enregistrement, et ses chemins et autres chaînes que lorsque les deux dernières
  sont une URL et un chemin Windows. Sinon, la ligne contient l'URL propre de l'entrée, et son heure d'accès lorsque le
  début ne peut pas être lu, avec un problème. `Publisher` est le nom enregistré par le navigateur, et non une
  vérification de la signature du fichier.
- Une entrée de cache indique que le fichier a été téléchargé et mis en cache. La copie sur le disque peut avoir
  disparu, avoir été remplacée ou n'avoir jamais été ouverte.
- Les entrées supprimées ne sont pas encore récupérées : ESE peut conserver un enregistrement supprimé dans ses pages
  jusqu'à ce qu'il les nettoie, mais Juntei ne lit que les enregistrements actifs.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` recherche `WebCacheV01.dat` par son nom ; `-f` lit une base de données quel que soit son nom.

## Tables de sortie { #output-tables }

Le format CSV crée cinq fichiers, chacun ne contenant que l'en-tête lorsqu'il n'a aucune ligne. Les lignes JSON et
JSONL comportent `Table` (`History`, `Downloads`, `Cookies`, `Entries` ou `Containers`) après `SourceFile`.

| Fichier CSV | Contenu |
|---|---|
| `WebCache-History.csv` | Les entrées des conteneurs d'historique. |
| `WebCache-Downloads.csv` | Téléchargements. |
| `WebCache-Cookies.csv` | Cookies (Windows 10 et versions ultérieures). |
| `WebCache-Entries.csv` | Les entrées de tous les autres conteneurs : fichiers mis en cache, fichiers de cookies, stockage web, listes de compatibilité et le reste. |
| `WebCache-Containers.csv` | Les conteneurs. |

## Champs { #fields }

Chaque table commence par ces champs :

| Champ | Interprétation |
|---|---|
| `SourceFile` | La base de données dans laquelle la ligne a été lue. |
| `User` | L'utilisateur Windows dans le profil duquel se trouve le chemin (le dossier qui suit `Users`). |
| `Package` | L'application à laquelle appartiennent les données de la ligne : le dossier sous `AppData\Local\Packages` du répertoire de son conteneur (ou de sa table de cookies). Vide pour les données propres de l'utilisateur. |

### Historique { #history }

| Champ | Interprétation |
|---|---|
| `Container` | Le conteneur : `History`, ou `MSHist01` suivi du premier jour de sa période et du jour qui suit son dernier. |
| `AccessedTime` | La dernière visite de l'URL (UTC) ; dans le conteneur d'un jour ou d'une semaine, la dernière de cette période. |
| `ModifiedTime` | La dernière modification de l'entrée (UTC) ; dans le conteneur d'un jour ou d'une semaine, l'heure locale du système, sans décalage. |
| `CreationTime` | La création de l'entrée (UTC). |
| `ExpiryTime`, `SyncTime` | Le moment où l'entrée doit quitter l'historique, et celui de sa dernière synchronisation (UTC). |
| `Account` | Le compte pour lequel l'entrée a été enregistrée. |
| `URL` | La page ou le fichier, sans le préfixe `Visited: account@`. |
| `Title` | Le titre de la page, tel que l'entrée l'a enregistré ; vide dans le conteneur d'un jour ou d'une semaine. |
| `AccessCount`, `SyncCount` | Le nombre de visites de l'URL, et de synchronisations de l'entrée. |
| `EntryType`, `Flags` | Le type et les indicateurs de l'entrée, tels qu'ils sont stockés. |
| `EntryID`, `ContainerID` | L'identifiant de l'entrée dans son conteneur, et celui du conteneur. |

### Téléchargements { #downloads }

| Champ | Interprétation |
|---|---|
| `StartTime` | Le début du téléchargement (UTC). |
| `URL` | L'origine du fichier. |
| `TargetPath`, `CurrentPath` | L'emplacement où le fichier a été enregistré, et celui où il a été écrit pendant le téléchargement : un fichier `.partial` pour l'ancien Edge, la copie du cache pour Internet Explorer. |
| `TotalBytes` | La taille du fichier. |
| `Referrer` | La page référente. |
| `MimeType` | Le type du fichier tel que servi (Internet Explorer). |
| `Publisher` | Le nom que l'ancien Edge a enregistré pour l'éditeur du fichier. |
| `AccessedTime` | L'heure d'accès propre de l'entrée (UTC), le plus souvent le début. |
| `DownloadID`, `ContainerID` | L'identifiant de l'entrée dans son conteneur, et celui du conteneur. |

### Cookies { #cookies }

| Champ | Interprétation |
|---|---|
| `LastModified`, `Expires` | Quand le cookie a été déposé, et quand il expire (UTC). |
| `Host`, `Name`, `Path` | Le site auquel le cookie est destiné, son nom et son chemin. |
| `Value` | La valeur telle qu'elle est stockée. |
| `Flags` | Les indicateurs du cookie, tels qu'ils sont stockés. |
| `Directory` | Le répertoire que la base de données attribue à la table de cookies : à qui appartiennent les cookies (`…\INetCookies\ESE\`, `…\MicrosoftEdge\Cookies\ESE\` pour l'ancien Edge, celui d'une application). |
| `CookieTable`, `EntryID` | La table (`CookieEntryEx_<n>`) et l'identifiant du cookie dans celle-ci. |

### Entrées { #entries }

| Champ | Interprétation |
|---|---|
| `Container` | Le conteneur : `Content` (fichiers mis en cache), `Cookies` (fichiers de cookies), `DOMStore` (stockage web), `iecompat`, …. |
| `AccessedTime` | La dernière utilisation de l'entrée (UTC). |
| `ModifiedTime` | La dernière modification de la ressource, telle que le serveur l'a indiquée (UTC). |
| `CreationTime` | La création de l'entrée (UTC). |
| `ExpiryTime`, `SyncTime` | Quand elle expire, et quand elle a été vérifiée auprès du serveur pour la dernière fois (UTC). |
| `URL` | L'URL telle qu'elle est stockée : une URL pour un fichier mis en cache, `Cookie:alice@example.com/` pour un fichier de cookie, `DOMStore:…`, `iecompat:…`. |
| `Account` | Le compte d'un fichier de cookie. |
| `Filename`, `FilePath` | Le fichier de l'entrée, et son emplacement : le répertoire du conteneur et le sous-dossier que nomme l'entrée. |
| `FileSize` | La taille du fichier. |
| `AccessCount`, `SyncCount` | Le nombre de fois où l'entrée a été utilisée, et vérifiée auprès du serveur. |
| `EntryType`, `Flags` | Le type et les indicateurs de l'entrée, tels qu'ils sont stockés. |
| `RedirectURL` | La destination vers laquelle l'URL a redirigé. |
| `ResponseHeaders`, `RequestHeaders` | Les en-têtes de la réponse HTTP (`HTTP/1.1 200 OK`, `Content-Type`, `Server`, …) et ceux de la requête, tels qu'ils sont stockés, lignes comprises. |
| `EntryID`, `ContainerID` | L'identifiant de l'entrée dans son conteneur, et celui du conteneur. |

### Conteneurs { #containers }

| Champ | Interprétation |
|---|---|
| `ContainerID`, `Name` | L'identifiant et le nom du conteneur. |
| `Directory` | L'emplacement de ses fichiers sur le disque. |
| `SecureDirectories` | Les sous-dossiers de `Directory` dans lesquels se trouvent ses fichiers (séparés par une virgule en CSV). |
| `PartitionID` | `M` pour les conteneurs propres de l'utilisateur, `L` pour ceux du mode protégé d'Internet Explorer (les dossiers `Low`), le SID d'un conteneur d'application pour ceux d'une application. |
| `LastAccessTime`, `LastScavengeTime` | La dernière utilisation du conteneur, et son dernier nettoyage (UTC). |

## Analyse automatique { #automatic-parsing }

`auto-parse` écrit les tables dans `BrowserActivity` sous les noms `WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv` et `WebCache-Containers.csv` (un seul fichier `WebCache.json` ou
`WebCache.jsonl` avec `-t json` ou `-t jsonl`), accompagnées de `BrowserActivity/WebCache-logs.txt`. La chronologie
combinée `-j` inclut chaque ligne, avec `Comp` et l'identifiant d'`Artifact` `webcache`. `-Z` convertit toutes les
heures, les champs se terminant par `Time`, `LastModified` et `Expires`, sauf le `ModifiedTime` local du conteneur
d'historique d'un jour ou d'une semaine.

## Plus de détails { #further-details }

- [Microsoft — À propos de WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — format de fichier des bases de données ESE](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — parseur MSIE WebCache](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
