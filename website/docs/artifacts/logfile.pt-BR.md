# Log de transações NTFS (`$LogFile`) { #ntfs-transaction-log-logfile }

## O que é e por que é importante { #what-it-is-and-why-it-matters }

O NTFS é o sistema de arquivos que o Windows normalmente usa no volume do sistema. Para se recuperar de uma falha, ele
registra as alterações que pretende fazer nos metadados do sistema de arquivos em um arquivo oculto do volume chamado
`$LogFile`. Cada atualização pode conter bytes de **redo** (o novo estado) e bytes de **undo** (o estado anterior
necessário para reverter uma alteração). Ele é distinto dos Logs de Eventos do Windows, dos arquivos `.LOG1`
do Registro e do [Diário USN](usn-journal.md).

Para o investigador, o estado anterior é particularmente útil. Um registro de undo pode preservar um
nome de arquivo anterior a uma renomeação, uma entrada de diretório excluída ou horários anteriores a uma atualização de metadados.
Alguns registros também contêm bytes que o NTFS gravou no Diário USN, incluindo registros que não estão mais no
`$J` coletado. No entanto, muitas transações dizem respeito à atividade rotineira do sistema operacional e não
descrevem um ataque.

**Exemplo:** uma entrada de índice removida menciona `report.txt`; uma entrada adicionada posteriormente menciona
`report.txt.encrypted`. A correspondência da referência de arquivo completa, do pai, da cadeia de transações e de outros
artefatos pode sustentar uma renomeação. Esses dois nomes, por si sós, não identificam ransomware, seu operador
nem o horário exato da criptografia.

## Localização e coleta { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

Cada volume NTFS tem seu próprio log. Adquira-o com uma ferramenta que leia metadados NTFS ou que o exporte
de uma imagem forense ou de um snapshot; uma cópia comum de arquivos do Windows pode não conseguir acessá-lo. Colete o
`$MFT` e o `$Extend\$UsnJrnl:$J` do mesmo volume no momento mais próximo possível. Preserve os arquivos
originais e os hashes. Uma aquisição posterior pode conter um trecho diferente de atividade.

O `$LogFile` é um **log circular**: novos registros sobrescrevem os antigos. A retenção depende do tamanho do log
e da carga de trabalho e pode ser de apenas minutos ou horas. A ausência de um nome de arquivo ou de uma operação não é
evidência de que ela nunca aconteceu.

## O que um registro prova, e cuidados { #what-a-record-proves-and-cautions }

- Uma atualização registra metadados que o NTFS registrou em log para recuperação. Uma única entrada de redo não prova que
  uma ação de nível mais alto inteira foi concluída nem que uma determinada pessoa a iniciou. Correlacione
  registros relacionados, a conclusão da transação, o sistema de arquivos resultante e evidências independentes.
- **Não existe um horário geral de transação.** Os horários embutidos descrevem metadados de arquivos, às vezes
  desatualizados, copiados ou modificados pelo atacante. Não coloque `CreatedOn` em uma linha do tempo do incidente
  como o horário de uma renomeação ou alteração de horário. Os registros USN podem fornecer um horário separado.
- Os LSNs (números de sequência do log) ordenam os registros **dentro do log de um único volume**. Eles não são horários de relógio
  e não devem ser comparados entre hosts ou volumes. Registros de passagens anteriores do buffer circular
  podem sobreviver ao lado de atividades mais recentes.
- Um slot da tabela de transações é reutilizado. Agrupe por `TransactionFirstLsn`, `TransactionId`, identidade do
  cliente e origem, e não apenas por `TransactionId`. As cadeias podem apontar para fora dos dados retidos.
- As referências de arquivo contêm um número de entrada **e um número de sequência**. Caso contrário, a reutilização de uma entrada da MFT pode
  fazer com que arquivos não relacionados pareçam ser o mesmo arquivo.
- Os caminhos fornecidos pelo `$MFT` descrevem o **estado da MFT coletada**, não um caminho histórico
  reconstruído. O Juntei faz a correspondência dos números de sequência e deixa explícitos os componentes desconhecidos. Um pai pode
  ter sido renomeado após a transação, e hard links podem oferecer mais de um nome atual.
- `TargetInferred` significa que um mapeamento de atributo aberto veio de um trecho diferente do log.
  Trate esse mapeamento com cuidado adicional. Entradas de índice não resolvidas podem ser decodificadas apenas pelo
  layout; valide seus nomes com os bytes brutos e outras fontes.
- Uma página corrompida (torn page) é ignorada; os registros legíveis ao redor continuam disponíveis. `Truncated` marca um
  registro cuja continuação está ausente. Revise os diagnósticos do analisador antes de tratar a saída como
  completa. Uma entrada de tamanho zero é um erro; um log não gravado, todo em `FF`, legitimamente não tem linhas.
- O analisador **não reproduz** transações para reconstruir uma MFT histórica. Ele não
  recupera conteúdos arbitrários de arquivos não residentes nem executa nenhum dado coletado. Páginas DAX protegidas por CRC
  não são suportadas. A validação com evidências reais abrange principalmente páginas de log de 4 KiB.

## Analisar com o Juntei { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

Consulte a [referência de comandos](../commands/filesystem.md#parse-ntfs-logfile) para ver todas as opções. A descoberta
em diretórios aceita `$LogFile`, `LogFile`, suas formas `.bin` e nomes como `C_$LogFile` ou
`C_$LogFile.bin`, sem diferenciar maiúsculas de minúsculas. Um único arquivo passado com `-f` pode ter qualquer nome.

O pareamento automático com a MFT respeita o prefixo de volume de uma exportação: `C_$LogFile` usa `C_$MFT`, e
`D_$LogFile.bin` tenta primeiro `D_$MFT.bin` e depois `D_$MFT`. Logs com prefixo nunca usam um `$MFT` simples
como alternativa. `LogFile` sem prefixo também aceita `MFT` ou `$MFT`. Use `-m` com `-f` para selecionar
explicitamente uma MFT correspondente quando sua exportação tiver outro esquema de nomes.

A saída é um registro por LSN, incluindo registros de atualização, de checkpoint e de clientes desconhecidos. JSON é um
fluxo de objetos formatados; JSONL tem um objeto por linha. CSV tem os mesmos nomes de campos;
arrays e campos estruturados são JSON compacto dentro de uma célula CSV. Escalares desconhecidos ficam vazios no
CSV e `null` no JSON. Arrays vazios significam que nenhum item foi decodificado, não prova de que nenhum existia.

Um log inteiro e seus registros analisados são mantidos em memória de cada vez. Resolver caminhos da MFT acrescenta
memória e trabalho. `--raw-data` pode aumentar muito o tamanho da saída. No `auto-parse`, use `-T` para
limitar processos de análise simultâneos em uma máquina com pouca memória.

Se uma MFT adjacente não puder ser lida, o Juntei emite um aviso e exporta o log sem caminhos; `MftSource`
permanece então nulo. Essa falha de enriquecimento não descarta a evidência primária legível.
Correspondências de MFT ausentes ou ambíguas também produzem um aviso e uma contagem `Logs without a matching MFT`
no resumo; elas não contam como falhas de análise.

## Campos de saída { #output-fields }

### Identidade do registro, integridade e procedência { #record-identity-integrity-and-provenance }

| Campo | Significado e interpretação |
|---|---|
| `SourceFile`, `FileOffset` | Caminho de entrada e o deslocamento em bytes em que o cabeçalho deste registro foi lido; use-os para validar os bytes. |
| `LSN` | O número de sequência do log do registro. Valores grandes são inteiros, não horários. |
| `PreviousLSN`, `UndoNextLSN` | Vínculos para os registros anterior e de reversão. Zero significa sem vínculo; vínculos diferentes de zero podem apontar para fora dos dados retidos. |
| `TransactionID`, `TransactionFirstLSN` | Slot da tabela de transações e o LSN mais antigo vinculado a esta transação; o slot, por si só, não é único. |
| `RecordType` | `ClientRecord` (atualização), `ClientRestart` (checkpoint) ou `Other`. |
| `ClientIndex`, `ClientSequence` | Identidade do cliente do log; normalmente o NTFS. Não são uma conta de usuário nem um ID de processo. |
| `Flags`, `SpansPages` | Flags brutas do registro e se os dados do cliente continuam em outra página. Abranger várias páginas é normal. |
| `Truncated` | Verdadeiro se o analisador encontrou uma continuação ausente para este LSN. Não resume todos os danos do arquivo inteiro. |
| `ClusterSize`, `ClusterSizeSource` | Bytes por cluster usados para os deslocamentos do destino; a origem é `Caller`, `Checkpoint` ou `Inferred`. Forneça o valor real com `--cluster-size`, se conhecido. |
| `LogVersion` | Versão da página de reinício legível mais recente, normalmente `1.1` ou `2.0`; nulo se nenhuma página de reinício legível sobreviver. |
| `MFT-Source` | A MFT usada para o enriquecimento com o caminho atual, se presente. Ela deve pertencer ao mesmo volume. |

No JSON/JSONL do `auto-parse`, `Comp` e `Artifact` são acrescentados como nos demais analisadores. O log `.txt`
da tarefa contém os diagnósticos; os manifestos informam se a tarefa foi bem-sucedida, falhou ou foi ignorada.
Danos recuperáveis podem produzir uma tarefa bem-sucedida com uma contagem `Parse problems` diferente de zero em seu log.

### Operações e destino { #operations-and-target }

| Campo | Significado e interpretação |
|---|---|
| `RedoOperation`, `UndoOperation` | Nomes das operações NTFS, ou hexadecimal para um código desconhecido. O redo descreve a atualização pretendida; o undo descreve seu inverso. |
| `TargetEntryNumber`, `TargetSequenceNumber` | Para uma operação em um registro de arquivo: o número do registro da MFT afetado, sem número de sequência inferido. Caso contrário: a referência de arquivo proprietária do atributo aberto resolvido. Uma atualização de índice de diretório inline tem como destino o diretório, não o filho descrito por sua entrada. |
| `TargetAttributeType`, `TargetAttributeName` | Tipo numérico e nome do atributo aberto usado pela operação. Para operações em registros de arquivo, normalmente é o `$DATA` do `$MFT`. Tipos comuns: 16 (`$STANDARD_INFORMATION`), 48 (`$FILE_NAME`), 128 (`$DATA`), 144 (`$INDEX_ROOT`), 160 (`$INDEX_ALLOCATION`), 176 (`$BITMAP`). |
| `TargetPath` | Caminho atual na MFT quando a referência completa do destino é conhecida, ou componentes desconhecidos. Vazio para operações em registros de arquivo cuja geração histórica é desconhecida; um número de registro, por si só, não deve ser resolvido para seu ocupante atual. |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | Arquivo proprietário do atributo aberto por meio do qual a operação grava, com seu caminho atual na MFT. Para operações em registros de arquivo, preservam o contexto do `$MFT` separadamente do registro afetado. |
| `TargetAttribute` | Deslocamento na tabela de atributos abertos do log; é diferente do tipo do atributo ou do identificador da MFT. |
| `TargetInferred` | Verdadeiro se o mapeamento do destino vem de outro trecho do log. Um destino nulo permanece não resolvido mesmo que esta flag seja falsa. |
| `TargetVCN`, `Lcns` | Cluster virtual no destino e números de cluster físico; úteis para validação de baixo nível, e não para cronologia. |
| `RecordOffset`, `AttributeOffset` | Deslocamentos do atributo/da entrada e da atualização dentro dele. |
| `ClusterBlockOffset`, `TargetBlockSize` | Deslocamento e tamanho do destino em unidades de 512 bytes. |
| `OffsetInTarget` | Deslocamento em bytes calculado no registro de arquivo afetado ou no fluxo não residente. Depende dos tamanhos de cluster e de registro da MFT. |
| `RedoLength`, `UndoLength` | Comprimentos de dados declarados. Uma operação de zeragem pode declarar um comprimento sem carregar essa quantidade de bytes de carga útil. |
| `RedoBytesAvailable`, `UndoBytesAvailable` | Bytes de carga útil efetivamente disponíveis para decodificação. Compare com os comprimentos declarados e a semântica da operação. |

### Colunas práticas de metadados de arquivos { #convenient-file-metadata-columns }

Estas colunas mostram um nome de arquivo preferencial não DOS do lado redo, recorrendo ao undo como alternativa.
Se nenhum dos lados contiver um nome de arquivo, os campos de atualização de horários preferem os horários de redo e depois os de undo.
As atualizações de entradas de diretório também expõem diretamente seus horários, tamanho e flags, preferindo redo a
undo. Atributos `$DATA` decodificados expõem seu tamanho mesmo quando não há nome de arquivo. O tamanho do `$DATA` sem nome
de um registro de arquivo decodificado tem prioridade sobre o tamanho do nome de arquivo, possivelmente desatualizado.
Elas são um auxílio para navegação; use os campos separados de cada lado para uma comparação completa de antes/depois.

| Campo | Significado e interpretação |
|---|---|
| `MetadataSide` | `Redo` ou `Undo`, identificando o lado que fornece as colunas práticas de nome de arquivo/horários. |
| `EntryNumber`, `SequenceNumber` | Arquivo descrito pelos metadados decodificados de nome de arquivo/registro de arquivo/atributo ou por uma atualização parcial de horários do `$STANDARD_INFORMATION`, com a sequência quando conhecida. Vazio para uma atualização de entrada de diretório que não identifica o filho e para atualizações brutas de tamanho/lista de runs. Use `TargetEntryNumber` para o registro afetado. |
| `Filename` | Nome decodificado preferencial; todos os nomes permanecem nos arrays de cada lado, incluindo aliases DOS e hard links. |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | Referência do pai e caminho atual na MFT; não é uma reconstrução do caminho histórico. |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | Horários de metadados em UTC, com precisão de 100 ns. Não são horários de transação. Horários ausentes/inválidos permanecem vazios/nulos. |
| `FileSize`, `FileAttributes` | Tamanho e flags numéricas do Windows dos metadados selecionados; os valores de entradas de diretório podem estar desatualizados. Atributos `$DATA` e registros de arquivo decodificados podem fornecer um tamanho mais direto. |
| `AttrType` | Tipo numérico de um registro de atributo decodificado; 48 para uma atualização decodificada de metadados de entrada de diretório. Distinto do atributo aberto que é o destino da operação. |

### Metadados e cargas úteis completos de redo/undo { #complete-redoundo-metadata-and-payloads }

| Campo | Significado e interpretação |
|---|---|
| `RedoFileNames`, `UndoFileNames` | Arrays com todos os `$FILE_NAME` decodificados daquele lado, preservando nomes adicionados/novos versus removidos/antigos. |
| `RedoTimes`, `UndoTimes` | Horários do `$STANDARD_INFORMATION` gravados por cada lado; atualizações parciais deixam ausentes os horários inalterados em vez de adivinhá-los. |
| `RedoDecoded`, `UndoDecoded` | Informações estruturadas de registro de arquivo, atributo, entrada de índice ou entrada de diretório, identificadas por `Kind`. Nulo pode significar uma operação não suportada ou bytes insuficientes. |
| `JournalWriteOffset`, `JournalWriteLength` | Deslocamento e contagem de bytes dos dados de redo que gravam em `$DATA:$J`. Identifica uma possível fonte de registros USN que, de outra forma, estariam perdidos; não exporta um diário reconstruído. |
| `Checkpoint` | Versão do checkpoint, LSN inicial, tamanho do cluster e o LSN/comprimento dos dumps da tabela de atributos abertos, dos nomes de atributos, das páginas sujas e da tabela de transações. |
| `OtherRecordType` | Tipo bruto do registro do cliente para um corpo não reconhecido ou não decodificável. |
| `RedoData`, `UndoData`, `OtherData` | Cargas úteis em hex somente com `--raw-data`; caso contrário, nulo/vazio. Mantenha-as para validar estruturas não suportadas ou danificadas. |

Cada objeto de nome de arquivo inclui as referências do arquivo e do pai (`Raw`, `EntryNumber`,
`SequenceNumber`), `Name`, `NameEscaped`, `Namespace`, o `ParentPath` atual, quatro horários,
`Size`, `AllocatedSize`, `Attr` e `ReparseValue`. `NameEscaped` identifica UTF-16 inválido
representado com escapes. Um valor de reparse pode ser uma tag de reparse ou o tamanho de atributos estendidos.

Os objetos de horário preservam tanto `Utc` quanto o inteiro `Filetime`; um valor sentinela ou inválido pode
ter uma representação UTC nula, mantendo seu inteiro original. `RedoTimes` e `UndoTimes`
contêm `Created`, `Modified`, `Changed` e `Accessed`; os valores dos membros ausentes são nulos.

Os objetos `FileRecord` decodificados incluem número, sequência, contagem de hard links, flags, referência base,
se apenas o cabeçalho sobreviveu e os atributos presentes. Cada atributo decodificado preserva
tipo, nome, estado residente, flags, ID, tamanho e tamanho alocado. `$STANDARD_INFORMATION` acrescenta os
quatro horários, atributos, IDs de proprietário/segurança, cota e USN; esses IDs precisam de resolução separada e
não são prova do usuário que alterou o arquivo. As entradas de índice incluem flags, comprimento da chave e um
nome de arquivo opcional. As atualizações de entradas de diretório preservam os quatro horários, tamanhos, atributos e o valor
de reparse mesmo quando o próprio nome não foi registrado em log.

## Fluxo de investigação { #investigation-workflow }

1. Revise os diagnósticos em busca de páginas de reinício ausentes, páginas corrompidas e registros truncados.
2. Pesquise tanto em `RedoFileNames` quanto em `UndoFileNames` um nome de arquivo ou extensão suspeitos.
3. Faça a correspondência dos números de entrada **e de sequência** e, em seguida, examine a cadeia de LSNs e as operações da transação.
4. Compare os horários de redo e undo em busca de alterações. Cópia, extração e restauração rotineiras também
   podem alterar horários; uma diferença é uma pista, não um veredito automático de timestomping.
5. Correlacione com o [Diário USN](usn-journal.md), o [$MFT](mft.md) e o [$I30](i30.md). Use logs de
   eventos, o [Prefetch](prefetch.md) e artefatos de atividade do usuário para estabelecer execução ou atribuição.

Por exemplo, encontre todos os registros que contêm um nome de arquivo criptografado em qualquer um dos lados:

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

Esta é uma pesquisa por nome de arquivo, não um detector completo de ransomware. Para inspecionar cargas úteis do diário,
exporte com `--raw-data` e selecione os registros com `JournalWriteOffset` não nulo; o hex de `RedoData`
representa os bytes gravados nesse deslocamento. Analisar e desduplicar esses registros USN embutidos
é uma etapa separada. O Juntei ainda não os mescla com a saída de `parse-ntfs usn-journal`.

## Mais detalhes { #further-details }

- [Crate logfile do ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile): estruturas suportadas, corpus de validação e limitações conhecidas.
- [ntfslog da Sootmark](https://github.com/Sootmark/ntfslog): implementação upstream em Rust.
- [Documentação do formato NTFS da libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation): páginas de log, fixups e estruturas de metadados.
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs): pesquisa e implementação independentes de análise de NTFS.
- [LogFileParser de Joakim Schicht](https://github.com/jschicht/LogFileParser): análise detalhada do log de transações.
- [ntfs-linker da Stroz Friedberg](https://github.com/strozfriedberg/ntfs-linker): correlação de fontes NTFS no histórico de arquivos.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum): amostras reais de NTFS do Windows.
- [go-ntfs da Velocidex](https://github.com/Velocidex/go-ntfs): implementação de NTFS e volume de teste público.
