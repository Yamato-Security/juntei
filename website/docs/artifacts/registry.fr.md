# Ruches du Registre Windows { #windows-registry-hives }

## Qu'est-ce qu'une ruche du Registre ? { #what-registry-hives-are }

Le Registre Windows stocke la configuration du système et des utilisateurs dans des fichiers de ruche binaires. Les clés forment une arborescence ;
les valeurs situées sous une clé contiennent des données typées. Les artefacts du Registre décrivent la persistance, les logiciels installés,
les services, les comptes, la configuration réseau, l'activité des utilisateurs, les paramètres de sécurité et presque tous les aspects
de l'état de Windows.

Le lecteur générique de Juntei décode n'importe quelle ruche principale, rejoue les journaux de transactions, signale les dommages et peut
récupérer les enregistrements supprimés/de cellules libres, de cell slack, rémanents, antérieurs à la relecture et présents uniquement dans les journaux de transactions.

<span class="evidence-strength">Meilleure conclusion :</span> la ruche contenait, ou conservait de manière récupérable,
l'état de la clé/valeur. L'heure de dernière écriture d'une clé montre que la clé a changé, et non quelle valeur a changé ni pourquoi.

## Ruches courantes et emplacements { #common-hives-and-locations }

| Ruche | Chemin habituel | Exemples de preuves |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | Services, pilotes, jeux de contrôle, nom d'hôte, périphériques montés, ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | Logiciels installés, TaskCache, profils, configuration de Windows |
| `SAM` | `C:\Windows\System32\config\SAM` | Comptes locaux et métadonnées des comptes |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | Stratégie de sécurité locale/métadonnées des secrets (selon les droits d'accès) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | Configuration du profil système par défaut |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | Préférences de l'utilisateur, clés Run, RecentDocs, activité des applications |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | Classes du shell, activité COM/du shell de l'utilisateur |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | Inventaire des applications (utilisez `parse-registry amcache` pour une sortie sémantique) |

Collectez les fichiers `.LOG`, `.LOG1` et `.LOG2` adjacents. Une ruche copiée pendant que Windows est en cours d'exécution peut être
dans un état « dirty » ; les journaux de transactions contiennent un état validé que le fichier principal seul ne reflète pas encore.

## Précautions d'analyse { #analyst-cautions }

- L'horodatage de dernière écriture d'une clé s'applique à la **clé**, et non à une valeur individuelle.
- Les valeurs du Registre n'ont généralement pas d'horodatage propre.
- `CurrentControlSet` est un alias utilisé par les analystes et Windows, résolu au moyen de `SYSTEM\Select` ; il ne s'agit pas
  d'une clé littéralement stockée.
- Les enregistrements supprimés peuvent être partiels, obsolètes, dupliqués entre plusieurs sources de récupération, ou écrasés.
- Les enregistrements présents uniquement dans les journaux de transactions montrent un état historique/intermédiaire. Expliquez `recovered_from`.
- Un résultat `absent_value` signifie que la clé interrogée existe mais ne possède pas cette valeur dans l'état analysé ;
  cela ne prouve pas que la valeur n'a jamais existé.
- Les chaînes, chemins et lignes de commande relèvent de la configuration, et ne prouvent pas que l'action configurée s'est exécutée.
- Une ruche « dirty » sans journaux applicables peut omettre un état ou exposer un état obsolète.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host` | Nom d'hôte déduit du contexte système disponible ; vide lorsque Juntei ne peut pas l'identifier de manière sûre. |
| `HiveType` | Catégorie de ruche détectée (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT`, etc.). À utiliser pour interpréter les chemins. |
| `SourcePath` | Ruche d'entrée exacte. Essentiel lorsque le mode répertoire combine plusieurs hôtes/utilisateurs. |
| `RecordType` | `key`, `value`, `default_value` ou `absent_value`. L'absence ne s'applique qu'à la requête/à l'état. |
| `KeyPath` | Chemin sous la racine de la ruche. `[unknown]` signale une ascendance supprimée non résolue. |
| `KeyLastWrittenUTC` | Dernière modification de la clé dans son ensemble. Ne permet pas d'identifier quelle valeur a changé. |
| `ValueName` | Valeur nommée ; vide pour une valeur par défaut/sans nom. |
| `ValueType` | Type du Registre (`REG_SZ`, `REG_DWORD`, `REG_BINARY`, etc.). Le type influe sur l'interprétation. |
| `ValueData` | Valeur restituée en fonction de son type. Les chaînes multiples restent des listes en JSON. |
| `DataSize` | Taille enregistrée par la valeur. À comparer avec les octets bruts et `Problems`. |
| `ValueDataRaw` | Octets exacts lorsque le décodage échoue ou que `--raw-data` est utilisé. À conserver pour la validation. |
| `ValueU64` | Représentation numérique entière des DWORD/QWORD, sans perte de précision liée à JavaScript. |
| `ValueFiletimeUTC` | Restitution facultative d'un QWORD en tant que FILETIME. N'a de sens que lorsque le schéma de la clé définit un FILETIME. |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only` ou `transaction_log_recovered`. Détermine le niveau de confiance. |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay`, ou une entrée de journal telle que `SYSTEM.LOG1#1234`. |
| `TransactionLogStatus` | `clean`, `replayed`, `unchanged` ou `not_applied` pour le contexte de cet enregistrement/de cette ruche. |
| `LogsApplied` | Noms des fichiers de journaux de transactions rejoués par Juntei. Consignez-les dans le rapport. |
| `Problems` | Dommages, cellules manquantes, incohérence de type, listes/parents non résolus ou autres limitations. |

## Interprétation de l'état de récupération { #recovery-status-interpretation }

| État | Signification |
|---|---|
| `deleted_complete` | Une clé/valeur supprimée a été récupérée avec un chemin/des données connus complets. Il s'agit toujours d'un état historique, et non actif. |
| `deleted_partial` | Une partie du chemin, de la liste de valeurs, de la valeur ou des données a été écrasée ou est illisible. Ne comblez pas les lacunes à partir de cellules réutilisées. |
| `residual_value_only` | La valeur a survécu sans contexte de clé suffisant ; `KeyPath` peut être absent. |
| `transaction_log_recovered` | L'état n'existe que dans les versions antérieures à la relecture/des journaux, et non dans les enregistrements alloués/libres actuels. |

## Stratégie d'investigation { #investigation-strategy }

1. Commencez par une clé sémantique exacte lorsqu'elle est connue ; utilisez la recherche par expression régulière/dans les données pour la découverte.
2. Vérifiez `TransactionLogStatus`, `AllocationStatus`, `RecoveredFrom` et `Problems` avant le contenu.
3. Interprétez les données selon le schéma documenté de la clé ; `REG_BINARY` ne se décrit pas lui-même.
4. Corrélez la configuration avec les preuves d'exécution (journaux d'événements, Prefetch, SRUM, services/tâches).
5. Conservez les données brutes pour les champs inédits ou contestés.

## Plus de détails { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Spécification du format du Registre de Maxim Suhanov](https://github.com/msuhanov/regf)
- [Parseur de Registre yarp](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Registry d'Eric Zimmerman](https://github.com/EricZimmerman/Registry)
- [Microsoft : types de valeurs du Registre](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
