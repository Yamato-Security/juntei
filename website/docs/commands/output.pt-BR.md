# Métodos comuns de saída { #common-output-patterns }

## Um artefato ou uma coleta { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d` pesquisa recursivamente os arquivos adequados para aquele comando. As colunas de caminho de origem distinguem
hosts e artefatos na saída combinada.

## Salvar em CSV { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Alguns analisadores precisam de um diretório porque emitem várias tabelas CSV:

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

Mensagens e progresso vão para a saída de erro padrão; a saída padrão continua analisável.

## Sobrescrever intencionalmente { #overwrite-intentionally }

Por padrão, o Juntei se recusa a sobrescrever uma saída existente. Confirme o destino, preserve qualquer resultado
anterior que seja importante e então adicione `-C` / `--clobber`:

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## Modos silencioso e sem cores { #quiet-and-no-color-modes }

Use `-q` para automação e `-K` quando os códigos de cor do terminal forem inadequados:

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## Maiúsculas/minúsculas e caminhos { #case-and-paths }

Coloque entre aspas os caminhos do Windows que contêm espaços, `$`, metacaracteres do shell ou barras invertidas, conforme
exigido pelo seu shell. Em shells do tipo Bash, aspas simples impedem que `$MFT` e `$J` se tornem variáveis:

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
