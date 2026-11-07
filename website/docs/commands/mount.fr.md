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

## Analyser chaque image avec auto-parse { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` analyse les systèmes Windows contenus dans chaque image disque VHDX du répertoire des preuves
(ou dans l'image unique indiquée avec `-d`), sans montage séparé :

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- Chaque image est montée en lecture seule dans un nouveau répertoire temporaire, dans un dossier portant le nom
  de l'image (`host1-triage`), et les systèmes qui s'y trouvent sont analysés comme dans une collecte sur
  disque. Les images sont démontées à la fin, ainsi qu'en cas de Ctrl+C.
- Le terminal liste les images ainsi que l'image de chaque système :
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- Dans le manifeste, `EvidenceDirectory` est le répertoire indiqué et le champ `Image` de chaque système nomme
  son image. `EvidenceRoot` et les commandes des parseurs utilisent le dossier de montage
  (`./host1-triage/C`).
- Si l'image contient des fichiers issus de Volume Shadow Copies dans des dossiers comme `VSS32`, chacun est détecté et analysé comme un autre système (`HOST1_2`, …).
- Une image qui ne peut pas être montée est signalée ; les autres sont tout de même analysées, et l'exécution se
  termine avec le code de sortie 1.

Si un volume NTFS ne peut pas être ouvert, Juntei le signale et conserve les volumes lisibles. Pour une image partiellement lisible, `auto-parse --vhdx` renvoie le code 1 ; `mount` renvoie également 1 après le démontage.

## Prérequis { #requirements }

- Linux : FUSE 3 (`fusermount3`, le paquet `fuse3`) et l'accès à `/dev/fuse`, dont disposent généralement les
  utilisateurs ordinaires. Les droits root ne sont pas nécessaires.
- Windows et macOS ne sont pas pris en charge. Sous Windows, attachez plutôt l'image en lecture seule dans un
  PowerShell avec privilèges élevés, puis indiquez sa lettre de lecteur aux outils :

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
