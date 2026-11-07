# Liste des commandes { #command-list }

Les parseurs individuels sont regroupés par type de source. Exécutez `juntei --help`,
`juntei <SOURCE> --help` ou `juntei <SOURCE> <ARTIFACT> --help` pour les explorer :

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Pour une collecte de preuves Windows portant sur plusieurs ordinateurs, utilisez
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md). Elle invoque les mêmes commandes regroupées
et organise les résultats et les journaux par ordinateur et par catégorie d'investigation.

## [Images disque](mount.md) { #disk-images }

| Commande | Entrée | Guide détaillé |
|---|---|---|
| `mount` | Image disque VHDX (KAPE `--vhdx`, Hyper-V) | [Monter des images disque](mount.md) |

## [Commandes ESE](user-activity.md) { #ese-commands }

| Commande | Artefact d'entrée | Guide détaillé |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [Journalisation des accès utilisateur (UAL)](../artifacts/ual.md) |

## [Commandes de fichiers](execution.md) { #file-commands }

| Commande | Artefact d'entrée | Guide détaillé |
|---|---|---|
| `parse-file defender-quarantine` | Arborescence ou entrée de quarantaine Defender | [Quarantaine Defender](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [Raccourcis LNK](../artifacts/lnk.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [Commandes NTFS](filesystem.md) { #ntfs-commands }

| Commande | Artefact d'entrée | Guide détaillé |
|---|---|---|
| `parse-ntfs boot` | `$Boot`, ou une image de volume NTFS | [Secteur de démarrage ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`), ou une image de disque | [Index de répertoires ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | `$LogFile` NTFS | [Journal des transactions NTFS](../artifacts/logfile.md) |
| `parse-ntfs mft` | `$MFT` NTFS | [Table de fichiers principale](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [Descripteurs de sécurité ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [Journal USN](../artifacts/usn-journal.md) |

## [Commandes du Registre](security-registry.md) { #registry-commands }

| Commande | Artefact d'entrée | Guide détaillé |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [Réseaux](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [Secrets LSA](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (et `BCD-Template`) | [Configuration de démarrage (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [Compatibilité des applications](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [Stratégie d’audit](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage, RecentApps et éléments épinglés de la barre des tâches](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [Historique des applications](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [Listes MRU (éléments récemment utilisés)](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (avec `SAM`, `SOFTWARE` pour les noms d'utilisateur) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (`NTUSER.DAT` sous Windows XP) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [Programmes installés](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [Boîtes de dialogue Ouvrir et Enregistrer sous](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [Démarrages automatiques du Registre](../artifacts/persistence.md) |
| `parse-registry read` | Toute ruche principale du Registre | [Ruches du Registre](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (avec `SOFTWARE` pour les SID des utilisateurs) | [Accès à distance (côté utilisateur)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [Comptes locaux SAM](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [Paramètres de sécurité](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Services Windows](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` ou une valeur AppCompatCache exportée | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` d'un même système | [Informations système](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Cache du Planificateur de tâches](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`, avec `SOFTWARE` et `NTUSER.DAT` | [Périphériques USB](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## Types de sources et changements de format { #source-types-and-format-changes }

Les groupes décrivent les formats que Juntei prend actuellement en charge. `parse-registry amcache` lit
`Amcache.hve` ; cela n'implique pas la prise en charge des bases de données d'inventaire Amcache SQLite. Les crates des parseurs
d'artefacts restent séparées afin que la prise en charge d'un autre format de stockage puisse être ajoutée indépendamment.

Les anciens noms de parseurs à plat ont été remplacés par les chemins source-et-artefact ci-dessus. Conservez les
options existantes des parseurs après le nom de l'artefact. `auto-parse` et les sous-commandes de Registre existantes
conservent leurs noms.

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` effectue sa chasse à travers plusieurs formats de source ; c'est donc une commande d'analyse
de premier niveau. Elle complète l'inventaire complet de la configuration produit par `parse-registry persistence` avec
131 vérifications, une classification autorisé/bloqué, les métadonnées des charges utiles, la couverture et la corrélation des horodatages.

## Utilisation de premier niveau { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Les pages des sources reproduisent l'utilisation des commandes et donnent de courts exemples. Les guides des artefacts expliquent
les preuves, les champs de sortie, les limites et la stratégie d'investigation.
