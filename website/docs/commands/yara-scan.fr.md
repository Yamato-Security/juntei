# Analyse YARA { #yara-scan }

Analyser les fichiers collectés avec un moteur Rust portable. Windows, macOS et Linux sont pris en charge. Les fichiers sont lus comme des données et ne sont jamais exécutés. Cette commande est indépendante de l’analyse automatique des artefacts.

## Choisir les règles { #choose-rules }

Core est le choix par défaut et privilégie la précision et les performances. Extended élargit la recherche. Full couvre le plus grand périmètre, avec davantage de faux positifs possibles.

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
```

`--rules <FILE_OR_DIRECTORY>`: Utiliser un fichier ou répertoire local de règles UTF-8, sans téléchargement. Les répertoires sont parcourus récursivement. Les inclusions restent dans le répertoire choisi et sont développées une seule fois dans l’ordre des dépendances. Toute règle invalide ou non prise en charge arrête la compilation au lieu d’être ignorée.

## Limites et périmètre { #limits-and-scope }

Tous les fichiers ordinaires sont admissibles, y compris les contenus restaurés depuis la quarantaine. Les liens symboliques et fichiers spéciaux sont ignorés. Les archives et images disque ne sont pas décompressées ; la mémoire vive des processus n’est pas analysée. Utiliser des fichiers extraits stables et un nouveau répertoire de sortie extérieur aux preuves.

- `-T, --threads <N>`: Analyses parallèles ; par défaut le nombre de processeurs, limité à 8.
- `--timeout <SECONDS>`: Délai maximal d’évaluation par fichier, en secondes ; défaut : 60.
- `--max-file-mib <MIB>`: Taille maximale en MiB ; défaut : 64. Les fichiers plus gros sont signalés comme ignorés. Plus de tâches et des tampons plus grands augmentent la mémoire utilisée.
- `--rules-release YYYYMMDD`: Date de publication Forge ou latest par défaut. Les sources téléchargées et leurs empreintes sont conservées pour reproduire l’analyse.

## Résultats { #results }

- `YaraDetections.csv` / `YaraDetections.jsonl`: Correspondances règle/fichier avec chemins relatifs, noms des machines sources, empreintes, métadonnées et positions limitées. Les octets correspondants ne sont pas exportés.
- `YaraFiles.csv` / `YaraFiles.jsonl`: Couverture par fichier, y compris fichiers ignorés, erreurs de lecture, délais dépassés et résultats partiels.
- `Summary.json`: État final, paramètres, compteurs, version du moteur et empreintes des sources.
- `Rules.yar`: Règles téléchargées exactes ou instantané des règles locales après résolution des inclusions.
- `CompilationWarnings.txt` / `CompilationError.txt`: Avertissements de compilation ou erreur si la compilation échoue.

Codes de sortie : 0 indique une analyse complète, 3 une couverture incomplète, 2 un échec de configuration/sortie et 130 une annulation. Une correspondance est une piste, pas une preuve de compromission ; son absence ne prouve pas l’innocuité. Examiner le titre de la règle et le contexte de l’artefact. Déchiffrer les contenus de quarantaine avant de les identifier.

## Correspondances MFT et récupération { #mft-recovery }

Les MFT sont analysées une par une comme des octets bruts, avec une limite distincte (512 MiB par défaut). Les enregistrements aux positions correspondantes sont interprétés, puis leurs flux DATA résidents sont analysés séparément. Seuls les flux correspondant seuls sont récupérés. Les correspondances de métadonnées, entre plusieurs enregistrements ou non résidentes sont signalées sans exporter de contenu.

`--max-mft-mib <MIB>` · `--max-mft-recoveries <N>`

La récupération est limitée par défaut à 1000 flux par exécution. Les limites, enregistrements endommagés et délais dépassés restent visibles. Les noms enregistrés portent une extension de quarantaine ; les répertoires des preuves ne sont pas recréés.

`YaraMFTResults.csv` / `YaraMFTResults.jsonl` · `RecoveredMFT/` · `.quarantined`

## Pour aller plus loin { #further-details }

- [Moteur: Boreal](https://github.com/vthib/boreal)
- [Paquets de règles: YARA Forge](https://github.com/YARAHQ/yara-forge)
- [Utilisation détaillée et référence des champs (anglais)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.md)
- [Utilisation détaillée et référence des champs (japonais)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.ja.md)
