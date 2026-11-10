# Journal des transactions NTFS (`$LogFile`) { #ntfs-transaction-log-logfile }

## De quoi s'agit-il et pourquoi est-ce important { #what-it-is-and-why-it-matters }

NTFS est le système de fichiers que Windows utilise habituellement pour son volume système. Afin de se rétablir après un plantage, il
enregistre les modifications qu'il s'apprête à apporter aux métadonnées du système de fichiers dans un fichier de volume masqué nommé
`$LogFile`. Chaque mise à jour peut contenir des octets **redo** (le nouvel état) et des octets **undo** (l'état précédent
nécessaire pour annuler une modification). Ce journal est distinct des journaux d'événements Windows, des fichiers `.LOG1` du Registre
et du [journal USN](usn-journal.md).

Pour un enquêteur, l'état précédent est particulièrement utile. Un enregistrement undo peut conserver un
nom de fichier antérieur à un renommage, une entrée de répertoire supprimée ou des horodatages antérieurs à une mise à jour des métadonnées.
Certains enregistrements contiennent également des octets que NTFS a écrits dans le journal USN, y compris des enregistrements qui ne figurent plus dans le
`$J` collecté. De nombreuses transactions concernent toutefois l'activité courante du système d'exploitation et ne
décrivent pas une attaque.

**Exemple :** une entrée d'index supprimée mentionne `report.txt` ; une entrée ajoutée ultérieurement mentionne
`report.txt.encrypted`. La mise en correspondance de la référence de fichier complète, du parent, de la chaîne de transactions et d'autres
artefacts peut étayer un renommage. Ces deux noms seuls n'identifient ni un rançongiciel, ni son opérateur,
ni l'heure exacte du chiffrement.

## Emplacement et collecte { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

Chaque volume NTFS possède son propre journal. Acquérez-le avec un outil qui lit les métadonnées NTFS ou qui l'exporte
depuis une image ou un instantané forensique ; une copie de fichier Windows ordinaire peut ne pas y avoir accès. Collectez la
`$MFT` et le `$Extend\$UsnJrnl:$J` du même volume à un moment aussi proche que possible. Conservez les fichiers
d'origine et leurs empreintes. Une acquisition ultérieure peut contenir une autre période d'activité.

`$LogFile` est un **journal circulaire** : les nouveaux enregistrements écrasent les anciens. La rétention dépend de la taille du journal
et de la charge de travail, et peut se limiter à quelques minutes ou heures. L'absence d'un nom de fichier ou d'une opération ne
prouve pas qu'elle n'a jamais eu lieu.

## Ce que prouve un enregistrement, et précautions { #what-a-record-proves-and-cautions }

- Une mise à jour enregistre des métadonnées que NTFS a journalisées en vue d'une récupération. Une seule entrée redo ne prouve pas
  qu'une action complète de plus haut niveau s'est achevée ni qu'une personne donnée l'a initiée. Corrélez
  les enregistrements liés, l'achèvement de la transaction, le système de fichiers résultant et des preuves indépendantes.
- **Il n'existe pas d'horodatage général des transactions.** Les horodatages intégrés décrivent des métadonnées de fichier,
  parfois obsolètes, copiées ou modifiées par l'attaquant. Ne placez pas `CreatedOn` sur une chronologie d'incident
  comme heure d'un renommage ou d'une modification d'horodatage. Les enregistrements USN peuvent fournir un horodatage distinct.
- Les LSN (numéros de séquence du journal) ordonnent les enregistrements **au sein du journal d'un seul volume**. Ce ne sont pas des heures
  d'horloge, et ils ne doivent pas être comparés entre hôtes ou entre volumes. Des enregistrements issus de passages antérieurs du tampon circulaire
  peuvent subsister à côté d'une activité plus récente.
- Un emplacement de la table des transactions est réutilisé. Regroupez par `TransactionFirstLsn`, `TransactionId`, identité
  du client et source, plutôt que par `TransactionId` seul. Les chaînes peuvent pointer en dehors des données conservées.
- Les références de fichier contiennent un numéro d'entrée **et un numéro de séquence**. Sans cela, la réutilisation d'une entrée MFT peut
  faire apparaître des fichiers sans rapport comme un même fichier.
- Les chemins fournis par `$MFT` décrivent **l'état de la MFT collectée**, et non un chemin historique
  reconstitué. Juntei fait correspondre les numéros de séquence et laisse explicites les composants inconnus. Un parent peut
  avoir été renommé après la transaction, et les liens physiques peuvent proposer plus d'un nom actuel.
- `TargetInferred` signifie qu'une correspondance d'attribut ouvert provient d'une autre portion du journal.
  Traitez cette correspondance avec une prudence accrue. Les entrées d'index non résolues peuvent être décodées d'après leur seule
  disposition ; validez leurs noms avec les octets bruts et d'autres sources.
- Une page déchirée (torn page) est ignorée ; les enregistrements lisibles environnants restent disponibles. `Truncated` signale un
  enregistrement dont la suite est manquante. Examinez les diagnostics du parseur avant de considérer la sortie comme
  complète. Une entrée de longueur nulle est une erreur ; un journal jamais écrit, entièrement à `FF`, ne produit légitimement aucune ligne.
- Le parseur ne **rejoue pas** les transactions pour reconstituer une MFT historique. Il ne
  récupère pas de contenu de fichier non résident arbitraire et n'exécute aucune donnée collectée. Les pages protégées par CRC DAX
  ne sont pas prises en charge. La validation sur des preuves réelles couvre principalement les pages de journal de 4 Kio.

## Analyser avec Juntei { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

Consultez la [référence des commandes](../commands/filesystem.md#parse-ntfs-logfile) pour toutes les options. La découverte
dans un répertoire accepte `$LogFile`, `LogFile`, leurs formes `.bin` et des noms tels que `C_$LogFile` ou
`C_$LogFile.bin`, sans tenir compte de la casse. Un fichier unique passé avec `-f` peut porter n'importe quel nom.

L'association automatique avec la MFT respecte le préfixe de volume d'une exportation : `C_$LogFile` utilise `C_$MFT`, et
`D_$LogFile.bin` essaie d'abord `D_$MFT.bin`, puis `D_$MFT`. Les journaux préfixés n'utilisent jamais une `$MFT` simple
comme solution de repli. Un `LogFile` sans préfixe accepte également `MFT` ou `$MFT`. Utilisez `-m` avec `-f` pour sélectionner
explicitement une MFT correspondante lorsque votre exportation suit un autre schéma de nommage.

La sortie comporte un enregistrement par LSN, y compris les enregistrements de mise à jour, de point de contrôle et de client inconnu. JSON est un
flux d'objets mis en forme ; JSONL comporte un objet par ligne. CSV utilise les mêmes noms de champs ;
les tableaux et champs structurés sont du JSON compact dans une cellule CSV. Les scalaires inconnus sont vides en
CSV et `null` en JSON. Un tableau vide signifie qu'aucun élément n'a été décodé, et non la preuve qu'aucun n'existait.

Un journal entier et ses enregistrements analysés sont conservés en mémoire à un instant donné. La résolution des chemins MFT ajoute
de la mémoire et du travail. `--raw-data` peut considérablement augmenter la taille de la sortie. Dans `auto-parse`, utilisez `-T` pour
limiter le nombre de processus de parseur simultanés sur une machine disposant de peu de mémoire.

Si une MFT adjacente ne peut pas être lue, Juntei émet un avertissement et exporte le journal sans chemins ; `MftSource`
reste alors nul. Cet échec d'enrichissement n'entraîne pas l'abandon des preuves principales lisibles.
Les correspondances MFT manquantes ou ambiguës produisent également un avertissement et un compteur `Logs without a matching MFT`
dans la synthèse ; elles ne sont pas comptées comme des échecs d'analyse.

## Champs de sortie { #output-fields }

### Identité, intégrité et provenance de l'enregistrement { #record-identity-integrity-and-provenance }

| Champ | Signification et interprétation |
|---|---|
| `SourceFile`, `FileOffset` | Chemin d'entrée et décalage en octets où l'en-tête de cet enregistrement a été lu ; utilisez-les pour valider les octets. |
| `LSN` | Numéro de séquence du journal de l'enregistrement. Les grandes valeurs sont des entiers, et non des horodatages. |
| `PreviousLSN`, `UndoNextLSN` | Liens vers les enregistrements précédents et d'annulation. Zéro signifie absence de lien ; des liens non nuls peuvent pointer en dehors des données conservées. |
| `TransactionID`, `TransactionFirstLSN` | Emplacement de la table des transactions et LSN le plus ancien lié à cette transaction ; l'emplacement seul n'est pas unique. |
| `RecordType` | `ClientRecord` (mise à jour), `ClientRestart` (point de contrôle) ou `Other`. |
| `ClientIndex`, `ClientSequence` | Identité du client du journal ; normalement NTFS. Il ne s'agit ni d'un compte utilisateur ni d'un ID de processus. |
| `Flags`, `SpansPages` | Indicateurs bruts de l'enregistrement et indication que les données client se poursuivent sur une autre page. Le chevauchement de pages est normal. |
| `Truncated` | Vrai si le parseur a constaté une suite manquante pour ce LSN. Ne résume pas l'ensemble des dommages du fichier entier. |
| `ClusterSize`, `ClusterSizeSource` | Octets par cluster utilisés pour les décalages de la cible ; la source est `Caller`, `Checkpoint` ou `Inferred`. Fournissez la valeur réelle avec `--cluster-size` si elle est connue. |
| `LogVersion` | Version issue de la dernière page de redémarrage lisible, généralement `1.1` ou `2.0` ; nulle si aucune page de redémarrage lisible ne subsiste. |
| `MFT-Source` | MFT utilisée pour l'enrichissement avec les chemins actuels, le cas échéant. Elle doit appartenir au même volume. |

Dans la sortie JSON/JSONL de `auto-parse`, `Comp` et `Artifact` sont ajoutés comme pour les autres parseurs. Le journal
`.txt` de la tâche contient les diagnostics ; les manifestes indiquent si la tâche a réussi, échoué ou été ignorée.
Des dommages récupérables peuvent produire une tâche réussie avec un compteur `Parse problems` non nul dans son journal.

### Opérations et cible { #operations-and-target }

| Champ | Signification et interprétation |
|---|---|
| `RedoOperation`, `UndoOperation` | Noms des opérations NTFS, ou valeur hexadécimale pour un code inconnu. Redo décrit la mise à jour prévue ; undo décrit son inverse. |
| `TargetEntryNumber`, `TargetSequenceNumber` | Pour une opération sur un enregistrement de fichier : le numéro d'enregistrement MFT concerné, sans numéro de séquence déduit. Sinon : la référence du fichier propriétaire de l'attribut ouvert résolu. Une mise à jour d'index de répertoire en ligne cible le répertoire, et non l'enfant décrit par son entrée. |
| `TargetAttributeType`, `TargetAttributeName` | Type numérique et nom de l'attribut ouvert utilisé par l'opération. Pour les opérations sur un enregistrement de fichier, il s'agit normalement du `$DATA` de `$MFT`. Types courants : 16 (`$STANDARD_INFORMATION`), 48 (`$FILE_NAME`), 128 (`$DATA`), 144 (`$INDEX_ROOT`), 160 (`$INDEX_ALLOCATION`), 176 (`$BITMAP`). |
| `TargetPath` | Chemin MFT actuel lorsque la référence complète de la cible est connue, ou composants inconnus. Vide pour les opérations sur un enregistrement de fichier dont la génération historique est inconnue ; un numéro d'enregistrement seul ne doit pas être résolu vers son occupant actuel. |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | Fichier propriétaire de l'attribut ouvert par lequel l'opération écrit, avec son chemin MFT actuel. Pour les opérations sur un enregistrement de fichier, ces champs conservent le contexte `$MFT` séparément de l'enregistrement concerné. |
| `TargetAttribute` | Décalage dans la table des attributs ouverts du journal ; ce n'est ni le type de l'attribut ni son identifiant MFT. |
| `TargetInferred` | Vrai si la correspondance de la cible provient d'une autre portion du journal. Une cible nulle reste non résolue même si cet indicateur est faux. |
| `TargetVCN`, `Lcns` | Cluster virtuel dans la cible et numéros de clusters physiques ; utiles pour la validation de bas niveau plutôt que pour la chronologie. |
| `RecordOffset`, `AttributeOffset` | Décalages de l'attribut/de l'entrée et de la mise à jour en son sein. |
| `ClusterBlockOffset`, `TargetBlockSize` | Décalage et taille de la cible en unités de 512 octets. |
| `OffsetInTarget` | Décalage en octets calculé dans l'enregistrement de fichier ou le flux non résident concerné. Dépend des tailles de cluster et d'enregistrement MFT. |
| `RedoLength`, `UndoLength` | Longueurs de données déclarées. Une opération de mise à zéro peut déclarer une longueur sans transporter autant d'octets de charge utile. |
| `RedoBytesAvailable`, `UndoBytesAvailable` | Octets de charge utile réellement disponibles pour le décodage. À comparer avec les longueurs déclarées et la sémantique de l'opération. |

### Colonnes pratiques de métadonnées de fichier { #convenient-file-metadata-columns }

Ces colonnes affichent un nom de fichier non DOS privilégié issu du côté redo, à défaut du côté undo.
Si aucun des deux côtés ne porte de nom de fichier, les champs de mise à jour d'horodatage privilégient les horodatages redo, puis undo.
Les mises à jour d'entrées de répertoire exposent également directement leurs heures, taille et indicateurs, en privilégiant redo sur
undo. Les attributs `$DATA` décodés exposent leur taille même en l'absence de nom de fichier. La taille du `$DATA` sans nom d'un
enregistrement de fichier décodé a priorité sur la taille, possiblement obsolète, de son nom de fichier.
Ces colonnes facilitent la consultation ; utilisez les champs de chaque côté pour une comparaison avant/après complète.

| Champ | Signification et interprétation |
|---|---|
| `MetadataSide` | `Redo` ou `Undo`, identifiant le côté qui fournit les colonnes pratiques de nom de fichier/d'horodatage. |
| `EntryNumber`, `SequenceNumber` | Fichier décrit par les métadonnées décodées de nom de fichier/d'enregistrement de fichier/d'attribut ou par une mise à jour partielle des horodatages `$STANDARD_INFORMATION`, avec la séquence lorsqu'elle est connue. Vides pour une mise à jour d'entrée de répertoire qui n'identifie pas l'enfant, et pour les mises à jour brutes de taille/de liste d'extensions (run list). Utilisez `TargetEntryNumber` pour l'enregistrement concerné. |
| `Filename` | Nom décodé privilégié ; tous les noms restent dans les tableaux de chaque côté, y compris les alias DOS et les liens physiques. |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | Référence du parent et chemin MFT actuel ; il ne s'agit pas d'une reconstitution du chemin historique. |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | Heures des métadonnées en UTC, à 100 ns près. Ce ne sont pas des horodatages de transaction. Les heures manquantes/non valides restent vides/nulles. |
| `FileSize`, `FileAttributes` | Taille et indicateurs Windows numériques issus des métadonnées sélectionnées ; les valeurs des entrées de répertoire peuvent être obsolètes. Les attributs `$DATA` et les enregistrements de fichier décodés peuvent fournir une taille plus directe. |
| `AttrType` | Type numérique d'un enregistrement d'attribut décodé ; 48 pour une mise à jour décodée des métadonnées d'une entrée de répertoire. Distinct de l'attribut ouvert ciblé par l'opération. |

### Métadonnées et charges utiles redo/undo complètes { #complete-redoundo-metadata-and-payloads }

| Champ | Signification et interprétation |
|---|---|
| `RedoFileNames`, `UndoFileNames` | Tableaux de tous les `$FILE_NAME` décodés de ce côté, distinguant les noms ajoutés/nouveaux des noms supprimés/anciens. |
| `RedoTimes`, `UndoTimes` | Horodatages `$STANDARD_INFORMATION` écrits par chaque côté ; les mises à jour partielles laissent absents les horodatages inchangés au lieu de les deviner. |
| `RedoDecoded`, `UndoDecoded` | Informations structurées sur l'enregistrement de fichier, l'attribut, l'entrée d'index ou l'entrée de répertoire, identifiées par `Kind`. Une valeur nulle peut signifier une opération non prise en charge ou des octets insuffisants. |
| `JournalWriteOffset`, `JournalWriteLength` | Décalage et nombre d'octets des données redo écrivant dans `$DATA:$J`. Identifie une source possible d'enregistrements USN autrement perdus ; n'exporte pas de journal reconstitué. |
| `Checkpoint` | Version du point de contrôle, LSN de début, taille de cluster, et LSN/longueur des vidages de la table des attributs ouverts, de la table des noms d'attributs, de la table des pages modifiées (dirty pages) et de la table des transactions. |
| `OtherRecordType` | Type brut d'enregistrement client pour un corps non reconnu ou indécodable. |
| `RedoData`, `UndoData`, `OtherData` | Charges utiles hexadécimales, uniquement avec `--raw-data` ; sinon nulles/vides. Conservez-les pour valider des structures non prises en charge ou endommagées. |

Chaque objet de nom de fichier comprend les références du fichier et du parent (`Raw`, `EntryNumber`,
`SequenceNumber`), `Name`, `NameEscaped`, `Namespace`, le `ParentPath` actuel, quatre horodatages,
`Size`, `AllocatedSize`, `Attr` et `ReparseValue`. `NameEscaped` signale de l'UTF-16 non valide
représenté au moyen de séquences d'échappement. Une valeur de reparse peut être une balise de point d'analyse (reparse tag) ou une taille d'attributs étendus.

Les objets d'horodatage conservent à la fois `Utc` et l'entier `Filetime` ; une valeur sentinelle ou non valide peut
avoir une représentation UTC nulle tout en conservant son entier d'origine. `RedoTimes` et `UndoTimes`
contiennent `Created`, `Modified`, `Changed` et `Accessed` ; les valeurs des membres manquants sont nulles.

Les objets `FileRecord` décodés comprennent le numéro, la séquence, le nombre de liens physiques, les indicateurs, la référence de base,
l'indication que seul l'en-tête subsiste, et les attributs présents. Chaque attribut décodé conserve
le type, le nom, l'état résident, les indicateurs, l'ID, la taille et la taille allouée. `$STANDARD_INFORMATION` ajoute les
quatre heures, les attributs, les ID de propriétaire/de sécurité, le quota et l'USN ; ces ID nécessitent une résolution distincte et
ne prouvent pas quel utilisateur a modifié le fichier. Les entrées d'index comprennent les indicateurs, la longueur de clé et un
nom de fichier facultatif. Les mises à jour d'entrées de répertoire conservent les quatre heures, les tailles, les attributs et la valeur
de reparse, même lorsque le nom lui-même n'a pas été journalisé.

## Méthode d'investigation { #investigation-workflow }

1. Examinez les diagnostics à la recherche de pages de redémarrage manquantes, de pages déchirées et d'enregistrements tronqués.
2. Recherchez un nom de fichier ou une extension suspects à la fois dans `RedoFileNames` et dans `UndoFileNames`.
3. Faites correspondre les numéros d'entrée **et de séquence**, puis examinez la chaîne de LSN et les opérations de la transaction.
4. Comparez les horodatages redo et undo pour repérer des modifications. La copie, l'extraction et la restauration courantes peuvent
   aussi modifier les horodatages ; une différence est une piste, et non un verdict automatique de timestomping.
5. Corrélez avec le [journal USN](usn-journal.md), la [$MFT](mft.md) et [$I30](i30.md). Utilisez les journaux
   d'événements, [Prefetch](prefetch.md) et les artefacts d'activité utilisateur pour établir l'exécution ou l'attribution.

Par exemple, pour trouver tous les enregistrements portant un nom de fichier chiffré d'un côté ou de l'autre :

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

Il s'agit d'une recherche par nom de fichier, et non d'un détecteur complet de rançongiciels. Pour examiner les charges utiles du journal,
exportez avec `--raw-data` et sélectionnez les enregistrements dont `JournalWriteOffset` n'est pas nul ; la valeur hexadécimale de `RedoData`
représente les octets écrits à ce décalage. L'analyse et la déduplication de ces enregistrements USN intégrés
constituent une étape distincte. Juntei ne les fusionne pas encore avec la sortie de `parse-ntfs usn-journal`.

## Plus de détails { #further-details }

- [Crate logfile de ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile) : structures prises en charge, corpus de validation et limitations connues.
- [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) : implémentation Rust en amont.
- [Documentation du format NTFS de libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation) : pages de journal, corrections (fixups) et structures de métadonnées.
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) : recherche et implémentation indépendantes d'analyse NTFS.
- [LogFileParser de Joakim Schicht](https://github.com/jschicht/LogFileParser) : analyse détaillée du journal des transactions.
- [ntfs-linker de Stroz Friedberg](https://github.com/strozfriedberg/ntfs-linker) : corrélation des sources NTFS en un historique des fichiers.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) : échantillons NTFS Windows réels.
- [Velocidex go-ntfs](https://github.com/Velocidex/go-ntfs) : implémentation NTFS et volume de test public.
