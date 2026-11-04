# BAM / DAM { #bam-and-dam }

## Que sont BAM et DAM { #what-bam-and-dam-are }

Le Background Activity Moderator (BAM) et le Desktop Activity Moderator (DAM) sont des services Windows
(Windows 10 1709 et versions ultérieures) qui gèrent ce que les applications peuvent faire en arrière-plan.
Ils conservent, par utilisateur, la dernière heure d'exécution de chaque programme, dans la ruche `SYSTEM` :

- une clé par utilisateur, nommée d'après le SID de l'utilisateur ;
- une valeur par programme : son nom est le chemin du programme sur son périphérique
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) ou l'identifiant d'une application empaquetée
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`) ;
- les 8 premiers octets des données de la valeur correspondent à l'heure de sa dernière exécution (FILETIME).

BAM associe un programme à un utilisateur et à une heure sans la ruche propre à l'utilisateur, et il liste les programmes exécutés
depuis des périphériques amovibles et réseau.

<span class="evidence-strength">Meilleure conclusion :</span> un processus de ce programme s'est exécuté dans la
session de cet utilisateur ; sa dernière exécution date de cette heure. BAM ne compte pas les exécutions, et une entrée peut subsister après la
suppression du programme.

## Emplacement { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

La clé de chaque utilisateur contient aussi les valeurs `Version` et `SequenceNumber` ; ce ne sont pas des programmes et elles sont
ignorées. Un système mis à niveau depuis 1709 ou 1803 peut présenter les deux dispositions.

Collectez `SYSTEM` avec ses journaux de transactions, ainsi que les ruches `SAM` et `SOFTWARE` du système : elles donnent le nom des
utilisateurs.

## Précautions d'analyse { #analyst-cautions }

- **Une heure par programme et par utilisateur.** Seule l'exécution la plus récente est conservée. Prefetch, Amcache et les
  journaux d'événements fournissent les exécutions antérieures.
- **Activité récente.** Les entrées des programmes qui n'ont pas été exécutés depuis un certain temps seraient
  supprimées au démarrage de Windows ; BAM montre donc surtout l'activité récente : un programme qui n'y figure pas
  peut néanmoins avoir été exécuté il y a longtemps.
- **Des chemins de périphérique, pas des lettres de lecteur.** `\Device\HarddiskVolume3` est le troisième volume numéroté par Windows,
  et non `C:`. `DevicePath` et `Path` séparent le périphérique du chemin qui s'y trouve ; faites correspondre le
  volume avec `MountedDevices`, le `$MFT` ou d'autres artefacts avant d'écrire une lettre de lecteur.
- **Pas tous les programmes.** BAM et DAM suivent les programmes dans les sessions des utilisateurs ; les services n'y figurent généralement
  pas. Les entrées `S-1-5-18` correspondent généralement à des processus de l'installation et de l'écran de connexion
  (`FirstLogonAnim.exe`), et les entrées `S-1-5-90-0-<n>` au Desktop Window Manager (`DWM-<n>`).
- **Jeux de contrôles.** Par défaut, le jeu de contrôles utilisé en dernier par Windows est lu. `--all-control-sets`
  les lit tous : un jeu de contrôles plus ancien peut conserver une entrée que le jeu actuel n'a plus.
- **Noms d'utilisateur.** Le nom d'un compte local provient de la ruche `SAM` (le SID de l'ordinateur et le RID du
  compte) ; celui d'un compte de domaine provient de son dossier de profil dans `ProfileList`. `UserSource`
  indique laquelle des deux (`sam`, `profile_list`, `well_known`). Sans elles, seul le SID est fourni. Un RID
  que la `SAM` ne contient plus (un compte supprimé) reste sans nom.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Le résumé liste chaque utilisateur avec son SID, son nom et son nombre d'entrées, ainsi que les derniers programmes exécutés
(du plus récent au plus ancien).

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath` | Hôte et ruche `SYSTEM`. |
| `ControlSet`, `IsCurrentControlSet` | Le jeu de contrôles contenant l'entrée, et s'il s'agit de celui utilisé en dernier par Windows. |
| `Moderator` | `bam` ou `dam`. |
| `UserSID`, `User`, `UserSource` | Le SID qui donne son nom à la clé, le nom de l'utilisateur et sa provenance (`sam`, `profile_list`, `well_known`). |
| `Program` | Le nom de la valeur tel que Windows l'a écrit : un chemin de périphérique ou l'identifiant d'une application empaquetée. |
| `DevicePath`, `Path` | Pour un chemin de périphérique, le périphérique (`\Device\HarddiskVolume3`) et le chemin qui s'y trouve (`\Windows\System32\cmd.exe`) ; vide pour une application empaquetée. |
| `LastRunUTC` | L'heure de la dernière exécution du programme (à 100 ns près). |
| `KeyPath`, `KeyLastWrittenUTC` | La clé de l'utilisateur et sa dernière écriture : la mise à jour la plus récente de l'une quelconque de ses entrées. |
| `DataRaw` | Les octets de la valeur (hexadécimal), lorsqu'ils ne contiennent pas d'heure : type autre que `REG_BINARY`, ou moins de 8 octets. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Données ne contenant pas d'heure, et valeurs illisibles. |

## Stratégie d'investigation { #investigation-strategy }

1. Triez par `LastRunUTC` et examinez les programmes situés dans des emplacements accessibles en écriture par l'utilisateur (`\Users\`, `Downloads`,
   `Temp`, `ProgramData`), sur des périphériques autres que le volume système, ainsi que les outils d'administration et
   d'accès à distance.
2. Vérifiez quel utilisateur les a exécutés, ainsi que les comptes qui ne correspondent à aucun profil actuel.
3. Corrélez avec Prefetch (nombre d'exécutions et exécutions antérieures), UserAssist (lancements depuis l'Explorateur), Amcache
   et ShimCache (présence), le `$MFT` (le fichier du programme) et les journaux d'événements.
4. Utilisez `--all-control-sets`, et `parse-registry read --recover-deleted` pour les valeurs BAM supprimées.

## Plus de détails { #further-details }

- [libyal winreg-kb : Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft : identificateurs de sécurité (SID connus)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

Les FILETIME de dernière exécution hors plage laissent `LastRunUTC` vide et conservent la valeur d'origine dans `DataRaw`, avec une explication dans `Problems`. Le résumé de l'activité récente identifie chaque hôte et maintient distinctes les observations de ruches/utilisateurs différents.
