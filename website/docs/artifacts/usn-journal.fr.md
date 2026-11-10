# Journal USN (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## Qu'est-ce que le Journal USN ? { #what-the-usn-journal-is }

NTFS enregistre les modifications des fichiers et des répertoires dans le journal des modifications USN (Update Sequence Number). Un
enregistrement peut décrire une création, une extension ou un écrasement de données, une modification d'attributs ou de sécurité, un renommage,
une fermeture et une suppression. Il stocke une référence de fichier, une référence parente, un nom, des motifs, des attributs,
un horodatage et un USN strictement croissant.

Le journal est excellent pour reconstituer une rafale d'opérations sur des fichiers menées par un attaquant, en particulier lorsque
les fichiers ont été supprimés. Il enregistre **le fait que les métadonnées/le contenu ont changé**, et non les octets modifiés.

<span class="evidence-strength">Meilleure conclusion :</span> NTFS a enregistré les motifs de modification indiqués pour
le fichier référencé à cet horodatage. Les motifs s'accumulent souvent jusqu'à un enregistrement `CLOSE`.

## Emplacement et collecte { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` est un flux nommé de métadonnées NTFS. Collectez-le avec un outil forensique capable d'accès brut. Collectez également le
`$MFT` du même volume, car les enregistrements USN stockent des références de fichiers parents plutôt que des chemins complets.

## Précautions d'analyse { #analyst-cautions }

- Le journal est circulaire et de taille limitée. Les anciens enregistrements sont écrasés.
- `FILE_DELETE` indique que NTFS a traité une suppression ; le contenu peut rester récupérable ailleurs.
- Un motif tel que `DATA_OVERWRITE` ne révèle ni les octets, ni le processus, ni l'utilisateur responsable.
- Plusieurs motifs peuvent être combinés dans un même enregistrement ; `CLOSE` marque souvent la fin des modifications accumulées.
- Les chemins sont reconstitués. Les répertoires parents peuvent avoir été renommés, supprimés ou réutilisés.
- Les numéros de séquence des références de fichiers comptent. Le numéro d'entrée seul peut renvoyer à la mauvaise génération.
- Le carving peut récupérer des enregistrements valides sans leur contexte de journal d'origine ; signalez cette distinction.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## Champs de sortie CSV { #csv-output-fields }

| Champ | Interprétation |
|---|---|
| `Name` | Nom du fichier/répertoire stocké dans cet enregistrement USN. Il ne s'agit pas d'un chemin complet. |
| `Extension` | Extension extraite de `Name` par commodité ; ne l'utilisez pas comme type de contenu fiable. |
| `EntryNumber`, `SequenceNumber` | Référence NTFS du fichier. Utilisez les deux pour identifier la génération de l'enregistrement. |
| `ParentEntryNumber`, `ParentSequenceNumber` | Référence NTFS du répertoire parent au moment de la modification. |
| `ParentPath` | Chemin parent reconstitué par Juntei en tenant compte du temps. Des marqueurs d'inconnu signifient que l'ascendance n'a pas pu être résolue. |
| `UpdateSequenceNumber` | Valeur d'ordre USN. Utile pour un tri stable et pour pivoter vers Amcache/MFT. |
| `UpdateTimestamp` | Horodatage stocké dans les enregistrements V2/V3. Traitez les horodatages issus d'un contexte de carving avec la même validation que pour tout enregistrement. |
| `UpdateReasons` | Un ou plusieurs indicateurs de modification nommés, joints par `|`. Interprétez les combinaisons et `CLOSE` ensemble. |
| `FileAttributes` | Attributs NTFS enregistrés au moment de la modification ; les bits inconnus restent en hexadécimal. |
| `OffsetToData` | Décalage en octets dans l'entrée analysée. Essentiel pour la validation et les enregistrements issus du carving. |
| `SourceFile` | Journal/image d'entrée ; distingue les hôtes et les volumes. |
| `SourceInfo` | Indicateurs de source décrivant comment la modification est entrée dans le journal. Généralement contextuels, et non une attribution. |
| `SecurityID` | Index de l'identificateur de sécurité NTFS de l'enregistrement, et non un SID d'utilisateur. |
| `Version` | Version de l'enregistrement USN. Juntei écrit les versions V2/V3 en CSV ; le JSON conserve les plages de suivi des intervalles (range tracking) V4. |

Le format JSON/JSONL représente correctement les références V3 sur 128 bits et conserve les enregistrements V4 de suivi des intervalles ainsi que
leurs plages. Utilisez-le lorsque des champs de bas niveau propres à une version sont importants.

## Stratégie d'investigation { #investigation-strategy }

1. Filtrez par heure, nom, extension, chemin parent ou référence de fichier.
2. Triez par `UpdateSequenceNumber` en plus de l'heure pour voir l'ordre exact du journal.
3. Associez `RENAME_OLD_NAME` à `RENAME_NEW_NAME` pour les déplacements/renommages.
4. Utilisez `$MFT` pour valider l'état des enregistrements, les chemins, les tailles et les horodatages.
5. Corrélez l'attribution aux processus à partir des journaux d'événements, de l'EDR, de Prefetch, des services/tâches ou de Defender.

## Plus de détails { #further-details }

- [Microsoft : journaux des modifications](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft : USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft : indicateurs de motif du journal des modifications](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [Crate usn de ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
