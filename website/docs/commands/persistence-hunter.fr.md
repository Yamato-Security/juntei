# Persistence Hunter { #persistence-hunter }

La persistance permet à un programme de s'exécuter à nouveau après un redémarrage, une ouverture de session, un déclencheur planifié ou un autre
événement. Un attaquant peut utiliser à cette fin la configuration normale de Windows. De nombreux programmes légitimes
utilisent exactement les mêmes mécanismes ; trouver une entrée de persistance n'est donc qu'un point de départ pour l'analyse.

`juntei persistence-hunter` est un portage natif en Rust de
[PyrsistenceSniper de Hexastrike](https://github.com/hexastrike/PyrsistenceSniper), comportant 131 vérifications : le
catalogue de 130 vérifications de PyrsistenceSniper, plus une vérification native du COM du menu Démarrer.

Pour une explication de chaque mécanisme, de ses emplacements de preuves et de la manière d'examiner un résultat,
consultez le [guide de la persistance en 131 vérifications](../persistence-hunter/index.md). Les
[remerciements relatifs à la persistance](../persistence-hunter/acknowledgements.md) décrivent les projets Python et
PowerShell à l'origine du catalogue.

## Utilisation { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Première analyse { #first-scan }

```bash
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

Les preuves peuvent être un volume Windows monté, une collecte Velociraptor ou un répertoire contenant
plusieurs ordinateurs collectés. Les ordinateurs restent séparés pendant l'analyse, et chaque résultat comporte
`Comp`. L'échappement des noms de fichiers collectés est décodé lors de la résolution des chemins Windows.

Pour une seule ruche :

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

Une seule ruche ne peut pas alimenter les vérifications qui nécessitent une autre ruche ou des preuves du système de fichiers. Lisez le rapport
de couverture avant de considérer qu'un résultat vide est significatif.

Listez les ID de vérification et leurs correspondances ATT&CK :

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

Utilisez les ID de vérification affichés par `--list-checks`, ou un identifiant ATT&CK tel que `T1547.001`.
Un identifiant parent tel que `T1547` sélectionne ses sous-techniques. `--technique` est un alias de `--check`.
Chaque vérification inclut sa technique, sa description et des liens de référence dans les résultats.

## Analyses parallèles { #parallel-scans }

La commande utilise un pool de workers partagé, dont la taille par défaut est le nombre de processeurs logiques. Définissez
`-T, --threads <N>` pour choisir le nombre de workers, ou `-T 1` pour une analyse monothread :

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

La découverte des preuves, les vérifications indépendantes, les métadonnées des charges utiles et les sources de chronologie MFT/EVTX s'exécutent en parallèle. Les
ruches du Registre chargées d'un hôte sont partagées en lecture seule, et les métadonnées mises en cache pour une même charge utile sont initialisées une seule fois.
Les hôtes sont chargés un par un afin d'éviter de multiplier la mémoire des ruches sur l'ensemble de la collecte. Les résultats,
la couverture, la sélection des horodatages et l'ordre des avertissements restent identiques quel que soit le nombre de threads.

`auto-parse` exécute déjà les processus de parseurs en parallèle ; il attribue donc un seul worker à chaque processus enfant
`persistence-hunter`. Sa propre option `-T` contrôle le nombre de processus de parseurs simultanés.

## Preuves à collecter { #evidence-to-collect }

| Source | Exemples de vérifications |
|---|---|
| `SYSTEM` et journaux de transactions adjacents | Services, commandes de récupération des services, paquets LSA, paramètres de démarrage, fournisseurs d'authentification |
| `SOFTWARE` et journaux de transactions adjacents | Clés Run, Winlogon, IFEO, enregistrements COM, TaskCache, paramètres Office |
| `NTUSER.DAT` et `UsrClass.dat` de chaque utilisateur | Démarrages automatiques utilisateur, variables d'environnement, remplacements COM, paramètres Office |
| `SAM` et `SOFTWARE` | Anomalies de RID, comptes cachés, appartenance inhabituelle au groupe Administrateurs local |
| `Windows/System32/Tasks` | Commandes des tâches planifiées et gestionnaires COM ; comparaison avec TaskCache |
| Fichiers `.library-ms` du menu Démarrer de chaque utilisateur et dossiers nommés d'après un CLSID | Candidats déclencheurs d'activation COM, joints aux enregistrements de classes utilisateur/machine correspondants |
| Dossiers Startup, profils PowerShell, modèles Office, paramètres de Terminal | Fichiers et commandes exécutés par des déclencheurs utilisateur ou système |
| `OBJECTS.DATA` de WMI | Instances de consommateurs d'événements et liaisons filtre-consommateur |
| `qmgr.db` ou `qmgr*.dat` de BITS | Commandes de notification récupérées et SID des propriétaires |
| `$MFT` et fichiers EVTX pertinents | Corrélation indépendante des horodatages |
| Exécutables, DLL et fichiers catalogue référencés | Empreintes SHA256 et métadonnées descriptives de PE/certificat |

Les journaux de transactions du Registre sont rejoués lorsqu'ils sont disponibles. `--no-logs` lit les ruches telles que collectées ;
une ruche « sale » non rejouée peut omettre des modifications récentes et est signalée dans la couverture. Le jeu de contrôles
actuel est utilisé pour les vérifications du système actif. La vérification de la porte dérobée DSRM examine en outre
`ControlSet001`, `ControlSet002` et `CurrentControlSet` lorsqu'ils sont présents ; comparez le chemin signalé
avec `SYSTEM\\Select` avant de décrire ce paramètre comme actif. Les autres jeux de contrôles historiques
peuvent être examinés avec les commandes individuelles d'analyse du Registre.

Les lectures de ruches partagent un budget d'entrée de 1 Gio par ordinateur. Les ruches machine sont tentées avant les ruches utilisateur ;
les entrées ou journaux de transactions qui ne tiennent pas dans ce budget sont nommés dans la couverture partielle. Les autres ruches lisibles et
les vérifications fondées sur des fichiers se poursuivent. Seuls les emplacements canoniques des ruches ou les exports explicitement fournis établissent
l'identité d'une ruche ; une copie située dans le répertoire temporaire d'un utilisateur ne peut pas remplacer la véritable ruche de cet utilisateur.
Le lecteur système d'origine enregistré dans SOFTWARE est mis en correspondance avec le volume système capturé ; ainsi, monter
un volume `C:` d'origine sous une autre lettre de lecteur de preuves n'entraîne pas la perte des métadonnées des charges utiles.

`auto-parse` exécute également le hunter et enregistre `persistence/PersistenceHunter.csv` (ou `.json`/`.jsonl`)
dans le répertoire de chaque ordinateur, avec `PersistenceHunter-logs.txt` à côté. Ses résultats alimentent aussi
la chronologie SIEM facultative `-j <FILE>`. L'analyse automatique utilise `--min-severity info` afin de conserver
l'inventaire complet de la configuration, y compris les entrées correspondant à des règles d'autorisation.

## Enregistrements COM et déclencheurs d'activation { #com-registrations-and-activation-triggers }

La vérification `scheduled_task_files` sélectionne le principal de chaque action à l'aide du `Actions Context` du XML de la tâche
et du `Principal id` correspondant. Un principal de groupe est conservé en tant que groupe ;
il ne permet pas d'établir quels enregistrements de classes d'un utilisateur individuel seraient utilisés.

Les candidats gestionnaires COM sont résolus séparément dans les vues de classes natives et 32 bits. Seuls le
`UsrClass.dat`/`NTUSER.DAT` capturé de l'utilisateur correspondant et le `SOFTWARE` de la machine sont pris en compte.
Les principaux exprimés sous forme de SID sont joints par l'intermédiaire des chemins `ProfileList` capturés. Les noms de compte, y compris
`HOST\user` et `DOMAIN\user`, privilégient un profil `user.DOMAIN` correspondant plutôt qu'un simple profil
`user`. Les alias de l'hôte local et `.\user` utilisent les noms d'hôte capturés pour la correspondance des suffixes ;
un suffixe exact prime sur les autres alias d'hôte. Le profil simple n'est pris en compte
que lorsqu'aucun profil qualifié ne correspond. Chacune de ces correspondances est explicitement qualifiée
d'**heuristique fondée sur le nom de dossier** dans `Description` ; elle ne prouve ni la propriété du compte ni l'appartenance au domaine. Les correspondances ambiguës
restent non résolues et sont signalées à la fois dans la description du résultat et dans la couverture ; une ambiguïté
entre profils qualifiés ne déclenche pas de repli sur le profil simple. Cela ne dépend pas
de `Volatile Environment`, qui n'existe normalement qu'en mémoire et n'est pas enregistré avec les ruches du Registre. Un CLSID utilisateur en double
masque les enfants du CLSID machine. Les principaux de groupe utilisent les candidats COM de la machine et conservent
cette limite dans la description du résultat ; un principal de groupe normal ne rend pas à lui seul
la couverture partielle et ne consomme pas les entrées du résumé des erreurs.

Le résolveur suit `InprocServer32` et `LocalServer32`, utilise les chemins `CodeBase` managés
(y compris les sous-clés de version), et joint le `LocalService` d'un AppID à l'`ImagePath` du service SYSTEM
collecté. Les associations d'AppID à un nom d'exécutable sont conservées, mais un nom de base ne permet pas d'établir
le chemin complet d'un exécutable. Les URL CodeBase locales `file:` sont décodées ; les URL distantes sont enregistrées
sans y accéder. Les enregistrements manquants conservent le CLSID avec
`ResolutionStatus: unresolved_com_class`, des champs de chemin/d'empreinte vides et un `Exists` inconnu.

La vérification `start_menu_com` lit les URL XML des fichiers `.library-ms` du menu Démarrer de chaque utilisateur et collecte
les dossiers nommés d'après un CLSID, y compris les répertoires vides. Elle signale le chemin du déclencheur, le CLSID, le serveur
candidat et la provenance dans le Registre. Une acquisition limitée aux fichiers ne peut pas conserver un répertoire vide,
et l'absence des ruches utilisateur empêche une recherche complète des enregistrements par utilisateur. Les liens symboliques ne sont
pas suivis. Il s'agit de candidats de configuration : l'architecture, l'élévation, l'UAC, les stratégies et le
contexte d'activation peuvent déterminer si un enregistrement est réellement utilisé. Corroborez l'exécution
avec des preuves de processus, de chargement de modules ou d'autres preuves indépendantes.

## Classification et profils { #classification-and-profiles }

La classification minimale par défaut est `medium`. Pour examiner l'inventaire complet, y compris
les entrées correspondant à la liste d'autorisation intégrée :

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "Utilisez tous les niveaux de gravité pour examiner des collectes incomplètes"
    Une règle d'autorisation large peut correspondre partiellement à la commande d'un attaquant, même lorsque son signataire est absent
    ou différent. Le projet amont classe les correspondances partielles comme `LOW`, que le seuil par défaut
    `medium` de la commande manuelle masque. Cela peut concerner des tâches planifiées malveillantes situées dans des répertoires Windows.
    Utilisez `--min-severity info` pour une investigation complète et examinez ensemble `PolicyReason`, `SignerStatus`,
    la commande et les preuves sources. `auto-parse` conserve déjà ces entrées.

| Classification | Signification |
|---|---|
| `HIGH` | Une règle de blocage configurée correspond entièrement |
| `MEDIUM` | Aucune règle d'autorisation ou de blocage ne correspond |
| `LOW` | Les conditions strictes d'une règle d'autorisation correspondent, mais les métadonnées du signataire ne correspondent pas ou sont indisponibles |
| `INFO` | Une règle d'autorisation correspond entièrement |

Il s'agit de classifications de stratégie, et non de probabilités mesurées de compromission. Une entrée `MEDIUM`
par défaut n'est pas une preuve de logiciel malveillant. `INFO` n'établit pas qu'un fichier possède une signature valide ni qu'un
attaquant ne l'a pas détourné. Une correspondance partielle avec la liste d'autorisation se situe sous le seuil par défaut ; incluez donc
`--min-severity low` ou `info` lorsque vous examinez des collectes clairsemées ou que vous ajustez la liste d'autorisation.

Fournissez un profil YAML avec `--profile profile.yaml`. Il **remplace** le profil intégré. Les règles globales
et par vérification du fichier sélectionné sont combinées ; toute règle de blocage entièrement satisfaite l'emporte.
Par exemple :

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

Les conditions de règle prises en charge sont `path_matches`, `value_matches`, `hash` (64 chiffres hexadécimaux
SHA256), `not_lolbin` et `signer` (sous-chaîne insensible à la casse). Toutes les conditions strictes indiquées doivent
correspondre. Les métadonnées du signataire constituent la condition souple décrite ci-dessus. Les expressions régulières utilisent la syntaxe
des expressions régulières de Rust : les assertions de type lookaround et les références arrière ne sont pas prises en charge. Les ID de vérification inconnus, les expressions non valides
et les champs de profil inconnus produisent une erreur au lieu d'affaiblir silencieusement la stratégie.

## Champs de sortie { #output-fields }

CSV est le format de fichier par défaut. Utilisez `-t json` pour des objets JSON lisibles ou `-t jsonl` pour une sortie compacte,
à raison d'un objet par ligne. Sans `-o`, un `-t` explicite n'écrit que les données sur stdout ; les avertissements de couverture
sont envoyés sur stderr. Sans aucune de ces options, Juntei affiche un résumé et jusqu'à 30 candidats.

| Champs | Comment les interpréter |
|---|---|
| `Comp`, `CheckID`, `Technique`, `MitreID` | L'ordinateur et le mécanisme de persistance décrits par la ligne |
| `Path`, `Value` | L'emplacement du Registre, le fichier, la tâche ou l'enregistrement récupéré, et son contenu configuré |
| `Description`, `References` | Pourquoi le mécanisme est important et où en savoir plus |
| `AccessGained` | La portée habituelle du mécanisme (`USER` ou `SYSTEM` ; BITS peut aussi identifier `LOCAL_SERVICE`/`NETWORK_SERVICE`) ; ne prouve pas qu'une commande donnée a obtenu ce privilège |
| `Severity`, `PolicyReason` | Classification selon le profil, explication de la stratégie correspondante, et toute réserve liée à une correspondance partielle ou à un signataire non vérifié |
| `SourceFile`, `RecordStatus` | La source collectée exacte, dont le chemin commence par l'entrée telle que fournie (`./HOST/...` depuis auto-parse), et le contexte d'allocation/de rejeu/de récupération ; remontez par ce biais jusqu'aux preuves d'origine |
| `KeyLastWrite` | L'horodatage de la **clé** du Registre. Il ne date pas une valeur individuelle |
| `LastChange`, `ChangeSource`, `ChangeEvidence` | Heure de modification corrélée de manière indépendante, preuve de la source, et indication de la réussite de la corrélation |
| `ResolvedPath`, `ResolutionStatus`, `Exists` | Chemin de la charge utile et résultat de la résolution hors ligne ; les preuves non collectées sont inconnues, et non `false` |
| `SHA256` | SHA256 des octets de la charge utile collectée, lorsqu'ils sont lisibles |
| `Signer`, `SignerStatus` | Métadonnées descriptives du certificat incorporé/du catalogue et leur disponibilité ; la confiance dans le certificat et la validité de la signature ne sont pas vérifiées |
| `IsLolbin`, `Launcher` | Indique si le lanceur ou la charge utile configurés figurent dans la liste LOLBAS intégrée, et le lanceur utilisé pour une charge utile relayée |
| `IsBuiltin`, `IsInOS-Directory` | Contexte relatif aux commandes Windows courantes et aux répertoires système ; aucun de ces champs n'établit la légitimité |

Une commande `rundll32.exe` peut nommer une charge utile DLL. Juntei conserve le lanceur tout en résolvant la charge utile et en
calculant son empreinte. Une référence extérieure à l'ordinateur collecté, un lecteur manquant, une correspondance ambiguë
insensible à la casse ou une variable d'environnement non résolue ne sont pas remplacés par un fichier provenant de
la machine de l'analyste.

## Horodatages et limites { #timestamps-and-limitations }

Le hunter n'utilise pas les heures de modification des fichiers copiés sur la VM de l'analyste comme horodatages de l'attaque.
Lorsqu'elles sont disponibles, il corrèle les écritures dans le Registre avec l'événement Sysmon 13, l'installation de tâches avec
l'événement Security 4698 ou Task Scheduler 106, l'installation de services avec l'événement System 7045, et la configuration fondée sur des fichiers
avec les horodatages d'origine du `$MFT`. La provenance figure dans `ChangeSource`.
Un événement peut dater une modification de configuration sans prouver que la charge utile configurée s'est exécutée.
L'événement System local 6011 peut prouver un nom d'ordinateur antérieur. Le hunter suit les chaînes de renommage vérifiées
ancrées sur l'ordinateur collecté, en conservant sa valeur `Comp` actuelle tout en enregistrant
l'ordinateur de l'événement d'origine et la preuve de l'enregistrement de renommage dans `ChangeSource`. Les noms étrangers non prouvés,
les journaux archivés, les journaux transférés et une autre installation de Windows ne sont pas acceptés par cette règle.

Les valeurs ComputerName/ActiveComputerName, Hostname/NV Hostname de TCP/IP et le nom d'hôte/FQDN des informations client capturés
fournissent des alias d'hôte, couvrant les différences DNS/NetBIOS et les renommages en attente. En l'absence d'identité capturée,
un nom unique dans les journaux principaux peut permettre une datation, avec une provenance explicite d'identité non vérifiée
et un avertissement de couverture. Plusieurs noms non vérifiés ne sont pas attribués à l'hôte. Les noms d'ordinateur
rejetés sont signalés comme une limite de couverture de la corrélation ; leurs canaux restent marqués comme collectés.

Utilisez `--mft <FILE>` pour sélectionner une MFT collectée en externe pour un seul ordinateur. Utilisez `--no-timeline`
pour ignorer la corrélation des horodatages. `NOT_RUN`, `NOT_APPLICABLE`, `NO_ARTIFACT`, `NO_MATCH` et `REJECTED`
expliquent un `LastChange` vide ; un champ vide ne prouve pas que l'entrée est inoffensive.

La récupération WMI et BITS est un carving borné, et non un parseur complet des transactions de base de données.
Les chaînes ou liaisons récupérées peuvent contenir des données anciennes/supprimées ; ces enregistrements doivent être corroborés.
Les schémas de classes WMI seuls ne sont pas comptés comme des abonnements. Une entrée TaskCache sans fichier XML de tâche
collecté est une divergence à examiner ; une collecte de triage peut simplement avoir omis ce fichier.
Les métadonnées de catalogue et de PE ne sont pas vérifiées et peuvent être absentes d'une collecte. Les vérifications du remplacement des fichiers
d'accessibilité dépendent de métadonnées de signature de référence appropriées ; l'absence de preuves de catalogue
ne doit pas entraîner le signalement de chaque exécutable d'accessibilité.

La couverture est enregistrée pour chaque vérification, y compris celles qui ne produisent aucun résultat. `complete` signifie que la
source collectée applicable a été lue ; cela ne prouve pas que la collecte est complète ni que
l'ordinateur ne présente aucune autre persistance. `partial`, `not_collected`, `failed` et `disabled` expliquent
les limites. Conservez `--coverage Coverage.csv` et le journal du parseur avec vos résultats.

Le portage utilise les formats CSV/JSON/JSONL de Juntei. Il n'appelle pas le programme Python amont, n'interroge pas
de services d'enrichissement en ligne et ne reproduit pas l'interface HTML/XLSX amont.

## Plus de détails { #further-details }

- [Implémentation et profils de PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper)
- [Techniques de persistance MITRE ATT&CK](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft : Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [Inventaire des démarrages automatiques du Registre](../artifacts/persistence.md)
- [Services](../artifacts/services.md) et [TaskCache](../artifacts/taskcache.md)
