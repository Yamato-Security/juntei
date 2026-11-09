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

`auto-parse --vhdx` lit les images VHDX nativement sous **Linux, macOS et Windows**, sans montage système, FUSE ni droits administrateur.

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Les parseurs exigent des chemins de fichiers : Juntei copie les fichiers exposés par le lecteur NTFS dans un répertoire temporaire privé. **Prévoyez suffisamment d’espace temporaire pour tous ces fichiers.** L’image source reste en lecture seule et les copies sont supprimées à la fin normale du traitement.

Les lectures impossibles et les noms dangereux sont consignés dans `ImageErrors` et les lignes en échec de `manifest.csv`. Les résultats lisibles sont conservés ; une entrée incomplète entraîne le code de sortie 1.

La commande `mount`, qui expose les fichiers aux autres programmes, nécessite toujours Linux, FUSE 3 et un accès à `/dev/fuse`.

[English: implementation details and limitations](https://yamato-security.github.io/juntei/commands/auto-parse/#vhdx-disk-images).
