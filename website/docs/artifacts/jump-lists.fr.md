# Jump Lists { #jump-lists }

## Que sont les Jump Lists { #what-jump-lists-are }

Les Jump Lists alimentent les menus affichés à partir des entrées d'application de la barre des tâches et du menu Démarrer. Les Jump
Lists automatiques sont des historiques gérés par Windows des éléments utilisés par une application. Les Jump Lists personnalisées sont
des catégories et des tâches définies par l'application. Les deux peuvent incorporer des structures LNK contenant des chemins de cible, des heures,
un contexte de volume/réseau, des références MFT et des arguments de lancement.

Les enregistrements automatiques peuvent révéler des fichiers ou dossiers ouverts par une application, l'ordre/l'heure de dernière utilisation,
le nombre d'interactions et l'état d'épinglage. Les listes personnalisées peuvent exposer des pages « récemment fermées » ou des tâches
d'application — y compris des arguments de commande malveillants.

<span class="evidence-strength">Meilleure conclusion :</span> l'application propriétaire a enregistré ou exposé
l'élément/la tâche dans sa Jump List. Le comportement diffère selon l'application ; corroborez l'exécution/l'accès effectif.

## Emplacements et collecte { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

Le préfixe hexadécimal du nom de fichier est un AppID. Collectez les deux répertoires et conservez le chemin du
profil utilisateur. Les listes automatiques sont des conteneurs OLE Compound File comportant un flux `DestList` et des flux LNK
numérotés ; les listes personnalisées utilisent leur propre format de catégories/liens.

## Précautions d'analyse { #analyst-cautions }

- Les descriptions des AppID proviennent d'une table de correspondance connue ; un AppID inconnu n'est pas suspect en soi.
- Les champs `LastModified` / de dernière utilisation appartiennent à une entrée DestList ; les heures de la cible incorporée sont différentes.
- Les nombres d'interactions n'existent que dans les versions prises en charge et ne représentent pas un nombre complet d'ouvertures.
- L'épinglage conserve des éléments au-delà de la récence normale ; un nettoyage ou les paramètres de l'application peuvent en supprimer d'autres.
- Une liste peut contenir des flux obsolètes/non rattachés ou des octets provenant d'une liste personnalisée antérieure plus longue.
- Les arguments et chemins des LNK incorporés sont contrôlés par l'attaquant. L'association à une application est un contexte, pas un gage de confiance.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

Avec `auto-parse`, les tables CSV sont `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`
et `FileAndFolderOpening/JumpLists-CustomDestinations.csv`, directement sous chaque ordinateur.
Le journal commun est `JumpLists-logs.txt` dans cette catégorie. Les manifestes listent les chemins exacts
dans `OutputFiles`. Les exports CSV autonomes conservent `AutomaticDestinations.csv` et
`CustomDestinations.csv` dans le répertoire que vous choisissez. Les exports automatiques JSON et JSONL sont
des fichiers uniques `JumpLists.json` ou `JumpLists.jsonl` ; la structure des enregistrements est inchangée.

## Champs CSV AutomaticDestinations { #automaticdestinations-csv-fields }

| Groupe de champs | Champs et interprétation |
|---|---|
| Source | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed` : le conteneur de la Jump List et ses horodatages du système de fichiers. |
| Application | `AppId`, `AppIdDescription` : AppID tiré du nom de fichier et correspondance avec une application connue. N'en déduisez pas l'identité à partir d'une empreinte inconnue. |
| Conteneur | `HasSps`, `DestListVersion`, `LastUsedEntryNumber` : structure/version du DestList et contexte de l'en-tête. |
| Ordre | `MRU`, `EntryNumber` : position d'utilisation la plus récente et identifiant hexadécimal de l'entrée/du flux. |
| Métadonnées d'utilisation | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition` : cycle de vie/utilisation/état d'épinglage du DestList. À interpréter selon la version. |
| Hôte/chemin | `Hostname`, `MacAddress`, `Path`, `DisplayPath` : emplacement de la cible et contexte de suivi. L'hôte/l'adresse MAC ne suffit pas à lui seul pour l'attribution. |
| Identifiants de suivi | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid` : permettent de corréler les cibles déplacées et le suivi des LNK incorporés. |
| Remarques | `Notes` : vous informe des flux non rattachés, des restes récupérés ou d'autres éléments de contexte du parseur. |
| LNK incorporé | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Args` : à interpréter comme décrit dans [Raccourcis LNK](lnk.md). |
| LNK supplémentaire | `NetworkPath`, `Name`, `IconLocation` : champs réseau et de présentation du raccourci incorporé. |
| Validation | `Problems` : flux manquants, troncature, désaccord entre les décomptes ou dommages des liens. |

## Champs CSV CustomDestinations { #customdestinations-csv-fields }

La sortie personnalisée commence par `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription` et `EntryName` (nom de catégorie ou `Tasks`), suivis de tous les champs LNK incorporés
listés ci-dessus, puis de `Notes`, `NetworkPath`, `Name`, `IconLocation` et `Problems`.

Les sorties JSON/JSONL conservent en outre l'en-tête DestList, le score/la somme de contrôle/le magasin de propriétés de chaque entrée,
`DestListPropertyStore`, les flux du fichier composé, les catégories personnalisées, les octets situés après la dernière catégorie,
ainsi que les champs imbriqués de chaque LNK incorporé.

## Stratégie d'investigation { #investigation-strategy }

1. Identifiez l'utilisateur et le profil source, puis faites correspondre l'AppID à une application avec prudence.
2. Triez les entrées automatiques par `LastModified` et `MRU` ; séparez les éléments épinglés.
3. Examinez les chemins, les partages réseau, les numéros de série des volumes amovibles, les références MFT et les arguments.
4. Inspectez `Notes` et les liens non rattachés/restants — des éléments supprimés ou antérieurs peuvent y survivre.
5. Corrélez avec RecentDocs, les LNK autonomes du dossier Recent, Prefetch, SRUM, `$MFT` et les journaux d'événements.

## Plus de détails { #further-details }

- [Microsoft : format Compound Binary File (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft : format Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [Projet de test/référence de parseur JumpList](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
