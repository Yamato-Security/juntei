# Compatibilité des applications { #application-compatibility }

## Ce que Windows conserve { #what-windows-keeps }

Outre le ShimCache (`parse-registry shimcache`) et Amcache (`parse-registry amcache`),
les fonctionnalités de compatibilité des applications de Windows conservent les éléments suivants sous
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags` :

| Source | Ruche et clé | Ce qu'elle montre |
|---|---|---|
| `layers` | `SOFTWARE` (et `Wow6432Node`) et `NTUSER.DAT` `Layers` | Programmes configurés pour s'exécuter avec des couches de compatibilité : `RUNASADMIN` (toujours en tant qu'administrateur), `WIN7RTM`, `HIGHDPIAWARE`, … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 et versions ultérieures) | Programmes que l'Assistant Compatibilité des programmes a vu l'utilisateur exécuter |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | La même chose, sous Windows 7 |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | Programmes ayant chargé un module suivi, et date de leur dernier chargement |
| `cit_system` | `SOFTWARE` `CIT\System` (de Windows 7 aux premières versions de Windows 10) | Chemins de programmes et lignes de commande dans les bases de données CIT compressées |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | Bases de données de shims personnalisées installées avec `sdbinst` : fichier, description, heure d'installation |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | Bases de données de shims personnalisées appliquées à chaque programme, avec leurs heures d'installation |

<span class="evidence-strength">Meilleure conclusion :</span> les couches et les bases de données de shims sont
une configuration qui modifie la manière dont un programme s'exécute ; les entrées de l'Assistant Compatibilité des programmes et de CIT
montrent qu'un programme s'est exécuté (ou a chargé un module) sous cet utilisateur ou sur ce système.

Les bases de données de shims personnalisées constituent une technique de persistance et d'injection (MITRE ATT&CK T1546.011) : un shim
peut charger une DLL dans un programme ou en démarrer un autre chaque fois que le programme s'exécute.

## Emplacement { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

Collectez `SOFTWARE`, chaque `NTUSER.DAT` et leurs journaux de transactions, ainsi que les fichiers `.sdb` désignés par le chemin d'une base de données
de shims (généralement `C:\Windows\AppPatch\Custom\`).

## Précautions d'analyse { #analyst-cautions }

- **L'expansion de CIT est bornée.** Les valeurs déclarant plus de 64 Mio non compressés sont conservées
  sous forme d'octets bruts avec un diagnostic. Les compressions malformées et les incohérences de taille sont signalées.


- **Pas d'heure d'exécution pour les entrées de l'Assistant Compatibilité des programmes.** Les données `Store` (`SACP`, conservées dans
  `DataRaw`) comportent une heure, mais sous Windows 8.1, 10 et 11 elle est identique pour tous les programmes d'une
  ruche, ainsi que sur les autres systèmes : ce n'est pas l'heure d'exécution du programme. Seule la dernière écriture de la clé `Store`
  date la modification la plus récente.
- **Les programmes situés sur des supports amovibles** sont nommés `SIGN.MEDIA=<id> <path>` : `Path` est le chemin sur le
  support, et `Details` contient l'identifiant du support.
- **Les bases de données CIT** sont compressées en LZNT1 et leur structure n'est pas documentée publiquement : Juntei
  liste les chemins de programmes et lignes de commande trouvés dans chacune, une fois par base de données, sans nombres ni
  heures. Une ligne de commande constitue un enregistrement à part entière, non rattaché à son programme. Windows 10 et les versions ultérieures conservent
  peu d'éléments ici.
- **Bases de données de shims.** Vérifiez chaque enregistrement `custom_sdb` et `installed_sdb` : les logiciels légitimes
  en installent rarement. Le fichier `.sdb` lui-même indique ce que fait le shim.
- **Les couches** définies par un utilisateur ne s'appliquent qu'à cet utilisateur ; celles de `SOFTWARE` s'appliquent à tous. `RUNASADMIN` amène un
  programme à demander une élévation à chaque fois.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

La synthèse liste les bases de données de shims personnalisées, les programmes dotés de couches et les dernières
utilisations des modules CIT.

## Champs de sortie { #output-fields }

| Champ | Interprétation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Hôte, fichier de ruche, et `SOFTWARE` ou `NTUSER.DAT`. |
| `User`, `UserSID` | Nom du dossier de profil d'une ruche utilisateur, et son SID issu de `ProfileList` du système. |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb` ; `machine` ou `user`. |
| `Path` | Le programme (nom de la valeur), un chemin ou une ligne de commande dans une base de données CIT, ou le programme auquel s'applique une base de données de shims. |
| `Layers` | Les couches de compatibilité, telles qu'écrites (`~ RUNASADMIN`). |
| `LastSeenUTC` | Moment où un programme a chargé un module CIT pour la dernière fois. |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | GUID, fichier et heure d'installation d'une base de données de shims. |
| `Details` | Nom d'un module CIT, nom de valeur d'une base de données CIT, identifiant du support amovible d'un programme (`SignMedia`), `DatabaseDescription` et `DatabaseType` d'une base de données de shims. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | Provenance de l'enregistrement, date de dernière modification de cette clé, et indication de sa présence sous `Wow6432Node`. |
| `DataRaw` | Données `SACP` de la valeur `Store`, ou données qui ne sont pas conformes à ce qui est attendu (hexadécimal). |
| `TransactionLogStatus`, `LogsApplied` | Contexte de récupération du Registre. |
| `Problems` | Données qui n'ont pas pu être décodées ou lues. |

## Stratégie d'investigation { #investigation-strategy }

1. Examinez chaque base de données de shims personnalisée : son fichier `.sdb`, son heure d'installation et les programmes auxquels elle
   s'applique ; corrélez avec `sdbinst.exe` dans Prefetch, Amcache et les événements de création de processus.
2. Recherchez, parmi les entrées de l'Assistant Compatibilité des programmes, des programmes situés dans des dossiers accessibles en écriture par l'utilisateur,
   ainsi que des programmes provenant de supports amovibles.
3. Vérifiez `RUNASADMIN` et les autres couches appliquées à des programmes inhabituels.
4. Corrélez avec ShimCache, Amcache, BAM, Prefetch et UserAssist pour les heures d'exécution.

## Plus de détails { #further-details }

- [Microsoft : base de données de compatibilité des applications](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011 : Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft : MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
