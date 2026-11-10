# Périphériques USB { #usb-devices }

## Qu'est-ce que l'historique USB { #what-the-usb-history-is }

Windows enregistre chaque périphérique qu'il installe. Pour le stockage USB, les autres périphériques USB et les appareils
portables (téléphones, appareils photo, lecteurs multimédias), la ruche `SYSTEM` conserve une clé par périphérique avec son nom,
son numéro de série et les heures auxquelles Windows l'a installé, l'a vu arriver pour la dernière fois et l'a vu retiré pour la dernière fois.
`MountedDevices` enregistre les GUID de volume et les lettres de lecteur attribués à leurs volumes. La ruche
`SOFTWARE` conserve les noms que l'Explorateur a donnés aux appareils portables et aux volumes, ainsi que l'enregistrement par ReadyBoost
des numéros de série de volume. Le `NTUSER.DAT` de chaque utilisateur enregistre les volumes que cet utilisateur a
montés.

`parse-registry usb` les réunit en un enregistrement par périphérique et par volume : de quel périphérique il s'agissait, quand
il a été installé et connecté, quelle lettre de lecteur et quel volume il avait, et quels utilisateurs avaient ce
volume monté.

<span class="evidence-strength">Meilleure conclusion :</span> un périphérique avec ce fabricant, ce produit et ce
numéro de série a été connecté à ce système, pour la première et la dernière fois à ces heures ; son volume avait cette
lettre de lecteur ; ces utilisateurs l'avaient monté. Les enregistrements du Registre ne montrent pas quels fichiers ont été copiés.

## Emplacement { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| Propriété de périphérique | Champ | Signification (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate` : date de la dernière installation de son pilote |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate` : date de la première installation du périphérique |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate` : date de sa dernière connexion (Windows 8 et versions ultérieures) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate` : date de son dernier retrait (Windows 8 et versions ultérieures) |

Windows 8 et versions ultérieures conservent chaque heure comme valeur par défaut de la clé `0064` … `0067`. Windows 7
la conserve dans `00000064\00000000\Data`. Juntei lit les deux.

De nombreuses références DFIR désignent `0064` comme la première installation et `0065` comme l'installation. Juntei suit
les définitions de `devpkey.h` de Microsoft ci-dessus, et nomme les colonnes d'après ce que chaque heure enregistre.

Fournissez la ruche `SYSTEM` à `-f` ou `-d`. La ruche `SOFTWARE` du système et les ruches `NTUSER.DAT` de ses utilisateurs
sont trouvées à côté d'elle, telles qu'une collecte les conserve (`Windows\System32\config\SOFTWARE`,
`Users\<user>\NTUSER.DAT`), ou dans le même répertoire et un niveau en dessous lorsque les ruches ont été
exportées ensemble. Collectez-les toutes avec leurs journaux de transactions.

## Comment les enregistrements sont joints { #how-the-records-are-joined }

- **Périphériques :** un enregistrement par clé de périphérique : `usbstor`, `usb` et `wpdbusenum` (`DeviceClass`). Un
  périphérique de stockage USB prend ses identifiants de fabricant et de produit du périphérique USB ayant le même
  identifiant de conteneur (à défaut, le même numéro de série).
- **Volumes :** les volumes d'un périphérique de stockage USB sont les valeurs `\??\Volume{GUID}` de
  `MountedDevices` dont les données sont un chemin qui nomme le périphérique. Un périphérique comportant deux volumes a deux
  enregistrements. Ses lettres de lecteur sont les valeurs `\DosDevices\X:` ayant les mêmes données.
- **Noms et numéros de série :** les noms de volume proviennent de `Windows Portable Devices` et des
  appareils portables qui nomment le même périphérique. Les étiquettes et numéros de série de volume proviennent des
  clés `EMDMgmt` qui le nomment.
- **Utilisateurs :** les utilisateurs dont le `MountPoints2` contient le GUID du volume, avec l'heure de dernière écriture de cette clé.
- **Volumes sans clé de périphérique** (`volume`) : un volume de `MountedDevices` qu'aucune clé de périphérique ne nomme (celui d'un
  disque interne, ou d'un périphérique retiré dont la clé `Enum` a disparu), une lettre de lecteur dont aucune valeur de GUID de volume ne partage
  les données, et un volume que seuls les `MountPoints2` des utilisateurs contiennent.
- **Périphériques que seule `SOFTWARE` nomme** (`portable_device`, `readyboost`).

`SourceKeys` liste chaque clé et valeur à partir desquelles un enregistrement a été joint, avec sa ruche.

## Précautions d'analyse { #analyst-cautions }

- **Les lettres de lecteur sont réutilisées.** `MountedDevices` conserve le dernier périphérique auquel chaque lettre a été attribuée. Une
  lettre n'est affichée que sur le périphérique dont les données sont toujours celles de cette lettre.
- **Les numéros de série peuvent être inventés.** Lorsque le deuxième caractère d'une instance est `&`, Windows
  l'a inventé parce que le périphérique ne signale aucun numéro de série ; `Serial` est alors vide. Une telle
  instance peut différer sur un autre ordinateur.
- **Les instances de stockage USB se terminent par l'unité logique** (`&0`). `Serial` l'omet ; `Instance`
  la conserve.
- **Les heures diffèrent selon la version de Windows.** Windows 7 ne dispose que des heures d'installation. `LastArrivalUTC` et
  `LastRemovalUTC` correspondent uniquement à la dernière connexion, et non à chaque connexion. Les journaux d'événements
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`) et
  `setupapi.dev.log` en fournissent davantage.
- **Jeux de contrôles :** par défaut, le jeu de contrôles utilisé en dernier par Windows est lu. `--all-control-sets`
  lit tous les jeux de contrôles ; un jeu plus ancien peut encore lister un périphérique que le jeu actuel n'a plus.
- **L'heure de dernière écriture d'une clé n'est pas une heure de connexion.** Windows réécrit les clés de périphérique pour de nombreuses
  raisons.
- **Les périphériques retirés laissent des traces.** Un enregistrement de volume comportant un fabricant et un produit mais sans clé de périphérique
  correspond à un périphérique dont la clé `Enum` a été supprimée (par une désinstallation ou par des outils de nettoyage) et dont le volume
  figure encore dans `MountedDevices`.
- **Les numéros de série ReadyBoost** sont le numéro de série du volume (`VolumeSerials`, tel que `dir` l'affiche),
  que les fichiers LNK et les Jump Lists enregistrent également.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

Le résumé liste les périphériques de stockage USB arrivés en dernier (du plus récent au plus ancien), avec leurs lettres
de lecteur, ainsi que le nombre de volumes appartenant à des périphériques de stockage USB sans clé de périphérique.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host` | Le nom de l'ordinateur, provenant de `SYSTEM`. |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum` ou, sans clé de périphérique, `volume`, `portable_device` ou `readyboost`. |
| `ControlSet` | Le jeu de contrôles de la clé de périphérique. |
| `Vendor`, `Product`, `Revision` | Stockage USB : d'après le nom du périphérique (`Disk&Ven_…&Prod_…&Rev_…`), y compris pour un périphérique nommé par un volume ou une clé `SOFTWARE`. |
| `Vid`, `PID` | Les identifiants USB de fabricant et de produit. |
| `Serial`, `Instance` | Le numéro de série, et le nom de la clé d'instance tel qu'écrit. |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | Les valeurs de la clé de périphérique, telles qu'écrites (`DeviceDesc` est souvent `@<inf>,%…%;<text>`). Le `FriendlyName` d'un appareil portable est généralement l'étiquette ou le lecteur de son volume. |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | Les heures des propriétés du périphérique (voir ci-dessus). |
| `DriveLetters` | Les lettres de lecteur dont les données `MountedDevices` sont celles du volume. |
| `VolumeGUID`, `VolumeData` | Le GUID du volume, et ce que ses données `MountedDevices` indiquent comme support : un chemin de périphérique, une signature de disque MBR et un décalage de partition, ou une partition GPT. |
| `VolumeNames`, `VolumeSerials` | Les noms que l'Explorateur a donnés au périphérique ou à son volume, les étiquettes ReadyBoost et les numéros de série de volume (`1234-ABCD`). |
| `User` | Les utilisateurs dont le `MountPoints2` contient le volume, avec la dernière écriture de cette clé (JSON : `User`, `MountPoints2LastWritten`, `SourcePath`). |
| `FoundIn`, `SourceKeys` | L'endroit où il a été trouvé (`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`), et chaque clé et valeur à partir desquelles il a été joint. |
| `KeyPath`, `KeyLastWrittenUTC` | La clé d'instance du périphérique et sa dernière écriture. |
| `SourcePath`, `SoftwarePath` | La ruche `SYSTEM`, et la ruche `SOFTWARE` lorsqu'elle a fourni des informations. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre des ruches à partir desquelles l'enregistrement a été joint. |
| `Problems` | Clés illisibles, et propriétés de périphérique qui ne sont pas des FILETIME (avec leurs octets). |

## Stratégie d'investigation { #investigation-strategy }

1. Triez les périphériques de stockage USB par `LastArrivalUTC` (à défaut, `DriverLastInstalledUTC`), et mettez de côté les
   lecteurs et concentrateurs propres à l'ordinateur.
2. Notez le fabricant, le produit, le numéro de série, le GUID de volume, la lettre de lecteur et le numéro de série de volume
   de chaque périphérique, et recherchez-les dans les fichiers LNK, les Jump Lists, les ShellBags et le `$MFT` (chemins situés sur cette
   lettre de lecteur ou ce volume).
3. Consultez `User` pour savoir qui avait monté le volume, et quand (la dernière écriture de la clé `MountPoints2`).
4. Recherchez les enregistrements `volume` qui nomment un périphérique de stockage USB : des périphériques dont les clés ont été supprimées.
5. Corrélez avec les journaux d'événements pour chaque connexion, et pas seulement la dernière.
6. Utilisez `parse-registry read --recover-deleted` pour les clés de périphérique supprimées.

## Plus de détails { #further-details }

- [Microsoft : propriétés de périphérique (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft : prise en charge des requêtes du gestionnaire de montage](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (tel que le contiennent les en-têtes de Wine)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h) : les numéros des propriétés de périphérique
- [libyal winreg-kb : USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md),
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) et
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

Les jointures d'utilisateurs s'appuient sur l'identité source de chaque ruche ; les ruches exportées portant le même nom de profil restent donc distinctes. `SourceKeys` conserve les chemins des ruches utilisateur contributrices. La découverte des utilisateurs associés ignore les liens symboliques, conformément à la découverte récursive des ruches. La corrélation VID/PID nécessite un identifiant de conteneur exploitable correspondant ; à défaut, la correspondance par numéro de série doit produire des identifiants sans ambiguïté. Les candidats contradictoires restent inconnus, avec un problème signalé.
