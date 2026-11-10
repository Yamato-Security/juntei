# Boîtes de dialogue Ouvrir et Enregistrer sous (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## Qu'est-ce que l'historique des boîtes de dialogue ? { #what-the-dialog-history-is }

Lorsqu'un programme affiche la boîte de dialogue commune Ouvrir ou Enregistrer sous, Windows enregistre dans le
`NTUSER.DAT` de l'utilisateur :

- **quel fichier a été ouvert ou enregistré :** `OpenSavePidlMRU` possède une sous-clé par extension (`docx`,
  `ps1`, …) et `*` pour toutes les extensions, chacune étant une liste des éléments les plus récemment utilisés ;
- **quel programme a utilisé la boîte de dialogue, et le dossier dans lequel elle se trouvait :** `LastVisitedPidlMRU` (et
  `LastVisitedPidlMRULegacy` pour les programmes qui utilisent les anciennes boîtes de dialogue) ;
- **les programmes qui ont affiché la boîte de dialogue :** `CIDSizeMRU` (la taille de la boîte de dialogue par programme), et
  `FirstFolder` (le chemin d'un programme et un dossier) ;
- sous Windows 2000 et XP, `OpenSaveMRU` (chemins par extension) et `LastVisitedMRU` (programmes et
  leurs dossiers sous forme de chemins).

Depuis Windows Vista, les fichiers et dossiers sont des listes d'éléments du shell (shell items), les mêmes structures que la
cible d'un raccourci : chaque élément est une partie du chemin (Ce PC, un lecteur, un dossier, le fichier), et
un élément de fichier ou de dossier peut porter ses propres horodatages et sa référence de fichier NTFS.

<span class="evidence-strength">Meilleure conclusion :</span> un programme a utilisé la boîte de dialogue Ouvrir ou Enregistrer sous
pour ouvrir ou enregistrer ce fichier, ou dans ce dossier, au cours de la session de cet utilisateur. Seule l'entrée la plus récente
de chaque liste est datée : par l'heure de dernière écriture de sa clé.

## Emplacement { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

Collectez `NTUSER.DAT` avec ses journaux de transactions pour chaque profil.

## Précautions d'analyse { #analyst-cautions }

- **Un seul horodatage par liste.** L'heure de dernière écriture d'une clé change lorsque l'une de ses entrées passe en
  tête de sa liste ; elle ne date donc que l'entrée en position 0 (`IsMostRecent`, `LastUsedUTC`).
  Les autres entrées sont ordonnées, de la plus récente à la plus ancienne en descendant dans la liste, mais ne sont pas datées.
- **Le même fichier dans deux listes.** Un fichier figure dans la liste de son extension et dans `*` ; ne le comptez pas
  comme deux utilisations. `*` donne l'ordre toutes extensions confondues.
- **Ouvert ou enregistré.** Les listes ne précisent pas lequel. Corrélez avec les enregistrements `$MFT` et du Journal
  USN du fichier, les fichiers LNK et les Jump Lists.
- **Les horodatages des éléments sont ceux du fichier, et non de la boîte de dialogue.** `ItemModifiedUTC`, `ItemCreatedUTC` et
  `ItemAccessedUTC` sont les horodatages du fichier au moment où l'élément a été écrit, à 2 secondes près (FAT). Les éléments des
  dossiers connus (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …) n'en ont souvent aucun. Ces horodatages FAT
  n'ont pas de décalage de fuseau horaire. Le suffixe de colonne `UTC` est une convention d'affichage, et non une
  preuve que l'horloge d'origine était en UTC ; corrélez avec le fuseau horaire du système source et
  d'autres horodatages avant de les comparer aux FILETIME du Registre. Consultez les
  [recommandations de Microsoft sur les horodatages de fichiers](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times).
- **Les chemins sont des chemins du shell.** Ils commencent au bureau : `This PC\C:\Users\…`, un dossier connu, une
  bibliothèque (`UsersLibraries\Documents\…`) ou un emplacement réseau. Un chemin qui commence par un dossier
  ou un fichier se trouve sur le bureau. Un chemin vide correspond au bureau lui-même.
- **Éléments non décodés.** Un élément d'un type que le parseur d'éléments du shell ne connaît pas est affiché par sa
  classe (`[class 0x00]`), son type est `unknown`, et les octets de la liste figurent dans `DataRaw`.
- **Les programmes sont des noms.** `LastVisitedPidlMRU` et `CIDSizeMRU` conservent un nom de fichier
  (`WINWORD.EXE`), et non un chemin. `FirstFolder` conserve un chemin.
- **Les listes sont courtes et nettoyées.** Chaque liste conserve un nombre limité d'entrées, et les outils de nettoyage
  suppriment les clés. Utilisez `parse-registry read --recover-deleted` pour les valeurs supprimées.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

Le résumé répertorie les entrées par liste, les programmes qui ont utilisé les boîtes de dialogue et le fichier le plus
récent de chaque ruche (la première entrée de `*`).

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | L'hôte, le propriétaire du profil (d'après le chemin ou les dossiers du shell), le SID provenant du `ProfileList` du système, et la ruche. |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU` ou `LastVisitedMRU`. |
| `Extension` | La sous-clé de l'extension (`docx`, `*`) ; vide pour les autres listes. |
| `MRU-Position`, `IsMostRecent` | La position dans la liste de la clé (0 est la plus récente) ; vide lorsque la liste ne nomme pas la valeur. |
| `ValueName` | Le numéro (`MRUListEx`) ou la lettre (`MRUList`) de la valeur. |
| `Program` | Le programme qui a affiché la boîte de dialogue. |
| `Path` | Le fichier ou le dossier : les noms des éléments du shell joints, ou le chemin stocké sous forme de texte. |
| `ItemType` | Le type du dernier élément du shell : `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | Les horodatages du fichier ou du dossier, tels que son élément du shell les a enregistrés. |
| `MFT-Entry`, `MFT-Sequence` | La référence de fichier NTFS enregistrée par l'élément du shell. |
| `LastUsedUTC` | L'heure de dernière écriture de la clé, pour l'entrée la plus récente uniquement. |
| `KeyPath`, `KeyLastWrittenUTC` | La clé et sa dernière écriture : l'heure de son entrée la plus récente, et non de chaque ligne. |
| `DataRaw` | Les octets de la valeur (en hexadécimal) lorsqu'ils ne se décodent pas, ou la liste d'éléments du shell lorsque le type d'un élément est inconnu. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Valeurs que la liste nomme mais que la clé ne contient pas, valeurs qu'elle ne nomme pas, types incorrects, noms ou listes tronqués. |

## Stratégie d'investigation { #investigation-strategy }

1. Lisez `*` dans l'ordre pour chaque utilisateur, et `LastUsedUTC` pour le fichier le plus récent.
2. Recherchez des scripts, archives, exécutables et documents situés dans des emplacements accessibles en écriture par l'utilisateur, amovibles ou réseau,
   ainsi que des fichiers dont le nom évoque des identifiants ou des exports.
3. Utilisez `LastVisitedPidlMRU` pour relier un programme au dossier qu'il a utilisé en dernier, et `CIDSizeMRU` et
   `FirstFolder` pour les programmes qui ont utilisé les boîtes de dialogue.
4. Corrélez les chemins et `MftEntry` avec le `$MFT`, le Journal USN, les fichiers LNK, les Jump Lists,
   ShellBags et RecentDocs.

## Plus de détails { #further-details }

- [libyal winreg-kb : clés des éléments les plus récemment utilisés (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi : format Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
