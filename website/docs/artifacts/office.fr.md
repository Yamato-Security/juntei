# Microsoft Office (fichiers récents, documents approuvés, paramètres des macros) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## Ce qu'Office enregistre { #what-office-records }

Microsoft Office conserve, par utilisateur dans `NTUSER.DAT`, et par version et par application :

- **Fichiers et emplacements récents :** `File MRU`, les documents ouverts par une application, et `Place MRU`,
  les dossiers à partir desquels elle les a ouverts, chaque élément étant accompagné de l'heure de sa dernière ouverture. Microsoft 365
  conserve également une liste par compte connecté (`User MRU\<account>`).
- **Documents approuvés :** lorsqu'un utilisateur clique sur *Activer la modification* ou *Activer le contenu* pour un document,
  Office le mémorise dans `TrustRecords` : le chemin du document, la date à laquelle il a été approuvé, et si
  ses macros ont été activées.
- **Paramètres des macros :** `VBAWarnings` (quelles macros s'exécutent), `BlockContentExecutionFromInternet`
  (si les macros des fichiers provenant d'Internet sont bloquées), `AccessVBOM` (si les programmes peuvent
  modifier les projets VBA) et les **emplacements approuvés**, des dossiers dont les macros des documents s'exécutent sans
  demande de confirmation. La stratégie de groupe peut définir les mêmes paramètres sous `Software\Policies`.

Un document malveillant nécessite généralement que l'utilisateur active ses macros : un enregistrement d'approbation dont les macros
ont été activées relie le document, l'utilisateur et une heure.

<span class="evidence-strength">Meilleure conclusion :</span> l'utilisateur a ouvert ce document dans cette
application (et pour la dernière fois à cette heure) ; l'utilisateur a activé la modification ou les macros de ce document à
cette heure ; Office était configuré pour exécuter les macros de cette façon.

## Emplacement { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

Versions : `16.0` correspond à Office 2016 et versions ultérieures (2019, 2021, 2024, Microsoft 365), `15.0` à Office 2013,
`14.0` à Office 2010, `12.0` à Office 2007.

Un élément ressemble à `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx` : `T`
est l'heure de dernière ouverture (un FILETIME en hexadécimal), le chemin suit `*`. Un enregistrement d'approbation fait 24
octets : le FILETIME auquel le document a été approuvé, puis ses indicateurs dans les 4 derniers octets, `0x7FFFFFFF`
lorsque ses macros ont été activées et `0x00000001` lorsque seule sa modification l'a été.

## Précautions d'analyse { #analyst-cautions }

- **Horodatages.** `LastOpenedUTC` est l'heure propre à chaque élément. `TrustedAtUTC` correspond au moment où l'utilisateur a approuvé
  le document, c'est-à-dire l'instant où il a cliqué sur *Activer la modification* ou *Activer le contenu*, ce qui peut être antérieur
  à l'ouverture la plus récente.
- **Macros activées, et non exécutées.** `MacrosEnabled` indique que l'utilisateur a activé les macros du document ; cela
  ne prouve pas qu'une macro s'est exécutée ni ce qu'elle a fait. Corrélez avec la création de processus, Prefetch, les journaux
  PowerShell et le document lui-même.
- **Chemins tels qu'Office les stocke.** Les enregistrements d'approbation désignent le document par un chemin qui peut utiliser
  `%USERPROFILE%`, des barres obliques, une URL `file:///` et un encodage d'URL (`%20`). Les éléments MRU des
  documents dans le cloud sont des URL.
- **Valeurs par défaut et stratégie effective.** Office crée des emplacements approuvés par défaut (`Templates`,
  `XLSTART`, `STARTUP`, …). La `Date` d'un emplacement est un indice utile, et non une preuve de l'identité de son créateur :
  des administrateurs, des scripts, des stratégies et des paramètres de Registre copiés peuvent également le renseigner. L'absence de
  `VBAWarnings` ne permet pas à elle seule d'établir la stratégie effective des macros. La version d'Office, le marquage des fichiers
  provenant d'Internet et d'autres paramètres de gestion comptent également.
  [Microsoft explique le blocage des macros provenant d'Internet](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked).
- **Documents supprimés.** Les listes survivent aux documents ; un fichier absent peut encore apparaître
  ici. Les listes sont courtes et sont effacées par la commande *Effacer les éléments non épinglés* d'Office et par les outils de nettoyage.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

Le résumé répertorie les documents dont les macros ont été activées, les paramètres qui permettent aux macros de s'exécuter plus
librement (`VBAWarnings` 1, `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1,
`AllowNetworkLocations` 1), les emplacements approuvés ajoutés par les utilisateurs ou définis par une stratégie, ainsi que les documents
ouverts en dernier.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | L'hôte, le propriétaire du profil (d'après le chemin ou les dossiers du shell), le SID provenant du `ProfileList` du système, et la ruche. |
| `OfficeVersion`, `App` | La clé de la version (`16.0`) et l'application (`Word`, `Excel`, …). |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security` ou `Trusted Locations`. |
| `Account` | Le compte Microsoft 365 d'une liste `User MRU`. |
| `Policy` | Si le paramètre ou l'emplacement est défini par une stratégie. |
| `MRU-Position`, `IsMostRecent` | La position d'un élément dans sa liste (`Item 1` vaut 0, le plus récent). |
| `Path` | Le document ou le dossier ; le chemin d'un document approuvé tel que stocké ; le `Path` d'un emplacement approuvé. |
| `LastOpenedUTC` | La dernière ouverture de l'élément. |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | La date d'approbation du document, si ses macros ont été activées, et les indicateurs de l'enregistrement. Des indicateurs inconnus laissent `MacrosEnabled` vide et produisent un diagnostic. |
| `Setting`, `Value` | Un paramètre de macros et sa valeur, avec sa signification (`1 (all macros enabled)`) ; la clé d'un emplacement approuvé (`Location3`) et ses autres valeurs (`AllowSubFolders=1; Date=…; Description=…`). |
| `KeyPath`, `KeyLastWrittenUTC` | La clé et sa dernière écriture. |
| `DataRaw` | Les octets de la valeur (en hexadécimal) lorsqu'ils ne se décodent pas. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Éléments sans `*` ou dont l'heure n'en est pas une, enregistrements d'approbation qui ne font pas 24 octets, emplacements sans `Path`, types incorrects. |

## Stratégie d'investigation { #investigation-strategy }

1. Commencez par les enregistrements dont `MacrosEnabled` vaut true : les documents, quand, et par qui. Retrouvez les fichiers (`$MFT`,
   Journal USN, `Zone.Identifier`), leur provenance (historique du navigateur et de la messagerie) et ce qui s'est exécuté ensuite.
2. Examinez les paramètres des macros et les emplacements approuvés : toutes les macros activées, les macros provenant d'Internet non
   bloquées, l'accès aux projets VBA ou un emplacement ajouté par l'utilisateur (dans `Downloads`, `Temp`, un partage réseau)
   affaiblissent les valeurs par défaut.
3. Placez `LastOpenedUTC` sur la chronologie avec les fichiers LNK, les Jump Lists et RecentDocs pour les
   chemins des documents, y compris les emplacements amovibles et réseau.

## Plus de détails { #further-details }

- [Microsoft : activer ou désactiver les macros dans les fichiers Microsoft 365](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft : documents approuvés](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft : ajouter, supprimer ou modifier un emplacement approuvé](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
