# Formats de sortie { #output-formats }

## Résumés dans le terminal { #terminal-summaries }

Sans `-o` ni `-t` structuré, les commandes affichent généralement des décomptes, des plages temporelles, des dommages et des constats
à forte valeur. Les résumés vous aident à comprendre la portée avant de créer une sortie volumineuse.

## CSV { #csv }

CSV est le format d'enregistrement par défaut. Il est pratique pour les tableurs, l'importation SQL et les visualiseurs
de chronologie. Certains parseurs créent plusieurs tables, car leurs artefacts contiennent différents types
d'enregistrements (Amcache, Jump Lists et SRUM).

!!! warning "Traitez les colonnes de preuves comme du texte"
    Un chemin ou une ligne de commande peut commencer par `=`, `+`, `-` ou `@`. Un tableur peut l'interpréter
    comme une formule. Importez tous les champs en tant que texte ou utilisez un visualiseur CSV sûr pour l'analyse forensique. Juntei préserve
    les chaînes de preuves plutôt que de les modifier.

## Noms de champs { #field-names }

Les en-têtes de colonnes CSV et les noms de champs JSON/JSONL utilisent le PascalCase, y compris pour Amcache et les
manifestes d'`auto-parse`. Les acronymes restent en majuscules : `ProcessID`, `TimestampUTC`, `UserSID`,
`SHA256`. Un mot qui suit un acronyme en est séparé par un trait d'union : `IP-Address`, `MFT-Entry`,
`USN-Journal`. Avec `jq`, mettez entre guillemets un champ comportant un trait d'union : `.["IP-Address"]`.

Le CSV MFT par défaut utilise des intitulés plus courts et lisibles tels que `EntryNum`, `Ext`, `Dir`, `HasADS`,
`Size`, `Created (SI)` et `Created (FN)`. Consultez la [référence des champs MFT](../artifacts/mft.md)
pour son schéma CSV complet. Les paires d'horodatages suivent l'ordre MACB : DataModified, Accessed,
MetadataChanged, Created, avec SI avant FN. Mettez entre guillemets les intitulés contenant des espaces ou de la ponctuation
dans les outils CSV.

Les valeurs des preuves conservent leur orthographe d'origine. Les clés de dictionnaire fournies par la source, telles que les noms de valeurs
du Registre sous `Details`/`DataRaw` et les noms `Zone.Identifier` sous `ZoneIdentifier`, restent également
inchangées. Les dictionnaires `ValueState` et `Raw` de SRUM utilisent les mêmes libellés que leurs colonnes de
données. Une collision de normalisation est signalée plutôt que de supprimer silencieusement un champ.

Cela modifie les schémas d'export précédents. Régénérez les résultats existants et mettez à jour les requêtes enregistrées
(par exemple, `.record_type` devient `.RecordType`, et `full_path` d'Amcache devient `FullPath`).
Les rapports texte lisibles par un humain ainsi que les formats d'entrée/de configuration conservent leurs conventions existantes.

## Noms de champs abrégés { #abbreviated-field-names }

Les libellés suivants s'appliquent aux exports CSV, JSON et JSONL, y compris l'analyse automatique et les
manifestes. Mettez à jour les filtres `jq`, `xsv` et de tableur enregistrés lorsque vous régénérez des résultats plus anciens.
Seuls les libellés de sortie changent ; les valeurs des preuves et les noms bruts des valeurs du Registre conservent leur orthographe d'origine.

| Ancien libellé | Libellé actuel |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

Les sorties CSV MFT utilisent les colonnes spécifiques `File` (Filename), `SN` (Sequence Number), `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number), `USN` (Update Sequence Number), `LogfileSN` (Logfile Sequence Number), `SID` (Security ID), `Type` (Name Type) et `RefCnt` (Reference Count). Cela concerne aussi les CSV MFT de `auto-parse` ; voir les [champs de sortie MFT](../artifacts/mft.md#csv-output-fields).

L'horodatage JSON natif des informations système est `CompNameLastWritten` (CSV :
`CompNameLastWrittenUTC`). `HasADS` est un indicateur booléen ; `Dir` dans Prefetch contient la
liste des répertoires enregistrée. `User` conserve son type propre à l'artefact : une chaîne pour un nom de compte,
ou la liste des utilisateurs associés dans la sortie USB. Il n'implique pas une identité d'exécution.
Mettez la ponctuation entre guillemets dans les requêtes : `."SI<FN"` et `.["ZeroU-Secs"]`.

## JSON { #json }

JSON stocke le résultat complet sous la forme d'un seul document. Il préserve les tableaux, les structures imbriquées, les valeurs null,
les grands entiers et des distinctions qu'un CSV plat ne peut pas toujours représenter.

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines { #json-lines }

JSONL écrit un objet JSON indépendant par ligne. Il se prête bien au traitement en flux et fonctionne naturellement avec `jq` :

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

Lorsque la sortie structurée est dirigée vers la sortie standard, Juntei envoie la progression et les messages sur la sortie
d'erreur standard, afin qu'un pipe ne reçoive que des données.

`auto-parse -j timeline.jsonl` crée un jeu de données JSONL supplémentaire pour une collecte multi-hôtes complète
tout en conservant les résultats CSV catégorisés. Ajoutez `-t json` ou `-t jsonl` pour modifier le format
des résultats individuels. Chaque enregistrement possède des champs `Comp` et `Artifact`, de sorte que sa source reste claire
après l'ingestion dans un SIEM.

## UTC et précision { #utc-and-precision }

Juntei formate les heures analysées en UTC selon la RFC 3339 lorsque le format le permet. Certains artefacts ont une
précision de 100 nanosecondes ; d'autres stockent des secondes, une heure FAT, des intervalles d'agrégation, ou aucune heure du tout.
Le nombre de chiffres n'est pas un gage d'exactitude : comprenez toujours le champ source décrit sur la page de son artefact.

L'UTC utilise la forme des journaux d'événements Windows `2026-10-20T08:46:38.2001668Z`, en conservant la précision
fractionnaire disponible. `auto-parse -Z "+09:00"` affiche les instants analysés avec un décalage fixe
explicite dans tous ses formats de résultats et dans son export `-j` facultatif. La valeur par défaut est l'UTC, quel que soit
l'ordinateur qui exécute Juntei. Les valeurs brutes et les heures murales sans fuseau restent telles qu'elles ont été enregistrées ; consultez
[fuseaux horaires de sortie](../commands/auto-parse.md#output-time-zone).

## Dommages et valeurs manquantes { #damage-and-missing-values }

Les valeurs vides, `null`, `NA`, `Problems` et les champs d'état propres aux parseurs sont significatifs. Ils peuvent signifier que
l'artefact n'a pas stocké de valeur, que l'enregistrement a été endommagé, que le format ne la prend pas en charge, ou que le
parseur n'a pas pu la résoudre. Ne remplacez pas toutes les valeurs vides par zéro.

## Chronologie normalisée { #normalized-timeline }

[`super-timeline`](../commands/super-timeline.md) crée un schéma d'événements CSV/JSONL commun, daté et relié aux sources, avec profil Timesketch.
La sortie n'est pas triée par défaut ; `--sort` active le tri chronologique et la consolidation des doublons. Les exports natifs et `auto-parse -j` conservent leurs schémas.
