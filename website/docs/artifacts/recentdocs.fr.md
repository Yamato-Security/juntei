# RecentDocs { #recentdocs }

## Qu'est-ce que RecentDocs { #what-recentdocs-is }

L'Explorateur Windows stocke des listes de fichiers et dossiers récemment utilisés (MRU) dans le `NTUSER.DAT` de chaque utilisateur.
La liste racine couvre les éléments récents de tous types ; des sous-clés regroupent les extensions telles que `.docx`, ainsi qu'une
liste `Folder`. `MRUListEx` indique l'ordre des valeurs numérotées.

RecentDocs est utile pour identifier les documents avec lesquels un utilisateur a interagi, y compris des fichiers qui ont été
supprimés ou déplacés par la suite. Les valeurs peuvent également contenir le nom du raccourci que Windows a utilisé dans le dossier Recent
de l'utilisateur, ce qui permet une corrélation avec les preuves LNK.

<span class="evidence-strength">Meilleure conclusion :</span> l'Explorateur a enregistré l'élément dans la liste
RecentDocs de cet utilisateur. L'heure de la clé ne date que l'élément le plus récent de cette liste précise.

## Emplacement { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

Collectez `NTUSER.DAT` et ses journaux de transactions pour chaque profil pertinent, ainsi que le répertoire de fichiers LNK
Recent de l'utilisateur.

## Précautions d'analyse { #analyst-cautions }

- `MRUListEx` fournit un ordre, et non un horodatage pour chaque élément.
- L'heure de dernière écriture de la clé correspond à la mise à jour la plus récente ; seule la position MRU 0 reçoit un
  `LastOpenedUTC` défendable qui en est dérivé.
- Un élément peut apparaître dans la liste racine et dans une liste d'extension. Ne le comptez pas comme deux ouvertures.
- Le comportement des applications et de l'Explorateur peut varier ; certaines méthodes d'accès peuvent ne pas mettre à jour RecentDocs.
- Les outils de nettoyage/de confidentialité et les limites de taille des listes suppriment les preuves plus anciennes.
- Un nom de fichier peut être contrôlé par l'attaquant et ne prouve pas le type de contenu.
- L'utilisateur est déduit des chemins de profil ou des données de dossiers du shell ; examinez les noms d'utilisateur vides/ambigus.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

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
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host` | Hôte déduit du contexte de collecte/du système lorsque c'est possible. |
| `User` | Propriétaire du profil déduit du chemin ou des chemins de dossiers du shell ; conservez l'incertitude lorsqu'il est vide. |
| `SourcePath` | `NTUSER.DAT` d'entrée. Essentiel lors de la combinaison de plusieurs utilisateurs. |
| `Extension` | Sous-liste (`.docx`, `Folder`, etc.) ; nul pour la liste RecentDocs racine. |
| `MRU-Position` | Zéro correspond à l'élément le plus récent de cette liste. Il s'agit d'un ordre relatif, et non d'un temps écoulé. |
| `ValueIndex` | Valeur de Registre numérotée référencée par `MRUListEx`. Pivot structurel uniquement. |
| `Filename` | Nom de l'élément décodé. Corrélez le chemin complet via LNK, Jump List, MFT ou les données de l'application. |
| `LinkName` | Nom du raccourci encodé dans l'élément du shell, souvent présent dans le répertoire Recent de l'utilisateur. |
| `KeyLastWrittenUTC` | Heure de la dernière modification de la clé de liste. S'applique à la liste, et non à chaque ligne. |
| `LastOpenedUTC` | Renseigné uniquement pour la position MRU 0, pour laquelle l'heure de la clé étaye cette interprétation. |
| `ValueDataRaw` | Octets bruts lorsque le décodage échoue ou que la preuve doit être validée. |
| `TransactionLogStatus`, `LogsApplied` | Indique si les journaux du Registre ont été rejoués, et lesquels. |
| `Problems` | Valeurs MRU manquantes, valeurs non listées, types incorrects ou dommages de décodage. |

## Stratégie d'investigation { #investigation-strategy }

1. Séparez la liste racine et les listes d'extension de chaque utilisateur ; dédupliquez avec soin un même élément.
2. Utilisez la position MRU pour l'ordre, et uniquement `LastOpenedUTC` pour une affirmation horodatée.
3. Rapprochez `LinkName`/`Filename` des données LNK Recent, des Jump Lists, de `$MFT` et de l'USN.
4. Examinez les noms leurres, les doubles extensions, les cibles réseau/amovibles et les documents proches de l'accès initial.
5. Corrélez avec Prefetch/SRUM de l'application et les journaux d'événements lorsque l'exécution est en jeu.

## Plus de détails { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [Poster SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/)
