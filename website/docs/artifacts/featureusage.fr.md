# FeatureUsage, RecentApps et éléments épinglés à la barre des tâches { #featureusage-recentapps-and-taskbar-pins }

## De quoi s'agit-il { #what-they-are }

L'Explorateur conserve, dans le `NTUSER.DAT` de l'utilisateur, plusieurs enregistrements de la manière dont celui-ci utilise les applications :

- **FeatureUsage** (Windows 10 et versions ultérieures) : des compteurs par application, selon le type d'utilisation.
  - `AppSwitched` : nombre de fois où l'utilisateur est passé à la fenêtre de l'application (clic dans la barre des tâches,
    Alt+Tab).
  - `AppLaunch` : nombre de fois où elle a été démarrée depuis la barre des tâches (un élément épinglé).
  - `ShowJumpView` : nombre de fois où sa jump list a été ouverte (clic droit dans la barre des tâches).
  - `AppBadgeUpdated` : nombre de fois où son badge dans la barre des tâches (un compteur de non-lus) a changé.
  - `TrayButtonClicked` : nombre de fois où chaque bouton de la barre des tâches (`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …) a été cliqué.

  Une application est un ID de modèle utilisateur d'application (`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`), un chemin dont la première partie peut être le GUID
  d'un dossier connu (`{1AC14E77-…}\cmd.exe` correspond à `%windir%\system32\cmd.exe`), ou un ID de processus
  (`*PID00001d74`) pour les fenêtres que l'Explorateur n'a pas pu rattacher à une application.
- **RecentApps** (de Windows 10 jusqu'à la version 1709) : les applications qu'un utilisateur a exécutées récemment, avec un nombre
  de lancements et la dernière heure ; sous chacune, les fichiers qu'elle a ouverts (`RecentItems`).
- **Éléments épinglés à la barre des tâches** (`Taskband`) : les programmes épinglés à la barre des tâches, sous forme d'éléments du shell
  (`Favorites`) et sous forme du lien de chaque raccourci épinglé (`FavoritesResolve`). Sous Windows 10 et
  versions ultérieures, chaque élément épinglé indique s'il l'a été par Windows (`SystemPinned`) ou par l'utilisateur (`UserPinned`).
- **ProgramsCache** (`StartPage`, `StartPage2` ; de Windows XP à 7) : les programmes mis en cache du menu Démarrer
  (`ProgramsCache`) et, sous Windows 7, les programmes épinglés au menu Démarrer et à la barre des tâches
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`).

<span class="evidence-strength">Meilleure conclusion :</span> cet utilisateur a utilisé cette application ce
nombre de fois de cette manière (FeatureUsage), l'a exécutée à cette heure (RecentApps) ou l'a épinglée. Les
compteurs n'ont pas d'heure propre ; seules la dernière écriture de la clé et les heures de RecentApps permettent de dater
quoi que ce soit.

## Emplacement { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

Le `KeyCreationTime` de `FeatureUsage` correspond au début du comptage ; la synthèse l'affiche. Collectez
`NTUSER.DAT` avec ses journaux de transactions pour chaque profil, ainsi que les ruches système `SYSTEM` et
`SOFTWARE` pour l'hôte et le SID de l'utilisateur.

## Précautions d'analyse { #analyst-cautions }

- **Des compteurs sans heures.** Un compteur FeatureUsage indique une fréquence, et non un moment. L'heure de dernière écriture de la clé
  du type (`KeyLastWrittenUTC`) correspond à la modification la plus récente de l'un quelconque de ses compteurs.
- **Pas chaque exécution.** `AppSwitched` compte les changements de fenêtre, et non les démarrages ; une application exécutée
  sans fenêtre, ou vers laquelle l'utilisateur n'a jamais basculé, n'est pas comptée. `AppLaunch` ne compte que les
  démarrages depuis la barre des tâches.
- **Applications empaquetées et chemins.** Une application est nommée selon la façon dont Windows a regroupé ses fenêtres : l'ID
  d'une application empaquetée, un ID qu'un programme s'est attribué lui-même (`Chrome`, `VMware.Workstation.vmui`) ou un
  chemin. `AppPath` donne le chemin dans lequel le GUID d'un dossier connu est remplacé par le chemin par défaut
  de ce dossier.
- **Éléments épinglés par défaut.** Les nouveaux profils reçoivent des éléments épinglés par défaut (Explorateur de fichiers, Edge, le Store) ; sous Windows 10
  et versions ultérieures, ils indiquent `SystemPinned`. Un élément `UserPinned` relève du choix de l'utilisateur.
- **Des éléments épinglés dans deux listes.** `Favorites` et `FavoritesResolve` décrivent les mêmes éléments épinglés dans le même
  ordre (`Index`), mais ils sont fournis sous forme d'enregistrements distincts.
- **Les chemins de ProgramsCache** sont relatifs : un dossier (`Programs\Accessories`) est suivi des
  raccourcis qu'il contient (`Notepad.lnk`).
- **Valeurs endommagées.** Un compteur qui n'est pas un `REG_DWORD`, une heure qui n'est pas un `REG_QWORD`, un élément épinglé
  non interprétable ou un lien qui n'en est pas un conserve ses octets (`DataRaw`) et est signalé dans
  `Problems`.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

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
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

La synthèse liste les enregistrements par source, le moment où FeatureUsage a commencé à compter dans chaque ruche, les
applications vers lesquelles l'utilisateur a le plus basculé, les applications et éléments récents (du plus récent au plus ancien) et les éléments épinglés à la barre des tâches.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Hôte, nom du dossier de profil, SID de l'utilisateur issu de `ProfileList` du système, et ruche. |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked` (FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` ou `ProgramsCache`. |
| `App` | Nom d'une valeur FeatureUsage, `AppId` d'une application récente, ou ID de modèle utilisateur d'application d'un élément épinglé. |
| `AppPath` | `App` sous forme de chemin, le GUID d'un dossier connu étant remplacé par son chemin par défaut ; vide pour les ID. |
| `Count` | Un compteur FeatureUsage, ou le `LaunchCount` d'une application récente. |
| `LastAccessedUTC` | `LastAccessedTime` d'une application ou d'un élément récent. |
| `ItemPath` | `Path` d'un élément récent (ou `AppPath` d'une application récente) ; chemin d'élément du shell d'un élément épinglé ou d'un programme mis en cache (`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`) ; chemin d'un lien `FavoritesResolve`. |
| `DisplayName` | `DisplayName` d'un élément récent. |
| `PinType` | `SystemPinned` ou `UserPinned`, pour un élément épinglé `Favorites`. |
| `Index` | Position d'un élément épinglé ou d'un programme mis en cache dans sa liste. |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | Valeur et clé de provenance, et dernière écriture de la clé. |
| `DataRaw` | Octets (hexadécimal) qui ne se décodent pas. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Valeurs de type incorrect, éléments épinglés ou listes non interprétables, liens qui n'en sont pas. |

## Stratégie d'investigation { #investigation-strategy }

1. Triez `AppSwitched` par `Count` pour identifier les applications dans lesquelles un utilisateur a le plus travaillé, et recherchez
   des outils d'administration, d'accès à distance, d'archivage et de transfert, ainsi que des chemins situés dans des emplacements accessibles en écriture par l'utilisateur
   (`\Users\`, `Downloads`, `Temp`).
2. Vérifiez les éléments épinglés : un outil `UserPinned`, ou un élément épinglé dont le raccourci se trouve dans un emplacement inhabituel.
3. Utilisez les heures de RecentApps, avec UserAssist, Prefetch, BAM et les Jump Lists, pour dater l'utilisation.
4. Comparez `KeyCreationTime` avec la création du profil : les compteurs démarrent à ce moment-là.
5. Utilisez `parse-registry read --recover-deleted` pour les compteurs et éléments épinglés supprimés.

## Plus de détails { #further-details }

- [Microsoft : ID de modèle utilisateur d'application](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft : KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb : Programs cache](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
