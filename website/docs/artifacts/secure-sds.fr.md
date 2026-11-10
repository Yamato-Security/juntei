# Descripteurs de sécurité (`$Secure:$SDS`) { #security-descriptors-securesds }

## Qu'est-ce que `$SDS` { #what-sds-is }

Chaque fichier et répertoire sur NTFS possède un **descripteur de sécurité** : son propriétaire, son groupe principal, une
**DACL** (qui peut faire quoi avec lui) et une **SACL** (quels accès sont audités, et son étiquette d'intégrité).
NTFS conserve chaque descripteur distinct une seule fois, dans le flux `$SDS` du fichier `$Secure` (entrée MFT 9),
et lui attribue un **ID de sécurité**. Le `$STANDARD_INFORMATION` d'un fichier n'enregistre que cet ID de sécurité : la
colonne `SID` de [`parse-ntfs mft`](mft.md). `$SDS` le convertit en propriétaire et en ACL.

`$SDS` est organisé en blocs de 256 Kio, chacun suivi d'une copie de lui-même (son miroir). Chaque entrée comporte une empreinte de
son descripteur, que NTFS utilise pour retrouver un descripteur déjà existant (`$SDH`), ainsi que
l'ID de sécurité (indexé par `$SII`).

<span class="evidence-strength">Meilleure conclusion :</span> les fichiers dont le `SID` désigne un
descripteur avaient, lors de la dernière écriture de leur enregistrement `$MFT`, ce propriétaire et ces ACL. Le propriétaire d'un fichier
est généralement le compte qui l'a créé, mais un administrateur, un programme d'installation ou un programme
qui s'en approprie la propriété peut le modifier.

## Emplacement et collecte { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

Collectez-le avec un outil forensique capable d'un accès brut ; les collectes le nomment généralement `$Secure_$SDS`, puisque `:`
ne peut pas figurer dans un nom de fichier. Collectez avec lui la `$MFT` du même volume.

## Précautions d'analyse { #analyst-cautions }

- Un propriétaire est un SID. Seuls les SID connus (`NT AUTHORITY\SYSTEM`, `BUILTIN\Administrators`,
  `NT SERVICE\TrustedInstaller`, …) et les comptes intégrés d'une machine ou d'un domaine (`-500`
  `Administrator`, …) sont nommés. Résolvez les autres avec la `SAM` de l'hôte
  ([comptes SAM](sam.md)), sa liste de profils ([Informations système](system-info.md)) ou le domaine.
- Un descripteur est partagé par tous les fichiers ayant le même propriétaire et les mêmes ACL, et reste généralement dans `$SDS`
  après la suppression de ces fichiers. Un descripteur sans fichier dans la `$MFT` ne correspond pas, à lui seul, à un
  fichier supprimé.
- Une ACE n'est pas un historique des accès. La DACL indique qui *peut* accéder à un fichier ; les journaux d'événements et d'autres
  artefacts indiquent qui l'a fait. Les entrées d'audit de la SACL décrivent une configuration, et non ce qui a été journalisé.
- Les ACE sont listées dans leur ordre au sein de l'ACL. Juntei ne calcule pas l'accès effectif : les entrées
  de refus, l'héritage, l'appartenance aux groupes, les privilèges et les droits implicites du propriétaire entrent tous en ligne de compte.
- `DACL` et `SACL` écrivent chaque ACE comme le fait `icacls`. `SDDL` représente les autorisations prises en charge, avec les masques en hexadécimal et les comptes sous forme de SID. Ce n'est pas une copie octet par octet du descripteur binaire ; un texte conditionnel non représentable le laisse vide et fait l'objet d'un diagnostic.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

La synthèse compte les descripteurs et liste leurs propriétaires, avec le nombre de descripteurs que possède chacun,
les descripteurs dotés d'une DACL NULL (aucune restriction) et les éventuels dommages.

## Champs de sortie CSV { #csv-output-fields }

| Champ | Interprétation |
|---|---|
| `SourcePath` | Fichier d'entrée. |
| `SecurityID` | L'ID de sécurité : joignez-le au `SID` de [`parse-ntfs mft`](mft.md) pour le même volume. |
| `Hash` | Empreinte enregistrée avec le descripteur. |
| `EntryOffset` | Position de l'entrée dans le flux. |
| `FromMirror` | `True` pour une copie lue dans le bloc miroir : une copie qui diffère de son entrée (listée après celle-ci), ou la copie d'une entrée endommagée. |
| `Size` | Taille de l'entrée, en-tête de 20 octets compris. |
| `Owner`, `OwnerName` | SID du propriétaire, et son nom s'il est connu. |
| `Group`, `GroupName` | SID du groupe principal, et son nom s'il est connu. Windows ne l'utilise pas pour les vérifications d'accès. |
| `Control` | Indicateurs de contrôle du descripteur : `SE_DACL_PRESENT`, `SE_DACL_PROTECTED` (héritage bloqué), `SE_DACL_AUTO_INHERITED`, … joints par `|`. |
| `DACL-ACE-Count`, `DACL` | Entrées de la DACL, séparées par `; `. Vide en l'absence de DACL ; `NULL DACL: no restriction` si le descripteur accorde tout à tout le monde. |
| `SACL-ACE-Count`, `SACL` | Entrées de la SACL : entrées d'audit, étiquette d'intégrité obligatoire, étiquettes de confiance de processus et attributs de ressource. |
| `SDDL` | Le descripteur complet en SDDL. Vide uniquement si l'un de ses éléments n'est pas décodé. |
| `Problems` | Empreinte ne correspondant pas au descripteur, parties indécodables, ACE de type inconnu. |

Chaque entrée de `DACL` et de `SACL` est écrite comme l'écrit `icacls` : le compte, le type s'il ne s'agit pas
d'une entrée d'autorisation, les indicateurs d'héritage et les droits.

| Partie | Signification |
|---|---|
| Compte | Le nom connu, ou le SID. |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | Le type d'ACE, lorsqu'il ne s'agit pas d'une entrée d'autorisation. Aucun type n'est affiché pour une étiquette obligatoire. |
| `(I)` | Hérité du répertoire parent. |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | Hérité par les fichiers (héritage d'objet), par les répertoires (héritage de conteneur), uniquement par ceux-ci et non par cet objet (héritage seul), et uniquement sur un niveau (pas de propagation). |
| `(SA)`, `(FA)` | Une entrée d'audit pour les accès réussis ou échoués. |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | Contrôle total, modification, lecture et exécution, lecture, écriture. |
| `(RD,WD,AD,…)` | Droits spécifiques dans les autres cas : lecture des données, écriture des données, ajout de données, lecture et écriture des attributs étendus (`REA`, `WEA`), exécution (`X`), suppression d'enfant (`DC`), lecture et écriture des attributs (`RA`, `WA`), suppression (`DE`), lecture du contrôle (`RC`), écriture de la DAC (`WDAC`), écriture du propriétaire (`WO`), synchronisation (`S`), et les droits génériques `GA`, `GR`, `GW`, `GE`. `N` signifie aucun accès. |
| `(NW)`, `(NR)`, `(NX)` | Stratégie d'une étiquette obligatoire : pas d'écriture vers le haut, pas de lecture vers le haut, pas d'exécution vers le haut. |
| `if (…)` | Condition d'une ACE de rappel (callback), telle que `(WIN://SYSAPPID Contains "…")` : l'entrée ne s'applique qu'aux processus pour lesquels elle est vraie. Windows les utilise pour donner accès aux packages de conteneurs d'applications (app containers). |

Les étiquettes de confiance de processus et les attributs de ressource affichent leur masque en hexadécimal, et un attribut
de ressource sa revendication (`("IMAGELOAD",TU,0x0,1)`).

JSON/JSONL comportent les mêmes champs et noms, avec les nombres sous forme de nombres, `null` pour les valeurs vides et
`Problems` sous forme de liste.

## Stratégie d'investigation { #investigation-strategy }

1. Analysez la `$MFT` du même volume et joignez son `SID` à celui de cette sortie : chaque fichier obtient son
   propriétaire et ses ACL.
2. Examinez les propriétaires qui sont des comptes utilisateur (`S-1-5-21-…`) : quels fichiers possède un compte suspect,
   et quels comptes possèdent des fichiers dans les répertoires système.
3. Recherchez des ACL permissives sur les répertoires à partir desquels des programmes s'exécutent ou sont chargés : `Everyone`,
   `Authenticated Users` ou `User` avec `(F)`, `(M)` ou `(W)`, ainsi que des DACL NULL.
4. Un descripteur utilisé par un seul fichier, ou par un ensemble inhabituel de fichiers, peut révéler des fichiers
   créés ou modifiés en dehors d'une installation normale.
5. Corroborez les modifications de propriété et d'autorisations avec les entrées du journal d'événements Security (4670, 4663 avec une
   SACL) et les enregistrements `SECURITY_CHANGE` du [journal USN](usn-journal.md).

## Plus de détails { #further-details }

- [Microsoft : descripteurs de sécurité](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications : MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (descripteurs de sécurité, ACE, expressions conditionnelles)
- [Microsoft : format de chaîne des descripteurs de sécurité (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft : icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs : documentation du format NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (son crate `secure`)

Avec une sortie structurée sur stdout (`-t` sans `-o`), les avertissements de récupération au niveau du flux sont envoyés sur stderr, de sorte que les pipes conservent un CSV/JSON/JSONL lisible sans masquer les régions endommagées ou récupérées.
