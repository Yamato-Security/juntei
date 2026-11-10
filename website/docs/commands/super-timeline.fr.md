# Chronologie globale { #super-timeline }

`super-timeline` convertit les artefacts analysés en événements de schéma commun. La sortie n'est **pas triée par défaut** : elle suit l'ordre de découverte des fichiers d'entrée et l'ordre de projection de chaque parseur, quel que soit l'ordre dans lequel les threads se terminent. Ajoutez `--sort` pour un tri chronologique et la consolidation des doublons. Chaque heure d'exécution d'un enregistrement Prefetch et chacun des huit horodatages ou plus d'un enregistrement MFT devient un événement distinct. La comparaison entre sources conserve ainsi le sens de chaque date.

Partez des résultats de [`auto-parse`](auto-parse.md) plutôt que du répertoire de preuves originales :

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

Conservez les fichiers analysés, les manifestes et les originaux. Cette projection d'analyse ne les remplace pas et ne crée pas un stockage Plaso natif.

## Utilisation { #usage }

```bash
# Native CSV (default)
juntei super-timeline -d parsed/ -o SuperTimeline.csv

# Chronological CSV with duplicate consolidation
juntei super-timeline -d parsed/ -o SortedTimeline.csv --sort

# Native JSON Lines
juntei super-timeline -d parsed/ -o SuperTimeline.jsonl -t jsonl

# Combine native records and an existing Hayabusa JSONL timeline
juntei super-timeline -f NativeRecords.jsonl --hayabusa Hayabusa.jsonl \
  -o SuperTimeline.jsonl -t jsonl

# Timesketch import profile; also available with CSV
juntei super-timeline -d parsed/ -o Timesketch.jsonl -t jsonl --profile timesketch

# One standalone CSV needs an artifact identifier and, if unavailable, a computer name
juntei super-timeline -f MFT.csv --input-artifact mft --computer HOST01 \
  -o Host01Timeline.csv

# Reduce the time range, memory used by sorting, or number of workers
juntei super-timeline -d parsed/ -o IncidentTimeline.jsonl -t jsonl \
  --timeline-start "2026-10-20T06:00:00Z" --timeline-end "2026-10-20T14:00:00Z" \
  --sort --memory-limit 64 -T 4 --temp-dir scratch/
```

`--hayabusa FILE` s'utilise seul ou plusieurs fois pour importer dates et contexte des alertes sans exécuter Hayabusa. Les événements informatifs dont `Details` est une chaîne sont acceptés. `--artifact mft,prefetch` limite les artefacts. Les bornes sont inclusives et exigent un décalage UTC explicite ou `Z`.

### Choisir une entrée { #choosing-an-input }

| Entrée | Identification |
|---|---|
| `-d DIR` | `manifest.json` fournit tâches principales réussies, identifiants, ordinateurs et fichiers exacts. Les tâches secondaires `auto-parse -j` sont exclues pour éviter les doublons. |
| `-f FILE` | Le format natif JSON/JSONL utilise `Artifact` et `Comp`. Les exports non identifiés nécessitent `--input-artifact`, toujours obligatoire pour un CSV autonome. |
| `--computer NAME` | Nom de secours en l'absence de nom d'ordinateur ; les ordinateurs inconnus sont explicitement `Unknown`. |
| `--hayabusa FILE` | JSONL contenant dates Hayabusa, métadonnées de règle, ordinateur et éventuellement `EvtxFile`. |

Les manifestes actuels permettent de déplacer les résultats : les chemins absolus sont résolus sous le nouveau répertoire. Toutes les entrées, y compris les cibles des liens symboliques, doivent y rester. Les anciens répertoires Amcache, SRUM et Jump Lists sont acceptés ; les répertoires par catégorie exigent `OutputFiles` explicite.

Les chemins relatifs à la sortie et exports multi-tables aplatis sont acceptés. Tous les artefacts programmés par l'analyse automatique sont reconnus, dont navigateur, WebCache, Outlook et UAL.

Pour une image VHDX ou E01 identifiée dans le manifeste, les références contiennent l'image et le chemin interne. Le nom de l'image reste dans le contexte ; le répertoire temporaire n'est plus nécessaire pour lire les résultats.

Une source brute est extraite de l'enregistrement ou d'une commande manifeste sans ambiguïté. Avec `-d`, les références de source brute sont relatives au répertoire de preuves, et `Details.ParsedFile` et le résumé `InputFiles` sont relatifs au répertoire analysé. Les noms des sorties/rapports récapitulatifs sont relatifs au répertoire de sortie de la timeline. Les tables SRUM CSV sont affectées à une base de données uniquement lorsque l'hôte possède un fichier SRUDB.dat collecté de manière unique et identifiable. Sinon, `SourceFile` pointe vers le fichier analysé et `Details.RawSourceAvailable` est `false`.

Une entrée autonome est référencée par son nom de fichier. Les entrées portant le même nom de fichier utilisent des références `./Inputs/<ID>/<filename>` distinctes. Les chemins absolus de source brute en dehors des racines de preuves connues utilisent des identifiants `./Sources/<ID>/<filename>`. Ces références opaques conservent les distinctions de source sans exporter les noms de répertoire absolus de l'analyste ; consultez l'enregistrement natif en utilisant `ParsedFile` et `InputRecord` pour la source originale. Les chemins enregistrés dans les preuves, tels que les chemins exécutables Windows, restent inchangés.

## Champs communs { #common-event-fields }

| Champ | Signification |
|---|---|
| `Timestamp` | UTC RFC 3339 avec neuf décimales pour un tri cohérent. Le remplissage ne crée pas de précision. |
| `Comp` | Ordinateur associé à l'enregistrement d'entrée. |
| `Artifact` | Identifiant stable : `mft`, `prefetch`, `srum` ou `hayabusa`, par exemple. |
| `TimestampType` | Sens de l'horodatage, par exemple `MFT SI Data modified`. |
| `User` | Compte ou SID enregistré si disponible ; peut être vide. |
| `Path` | Fichier, clé de registre ou autre objet propre à l'artefact. |
| `Message` | Description courte pour la visualisation. |
| `SourceFile` | Référence de source brute relative aux preuves, ou un identifiant de source relatif lorsque sa racine est inconnue ; sinon, la solution de secours du fichier analysé décrite ci-dessus. |
| `SourceField` | Champ exact ou emplacement imbriqué fournissant la date. |
| `RecordID` | Identité propre à l'artefact, pas nécessairement un identifiant d'enregistrement EVTX ID. |
| `Details` | Contexte structuré : date originale, précision, base et rôle temporels, fichier analysé, position et métadonnées de récupération/règle disponibles. |

En CSV, `Details` est une cellule JSON entre guillemets ; en JSONL, un objet. Le profil Timesketch renomme `Timestamp`, `TimestampType` et `Message` en `datetime`, `timestamp_desc` et `message` sans perdre les autres références. Ce sont les [trois champs requis par Timesketch](https://timesketch.org/guides/user/import-from-json-csv/).

`Details` contient un contexte sélectionné, pas toutes les données natives ou binaires. Les chaînes et structures longues sont limitées et marquées `ProjectionOmitted`. Consultez `Details.ParsedFile`, `InputRecord` et `Location` pour retrouver l'original. En CSV, le numéro compte les enregistrements CSV après l'en-tête, pas les lignes internes des cellules ; en JSONL, les lignes ; en JSON, le document et la position imbriquée.

## Interpréter les dates { #what-the-times-mean }

Une date renseigne sur un objet enregistré ; elle ne prouve seule ni exécution ni attaque. Les adaptateurs distinguent :

| Artefacts | Interprétation chronologique |
|---|---|
| MFT | Dates SI et FN de modification, accès, changement de métadonnées et création ; les attributs FN nommés restent distincts. Avec `--sort`, les SI répétés sont consolidés. Les dates partagées du fichier de base des lignes ADS sont exclues dans les deux modes. Les noms récupérés dans le slack d'enregistrements FILE conservent leurs événements FN récupérés. |
| I30 | Métadonnées FN/index avec contexte de récupération ou slack, pas dates d'écriture de l'index de répertoire. |
| Journal USN | Date du journal et motifs de changement : plusieurs modifications peuvent partager une entrée sans heure de début exacte. |
| LogFile | Instantanés temporels et noms des opérations redo/undo, conservés séparément. LSN et positions de fichier sont des identifiants, jamais des dates. |
| Prefetch | Exécutions enregistrées. Avec `--sort`, elles sont consolidées avec la chronologie Prefetch correspondante. La création du volume est étiquetée séparément. |
| Bases SQLite des navigateurs | Visites, dernières visites récupérées, débuts/fins de téléchargements, dates des cookies et première/dernière utilisation ou suppression des formulaires. Une fin ne prouve ni réussite ni exécution ; l'expiration est une échéance configurée. Récupération, confiance et origine sont conservées. |
| Internet Explorer / ancien Edge WebCache | Accès, création/modification, expiration, synchronisation, téléchargements, cookies et entretien des conteneurs. La gestion du cache ne prouve pas automatiquement une visite de l'utilisateur. |
| Outlook PST, OST, MSG et OFT | Envoi/réception ; création/modification des dossiers, éléments et pièces jointes ; dates de calendrier, tâches, rappels et propriétés temporelles typées. Une date planifiée ne prouve ni réunion, ni rappel déclenché, ni exécution. La réception ne prouve pas la lecture. |
| Journalisation des accès utilisateur (UAL) | Premier/dernier accès, insertion/création de base, première/dernière observation et démarrage système enregistré. Rôle, compte, adresse et compteurs quotidiens restent du contexte. Aucune connexion à minuit n'est inventée ; identités et lignes DNS sans date restent dans le rapport complémentaire. |
| Amcache | Écritures des clés d'inventaire, contexte du volume parent, installation et liaison PE. La présence dans l'inventaire ou LinkDate ne prouve pas l'exécution. |
| ShimCache | Modification de fichier, mise à jour du cache et écriture de clé : aucune n'est automatiquement une heure d'exécution. |
| SRUM | Date/intervalle du fournisseur, début de connexion, bornes explicites et métadonnées d'application. Octets, durées et compteurs ne sont pas des dates. |
| LNK | Dates MAC de la cible, création des identifiants de suivi et éléments shell : métadonnées stockées, pas forcément ouverture du raccourci. |
| Jump Lists | Dernière utilisation, identifiant de suivi et dates de cible LNK intégrées. Les dates du conteneur sur le système de collecte sont omises par défaut. |
| Quarantaine Defender | Date de l'entrée et création/accès/écriture de la ressource. L'extraction n'est pas une activité du poste d'origine. |
| BAM, UserAssist, FeatureUsage | Activité enregistrée et écriture de clé distinguées. Un compteur sans date ne produit aucun événement. |
| RecentDocs, MRU, ouverture/enregistrement MRU, ShellBags, Office | Utilisation, confiance, éléments shell et contexte de clé. La date de la clé d'une liste n'est pas la dernière utilisation de chaque élément. |
| SAM et secrets LSA | Connexions, mots de passe, expiration, secrets et dates de clés ; ces métadonnées ne remplacent pas les journaux d'authentification. |
| Services, TaskCache, persistance du registre, BCD | Configuration, clés et dates explicites des tâches. Une écriture de clé de service ne prouve pas son exécution. |
| Réseaux et USB | Profils/baux réseau, installation/arrivée/retrait des périphériques et clés. Les dates natives USB des MountPoints2 utilisateur conservent leur ruche utilisateur d'origine. |
| Informations système, programmes installés, AppCompat, historique applicatif | Installation, arrêt, inventaire/activité et clés. Les tableaux natifs de profils/versions conservent leurs dates imbriquées. |
| Audit, paramètres de sécurité, MuiCache, registre générique | Dernière écriture de clé si disponible ; valeurs et compteurs sans date restent dans l'entrée native. |
| Persistence Hunter | Écriture de clé et changement corrélé. Un résultat ne prouve seul ni installation datée ni compromission. |
| Hayabusa | Date d'événement/alerte, règle, niveau, canal, ID et contexte. Les règles et lignes de corrélation distinctes restent séparées. |

L'export par défaut conserve toutes les projections de dates, y compris les métadonnées répétées, sans tri global ni consolidation des doublons. Avec `--sort`, les projections en double sont consolidées ; un export trié peut donc compter moins de lignes qu'un export non trié des mêmes entrées. Les dates égales de sens ou de sources différents restent distinctes dans les deux modes. L'identité des doublons inclut ordinateur, source, artefact, enregistrement, sens et récupération. Les consolidations indiquent `DuplicateCount` et des références alternatives limitées. Des sources différentes ne fusionnent jamais sur la seule similarité des chemins/dates. Sans source brute, l'identité d'entrée protège les captures distinctes.

### Précision, heure locale et hypothèses { #precision-local-times-and-assumptions }

Les champs UTC connus et décalages explicites sont normalisés en UTC. Originaux et précision, dont FILETIME à 100 ns, restent conservés ; une précision déjà arrondie ne peut être restaurée.

Certains champs shell/FAT sont locaux même sérialisés avec `Z`. La valeur et l'origine sont conservées, mais pas le fuseau manquant ni une conversion antérieure vérifiable. Contrôlez l'artefact et le poste avant corrélation temporelle précise.

Les dates sans fuseau défini par l'artefact sont signalées, pas supposées UTC. Si les preuves le justifient, fournissez un décalage fixe signé :

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Les dates seules exigent aussi cette option : leur minuit est une hypothèse de **précision journalière**, pas un événement observé. Noms de fuseaux et heure d'été sont refusés. Absence, zéro, erreur ou dépassement ne créent pas d'événement d'époque artificiel ; une vraie date 1970 reste acceptée.

Deux catégories sont exclues par défaut :

- `--include-collection-times` : métadonnées des fichiers sources LNK/Jump List sur le système de collecte/analyse, étiquetées `collection_filesystem` et séparées de l'activité du poste.
- `--include-interpreted-times` : valeurs QWORD du registre générique interprétées en FILETIME. Elles portent `interpreted_value` car beaucoup sont des compteurs.

## Rapports complémentaires { #companion-reports }

Pour `SuperTimeline.csv`, la commande écrit également :

- `SuperTimeline.csv-Untimed.jsonl` : emplacements natifs sans horodatage exploitable associé.
- `SuperTimeline.csv-Diagnostics.jsonl` : dates invalides/ambiguës, métadonnées communes volontairement exclues et limites nécessitant l'original.
- `SuperTimeline.csv-Summary.json` : fichiers, compteurs par artefact, événements filtrés/candidats/consolidés, sorties, threads, activation du tri, budget/fichiers/passes de tri et durée.

Les noms de compagnon incluent le nom complet du fichier de sortie. Par exemple, `SuperTimeline.jsonl` possède son propre `SuperTimeline.jsonl-Summary.json` et les rapports non chronométrés/diagnostiques correspondants. Les exportations CSV et JSONL avec la même tige peuvent coexister ; `-C` est nécessaire uniquement pour remplacer un jeu de résultats existant.

Dates valides en attente d'un incrément d'hypothèse de fuseau horaire justifié `UnresolvedTimeZones`, plutôt que `InvalidTimestamps`. Les deux restent traçables dans les diagnostics ; aucun horodatage n'est inventé.

Les dates hors intervalle sont filtrées, pas sans date. Les candidats précèdent la consolidation. `Sorted` et `DuplicateConsolidation` indiquent si `--sort` était activé. Sans cette option, `DuplicateEvents`, `LargestSerializedSortRowBytes`, `InitialSortChunks` et `MergePasses` valent zéro et `SortMemoryBudgetMiB` vaut `null`, car aucun tampon de tri n'est alloué. `Complete: true` signifie traitement réussi des entrées choisies, pas collecte exhaustive, date dans chaque champ ou détection de toutes les attaques.

CSV peut aplatir profils système, montages utilisateur USB et éléments shell ; JSON/JSONL natifs sont plus riches. Les données omises par CSV ne se reconstruisent pas. Un artefact inconnu ou fichier malformé provoque une erreur, pas un succès partiel.

## Performances et protection des sorties { #performance-and-output-safety }

Les entrées sont lues en flux et traitées en parallèle. Par défaut, chaque thread écrit un fragment d'événements formatés ; les fragments sont concaténés dans l'ordre des fichiers d'entrée, sans tri ni relecture de leurs structures d'événements. Le CSV n'a qu'un en-tête, même avec plusieurs fichiers. L'ordre de sortie est déterministe quel que soit le nombre de threads, mais n'implique aucune chronologie.

Avec `--sort`, les tampons débordent dans un répertoire temporaire privé, puis des passes de fusion bornées produisent une sortie chronologique. Le tri utilise secondes et nanosecondes entières, dates antérieures à 1970 comprises. L'ordre des égalités reste stable entre budgets et nombres de threads. Une barre de progression avance à chaque événement traité pendant les fusions intermédiaires et finales, passes répétées comprises. Elle s'affiche dans un terminal et est masquée quand la sortie est redirigée.

`--memory-limit` ne s'applique qu'avec `--sort` : c'est le **budget total des tampons de tri**, pas une limite de mémoire RSS du processus. Enregistrements, objets/chaînes JSON, tampons CSV et fusions ajoutent de la mémoire ; les petits budgets réduisent les threads. `--max-record-size` limite un enregistrement à 16 MiB par défaut. Un tableau JSON formaté compte comme un document entier ; augmentez la limite ou préférez JSONL. La lecture anticipée CSV compte aussi.

L'espace temporaire doit contenir les fragments d'événements formatés (par défaut) ou les enregistrements de tri développés et les passes de fusion (`--sort`), en plus des résultats. Les deux modes préparent les sorties dans un espace temporaire et empêchent toute publication si une entrée échoue. Un enregistrement peut produire beaucoup d'événements : la chronologie peut largement dépasser l'entrée. Filtrez la période ou les artefacts si nécessaire et choisissez un volume temporaire assez libre.

Ctrl+C annule de manière coopérative, supprime les répertoires de tri/mise en scène privés et quitte avec le code `130`. Sur Linux et macOS, cela s'applique également à SIGTERM et SIGHUP. L'annulation est vérifiée lors de la lecture, de la normalisation, de la fusion et de la rédaction des rapports, avant le début de la publication. Une interruption irrécupérable ou une coupure de courant peut toujours laisser des fichiers temporaires.

Les sorties existantes nécessitent `-C`. Les entrées, manifestes et preuves protégées ne peuvent être écrasés. Les erreurs de lecture/tri préservent les anciens résultats. Les fichiers sont préparés près de la destination et remplacés individuellement, chronologie en dernier : **pas** de transaction des quatre fichiers. Une erreur finale peut avoir remplacé certains rapports. Utilisez un nouveau nom de base pour conserver un export antérieur complet.

## Pour approfondir { #further-details }

- [Import Timesketch CSV/JSONL](https://timesketch.org/guides/user/import-from-json-csv/)
- [Développement des dates Plaso et ancien CSV](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Sortie et formatage Plaso](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Preuves et interprétation](../overview/evidence-model.md)
- [Formats de sortie](../getting-started/output-formats.md)

Les entrées Juntei utilisent `Comp`, `Filename`, `Args` et `Attr`. Le MFT CSV accepte aussi `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` et `HasADS` en préservant générations de séquence et distinctions DOS/ADS. Les anciens noms restent acceptés ; `SourceField` garde l'orthographe originale. Hayabusa conserve `Computer`. Secteurs d'amorçage et descripteurs de sécurité restent au rapport sans date, faute d'heure d'événement.

## Nouveaux parseurs multi-tables { #newer-multi-table-parsers }

Le JSON/JSONL natif contient le nom de table. En CSV autonome, navigateurs, WebCache et Outlook utilisent le schéma des colonnes : renommer le CSV ou ses suffixes ne change pas l'interprétation. UAL utilise ses champs temporels connus sans deviner des dates à partir des compteurs ou du texte.

Les propriétés Outlook ne sont projetées que si leur type déclaré est temporel, simple ou multiple. Une date textuelle reste du texte. Les messages intégrés gardent identifiants et parents ; les dates d'extraction ne remplacent pas celles des pièces jointes. Les lignes sans date restent traçables dans les données natives et le rapport complémentaire.

Les schémas CSV/JSONL restent les mêmes ; un nouvel artefact n'ajoute pas des centaines de colonnes CSV. Le contexte spécifique reste dans `Details`.
