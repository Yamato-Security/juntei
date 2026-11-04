# Historique des applications { #application-history }

## Ce que laissent les outils { #what-tools-leave-behind }

De nombreux outils tiers conservent leurs paramètres et leurs historiques dans le `NTUSER.DAT` de l'utilisateur. Plusieurs d'entre eux
figurent parmi les outils qu'utilisent les attaquants, ou qu'ils utilisent pour préparer des données avant de les emporter : outils Sysinternals, archiveurs,
clients de synchronisation cloud et outils d'accès à distance. Juntei lit les suivants :

| Application | Clé sous `HKCU\Software` | Ce qu'elle montre |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | Chaque outil dont l'utilisateur a accepté la licence (`EulaAccepted`) : PsExec, ProcDump, Sysmon, … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | Archives ouvertes ; noms d'archives et dossiers d'extraction saisis dans ses boîtes de dialogue |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | Archives créées ; dossiers d'extraction ; dossiers parcourus et dossiers de destination de copie dans son gestionnaire de fichiers |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | Archives de son menu Fichier, dossiers d'extraction, archives récentes |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | Adresse e-mail, dossier local et noms de chaque compte ; bibliothèques synchronisées (`ScopeIdToMountPointPathCache`) et dossiers (`Tenants\<tenant>`) |
| Adobe Acrobat et Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | Derniers fichiers PDF et dossiers ouverts, avec noms de fichiers, tailles et dates |
| TeamViewer | `TeamViewer` (et `Version<N>` pour une version plus ancienne) | Toutes les valeurs : noms de compte et d'utilisateur, dossiers de départ des transferts de fichiers (`<partner ID>?<local>|<remote>`), … |
| CCleaner | `Piriform\CCleaner` | Toutes les valeurs : ce qu'il est configuré pour nettoyer, `UpdateKey`, … |

<span class="evidence-strength">Meilleure conclusion :</span> cet utilisateur a exécuté l'outil (ou accepté sa
licence), ou l'outil a ouvert, créé ou extrait ces archives et dossiers, ou ce compte cloud
a été configuré pour l'utilisateur. La plupart des historiques n'ont pas d'heure par entrée : seule l'entrée la plus récente reçoit
l'heure de dernière écriture de la clé de sa liste.

## Emplacement { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

Collectez le `NTUSER.DAT` de chaque utilisateur avec ses journaux de transactions, ainsi que la ruche `SOFTWARE` du système pour
les SID des utilisateurs.

## Précautions d'analyse { #analyst-cautions }

- **Acceptation de l'EULA Sysinternals.** `FirstUsedUTC` est une estimation de la première utilisation, tirée de l'heure de dernière écriture
  de la clé lorsque sa seule valeur est un `EulaAccepted` lisible défini à `1`. Une EULA refusée ou
  endommagée ne reçoit pas cette estimation. Les paramètres du Registre peuvent aussi être préconfigurés,
  copiés ou modifiés : corroborez l'exécution et son heure avec d'autres artefacts.
  Les outils qui y conservent leurs paramètres (Process Explorer, Autoruns, …) réécrivent la clé ultérieurement :
  `FirstUsedUTC` est alors vide et `KeyLastWrittenUTC` correspond seulement à la dernière modification des paramètres. Un outil
  exécuté avec `-accepteula`, ou sous un autre utilisateur (le côté distant de PsExec s'exécute en tant que SYSTEM), écrit la
  clé de cet utilisateur.
- **Des positions, pas des heures.** Les entrées des listes sont ordonnées (`MruPosition` 0 est la plus récente), mais
  seule la dernière écriture de la clé de liste date l'entrée la plus récente. Les entrées plus anciennes ont été utilisées auparavant, à une
  heure inconnue.
- **Noms saisis.** `ArcName` de WinRAR est le nom d'archive saisi lors de la création d'une archive : souvent un nom de
  fichier sans dossier.
- **Les chemins Adobe** sont conservés tels qu'Adobe les écrit (`/C/Users/alice/a.pdf` correspond à
  `C:\Users\alice\a.pdf`). Son `sDate` (dans `Details`) est une chaîne de date PDF, conservée telle qu'écrite.
- **`UpdateKey` de CCleaner** est une heure locale sans fuseau ; elle est conservée telle qu'écrite.
- **TeamViewer.** Les journaux de connexion sont des fichiers (`Connections_incoming.txt`, `TeamViewer*_Logfile.log`),
  et non des valeurs du Registre : collectez-les également.
- **Absence.** Les utilisateurs et les outils peuvent effacer ces historiques (CCleaner lui-même en est capable).

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

Le résumé liste les estimations de première utilisation Sysinternals les plus récentes, les archives et dossiers utilisés en dernier
dans chaque liste, ainsi que les comptes OneDrive.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath` | Hôte et `NTUSER.DAT`. |
| `User`, `UserSID` | Le nom du dossier de profil, et son SID provenant du `ProfileList` du système. |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`. |
| `Item` | L'outil Sysinternals, la liste (`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …), le compte OneDrive ou la clé de ses dossiers synchronisés, ou le nom de la valeur. |
| `MRU-Position` | La position de l'entrée dans sa liste, 0 étant la plus récente. |
| `Path` | L'archive, le dossier ou le document. |
| `Value` | `EulaAccepted`, l'adresse e-mail d'un compte OneDrive ou l'identifiant d'une bibliothèque synchronisée, une valeur TeamViewer ou CCleaner. |
| `FirstUsedUTC` | Estimation de la première utilisation tirée de l'acceptation de l'EULA (voir les précautions ci-dessus). |
| `Details` | `DisplayName`, `UserCID`, `ConfiguredTenantId`, … d'un compte OneDrive ; `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate` d'un document Adobe. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | La provenance de l'enregistrement, et la date de dernière modification de cette clé. |
| `DataRaw` | Les octets de la valeur (hexadécimal), lorsqu'ils ne se décodent pas. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Données qui n'ont pas pu être décodées ou lues. |

## Stratégie d'investigation { #investigation-strategy }

1. Recherchez les outils Sysinternals qu'un utilisateur ordinaire n'exécute pas (PsExec, ProcDump, PsLoggedOn,
   SDelete) et les horodatages de leur EULA.
2. Examinez les noms d'archives et les dossiers d'extraction autour de l'incident : archives créées dans
   `ProgramData`, `Temp` ou le dossier d'un utilisateur avant que des données ne quittent le système.
3. Vérifiez les comptes OneDrive qui n'appartiennent pas à l'organisation, ainsi que les noms de compte TeamViewer et les dossiers de
   transfert de fichiers.
4. Corrélez avec Prefetch, Amcache, BAM et UserAssist (l'exécution de l'outil), le `$MFT` et le journal
   USN (le fichier de l'archive), les fichiers LNK et les Jump Lists (les documents ouverts), ainsi que les journaux d'événements.

## Plus de détails { #further-details }

- [Microsoft : Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
