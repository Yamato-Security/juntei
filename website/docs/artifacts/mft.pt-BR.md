# Tabela Mestre de Arquivos (`$MFT`) { #master-file-table-mft }

## O que é a MFT { #what-the-mft-is }

O NTFS representa cada arquivo e diretório com um registro na Tabela Mestre de Arquivos (Master File Table). Os registros contêm
atributos como nomes, referências ao diretório pai, carimbos de data/hora, tamanhos, flags, identificadores de segurança,
informações de reparse e metadados de fluxos de dados. O conteúdo de arquivos pequenos e os fluxos de dados alternativos podem
ser **residentes**, o que significa que seus bytes são armazenados dentro do próprio registro da MFT.

A MFT é fundamental porque abrange registros alocados e excluídos em todo o volume.
Ela pode mostrar que um arquivo existiu, onde estava vinculado, se foi excluído e como seus metadados
mudaram — mesmo quando a navegação comum pelos diretórios já não o mostra.

<span class="evidence-strength">Melhor conclusão:</span> um registro de arquivo NTFS com os metadados
exibidos existia no momento da coleta ou sobreviveu em um registro não alocado. A MFT sozinha não
prova execução nem intenção do usuário.

## Localização e coleta { #location-and-collection }

```text
C:\$MFT
```

`$MFT` são metadados protegidos do NTFS. Colete-o com uma ferramenta de aquisição forense ou de triagem capaz
de acesso bruto ao NTFS; uma cópia comum em modo de usuário normalmente não consegue lê-lo. Combine-o com `$J`, `$LogFile`
e os metadados do volume sempre que possível.

## Cuidados para o analista { #analyst-cautions }

- Um registro excluído pode estar parcialmente sobrescrito ou seu número de registro pode ter sido reutilizado.
- `SN` distingue gerações do mesmo número de entrada. Associe a entrada **e** a sequência.
- Os carimbos de data/hora de `$STANDARD_INFORMATION` (`0x10`) e de `$FILE_NAME` (`0x30`) têm comportamentos de
  atualização diferentes. Nenhum dos dois é imune a manipulação.
- `SI<FN`, `ZeroU-Secs` e `Copied` são pistas, não veredictos automáticos de timestomping.
- Hard links produzem vários atributos `$FILE_NAME` e caminhos para um único registro de arquivo.
- O NTFS pode armazenar vários nomes (incluindo DOS 8.3), fluxos de dados alternativos e registros de extensão.
- As atualizações do horário de acesso podem estar desativadas, atrasadas ou agrupadas.
- Um caminho completo é reconstruído a partir dos registros pais. `[Orphaned]`, `[Unknown]` ou `PathUnknown`
  sinalizam uma ascendência ausente/reutilizada/danificada — não um nome de diretório literal.
- O **slack** de um registro FILE (seus bytes além do tamanho em uso) pode manter nomes que ele continha antes: as
  entradas de índice de um diretório pequeno cujo índice foi movido para uma alocação de índice (a maioria delas ainda
  está em uso ali), um nome anterior ou o ocupante anterior de um registro reutilizado. Os nomes recuperados
  não têm `$STANDARD_INFORMATION`, e o cabeçalho de entrada de um nome pode ter sido sobrescrito. Quando o
  tamanho em uso ou os atributos de um registro não indicam onde seu slack começa, nada é recuperado dele
  e o `ParseErrors` de sua linha informa `slack not read` e o motivo.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

Nunca execute fluxos residentes extraídos com `-e`; o Juntei lhes dá nomes `.dontrun` por um motivo.
As colunas de dados residentes de `-i` são os mesmos bytes: trate-as como conteúdo não confiável.

`-D` imprime uma entrada: seu cabeçalho, todos os atributos (com data runs) e os nomes em seu slack conforme
`-t json -x` os grava, e depois os bytes do registro. `-L` lista todos os nomes de um diretório, em uso e
excluídos, com seu número de entrada e de sequência, tipo, tamanho e horário de modificação. Um diretório é informado por
entrada (`30`, `0x1E`, `30-1`) ou por caminho (`\Windows\System32`, `C:/Windows`; sem distinção entre maiúsculas e minúsculas).

## Campos de saída CSV { #csv-output-fields }

| Campo | Interpretação |
|---|---|
| `EntryNum` (Entry Number) | Número do registro da MFT. Combine com `SN`; números de registro são reutilizados. |
| `SN` (Sequence Number) | Contador de geração usado pelas referências do NTFS. Uma divergência pode revelar relações obsoletas/excluídas. |
| `InUse` | Flag de alocação no momento da coleta. `false` significa que o registro está livre/excluído, não que todos os bytes estão intactos. |
| `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number) | Referência ao diretório pai a partir de `$FILE_NAME`. Valide ambas as partes. |
| `ParentPath` | Caminho do pai reconstruído. Marcadores de desconhecido/órfão expõem ascendência não resolvida. |
| `File` (Filename) | Nome a partir do `$FILE_NAME` ou do fluxo da linha atual. Um registro pode ter várias linhas. |
| `Ext` (Extension) | Extensão extraída do nome por conveniência; não é uma determinação confiável do tipo de arquivo. |
| `Size` (File Size) | Tamanho lógico associado à linha/ao fluxo. A alocação esparsa/compactada pode ser diferente. |
| `RefCnt` (Reference Count) | Contagem de referências de hard links do NTFS; compare com cautela com o número de nomes sobreviventes. |
| `ReparseTarget` | Destino decodificado de pontos de reparse compatíveis, como junções/links simbólicos. Os caminhos podem redirecionar. |
| `Dir` (Directory) | O registro tem a flag de diretório. |
| `HasADS` (Has Alternate Data Streams) | O registro contém fluxos de dados alternativos nomeados. Isso não o torna malicioso. |
| `IsADS` (Is Alternate Data Stream) | Esta linha representa um fluxo de dados alternativo nomeado em vez dos dados sem nome do arquivo. |
| `SI<FN` (Standard Information Before File Name) | A criação SI é anterior a um horário de criação FN diferente. Investigue cópia/timestomping e causas legítimas. |
| `ZeroU-Secs` (Microsecond Zeros) | O horário de criação/modificação SI cai exatamente em um limite de milissegundo. Apenas uma heurística fraca de timestomping. |
| `Copied` | O horário de modificação é anterior ao de criação. Frequentemente consistente com cópia de arquivo; não prova ação de atacante. |
| `SI-Flags` (Standard Information Flags) | Atributos/flags de arquivo de `$STANDARD_INFORMATION`. As flags de oculto/sistema podem ser pivôs úteis. |
| `Type` (Name Type) | Namespace do `$FILE_NAME` (Win32, DOS, POSIX ou combinado). Use para evitar contar nomes em duplicidade. |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | Horários SI. Campos comuns de linha do tempo, mas ferramentas em modo de usuário podem alterar vários deles. |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | Horários FN. São atualizados de forma diferente e geralmente mais difíceis de alterar com APIs comuns; ainda assim não são imutáveis. |
| `USN` (Update Sequence Number) | Último USN associado ao atributo SI do registro. Pivô para o `$J` se esse USN sobreviver. |
| `LogfileSN` (Logfile Sequence Number) | Número de sequência do `$LogFile` do NTFS a partir do cabeçalho do registro FILE; útil na análise de transações de baixo nível. |
| `SID` (Security ID) | Índice no `$Secure` do NTFS; não é um SID do Windows por si só. Una-o ao `SecurityID` de [`parse-ntfs secure-sds`](secure-sds.md) para obter o proprietário e as ACLs do arquivo. |
| `ObjectID-FileDroid` | Object ID / identificador de rastreamento de links distribuídos, quando presente; correlacione com os droids de LNK/Jump List. |
| `LoggedUtilStream` | Contexto decodificado de `$LOGGED_UTILITY_STREAM`, quando compatível. Revise valores desconhecidos. |
| `ZoneID-Contents` | Conteúdo residente de `Zone.Identifier`, como ZoneId e URLs de download. Pode estar ausente, ter sido removido ou ser não residente. |
| `SourceFile` | O `$MFT` de entrada, essencial ao combinar hosts ou volumes. |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | Com `-i`: o conteúdo do `$DATA` residente da linha (do arquivo ou do fluxo de dados alternativo), em Base64, como bytes (`41-42`) e como texto quando é ASCII imprimível. Vazio quando os dados não são residentes ou foram filtrados por `-E`/`-M`. |
| `ParseErrors` | Danos ou relações não compatíveis que afetam esta linha. Nunca descarte esta coluna ao filtrar. |
| `FromSlack` | Com `-x`: `True` para um nome recuperado do slack de um registro FILE. Seus `EntryNum` e `SN` são o arquivo para o qual o nome apontava (vazios quando o cabeçalho de sua entrada foi sobrescrito), seus carimbos de data/hora recuperados estão nas colunas `(FN)` e `InUse` é `False`. |
| `SlackRecord`, `SlackOffset` | Com `-x`: o registro FILE cujo slack continha o nome, pela sua posição no `$MFT` e não pelo número de registro gravado dentro dele (o do próprio registro de extensão, ainda que seus nomes sejam listados com o registro base), e o deslocamento em bytes do nome no `$MFT`. Os `ParseErrors` de uma linha de slack são os problemas desse registro, como `fixup values do not match`: os bytes recuperados podem estar afetados. |
| `CopyOfEntryInUse` | Com `-x`: um arquivo em uso ainda tem este nome no mesmo diretório (e é o mesmo arquivo, quando a referência sobreviveu). Os nomes que nenhum arquivo em uso tem são os interessantes: excluídos, renomeados ou o ocupante anterior de um registro reutilizado. |

As colunas de carimbos de data/hora seguem a ordem MACB (Modified, Accessed, Changed, Birth/criação):
`DataModified` SI/FN, `Accessed` SI/FN, `MetadataChanged` SI/FN e então `Created` SI/FN.

`(SI)` significa `$STANDARD_INFORMATION`; `(FN)` significa `$FILE_NAME`. Esses nomes descrevem a
origem usual para um registro completo. Para uma linha de nome de arquivo normal sem atributo SI, o
mecanismo de fallback existente coloca a criação FN em `Created (FN)` e os valores FN de modificação/alteração do registro/acesso
em `DataModified (SI)`, `MetadataChanged (SI)` e `Accessed (SI)`, respectivamente.
As linhas recuperadas do slack colocam os quatro horários nas colunas `(FN)` e deixam as colunas SI vazias.
Os valores de fallback das linhas normais **não** são evidência SI independente. Use os atributos JSON originais
e a proveniência do slack ao determinar a origem do carimbo de data/hora.

`-t ys-ntfs-parser-csv` (antes `ys-mft-parser-csv`, ainda aceito) usa o esquema orientado a entradas
do analisador mantido. Esses nomes de coluna legíveis se aplicam à saída padrão `-t csv`,
incluindo o `MFT.csv` do `auto-parse`. O JSON/JSONL mantém todos os
atributos (inclusive o `$DATA` residente, em hexadecimal) e `FullPath`; use-o quando o CSV
investigativo plano omitir a estrutura de baixo nível. Com `-x`, cada entrada JSON também tem
`SlackFileNames`.

## Estratégia de investigação { #investigation-strategy }

1. Filtre uma janela de tempo, mas mantenha os registros excluídos e ambos os conjuntos de carimbos de data/hora.
2. Pesquise caminhos completos, extensões duplas suspeitas, ADS, diretórios de preparação e hashes/nomes conhecidos.
   Com `-x`, inclua os nomes do slack que nenhum arquivo em uso tem (`CopyOfEntryInUse` `False`); com
   `-i -E ps1,bat,vbs,js`, leia scripts pequenos diretamente do CSV.
3. Use a entrada/sequência e o `USN` como pivô para o Diário USN.
4. Compare as referências MFT de Prefetch/LNK/Jump List com o registro de destino.
5. Correlacione a exclusão com a quarentena do Defender ou com a atividade USN de `FILE_DELETE`/renomeação.

## Mais detalhes { #further-details }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Crate mft do ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [Analisador mft de Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [Documentação de formato do libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
