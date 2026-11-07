# Journalisation des accès utilisateur (UAL) { #user-access-logging-ual }

## Qu'est-ce que l'UAL ? { #what-ual-is }

La journalisation des accès utilisateur (User Access Logging, UAL) est une fonctionnalité de Windows Server, installée et
activée par défaut depuis Windows Server 2012, qui agrège dans des bases de données ESE les requêtes que les clients adressent
aux rôles du serveur. Pour chaque rôle, adresse de client et compte, l'UAL conserve le moment où le client a été vu pour la
première et pour la dernière fois dans l'année, le nombre total de requêtes et un décompte pour chaque jour de l'année. Les rôles
comprennent File Server (SMB), Active Directory Domain Services, Active Directory Certificate Services, DHCP
Server, Print and Document Services, Remote Access, Web Server (IIS) et Windows Server Update
Services.

L'UAL est précieux pour les investigations sur un déplacement latéral : ses lignes de clients montrent quelles adresses et quels
comptes ont utilisé un serveur, et quels jours, souvent bien après que les journaux d'événements ont été écrasés.

<span class="evidence-strength">Meilleure conclusion :</span> un client situé à l'adresse enregistrée, authentifié avec le
compte enregistré, a utilisé le rôle chaque jour pour lequel un décompte existe.

## Emplacement et collecte { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` contient l'activité la plus récente. `SystemIdentity.mdb` décrit le serveur, nomme les
rôles et indique le fichier `{GUID}.mdb` qui contient chaque année (`CHAINED_DATABASES`). Le service UAL
garde les bases de données ouvertes : collectez l'intégralité du répertoire `Sum` avec un collecteur forensique ou à partir d'un
instantané de volume.

!!! warning "Bases de données « dirty »"
    `Current.mdb` est souvent dans un état « dirty » lorsqu'il est collecté sur un serveur en cours d'exécution. Juntei le lit
    tel qu'il a été collecté et le signale, mais ne rejoue pas `Svc*.log` : l'activité présente uniquement dans les journaux de
    transactions est absente.

## Précautions d'analyse { #analyst-cautions }

- L'UAL compte les requêtes par jour, sans journaliser chaque requête. Les seules heures d'une ligne de client sont
  `InsertDate` et `LastAccess`.
- Les jours de `DailyAccesses` sont des jours UTC.
- `Address` est l'adresse vue par le serveur : une passerelle NAT, un proxy ou un VPN peut s'interposer entre celui-ci et le
  client réel. `::1` et `127.0.0.1` désignent le serveur lui-même.
- Analysez toutes les bases de données du répertoire `Sum` : un même client peut avoir des lignes dans `Current.mdb` et dans
  le `{GUID}.mdb` de son année.
- L'absence de lignes ne prouve pas que rien ne s'est connecté : l'UAL peut être désactivé (`Disable-Ual`, ou en
  arrêtant le service User Access Logging), et une base de données « dirty » ne contient pas ses modifications les plus récentes.
- Sans `SystemIdentity.mdb`, `RoleName` provient des noms que Windows Server donne à ses rôles.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Current.mdb, {GUID}.mdb and SystemIdentity.mdb
  -f, --file <FILE>      Parse one UAL database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl \
  | jq 'select(.Table == "CLIENTS") | {Address, AuthenticatedUserName, RoleName, LastAccess}'
```

## Tables de sortie { #output-tables }

Le format CSV crée un fichier par table, qui ne contient que l'en-tête lorsque la table est vide. Les lignes JSON et JSONL
comportent `SourceFile` (la base de données) et `Table` (le nom de la table dans la base de données).

| Fichier CSV | Table | Contenu |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | Les clients de chaque rôle : adresse, compte, premier et dernier accès de l'année, nombre total de requêtes et décomptes quotidiens. |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | La première et la dernière utilisation de chaque rôle. |
| `UAL-DNS.csv` | `DNS` | Des paires adresse/nom d'hôte, avec le moment où chacune a été vue pour la dernière fois. |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Les machines virtuelles Hyper-V : identifiants, numéro de série, heures de création et de dernière activité. |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | Le serveur : nom, domaine, version de Windows, type de produit, matériel, fuseau horaire et dernier démarrage ; une nouvelle ligne peut révéler un changement de nom, une jonction à un domaine ou une promotion en contrôleur de domaine. |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | Le fichier `{GUID}.mdb` de chaque année. |
| `UAL-Roles.csv` | `ROLE_IDS` | Les GUID des rôles avec leurs noms. |
| `UAL-<table>.csv` | Une autre table | Toutes les colonnes d'une table que Juntei ne connaît pas. |

## Champs { #fields }

| Champ | Interprétation |
|---|---|
| `SourceFile` | La base de données dans laquelle la ligne a été lue. |
| `RoleGUID`, `RoleName` | Le rôle, et son nom tiré du `SystemIdentity.mdb` situé à côté de la base de données (noms intégrés en son absence). |
| `TenantID` | Le GUID du locataire (tenant) du client, lorsque le rôle en indique un. |
| `TotalAccesses` | Les requêtes du client pendant l'année de la base de données ; la somme de `DailyAccesses`. |
| `InsertDate` | La création de la ligne : la première requête du client dans l'année, dans cette base de données (UTC). |
| `LastAccess` | La requête la plus récente du client (UTC). |
| `Address` | L'adresse IPv4 ou IPv6 du client. |
| `AuthenticatedUserName` | Le compte sous lequel le client s'est authentifié (`DOMAIN\user` ; les comptes d'ordinateur se terminent par `$`). |
| `ClientName` | Le nom du client, lorsque le rôle en indique un. |
| `DailyAccesses` | Les jours comportant des requêtes et leurs décomptes, tirés des colonnes `Day1` … `Day366` et datés dans l'année de `LastAccess` : `2023-01-12: 5, 2023-01-13: 2` en CSV, des objets avec `Date` et `Accesses` en JSON. Un jour dont l'année est inconnue comporte `DayOfYear` au lieu de `Date`. |
| `FirstSeen`, `LastSeen` | La première et la dernière utilisation d'un rôle (`ROLE_ACCESS`), la dernière observation d'une paire DNS (`DNS`), en UTC. |
| `CreationTime`, `LastSeenActive` | La création d'une machine virtuelle ou d'une ligne `SYSTEM_IDENTITY`, et la dernière activité d'une machine virtuelle, en UTC. |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server`. |
| `OS-CurrentTimeZone` | Le décalage du serveur par rapport à UTC, en minutes (`-420` correspond à UTC-07:00). |
| `OS-LastBootUpTime` | Le dernier démarrage du serveur, en UTC (le texte WMI tel qu'il est stocké lorsqu'il ne peut pas être lu). |
| Autres colonnes | Toutes les autres colonnes de la table, sous les libellés de champs en PascalCase de Juntei, par exemple `SystemDNS-HostName`, `SystemDomainName`, `OS-BuildNumber` et `SystemSerialNumber`. |

## Analyse automatique { #automatic-parsing }

`auto-parse` écrit les tables dans `LateralMovement` sous les noms `UAL-Clients.csv`, `UAL-RoleAccess.csv`, etc. (un
seul fichier `UAL.json` ou `UAL.jsonl` avec `-t json` ou `-t jsonl`), accompagnées de
`LateralMovement/UAL-logs.txt`. La chronologie combinée `-j` inclut chaque ligne, avec `Comp` et l'identifiant
d'`Artifact` `ual`. `-Z` convertit `InsertDate`, `LastAccess`, `FirstSeen`, `LastSeen`,
`CreationTime`, `LastSeenActive` et `OS-LastBootUpTime`.
