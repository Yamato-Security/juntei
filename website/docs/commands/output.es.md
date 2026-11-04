# Métodos comunes de salida { #common-output-patterns }

## Un artefacto o una recopilación { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d` busca de forma recursiva los archivos adecuados para ese comando. Las columnas de ruta de origen distinguen
los hosts y los artefactos en la salida combinada.

## Guardar en CSV { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Algunos analizadores necesitan un directorio porque generan varias tablas CSV:

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## Transmitir JSONL { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

Los mensajes y el progreso se envían a la salida de error estándar; la salida estándar sigue siendo analizable.

## Sobrescribir de forma intencionada { #overwrite-intentionally }

De forma predeterminada, Juntei se niega a sobrescribir una salida existente. Confirme el destino, conserve cualquier resultado
anterior que sea importante y, a continuación, añada `-C` / `--clobber`:

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## Modos silencioso y sin color { #quiet-and-no-color-modes }

Utilice `-q` para la automatización y `-K` cuando los códigos de color del terminal no sean adecuados:

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## Mayúsculas y rutas { #case-and-paths }

Ponga entre comillas las rutas de Windows que contengan espacios, `$`, metacaracteres del shell o barras invertidas, según lo requiera su
shell. En los shells de tipo Bash, las comillas simples impiden que `$MFT` y `$J` se conviertan en variables:

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
