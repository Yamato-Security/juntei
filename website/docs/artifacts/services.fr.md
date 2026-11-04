# Services Windows { #windows-services }

## Ce qu'enregistre la configuration des services { #what-service-configuration-records }

Les services Windows et les pilotes du noyau sont configurés sous chaque jeu de contrôles (control set) de `SYSTEM`. Les enregistrements nomment
l'exécutable/le pilote ou la DLL du service, le mode de démarrage, le type de service, le compte, le groupe d'ordre de chargement, le démarrage
différé, le comportement en cas d'erreur et les actions que Windows effectue après un échec.

Les attaquants créent ou modifient des services pour la persistance, l'élévation de privilèges, le chargement de pilotes et le
déplacement latéral. Les logiciels légitimes utilisent exactement le même mécanisme ; les chemins, les comptes, la signature,
le contexte de création et les preuves d'exécution sont donc déterminants.

<span class="evidence-strength">Meilleure conclusion :</span> le jeu de contrôles analysé configurait le
service/pilote tel qu'indiqué. La configuration seule ne prouve pas que le service a démarré avec succès.

## Emplacement { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

Collectez `SYSTEM` avec ses journaux de transactions. Des ID d'événements tels que System 7045 et Security 4697 peuvent
enregistrer l'installation ; les événements de contrôle des services et de processus peuvent étayer un démarrage/une exécution effectifs.

## Précautions d'analyse { #analyst-cautions }

- `ImagePath` peut contenir des variables d'environnement, des guillemets, des arguments, des chemins de périphérique ou un chemin
  non entre guillemets comportant des espaces. Conservez-le tel quel avant de le normaliser.
- `ServiceDll` est essentiel pour les services hébergés par `svchost.exe` ; `ImagePath` seul peut ne nommer que l'hôte.
- `StartType=automatic` ne prouve pas que le service a démarré ni qu'il est resté activé.
- `ObjectName` est le compte configuré, pas nécessairement l'utilisateur qui a installé le service.
- Les pilotes et les services en mode utilisateur utilisent des indicateurs `ServiceType` et des conventions de chemin différents.
- Les actions en cas d'échec peuvent exécuter une commande sous le compte du service après un échec ; il s'agit d'une surface de persistance
  facile à manquer.
- D'autres jeux de contrôles peuvent conserver une configuration antérieure ; utilisez `IsCurrentControlSet`.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath` | Identité du système et ruche d'entrée. À conserver d'une collecte à l'autre. |
| `ControlSet`, `IsCurrentControlSet` | Jeu de contrôles source et indication que `Select\Current` le désigne ou non. Privilégiez le jeu courant pour toute affirmation sur l'état actif. |
| `ServiceName` | Nom de la clé de Registre/nom interne du service. Pivot solide vers les événements et les commandes du Service Control Manager. |
| `DisplayName`, `Description` | Texte destiné à l'utilisateur ; il peut s'agir de références à des ressources ou de texte contrôlé par l'attaquant. |
| `ImagePath` | Commande de l'exécutable/du pilote telle que configurée. Analysez les guillemets, les arguments, les variables d'environnement et les chemins de périphérique. |
| `ServiceDLL` | DLL chargée par un service hôte. Souvent la véritable charge utile des services à processus partagé. |
| `StartType`, `StartValue` | Mode de démarrage lisible/brut : boot (0), system (1), automatic (2), manual (3), disabled (4). |
| `DelayedAutostart` | Le service automatique démarre après la phase initiale du démarrage ; il permet tout de même la persistance. |
| `ServiceType`, `TypeValue` | Indicateurs décodés/bruts : pilote du noyau, pilote de système de fichiers, processus propre/partagé, service utilisateur, etc. |
| `ErrorControl` | Niveau de gestion des erreurs au démarrage du système/du service. Principalement un contexte opérationnel. |
| `ObjectName` | Compte de service configuré ou nom d'objet du pilote. Pas l'identité de l'installateur. |
| `Group` | Groupe d'ordre de chargement, particulièrement important pour les pilotes boot/system. |
| `FailureResetPeriod` | Nombre de secondes avant la remise à zéro du compteur d'échecs ; `4294967295` signifie jamais. |
| `FailureCommand` | Commande configurée pour une action d'échec `run_command`. Champ de persistance à forte valeur. |
| `FailureActions` | Liste complète des actions décodées (`restart`, `reboot`, `run_command`, `none`) et délais. |
| `FailureActionType`, `FailureActionDelayMs` | Première action, aplatie pour le filtrage ; examinez également la liste complète. |
| `FailureActionsRaw` | Octets d'origine lorsque le décodage est incomplet/endommagé. |
| `KeyLastWrittenUTC` | Dernière modification de la clé du service, pas nécessairement l'heure d'installation/de démarrage. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de rejeu du Registre. |
| `Problems` | Incohérence de type, structure d'échec malformée ou champs manquants/illisibles. |

## Stratégie d'investigation { #investigation-strategy }

1. Examinez les chemins non standard, les répertoires accessibles en écriture, les comptes inhabituels, les horodatages de clé récents et les commandes d'échec.
2. Développez les variables d'environnement selon la configuration de l'hôte concerné, et non celle de la machine de l'analyste.
3. Calculez l'empreinte et vérifiez la signature de l'exécutable ou de la DLL effectifs s'ils ont été collectés.
4. Corrélez le nom/chemin du service avec les événements 7045/4697, la création de processus, Prefetch, `$MFT` et l'USN.
5. Comparez les jeux de contrôles courant et non courants pour repérer une persistance modifiée/supprimée.

## Plus de détails { #further-details }

- [Microsoft : arborescence du Registre pour les services de périphériques et de pilotes](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft : SERVICE_STATUS / types et états des services](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft : actions en cas d'échec de ChangeServiceConfig2](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
