# Formatos de saída { #output-formats }

## Resumos no terminal { #terminal-summaries }

Sem `-o` ou um `-t` estruturado, os comandos geralmente mostram contagens, intervalos de tempo, danos e constatações
de alto valor. Os resumos ajudam você a entender o escopo antes de gerar uma saída grande.

## CSV { #csv }

CSV é o formato de gravação padrão. Ele é conveniente para planilhas, importação em SQL e visualizadores
de linha do tempo. Alguns analisadores criam várias tabelas porque seus artefatos contêm tipos de registro
diferentes (Amcache, Jump Lists e SRUM).

!!! warning "Trate as colunas de evidências como texto"
    Um caminho ou uma linha de comando pode começar com `=`, `+`, `-` ou `@`. Um software de planilhas pode interpretá-lo
    como fórmula. Importe todos os campos como texto ou use um visualizador de CSV seguro para uso forense. O Juntei preserva
    as cadeias de caracteres das evidências em vez de alterá-las.

## Nomes de campos { #field-names }

Os cabeçalhos das colunas CSV e os nomes dos campos JSON/JSONL usam PascalCase, incluindo o Amcache e os
manifestos do `auto-parse`. As siglas permanecem em maiúsculas: `ProcessID`, `TimestampUTC`, `UserSID`,
`SHA256`. Uma palavra que segue uma sigla é separada por hífen: `IP-Address`, `MFT-Entry`,
`USN-Journal`. Com `jq`, coloque entre aspas um campo com hífen: `.["IP-Address"]`.

O CSV padrão do MFT usa títulos mais curtos e legíveis, como `EntryNum`, `Ext`, `Dir`, `HasADS`,
`Size`, `Created (SI)` e `Created (FN)`. Consulte a [referência de campos do MFT](../artifacts/mft.md)
para ver seu esquema CSV completo. Os pares de carimbos de data/hora seguem a ordem MACB: DataModified, Accessed,
MetadataChanged, Created, com SI antes de FN. Coloque entre aspas, nas ferramentas de CSV, os títulos que contêm espaços ou pontuação.

Os valores das evidências mantêm sua grafia original. As chaves de dicionário fornecidas pela origem, como os nomes de
valores do Registro em `Details`/`DataRaw` e os nomes do `Zone.Identifier` em `ZoneIdentifier`, também
permanecem inalteradas. Os dicionários `ValueState` e `Raw` do SRUM usam os mesmos rótulos de suas colunas
de dados. Uma colisão de normalização é relatada, em vez de um campo ser descartado silenciosamente.

Isso altera os esquemas de exportação anteriores. Gere novamente os resultados existentes e atualize as consultas salvas
(por exemplo, `.record_type` passa a ser `.RecordType`, e `full_path` do Amcache passa a ser `FullPath`).
Os relatórios em texto legível e os formatos de entrada/configuração mantêm suas convenções existentes.

## Nomes de campos abreviados { #abbreviated-field-names }

Os rótulos a seguir se aplicam às exportações CSV, JSON e JSONL, incluindo a análise automática e os
manifestos. Atualize os filtros salvos de `jq`, `xsv` e planilhas ao gerar novamente resultados antigos.
Apenas os rótulos de saída mudam; os valores das evidências e os nomes brutos dos valores do Registro mantêm sua grafia original.

| Rótulo anterior | Rótulo atual |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

A saída CSV da MFT usa colunas específicas: `File` (Filename), `SN` (Sequence Number), `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number), `USN` (Update Sequence Number), `LogfileSN` (Logfile Sequence Number), `SID` (Security ID), `Type` (Name Type) e `RefCnt` (Reference Count). Isso também se aplica aos CSVs da MFT de `auto-parse`; consulte os [campos de saída da MFT](../artifacts/mft.md#csv-output-fields).

O horário nativo no JSON de informações do sistema é `CompNameLastWritten` (CSV:
`CompNameLastWrittenUTC`). `HasADS` é um indicador booleano; `Dir` no Prefetch contém a
lista de diretórios registrada. `User` mantém seu tipo específico de cada artefato: uma cadeia de caracteres para um nome de conta,
ou a lista de usuários associados na saída de USB. Ele não implica uma identidade em execução.
Coloque a pontuação entre aspas nas consultas: `."SI<FN"` e `.["ZeroU-Secs"]`.

## JSON { #json }

O JSON armazena o resultado completo como um único documento. Ele preserva arrays, estruturas aninhadas, nulos,
inteiros grandes e distinções que um CSV plano nem sempre consegue representar.

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines { #json-lines }

O JSONL grava um objeto JSON independente por linha. Ele funciona bem em streaming e se integra naturalmente ao `jq`:

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

Quando a saída estruturada vai para a saída padrão, o Juntei envia o progresso e as mensagens para a saída
de erro padrão, de modo que um pipe receba apenas dados.

`auto-parse -j timeline.jsonl` cria um conjunto de dados JSONL adicional para uma coleta completa de vários hosts,
mantendo os resultados CSV categorizados. Adicione `-t json` ou `-t jsonl` para alterar o
formato dos resultados individuais. Cada registro tem os campos `Comp` e `Artifact`, para que sua origem continue clara
após a ingestão no SIEM.

## UTC e precisão { #utc-and-precision }

O Juntei formata os horários analisados em UTC usando o RFC 3339 quando o formato permite. Alguns artefatos têm
precisão de 100 nanossegundos; outros armazenam segundos, horário FAT, intervalos de agregação ou nenhum horário.
Dígitos não são exatidão: sempre entenda o campo de origem descrito na página do respectivo artefato.

O UTC usa a forma dos logs de eventos do Windows `2026-10-20T08:46:38.2001668Z`, mantendo a precisão
fracionária disponível. `auto-parse -Z "+09:00"` exibe os instantes analisados com um deslocamento fixo
explícito em todos os seus formatos de resultado e em sua exportação `-j` opcional. O padrão é UTC, independentemente
do computador que executa o Juntei. Os valores brutos e os horários de relógio sem fuso permanecem como foram registrados; consulte
[fusos horários da saída](../commands/auto-parse.md#output-time-zone).

## Danos e valores ausentes { #damage-and-missing-values }

Vazio, `null`, `NA`, `Problems` e os campos de estado específicos de cada analisador têm significado. Eles podem indicar que o
artefato não armazenou um valor, que o registro estava danificado, que o formato não oferece suporte a ele ou que o
analisador não conseguiu resolvê-lo. Não substitua todos os valores vazios por zero.

## Linha do tempo normalizada { #normalized-timeline }

O [`super-timeline`](../commands/super-timeline.md) cria um esquema comum de eventos em CSV/JSONL, com rótulos
que indicam o significado de cada carimbo de data/hora e referências à origem. Por padrão, a saída não é
ordenada; `--sort` ativa a ordenação cronológica e a consolidação de duplicatas. Ele também oferece
um perfil de importação para o Timesketch. As exportações nativas dos analisadores e o `auto-parse -j` mantêm seus esquemas atuais.
