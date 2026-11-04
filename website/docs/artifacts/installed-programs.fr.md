# Programmes installés { #installed-programs }

## Ce que Windows conserve { #what-windows-keeps }

Windows et les programmes d'installation enregistrent les logiciels installés dans plusieurs listes du Registre. Chaque liste comporte une clé
par programme, produit, distribution ou package ; Juntei produit un enregistrement par clé.

| Source | Ruche et clé | Contenu |
|---|---|---|
| `uninstall` | `SOFTWARE` et `NTUSER.DAT` `…\CurrentVersion\Uninstall\*`, et `SOFTWARE\Wow6432Node\…` | Ce que listent les Paramètres et *Programmes et fonctionnalités* : nom, version, éditeur, date d'installation, dossiers, commandes de désinstallation, taille, masquage éventuel |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | Produits Windows Installer : nom et version du produit, nom de fichier du package et dossier à partir duquel il a été installé |
| `app_path` | `SOFTWARE` et `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | Programmes que Windows démarre par leur nom (`Win+R`, `start tool`) : chemin du programme et son dossier |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | Distributions du Sous-système Windows pour Linux : nom, emplacement de ses fichiers ou de son disque, WSL 1 ou 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | Applications empaquetées (Store) : provisionnées pour tous les utilisateurs, propres à Windows, préparées (staged), déprovisionnées, et celles de chaque utilisateur |

<span class="evidence-strength">Meilleure conclusion :</span> ce programme, produit ou package a été
installé (ou inscrit) sur le système, pour tous les utilisateurs ou pour cet utilisateur. La date d'installation est
celle qu'a écrite le programme d'installation ; la dernière écriture de la clé correspond à la dernière modification de l'entrée.

## Emplacement { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

Collectez `SOFTWARE`, chaque `NTUSER.DAT` et leurs journaux de transactions. `SYSTEM` nomme l'hôte, et
`ProfileList` dans `SOFTWARE` fournit le SID de chaque utilisateur.

## Précautions d'analyse { #analyst-cautions }

- **Date d'installation et heure de la clé.** `InstallDate` est une date (sans heure) écrite par le programme d'installation, en
  heure locale ; certains programmes d'installation ne l'écrivent pas, et d'autres utilisent un autre format (conservé tel qu'écrit dans
  `InstallDateRaw`). Une mise à jour peut réécrire l'entrée sans modifier la date, ou modifier la date
  sans que le programme change beaucoup : comparez `InstallDate` avec `KeyLastWrittenUTC`.
- **Logiciels désinstallés.** Un programme supprimé proprement perd ses clés. Utilisez `parse-registry read --recover-deleted`
  pour les clés `Uninstall` supprimées, et Amcache, Prefetch et la `$MFT` pour les traces
  laissées.
- **Inscriptions par utilisateur.** Les entrées de `NTUSER.DAT` identifient les logiciels inscrits pour un profil.
  Cette portée n'établit ni qui a exécuté le programme d'installation, ni si des droits d'administrateur ont été utilisés.
- **Entrées masquées.** `SystemComponent` masque une entrée dans *Programmes et fonctionnalités* ; les entrées
  sans `DisplayName` (mises à jour, composants) ne sont pas affichées non plus. Elles sont toutes signalées.
- **Codes produit.** Une clé `Uninstall` en forme de GUID peut être rapprochée d'une clé
  `msi_product` compactée via `ProductCode`. Le nom en forme de GUID ne prouve pas à lui seul que
  Windows Installer a créé l'entrée.
- **App Paths.** Il s'agit d'inscriptions servant à localiser des programmes, et non d'enregistrements
  d'exécution. La valeur `Path` peut contenir plusieurs répertoires de recherche et n'est pas nécessairement le
  répertoire d'installation.
- **Éditeur.** Il s'agit de métadonnées du Registre fournies par le programme d'installation, et non d'informations
  de signature vérifiées.
- **Architecture.** `Wow6432Node` contient les programmes 32 bits sous Windows 64 bits (`Architecture` `x86`,
  `Wow64` `True`) ; les autres entrées relèvent de l'architecture propre du système (vide). L'architecture d'un package
  est la sienne (`x64`, `neutral`, …).
- **WSL.** `Version` est la version de WSL déduite du `Flags` de la distribution (`0x8` : elle s'exécute dans la
  machine virtuelle de WSL 2). La valeur `Version` propre à la distribution (dans `Details`) est la version de son système de fichiers,
  qui vaut 2 également sur les distributions WSL 1. `BasePath` contient son `ext4.vhdx` (WSL 2) ou `rootfs`
  (WSL 1) : collectez-le pour en examiner le contenu.
- **Packages.** Les listes `<SID>` sont les packages inscrits pour chaque utilisateur ; la plupart sont propres à
  Windows. Les packages `Deprovisioned` ont été supprimés pour les nouveaux utilisateurs, ceux de `EndOfLife\<SID>` pour cet utilisateur.
- **Valeurs endommagées.** Les nombres malformés, les types de valeur incorrects et l'UTF-16 non valide sont signalés dans
  `Problems` et conservés dans `DataRaw`, y compris une chaîne hexadécimale vide pour une valeur présente de
  zéro octet. Une chaîne vide propre dans un champ numérique facultatif est traitée comme absente. Une version
  de package doit comporter quatre composants décimaux, chacun dans la plage des entiers non signés sur 16 bits.
  Les avertissements au niveau de la ruche restent sur stderr lorsque `-t` diffuse les enregistrements sur stdout.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

La synthèse liste les derniers programmes installés (par `InstallDate`), ce que les utilisateurs ont installé pour
eux-mêmes, et les distributions WSL.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Hôte, fichier de ruche, et `SOFTWARE` ou `NTUSER.DAT`. |
| `User`, `UserSID` | Utilisateur d'une ruche utilisateur (son dossier de profil et son SID `ProfileList`), ou utilisateur pour lequel un package est listé. |
| `Source`, `Scope` | La liste (`uninstall`, `msi_product`, `app_path`, `wsl`, `package`), et `machine` ou `user`. |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher` ; `ProductName` et `Version` d'un produit ; nom de fichier du programme d'une App Path ; nom et version WSL d'une distribution ; nom et version d'un package. |
| `InstallDate`, `InstallDateRaw` | `InstallDate` sous forme de date lorsqu'elle est écrite au format `YYYYMMDD` ; sinon telle qu'écrite. |
| `KeyLastWrittenUTC` | Dernière modification de la clé de l'entrée. |
| `InstallLocation`, `InstallSource` | `InstallLocation` (`Path` d'une App Path, `BasePath` d'une distribution, `Path` d'un package) ; `InstallSource` (dernier dossier source utilisé pour un produit). |
| `Program` | Programme d'une App Path. |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`. |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | `EstimatedSize` en Kio ; masquage éventuel ; installation ou non par Windows Installer. |
| `ProductCode` | Code produit Windows Installer (`{GUID}`). |
| `Architecture`, `WOW64` | `x86` sous `Wow6432Node`, ou architecture d'un package ; indication de la présence de la clé sous `Wow6432Node`. |
| `Details` | Informations complémentaires : `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User` (le compte ayant exécuté un programme d'installation Inno Setup) ; `PackageCode`, `PackageName` d'un produit ; `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName`, `Default` d'une distribution ; `List`, `ResourceId`, `PublisherId` d'un package. |
| `KeyPath` | Clé de l'entrée. |
| `DataRaw` | Octets (hexadécimal) des valeurs qui ne correspondent pas à ce qu'indique leur nom. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Valeurs illisibles ou ne correspondant pas à ce qu'indique leur nom. |

## Stratégie d'investigation { #investigation-strategy }

1. Triez par `InstallDate` et `KeyLastWrittenUTC` autour de l'incident ; examinez les outils d'accès à distance,
   les clients de tunnellisation et de transfert de fichiers, les archiveurs, les environnements d'exécution de scripts et les outils de sécurité.
2. Examinez les enregistrements dont `Scope` vaut `user` : logiciels installés sans droits d'administrateur.
3. Vérifiez si `InstallLocation`, `InstallSource` et `UninstallCommand` désignent des dossiers accessibles en écriture par l'utilisateur
   (`Downloads`, `AppData`, `Temp`, `ProgramData`).
4. Recherchez des distributions WSL et collectez leur `BasePath`.
5. Corrélez avec Amcache et Prefetch (le programme d'installation et l'exécution du programme), la `$MFT` (la création du
   dossier d'installation) et le journal d'événements Application (événements MsiInstaller).

## Plus de détails { #further-details }

- [Microsoft : clé de Registre Uninstall](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft : inscription des applications (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft : vue d'ensemble de l'identité de package](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
