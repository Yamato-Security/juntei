# Prefetch { #prefetch }

## Qu'est-ce que Prefetch { #what-prefetch-is }

Les fichiers Prefetch de Windows enregistrent des informations servant à optimiser le démarrage des applications. Un fichier `.pf`
propre à un programme stocke généralement le nom et l'empreinte de l'exécutable, le nombre d'exécutions, les heures d'exécution récentes, les informations
de volume, les répertoires et les fichiers référencés pendant le démarrage.

Sur les versions de Windows prises en charge, un enregistrement Prefetch d'application valide constitue généralement une preuve solide
que l'exécutable a été exécuté. C'est l'un des artefacts d'exécution les plus utiles en dehors des journaux d'événements.

<span class="evidence-strength">Meilleure conclusion :</span> l'exécutable nommé a été lancé à la ou aux
heures d'exécution récentes enregistrées, sous réserve du comportement du système d'exploitation et de la validité du parseur.

## Emplacement et collecte { #location-and-collection }

```text
C:\Windows\Prefetch\*.pf
```

Juntei prend en charge les versions de format 17 à 31 (de Windows XP à Windows 11), y compris les fichiers compressés
de Windows 8 et versions ultérieures. Collectez l'ensemble du répertoire afin de pouvoir comparer les empreintes des noms d'exécutables et les enregistrements
voisins.

## Précautions d'analyse { #analyst-cautions }

- Windows ne stocke qu'une seule heure d'exécution historique sur les anciennes versions, et jusqu'à huit sur les versions récentes.
  `RunCount` peut dépasser le nombre d'horodatages disponibles.
- Prefetch peut être désactivé, nettoyé, exclu ou indisponible sur certains systèmes/rôles. Son absence ne
  prouve pas qu'un programme n'a pas été exécuté.
- Les listes de fichiers chargés décrivent les dépendances et les accès au démarrage ; elles ne prouvent pas que chaque fichier listé
  était malveillant, ouvert délibérément ou entièrement lu.
- La longueur du nom de l'exécutable dans l'en-tête Prefetch est limitée ; Juntei résout un chemin à partir des fichiers
  chargés lorsque cela est possible sans ambiguïté.
- Les horodatages du système de fichiers d'un fichier source `.pf` sont distincts des heures d'exécution incorporées.
- `Findings` sont des observations de chasse. Les programmes d'installation et l'administration légitime utilisent souvent des chemins de préparation (staging).

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.ExecutableName? | test("powershell|rundll32"; "i"))'
```

## Champs CSV standard { #standard-csv-fields }

| Champ | Interprétation |
|---|---|
| `SourceFile` | Fichier `.pf` d'entrée ; conservez-le pour distinguer les hôtes et les noms d'exécutables en double. |
| `ExecutableName` | Nom de l'exécutable dans l'en-tête. Insensible à la casse pour la plupart des recherches ; conservez la graphie d'origine. |
| `ExecutablePath` | Chemin chargé résolu par Juntei lorsque c'est possible. Confirmez-le avec `Files` et d'autres artefacts. |
| `PrefetchHash` | Empreinte utilisée par Windows dans le nom du fichier `.pf` ; il ne s'agit pas d'une empreinte cryptographique du contenu du fichier. |
| `FormatVersion` | Version de la structure Prefetch ; détermine les champs disponibles et le nombre d'horodatages. |
| `RunCount` | Compteur cumulatif enregistré par Windows. Utile pour la fréquence, pas pour un historique exact. |
| `LastRun` | Heure d'exécution conservée la plus récente. |
| `PreviousRun1` … `PreviousRun7` | Heures d'exécution conservées antérieures, de la plus récente à la plus ancienne. Vides lorsque le format/l'historique ne les contient pas. |
| `VolumeCount` | Nombre de volumes sources enregistrés. |
| `VolumeDevicePaths` | Chemins de périphérique NT des volumes utilisés pendant le démarrage. À corréler avec les numéros de série. |
| `VolumeSerialNumbers` | Identifiants de volume ; utiles pour relier des preuves de lecteurs amovibles ou reformatés. |
| `VolumeCreated` | Heures de création des volumes enregistrées, et non heures de création de l'exécutable. |
| `DirectoryCount` | Nombre de chaînes de répertoires enregistrées. |
| `FileCount` | Nombre de références de fichiers chargés enregistrées. |
| `Findings` | Observations de Juntei, telles que des binaires système déplacés ou des chemins d'exécution suspects. À examiner manuellement. |
| `Files` | Fichiers chargés/référencés avec les références NTFS disponibles. Excellent pivot pour les DLL/chemins, mais pas une preuve de l'intention liée au fichier. |
| `Dir` | Chaînes de répertoires associées au démarrage. Utiles lorsque les chemins des fichiers chargés individuels sont incomplets. |

## Champs CSV de chronologie (`-T`) { #timeline-csv-fields-t }

| Champ | Interprétation |
|---|---|
| `RunTime` | Une heure d'exécution Prefetch conservée. |
| `ExecutableName`, `ExecutablePath` | Contexte d'identité/de chemin du programme. |
| `PrefetchHash` | Distingue les variantes/chemins partageant un même nom d'exécutable. |
| `RunCount` | Compteur du fichier Prefetch ; il est répété sur chaque ligne de la chronologie. |
| `SourceFile` | Fichier `.pf` source, pour validation. |

Les sorties JSON/JSONL conservent les volumes, répertoires, fichiers chargés et références de fichiers NTFS sous forme de tableaux
structurés plutôt que de cellules CSV jointes par des virgules.

## Stratégie d'investigation { #investigation-strategy }

1. Utilisez `-T` pour ajouter les exécutions conservées à une chronologie.
2. Regroupez par nom d'exécutable et empreinte Prefetch ; un même nom peut avoir des chemins/empreintes différents.
3. Examinez `ExecutablePath`, les chemins de préparation, les constatations de binaires système déplacés et les DLL chargées.
4. Faites correspondre les références MFT des cibles et les chemins de fichiers aux enregistrements `$MFT`/USN.
5. Corrélez les heures d'exécution avec les journaux d'événements, SRUM, la télémétrie réseau, les services, les tâches et les artefacts utilisateur.

## Plus de détails { #further-details }

- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [Security Ronin prefetch-forensic](https://github.com/SecurityRonin/prefetch-forensic)
