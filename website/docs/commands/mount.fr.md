# Monter des images disque { #mounting-disk-images }

`juntei mount` rend les fichiers d'une image disque VHDX accessibles en lecture seule dans un répertoire pendant son exécution. Hayabusa, Juntei et d'autres outils peuvent les analyser sans extraction ni droits root. La fonction utilise FUSE et fonctionne sous Linux. Le lecteur et la bibliothèque de montage sont maintenus dans [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

Dans un autre terminal :

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

Appuyez sur Ctrl+C dans le premier terminal pour démonter, ou exécutez `fusermount3 -u /mnt/host1`.

## Ce que montre le montage { #what-the-mount-shows }

- Les fichiers du volume NTFS apparaissent au point de montage. Une image collectée peut contenir des répertoires tels que `C` et `LongFileNames`, ainsi que des métadonnées du volume comme `$MFT`. Plusieurs volumes NTFS apparaissent sous `Volume1`, `Volume2`, etc.
- Chaque fichier est en lecture seule et appartient au propriétaire du point de montage. Les horodatages des
  fichiers sont les horodatages NTFS `$STANDARD_INFORMATION`.
- Les noms sont listés tels que NTFS les stocke et sont recherchés à l'identique. Les noms courts DOS 8.3 et les
  flux de données alternatifs ne sont pas affichés.
- Les fichiers compressés sont décompressés (jusqu'à 1 Gio chacun). Les fichiers chiffrés par EFS ne peuvent pas
  être lus.

## Les preuves ne sont pas modifiées { #the-evidence-is-not-changed }

L'image est ouverte en lecture seule et le montage refuse les écritures. Pour un VHDX mal fermé, le journal en attente est appliqué en mémoire, sans jamais être réécrit dans l'image. Certains composants DiscUtils laissent à zéro l'instruction de saut et la signature `55 AA` du secteur de démarrage NTFS ; Juntei lit ces volumes tels quels.

## Utilisation { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Analyse native des images { #parsing-every-image-with-auto-parse }

Avec `--vhdx`, `auto-parse` lit VHDX nativement sous **Linux, macOS et Windows**, via [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser). Aucun montage système, FUSE ou droit administrateur n’est nécessaire.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Les parseurs exigent des chemins : Juntei copie donc les fichiers NTFS exposés dans un espace temporaire privé avec des tampons bornés. **Prévoyez de la place pour tous ces fichiers**, pas seulement les artefacts produisant des résultats. Les zones nulles sont écrites comme des trous si le système le permet. Noms, structure, liens physiques et dates de modification sont préservés. Les dates sources des LNK et Jump Lists viennent des métadonnées NTFS originales, jamais des copies temporaires. Une fin normale, même avec erreurs de parsing, supprime les copies ; un arrêt forcé peut les laisser. La source reste en lecture seule ; les journaux VHDX en attente sont appliqués en mémoire uniquement.

La découverte des hôtes, CSV/JSON/JSONL et `-j` utilisent le traitement habituel. `Image` dans `manifest.json` identifie l’image originale, `EvidenceRoot` son chemin relatif interne. Les erreurs de lecture et noms dangereux figurent dans `ImageErrors` et les lignes en échec de `manifest.csv`. Les résultats lisibles sont conservés ; une entrée incomplète produit le code 1. Les noms dangereux ou ambigus ne sont pas renommés. Les flux NTFS nommés ne deviennent pas des fichiers supplémentaires ; les artefacts collectés comme fichiers ordinaires restent disponibles.

La [commande `mount`](mount.md) expose un système de fichiers aux autres programmes et nécessite toujours Linux FUSE. `auto-parse --vhdx` ne l’utilise pas.

La préparation utilise par défaut un répertoire privé sur le système de fichiers de sortie. Utilisez `--temp-dir` pour choisir un répertoire existant sur un autre disque. Les métadonnées source restent séparées sur le volume de sortie ; un disque de préparation plein ne bloque donc pas l’analyse des fichiers déjà lisibles. Prévoyez de la place pour les copies et les résultats ; si le volume de sortie est plein, les résultats ne peuvent pas être enregistrés. Ctrl+C arrête les parseurs, nettoie l’espace privé et termine avec le code 130. SIGKILL ou une coupure de courant peuvent laisser des copies. La recherche ignore les sous-répertoires illisibles, mais une racine d’entrée illisible reste une erreur.

Le répertoire de préparation doit se trouver hors du répertoire de sortie. Sous Linux et macOS, SIGTERM et SIGHUP arrêtent également les processus d’analyse et nettoient l’espace de travail.

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## Conditions du montage visible par le système { #os-visible-mount-requirements }

`juntei mount` nécessite Linux FUSE 3 (`fusermount3`) et un accès à `/dev/fuse`. Sous Windows et macOS, utilisez l’analyse native ci-dessus.
