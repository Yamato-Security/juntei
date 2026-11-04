# Cache du Planificateur de tâches { #task-scheduler-cache }

## Qu'est-ce que TaskCache { #what-taskcache-is }

Le Planificateur de tâches de Windows conserve sous `SOFTWARE` un cache de Registre reliant les chemins des tâches à des GUID,
aux déclencheurs/listes, aux principaux de sécurité, aux actions, aux métadonnées d'exécution et aux descripteurs de sécurité. Juntei décode
les actions d'exécution et de gestionnaire COM, ainsi que les anciennes actions d'e-mail/de message, les horodatages des tâches et les indices
de tâches masquées dans l'interface normale du Planificateur de tâches.

Les tâches sont un mécanisme courant de persistance et d'exécution à distance. Le cache du Registre est également précieux
lorsqu'un attaquant supprime ou altère le fichier XML de la tâche.

<span class="evidence-strength">Meilleure conclusion :</span> le TaskCache a configuré ou conservé la
tâche/l'action. Les champs d'exécution et des journaux indépendants sont nécessaires pour prouver une exécution particulière.

## Emplacements { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

Collectez `SOFTWARE` avec ses journaux de transactions et l'arborescence complète de `System32\Tasks`. Les journaux d'événements
Operational du Planificateur de tâches et Security apportent des preuves de création/de mise à jour/d'exécution.

## Précautions d'analyse { #analyst-cautions }

- Le cache du Registre et le XML de la tâche peuvent diverger après une altération, une suppression, une inscription incomplète ou des dommages.
- Le champ `Hidden` de Juntei identifie un masquage structurel (absence de liaison Tree/de descripteur de sécurité dans
  les conditions pertinentes), et non simplement une tâche configurée avec l'indicateur d'interface « Masqué » habituel.
- Windows 7 stocke les actions des tâches dans le XML plutôt que dans la valeur `Actions` du cache ; une action vide n'y est
  donc pas automatiquement suspecte.
- `LastRun` et `LastResult` sont utiles, mais ne constituent pas un historique d'exécution complet et infalsifiable.
- `Context` nomme le principal/contexte configuré, et non l'utilisateur qui a inscrit la tâche.
- Les CLSID des gestionnaires COM nécessitent une résolution distincte dans le Registre/les fichiers.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath` | Contexte de l'hôte et ruche `SOFTWARE` source. |
| `TaskName` | Chemin complet dans le planificateur. Pivot principal vers le XML et les journaux d'événements. |
| `TaskGUID` | GUID reliant les entrées Tree et Tasks du cache. Clé de corrélation solide. |
| `TaskIndex` | Métadonnées Tree/d'index utilisées par le planificateur ; élément de contexte, et non nombre d'exécutions. |
| `TaskLists` | Listes du cache (`Boot`, `Logon`, `Plain`, `Maintenance`) qui catégorisent le déclenchement/le traitement. |
| `Hidden` | Indicateur structurel de tâche masquée. Examinez `InTree`, `InTasks` et l'état du descripteur avant de conclure à une altération. |
| `InTree`, `InTasks` | Indiquent si les deux côtés du cache existent. Une liaison manquante peut indiquer une suppression/une altération/une inscription incomplète. |
| `HasSecurityDescriptor` | Indique si l'entrée Tree comporte `SD` ; son absence peut contribuer au masquage de la tâche sur les versions concernées. |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | Modifications au niveau des clés pour chaque côté du cache ; il ne s'agit pas des heures exactes de création/d'exécution. |
| `ActionIndex` | Action (numérotée à partir de zéro) au sein de la tâche. Une tâche peut produire plusieurs lignes. |
| `ActionType` | `exec`, `com_handler`, `send_email` ou `show_message`. |
| `Executable`, `Args`, `WorkingDirectory` | Action d'exécution effective. Interprétez les guillemets, les variables, les chemins relatifs et les LOLBins. |
| `COM-ClassID`, `ActionData` | CLSID et données du gestionnaire COM, ou charge utile d'un ancien e-mail/message. Résolvez le CLSID séparément. |
| `Context` | Contexte du principal encodé avec les actions (`LocalSystem`, `Author`, etc.) ; identité configurée, et non auteur de l'inscription. |
| `Author`, `URI`, `Description` | Métadonnées de la tâche. Facilement contrôlées par l'attaquant, elles ne constituent pas une preuve de confiance. |
| `LastRegisteredUTC` | Heure d'inscription/de mise à jour de DynamicInfo, lorsqu'elle est présente. |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | Métadonnées d'exécution/de résultat mises en cache. Corrélez avec les journaux opérationnels et les preuves de processus. |
| `ActionsRaw` | Octets d'origine des actions si le décodage est incomplet ou si une validation est nécessaire. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Incohérence chemin/GUID, type de valeur incorrect, structure d'action tronquée ou éléments manquants. |

## Stratégie d'investigation { #investigation-strategy }

1. Examinez d'abord les tâches masquées/incohérentes et les chemins non Microsoft.
2. Reconstituez les commandes effectives, y compris les arguments, le répertoire de travail, l'expansion des variables d'environnement et la résolution COM.
3. Comparez l'enregistrement du Registre avec le XML de la tâche, et calculez l'empreinte/relevez les horodatages de l'exécutable référencé.
4. Corrélez `TaskName`/le GUID/l'heure avec les événements du Planificateur de tâches, Security 4698/4702, les journaux de processus, Prefetch et la MFT/l'USN.
5. Recherchez dans les données de Registre supprimées/récupérées lorsqu'un attaquant a supprimé la tâche.

## Plus de détails { #further-details }

- [Microsoft : schéma du Planificateur de tâches](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft : journal d'événements du Planificateur de tâches](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
