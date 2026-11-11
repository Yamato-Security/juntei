# Amcache { #amcache }

## Qu'est-ce qu'Amcache { #what-amcache-is }

Amcache est un inventaire de compatibilité des applications de Windows, stocké dans une ruche du Registre. Les systèmes récents
enregistrent les fichiers exécutables sous `InventoryApplicationFile` et les périphériques Plug-and-Play sous
`InventoryDevicePnp` ; les versions plus anciennes de Windows utilisent une structure héritée `Root\File`.

Pour l'investigation des exécutables, Amcache est précieux car il peut conserver un chemin complet, le SHA-1,
la taille du fichier, les métadonnées PE, les informations de version, le texte de l'éditeur, la date d'édition des liens et les horodatages du Registre
après la disparition du fichier d'origine. Les enregistrements de périphériques peuvent identifier le matériel connecté, les pilotes,
les dates d'installation et les identifiants.

<span class="evidence-strength">Meilleure conclusion :</span> Windows a inventorié le fichier ou le
périphérique enregistré. Un enregistrement d'exécutable est une **preuve de présence, et non une preuve d'exécution**.

## Emplacement et collecte { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

Collectez la ruche et les fichiers voisins `Amcache.hve.LOG`, `.LOG1` et `.LOG2`. Si la ruche est marquée comme non synchronisée, `parse-registry amcache` rejoue automatiquement les journaux de transactions utilisables en mémoire avant analyse, sans modifier les preuves. Les détails figurent dans le journal du parseur ou le terminal ; si les enregistrements sont envoyés sur stdout, les diagnostics vont sur stderr. Des journaux absents ou inutilisables peuvent laisser des données incomplètes ou rendre une ruche incohérente illisible.

## Précautions d'analyse { #analyst-cautions }

- Une entrée d'exécutable Amcache ne prouve pas qu'un utilisateur a lancé le fichier.
- `Publisher` est une métadonnée de ressource PE/de version enregistrée par Windows. Elle ne prouve pas que le fichier disposait
  d'une signature Authenticode valide au moment de la collecte.
- Les empreintes peuvent être absentes, et les schémas hérité/récent n'exposent pas des valeurs identiques.
- `LinkDate` provient de l'en-tête PE et peut être falsifié, copié ou produit par des compilations reproductibles.
- L'heure de dernière écriture d'une clé décrit la mise à jour de l'enregistrement du Registre, pas nécessairement le téléchargement, la création,
  la première exécution ou la dernière exécution.
- Les chemins peuvent faire référence à des volumes amovibles ou qui n'existent plus.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

La sortie CSV contient les tables de synthèse, de constats, de fichiers et de périphériques. Avec `auto-parse`, il s'agit de
`Registry-Amcache-Summary.csv`, `Registry-Amcache-Findings.csv`, `Registry-Amcache-Files.csv` et
`Registry-Amcache-Devices.csv`, placés directement dans le répertoire `FileExistence/` de chaque ordinateur, avec
`Registry-Amcache-logs.txt` à côté des tables. Plusieurs ruches ajoutent `-Source1`, `-Source2`, etc.
avant l'extension ; `OutputFiles` dans les manifestes indique les chemins exacts. Les exportations CSV autonomes
conservent leurs noms de tables natifs et l'emplacement de sortie demandé. Les enregistrements JSONL utilisent
les valeurs de `RecordType` `summary`, `finding`, `file` et `device`.

## Champs de sortie des fichiers { #file-output-fields }

| Champ | Utilisation par l'analyste |
|---|---|
| `KeyName`, `SourceKeyPath` | Identifient l'enregistrement exact du Registre source et la branche du schéma. À conserver pour la reproductibilité. |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | Heure de mise à jour de l'enregistrement du Registre, sous forme lisible et brute. Ne la qualifiez pas d'heure d'exécution. |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | Heure de la clé de volume parente dans les schémas hérités ; élément de contexte, et non heure d'une action sur le fichier. |
| `FileID` | Identifiant de fichier enregistré par Amcache. Conservé tel quel ; ne supposez pas qu'il s'agit toujours d'un SHA-1. |
| `SHA1`, `SHA256` | Pivots solides vers Defender, le renseignement sur les menaces, l'EDR et les référentiels de fichiers. Une valeur nulle signifie non enregistré. |
| `Name`, `OriginalFileName`, `Description` | Noms PE/de version enregistrés. Comparez-les au chemin réel pour détecter une usurpation (masquerading) ; ces métadonnées sont contrôlées par l'attaquant. |
| `FullPath` | Pivot d'emplacement le plus utile. Normalisez la casse avec prudence, mais conservez la chaîne de preuve d'origine. |
| `LongPathHash` | Identifiant de Windows lié au chemin ; utile pour faire correspondre des enregistrements, ce n'est pas une empreinte du contenu. |
| `ProgramID` | Relie le fichier à un enregistrement d'inventaire d'application lorsqu'il existe. |
| `Publisher` | Texte de l'éditeur enregistré. À traiter comme une métadonnée, et non comme une validation de signature. |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | Contexte des ressources de version. Des différences ou des valeurs invraisemblables peuvent étayer une analyse d'usurpation. |
| `BinaryType` | Contexte d'architecture/de type enregistré ; à comparer avec l'analyse PE lorsque le fichier est récupéré. |
| `LinkDate` | Horodatage de compilation/d'édition des liens PE tel qu'enregistré. Utile uniquement avec scepticisme et corroboration. |
| `Size` | Taille enregistrée ; à comparer avec `$MFT`, le contenu Defender ou un fichier récupéré. |
| `Language` | Identifiant de langue PE ; indice contextuel faible, et non élément d'attribution. |
| `IsPE-File`, `IsOS-Component` | Indicateurs de classification enregistrés par Windows. Une valeur fausse ne prouve pas l'innocuité. |
| `USN` | Valeur de séquence de mise à jour NTFS enregistrée par Amcache ; peut aider à pivoter vers les enregistrements USN subsistants. |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | Associent des applications/composants empaquetés au fichier. |
| `RegistryValues` | Chaque valeur de Registre d'origine avec son type, sa valeur décodée et sa forme hexadécimale brute. À utiliser lorsqu'un champ typé est absent ou pour des champs futurs/hérités. |

## Champs de sortie des périphériques { #device-output-fields }

| Groupe de champs | Champs et interprétation |
|---|---|
| Source | `KeyName`, `SourceKeyPath`, `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` : enregistrement exact du périphérique et heure de mise à jour du Registre. |
| Identité | `Description`, `BusDescription`, `Manufacturer`, `Model` : descriptions du périphérique destinées à l'utilisateur ; les chaînes du fabricant peuvent être génériques. |
| Identifiants matériels | `HWID`, `CompID`, `Class`, `ClassGUID`, `ContainerID`, `DeviceInterfaceClasses` : pivots les plus solides vers la famille de périphériques et l'instance. |
| État | `DeviceState`, `ProblemCode`, `ParentID`, `MatchingID`, `Enumerator` : contexte d'énumération et de relations dans Windows. |
| Pilote | `DriverID`, `DriverName`, `DriverPackageStrongName`, `DriverVerDate`, `DriverVerVersion`, `Provider`, `Service`, `INF` : associations au package de pilote et au service ; à corréler avec les fichiers et services installés. |
| Installation | `InstallDate`, `FirstInstallDate`, `InstallState`, `ExtendedInfs` : cycle de vie d'installation enregistré. L'absence de date ne prouve pas que le périphérique n'a jamais été connecté. |
| Filtres et pile | `LowerClassFilters`, `UpperClassFilters`, `LowerFilters`, `UpperFilters`, `StackID`, `LocationPaths` : contexte avancé de la pile de pilotes ; les pilotes de filtre inattendus méritent un examen. |
| Conservation brute | `RegistryValues` : valeurs source complètes, typées/brutes, pour la validation et les champs non pris en charge. |

Les colonnes de tableaux et de valeurs brutes en CSV utilisent un suffixe `JSON` (par exemple `HWID-JSON` et
`RegistryValuesJSON`) ; leurs cellules contiennent du JSON, tandis que JSON/JSONL conserve les tableaux natifs.

## Stratégie d'investigation { #investigation-strategy }

1. Pivotez sur `FullPath`, `SHA1`, `Name` et `USN`.
2. Recherchez une entrée Prefetch correspondante pour étayer l'exécution.
3. Utilisez `$MFT` et le journal USN pour établir la création, le renommage, la suppression et la taille du fichier.
4. Comparez les métadonnées d'éditeur/de version avec un PE récupéré indépendamment et une validation de signature.
5. Vérifiez la quarantaine Defender et les journaux d'événements pour une détection ou une suppression.

## Plus de détails { #further-details }

- [Schéma de sortie de ys-amcache-parser utilisé par Juntei](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [AmcacheParser d'Eric Zimmerman](https://github.com/EricZimmerman/AmcacheParser)
- [amcache-forensic de Security Ronin](https://github.com/SecurityRonin/amcache-forensic)
- [Plugin Amcache de Regipy](https://github.com/mkorman90/regipy)
