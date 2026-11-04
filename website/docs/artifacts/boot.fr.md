# Secteur de démarrage (`$Boot`) { #boot-sector-boot }

## Qu'est-ce que `$Boot` { #what-boot-is }

`$Boot` est l'entrée MFT 7 de chaque volume NTFS : les 8 premiers Kio du volume. Son premier secteur, le
**secteur de démarrage du volume** (volume boot record), contient le bloc de paramètres du BIOS (BIOS parameter block) qui décrit le système de fichiers :

- la taille des secteurs et des clusters, le nombre de secteurs du volume, et les **secteurs cachés**
  (les secteurs du disque situés avant le volume, c'est-à-dire l'endroit où commence sa partition) ;
- l'emplacement de début du `$MFT` et du `$MFTMirr`, ainsi que la taille des enregistrements FILE et INDX ;
- le **numéro de série du volume** sur 64 bits, défini lors du formatage du volume.

Le reste du premier secteur et les 15 secteurs qui le suivent contiennent le **code de démarrage** qui charge
Windows.

Les 32 bits de poids faible du numéro de série correspondent à ce que Windows affiche sous la forme `XXXX-XXXX` (`vol`, `dir`) et à ce que
les [raccourcis LNK](lnk.md), les [Jump Lists](jump-lists.md) et [Prefetch](prefetch.md) enregistrent du
volume sur lequel se trouvait un fichier. `$Boot` relie ces enregistrements à un volume d'un hôte collecté.

<span class="evidence-strength">Meilleure conclusion :</span> un numéro de série de volume LNK, Jump List ou Prefetch
égal au `VolumeSerialNumber` d'un `$Boot` désigne ce volume, ou un autre volume auquel on a
attribué le même numéro de série ou qui a été copié avec celui-ci. Les numéros de série peuvent être modifiés et ne sont pas
uniques d'un hôte à l'autre.

## Emplacement et collecte { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

Collectez-le avec un outil forensique capable d'un accès brut, ou lisez le premier secteur d'une image de volume : il s'agit
des mêmes octets. Juntei lit tout fichier commençant par un secteur de démarrage NTFS ; une image de volume peut donc servir
d'entrée. Une image de disque commence en revanche par la table de partitions ; fournissez à Juntei le volume de la partition.

## Précautions d'analyse { #analyst-cautions }

- Le numéro de série change lors du formatage du volume. Un fichier LNK dont le numéro de série ne correspond
  à aucun volume actuel peut désigner un support amovible, le serveur d'un partage réseau ou un volume formaté
  depuis.
- Le code de démarrage est identique sur les volumes formatés par la même version de Windows : il est identique dans les fichiers `$Boot`
  de Windows 10, 11 et Server 2012 R2 du DFIR Artifact Museum. Un bootkit de secteur de démarrage de volume
  le modifie, ou modifie le champ des secteurs cachés, que le code de démarrage utilise pour trouver le reste de
  `$Boot`. Comparez `BootCodeSHA256`, `JumpInstruction` et `HiddenSectors` avec d'autres hôtes et avec la table de
  partitions avant de conclure quoi que ce soit : un gestionnaire de démarrage ou un logiciel de chiffrement peut aussi modifier le code
  de démarrage.
- Une image d'un volume chiffré par BitLocker commence par le secteur de démarrage propre à BitLocker (identifiant OEM
  `-FVE-FS-`). Lorsqu'elle est fournie avec `-f`, Juntei la signale, et `Problems` indique qu'il ne s'agit pas de NTFS ; le secteur de démarrage
  NTFS se trouve dans les données chiffrées. Un `$Boot` collecté depuis le volume déverrouillé est celui de NTFS.
- `Problems` liste les incohérences structurelles et les troncatures reconnues, sans constituer un test complet de montabilité par Windows. Utilisez-le pour évaluer les secteurs récupérés par carving et les copies, et non comme une constatation d'attaque en soi. L'empreinte du code de démarrage exclut le saut initial, le bloc de paramètres du BIOS et la signature de fin ; examinez également ces champs.

## Analyser avec Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

Le résumé liste, pour chaque secteur de démarrage, le numéro de série du volume (tel que Windows l'affiche, et tel que les champs LNK et
Prefetch l'affichent), la taille, la taille de cluster et le cluster du `$MFT`. Lorsque plusieurs hôtes sont analysés, il
les regroupe par empreinte du code de démarrage, de sorte qu'un volume dont le code de démarrage diffère des autres ressorte.

## Champs de sortie CSV { #csv-output-fields }

| Champ | Interprétation |
|---|---|
| `SourcePath` | Fichier d'entrée. |
| `VolumeSerialNumber` | Le numéro de série sur 32 bits, tel que l'affichent les sorties LNK, Jump List et Prefetch (`180B1E67`). Windows l'affiche sous la forme `180B-1E67`. |
| `VolumeSerialNumber64` | Le numéro de série complet sur 64 bits. |
| `OEM-ID` | `NTFS` sur un volume NTFS. |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | La géométrie du volume. Un nombre de secteurs par cluster supérieur à 128 est enregistré sous forme de puissance de deux, décodée ici. Vide lorsqu'il n'est pas valide. |
| `TotalSectors`, `VolumeSize` | Nombre de secteurs du volume et leur taille en octets, sans la copie du secteur de démarrage située à sa fin. |
| `HiddenSectors` | Secteurs du disque situés avant le volume : l'endroit où commence sa partition. |
| `MFT-Cluster`, `MFT-Offset` | L'emplacement de début du `$MFT`, en clusters et en octets depuis le début du volume. |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | L'emplacement de début du `$MFTMirr`. |
| `FileRecordSize`, `IndexRecordSize` | Octets par enregistrement FILE (généralement 1 024) et par enregistrement INDX (généralement 4 096). |
| `MediaDescriptor` | `F8` pour un disque dur. |
| `JumpInstruction` | Les trois premiers octets (`EB 52 90`) : un saut par-dessus le bloc de paramètres du BIOS vers le code de démarrage. |
| `EndSignature` | `55 AA` à la fin d'un secteur de démarrage valide. |
| `BootCodeSHA256` | SHA-256 du code de démarrage : les octets 0x54 à 0x1FD du premier secteur et le reste des 8 Kio. |
| `Problems` | Ce que Windows ne monterait pas : un identifiant OEM autre que `NTFS`, absence de `55 AA`, tailles non valides, champs FAT que NTFS maintient à zéro mais qui sont renseignés, `$MFT` au-delà de la fin du volume ; ainsi qu'un fichier de moins de 8 Kio. |

Les sorties JSON/JSONL comportent les mêmes champs et noms, avec les nombres sous forme numérique, `null` pour les valeurs inconnues et
`Problems` sous forme de liste.

## Stratégie d'investigation { #investigation-strategy }

1. Placez le `VolumeSerialNumber` de chaque hôte à côté des numéros de série de volume de ses fichiers LNK, Jump
   Lists et fichiers Prefetch : ceux qui ne correspondent à aucun volume collecté sont d'autres volumes (clés
   USB, partages, volumes formatés depuis).
2. Recherchez les autres numéros de série dans la sortie des [périphériques USB](usb.md), qui liste les numéros de série
   de volume des volumes amovibles montés.
3. Comparez `BootCodeSHA256` et `HiddenSectors` d'un hôte à l'autre. Examinez un code de démarrage divergent
   avec un désassembleur et la table de partitions du disque, et recherchez les entrées des journaux d'événements liées au démarrage.
4. Utilisez `MFT-Offset` et `ClusterSize` pour trouver le `$MFT` dans une image de volume.

## Plus de détails { #further-details }

- [Microsoft : vue d'ensemble de NTFS](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs : documentation du format NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (l'en-tête de volume)
- [The Sleuth Kit : fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (sa crate `boot`)
