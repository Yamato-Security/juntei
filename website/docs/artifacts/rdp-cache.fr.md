# Cache bitmap RDP { #rdp-bitmap-cache }

## Ce que le cache bitmap RDP enregistre { #what-the-rdp-bitmap-cache-records }

Lorsque le client Bureau à distance (`mstsc.exe`) se connecte avec la mise en cache persistante des bitmaps activée, ce
qui est le cas par défaut, il enregistre sur le disque des morceaux de l'écran distant afin que les sessions suivantes
n'aient pas à les recevoir de nouveau. Chaque morceau est une tuile de 64 × 64 pixels au plus. Ensemble, les tuiles
forment de petits fragments de ce que l'utilisateur a vu sur les ordinateurs distants : fenêtres, texte, noms de
fichiers, commandes et bureaux.

| Client | Fichiers de cache | Tuiles |
|---|---|---|
| Windows 7 et versions ultérieures | `Cache0000.bin`, `Cache0001.bin`, …, jusqu'à 100 Mo chacun | 32 bits, les unes après les autres |
| Windows XP, Server 2003 et versions antérieures | `bcache2.bmc` (8 bits par pixel), `bcache22.bmc` (16), `bcache23.bmc` (24) et `bcache24.bmc` (32), jusqu'à 10 Mo chacun | Une par emplacement de 64 × 64 ; certaines compressées |

Windows 10 et versions ultérieures laissent aussi un fichier `bcache24.bmc` vide à côté des fichiers `.bin`.

<span class="evidence-strength">Meilleure conclusion :</span> l'utilisateur dont le profil contient le cache s'est
connecté avec le client Bureau à distance à un ordinateur qui affichait le contenu mis en cache. Le cache n'indique ni
de quel ordinateur il s'agissait, ni quand.

## Emplacements et collecte { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

La valeur `BitmapPersistCacheLocation` de `HKCU\Software\Microsoft\Terminal Server Client` peut déplacer le cache.
Collectez le dossier `Cache` complet de chaque profil, y compris depuis les clichés instantanés de volume : une copie
plus ancienne d'un fichier de cache peut contenir des tuiles que le fichier actuel n'a plus.

## Précautions d'analyse { #analyst-cautions }

- Les tuiles n'ont pas de dates, et le cache ne nomme pas les ordinateurs dont elles proviennent. Rattachez-les à des
  connexions RDP à l'aide d'autres preuves : les clés de registre `Terminal Server Client` du client
  (`parse-registry remote-access`), `Default.rdp` et les autres fichiers `.rdp`, les Jump Lists de `mstsc.exe`, les
  dates des fichiers de cache dans `$MFT` et les événements d'ouverture de session des ordinateurs distants.
- Un fichier de cache mêle des tuiles de nombreuses sessions et de plusieurs ordinateurs distants. Le client remplace
  les anciennes tuiles à mesure qu'un fichier se remplit ; une tuile peut donc dater de plusieurs jours, voire de
  plusieurs mois.
- Les tuiles sont des morceaux d'écrans, pas des écrans entiers. Des tuiles voisines à l'écran sont souvent voisines
  dans le fichier, mais pas toujours. Reconstituez les écrans avec
  [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher), ou à l'œil à partir des collages.
- La plupart des tuiles font 64 × 64 pixels. Les tuiles situées aux bords droit et inférieur d'un écran sont plus
  étroites ou moins hautes selon la résolution : par exemple 58 ou 62 pixels de large, ou 28 ou 35 de haut. Juntei lit
  chaque tuile à sa propre taille. Les outils qui lisent toutes les tuiles comme si elles faisaient 64 pixels de large
  affichent les tuiles étroites déformées.
- La plupart des tuiles unies (`SingleColor`) sont de l'arrière-plan. Filtrez-les pour trouver le contenu.
- Dans les fichiers `.bmc`, les tuiles 8 bits sont des couleurs de la palette de la session, que le cache ne conserve
  pas. Juntei utilise la palette par défaut de Windows ; leurs couleurs peuvent donc être fausses. Les tuiles
  compressées utilisent le RLE entrelacé du protocole Bureau à distance (MS-RDPBCGR 3.1.9), que Juntei décompresse tel
  que la spécification le définit.
- Une tuile `.bmc` plus petite que son emplacement de 64 × 64 laisse après elle la fin d'une tuile antérieure. Juntei
  enregistre ces lignes restantes dans une image distincte (`RemnantImage`). Elles appartiennent à une tuile plus
  ancienne, pas à la tuile de l'emplacement.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```

```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```

Avec `-d`, Juntei lit les fichiers dont le nom est formé de `Cache`, de chiffres et de `.bin`, ou de `bcache`, de
chiffres et de `.bmc`. Un fichier `Cache????.bin` qui ne commence pas par la signature `RDP8bmp` appartient à un autre
programme : il est compté dans le résumé et ignoré. Avec `-f`, Juntei lit le fichier quel que soit son nom.

### Images { #images }

`--extract` crée un dossier pour chaque fichier de cache, `<DIR>/<user>/<cache file>/`. Lorsque deux fichiers de cache
devraient partager le même dossier, par exemple un fichier de cache et sa copie issue d'un cliché instantané de volume,
le second dossier est numéroté (`Cache0001.bin_2`). Le dossier contient :

- `<cache file>_<index>.bmp` : chaque tuile, à sa propre taille. Les noms sont ceux qu'utilise bmc-tools, si bien que
  RdpCacheStitcher peut ouvrir le dossier.
- `<cache file>_old_<index>.bmp` : `.bmc` uniquement, la fin d'une tuile antérieure restée après une tuile plus petite.
- `<cache file>_collage.png` : toutes les tuiles dans l'ordre du fichier, 64 par ligne (`--collage-width`), chacune
  dans une cellule de 64 × 64 à partir de son coin supérieur gauche. La tuile d'index *i* se trouve à la ligne *i* / 64
  et à la colonne *i* % 64, en comptant à partir de 0. Le magenta remplit les parties vides des cellules : à côté d'une
  tuile étroite, sous une tuile moins haute, et là où une tuile n'a pas pu être lue.

`--collage-only` n'enregistre que les collages. Une image qui existe déjà n'est pas remplacée sans `-C`, et le champ
`Problems` de sa tuile le signale. Les fichiers de cache eux-mêmes ne sont jamais remplacés, même avec `-C`.

## Champs de sortie CSV { #csv-output-fields }

| Champ | Signification |
|---|---|
| `SourceFile` | Le fichier de cache. |
| `User` | L'utilisateur dans le dossier de profil duquel se trouve le fichier de cache, d'après son chemin. |
| `TileIndex` | Le numéro de la tuile dans le fichier, à partir de 0, comme dans le nom de son image. |
| `Width`, `Height` | La taille de la tuile en pixels. |
| `SingleColor` | `true` lorsque tous les pixels de la tuile ont la même couleur. |
| `TileImage` | Avec `--extract` : le fichier BMP de la tuile. |
| `Collage` | Avec `--extract` : le collage du fichier de cache. |
| `RemnantImage` | `.bmc` avec `--extract` : la fin d'une tuile antérieure restée dans l'emplacement. |
| `Key` | La clé 64 bits de la tuile dans le protocole Bureau à distance, en hexadécimal. |
| `Format`, `FormatVersion` | `bin` avec la version qui suit sa signature (6 sous Windows 10 et versions ultérieures), ou `bmc`. |
| `BitsPerPixel` | 32 pour `.bin` ; 8, 16, 24 ou 32 pour `.bmc`. |
| `Compressed`, `Flags` | `.bmc` : si la tuile était compressée, et les indicateurs de son en-tête. |
| `Offset` | Position à laquelle l'en-tête de la tuile commence dans le fichier. |
| `Problems` | Dommages : fichier qui se termine au milieu d'une tuile, en-tête de tuile qu'aucune tuile ne pourrait avoir, données qui ne correspondent pas à la taille de la tuile, données compressées impossibles à décompresser, image qui n'a pas pu être enregistrée. |

Les sorties JSON et JSONL comportent les mêmes champs, avec les nombres et les booléens typés, `null` pour les valeurs
qu'une tuile ne possède pas, et `Problems` sous forme de liste. Les dommages qui ne concernent aucune tuile en
particulier, comme des données situées après la dernière tuile, sont signalés dans les messages de l'exécution.

## Analyse automatique { #automatic-parsing }

`auto-parse` écrit `LateralMovement/RdpCache.csv` (`RdpCache.json` ou `RdpCache.jsonl` avec
`-t json` ou `-t jsonl`), accompagné de `LateralMovement/RdpCache-logs.txt`, et enregistre le collage de chaque
fichier de cache dans un dossier `LateralMovement/RdpCache/`. Ajoutez `--extract-rdp-tiles` pour enregistrer aussi
chaque tuile, ce qui peut représenter des dizaines de milliers de fichiers pour un seul utilisateur. Depuis des images
disque, les fichiers de cache sont copiés. La chronologie combinée `-j` inclut chaque ligne, avec l'identifiant
d'`Artifact` `rdp-cache`. Les tuiles n'ont pas de dates ; [`super-timeline`](../commands/super-timeline.md) n'en
produit donc aucun événement.

## Stratégie d'investigation { #investigation-strategy }

1. Ouvrez les collages et cherchez du texte : noms d'ordinateurs et d'utilisateurs, chemins de fichiers, commandes,
   fenêtres de console, barres d'adresse de navigateurs et boîtes de dialogue.
2. Reconstituez les écrans avec RdpCacheStitcher à partir d'un dossier de tuiles enregistrées avec `--extract`.
3. Identifiez les ordinateurs auxquels l'utilisateur s'est connecté, à partir du registre du client
   (`parse-registry remote-access`), des fichiers `.rdp` et des Jump Lists de `mstsc.exe`. Sur ces ordinateurs,
   recherchez les ouvertures de session RDP correspondantes et les enregistrements de la journalisation des accès
   utilisateur.
4. Utilisez les dates `$MFT` des fichiers de cache, et celles de leurs copies dans les clichés instantanés de volume,
   pour cerner le moment où les tuiles ont été écrites.

## Plus de détails { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017 : le cache bitmap RDP](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — plugin de cache RDP](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9 : Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — cache bitmap persistant](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
