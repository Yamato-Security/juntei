# Corbeille { #recycle-bin }

## Ce que la Corbeille enregistre { #what-the-recycle-bin-records }

Lorsqu'un utilisateur supprime un fichier ou un dossier dans l'Explorateur de fichiers, Windows le déplace dans la Corbeille
de son volume et écrit à côté un petit enregistrement d'index : le chemin qu'il avait, sa taille et la date de sa
suppression. Chaque utilisateur possède un dossier de corbeille sur chaque volume, nommé d'après son SID ; le dossier
indique donc aussi quel compte a supprimé l'élément.

| Version de Windows | Index | Contenu supprimé |
|---|---|---|
| Vista et versions ultérieures | Un fichier `$I…` par élément (version de format 1 de Vista à 8.1, version 2 à partir de Windows 10) | `$R…`, avec le même nom après `$R` : le fichier, ou le dossier avec son contenu |
| NT 4.0, 2000, XP, Server 2003 | Un fichier `INFO2` par corbeille : enregistrements de 800 octets avec un chemin ANSI et un chemin Unicode | `D<drive letter><index><extension>`, par exemple `Dc1.txt` |
| 95, 98, ME | Un fichier `INFO2` (`INFO` sous Windows 95) : enregistrements de 280 octets avec un chemin ANSI uniquement | Les mêmes noms `D…` |

<span class="evidence-strength">Meilleure conclusion :</span> le compte dont la corbeille contient l'enregistrement a
déplacé l'élément situé au chemin d'origine vers la Corbeille à la date enregistrée. Une suppression par Maj+Suppr,
depuis la ligne de commande ou depuis la plupart des programmes contourne la Corbeille et ne laisse aucun enregistrement.

## Emplacements et collecte { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

Collectez les fichiers `$I` ou INFO2 de la corbeille de chaque volume. Collecter aussi les fichiers `$R` et `D…`
permet de voir quels éléments supprimés sont encore présents ; sans eux, `RecycledFileFound` vaut `false` pour tous
les éléments. Le contenu supprimé est une preuve à part entière et peut être un logiciel malveillant : traitez-le comme
tel.

## Précautions d'analyse { #analyst-cautions }

- `DeletedTime` est la date à laquelle l'élément a été déplacé dans la Corbeille, selon l'horloge système, sous forme de
  FILETIME UTC. Ce n'est pas la date à laquelle la Corbeille a été vidée. Les dates de création et de modification
  propres à l'élément se trouvent dans `$MFT` et dans le fichier `$R`.
- Le SID désigne le compte qui a supprimé l'élément, pas nécessairement le propriétaire du fichier. Un SID sans profil
  dans la `ProfileList` de la ruche SOFTWARE (compte supprimé, ou compte de domaine qui ne s'est jamais connecté) n'a
  pas de `User`.
- Lorsque la Corbeille est vidée, ou qu'un élément est restauré ou supprimé de la Corbeille, Windows Vista et versions
  ultérieures suppriment ses fichiers `$I` et `$R`. Leurs noms peuvent subsister dans le journal USN, dans `$MFT` et
  dans le slack de `$I30` ; Juntei n'en récupère pas les enregistrements `$I` supprimés. INFO2 conserve ces
  enregistrements et met à zéro le premier octet de leur chemin ANSI : `RemovedFromBin` vaut `true`, et Juntei rétablit
  ce caractère à partir du numéro de lecteur.
- Dans un fichier `$I`, `FileSize` est la taille de l'élément ; pour un dossier, la taille totale de son contenu. Dans
  INFO2, c'est l'espace que l'élément occupait sur le disque, en clusters entiers (4 096 octets pour un petit fichier).
- Le chemin ANSI d'INFO2 est souvent au format 8.3 et nécessite la page de codes du système : utilisez `--code-page 932`
  pour un Windows japonais, 936 ou 950 pour un Windows chinois. Le chemin Unicode, lorsqu'il existe, figure dans
  `OriginalPath`.
- Certains systèmes Windows Vista ont écrit des fichiers `$I` de 543 octets dont le champ de taille est trop court d'un
  octet. Juntei en lit la date et le chemin, laisse `FileSize` vide et signale un problème.
- `RecycledFileFound` décrit la collecte, pas le poste d'origine : `false` peut signifier que le collecteur n'a pas récupéré
  les fichiers `$R`, ou que l'élément n'est plus dans la Corbeille.
- Les corbeilles des volumes non système utilisent uniquement les profils du même ordinateur collecté, identifié par les métadonnées ou une structure connue de dossiers de lecteurs/volumes. Si l’attribution est ambiguë, l’utilisateur reste inconnu ; aucun profil d’un autre hôte n’est emprunté.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

Avec `-d`, Juntei lit les fichiers dont le nom est `$I` suivi de six lettres ou chiffres (avec une extension
quelconque), `INFO2`, ainsi que `INFO` dans un dossier `RECYCLED`. Avec `-f`, il lit le fichier quel que soit son nom.

## Champs de sortie CSV { #csv-output-fields }

| Champ | Signification |
|---|---|
| `SourceFile` | Le fichier `$I` ou INFO2. |
| `DeletedTime` | Date à laquelle l'élément a été déplacé dans la Corbeille (UTC). |
| `OriginalPath` | Emplacement d'origine de l'élément. Pour INFO2, le chemin Unicode lorsque l'enregistrement en contient un, sinon le chemin ANSI. |
| `FileSize` | `$I` : taille de l'élément en octets (taille totale pour un dossier). INFO2 : espace occupé sur le disque, en clusters entiers. Vide lorsqu'elle ne peut pas être lue. |
| `User`, `SID` | Le SID du dossier de corbeille, et l'utilisateur d'après lequel le profil correspondant de `ProfileList` est nommé. |
| `RecycledFile`, `RecycledFileFound` | Le chemin du contenu supprimé, à côté de l'index (`$R…`, `D<drive><index><extension>`), et sa présence ou non dans la collecte. |
| `Format`, `FormatVersion` | `$I` 1 (Vista à 8.1) ou 2 (Windows 10 et versions ultérieures) ; `INFO2` 0 (95), 2 (NT 4.0), 4 (98) ou 5 (ME, 2000 à Server 2003). |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2 : le numéro de l'enregistrement (celui de son nom de fichier `D…`), le lecteur, et la position à laquelle l'enregistrement commence dans INFO2. Un chemin UNC n'a pas de lecteur. |
| `RemovedFromBin` | INFO2 : `true` lorsque l'élément a été restauré ou supprimé de la Corbeille. |
| `ShortPath` | INFO2 : le chemin ANSI, lorsqu'il diffère de `OriginalPath` (souvent des noms 8.3). |
| `Problems` | Dommages : fichier tronqué ou trop long, numéro de lecteur invalide, chemin qui n'est pas de l'UTF-16 valide, date de suppression qui n'est pas une date valide. |

Les sorties JSON et JSONL comportent les mêmes champs, avec les nombres et les booléens typés, `null` pour les valeurs
qu'un enregistrement ne possède pas, et `Problems` sous forme de liste.

Si un répertoire de corbeille ne peut être listé ou si un nom de contenu est ambigu, `RecycledFileFound` reste inconnu (cellule CSV vide ou JSON `null`), avec la cause dans `Problems`. La sortie ne peut remplacer le contenu recyclé, même via un lien physique, ni être écrite dans un dossier recyclé. Les résultats existants sont conservés si aucun index n’est lisible.

## Analyse automatique { #automatic-parsing }

`auto-parse` écrit `FileExistence/RecycleBin.csv` (`RecycleBin.json` ou `RecycleBin.jsonl` avec
`-t json` ou `-t jsonl`), accompagné de `FileExistence/RecycleBin-logs.txt`. Son option `--code-page` s'applique à
INFO2. Depuis des images disque, les fichiers `$I` et INFO2 sont copiés ; les fichiers `$R` et `D…` sont des espaces
réservés vides, si bien que `RecycledFileFound` reste renseigné. La chronologie combinée `-j` inclut chaque ligne,
avec l'identifiant d'`Artifact` `recycle-bin`, `-Z` convertit `DeletedTime`, et
[`super-timeline`](../commands/super-timeline.md) produit un événement `Recycle Bin deletion (moved to the bin)`
par élément.

## Stratégie d'investigation { #investigation-strategy }

1. Triez par `DeletedTime` pour chaque utilisateur, et comparez avec les dates de l'activité sur laquelle vous enquêtez.
2. Recherchez `OriginalPath` dans les LNK, les Jump Lists, les ShellBags et RecentDocs pour savoir si l'élément a été
   utilisé avant sa suppression.
3. Dans le journal USN, la suppression apparaît comme un renommage vers `$Recycle.Bin` aux alentours de `DeletedTime` ;
   `$MFT` fournit les dates propres à l'élément.
4. Lorsque `RecycledFileFound` vaut `true`, calculez l'empreinte du contenu `$R` ou `D…` et examinez-le.
5. Si la Corbeille a été vidée, recherchez les noms `$I` et `$R` dans le journal USN et dans `$MFT`.

## Plus de détails { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
