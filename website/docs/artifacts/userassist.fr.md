# UserAssist { #userassist }

## Qu'est-ce qu'UserAssist { #what-userassist-is }

L'Explorateur Windows comptabilise les programmes et raccourcis qu'un utilisateur lance depuis le shell de l'Explorateur : le
menu Démarrer, la barre des tâches, le Bureau et les doubles-clics dans les dossiers. Il conserve ces décomptes dans le
`NTUSER.DAT` de l'utilisateur. Chaque élément comporte :

- un nombre d'exécutions ;
- sur Windows 7 et versions ultérieures, combien de fois et pendant combien de temps (en millisecondes) il a eu le focus ;
- l'heure de sa dernière exécution.

Les noms des valeurs sont encodés en ROT13. Le nom d'un programme commence généralement par le GUID d'un dossier connu
(`{1AC14E77-…}` correspond à `System32`).

UserAssist montre ce qu'un utilisateur donné a lancé de manière interactive, ce que la plupart des autres preuves d'exécution
n'attribuent pas à un utilisateur.

<span class="evidence-strength">Meilleure conclusion :</span> l'utilisateur a lancé ce programme ou ce raccourci
via l'Explorateur, ce nombre de fois, la dernière fois à cette heure. Les programmes lancés d'une autre manière (ligne de
commande, service, tâche planifiée, autre programme) ne sont pas comptabilisés.

## Emplacement { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | Contenu |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | Programmes (`executable`), Windows 7 et versions ultérieures |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | Raccourcis (`shortcut`), Windows 7 et versions ultérieures |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | Programmes et raccourcis (`active_desktop`), de Windows 2000 à Vista |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | Barre d'outils Internet (`internet_toolbar`), de Windows 2000 à Vista |

Les autres GUID sont signalés comme `other`. Collectez `NTUSER.DAT` avec ses journaux de transactions pour chaque
profil, ainsi que la ruche `SOFTWARE` du système, qui fournit le SID de l'utilisateur.

## Précautions d'analyse { #analyst-cautions }

- **Les nouveaux profils sont pré-remplis.** Sur Windows 7 et versions ultérieures, un nouveau profil commence avec des éléments par défaut
  (Getting Started, Calculatrice, Paint, Pense-bêtes, …). Leurs nombres d'exécutions décomptent à partir de 14,
  et ils partagent tous une même heure de dernière exécution, proche de la création du profil. Il ne s'agit pas d'une
  activité de l'utilisateur.
- **La version 3 compte à partir de 5** (de Windows 2000 à Vista). Le décompte stocké commence à 5 ; 5 en est donc
  soustrait. Un décompte stocké inférieur à 5 est signalé dans `Problems`.
- **Les chemins utilisent le chemin par défaut du dossier connu** (`%windir%\system32`, `%ProgramFiles%`,
  `%APPDATA%\…`). Un dossier redirigé ailleurs affiche toujours son chemin par défaut. Les identifiants d'application
  (`Microsoft.Windows.Explorer`, `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) et les éléments du Panneau de configuration
  (`::{…}`) n'ont pas de chemin.
- **Le suivi peut être désactivé** avec `Explorer\Advanced` `Start_TrackProgs` = 0,
  `Policies\Explorer` `NoInstrumentation` = 1, ou `UserAssist\Settings` `NoLog` = 1. Le résumé
  liste les ruches où il est désactivé. Un UserAssist vide ou absent sur un profil actif peut indiquer
  de l'anti-forensique.
- **`NoEncrypt` = 1** amène Windows à écrire les noms sans ROT13. Ces noms sont mal décodés ; c'est pourquoi
  chaque élément de cette ruche le signale dans `Problems` (et sur stderr lorsque les enregistrements sont envoyés vers stdout).
- **L'heure de dernière exécution et les décomptes sont conservés par élément,** mais l'heure de dernière écriture de la clé
  (`KeyLastWrittenUTC`) correspond seulement à la mise à jour la plus récente de l'un quelconque des éléments.
- **Les nombres d'exécutions peuvent valoir 0** pour les éléments qui ont seulement eu le focus. Les décomptes et durées de focus n'existent que dans
  la version 5 (Windows 7 et versions ultérieures).

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

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
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Le résumé liste les éléments exécutés en dernier (du plus récent au plus ancien) avec leurs utilisateurs, ainsi que les ruches où
UserAssist est désactivé.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Hôte, nom du dossier de profil, SID de l'utilisateur provenant du `ProfileList` du système (avec la ruche `SOFTWARE` du système), et ruche. |
| `GUID`, `Kind` | La clé `{GUID}` et son contenu (`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`). |
| `NameRot13`, `Name` | Le nom de la valeur tel que stocké, et décodé. |
| `Path` | Le nom décodé sous forme de chemin, le GUID d'un dossier connu étant remplacé par son chemin par défaut ; vide pour les identifiants d'application et les éléments du shell. |
| `RunCount` | Le nombre de lancements depuis l'Explorateur (version 3 : le décompte stocké moins 5). |
| `FocusCount`, `FocusTimeMs` | Combien de fois il a eu le focus, et pendant combien de temps en millisecondes (version 5). |
| `LastRunUTC` | L'heure de son dernier lancement. Vide lorsque Windows a stocké 0, ou lorsque l'heure stockée dépasse l'an 9999 (`Problems` l'indique alors et `DataRaw` conserve les octets). |
| `Format`, `SessionID` | La disposition des données (5 : 72 octets, 3 : 16 octets) et le numéro de la session au cours de laquelle elles ont été écrites pour la dernière fois. |
| `KeyLastWrittenUTC` | La dernière écriture de la clé `Count` : la mise à jour la plus récente de l'un quelconque de ses éléments. |
| `DataRaw` | Les octets de la valeur (hexadécimal), lorsqu'ils ne se décodent pas. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Données de taille inconnue, taille en désaccord avec la `Version` de la clé, décompte de version 3 inférieur à 5, dernière exécution au-delà de 9999, `NoEncrypt`. |

## Stratégie d'investigation { #investigation-strategy }

1. Triez par `LastRunUTC` et mettez de côté les éléments pré-remplis : ils partagent une même heure, proche de la
   création du profil.
2. Recherchez les programmes situés dans des emplacements accessibles en écriture par l'utilisateur (`%USERPROFILE%`, `Downloads`, `Temp`,
   `ProgramData`), sur des lecteurs amovibles et des chemins réseau, ainsi que les outils d'administration et d'accès
   à distance.
3. Corrélez avec Prefetch (exécution avec heures d'exécution), Amcache et ShimCache (présence),
   les Jump Lists et les fichiers LNK (ce qui a été ouvert avec eux), et le `$MFT` pour les fichiers des programmes.
4. Vérifiez les ruches où le suivi est désactivé, ainsi que les profils dont l'UserAssist est vide, à la recherche d'un effacement.
5. Utilisez `parse-registry read --recover-deleted` pour les valeurs UserAssist supprimées.

## Plus de détails { #further-details }

- [Microsoft : KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb : User Assist keys](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
