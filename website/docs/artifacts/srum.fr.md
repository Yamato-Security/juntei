# SRUM { #srum }

## Qu'est-ce que SRUM ? { #what-srum-is }

Le moniteur d'utilisation des ressources système (System Resource Usage Monitor, SRUM) stocke une base de données ESE de l'utilisation des ressources par les applications et les
services. Selon la version de Windows, il enregistre les octets réseau, les identifiants d'interface/de profil,
les connexions, le temps CPU, l'activité au premier plan/en arrière-plan, l'activité disque, la consommation d'énergie, les notifications
push et les données de chronologie des applications. Les entrées sont associées à des identifiants d'application et à des SID d'utilisateur
au moyen de `SruDbIdMapTable`.

SRUM est particulièrement utile lorsqu'un exécutable a été supprimé : son identifiant d'application, son utilisateur et ses lignes
d'utilisation des ressources peuvent survivre au fichier. Les totaux réseau peuvent étayer une hypothèse d'exfiltration, mais
SRUM n'est pas une capture de paquets et n'identifie généralement pas chaque point de terminaison distant ni chaque fichier transféré.

<span class="evidence-strength">Meilleure conclusion :</span> Windows a attribué l'utilisation des ressources enregistrée
à l'application/au compte pendant la période de mesure de la table.

## Emplacement et collecte { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM existe sous Windows 8 et versions ultérieures. Copiez `SRUDB.dat` de manière cohérente, idéalement à partir d'un instantané ou avec
un collecteur forensique. Collectez également `SOFTWARE` ainsi que les fichiers de transactions/journaux ESE du répertoire
`sru`.

La ruche SOFTWARE correspondante est distincte de la base ESE : lorsqu’elle est non synchronisée, Juntei rejoue automatiquement en mémoire ses journaux de transactions du Registre voisins `SOFTWARE.LOG`, `.LOG1` et `.LOG2`. Collectez-les également. Les détails de récupération et avertissements sont affichés ; sans ces journaux, les noms de comptes ou Wi-Fi peuvent être incomplets ou anciens. Cela ne rejoue pas les journaux de la base SRUM.

!!! warning "Bases de données « dirty »"
    Juntei lit un `SRUDB.dat` dans un état « dirty » tel qu'il a été collecté et le signale. Il ne rejoue pas actuellement
    `SRU*.log` ; les modifications présentes uniquement dans ces journaux sont donc absentes. Ne décrivez pas implicitement une
    base de données « dirty » comme complète.

## Précautions d'analyse { #analyst-cautions }

- De nombreuses lignes représentent une agrégation périodique, généralement autour de limites horaires, et non l'heure exacte d'une action.
- Les totaux d'octets n'identifient pas à eux seuls le contenu, l'intention, le protocole ou la destination.
- Une identité d'application peut être un chemin, un service, une application empaquetée ou un objet ; examinez `AppType`.
- Les noms d'utilisateur et de réseaux Wi-Fi sont un enrichissement provenant de la ruche `SOFTWARE` correspondante. L'absence de noms ne signifie pas que
  l'identifiant sous-jacent est absent.
- `null`, zéro, les colonnes absentes, les types non pris en charge et les valeurs endommagées sont des états différents.
- Les schémas des fournisseurs varient selon la version de Windows. Conservez les tables inconnues et les noms de colonnes d'origine.
- Les services Windows en arrière-plan génèrent de manière légitime des totaux de ressources importants.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## Tables de sortie { #output-tables }

Le format CSV crée `SRUM-IdentityMap.csv` pour `SruDbIdMapTable` et une table par fournisseur. Les noms de sortie connus sont notamment :

| Fournisseur/table | Intérêt pour l'investigation |
|---|---|
| `NetworkUsages` | Octets envoyés/reçus par application, utilisateur, interface et profil. |
| `NetworkConnections` | Activité de type nombre/durée de connexions et heures de début de connexion. |
| `AppResourceUseInfo` | Utilisation du CPU, du disque, du réseau au premier plan/en arrière-plan et utilisation associée des applications. |
| `AppTimelineProvider` | Activité de chronologie des applications avec des champs d'heure de fin et d'état d'utilisation. |
| `EnergyUsage` | Mesures d'énergie/des fournisseurs à court et à long terme ; `IsLt` les distingue. |
| `PushNotifications` | Activité des notifications push. |
| `VfuProvider` | Le fournisseur `vfuprov` ; son rôle détaillé est mal documenté, interprétez donc ses champs avec prudence. |
| `EnergyEstimation`, `TaggedEnergy` | Fournisseurs d'énergie et d'utilisation propres à certaines versions. |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Tables de fournisseurs de Windows Server. |
| Nom de table inconnu | Juntei conserve les fournisseurs inconnus sous leur GUID/nom de table et émet chaque colonne du catalogue. |

Comme SRUM est piloté par son catalogue et que Windows ajoute des colonnes aux fournisseurs, il n'existe volontairement pas de liste
finie et fixe de champs propres à chaque fournisseur. Juntei émet **chaque colonne présente dans la base de données avec les
libellés de champs en PascalCase de Juntei**. Les champs suivants constituent le contrat de sortie stable/commun.

## Sortie catégorisée de auto-parse { #categorized-auto-parse-output }

`auto-parse` place chaque table SRUM directement dans sa catégorie d'investigation. Les formats CSV, JSON et JSONL
utilisent le même préfixe de nom de fichier `SRUM-` ; il n'y a pas de répertoire `SRUM/` englobant.

| Catégorie | Fichiers CSV |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`, `SRUM-EnergyEstimation.csv`, `SRUM-TaggedEnergy.csv`, `SRUM-SizeInBytes.csv`, `SRUM-TotalUsed.csv`, `SRUM-IdentityMap.csv`, `SRUM-VfuProvider.csv` et les tables des fournisseurs inconnus |

Seules les tables présentes dans la capture sont exportées. Les compteurs réseau et les enregistrements de notifications sont
placés selon leur usage principal en investigation ; cela ne signifie pas que chaque ligne décrit une connexion
distante. La table de correspondance des identités sert à plusieurs catégories. VFU et les fournisseurs inconnus restent dans
`SystemInformation`, car leur présence seule n'établit pas une exécution.

`SystemInformation/SRUM-logs.txt` contient les diagnostics du parseur et la correspondance entre la sortie d'origine et la sortie
finale. Les deux manifestes répertorient les chemins réels des tables dans `OutputFiles` ; l'`Output` de la tâche SRUM
est le répertoire de l'ordinateur et `Category` identifie l'emplacement partagé de son journal/de ses métadonnées.

Avec plusieurs bases de données, les noms de fichiers reçoivent `-Source1`, `-Source2`, etc., avant l'extension.
Le journal conserve les alias de capture d'origine, et les lignes JSON/JSONL conservent leur provenance `SourceFile`,
`Table`, `Provider`, `Comp` et `Artifact`. Aucune table n'est copiée dans plusieurs
catégories. La chronologie SIEM combinée `-j` inclut chaque enregistrement SRUM conservé une seule fois.

La commande autonome `parse-ese srum` conserve l'emplacement de sortie demandé. Ses fichiers de tables CSV portent également
le préfixe `SRUM-` ; les noms de fichiers de sortie JSON/JSONL explicites restent choisis par l'analyste.

## Champs de sortie communs { #common-output-fields }

| Champ | Interprétation |
|---|---|
| `SourceFile` | Base de données d'entrée ; présente en JSON/JSONL et utile pour plusieurs hôtes. |
| `Table` | Nom/GUID d'origine de la table ESE. À conserver pour comparer des versions ou des fournisseurs inconnus. |
| `Provider` | Nom convivial du fournisseur/de la sortie, ou nom d'origine de la table s'il est inconnu. |
| `ID` | `AutoIncId` de la ligne ; un identifiant de ligne de la base de données, et non un identifiant de processus. |
| `Timestamp` | Moment où SRUM a enregistré/agrégé les valeurs, arrondi à la milliseconde. Généralement pas l'heure exacte de l'événement. |
| `EXE-Info` | Identité de l'application/du service/du package résolue à partir de `AppID`. Pivot principal pour le programme. |
| `EXE-InfoDescription` | Description extraite des noms d'application SRUM composites. Contexte uniquement. |
| `EXE-Timestamp` | Heure intégrée dans une identité d'application composite, le cas échéant ; pas l'heure d'utilisation de la ligne. |
| `AppType` | `NormalApp`, `Service`, `ModernApp` ou `Object` ; détermine la manière d'interpréter `EXE-Info`. |
| `SID` | SID du compte résolu à partir de `UserID`. Pivot solide pour l'identité de l'utilisateur/du service. |
| `User` | Nom de profil convivial enrichi à partir de `SOFTWARE` ; peut être null alors que `SID` reste valide. |
| `UserID`, `AppID` | Références numériques brutes vers `SruDbIdMapTable` ; à conserver pour les noms non résolus ou en double. |
| `IsLt` | `true` pour la table EnergyUsage à long terme, `false` pour celle à court terme. |
| `InterfaceLUID` | LUID brut de l'interface réseau provenant des lignes du fournisseur. |
| `InterfaceType` | Classe d'interface décodée (par exemple `IF_TYPE_IEEE80211`) ; une classe, et non un adaptateur unique. |
| `L2ProfileID` | Référence numérique du profil réseau. |
| `ProfileName` | Nom du réseau Wi-Fi/du profil enrichi à partir de `SOFTWARE` ; potentiellement sensible, et ne prouve pas une connexion en cours. |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | Colonnes FILETIME connues des fournisseurs, décodées en UTC. Leur signification exacte dépend du fournisseur ; il ne s'agit pas d'une heure d'exécution universelle. |
| Colonnes propres aux fournisseurs | Colonnes de fournisseurs exportées telles que `BytesSent`, `BytesReceived` (la colonne de base de données `BytesRecvd`), les compteurs CPU/disque, les durées, les nombres, les indicateurs ou les estimations. Interprétez les unités selon le fournisseur et comparez les lignes au sein d'un même schéma. |

## Champs de diagnostic (`--include-value-state`) { #diagnostic-fields-include-value-state }

| Champ | Interprétation |
|---|---|
| `ValueState` | Correspondance entre chaque champ de la base de données et `parsed`, `null`, `absent`, `unsupported` ou `damaged`. Permet de distinguer l'absence de preuve d'un échec du parseur. |
| `Raw` | Octets d'origine en hexadécimal pour les valeurs qui comportent des octets. À utiliser pour la validation et un décodage ultérieur. |

`SRUM-IdentityMap.csv` (issu de `SruDbIdMapTable`) contient `IdIndex`, `IdType` et `IdBlob` : l'index brut, la catégorie d'identité
et le texte décodé du programme/SID/objet utilisé par les lignes des fournisseurs.

## Stratégie d'investigation { #investigation-strategy }

1. Identifiez le programme avec `EXE-Info`, `AppType` et `AppID` ; ne confondez pas `Id` avec le PID.
2. Regroupez les lignes réseau par programme, utilisateur, profil, interface et intervalle de temps.
3. Comparez les octets envoyés/reçus avec les références habituelles et avec d'autres hôtes.
4. Corrélez les intervalles suspects avec Prefetch, les journaux d'événements, les données DNS/proxy/pare-feu et les accès aux fichiers.
5. Signalez l'état « dirty » de la base de données et l'absence de relecture des journaux de transactions comme une limitation de la collecte.

## Plus de détails { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft : Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
