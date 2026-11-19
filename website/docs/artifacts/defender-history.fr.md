# Historique des détections Microsoft Defender { #microsoft-defender-detection-history }

## Ce que l'historique enregistre { #what-the-history-records }

Microsoft Defender écrit un fichier d'historique de détection pour chaque détection. Chaque fichier enregistre :

- l'identifiant, le nom, la gravité et la catégorie de la menace ;
- ce que Defender a fait et si l'action a réussi ;
- si la menace était en cours d'exécution ;
- l'utilisateur et le processus impliqués, et ce qui a détecté la menace ;
- l'entrée de quarantaine, s'il y en a une ;
- chaque ressource dans laquelle la menace a été trouvée : fichiers, membres d'archives, téléchargements, processus, valeurs du Registre,
  services, lignes de commande et comportements.

Defender écrit aussi ces fichiers pour les menaces qu'il n'a jamais mises en quarantaine : celles qu'il a seulement détectées, celles
qui ont été autorisées et celles qu'il n'a pas réussi à supprimer. L'historique de protection de Sécurité Windows et la classe WMI
`MSFT_MpThreatDetection` (`Get-MpThreatDetection`) affichent ces enregistrements. Lorsque Orange
Cyberdefense a modifié des valeurs dans un fichier, WMI a renvoyé les valeurs modifiées.

<span class="evidence-strength">Meilleure conclusion :</span> Defender a enregistré une détection de la menace
nommée dans les ressources listées, avec le statut et les heures enregistrés. Le nom de la menace est le verdict
du produit. Un enregistrement de détection seul ne montre ni que la menace s'est exécutée, ni qu'elle a été
entièrement supprimée.

## Emplacement et collecte { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

Chaque fichier porte le nom de son `DetectionID`. Collectez l'intégralité du dossier `DetectionHistory`. L'entrée par
répertoire et `auto-parse` lisent chaque fichier situé dans un dossier `DetectionHistory`, à n'importe quelle profondeur, qui commence
comme un fichier d'historique de détection. Utilisez `-f` pour une copie conservée hors d'un tel dossier. Defender supprime
les éléments de son historique d'analyse après un nombre de jours configurable (`ScanPurgeItemsAfterDelay` dans
`Set-MpPreference`) ; les détections plus anciennes peuvent donc avoir disparu.

La quarantaine est un artefact distinct : voir [Quarantaine Defender](defender-quarantine.md). Le
`QuarantineID` d'une détection utilise la même graphie que l'`EntryID` de la sortie de quarantaine (en majuscules,
sans accolades), ce qui permet de relier les deux.

## Précautions d'analyse { #analyst-cautions }

- La détection, une tentative de correction et une suppression réussie sont des états distincts. Lisez
  `ThreatStatus` (`Detected`, `Quarantined`, `Removed`, `Allowed`, `RemoveFailed`, …) avec
  `ThreatStatusErrorCode`, qui vaut 0 lorsque le dernier changement de statut a réussi.
- `ExecutionStatus` est le point de vue de Defender au moment de la détection (`Blocked`, `Allowed`, `Executing`,
  `NotExecuting`, `Unknown`). Ce n'est pas une preuve que la menace s'est exécutée, ni qu'elle ne s'est jamais exécutée.
  Recoupez avec Prefetch, Amcache, les journaux d'événements et la télémétrie des processus.
- `ThreatName` est le nom de la signature à ce moment-là. Des faux positifs et une reclassification ultérieure sont
  possibles. Les fichiers de test comme EICAR sont signalés comme de vraies menaces.
- `User` est la valeur `DomainUser` enregistrée par Defender (`UserSource` vaut `DomainUser`). Elle peut
  être vide, correspondre à un compte de service ou désigner un autre utilisateur que celui qui a créé le fichier.
- Defender peut écrire plusieurs fichiers pour une même menace, et Juntei les conserve tous. Deux fichiers peuvent
  même contenir le même `DetectionID`, comme c'est le cas d'un fichier copié ou restauré. Regroupez à la fois par `DetectionID` et
  par `SourceFile`.
- Les valeurs dont la signification est inconnue sont conservées, et non interprétées : leur section et leur position figurent dans
  `UnknownValues`.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

CSV est le format de fichier par défaut. Avec `-t csv`, `-t json` ou `-t jsonl` sans `-o`, les enregistrements sont
écrits sur stdout. Sans aucune de ces options, seul un résumé s'affiche. Ajoutez `-C` pour écraser une sortie
existante. La sortie n'est jamais écrite par-dessus l'un des fichiers d'entrée, même avec `-C`.

Juntei écrit un enregistrement par ressource de chaque détection. Une détection sans ressources reçoit un
enregistrement dont les champs de ressource sont vides. Les enregistrements d'une même détection partagent son `DetectionID` et sont
numérotés par `ResourceIndex` dans l'ordre de stockage.

Les dommages n'interrompent pas l'exécution. Un fichier tronqué conserve les valeurs lues avant le dommage, et
`Problems` indique ce qui manque. Un fichier d'un dossier `DetectionHistory` qui n'est pas un fichier d'historique
de détection est signalé et ignoré. Les autres fichiers sont tout de même lus.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Record` | `resource`, ou `detection` pour une détection sans ressources. |
| `InitialDetectionTime` | Moment où Defender a détecté la menace pour la première fois (UTC). Vide lorsque le fichier contient 0 ; une valeur qui n'est pas une heure est conservée dans `UnknownValues`. |
| `LastThreatStatusChangeTime` | Moment du dernier changement de statut de la menace (UTC). |
| `RemediationTime` | Moment où Defender a corrigé la menace (UTC). C'est l'heure d'une tentative, et non une preuve de suppression ; vérifiez `ThreatStatus`. |
| `ThreatName`, `ThreatID` | Nom de signature et identifiant numérique de Defender (`Get-MpThreatCatalog`). |
| `Severity`, `SeverityID` | `Low`, `Moderate`, `High`, `Severe` ou `Unknown`, avec le nombre stocké. |
| `Category`, `CategoryID` | Catégorie de menace, comme `TROJAN`, `TOOL` ou `BEHAVIOR`, avec le nombre stocké. |
| `ThreatStatus`, `ThreatStatusID` | Statut de la menace selon Defender. Un nombre sans nom documenté est conservé avec un `ThreatStatus` vide. |
| `ThreatStatusErrorCode` | Code d'erreur du dernier changement de statut ; 0 signifie qu'il a réussi. |
| `ExecutionStatus`, `ExecutionStatusID` | Indique si Defender a vu la menace s'exécuter au moment de sa détection. |
| `DetectionSource`, `DetectionSourceTypeID` | Ce qui a détecté la menace : `Real-time`, `IOAV` (téléchargements et pièces jointes), `User`, `System`, `AMSI`, entre autres. |
| `User`, `UserSource` | La valeur `DomainUser` enregistrée par Defender, et sa base (`DomainUser`). Vide lorsque le fichier ne nomme aucun utilisateur. |
| `Process` | Le processus impliqué (`ProcessName`) ; peut valoir `Unknown`. |
| `ResourceIndex`, `ResourceCount` | Position de cette ressource, et nombre de ressources dans la détection. |
| `ResourceType` | `file`, `containerfile`, `webfile`, `process`, `regkeyvalue`, `service`, `CmdLine`, `behavior`, `amsi`, entre autres, tels que Defender les nomme. |
| `Path` | La ressource : un chemin, un membre d'archive (`archive.zip->member`), une valeur du Registre, une ligne de commande ou un processus. Un `webfile` indique aussi son URL et le processus qui l'a téléchargé (`pid:…,ProcessStart:…`). Conservez l'orthographe exacte. |
| `ResourceFlags` | Indicateurs de la ressource en hexadécimal ; leur signification n'est pas documentée. |
| `TrackingStartTime` | Le `ThreatTrackingStartTime` de la ressource, un entier lu comme un FILETIME (UTC), de la même façon que plaso date une détection. |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | Empreintes et taille issues des données de suivi de la ressource. Elles décrivent le fichier au moment où Defender l'a analysé. |
| `TrackingID` | Le `ThreatTrackingId` de la ressource. |
| `OtherTracking` | Toutes les autres valeurs de suivi, sous la forme `Key=Value` dans l'ordre de stockage, comme `ThreatTrackingScanSource` ou `ThreatTrackingBMContextRich` pour les détections comportementales. |
| `QuarantineID` | L'entrée de quarantaine (`Quarantine\Entries\{GUID}`) ; vide lorsque le fichier contient un GUID nul. |
| `DetectionID` | Le GUID de la détection ; c'est aussi le nom du fichier. Une non-concordance est signalée dans `Problems`. |
| `MagicVersion` | Le `Magic.Version:1.2` de l'en-tête ; vide dans les fichiers écrits sans lui, comme certains fichiers de Windows Server 2016. |
| `UnknownValues` | Valeurs dont la signification est inconnue, sous la forme `position=value` (par exemple `6.13=NT AUTHORITY\SYSTEM`) ; la liste comptée qui suit le statut figure sous `2.list`. |
| `SourceFile` | Le fichier lu, relatif au répertoire d'entrée, ou son nom avec `-f`. |
| `Problems` | Dommages, valeurs manquantes, types inattendus ou nom de fichier non concordant. Ne les masquez pas. |

JSON et JSONL conservent les mêmes champs avec des valeurs typées : les nombres restent des nombres et les valeurs manquantes valent
`null`. À la place de `OtherTracking`, ils contiennent `Tracking`, chaque valeur de suivi avec la clé de Defender,
son type et sa valeur. `UnknownValues` devient une liste de positions avec leurs types. `CountedList`
contient la liste comptée. Un FILETIME stocké conserve son nombre brut de ticks à côté de l'heure qu'il représente.

## Comportement dans la chronologie { #timeline-behavior }

`auto-parse` enregistre les données dans `SecurityProducts/DefenderHistory.csv` (ou `.json`/`.jsonl`) et
les ajoute à la chronologie combinée `-j`. `super-timeline` crée des événements à partir de
`InitialDetectionTime`, `LastThreatStatusChangeTime` et `RemediationTime` dans l'enregistrement de chaque ressource,
avec le statut, le code d'erreur, le statut d'exécution et la source de détection dans les détails de
l'événement. `TrackingStartTime` est stocké sous forme d'entier ; il ne devient donc un événement qu'avec
`--include-interpreted-times`. Les heures propres au fichier ne sont jamais utilisées. Un enregistrement sans heure exploitable
reste dans la sortie analysée et est compté comme non daté.

## Stratégie d'investigation { #investigation-strategy }

1. Partez de `ThreatName`, `Path`, `SHA256`, `InitialDetectionTime` et `ThreatStatus`.
2. Vérifiez `ThreatStatusErrorCode` et `Problems` avant de conclure qu'une menace a été supprimée.
3. Reliez `QuarantineID` à l'`EntryID` de la quarantaine pour récupérer les octets mis en quarantaine.
4. Pivotez sur les empreintes et les chemins vers `$MFT`, le journal USN, Prefetch, Amcache, l'historique du navigateur et les journaux
   d'événements (journal opérationnel de Defender : ID d'événement 1116, 1117, 1118 et 1119).
5. Pour les ressources `webfile`, utilisez l'URL et le démarrage du processus pour enquêter sur l'accès initial, et
   validez-les avec les preuves du navigateur, du proxy et du DNS.

## Versions testées { #tested-versions }

Les tests utilisent des fichiers publics dont d'autres outils ont lu les résultats de manière indépendante :

- L'échantillon Windows 10 d'Orange Cyberdefense (Defender 4.18.24090.11), vérifié par rapport aux valeurs renvoyées
  par WMI.
- Les deux fichiers de test Windows de plaso, vérifiés par rapport au parseur `windefender_history` de plaso.
- 14 fichiers de la VM du simulateur d'APT de DFIRArtifactMuseum.
- Une copie écrite sans la version de l'en-tête, comme le sont certains fichiers de Windows Server 2016.
- Des fichiers tronqués, vides ou d'un autre type.

## Plus de détails { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [wddh-parser d'Orange Cyberdefense](https://github.com/cert-orangecyberdefense/wddh-parser)
- [Parseur d'historique Defender de plaso](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft : classe MSFT_MpThreatDetection](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft : ID d'événement de l'Antivirus Microsoft Defender](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
