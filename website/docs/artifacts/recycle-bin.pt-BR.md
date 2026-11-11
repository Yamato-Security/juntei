# Lixeira { #recycle-bin }

## O que a Lixeira registra { #what-the-recycle-bin-records }

Quando um usuário exclui um arquivo ou uma pasta no Explorador de Arquivos, o Windows o move para a Lixeira do seu
volume e grava ao lado um pequeno registro de índice: o caminho que ele tinha, seu tamanho e quando foi excluído.
Cada usuário tem uma pasta de lixeira em cada volume, nomeada com o SID do usuário; assim, a pasta também indica qual
conta excluiu o item.

| Versão do Windows | Índice | Conteúdo excluído |
|---|---|---|
| Vista e posteriores | Um arquivo `$I…` por item (versão de formato 1 do Vista ao 8.1, versão 2 no Windows 10 e posteriores) | `$R…`, com o mesmo nome após `$R`: o arquivo, ou a pasta com seu conteúdo |
| NT 4.0, 2000, XP, Server 2003 | Um arquivo `INFO2` por lixeira: registros de 800 bytes com um caminho ANSI e um Unicode | `D<drive letter><index><extension>`, como `Dc1.txt` |
| 95, 98, ME | Um arquivo `INFO2` (`INFO` no Windows 95): registros de 280 bytes apenas com um caminho ANSI | Os mesmos nomes `D…` |

<span class="evidence-strength">Melhor conclusão:</span> a conta cuja lixeira contém o registro moveu o item do
caminho original para a Lixeira no horário registrado. Excluir com Shift+Delete, pela linha de comando ou pela maioria
dos programas ignora a Lixeira e não deixa registro.

## Locais e coleta { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

Colete os arquivos `$I` ou o INFO2 da lixeira de cada volume. Coletar também os arquivos `$R` e `D…` mostra quais
itens excluídos ainda estão lá; sem eles, `RecycledFileFound` é `false` para todos os itens. O conteúdo excluído é uma
evidência por si só e pode ser malware: trate-o como tal.

## Cuidados para o analista { #analyst-cautions }

- `DeletedTime` é quando o item foi movido para a Lixeira, pelo relógio do sistema, como FILETIME em UTC. Não é quando
  a Lixeira foi esvaziada. Os horários de criação e modificação do próprio item estão no `$MFT` e no arquivo `$R`.
- O SID corresponde à conta que excluiu o item, não necessariamente o proprietário do arquivo. Um SID sem perfil no
  `ProfileList` do hive SOFTWARE (uma conta excluída, ou uma conta de domínio que nunca fez logon) não tem `User`.
- Quando a Lixeira é esvaziada, ou quando um item é restaurado ou excluído da Lixeira, o Windows Vista e posteriores
  excluem seus arquivos `$I` e `$R`. Os nomes deles podem permanecer no Diário USN, no `$MFT` e no slack de `$I30`; o
  Juntei não recupera deles registros `$I` excluídos. O INFO2 mantém esses registros e zera o primeiro byte do caminho
  ANSI deles: `RemovedFromBin` é `true`, e o Juntei restaura esse caractere a partir do número da unidade.
- `FileSize` em um arquivo `$I` é o tamanho do item; para uma pasta, o tamanho total do seu conteúdo. No INFO2, é o
  espaço que o item ocupava em disco, em clusters inteiros (4.096 bytes para um arquivo pequeno).
- O caminho ANSI do INFO2 costuma estar no formato 8.3 e exige a página de código do sistema: use `--code-page 932`
  para Windows em japonês, 936 ou 950 para Windows em chinês. O caminho Unicode, quando existe, fica em
  `OriginalPath`.
- Alguns sistemas Windows Vista gravaram arquivos `$I` de 543 bytes cujo campo de tamanho tem um byte a menos. O
  Juntei lê o horário e o caminho deles e deixa `FileSize` vazio e relata um problema.
- `RecycledFileFound` descreve a coleta, não o endpoint: `false` pode significar que o coletor não coletou os arquivos
  `$R`, ou que o item não está mais na Lixeira.
- Lixeiras em volumes sem uma instalação do Windows obtêm os nomes de usuário da única instalação do Windows da
  coleta, se houver exatamente uma.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

Com `-d`, o Juntei lê arquivos cujo nome é `$I` seguido de seis letras ou dígitos (com qualquer extensão), `INFO2`
e `INFO` em uma pasta `RECYCLED`. Com `-f`, lê o arquivo qualquer que seja o nome.

## Campos de saída CSV { #csv-output-fields }

| Campo | Significado |
|---|---|
| `SourceFile` | O arquivo `$I` ou INFO2. |
| `DeletedTime` | Quando o item foi movido para a Lixeira (UTC). |
| `OriginalPath` | Onde o item estava. No INFO2, o caminho Unicode quando o registro tem um; caso contrário, o caminho ANSI. |
| `FileSize` | `$I`: o tamanho do item em bytes (o total, para uma pasta). INFO2: o espaço que ocupa em disco, em clusters inteiros. Vazio quando não pode ser lido. |
| `User`, `SID` | O SID da pasta da lixeira e o usuário que dá nome ao perfil correspondente em `ProfileList`. |
| `RecycledFile`, `RecycledFileFound` | O caminho do conteúdo excluído ao lado do índice (`$R…`, `D<drive><index><extension>`) e se ele está na coleta. |
| `Format`, `FormatVersion` | `$I` 1 (Vista ao 8.1) ou 2 (Windows 10 e posteriores); `INFO2` 0 (95), 2 (NT 4.0), 4 (98) ou 5 (ME, 2000 ao Server 2003). |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2: o número do registro (o número no nome do seu arquivo `D…`), a unidade e onde o registro começa no INFO2. Um caminho UNC não tem unidade. |
| `RemovedFromBin` | INFO2: `true` quando o item foi restaurado ou excluído da Lixeira. |
| `ShortPath` | INFO2: o caminho ANSI, quando difere de `OriginalPath` (muitas vezes nomes 8.3). |
| `Problems` | Danos: um arquivo truncado ou longo demais, um número de unidade inválido, um caminho que não é UTF-16 válido, um horário de exclusão que não é um horário válido. |

JSON e JSONL têm os mesmos campos, com números e booleanos tipados, `null` para valores que um registro não tem e
`Problems` como uma lista.

## Análise automática { #automatic-parsing }

O `auto-parse` grava `FileExistence/RecycleBin.csv` (`RecycleBin.json` ou `RecycleBin.jsonl` com
`-t json` ou `-t jsonl`), junto com `FileExistence/RecycleBin-logs.txt`. Sua opção `--code-page` se aplica ao
INFO2. A partir de imagens de disco, os arquivos `$I` e o INFO2 são copiados; os arquivos `$R` e `D…` são marcadores
vazios, de modo que `RecycledFileFound` continua preenchido. A linha do tempo combinada de `-j` inclui todas as
linhas, com o identificador de `Artifact` `recycle-bin`; `-Z` converte `DeletedTime`, e o
[`super-timeline`](../commands/super-timeline.md) gera um evento `Recycle Bin deletion (moved to the bin)`
por item.

## Estratégia de investigação { #investigation-strategy }

1. Ordene por `DeletedTime` para cada usuário e compare com os horários da atividade que você está investigando.
2. Procure `OriginalPath` em LNK, Jump Lists, ShellBags e RecentDocs para ver se o item foi usado antes de ser
   excluído.
3. No Diário USN, a exclusão aparece como uma renomeação para `$Recycle.Bin` por volta de `DeletedTime`; o `$MFT`
   fornece os horários do próprio item.
4. Quando `RecycledFileFound` for `true`, calcule o hash do conteúdo `$R` ou `D…` e examine-o.
5. Se a Lixeira foi esvaziada, pesquise nomes `$I` e `$R` no Diário USN e no `$MFT`.

## Mais detalhes { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
