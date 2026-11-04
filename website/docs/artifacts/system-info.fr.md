# Informations système { #system-information }

## De quoi s'agit-il { #what-it-is }

Les quatre ruches machine d'un système indiquent ce dont un analyste a besoin avant de lire sa chronologie :

- ses noms (NetBIOS, nom d'hôte DNS, domaine DNS) et le domaine ou groupe de travail auquel il appartient ;
- sa version de Windows, sa build et sa révision de mise à jour, ainsi que la date d'installation ou de dernière mise à niveau de Windows ;
- son fuseau horaire, qui vous indique les heures locales que le système a conservées ;
- la date de son dernier arrêt propre ;
- l'architecture de son processeur et ses pages de codes ;
- le dernier utilisateur qui s'est connecté, et les profils de chaque utilisateur qui s'est connecté ;
- ses SID : celui de l'ordinateur (les comptes locaux correspondent à ce SID suivi d'un RID) et celui de son domaine ;
- les versions de Windows à partir desquelles il a été mis à niveau.

`parse-registry system-info` écrit un enregistrement par système contenant toutes ces informations.

<span class="evidence-strength">Meilleure conclusion :</span> la configuration telle qu'elle était lorsque les ruches
ont été collectées (ou écrites pour la dernière fois). Les horodatages des clés indiquent quand certaines de ses parties ont changé pour la dernière fois.

## Emplacement { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

Collectez les quatre ruches avec leurs journaux de transactions. Chacune d'elles fournit à elle seule les informations qu'elle contient.

## Précautions d'analyse { #analyst-cautions }

- **Les horodatages du Registre sont en UTC ; le fuseau horaire sert pour tout le reste.** Les horodatages des clés, les FILETIME et la plupart
  des horodatages d'événements sont en UTC. Les journaux texte, certains journaux d'applications et les dates écrites dans des noms ou des chaînes
  (comme `Source OS (Updated on …)`) peuvent être en heure locale. `Bias` est le nombre de minutes entre l'heure standard locale
  et l'UTC (`UTC = local + Bias`). `UtcOffset` et `DaylightUtcOffset` donnent les deux décalages,
  et `TimeZoneDescription` indique quand l'heure d'été commence et se termine.
- **Le fuseau horaire peut changer.** `TimeZoneLastWrittenUTC` est la dernière écriture de la clé. Windows
  réécrit également la clé de lui-même lorsqu'il met à jour `ActiveTimeBias` (le décalage alors en vigueur) ; un horodatage
  récent ne prouve donc pas que quelqu'un a modifié le fuseau. Comparez avec les journaux d'événements : Security 4616
  (l'heure système a été modifiée) et les événements `Microsoft-Windows-Kernel-General` du journal System.
- **`StandardStart` et `DaylightStart` sont des `TIME_FIELDS` du noyau, et non des `SYSTEMTIME`.** Le
  jour de la semaine est leur dernier champ, et non leur troisième (comme dans la valeur `TZI` du fuseau dans `SOFTWARE`). Dans
  les fuseaux américains, les deux dispositions se lisent de la même façon, car le jour de la semaine est le dimanche (0). Juntei les décode
  en tant que `TIME_FIELDS` : un système public de l'île de Pâques (premier samedi à 22:00) montre la
  différence. Un mois égal à 0 signifie qu'il n'y a pas d'heure d'été.
- **Windows 11 indique « Windows 10 ».** `ProductName` conserve `Windows 10 …` sous Windows 11. Les builds 22000
  et ultérieures correspondent à Windows 11, et le résumé l'indique. `CurrentVersion` vaut `6.3` sous Windows 8.1, 10
  et 11 ; `Version` utilise `CurrentMajorVersionNumber` (10) lorsqu'il est présent.
- **`InstallDate` évolue avec les mises à niveau de fonctionnalités.** Une mise à niveau sur place vers une nouvelle version (de 1809 à
  21H2, par exemple) la réinitialise. La version remplacée par chaque mise à niveau, avec sa propre `InstallDate`, figure
  dans `PreviousVersions` (`Setup\Source OS`). La plus ancienne de ces `InstallDate` est la plus proche de
  l'installation d'origine. La date figurant dans le nom d'une clé `Source OS` n'indique pas son fuseau horaire.
- **`ShutdownTime` correspond au dernier arrêt propre.** Un plantage ou une coupure de courant ne la met pas à jour.
  Comparez-la avec les journaux d'événements (6006, 6008, 1074) et avec les heures des dernières écritures dans les ruches.
- **`LastLoggedOnUser` est le dernier utilisateur de l'écran de connexion,** et non chaque ouverture de session. Le Bureau à distance,
  `runas`, les services et les tâches planifiées ne la définissent pas tous. `LogonUiLastWrittenUTC` correspond approximativement au moment où
  elle a été définie pour la dernière fois.
- **Profils :** `ProfileList` répertorie chaque utilisateur (et service) ayant eu un profil sur le
  système, y compris les comptes supprimés tant que leur profil n'a pas été retiré. Windows 10 et les versions ultérieures écrivent
  `LocalProfileLoadTime` et `LocalProfileUnloadTime` : approximativement la dernière connexion et la dernière déconnexion.
- **SID :** les comptes locaux sont de la forme `MachineSid-<RID>`, les comptes de domaine `DomainSid-<RID>`.
  `MachineSid` provient de la `SAM` (sinon de `PolAcDmS` dans `SECURITY`). Les deux devraient correspondre ;
  lorsque ce n'est pas le cas, `Problems` le signale, comme ce serait le cas pour des ruches provenant de systèmes différents. `DomainSid` est vide
  dans un groupe de travail.
- **Un répertoire, un système.** Avec `-d`, les ruches d'un même répertoire appartiennent à un même système.
  Les copies `RegBack` et une seconde ruche du même type (`SYSTEM.old`) constituent des enregistrements distincts.
  Avec `-f`, les ruches fournies appartiennent à un même système, et les autres sont recherchées à côté d'elles.
- **Jeux de contrôle :** seul le jeu de contrôle utilisé en dernier par Windows est lu. `parse-registry read` affiche
  les autres.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

Le résumé indique, pour chaque système : sa version de Windows, sa date d'installation, son fuseau horaire en
toutes lettres, le décalage en vigueur lors de la dernière écriture de la clé, son dernier arrêt, son domaine ou
groupe de travail, le dernier utilisateur qui s'est connecté, son SID, le nombre de ses profils et les versions à partir desquelles il
a été mis à niveau. `auto-parse` affiche également le fuseau horaire de chaque système lorsqu'il répertorie les systèmes qu'il a
trouvés.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | L'hôte et les ruches lues (`SourcePath` est la ruche `SYSTEM`, sinon la première lue). |
| `CompName`, `Hostname`, `Domain` | Le nom NetBIOS (`SYSTEM`, sinon `PolAcDmN` de `SECURITY`), le nom d'hôte DNS et le domaine DNS (`Tcpip\Parameters`). |
| `PrimaryDomain`, `DNS-Domain` | Le domaine auquel l'ordinateur est joint, ou son groupe de travail (`PolPrDmN`), et le nom DNS du domaine (`PolDnDDN`). |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion` : le nom du produit tel qu'écrit, `EditionID`, `DisplayVersion` (sinon `ReleaseId`), la version majeure et mineure (sinon `CurrentVersion`), la build, la révision de build de mise à jour, `CSDVersion`, et `Client`, `Server` ou `Server Core`. |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (à la seconde) et `InstallTime` (à 100 ns près, Windows 10 et versions ultérieures) : l'installation ou la dernière mise à niveau de fonctionnalités. |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | Tels qu'écrits. |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (sinon `StandardName`), et le fuseau en toutes lettres avec ses décalages et ses dates d'heure d'été. |
| `UTC-Offset`, `DaylightUTC-Offset` | Le décalage de l'heure standard locale par rapport à l'UTC, et celui de l'heure d'été (vide en l'absence d'heure d'été). |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`, `ActiveTimeBias` (en vigueur lors de l'écriture de la clé), `StandardBias`, `DaylightBias`. |
| `StandardName`, `DaylightName` | Tels qu'écrits, souvent une référence de ressource (`@tzres.dll,-112`). |
| `StandardStart`, `DaylightStart` | Quand commencent l'heure standard et l'heure d'été, en toutes lettres (`first Sunday of November, 02:00`). |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | L'heure d'été dynamique est désactivée ; l'horloge matérielle est maintenue en UTC. |
| `TimeZoneLastWrittenUTC` | La dernière écriture de la clé du fuseau horaire. |
| `LastShutdownUTC` | `ShutdownTime` : le dernier arrêt propre. |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | Le jeu de contrôle lu, `PROCESSOR_ARCHITECTURE`, `ACP` et `OEMCP`. |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | Le dernier utilisateur de l'écran de connexion, et la dernière écriture de sa clé. |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | Le SID de l'ordinateur, le SID de son domaine (`PolPrDmS`) et le SID du compte d'ordinateur dans le domaine (`PolMachineAccountS`). |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | Le moment approximatif où l'ordinateur a été nommé pour la dernière fois, a rejoint un domaine ou un groupe de travail pour la dernière fois, et a été mis à jour pour la dernière fois (clé de `CurrentVersion`). |
| `PreviousVersions` | Chaque clé `Source OS` : la version, sa date d'installation et la date figurant dans le nom de la clé (JSON : un objet chacune). |
| `Profiles` | Chaque profil : SID, dossier, horodatage de la clé, dernier chargement et dernier déchargement (JSON : un objet chacun). |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre, sur l'ensemble des quatre ruches. |
| `Problems` | Valeurs de type ou de taille incorrects (leurs octets sont conservés), et une `SAM` et une `SECURITY` dont les SID d'ordinateur diffèrent. |

## Stratégie d'investigation { #investigation-strategy }

1. Lisez d'abord le fuseau horaire : notez `UtcOffset`, les dates d'heure d'été et si le fuseau a été
   modifié récemment. Convertissez avec lui les journaux en heure locale.
2. Notez la version, la build et la date d'installation. Consultez `PreviousVersions` pour les mises à niveau, et comparez
   la date d'installation avec les horodatages les plus anciens du système de fichiers et des journaux d'événements.
3. Notez `LastShutdownUTC` : une activité postérieure sans nouvel horodatage d'arrêt suggère un plantage, une coupure
   de courant ou une collecte à chaud.
4. Utilisez `MachineSid` et `DomainSid` pour distinguer les comptes locaux des comptes de domaine dans les SID présents ailleurs (BAM,
   `ProfileList`, journaux d'événements), et les heures de chargement et de déchargement des profils pour les dernières connexions.
5. Corrélez les noms d'ordinateur et de domaine avec les journaux d'événements, ainsi qu'avec `CompNameLastWrittenUTC`
   et `PrimaryDomainLastWrittenUTC` pour les changements de nom et les jonctions à un domaine.

## Plus de détails { #further-details }

- [Microsoft : TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft : DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft : TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft : identificateurs de sécurité](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb : fuseaux horaires](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb : informations sur le produit Windows](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
