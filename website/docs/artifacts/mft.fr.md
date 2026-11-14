# Table de fichiers principale (`$MFT`) { #master-file-table-mft }

## Qu'est-ce que la MFT { #what-the-mft-is }

NTFS représente chaque fichier et chaque répertoire par un enregistrement dans la table de fichiers principale (Master File Table). Les enregistrements contiennent
des attributs tels que les noms, les références au parent, les horodatages, les tailles, les indicateurs, les identifiants de sécurité,
les informations de point d'analyse (reparse) et les métadonnées des flux de données. Le contenu des petits fichiers et les flux de données alternatifs peuvent être
**résidents**, c'est-à-dire que leurs octets sont stockés à l'intérieur même de l'enregistrement MFT.

La MFT est fondamentale car elle couvre les enregistrements alloués et supprimés de l'ensemble du volume.
Elle peut montrer qu'un fichier a existé, où il était lié, s'il a été supprimé et comment ses métadonnées
ont changé — même lorsque la navigation ordinaire dans les répertoires ne l'affiche plus.

<span class="evidence-strength">Meilleure conclusion :</span> un enregistrement de fichier NTFS comportant les
métadonnées affichées existait au moment de la collecte ou a survécu dans un enregistrement non alloué. La MFT à elle seule ne
prouve ni une exécution ni une intention de l'utilisateur.

## Emplacement et collecte { #location-and-collection }

```text
C:\$MFT
```

`$MFT` est une métadonnée NTFS protégée. Collectez-la avec un outil d'acquisition forensique ou de triage capable
d'un accès NTFS brut ; une copie ordinaire en mode utilisateur ne peut normalement pas la lire. Associez-la à `$J`, `$LogFile`
et aux métadonnées du volume lorsque c'est possible.

## Précautions d'analyse { #analyst-cautions }

- Un enregistrement supprimé peut être partiellement écrasé, ou son numéro d'enregistrement peut être réutilisé.
- `SN` distingue les générations d'un même numéro d'entrée. Faites correspondre l'entrée **et** la séquence.
- Les horodatages de `$STANDARD_INFORMATION` (`0x10`) et de `$FILE_NAME` (`0x30`) sont mis à jour selon des
  comportements différents. Aucun des deux n'est à l'abri d'une manipulation.
- `SI<FN`, `uSecZeros` et `Copied` sont des pistes, et non des verdicts automatiques de timestomping.
- Les liens physiques (hard links) produisent plusieurs attributs `$FILE_NAME` et plusieurs chemins pour un même enregistrement de fichier.
- NTFS peut stocker plusieurs noms (y compris DOS 8.3), des flux de données alternatifs et des enregistrements d'extension.
- Les mises à jour de l'heure d'accès peuvent être désactivées, différées ou regroupées.
- Un chemin complet est reconstruit à partir des enregistrements parents. `[Orphaned]`, `[Unknown]` ou `PathUnknown`
  signalent une ascendance manquante/réutilisée/endommagée — et non un nom de répertoire littéral.
- Le **slack** d'un enregistrement FILE (ses octets au-delà de la taille utilisée) peut conserver des noms qu'il contenait auparavant : les
  entrées d'index d'un petit répertoire dont l'index a été déplacé vers une allocation d'index (la plupart d'entre elles y sont
  encore utilisées), un nom antérieur, ou l'occupant précédent d'un enregistrement réutilisé. Les noms récupérés
  n'ont pas de `$STANDARD_INFORMATION`, et l'en-tête d'entrée d'un nom peut avoir été écrasé. Lorsque la
  taille utilisée ou les attributs d'un enregistrement n'indiquent pas où commence son slack, rien n'en est récupéré
  et le champ `ParseErrors` de sa ligne indique `slack not read` ainsi que la raison.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams
  -R, --reconstruct-filesystem <DIR>  Recreate a tree; recovered content gets .dontrun
      --reconstruction-i30 <FILE>      Supplement the tree with an I30 file (repeatable)

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

N'exécutez jamais les flux résidents extraits avec `-e` ; ce n'est pas sans raison que Juntei leur attribue des noms en `.dontrun`.
Les colonnes de données résidentes de `-i` contiennent les mêmes octets : traitez-les comme un contenu non fiable.

`-D` affiche une entrée : son en-tête, chaque attribut (avec les data runs) et les noms présents dans son slack tels que
`-t json -x` les écrit, puis les octets de l'enregistrement. `-L` liste chaque nom d'un répertoire, utilisé ou
supprimé, avec son numéro d'entrée et de séquence, son type, sa taille et son heure de modification. Un répertoire est désigné par
son entrée (`30`, `0x1E`, `30-1`) ou par son chemin (`\Windows\System32`, `C:/Windows` ; insensible à la casse).

Cet exemple reconstruit des chemins consultables. Les contenus non résidents deviennent des fichiers vides. Détails : [Reconstruction du système de fichiers](../commands/auto-parse.md#filesystem-reconstruction).

```bash
juntei parse-ntfs mft -f '$MFT' -R reconstructed
```

## Champs de sortie CSV { #csv-output-fields }

| Champ | Interprétation |
|---|---|
| `EntryNum` (Entry Number) | Numéro d'enregistrement MFT. À combiner avec `SN` ; les numéros d'enregistrement sont réutilisés. |
| `SN` (Sequence Number) | Compteur de génération utilisé par les références NTFS. Une non-correspondance peut révéler des relations obsolètes/supprimées. |
| `InUse` | Indicateur d'allocation au moment de la collecte. `false` signifie que l'enregistrement est libre/supprimé, et non que tous les octets sont intacts. |
| `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number) | Référence du répertoire parent provenant de `$FILE_NAME`. Validez les deux parties. |
| `ParentPath` | Chemin parent reconstruit. Les marqueurs inconnu/orphelin révèlent une ascendance non résolue. |
| `File` (Filename) | Nom provenant du `$FILE_NAME` ou du flux de la ligne courante. Un enregistrement peut comporter plusieurs lignes. |
| `Ext` (Extension) | Extension extraite du nom à titre pratique ; ne constitue pas une détermination fiable du type de fichier. |
| `Size` (File Size) | Taille logique associée à la ligne/au flux. L'allocation éparse/compressée peut différer. |
| `RefCnt` (Reference Count) | Nombre de références de liens physiques NTFS ; comparez-le avec prudence au nombre de noms subsistants. |
| `ReparseTarget` | Cible décodée des points d'analyse pris en charge, tels que les jonctions/liens symboliques. Les chemins peuvent rediriger. |
| `Dir` (Directory) | L'enregistrement porte l'indicateur de répertoire. |
| `HasADS` (Has Alternate Data Streams) | L'enregistrement contient des flux de données alternatifs nommés. Cela ne le rend pas malveillant. |
| `IsADS` (Is Alternate Data Stream) | Cette ligne représente un flux de données alternatif nommé plutôt que les données sans nom du fichier. |
| `SI<FN` (Standard Information Before File Name) | La création SI précède une heure de création FN différente. Examinez la copie/le timestomping et les causes légitimes. |
| `uSecZeros` (Microsecond Zeros) | L'heure de création/modification SI tombe exactement sur une limite de milliseconde. Simple heuristique faible de timestomping. |
| `Copied` | L'heure de modification précède l'heure de création. Souvent cohérent avec une copie de fichier ; ne prouve pas une action de l'attaquant. |
| `SI-Flags` (Standard Information Flags) | Attributs/indicateurs de fichier de `$STANDARD_INFORMATION`. Les indicateurs caché/système peuvent être des pivots utiles. |
| `Type` (Name Type) | Espace de noms de `$FILE_NAME` (Win32, DOS, POSIX ou combiné). À utiliser pour éviter de compter deux fois les noms. |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | Heures SI. Champs courants des chronologies, mais les outils en mode utilisateur peuvent en modifier plusieurs. |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | Heures FN. Mises à jour différemment et généralement plus difficiles à modifier par les API ordinaires ; elles ne sont pas pour autant immuables. |
| `USN` (Update Sequence Number) | Dernier USN associé à l'attribut SI de l'enregistrement. Pivotez vers `$J` si cet USN subsiste. |
| `LogfileSN` (Logfile Sequence Number) | Numéro de séquence `$LogFile` NTFS provenant de l'en-tête de l'enregistrement FILE ; utile pour l'analyse des transactions de bas niveau. |
| `SID` (Security ID) | Index dans `$Secure` de NTFS ; ce n'est pas en soi un SID Windows. Joignez-le au `SecurityID` de [`parse-ntfs secure-sds`](secure-sds.md) pour obtenir le propriétaire et les ACL du fichier. |
| `ObjectID-FileDroid` | Identifiant d'objet / identifiant de suivi des liens distribués, s'il est présent ; à corréler avec les droids des LNK/Jump Lists. |
| `LoggedUtilStream` | Contexte décodé de `$LOGGED_UTILITY_STREAM`, lorsqu'il est pris en charge. Examinez les valeurs inhabituelles. |
| `ZoneID-Contents` | Contenu résident de `Zone.Identifier`, tel que ZoneId et les URL de téléchargement. Il peut être absent, supprimé ou non résident. |
| `SourceFile` | Le `$MFT` d'entrée, indispensable lorsque l'on combine des hôtes ou des volumes. |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | Avec `-i` : le contenu du `$DATA` résident de la ligne (celui du fichier ou du flux de données alternatif), en Base64, sous forme d'octets (`41-42`) et sous forme de texte lorsqu'il s'agit d'ASCII imprimable. Vide lorsque les données sont non résidentes ou filtrées par `-E`/`-M`. |
| `ParseErrors` | Dommages ou relations non prises en charge affectant cette ligne. N'écartez jamais cette colonne lors du filtrage. |
| `FromSlack` | Avec `-x` : `True` pour un nom récupéré dans le slack d'un enregistrement FILE. Ses `EntryNum` et `SN` désignent le fichier vers lequel pointait le nom (vides lorsque son en-tête d'entrée a été écrasé), ses horodatages récupérés figurent dans les colonnes `(FN)`, et `InUse` vaut `False`. |
| `SlackRecord`, `SlackOffset` | Avec `-x` : l'enregistrement FILE dont le slack contenait le nom, identifié par sa place dans le `$MFT` plutôt que par le numéro d'enregistrement écrit à l'intérieur (celui propre à un enregistrement d'extension, même si ses noms sont listés avec son enregistrement de base), et le décalage en octets du nom dans le `$MFT`. Les `ParseErrors` d'une ligne de slack sont les problèmes de cet enregistrement, comme `fixup values do not match` : les octets récupérés peuvent en être affectés. |
| `CopyOfEntryInUse` | Avec `-x` : un fichier utilisé porte encore ce nom dans le même répertoire (et il s'agit du même fichier, lorsque la référence a subsisté). Les noms qu'aucun fichier utilisé ne porte sont les plus intéressants : supprimés, renommés, ou occupant précédent d'un enregistrement réutilisé. |

Les colonnes d'horodatage suivent l'ordre MACB (Modified, Accessed, Changed, Birth/création) :
`DataModified` SI/FN, `Accessed` SI/FN, `MetadataChanged` SI/FN, puis `Created` SI/FN.

`(SI)` signifie `$STANDARD_INFORMATION` ; `(FN)` signifie `$FILE_NAME`. Ces noms décrivent la
source habituelle pour un enregistrement complet. Pour une ligne de nom de fichier normale sans attribut SI, le
mécanisme de repli existant place la création FN dans `Created (FN)` et les valeurs FN de modification/de changement d'enregistrement/d'accès
respectivement dans `DataModified (SI)`, `MetadataChanged (SI)` et `Accessed (SI)`.
Les lignes de slack récupérées placent les quatre heures dans les colonnes `(FN)` et laissent les colonnes SI vides.
Les valeurs de repli des lignes normales ne constituent **pas** une preuve SI indépendante. Utilisez les attributs JSON d'origine
et la provenance du slack pour déterminer la source de l'horodatage.

`-t ys-ntfs-parser-csv` (anciennement `ys-mft-parser-csv`, toujours accepté) utilise le schéma orienté entrées
du parseur maintenu. Ces noms de colonnes lisibles s'appliquent à la sortie `-t csv` par défaut,
y compris le `MFT.csv` d'`auto-parse`. Les sorties JSON/JSONL conservent chaque
attribut (y compris le `$DATA` résident, en hexadécimal) et `FullPath` ; utilisez-les lorsque le CSV
d'investigation aplati omet la structure de bas niveau. Avec `-x`, chaque entrée JSON comporte également
`SlackFileNames`.

## Stratégie d'investigation { #investigation-strategy }

1. Filtrez une fenêtre temporelle, mais conservez les enregistrements supprimés et les deux jeux d'horodatages.
2. Recherchez les chemins complets, les doubles extensions suspectes, les ADS, les répertoires de préparation (staging) et les empreintes/noms connus.
   Avec `-x`, incluez les noms du slack qu'aucun fichier utilisé ne porte (`CopyOfEntryInUse` `False`) ; avec
   `-i -E ps1,bat,vbs,js`, lisez les petits scripts directement dans le CSV.
3. Pivotez sur l'entrée/la séquence et sur `USN` vers le journal USN.
4. Comparez les références MFT des Prefetch/LNK/Jump Lists avec l'enregistrement cible.
5. Corrélez la suppression avec la quarantaine Defender ou avec l'activité USN `FILE_DELETE`/de renommage.

## Plus de détails { #further-details }

- [Microsoft : Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications : NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Crate mft de ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [Parseur mft d'Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [Documentation du format libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
