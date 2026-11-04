# Raccourcis LNK { #lnk-shortcuts }

## Que sont les fichiers LNK { #what-lnk-files-are }

Les fichiers Windows Shell Link (`.lnk`) décrivent une cible ainsi que les arguments de lancement, le répertoire de travail, l'icône,
l'état de la fenêtre, les métadonnées de la cible, les informations de volume/réseau, les éléments de l'espace de noms du shell et, de manière facultative,
des identifiants de suivi des liens distribués (distributed link tracking). Windows et les applications les créent dans les éléments récents (Recent Items),
le Bureau, le menu Démarrer et d'autres emplacements ; les attaquants livrent également directement des raccourcis malveillants.

Un fichier LNK du dossier Recent peut étayer une interaction de l'utilisateur avec une cible qui n'existe plus, y compris sur un
lecteur amovible ou un partage réseau. Un fichier LNK quelconque peut exister sans jamais avoir été cliqué.

<span class="evidence-strength">Meilleure conclusion :</span> le raccourci a été créé/mis à jour avec le
contexte de cible enregistré. Son emplacement et les artefacts environnants déterminent s'il étaye une utilisation.

## Emplacements et collecte { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

Recherchez dans l'ensemble du profil utilisateur ainsi que dans les archives et contenus de messagerie collectés, pas seulement dans `Recent`. Préservez
les horodatages et le chemin propres au fichier source, car l'emplacement modifie l'interprétation.

## Précautions d'analyse { #analyst-cautions }

- Les horodatages, la taille, la référence MFT et les données de volume de la cible sont des instantanés stockés dans le LNK ; ce ne sont pas
  les métadonnées propres au fichier de raccourci, et ils peuvent être obsolètes.
- `MachineID`, l'adresse MAC et l'heure de création du tracker décrivent le contexte du suivi des liens distribués, généralement
  la machine qui a créé/mis à jour le lien — pas nécessairement la machine sur laquelle vous l'avez trouvé.
- Un nœud d'UUID ressemblant à une adresse MAC peut provenir de la virtualisation, d'une randomisation ou d'identifiants non réseau.
- `Args` peut rendre dangereux un exécutable bénin (`powershell.exe`, `rundll32.exe`, `mshta.exe`).
- Les chaînes ANSI nécessitent la page de codes du système source ; utilisez `--code-page 932` pour de nombreux liens japonais.
- Un raccourci peut être fabriqué ou copié. N'assimilez pas ses champs à une exécution sans le soutien de Prefetch,
  des Jump Lists, de RecentDocs, des journaux d'événements ou d'autres éléments.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## Champs de sortie CSV { #csv-output-fields }

| Groupe de champs | Champs et interprétation |
|---|---|
| Source | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed` : chemin du raccourci et horodatages du système de fichiers hôte. Ils sont distincts des horodatages de la cible. |
| Horodatages de la cible | `TargetCreated`, `TargetModified`, `TargetAccessed` : instantanés enregistrés dans le LNK ; indices historiques utiles, et non métadonnées actuelles de la cible. |
| Informations de base de la cible | `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags` : contexte de cible/de lancement enregistré. Les arguments peuvent en changer le sens. |
| Volume | `DriveType`, `VolumeSerialNumber`, `VolumeLabel` : identifient le contexte fixe/amovible/réseau et permettent la corrélation avec d'autres raccourcis. |
| Chemins | `LocalPath`, `NetworkPath`, `CommonPath`, `TargetIDAbsolutePath` : représentations complémentaires du chemin. Examinez les divergences et les chemins de l'espace de noms du shell. |
| Comportement de lancement | `Args`, `ShowWindow`, `HotKey`, `ShimLayer`, `DarwinId` : manière dont le shell lance ou configure la cible. `Args` est essentiel. |
| Identité NTFS | `TargetMFTEntryNumber`, `TargetMFTSequenceNumber` : référence de la cible au moment de la création du lien ; faites correspondre les deux avec `$MFT`. |
| Suivi | `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `VolumeDroid`, `FileDroid`, `VolumeDroidBirth`, `FileDroidBirth` : suivi des liens distribués. À utiliser pour la corrélation, et non comme seule base d'attribution. |
| Blocs supplémentaires | `ExtraBlocksPresent` : liste des structures facultatives disponibles ; explique quels autres champs sont renseignés. |
| Présentation | `Name`, `IconLocation`, `IconIndex`, `IconEnvironment`, `EnvironmentVariables` : données d'affichage et données de cible après expansion des variables d'environnement. Les attaquants peuvent utiliser des noms/icônes trompeurs. |
| Dossiers du shell | `KnownFolder`, `SpecialFolder` : identifiants de dossiers du shell Windows décodés. |
| Réseau | `NetworkDevice`, `NetworkProvider` : contexte du partage/périphérique réseau et du fournisseur. |
| Encodage | `ConsoleCodePage` : page de codes de la console enregistrée, si présente ; distincte de l'option `--code-page` du parseur. |
| Données de fin | `AppendedBytes` : octets situés après la fin logique. Des données inattendues peuvent indiquer une contrebande de données (smuggling) ou des fichiers malformés. |
| Validation | `Problems` : dommages, troncature, blocs incohérents ou limites de décodage. À inclure dans l'examen. |

Les sorties JSON/JSONL conservent en outre chaque élément d'identifiant de cible (type, nom, GUID, horodatages FAT et référence
MFT), tous les blocs de données supplémentaires, les propriétés du magasin de propriétés (property store), les paramètres de console ainsi que les relations
structurelles brutes. Utilisez-les pour un raccourci suspect plutôt que de vous fier uniquement à la ligne CSV aplatie.
Les horodatages imbriqués de `TargetItems` proviennent d'horloges FAT d'éléments du shell, sans décalage stocké.
Leur rendu historique en `Z` ne prouve pas qu'il s'agit d'UTC ; `auto-parse --timezone` les conserve tels quels.
Les FILETIME de la cible et les horodatages du tracker figurant dans l'en-tête LNK restent des horodatages absolus convertibles.

## Stratégie d'investigation { #investigation-strategy }

1. Examinez l'emplacement et les horodatages de la source avant d'interpréter les champs de la cible.
2. Reconstituez le lancement effectif à partir du chemin de la cible, de `Args`, de `WorkingDirectory` et des blocs d'environnement.
3. Pivotez sur la référence MFT de la cible, le numéro de série du volume, le chemin réseau, les droids, le MachineID et l'adresse MAC à travers les preuves.
4. Vérifiez Prefetch et les journaux d'événements pour l'exécution ; vérifiez les Jump Lists/RecentDocs pour l'interaction de l'utilisateur.
5. Pour les pièces jointes malveillantes, préservez les données ajoutées en fin de fichier et calculez l'empreinte du LNK lui-même pour l'analyser.

## Plus de détails { #further-details }

- [Microsoft Open Specifications : MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [LECmd d'Eric Zimmerman](https://github.com/EricZimmerman/LECmd)
