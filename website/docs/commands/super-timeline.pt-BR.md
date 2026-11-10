# Super Linha do Tempo { #super-timeline }

`super-timeline` transforma artefatos analisados ​​em eventos com um esquema comum. A saída é **não classificada por padrão**: segue a ordem do arquivo de entrada descoberto e a ordem de projeção de cada analisador, independentemente da ordem de conclusão do trabalhador. Adicione `--sort` para ordem cronológica e consolidação duplicada. Um registro Prefetch pode conter vários tempos de execução; um registro MFT pode conter oito ou mais registros de data e hora de arquivo. Cada um se torna um evento rotulado separadamente. Isto torna mais fácil comparar evidências de diferentes fontes sem perder o que cada carimbo de data/hora realmente significa.

Comece com os resultados de [`auto-parse`](auto-parse.md), em vez do diretório de evidências original:

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

Mantenha os arquivos analisados, manifestos e evidências originais. A linha do tempo é uma projeção para análise; ele não substitui esses registros nem cria um arquivo de armazenamento nativo do Plaso.

## Uso { #usage }

```bash
# Native CSV (default)
juntei super-timeline -d parsed/ -o SuperTimeline.csv

# Chronological CSV with duplicate consolidation
juntei super-timeline -d parsed/ -o SortedTimeline.csv --sort

# Native JSON Lines
juntei super-timeline -d parsed/ -o SuperTimeline.jsonl -t jsonl

# Combine native records and an existing Hayabusa JSONL timeline
juntei super-timeline -f NativeRecords.jsonl --hayabusa Hayabusa.jsonl \
  -o SuperTimeline.jsonl -t jsonl

# Timesketch import profile; also available with CSV
juntei super-timeline -d parsed/ -o Timesketch.jsonl -t jsonl --profile timesketch

# One standalone CSV needs an artifact identifier and, if unavailable, a computer name
juntei super-timeline -f MFT.csv --input-artifact mft --computer HOST01 \
  -o Host01Timeline.csv

# Reduce the time range, memory used by sorting, or number of workers
juntei super-timeline -d parsed/ -o IncidentTimeline.jsonl -t jsonl \
  --timeline-start "2026-10-20T06:00:00Z" --timeline-end "2026-10-20T14:00:00Z" \
  --sort --memory-limit 64 -T 4 --temp-dir scratch/
```

`--hayabusa FILE` pode ser repetido ou usado sozinho. Ele importa carimbos de data/hora de eventos/alertas existentes e contexto de regras sem executar Hayabusa. Eventos informativos cujo `Details` é uma sequência são suportados. `--artifact mft,prefetch` restringe a projeção aos identificadores de artefato selecionados. Os limites de tempo são inclusivos e exigem um deslocamento UTC explícito ou `Z`.

### Escolhendo uma entrada { #choosing-an-input }

| Entrada | Como é identificado |
|---|---|
| `-d DIR` | `manifest.json` fornece tarefas primárias bem-sucedidas, IDs de artefatos, computadores e arquivos de saída exatos. As tarefas secundárias `auto-parse -j` são excluídas para evitar a importação de ambas as cópias. |
| `-f FILE` | JSON/JSONL nativo usa campos `Artifact` e `Comp` integrados. As exportações não marcadas precisam de `--input-artifact`; o CSV autônomo sempre exige isso. |
| `--computer NAME` | Fallback quando o registro de entrada não possui nome de computador; caso contrário, um computador desconhecido será explicitamente `Unknown`. |
| `--hayabusa FILE` | JSONL com carimbos de data/hora Hayabusa, metadados de regras, computador e, quando presente, `EvtxFile`. |

Os manifestos atuais suportam a relocação do diretório analisado: os caminhos de saída absolutos registrados são resolvidos no novo diretório. As entradas devem permanecer dentro dele, inclusive os alvos de links simbólicos. Os diretórios legados por artefato Amcache, SRUM e Jump Lists são suportados; diretórios de categoria precisam de `OutputFiles` explícito.

Os manifestos atuais com caminhos relativos à saída e exportações simplificadas de várias tabelas são suportados. O comando reconhece cada artefato programado por análise automática, incluindo o navegador mais recente, tabelas WebCache, Outlook e UAL.

Quando um manifesto identifica uma imagem VHDX ou E01, as referências de origem incluem a imagem e o caminho interno, e o nome da imagem é retido no contexto do evento. O espaço de trabalho temporário temporário não é necessário para ler os resultados analisados.

Uma fonte bruta é obtida do registro ou de um comando manifesto inequívoco. Com `-d`, as referências de origem bruta são relativas ao diretório de evidências e `Details.ParsedFile` e o resumo `InputFiles` são relativos ao diretório analisado. Os nomes de saída/relatório resumidos são relativos ao diretório de saída da linha do tempo. SRUM As tabelas CSV são designadas a um banco de dados somente quando o host tem um SRUDB.dat coletado exclusivamente identificável. Caso contrário, `SourceFile` aponta para o arquivo analisado e `Details.RawSourceAvailable` é `false`.

Uma entrada independente é referenciada pelo seu nome de arquivo. Entradas com o mesmo nome de arquivo usam referências `./Inputs/<ID>/<filename>` distintas. Caminhos absolutos de origem bruta fora das raízes de evidências conhecidas usam identificadores `./Sources/<ID>/<filename>`. Essas referências opacas retêm distinções de origem sem exportar os nomes absolutos do diretório do analista; consulte o registro nativo usando `ParsedFile` e `InputRecord` para a fonte original. Os caminhos registrados na evidência, como os caminhos executáveis ​​Windows, permanecem inalterados.

## Campos de eventos comuns { #common-event-fields }

| Campo | Significado |
|---|---|
| `Timestamp` | UTC RFC 3339, escrito com nove dígitos fracionários para ordenação consistente. O preenchimento não adiciona precisão. |
| `Comp` | Computador associado ao registro de entrada. |
| `Artifact` | Identificador de artefato estável, como `mft`, `prefetch`, `srum` ou `hayabusa`. |
| `TimestampType` | Descrição do significado do carimbo de data/hora, como `MFT SI Data modified`. |
| `User` | Conta registrada ou SID quando o adaptador possui; pode estar vazio. |
| `Path` | Arquivo afetado, chave de registro ou outro objeto específico do artefato. |
| `Message` | Breve descrição para um visualizador de linha do tempo. |
| `SourceFile` | Referência de fonte bruta relativa à evidência ou um identificador de fonte relativo quando sua raiz é desconhecida; caso contrário, o substituto do arquivo analisado descrito acima. |
| `SourceField` | Campo analisado exato ou local aninhado usado para o carimbo de data/hora. |
| `RecordID` | Identidade de registro específica do artefato. Não é necessariamente um registro EVTX ID. |
| `Details` | Contexto estruturado, incluindo carimbo de data/hora original, precisão, base de tempo, função de tempo, arquivo analisado, registro/localização de entrada e metadados de regra/recuperação disponíveis. |

Em CSV, `Details` é uma célula JSON citada. Em JSONL é um objeto. O perfil Timesketch altera `Timestamp`, `TimestampType` e `Message` para `datetime`, `timestamp_desc` e `message`, respectivamente, mantendo os demais campos de proveniência. Esses são os [três campos de importação obrigatórios do Timesketch](https://timesketch.org/guides/user/import-from-json-csv/).

`Details` contém deliberadamente o contexto selecionado, em vez de dados binários completos ou todos os campos de registro nativos. Sequências longas, matrizes e objetos são delimitados e marcados como `ProjectionOmitted`. Procure `Details.ParsedFile`, `InputRecord` e `Location` para inspecionar o registro nativo completo. Para CSV, o número do registro conta os registros CSV após o cabeçalho, e não as linhas físicas dentro das células entre aspas. Para JSONL é o número da linha; para JSON é o número do documento de nível superior mais um local aninhado.

## O que os tempos significam { #what-the-times-mean }

Um carimbo de data/hora é uma evidência sobre um objeto registrado. Não é, por si só, prova de execução ou ataque. Os adaptadores mantêm intencionalmente estas distinções:

| Artefatos | Interpretação da linha do tempo |
|---|---|
| MFT | Modificação, acesso, alteração de metadados e tempos de criação separados de SI e FN; atributos nomeados FN permanecem distintos. Com `--sort`, os campos SI repetidos nas linhas de nome de arquivo são consolidados. Os tempos do arquivo base compartilhado das linhas ADS são excluídos em ambos os modos. Os nomes de folga do registro FILE recuperados mantêm eventos FN recuperados separados. |
| I30 | Metadados FN/índice armazenados, com contexto de folga/recuperação quando disponível. Estes não são carimbos de data/hora da gravação do índice de diretório. |
| Diário USN | Carimbo de data e hora do registro do diário e motivos de atualização. Um registro pode descrever diversas alterações; ele não registra necessariamente o horário de início preciso. |
| LogFile | Instantâneos de carimbo de data/hora de refazer/desfazer armazenados e metadados de nome de arquivo. Refazer e desfazer permanecem separados. LSNs e deslocamentos de arquivo são identificadores, nunca convertidos em datas. |
| Prefetch | Tempos de execução registrados. Com `--sort`, eles são consolidados com a exportação de linha do tempo Prefetch correspondente. Os metadados de criação de volume referenciados são rotulados separadamente. |
| Bancos de dados SQLite do navegador | Visitas registradas, metadados da última visita à página recuperada, início/fim de download, criação/acesso/expiração de cookies e primeiro/último uso ou exclusão de histórico de formulário. O término do download não prova, por si só, sucesso ou execução; a expiração de um cookie é um prazo configurado. O estado de recuperação, a confiança e a origem permanecem no contexto. |
| Internet Explorer/Edge herdado WebCache | Acesso de entrada armazenada, criação/modificação, expiração e sincronização, início de download, metadados de cookies e acesso/eliminação de contêiner. A contabilidade do cache não é automaticamente uma evidência de que um usuário abriu uma página. |
| Outlook PST, OST, MSG e OFT | Envio/entrega de mensagens; criação/modificação de pasta, item e anexo; início/fim/lembretes do calendário; início/vencimento/conclusão/lembretes de tarefas; e propriedades de tempo digitadas explicitamente. Os horários agendados não provam que ocorreu uma reunião, que um lembrete foi disparado ou que um programa foi executado. Um prazo de entrega armazenado não prova que a mensagem foi lida. |
| Registro de acesso do usuário (UAL) | Metadados do primeiro/último acesso, inserção/criação de banco de dados, primeiro/último visto e tempo de inicialização do sistema operacional armazenado. Função, conta, endereço de origem e contadores diários permanecem no contexto. As datas/contagens diárias não são inventadas para logons à meia-noite; a identidade de duração indeterminada e as linhas DNS permanecem disponíveis no relatório complementar. |
| Amcache | Últimas gravações da chave de inventário, contexto da chave de volume pai, datas de instalação registradas e metadados de link PE. A presença de inventário ou LinkDate por si só não comprova a execução. |
| ShimCache | Modificação de arquivo armazenada, metadados de atualização do cache e última gravação da chave do cache. Nenhum desses valores representa automaticamente um horário de execução. |
| SRUM | Carimbo de data/hora do provedor, início da conexão, campos explícitos de intervalo de início/término e metadados do identificador do aplicativo. Bytes, duração e outros contadores não são datas. |
| LNK | Carimbos de data e hora MAC de destino, criação de identificador de rastreamento e tempos de item de shell. Eles descrevem metadados de destino armazenados, não necessariamente quando o atalho foi aberto. |
| Jump Lists | Insira metadados de último uso, criação de identificador de rastreamento e tempos de destino LNK incorporados. O padrão omite os tempos dos arquivos contêineres do lado da coleção. |
| Quarentena Defender | Carimbo de data/hora de entrada e tempos de criação/acesso/gravação de recursos armazenados. Os tempos de extração não são evidência de atividade de endpoint. |
| BAM, UserAssist, FeatureUsage | Os campos de atividade registrados estavam presentes, com o contexto chave da última gravação mantido distinto. Um contador sem hora não cria um evento. |
| RecentDocs, MRU, Abrir/Salvar MRU, ShellBags, Escritório | Tempos de uso/confiança/item de shell registrados e contexto chave. O horário chave de uma lista não é o horário individual da última utilização de cada item da lista. |
| Segredos SAM e LSA | Logon/senha/expiração da conta registrados ou metadados de conjunto secreto e horários-chave relacionados. Eles não substituem os logs de eventos de autenticação. |
| Serviços, TaskCache, persistência de registro, BCD | Metadados de chave/configuração e tempos de tarefas registrados explicitamente. As gravações da chave de serviço por si só não comprovam a execução do serviço. |
| Redes e USB | Campos de perfil/aluguel de rede e instalação/chegada/remoção de dispositivo, com contexto de chave de registro. Os tempos MountPoints2 do usuário nativo USB retêm a origem da seção do usuário correspondente. |
| Informações do sistema, programas instalados, AppCompat, histórico de aplicativos | Campos explícitos de instalação, desligamento, inventário/atividade e metadados principais. As matrizes de perfil/versão de informações do sistema nativo mantêm seus tempos aninhados. |
| Política de auditoria, configurações de segurança, MuiCache, registro genérico | Chave os horários da última gravação quando fornecidos; valores/contadores de duração indeterminada permanecem disponíveis na entrada nativa. |
| Persistence Hunter | Última gravação da chave de suporte e horário de alteração correlacionado. Uma descoberta não é em si uma instalação com registro de data e hora ou um veredicto de comprometimento. |
| Hayabusa | Carimbo de data e hora do evento/alerta com regra, nível, canal, evento ID e contexto de detalhes disponíveis. Várias regras e linhas de correlação distintas permanecem separadas. |

A exportação padrão retém todas as projeções de carimbo de data/hora, incluindo metadados repetidos. Ele não realiza classificação global ou consolidação duplicada. Com `--sort`, as projeções duplicadas são consolidadas; isso pode produzir menos linhas do que uma exportação não classificada das mesmas entradas. Vários carimbos de data/hora iguais com diferentes significados ou fontes de evidências são retidos em ambos os modos. A identidade duplicada inclui o computador, a origem, o artefato, o registro, o significado da hora e o estado de recuperação. As linhas consolidadas registram `DuplicateCount` e caminhos alternativos/campos de origem limitados. Fontes distintas nunca são mescladas apenas porque seus caminhos e carimbos de data/hora são parecidos. Quando a fonte bruta não pode ser resolvida, a identidade do registro de entrada é mantida para evitar o colapso de diferentes capturas.

### Precisão, horários locais e suposições { #precision-local-times-and-assumptions }

Campos UTC conhecidos e carimbos de data/hora com deslocamentos explícitos são normalizados para UTC. Os valores originais e sua precisão registrada são mantidos, incluindo 100 ns FILETIMEs. Um analisador de origem ou visualizador downstream pode já ter arredondado um tempo; este comando não pode restaurar a precisão perdida.

Alguns campos shell-item/FAT possuem semântica de horário local mesmo quando um analisador upstream os serializa com `Z`. A linha do tempo preserva o valor analisado e a origem do campo; ele não pode recuperar o fuso horário do terminal ausente ou validar uma conversão upstream. Verifique o artefato bruto e as configurações do endpoint antes de usar esse tempo para obter uma correlação precisa entre hosts.

Um carimbo de data/hora ingênuo sem fuso horário definido por artefato é diagnosticado, em vez de tratado silenciosamente como UTC. Se a evidência justificar, forneça uma compensação fixa assinada:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Valores somente de data também exigem essa opção. Sua âncora da meia-noite é rotulada como uma suposição com **precisão do dia**; não é um evento observado à meia-noite. Nomes de fuso horário e regras de horário de verão não são aceitos. Valores ausentes, zero, inválidos e fora do intervalo não produzem eventos de época sintéticos. Uma data 1970 válida explicitamente registrada é retida.

Duas categorias opcionais são excluídas por padrão:

- `--include-collection-times`: LNK/Metadados do arquivo de origem da lista de atalhos no sistema de arquivos de coleta/análise. Esses eventos são rotulados como `collection_filesystem`, separados da atividade do endpoint.
- `--include-interpreted-times`: valores QWORD de registro genérico renderizados como FILETIME pelo leitor de registro. Eles são rotulados como `interpreted_value` porque muitos desses valores são, na verdade, contadores.

## Relatórios complementares { #companion-reports }

Para `SuperTimeline.csv`, o comando também escreve:

- `SuperTimeline.csv-Untimed.jsonl`: locais de registro nativo sem um carimbo de data/hora de evento mapeado utilizável.
- `SuperTimeline.csv-Diagnostics.jsonl`: horários inválidos/ambíguos, metadados compartilhados ignorados intencionalmente e limitações de fidelidade que exigem o registro original analisado.
- `SuperTimeline.csv-Summary.json`: arquivos de entrada, contagens por artefato, eventos filtrados/candidatos/consolidados, contagem de saída, trabalhadores, classificação de orçamento/partes/aprovações e tempo decorrido.

Os nomes complementares incluem o nome completo do arquivo de saída. Por exemplo, `SuperTimeline.jsonl` tem seu próprio `SuperTimeline.jsonl-Summary.json` e relatórios de diagnóstico/de duração indeterminada correspondentes. As exportações CSV e JSONL com a mesma haste podem coexistir; `-C` é necessário apenas para substituir um conjunto de resultados existente.

Datas válidas aguardando um incremento justificado de suposição de fuso horário `UnresolvedTimeZones`, em vez de `InvalidTimestamps`. Ambos permanecem rastreáveis ​​em diagnósticos; nenhum carimbo de data/hora é inventado.

Registros com horários válidos fora do intervalo solicitado são contados como filtrados e não sem tempo. As contagens de eventos candidatos precedem a consolidação duplicada. `Sorted` e `DuplicateConsolidation` registram se `--sort` foi ativado. Sem ele, `DuplicateEvents`, `InitialSortChunks` e `MergePasses` são zero e `SortMemoryBudgetMiB` é `null` porque nenhum buffer de classificação está alocado. `Complete: true` significa que todas as entradas analisadas selecionadas foram processadas com êxito; isso não significa que todos os artefatos foram coletados, todos os campos possuem carimbo de data/hora ou que todos os ataques são detectáveis.

As exportações CSV podem nivelar metadados aninhados: perfis de informações do sistema, pontos de montagem do usuário USB, itens de shell e outros contextos estruturados podem ser mais ricos em JSON/JSONL nativo. Use entrada nativa quando esses campos forem importantes. A linha do tempo não pode reconstruir dados omitidos por uma exportação CSV. IDs de artefatos não suportados e arquivos de entrada malformados causam um erro em vez de um sucesso contendo resultados parciais.

## Desempenho e segurança de saída { #performance-and-output-safety }

As entradas são transmitidas e processadas em paralelo. Por padrão, cada trabalhador grava um fragmento de evento formatado; os fragmentos são concatenados na ordem do arquivo de entrada sem classificar ou reler suas estruturas de eventos. CSV possui um cabeçalho mesmo quando vários arquivos são processados. A ordem de saída é determinística nas contagens de trabalhadores; isso não implica cronologia.

Com `--sort`, os buffers são transferidos para um diretório temporário privado e, em seguida, as passagens de mesclagem limitadas produzem uma saída cronológica. A classificação usa segundos inteiros e nanossegundos, incluindo datas anteriores a 1970. A ordenação em tempo igual é determinística nas contagens de trabalhadores e nos orçamentos de classificação. Uma barra de progresso avança para cada evento processado durante mesclagens intermediárias e finais, incluindo passagens repetidas. Ele aparece em um terminal e fica oculto quando a saída é redirecionada.

`--memory-limit` se aplica apenas a `--sort`: é o **orçamento do buffer de classificação** agregado, não um limite do processo RSS. Registros de entrada, sobrecarga de objeto/string JSON, buffers CSV e registros de mesclagem requerem memória adicional. A contagem de trabalhadores é reduzida para orçamentos pequenos. `--max-record-size` limita um registro de entrada analisado (padrão 16 MiB); uma bela matriz/relatório JSON conta como um documento de entrada de nível superior. Documentos maiores podem necessitar de um limite maior ou, preferencialmente, JSONL. A leitura antecipada do analisador CSV também contribui para esse limite.

O espaço em disco temporário deve acomodar os fragmentos de eventos formatados (padrão) ou registros de classificação expandidos e passagens de mesclagem (`--sort`), além dos resultados finais. Ambos os modos mantêm a preparação temporária de saída e a proteção contra a publicação de entradas com falha. Um registro pode produzir muitos eventos, portanto a linha do tempo pode ser muito maior que a entrada. Filtre o intervalo de tempo/artefatos quando apropriado e selecione um volume temporário com espaço livre adequado.

Ctrl+C cancela cooperativamente, remove os diretórios privados de classificação/preparo e sai com o código `130`. Em Linux e macOS isso também se aplica a SIGTERM e SIGHUP. O cancelamento é verificado durante a leitura, normalização, mesclagem e redação dos relatórios, antes do início da publicação. A terminação indetectável ou a perda de energia ainda podem deixar arquivos temporários.

As saídas existentes requerem `-C`. A linha do tempo e os complementos não devem substituir entradas, manifestos, evidências locais referenciadas ou seus diretórios de evidências protegidos. Erros de entrada ou falhas de classificação deixam as saídas existentes intactas. Os arquivos concluídos são preparados ao lado do destino e substituídos individualmente, publicando a linha do tempo por último. Isto **não** é uma transação entre todos os quatro arquivos: uma falha de E/S durante a publicação final pode fazer com que alguns relatórios complementares sejam substituídos. Mantenha um novo nome base de saída para cada execução ao reter uma exportação completa anterior.

## Mais detalhes { #further-details }

- [Timesketch CSV/JSONL importação](https://timesketch.org/guides/user/import-from-json-csv/)
- Expansão de carimbo de data/hora [Plaso e legado CSV](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Saída e formatação Plaso](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Evidências e interpretação](../overview/evidence-model.md)
- [Formatos de saída](../getting-started/output-formats.md)

As entradas Juntei atuais usam rótulos abreviados como `Comp`, `Filename`, `Args` e `Attr`. MFT CSV também aceita os atuais `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` e Cabeçalhos `HasADS`, mantendo intactas as gerações de números de sequência e as distinções DOS/ADS. Os rótulos exportados mais antigos permanecem aceitos e `SourceField` mantém a ortografia na entrada real. A entrada Hayabusa mantém seu próprio campo `Computer`. Os metadados do setor de inicialização e do descritor de segurança são retidos no relatório de duração indeterminada porque não fornecem um carimbo de data/hora do evento.

## Analisadores de múltiplas tabelas mais recentes { #newer-multi-table-parsers }

JSON/JSONL nativo carrega cada nome de tabela. Para um CSV independente, o navegador, o WebCache e os adaptadores do Outlook identificam sua tabela a partir do esquema de coluna, portanto, renomear o CSV ou usar os sufixos de origem da análise automática não altera a interpretação do carimbo de data/hora. UAL mantém seus campos de carimbo de data/hora conhecidos explícitos; contadores de acesso e textos não relacionados com datas não são adivinhados como datas.

As linhas de propriedades do Outlook são projetadas somente quando o tipo declarado é de hora ou tipo de hora múltipla. Uma string contendo uma data permanece uma string. As mensagens incorporadas mantêm seus IDs de item e referências pai, e os tempos de extração de anexos não são substituídos por carimbos de data/hora de anexos armazenados. As linhas de arquivo, destinatário, contato ou propriedade de duração indeterminada permanecem rastreáveis ​​na saída nativa e no relatório de duração indeterminada.

Os mesmos esquemas de saída CSV e JSONL se aplicam a esses analisadores; adicionar um novo artefato não adiciona centenas de colunas CSV. O contexto específico do artefato permanece em `Details`.
