# Données de configuration de démarrage (BCD) { #boot-configuration-data-bcd }

## Qu'est-ce que le magasin BCD ? { #what-the-bcd-store-is }

Windows Vista et les versions ultérieures conservent leur configuration de démarrage dans une ruche du Registre, le magasin BCD :
`\Boot\BCD` sur les systèmes BIOS, `\EFI\Microsoft\Boot\BCD` sur la partition système EFI des systèmes
UEFI. `bcdedit` et le programme d'installation de Windows le modifient ; le gestionnaire de démarrage le lit à chaque démarrage.

Le magasin contient des objets, chacun nommé d'après un GUID :

- le **gestionnaire de démarrage Windows** (`{bootmgr}`), avec l'entrée par défaut, l'ordre d'affichage et le
  délai d'attente du menu ; sur les systèmes UEFI, également le **gestionnaire de démarrage du firmware** (`{fwbootmgr}`) avec
  l'ordre de démarrage du firmware ;
- un **chargeur de démarrage Windows** par installation de Windows (y compris l'environnement de récupération), une
  **application de reprise** (hibernation) et le **testeur de mémoire** ;
- des objets de **paramètres hérités** (`{globalsettings}`, `{bootloadersettings}`, …) dont les paramètres sont
  hérités par les autres ;
- des objets **périphérique** (options ramdisk de l'environnement de récupération).

Chaque objet possède des éléments : une description, un périphérique et un chemin, ainsi que des paramètres tels que
`recoveryenabled`, `bootstatuspolicy`, `testsigning`, `nointegritychecks` et `safeboot`.

Deux types d'attaques les modifient :

- Les **ransomwares** désactivent la récupération afin que le système ne puisse pas être réparé :
  `bcdedit /set {default} recoveryenabled No` et
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`. Certaines familles définissent `safeboot` afin
  que Windows redémarre en mode sans échec, où les produits de sécurité ne s'exécutent pas.
- Les **rootkits et pilotes non signés** peuvent utiliser `testsigning on` ou `nointegritychecks on` pour affaiblir les contrôles au démarrage. Ces paramètres ne prouvent pas à eux seuls qu'un rootkit s'est exécuté.

<span class="evidence-strength">Meilleure conclusion :</span> au moment de la collecte, cette entrée de démarrage
avait ce paramètre. L'heure de dernière écriture de la clé de l'élément correspond au moment où le paramètre a été écrit pour la dernière fois.

## Emplacement { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

Le type d'un élément est composé de 8 chiffres hexadécimaux : sa classe (`1` toutes les applications, `2` un type
d'application, `3` périphériques), son format (`1` périphérique, `2` chaîne, `3` objet, `4` liste d'objets, `5`
entier, `6` booléen, `7` liste d'entiers) et son numéro. La commande nomme les éléments tels que
Microsoft les documente (`BcdLibraryBoolean_AutoRecoveryEnabled`).

Les outils de collecte ne copient souvent pas le magasin : la partition système EFI n'est pas montée et `\Boot`
est masqué. Collectez-le explicitement ou montez la partition. `-d` lit également `BCD-Template`
(`HiveType` indique lequel) ; le modèle est ce à partir de quoi Windows démarre, et non le magasin avec lequel il amorce.

## Précautions d'analyse { #analyst-cautions }

- **Les paramètres sont hérités.** Un chargeur de démarrage hérite des objets de sa
  `BcdLibraryObjectList_InheritedObjects` (généralement `{bootloadersettings}`, qui hérite de
  `{globalsettings}`). `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks`, `SafeBoot` et
  `BootStatusPolicy` sont des valeurs stockées résolues (propres, sinon héritées), et `Inherited` indique lesquelles
  proviennent d'un autre objet. L'héritage est suivi sur huit niveaux au maximum, les cycles étant bornés. Un objet référencé peut être absent d'un magasin partiel. Des champs vides n'établissent pas une valeur par défaut effective de Windows. Le firmware et Secure Boot peuvent également empêcher une option stockée de prendre effet.
- **Non défini ne signifie pas désactivé.** Un `RecoveryEnabled` vide signifie que l'élément n'est pas présent ; la valeur
  par défaut de Windows s'applique.
- **Les horodatages sont propres à chaque élément.** L'heure de la clé de chaque élément (`key_last_written` dans `Elements`) correspond au moment où il
  a été écrit pour la dernière fois. Windows réécrit certains éléments lors des mises à jour et des mises à niveau ; comparez avec les horodatages
  des autres éléments.
- **Les périphériques sont partiellement décodés.** Une partition GPT est indiquée par les GUID de sa partition et de son disque ;
  un ramdisk par son objet d'options. Les autres périphériques ne conservent que leurs octets (`data_raw`).
- **Les éléments non documentés** (`0x15000065`, la classe `0x4…`) n'ont pas de nom et conservent leurs octets.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

Le résumé répertorie l'entrée par défaut de chaque magasin ainsi que chaque paramètre de signature de test, de contrôle d'intégrité, de récupération,
de stratégie d'état de démarrage (tout `Ignore…`) et de mode sans échec, avec l'heure à laquelle il a été défini ou l'objet
dont il est hérité.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | L'hôte, la ruche, et `BCD` ou `BCD-Template`. |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | Le GUID de l'objet, son type (`0x10200003`) et sa nature (`Windows boot loader`, `inherited settings for Windows boot loader`, …). |
| `Description`, `Path`, `Device` | La description, le chemin de l'application et son périphérique (une partition et un disque GPT, ou l'objet d'options d'un ramdisk). |
| `IsDefault`, `DisplayOrder` | Si le gestionnaire de démarrage Windows le démarre par défaut, et sa position dans le menu de démarrage. |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | `recoveryenabled`, `testsigning`, `nointegritychecks`, tels que résolus à partir du magasin. |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`, `Network`, `DsRepair`) et `bootstatuspolicy` (`IgnoreAllFailures`, …), avec le numéro. |
| `Inherited` | Lesquels de ces paramètres proviennent d'un autre objet, et de quel objet. |
| `Elements` | Chaque élément : son type, son nom, sa valeur, ses octets (pour les périphériques, les types non documentés et les données qui ne se décodent pas) et l'heure de sa clé. |
| `KeyLastWrittenUTC` | La dernière écriture de la clé de l'objet. |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Objets sans type, éléments dont les données ne correspondent pas à leur format. |

## Stratégie d'investigation { #investigation-strategy }

1. Vérifiez l'entrée par défaut et chaque chargeur de démarrage pour `TestSigning`, `NoIntegrityChecks`,
   `RecoveryEnabled = false`, une `BootStatusPolicy` de type `Ignore…` et `SafeBoot`.
2. Utilisez les horodatages des éléments pour dater la modification, et corrélez avec `bcdedit.exe` dans Prefetch,
   Amcache, les événements de création de processus (4688, Sysmon 1) et le journal d'événements System.
3. Recherchez des entrées de démarrage ou des périphériques qui ne devraient pas être présents (un autre chemin de chargeur, un ramdisk), ainsi
   qu'une modification de l'entrée par défaut ou de l'ordre d'affichage.
4. Comparez le magasin avec `BCD-Template` et avec un autre système de la même build.

## Plus de détails { #further-details }

- [Microsoft : référence des options de démarrage BCD](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft : BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft : BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft : BcdObject (types d'objets)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
