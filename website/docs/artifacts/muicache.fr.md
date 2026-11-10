# MuiCache { #muicache }

## Qu'est-ce que MuiCache { #what-muicache-is }

Lorsqu'un utilisateur démarre un programme, Windows lit les ressources de version du programme pour obtenir le nom que le
shell doit afficher (le `FileDescription`, par exemple « Windows Command Processor » pour
`cmd.exe`) et le met en cache, par utilisateur, dans `MuiCache`. Windows 8 et les versions ultérieures mettent également en cache la société
(`CompanyName`). Le cache est indexé par le chemin complet du programme :

- **Windows 8 et versions ultérieures :** deux valeurs par programme, `<path>.FriendlyAppName` et
  `<path>.AppCompany`.
- **Windows Vista et 7 :** une valeur par programme, `<path>`, contenant le nom convivial.
- **Windows XP :** la même forme, dans `NTUSER.DAT`.

MuiCache peut fournir des pistes d'exécution de programmes pour la session shell d'un utilisateur, y compris pour des programmes
situés sur des lecteurs amovibles, des partages réseau et des dossiers qui n'existent plus, avec le nom et la société
indiqués par le fichier lui-même.

<span class="evidence-strength">Meilleure conclusion :</span> ce profil shell a mis en cache le chemin, la
description et la société d'un fichier. Corrélez cela avec d'autres artefacts d'exécution. Le profil n'identifie pas
le compte sous lequel le processus s'est exécuté. MuiCache ne comporte ni heure par programme ni nombre d'exécutions.

## Emplacement { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (la langue des chaînes mises en cache) n'est pas un programme et est ignoré. Les valeurs nommées
`@<file>,-<number>` (`@shell32.dll,-22019`) sont des chaînes de ressources que le shell a mises en cache pour ses propres
menus et types de fichiers, et non des programmes ; elles sont ignorées et comptées dans la synthèse. Sous Windows Vista
et versions ultérieures, `NTUSER.DAT` ne contient pas de MuiCache.

Collectez `UsrClass.dat` (et `NTUSER.DAT`) avec leurs journaux de transactions pour chaque profil, ainsi que les
ruches système `SYSTEM` et `SOFTWARE` pour l'hôte et les noms d'utilisateurs.

## Précautions d'analyse { #analyst-cautions }

- **Le propriétaire du profil n'est pas nécessairement le compte du processus.** Un programme démarré avec un autre
  compte peut afficher une fenêtre dans la session de cet utilisateur et laisser des métadonnées dans le cache de cet utilisateur.
  `User` et `UserSid` identifient le propriétaire de la ruche, et non le contexte de sécurité du processus.
- **Pas d'heure par programme.** `KeyLastWrittenUTC` est la dernière écriture de la clé : la modification la plus récente de
  l'un quelconque de ses programmes, et non le moment où un programme particulier s'est exécuté. Prefetch, BAM, UserAssist et les journaux d'événements fournissent
  des heures.
- **Pas uniquement les programmes que l'utilisateur a démarrés manuellement.** Une DLL ou un programme chargé par le shell lui-même (pour
  `Open with`, une extension du shell, un processus enfant d'un programme d'installation) peut apparaître, par exemple `explorerframe.dll` ou
  `shell32.dll`.
- **Le nom provient du fichier.** `FriendlyAppName` et `AppCompany` correspondent à ce qu'indiquaient les ressources
  de version du fichier lors de son exécution. Un logiciel malveillant peut revendiquer n'importe quelle description et société ; un outil renommé
  conserve sa description d'origine, ce qui peut le trahir (un fichier nommé `svchost.exe` dont le
  nom convivial est celui d'un outil d'accès à distance).
- **Une entrée par chemin.** Exécuter à nouveau le même programme n'ajoute pas d'entrée ; une copie située à un autre
  chemin, si. L'entrée d'un programme peut subsister après la suppression du fichier.
- **Les deux formes pour un même chemin.** Lorsqu'un chemin possède un nom convivial sous les deux formes (profil
  mis à niveau), chacune donne lieu à son propre enregistrement, de sorte qu'aucune valeur n'est perdue.
- **Valeurs endommagées.** Une valeur qui n'est pas une chaîne, n'est pas de l'UTF-16 valide ou ne peut pas être lue intégralement conserve
  ses octets (`FriendlyAppNameRaw`, `AppCompanyRaw`) et est signalée dans `Problems`.
  De l'UTF-16 non valide peut aussi produire des caractères de remplacement dans le texte affiché ; utilisez la colonne
  brute pour les octets d'origine.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

La synthèse liste les programmes situés en dehors des dossiers Windows et Program Files (dossiers utilisateur,
autres lecteurs, partages réseau) avec leurs utilisateurs, ainsi que le nombre de chaînes de ressources ignorées.
Les dossiers Program Files portant d'autres noms (sur des versions de Windows dans d'autres langues) sont également listés.
Les chemins relatifs et les chemins qui sortent d'un dossier défini par une variable d'environnement restent listés. Les chemins de lecteur
ordinaires sont vérifiés après résolution des composants `.` et `..`, sans accéder au système de fichiers
de l'examinateur.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Hôte, nom du dossier de profil, SID de l'utilisateur (indiqué par la clé racine d'un `UsrClass.dat` ; à défaut, par `ProfileList` du système), et ruche. |
| `HiveType` | `UsrClass.dat` ou `NTUSER.DAT`. |
| `Path` | Chemin du programme : le nom de la valeur sans `.FriendlyAppName` ni `.AppCompany`. |
| `FriendlyAppName` | Description du programme, issue de `<path>.FriendlyAppName` ou de `<path>`. |
| `AppCompany` | Société du programme, issue de `<path>.AppCompany` (Windows 8 et versions ultérieures). |
| `KeyPath`, `KeyLastWrittenUTC` | Clé MuiCache et sa dernière écriture : la modification la plus récente de l'un quelconque de ses programmes. |
| `FriendlyAppNameRaw`, `AppCompanyRaw` | Octets de la valeur (hexadécimal), lorsqu'il ne s'agit pas d'un texte qui se décode proprement. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre : `replayed` lorsque les journaux ont écrit l'une des valeurs du programme. |
| `Problems` | Valeurs qui ne sont pas des chaînes, ne sont pas de l'UTF-16 valide ou sont illisibles. |

## Stratégie d'investigation { #investigation-strategy }

1. Examinez les programmes situés dans des emplacements accessibles en écriture par l'utilisateur (`\Users\`, `Downloads`, `Temp`, `ProgramData`), sur
   d'autres lecteurs et sur des partages réseau, ainsi que les outils d'administration, d'accès à distance et d'archivage.
2. Comparez le nom du fichier avec son nom convivial et sa société : une incohérence peut révéler un outil
   renommé ou un fichier usurpant une identité.
3. Corrélez avec Prefetch, BAM et UserAssist (moment de l'exécution), Amcache et ShimCache (empreinte et
   présence du fichier), et la `$MFT` (présence actuelle du fichier).
4. Utilisez `parse-registry read --recover-deleted` sur `UsrClass.dat` pour les valeurs MuiCache supprimées.

## Plus de détails { #further-details }

- [Microsoft : ressource VERSIONINFO](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage : expériences sur MuiCache, sessions graphiques et limites liées au compte du processus](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
