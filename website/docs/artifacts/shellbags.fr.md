# ShellBags { #shellbags }

## Que sont les ShellBags { #what-shellbags-are }

L'Explorateur mémorise la manière dont chaque dossier ouvert par un utilisateur est affiché : son affichage, la taille des icônes, l'ordre de tri et
la position de la fenêtre. Pour retrouver les paramètres d'un dossier, il conserve l'identité du dossier dans le
Registre de l'utilisateur sous la forme d'une arborescence d'éléments du shell (shell items), les mêmes éléments que contient la liste d'identifiants de cible d'un lien : un lecteur, puis chaque
dossier jusqu'à celui qui a été ouvert. Ces arborescences sont les ShellBags.

Elles couvrent les dossiers ouverts dans l'Explorateur et dans les boîtes de dialogue d'ouverture et d'enregistrement, y compris les dossiers situés sur des lecteurs
amovibles, des partages réseau, des appareils portables (téléphones, appareils photo), des archives ZIP et des serveurs FTP, ainsi que
les pages du Panneau de configuration. Elles subsistent après la disparition du dossier, du lecteur ou du partage réseau.

<span class="evidence-strength">Meilleure conclusion :</span> l'Explorateur de cet utilisateur a affiché ce dossier
(ou un dossier situé en dessous). Une ShellBag ne montre pas ce qui a été fait dans le dossier, et elle ne date pas
chaque visite : voir les heures ci-dessous.

## Emplacement { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

Une clé `BagMRU` contient des valeurs numérotées, chacune correspondant à un élément du shell, et une valeur `MRUListEx` donnant leur
ordre, le plus récemment ouvert en premier. Chaque valeur possède une sous-clé de même nom pour les éléments
situés en dessous ; un chemin correspond donc aux éléments allant de `BagMRU` jusqu'à une valeur. Le `NodeSlot` d'une sous-clé est le
numéro de sa clé `Bags\<N>`, où se trouvent les paramètres d'affichage du dossier.

Collectez `UsrClass.dat` et `NTUSER.DAT` avec leurs journaux de transactions (`.LOG1`, `.LOG2`) pour chaque
profil, ainsi que la ruche `SOFTWARE` du système, qui fournit le SID de l'utilisateur d'un `NTUSER.DAT`.

## Ce que dit chaque heure { #which-times-say-what }

- **Les heures de l'élément du shell** (`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`) sont les
  heures propres du dossier dans le système de fichiers (précision FAT : 2 secondes) au moment où l'Explorateur a écrit l'élément. Elles
  datent le dossier, et non la visite, et ne sont pas mises à jour lors des visites ultérieures.
  Les horloges FAT n'ont pas de décalage UTC stocké. Les intitulés hérités `UTC` et le suffixe `Z` affiché sont des
  conventions d'affichage, et non une preuve du fuseau horaire de la source. `auto-parse --timezone` conserve
  ces champs tels quels ; interprétez-les en tenant compte du contexte du système source.
- **L'heure de dernière écriture d'une clé** change lorsque son `MRUListEx` est réécrit, c'est-à-dire lorsque l'un de ses
  éléments est ouvert et placé en tête. Ainsi, l'heure de la clé contenant la valeur d'un élément
  (`ParentKeyLastWrittenUTC`) correspond à la dernière ouverture de l'élément **uniquement lorsqu'il est l'élément le plus récent
  de cette clé** (`IsMostRecentInParent` à true, `MruPosition` 0). Pour les autres éléments, il s'agit d'une
  heure ultérieure, celle de l'ouverture d'un élément frère.
- La clé propre d'un élément (`KeyLastWrittenUTC`) date de la même manière l'ouverture de son enfant le plus
  récent.

## Précautions d'analyse { #analyst-cautions }

- **L'absence prouve peu de chose.** L'Explorateur écrit les ShellBags lorsque l'affichage d'un dossier est enregistré ; un dossier
  vu seulement brièvement, ou ouvert d'une autre manière (ligne de commande, script, programme), peut ne pas en avoir.
  Un utilisateur peut supprimer les clés.
- **Une référence NTFS est celle du dossier** (`MftEntry`, `MftSequence`) au moment où l'élément a été écrit.
  Recherchez-la dans le `$MFT` pour voir si le même dossier existe toujours, ou si un autre fichier a réutilisé
  l'entrée.
- **Les lecteurs amovibles** sont nommés par leur lettre de lecteur. Il s'agit de la lettre qu'avait alors le lecteur ;
  faites-la correspondre avec l'historique USB et des périphériques montés pour identifier l'appareil.
- **Les noms proviennent de l'élément.** Un dossier renommé ultérieurement conserve ici son ancien nom. Les dossiers racine et les dossiers
  connus sont nommés comme les outils forensiques les nomment couramment (`This PC`, `Quick Access`, `Downloads`) ;
  `Guid` donne leur identifiant.
- **Les résultats de recherche, les bibliothèques et les pages du Panneau de configuration** sont des vues, et non des dossiers sur disque
  (`property_view`, `control_panel_item`, `control_panel_category`).
- **Un élément qu'aucun parseur ne connaît** conserve ses octets dans `DataRaw`, avec l'`ItemType` `unknown` et son
  octet de classe dans `ItemClass`. Les éléments endommagés ou inhabituels sont signalés dans `Problems`, et leurs octets
  sont également conservés.
- **Une ruche « sale » (dirty)** peut être dépourvue de ses éléments les plus récents sans ses journaux. Juntei rejoue les journaux situés à côté de la
  ruche, sauf avec `--no-logs` ; `TransactionLogStatus` indique si un enregistrement en provient.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

Les éléments du shell sont décodés avec ys-lnk-parser, le parseur d'éléments du shell de `parse-file lnk`. Le
résumé liste les derniers dossiers ouverts (du plus récent au plus ancien), d'après l'heure de dernière écriture de leur clé parente,
avec leurs utilisateurs.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | Hôte, nom du dossier de profil, SID de l'utilisateur (la clé racine d'un `UsrClass.dat` le nomme ; pour `NTUSER.DAT`, le `ProfileList` du système), la ruche et son type. |
| `BagKey` | La clé de l'élément, sous la racine de la ruche (`…\BagMRU\4\3\0`) : sa valeur se trouve dans la clé située au-dessus. |
| `MRU-Position`, `IsMostRecentInParent` | Sa place dans le `MRUListEx` de sa clé parente (0 : la plus récemment ouverte) ; vide lorsque la liste ne la contient pas. |
| `NodeSlot` | Le numéro de sa clé `Bags\<N>` (les paramètres d'affichage du dossier). |
| `Path` | Les éléments allant du sommet de son arborescence jusqu'à lui, joints par `\`. `[unknown]` représente un élément dont la valeur est manquante. |
| `ItemType`, `ItemClass` | La nature de l'élément (`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`), et son octet de classe. |
| `ItemName`, `GUID` | Son nom (le nom long d'un dossier), et l'identifiant de classe d'un dossier racine, d'un élément du Panneau de configuration ou d'un dossier délégué. |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **Provenant de l'élément du shell :** les heures du dossier au moment où l'élément a été écrit (FAT, précision de 2 secondes). |
| `MFT-Entry`, `MFT-Sequence` | **Provenant de l'élément du shell :** la référence de fichier NTFS du dossier. |
| `ExtensionBlocks` | Le nombre de blocs d'extension de l'élément. |
| `KeyLastWrittenUTC` | **Provenant du Registre :** la dernière écriture de la clé propre de l'élément : le moment où son enfant le plus récent a été ouvert. |
| `ParentKeyLastWrittenUTC` | **Provenant du Registre :** la dernière écriture de la clé contenant sa valeur : le moment de sa dernière ouverture, si `IsMostRecentInParent` vaut true. |
| `DataRaw` | Les octets de la valeur (hexadécimal), pour un élément qui n'est pas entièrement compris ou qui présente des problèmes. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Une valeur que le `MRUListEx` de sa clé nomme mais que la clé ne contient pas, une valeur que la liste ne nomme pas, une sous-clé sans sa valeur, une valeur qui n'est pas binaire, un `NodeSlot` de type incorrect, une liste d'éléments du shell endommagée. |

## Stratégie d'investigation { #investigation-strategy }

1. Listez les lecteurs amovibles (`volume` avec le `Guid` Removable Drives), les emplacements réseau, les dossiers
   FTP et les appareils portables, ainsi que les dossiers situés en dessous.
2. Recherchez les dossiers dignes d'intérêt : emplacements de préparation accessibles en écriture par l'utilisateur (`Temp`, `ProgramData`, `Public`),
   dossiers d'outils et d'archives, profils d'autres utilisateurs, et dossiers qui n'existent plus (vérifiez la
   référence NTFS dans le `$MFT`).
3. Ne datez les visites qu'avec le `ParentKeyLastWrittenUTC` des éléments dont `IsMostRecentInParent` vaut true ;
   corrélez le reste avec les fichiers LNK, les Jump Lists et RecentDocs, qui datent les ouvertures de fichiers.
4. Utilisez `parse-registry read --recover-deleted` pour les clés et valeurs `BagMRU` supprimées.

## Plus de détails { #further-details }

- [libyal libfwsi : format Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb : Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

La recherche du SID dans `ProfileList` nécessite un SID unique et syntaxiquement valide. Les correspondances ambiguës avec des dossiers de profil et les clés de sauvegarde telles que `S-1-...bak` laissent `UserSid` inconnu.
