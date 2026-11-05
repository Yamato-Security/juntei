# Analyse automatique des collectes { #automatic-collection-parsing }

`auto-parse` est le moyen le plus rapide de transformer une collecte de triage Windows portant sur plusieurs ordinateurs en un
répertoire d'analyse structuré. Indiquez-lui la racine des preuves et un nouveau répertoire de sortie. Il découvre
les systèmes collectés, identifie leurs noms d'ordinateur et exécute en parallèle tous les parseurs Juntei
compatibles.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

La sortie par défaut est au format CSV. Ajoutez une chronologie JSONL pour `jq` ou pour l'importation dans un SIEM tout en conservant ces résultats
CSV :

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "Séparez l'arborescence de sortie de celle des preuves"
    Juntei refuse un répertoire de sortie situé à l'intérieur de `<EVIDENCE>` ou contenant `<EVIDENCE>`. Cela
    évite que des données générées soient redécouvertes comme preuves et empêche `--clobber` de
    toucher aux fichiers de la collecte source.

## Fuseau horaire de sortie { #output-time-zone }

Les horodatages analysés sont par défaut en UTC, indépendamment du fuseau horaire local de l'ordinateur de l'analyste. L'UTC
utilise la forme des journaux d'événements Windows `2026-10-20T08:46:38.2001668Z` : une date/heure ISO 8601, `T`
entre la date et l'heure, et `Z` pour UTC. Les chiffres fractionnaires disponibles sont conservés.
Le champ `GeneratedUTC` de `manifest.json` utilise également cette forme. Les valeurs `Start time` et `End time` de la console
sont exprimées dans l'heure locale de l'ordinateur de l'analyste, par exemple `2026-11-04 09:15:02`, quel que soit le fuseau horaire
de sortie.

Choisissez un décalage d'affichage fixe avec `-Z, --timezone` :

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

Un décalage est une valeur `HH:MM` signée à deux chiffres (heures `00–23`, minutes `00–59`). Les noms tels que
`JST`, les valeurs abrégées telles que `+9` et les plages invalides produisent une erreur accompagnée d'un exemple de syntaxe.
Omettez l'option ou utilisez `--timezone "+00:00"` pour l'UTC. Un décalage fixe n'applique pas les règles
d'heure d'été.

L'option s'applique aux horodatages analysés des résultats CSV, JSON et JSONL par artefact, ainsi qu'à
l'export JSONL supplémentaire `-j`. Par exemple, `2026-10-20T08:46:38.2001668Z` devient
`2026-10-20T17:46:38.2001668+09:00` ; les deux décrivent le même instant.

Les noms de champs restent stables, y compris les noms existants se terminant par `UTC`. Les valeurs converties portent
explicitement le décalage sélectionné. `OutputTimeZone` dans les deux manifestes enregistre le décalage d'affichage
demandé, tandis que `GeneratedUTC` reste une métadonnée d'exécution en UTC. Les journaux par parseur conservent les résumés natifs en UTC des parseurs.

Les nombres FILETIME/epoch bruts, les dictionnaires de valeurs du Registre, la configuration du fuseau horaire source, les métadonnées
textuelles, les valeurs ne comportant qu'une date et les valeurs d'horloge locale sans décalage connu sont conservés. Juntei
n'ajoute pas `Z` à une heure locale inconnue. Les horloges des éléments du shell issues de FAT (`Item*UTC` dans
ShellBags/OpenSaveMRU et les `TargetItems` des LNK/Jump Lists) n'ont pas de décalage enregistré ; leurs anciens libellés
UTC ou le suffixe `Z` affiché ne prouvent pas qu'il s'agit d'UTC, et l'option de décalage les conserve donc.
Les colonnes CSV contenant des résumés en texte libre conservent
leur texte enregistré ; le JSON/JSONL conserve les horodatages analysés imbriqués sous forme de champs structurés.

L'UTC utilise directement la sortie existante du parseur. Un décalage explicite non nul ajoute une passe de
formatage en flux dans chaque worker de parseur ; il modifie les valeurs affichées sans modifier les preuves.

Si un décalage devait faire sortir un horodatage des années `0000–9999`, cette valeur conserve son décalage
explicite d'origine. Les valeurs comportant plus de neuf chiffres fractionnaires restent également inchangées plutôt que
d'être arrondies. Juntei consigne un avertissement dans le fichier `*-logs.txt` du parseur et convertit les autres
horodatages. Un CSV/JSON mal formé ou une erreur de sortie fait tout de même échouer la tâche et laisse inchangé le fichier
de résultat d'origine concerné.

## Ce qu'il découvre { #what-it-discovers }

Juntei recherche de manière récursive les répertoires nommés `Windows` qui contiennent un répertoire `System32`, `Prefetch`
ou `AppCompat`, sans tenir compte de la casse. La reconnaissance des deux derniers prend également en charge les collectes ciblées
qui ne contiennent que Prefetch ou Amcache. Cela exclut les répertoires de profil utilisateur tels que
`AppData\Roaming\Microsoft\Windows`. Les anciennes installations imbriquées telles que
`C:\Windows.old\Windows` sont rattachées au volume collecté englobant au lieu de devenir des ordinateurs
en double. La profondeur maximale par défaut est de 10. Cela couvre les chemins Velociraptor courants tels que :

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

Ne modifiez la limite que lorsque la collecte est imbriquée plus profondément :

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

Pour chaque système découvert, le nom d'ordinateur est choisi dans cet ordre :

1. `hostname` dans le `client_info.json` ancêtre le plus proche (sa valeur étant conservée comme `FQDN` dans le manifeste).
2. `ComputerName` dans une ruche du Registre `SYSTEM` collectée.
3. Le nom de répertoire de collecte utile le plus proche, en ignorant le lecteur et l'arborescence `uploads/auto`
   de Velociraptor, et en supprimant un suffixe tel que `_F.…`.

Les noms sont rendus sûrs pour être utilisés comme noms de répertoire. Les noms en double reçoivent un suffixe numérique à la fois dans
leur répertoire de sortie et dans leur valeur `Comp`, afin que les systèmes distincts restent différenciables en JSON.
Le manifeste conserve le nom d'hôte exactement tel qu'il a été découvert dans un champ `Hostname` distinct.

Après avoir listé les systèmes, `auto-parse` affiche une ligne `Timezone:` juste au-dessus de
`Parser tasks`. Elle indique le fuseau horaire de sortie sélectionné : `UTC` par défaut, ou une valeur telle que
`UTC+09:00` lorsque `--timezone "+09:00"` est utilisé. Le libellé est en vert et la valeur utilise la
couleur de texte par défaut du terminal, comme les autres valeurs du résumé. `-K, --no-color` laisse la ligne sans mise en forme.
La configuration du fuseau horaire des systèmes collectés est disponible dans les résultats
[Informations système](../artifacts/system-info.md).

## Parseurs exécutés { #parsers-it-runs }

L'échec d'un parseur n'arrête pas les autres. `auto-parse` exécute ces tâches pour chaque ordinateur
découvert. Les tâches utilisent les mêmes commandes regroupées par source que l'analyse manuelle, telles que
`parse-ntfs mft`, `parse-registry amcache`, `parse-ese srum` et `parse-file prefetch` :

- Amcache
- `$MFT`
- Enregistrements redo/undo NTFS de `$LogFile`
- Enregistrements Prefetch et une chronologie d'exécution Prefetch distincte
- Journal USN
- Index de répertoires `$I30`
- Secteurs de démarrage `$Boot` et descripteurs de sécurité `$Secure:$SDS`
- SRUM
- Raccourcis LNK
- Jump Lists
- ShimCache
- Persistence Hunter (131 vérifications de persistance multi-sources avec des profils d'autorisation/de blocage)
- Métadonnées de quarantaine de Defender
- Historique des boîtes de dialogue Ouvrir et Enregistrer sous
- toutes les clés et valeurs du Registre, y compris les données supprimées récupérables
- services, RecentDocs, TaskCache, comptes SAM, emplacements de démarrage automatique du Registre, UserAssist, BAM,
  programmes installés, MuiCache, ShellBags et périphériques USB
- informations système : noms, version de Windows, fuseau horaire, dernier arrêt, profils et SID
- la stratégie d'audit : quels événements de sécurité le système était configuré pour journaliser
- configuration de démarrage (BCD et BCD-Template)
- profils réseau, interfaces, partages, proxys de ports et routes
- noms et horodatages des secrets LSA, et ouvertures de session de domaine mises en cache en cours d'utilisation (rien n'est déchiffré)

Persistence Hunter conserve toutes les sévérités (`--min-severity info`), y compris les correspondances partielles d'autorisation.
`LOW` est une classification à examiner ; l'absence de métadonnées de signature peut également la produire pour des commandes
malveillantes. L'inventaire automatique n'écarte pas ces entrées en appliquant le seuil moyen par défaut
de la commande manuelle.

Le hunter utilise un seul worker dans chaque processus de parseur automatique. Cela évite que `auto-parse -T <N>`
ne multiplie son budget de processus par un autre pool dimensionné selon le nombre de CPU dans chaque processus enfant. La commande autonome
`persistence-hunter -T <N>` contrôle les threads de travail propres au hunter.

La sortie CSV de `$MFT` inclut tous les horodatages et les noms courts. La sortie du Registre inclut les données brutes et une
interprétation FILETIME des valeurs `REG_QWORD`. La sortie SRUM JSON et JSONL inclut les métadonnées d'état
des valeurs. Les fichiers Defender et les flux résidents de `$MFT` ne sont **pas extraits** automatiquement.

Un artefact qui n'a pas été collecté constitue une tâche `skipped` normale, et non un échec.

## Organisation de la sortie { #output-layout }

Les résultats sont organisés par ordinateur et par catégorie d'investigation. Les répertoires de catégorie utilisent le
PascalCase, par exemple `Accounts` et `FileAndFolderOpening`. Les tables Amcache et Jump Lists se trouvent
directement dans leurs catégories, avec des préfixes de nom de fichier. Les noms des répertoires d'ordinateur conservent l'orthographe découverte. Les répertoires de sortie
existants ne sont pas migrés ; choisissez un nouveau répertoire de sortie lorsque vous passez de l'ancien nommage au nouveau. Chaque parseur écrit un
fichier `<artifact>-logs.txt` à côté de ses résultats, y compris lorsque l'artefact n'a pas été collecté.
Le format CSV conserve des tables séparées, car chaque artefact possède des colonnes différentes :

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   └── Registry-RemoteAccess-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| Catégorie | Artefacts |
|---|---|
| `Accounts` | Comptes locaux SAM, noms et horodatages des secrets LSA |
| `Applications` | Historique des applications (traces d'outils tiers) |
| `Applications` | Programmes installés |
| `ExternalDevices` | Périphériques et volumes USB |
| `FileAndFolderOpening` | Jump Lists, raccourcis LNK, Microsoft Office, listes MRU, boîtes de dialogue Ouvrir et Enregistrer sous, RecentDocs, ShellBags |
| `FileExistence` | Amcache, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, Journal USN |
| `NetworkActivity` | Profils réseau, interfaces, partages, proxys de ports et routes ; connexions, trafic et notifications push SRUM |
| `LateralMovement` | Connexions sortantes Bureau à distance, PuTTY et WinSCP des utilisateurs, lecteurs mappés et partages |
| `Persistence` | Configuration de démarrage (BCD), démarrages automatiques du Registre, services, TaskCache |
| `ProcessExecution` | Compatibilité des applications, BAM et DAM, FeatureUsage et éléments épinglés, MuiCache, Prefetch, chronologie Prefetch, activité des applications/du CPU SRUM, UserAssist |
| `Registry` | Clés et valeurs générales des ruches, y compris les enregistrements récupérés |
| `SecurityProducts` | Métadonnées de quarantaine de Defender et contenu extrait facultatif ; paramètres de sécurité (Defender, UAC, protection des identifiants, RDP, pare-feu, journalisation) |
| `SystemInformation` | Noms, version de Windows, fuseau horaire, dernier arrêt, profils, SID et stratégie d'audit ; énergie/capacité SRUM, tables de correspondance des identités et fournisseurs inconnus |

Les catégories aident à organiser les investigations ; elles ne prouvent pas une activité particulière. Par exemple,
les entrées Amcache et ShimCache n'établissent pas à elles seules une exécution. SRUM enregistre l'utilisation des ressources ;
consultez le guide des artefacts avant de tirer des conclusions sur l'exécution d'un programme.

Les formats CSV, JSON et JSONL utilisent les mêmes catégories. Un fichier `<artifact>-logs.txt` par tâche de parseur contient sa
commande exacte et ses messages ; par exemple, `HOST1/FileSystem/MFT-logs.txt` et
`HOST1/ProcessExecution/Prefetch-logs.txt`. Les formats JSON et JSONL utilisent les mêmes noms de fichiers journaux.
Amcache et Jump Lists disposent chacun d'un journal partagé à côté de leurs tables préfixées. Les tables des fournisseurs SRUM sont placées directement
dans `NetworkActivity`, `ProcessExecution` et `SystemInformation`, avec un fichier partagé
`SystemInformation/SRUM-logs.txt`. Aucun sous-répertoire `logs/` n'est créé.

Les noms de fichiers CSV par défaut sont `Registry-SAM.csv`, `LNK.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv` et
`DefenderQuarantine.csv`. Les tables issues du Registre utilisent le préfixe `Registry-`, notamment
`Registry-ShimCache.csv`, `Registry-UserAssist.csv`, `Registry-BAM.csv` et `Registry-SystemInfo.csv`.
Les clés et valeurs générales des ruches se trouvent dans `Registry-Dump.csv`. Les tables Amcache sont
`FileExistence/Registry-Amcache-Devices.csv`, `Registry-Amcache-Files.csv`,
`Registry-Amcache-Findings.csv` et `Registry-Amcache-Summary.csv`. Les Jump Lists utilisent
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` et
`JumpLists-CustomDestinations.csv`. Il n'existe pas de sous-répertoires Amcache ou Jump Lists.
Les tables SRUM utilisent des noms de fichiers `SRUM-` directement sous leur catégorie, par exemple
`NetworkActivity/SRUM-NetworkConnections.csv`.
Consultez le [tableau complet de placement des tables SRUM](../artifacts/srum.md#categorized-auto-parse-output).

Les journaux reprennent le nom de l'artefact avec ses majuscules, par exemple `Registry-SAM-logs.txt`, `PrefetchTimeline-logs.txt`
et `Registry-UserAssist-logs.txt`. Les fichiers JSON/JSONL utilisent les mêmes radicaux, par exemple `MFT.jsonl` ; l'identifiant
`Artifact` de l'enregistrement est `mft` pour les requêtes. Les sorties à plusieurs tables ont
`Registry-Amcache-logs.txt` ou `JumpLists-logs.txt` à côté de leurs tables. Le journal SRUM partagé se trouve
dans `SystemInformation`.


`manifest.json` enregistre le `Hostname` d'origine découvert, l'identité unique `Comp` de sortie/des enregistrements,
la provenance du nom d'hôte, les chemins des preuves, la `Category` de l'artefact, les lignes de commande, la durée, le code
de sortie, le chemin du résultat, le chemin du journal, la chronologie JSONL demandée et toute erreur d'export, si l'extraction Defender a été demandée,
et si chaque tâche
a réussi, a été ignorée ou a échoué.

Auto-parse n'écrit aucun chemin absolu ; les résultats peuvent donc être partagés sans révéler où se trouvent les preuves,
les résultats ou Juntei sur le système d'analyse :

- Les chemins des preuves sont relatifs au répertoire des preuves. Le terminal répertorie chaque ordinateur avec son
  répertoire dans les preuves (`HOST1: ./HOST1_F`), et les parseurs s'exécutent dans le répertoire des preuves et
  reçoivent de tels chemins. Les colonnes de chemin source des résultats (`SourceFile`, `SourcePath`, …), les journaux
  ainsi que les champs `EvidenceRoot` et `WindowsDirectories` des manifestes indiquent où se trouve chaque fichier dans les
  preuves (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`).
- Les chemins des résultats dans les manifestes et les journaux (`Output`, `OutputFiles`, `Log`, `JSON-Timeline`, les
  chemins de sortie dans `Command`) ainsi que `ExtractedFile` de Defender sont relatifs au répertoire de sortie
  (`./HOST1/ProcessExecution/Prefetch.csv`, `../siem.jsonl` pour une chronologie placée à côté). `Command`
  commence par le nom du programme (`juntei`) ; ses chemins de preuves sont relatifs au répertoire des
  preuves et ses chemins de sortie au répertoire de sortie.
- `EvidenceDirectory` et `OutputDirectory` sont les noms des deux répertoires. `ManifestVersion` 2
  identifie les manifestes qui enregistrent les chemins des résultats à partir du répertoire de sortie ; les manifestes antérieurs n'ont pas de
  `ManifestVersion` et les enregistrent tels qu'ils ont été fournis à `-o`. Une réexécution avec `-C` lit les deux.
- Le terminal affiche un chemin de sortie ou de chronologie tel qu'il a été fourni lorsqu'il est relatif, et à partir du
  répertoire courant lorsqu'il est absolu (`../results/`).

Après le nombre de processus parallèles, le terminal affiche la taille totale des fichiers des
systèmes Windows trouvés (`Total filesize: 26.1 MiB`).

Le message de progression indique l'ordinateur et l'artefact en cours, suivis d'une taille d'entrée
approximative, par exemple `HOST1 / prefetch (~2.0 MiB)`. Un groupe de fichiers utilise leur taille combinée ; chaque
tâche MFT, LogFile ou Journal USN affiche sa propre taille d'entrée. Les estimations excluent les journaux de transactions
auxiliaires et les fichiers d'enrichissement, et ne prédisent pas la taille de la sortie analysée.

`manifest.csv` fournit le même index des tâches sous la forme d'une table adaptée aux tableurs, avec une ligne par
tâche de parseur. Il inclut `Comp`, le `Hostname` d'origine, `FQDN`, `HostnameSource`, `EvidenceRoot`,
`Artifact`, `Category`, `OutputType`, `Purpose`, `Status`, `DurationMs`, `ExitCode`, `Output`, `OutputFiles`, `Log`,
`Error` et `Command`. `Purpose` vaut `artifact` pour les résultats catégorisés ou `json_timeline` pour
la passe JSONL native supplémentaire utilisée avec le CSV. `OutputType` est le format de sortie réel du parseur.
`Command` est un tableau JSON contenant l'exécutable et ses arguments, en préservant les espaces et les guillemets.
Pour SRUM, `Output` est le répertoire de l'ordinateur, `OutputFiles` est un tableau JSON de ses chemins de sortie
exacts par fournisseur, et `Category` nomme l'emplacement du journal partagé (`SystemInformation`).
Les fichiers sont catégorisés individuellement ; `OutputFiles` apparaît également dans le manifeste JSON.

Filtrez `Status` sur `skipped` ou `failed` pour trouver les tâches nécessitant un examen ; l'`Error` d'une tâche ignorée
peut être vide, l'explication figurant dans son fichier `Log`.

Le résumé du terminal inclut le nombre total et la taille des fichiers ordinaires actuellement présents sous le
répertoire de sortie, y compris les deux manifestes, les journaux des parseurs et les fichiers extraits. Les liens symboliques ne sont
pas suivis. Les tailles s'adaptent automatiquement à l'aide du même formateur que `Total filesize`, avec
des unités telles que `KiB`, `MiB` et `GiB` :

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

Les tables Amcache et SRUM utilisent `-Source1`, `-Source2`, etc. lorsque plusieurs bases de données sources
ont été collectées. Le journal partagé du parseur enregistre la correspondance entre les chemins d'origine et les chemins finaux, et les enregistrements JSON/JSONL
conservent la provenance de la source. Les tâches CSV Amcache/Jump Lists répertorient les chemins exacts de leurs tables dans le champ
`OutputFiles` du manifeste ; leur `Output` est le répertoire de la catégorie d'investigation. Les exports JSON et JSONL
d'Amcache et des Jump Lists sont déjà des fichiers uniques nommés `Registry-Amcache.json[l]` et
`JumpLists.json[l]`. La chronologie combinée `-j` conserve le même contenu d'enregistrement et les mêmes valeurs `Artifact`.
`--clobber` utilise le manifeste précédent pour effacer les sorties enregistrées, y compris les anciens noms,
tout en préservant les fichiers de catégorie sans rapport. Une collecte normale d'un seul volume écrit les enregistrements `$MFT` et du Journal USN directement dans
`FileSystem/MFT.csv` et `FileSystem/UsnJournal.csv`. Les volumes collectés supplémentaires utilisent des fichiers voisins
numérotés tels que `FileSystem/MFT-2.csv` et `FileSystem/MFT-2-logs.txt`.

## Ingestion JSON et SIEM { #json-and-siem-ingestion }

Chaque enregistrement JSON ou JSONL reçoit deux champs de premier niveau :

- `Comp` : le système source.
- `Artifact` : le parseur Juntei qui a produit l'enregistrement, par exemple `mft` ou `prefetch-timeline`.

`-j, --json-timeline <FILE>` crée un fichier JSONL supplémentaire, quel que soit le format des résultats
individuels. Toutes les sorties catégorisées et leurs journaux sont conservés :

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

Le nom de fichier est utilisé tel qu'il est fourni ; les chemins relatifs sont relatifs au répertoire de travail courant.
Il peut se trouver à la racine de la sortie, dans un autre sous-répertoire de sortie non réservé, ou en dehors du répertoire
de sortie. Les preuves, les manifestes, les résultats/journaux par ordinateur et les répertoires temporaires des parseurs ne peuvent pas
servir de destination. Un fichier existant nécessite `-C` ; un export qui échoue le laisse intact.

En **mode CSV**, Juntei effectue une passe JSONL native supplémentaire afin que le fichier SIEM préserve les types
numériques et booléens, les données imbriquées, les champs bruts et les métadonnées que les tables CSV peuvent omettre. Les
tâches supplémentaires partagent la même limite de concurrence `-T`, n'extraient pas une nouvelle fois le contenu de la quarantaine et
disposent de journaux annexes `*-json-timeline-logs.txt` distincts. Les deux manifestes distinguent leur finalité et
leur format de sortie. Les fichiers JSONL temporaires sont supprimés après une exécution réussie ; ils sont conservés si un
parseur ou l'export échoue. Prévoyez le temps de traitement et l'espace disque temporaire supplémentaires.

En **mode JSON**, Juntei transforme en flux les résultats existants en JSONL compact sans réanalyser les
preuves. Les rapports Amcache sont développés en enregistrements individuels de résumé, de constat, de fichier et de périphérique,
identiques à ceux du JSONL natif. En **mode JSONL**, les enregistrements annotés existants sont combinés directement.
Tous les modes utilisent la même structure d'enregistrement SIEM. Le jeu de données conserve les horodatages natifs et les champs
des artefacts ; il n'invente pas d'horodatage d'événement commun et ne trie pas globalement des artefacts hétérogènes.

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json` enregistre le chemin `JSON-Timeline` demandé et toute `JSON-TimelineError`. Ses enregistrements
de tâches, ainsi que `manifest.csv`, pointent toujours vers les sorties d'artefacts conservées. Les tâches CSV réussies destinées uniquement à la chronologie
référencent le fichier JSONL final au lieu des fichiers temporaires supprimés. Si un parseur échoue, l'exécution
se termine en échec et la chronologie ne contient que les résultats réussis ; examinez les manifestes
et les journaux des catégories avant de la considérer comme complète. Le résumé du nombre et de la taille des fichiers couvre les fichiers situés dans
le répertoire de sortie ; une chronologie externe est signalée séparément sous la forme `JSONL timeline: <FILE>`.

## Performances { #performance }

Les tâches des parseurs sont des processus distincts ; les artefacts gourmands en CPU peuvent donc s'exécuter en même temps, et un dommage dans
un parseur ne peut pas interrompre l'ensemble de l'exécution. Par défaut, Juntei utilise le nombre de CPU logiques. Limitez la
concurrence lorsque les preuves se trouvent sur un disque lent ou que le système dispose de peu de mémoire :

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## Utilisation { #usage }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK and Jump List files [default: 1252]

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Utilisez un nouveau répertoire de sortie lorsque c'est possible. `-C` supprime ou écrase uniquement les cibles générées exactes
de cette exécution, de sorte qu'un artefact désormais absent ne peut pas laisser un résultat obsolète. Il ne supprime pas
les fichiers sans rapport ni l'arborescence de sortie complète. Il supprime également, avant l'analyse, le répertoire
`SecurityProducts/DefenderQuarantined/` de chaque ordinateur, ainsi que le répertoire réservé `defender-quarantined/`
de l'ancienne organisation à plat, que la nouvelle exécution utilise `-x` ou non.

Utilisez un nouveau répertoire de sortie lorsque vous abandonnez une ancienne organisation à plat. Les anciens fichiers CSV/JSON
ne sont ni déplacés ni supprimés automatiquement.

!!! danger "Traitez le contenu Defender extrait comme un logiciel malveillant"
    `-x, --extract-defender-quarantined` écrit le contenu mis en quarantaine sur le disque, dans le répertoire
    `SecurityProducts/DefenderQuarantined/` de chaque ordinateur. Les flux binaires récupérés se terminent par `.quarantined`, et Juntei
    supprime l'autorisation d'exécution. Chaque descripteur de sécurité binaire reçoit également un rapport lisible `.security_descriptor.txt`
    indiquant le propriétaire, les autorisations et le SDDL. N'exécutez jamais les charges utiles récupérées. Utilisez un système d'analyse
    isolé et attendez-vous à ce que les logiciels antivirus ou EDR émettent une alerte sur le contenu récupéré ou le suppriment.

Consultez la section [lire le descripteur de sécurité récupéré](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
pour connaître les champs du rapport et leurs limites forensiques.

Auto-parse lit également les connexions à distance des utilisateurs (RDP, PuTTY, WinSCP et lecteurs mappés).
