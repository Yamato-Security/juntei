# Méthodes de sortie communes { #common-output-patterns }

## Un artefact ou une collecte { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d` recherche récursivement les fichiers appropriés à cette commande. Les colonnes de chemin source permettent de distinguer
les hôtes et les artefacts dans une sortie combinée.

## Enregistrer en CSV { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Certains parseurs nécessitent un répertoire, car ils produisent plusieurs tables CSV :

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## Diffuser en JSONL { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

Les messages et la progression sont envoyés sur la sortie d'erreur standard ; la sortie standard reste analysable.

## Écraser délibérément { #overwrite-intentionally }

Par défaut, Juntei refuse d'écraser une sortie existante. Confirmez la cible, préservez tout résultat
antérieur important, puis ajoutez `-C` / `--clobber` :

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## Modes silencieux et sans couleur { #quiet-and-no-color-modes }

Utilisez `-q` pour l'automatisation et `-K` lorsque les codes de couleur du terminal ne conviennent pas :

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## Casse et chemins { #case-and-paths }

Mettez entre guillemets les chemins Windows contenant des espaces, `$`, des métacaractères du shell ou des barres obliques inverses, selon les exigences de votre
shell. Dans les shells de type Bash, les guillemets simples empêchent `$MFT` et `$J` de devenir des variables :

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
