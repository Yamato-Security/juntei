# Carving des journaux d'événements { #carve-evtx }

Récupérer les chunks de journaux d'événements qui subsistent après l'effacement ou la suppression d'un journal : depuis une image disque, l'espace non alloué, un fichier d'échange ou une image mémoire. Chaque chunk EVTX de 64 Kio contient ses propres tables de chaînes et de modèles ; ses enregistrements peuvent donc être lus sans le reste du journal. Juntei écrit les chunks récupérés sous forme de fichiers `.evtx` ordinaires, un par canal, exploitables avec Hayabusa ou l'Observateur d'événements.

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
```

Sans `-o`, seul le résumé est affiché.

## Recherche des chunks { #finding-chunks }

- La signature de chunk est recherchée à chaque décalage d'octet, et chaque en-tête doit être plausible (taille de l'en-tête, numéros d'enregistrement, décalages). Les sommes de contrôle sont signalées, mais pas exigées : un chunk partiellement écrasé contient encore des enregistrements lisibles.
- Les chunks des journaux compressés par NTFS sont décompressés (LZNT1, unités de compression de 16 clusters). Sur une image disque, la taille de cluster du volume est utilisée ; pour les autres fichiers, c'est `--cluster-size <BYTES>` (4096 par défaut).
- Les enregistrements sont parcourus d'après leur taille jusqu'au décalage du dernier enregistrement indiqué dans l'en-tête ; un identifiant d'enregistrement répété n'interrompt jamais le parcours. Après un enregistrement endommagé, le parcours reprend à l'enregistrement valide suivant. Les anciens enregistrements présents dans le slack du chunk sont également listés.

## Images disque et volumes NTFS { #disk-images }

Avec une image E01 ou VHDX, ou un disque ou volume brut contenant du NTFS, chaque chunk est classé selon le cluster où il se trouve :

| Emplacement | Signification |
|---|---|
| `FreeSpace` | Un cluster qu'aucun fichier n'utilise. |
| `DeletedLog` | Un cluster libre que l'enregistrement `$MFT` d'un journal supprimé référence encore, ou un journal reconstruit à partir de ces clusters. |
| `LiveLog` | Un cluster d'un journal d'événements en cours d'utilisation. Exclu, sauf avec `--keep-live`. |
| `AllocatedFile` | Un cluster d'un autre fichier, comme `pagefile.sys` ou le fichier de stockage d'un cliché instantané. |
| `AllocatedUnknown` | Un cluster alloué qu'aucun enregistrement `$MFT` ne référence. |
| `OutsideVolumes` | En dehors de tous les volumes NTFS du disque. |
| `Unclassified` | L'entrée ne contient aucun volume NTFS (un fichier d'échange ou une image mémoire). |

Les journaux supprimés dont l'enregistrement `$MFT` ne référence que des clusters libres sont reconstruits en entier, y compris s'ils sont compressés, et enregistrés sous `Recovered/`. `--free-space-only` ne recherche que dans les clusters libres : c'est plus rapide, mais le fichier d'échange, les clichés instantanés et les autres fichiers sont ignorés.

## Déduplication { #deduplication }

Chaque chunk n'est écrit qu'une fois. Un chunk est exclu lorsqu'il appartient à un journal actif, lorsqu'il est identique à un autre chunk, ou lorsque tous ses enregistrements figurent déjà dans les journaux actifs ou dans un chunk déjà écrit. Les chunks qui contiennent le plus d'enregistrements sont examinés en premier ; une copie plus ancienne et plus courte d'un chunk n'apporte donc rien. Pour une entrée sans NTFS, indiquez les journaux actifs avec `--live-logs <DIR>`.

## Résultats { #results }

- `Carved/<CHANNEL>.evtx` : un journal par canal, nommé comme Windows nomme ses journaux (`Microsoft-Windows-Sysmon%4Operational.evtx`). Les numéros d'enregistrement des en-têtes de chunk sont renumérotés et les sommes de contrôle recalculées. Un enregistrement endommagé et tout ce qui le suit deviennent du slack. Pour le reste, chaque chunk est conservé en entier, slack compris. L’option `-x` de Hayabusa examine les blocs marqués comme vides. Consultez le fichier `records.csv` de Juntei pour retrouver les enregistrements dans l’espace résiduel des blocs partiellement occupés.
- `Recovered/Volume<N>/...` : les journaux supprimés reconstruits à partir de leurs enregistrements `$MFT`.
- `chunks.csv` / `chunks.jsonl` : chaque chunk trouvé, avec son emplacement, ce qu'il est devenu (`Status`), son canal, les identifiants d'enregistrement, les horodatages et l'état de la somme de contrôle.
- `records.csv` / `records.jsonl` : chaque enregistrement des chunks écrits. `OnlyInCarvedData` signale les enregistrements absents des journaux actifs.

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

Le carving ne trouve que des chunks contigus. Un chunk réparti sur des clusters non contigus, ou sur des pages physiques non contiguës en mémoire, est incomplet, et ses derniers enregistrements peuvent être endommagés ou manquants. Décompressez d'abord `hiberfil.sys`. Le code de sortie vaut 0 lorsque la recherche s'est terminée et 1 en cas d'erreur.

## Pour aller plus loin { #further-details }

- [Utilisation détaillée et référence des champs (anglais)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.md)
- [Utilisation détaillée et référence des champs (japonais)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.ja.md)
- [Format EVTX (documentation de libevtx)](https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc)
