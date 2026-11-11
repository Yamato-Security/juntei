# Super linha do tempo { #super-timeline }

O `super-timeline` converte artefatos analisados em eventos com um esquema comum. A saída **não é ordenada
por padrão**: ela segue a ordem em que os arquivos de entrada foram descobertos e a ordem de projeção de cada
analisador, independentemente da ordem em que os workers terminam. Adicione `--sort` para obter ordem
cronológica e consolidação de duplicatas. Um registro do Prefetch pode conter vários horários de execução; um
registro da MFT pode conter oito ou mais carimbos de data/hora de arquivo. Cada um deles vira um evento com
rótulo próprio. Assim fica mais fácil comparar evidências de fontes diferentes sem perder o que cada carimbo
de data/hora realmente significa.

Comece pelos resultados do [`auto-parse`](auto-parse.md), e não pelo diretório de evidências original:

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

Guarde os arquivos analisados, os manifestos e as evidências originais. A linha do tempo é uma projeção para
análise; ela não substitui esses registros nem cria um arquivo de armazenamento nativo do Plaso.

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

`--hayabusa FILE` pode ser repetido ou usado sozinho. Ele importa os carimbos de data/hora de eventos/alertas
já existentes e o contexto das regras, sem executar o Hayabusa. Eventos informativos cujo `Details` é uma
cadeia de caracteres também são compatíveis. `--artifact mft,prefetch` restringe a projeção aos identificadores
de artefato selecionados. Os limites de tempo são inclusivos e exigem um deslocamento UTC explícito ou `Z`.

### Escolha da entrada { #choosing-an-input }

| Entrada | Como é identificada |
|---|---|
| `-d DIR` | O `manifest.json` informa as tarefas primárias bem-sucedidas, os IDs de artefato, os computadores e os arquivos de saída exatos. As tarefas secundárias de `auto-parse -j` são excluídas para que as duas cópias não sejam importadas. |
| `-f FILE` | O JSON/JSONL nativo usa os campos `Artifact` e `Comp` embutidos nos registros. Exportações sem essa identificação precisam de `--input-artifact`; um CSV avulso sempre exige essa opção. |
| `--computer NAME` | Valor de reserva quando o registro de entrada não tem nome de computador; sem essa opção, um computador desconhecido aparece explicitamente como `Unknown`. |
| `--hayabusa FILE` | JSONL com carimbos de data/hora do Hayabusa, metadados das regras, computador e, quando presente, `EvtxFile`. |

Os manifestos atuais permitem mover o diretório de resultados analisados: os caminhos de saída absolutos
registrados são resolvidos dentro do novo diretório. As entradas precisam continuar dentro dele, inclusive os
destinos de links simbólicos. Os diretórios legados por artefato do Amcache, do SRUM e das Jump Lists são
compatíveis; diretórios de categoria precisam de `OutputFiles` explícitos.

Há suporte a manifestos atuais com caminhos relativos à saída e a exportações de várias tabelas em layout plano. O comando reconhece todos os artefatos agendados pela análise automática, incluindo as tabelas mais recentes de navegadores, WebCache, Outlook e UAL.

Quando um manifesto identifica uma imagem VHDX ou E01, as referências de origem incluem a imagem e o caminho interno, e o nome da imagem é mantido no contexto do evento. A área temporária de preparação não é necessária para ler os resultados analisados.

Uma fonte bruta é obtida do registro ou de um comando manifesto inequívoco. Com `-d`, as referências de origem bruta são relativas ao diretório de evidências e `Details.ParsedFile` são relativas ao diretório analisado. SRUM As tabelas CSV são designadas a um banco de dados somente quando o host tem um SRUDB.dat coletado exclusivamente identificável. Caso contrário, `SourceFile` aponta para o arquivo analisado e `Details.RawSourceAvailable` é `false`.

Uma entrada avulsa é referenciada pelo nome do arquivo. Entradas com o mesmo nome de arquivo recebem
referências `./Inputs/<ID>/<filename>` distintas. Caminhos absolutos de origem bruta fora das raízes de
evidências conhecidas recebem identificadores `./Sources/<ID>/<filename>`. Essas referências opacas mantêm as
origens distinguíveis sem exportar os nomes absolutos dos diretórios do analista; para chegar à fonte
original, consulte o registro nativo por meio de `ParsedFile` e `InputRecord`. Os caminhos registrados nas
evidências, como os caminhos de executáveis do Windows, permanecem inalterados.

## Campos comuns dos eventos { #common-event-fields }

| Campo | Significado |
|---|---|
| `Timestamp` | UTC no formato RFC 3339, sempre com nove dígitos fracionários para que a ordenação seja consistente. Os zeros de preenchimento não acrescentam precisão. |
| `Comp` | Computador associado ao registro de entrada. |
| `Artifact` | Identificador estável do artefato, como `mft`, `prefetch`, `srum` ou `hayabusa`. |
| `TimestampType` | Descrição do que o carimbo de data/hora significa, como `MFT SI Data modified`. |
| `User` | Conta ou SID registrado, quando o adaptador dispõe dessa informação; pode ficar vazio. |
| `Path` | Arquivo, chave do Registro ou outro objeto específico do artefato a que o evento se refere. |
| `Message` | Descrição curta para um visualizador de linha do tempo. |
| `SourceFile` | Referência à origem bruta relativa às evidências ou, quando a raiz é desconhecida, um identificador relativo de origem; caso contrário, o arquivo analisado usado como alternativa, conforme descrito acima. |
| `SourceField` | Campo analisado exato, ou localização aninhada, de onde vem o carimbo de data/hora. |
| `RecordID` | Identidade do registro, específica de cada artefato. Não é necessariamente um Record ID do EVTX. |
| `Details` | Contexto estruturado com o carimbo de data/hora original, a precisão, a base de tempo, o papel do horário, o arquivo analisado, o registro/a localização de entrada e os metadados de recuperação/regra disponíveis. |

No CSV, `Details` é uma célula JSON entre aspas. No JSONL, é um objeto. O perfil do Timesketch troca
`Timestamp`, `TimestampType` e `Message` por `datetime`, `timestamp_desc` e `message`, respectivamente, e
mantém os demais campos de procedência. Esses são os
[três campos obrigatórios na importação do Timesketch](https://timesketch.org/guides/user/import-from-json-csv/).

`Details` traz, de propósito, um contexto selecionado, e não os dados binários completos nem todos os campos
do registro nativo. Cadeias de caracteres longas, arrays e objetos têm o tamanho limitado e são marcados com
`ProjectionOmitted`. Para inspecionar o registro nativo completo, consulte `Details.ParsedFile`, `InputRecord`
e `Location`. No CSV, o número do registro conta os registros CSV depois do cabeçalho, e não as linhas físicas
dentro de células entre aspas. No JSONL, é o número da linha; no JSON, é o número do documento de nível
superior mais uma localização aninhada.

## Como interpretar os horários { #what-the-times-mean }

Um carimbo de data/hora é uma evidência sobre um objeto registrado. Por si só, ele não prova uma execução nem
um ataque. Os adaptadores preservam intencionalmente estas distinções:

| Artefatos | Interpretação na linha do tempo |
|---|---|
| MFT | Horários de modificação, acesso, alteração de metadados e criação, com SI e FN em eventos separados; cada atributo FN nomeado permanece distinto. Com `--sort`, os campos SI repetidos nas linhas de cada nome de arquivo são consolidados. Nas linhas de ADS, os horários compartilhados com o arquivo base são excluídos nos dois modos. Os nomes recuperados do slack de registros FILE geram eventos próprios de FN recuperado. |
| I30 | Metadados FN/de índice armazenados, com contexto de slack/recuperação quando disponível. Não são carimbos de data/hora da gravação no índice do diretório. |
| Diário USN | Carimbo de data/hora do registro do diário e motivos da atualização. Um registro pode descrever várias alterações; ele não registra necessariamente o momento exato em que elas começaram. |
| LogFile | Instantâneos de carimbos de data/hora armazenados nos dados de redo/undo e metadados de nome de arquivo. Redo e undo permanecem separados. LSNs e deslocamentos de arquivo são identificadores e nunca são convertidos em datas. |
| Prefetch | Horários de execução registrados. Com `--sort`, eles são consolidados com os da exportação correspondente da linha do tempo do Prefetch. Os metadados de criação do volume referenciado recebem um rótulo separado. |
| Bancos de dados SQLite de navegadores | Visitas registradas, metadados de última visita de páginas recuperadas, início/fim de downloads, criação/acesso/expiração de cookies e primeiro/último uso no histórico de formulários ou marcas de exclusão. O fim de um download não prova, por si só, sucesso nem execução; a expiração de um cookie é um prazo configurado. O estado da recuperação, a confiança e a origem permanecem no contexto. |
| WebCache do Internet Explorer e do Edge legado | Acesso, criação/modificação, expiração e sincronização das entradas armazenadas, início de downloads, metadados de cookies e acesso/limpeza de contêineres. O controle interno do cache não é, automaticamente, evidência de que um usuário abriu uma página. |
| PST, OST, MSG e OFT do Outlook | Envio/entrega de mensagens; criação/modificação de pastas, itens e anexos; início/fim/lembretes de calendário; início/vencimento/conclusão/lembretes de tarefas; e propriedades declaradas explicitamente com tipo de horário. Horários agendados não provam que uma reunião aconteceu, que um lembrete disparou nem que um programa foi executado. Um horário de entrega armazenado não prova que a mensagem foi lida. |
| Registro de acesso do usuário (UAL) | Metadados do primeiro/último acesso, inserção/criação de banco de dados, primeiro/último visto e tempo de inicialização do sistema operacional armazenado. Função, conta, endereço de origem e contadores diários permanecem no contexto. As datas/contagens diárias não são inventadas para logons à meia-noite; a identidade de duração indeterminada e as linhas DNS permanecem nos resultados originais analisados. |
| Amcache | Últimas gravações das chaves de inventário, contexto da chave do volume pai, datas de instalação registradas e metadados de link do PE. A presença no inventário ou o LinkDate, por si sós, não provam execução. |
| ShimCache | Modificação de arquivo armazenada, metadados de atualização do cache e última gravação da chave do cache. Nenhum deles é, automaticamente, um carimbo de data/hora de execução. |
| SRUM | Carimbo de data/hora/intervalo do provedor, início da conexão, campos explícitos de intervalo de início/fim e metadados do identificador do aplicativo. Bytes, duração e outros contadores não são datas. |
| LNK | Carimbos de data/hora MAC (modificação, acesso e criação) do destino, criação do identificador de rastreamento e horários dos itens de shell. Eles descrevem os metadados armazenados sobre o destino, e não necessariamente quando o atalho foi aberto. |
| Jump Lists | Metadados de último uso das entradas, criação do identificador de rastreamento e horários do destino dos LNK embutidos. Por padrão, os horários do próprio arquivo contêiner no lado da coleta são omitidos. |
| Lixeira | Quando cada item foi movido para a Lixeira, com o usuário da lixeira, o caminho original e se o conteúdo excluído foi coletado. Não é quando a Lixeira foi esvaziada, nem os horários de arquivo do próprio item. |
| Quarentena do Defender | Carimbo de data/hora da entrada e horários armazenados de criação/acesso/gravação do recurso. Os horários de extração não são evidência de atividade no endpoint. |
| BAM, UserAssist, FeatureUsage | Campos de atividade registrados, quando existem, com o contexto da última gravação da chave mantido à parte. Um contador sem horário não gera evento. |
| RecentDocs, MRU, Open/Save MRU, ShellBags, Office | Horários registrados de uso/confiança/itens de shell e o contexto da chave. O horário da chave de uma lista não é o horário de último uso de cada item da lista. |
| SAM e segredos LSA | Metadados registrados de logon/senha/expiração das contas ou de definição dos segredos, e os horários das chaves relacionadas. Eles não substituem os logs de eventos de autenticação. |
| Serviços, TaskCache, persistência no Registro, BCD | Metadados de chave/configuração e horários de tarefas registrados explicitamente. Gravações em chaves de serviço, por si sós, não provam a execução do serviço. |
| Redes e USB | Campos de perfil/concessão de rede e de instalação/chegada/remoção de dispositivos, com o contexto da chave do Registro. Nos dados nativos de USB, os horários de MountPoints2 dos usuários mantêm como origem o hive do usuário correspondente. |
| Informações do sistema, programas instalados, AppCompat, histórico de aplicativos | Campos explícitos de instalação, desligamento, inventário/atividade e metadados de chave. Nos dados nativos das Informações do sistema, os arrays de perfis/versões mantêm seus horários aninhados. |
| Política de auditoria, configurações de segurança, MuiCache, Registro genérico | Horários de última gravação das chaves, quando fornecidos; valores/contadores sem horário continuam disponíveis na entrada nativa. |
| Persistence Hunter | Última gravação da chave que sustenta a constatação e horário da alteração correlacionada. Uma constatação não é, por si só, uma instalação com data e hora nem um veredicto de comprometimento. |
| Hayabusa | Carimbo de data/hora do evento/alerta com regra, nível, canal, ID do evento e o contexto de detalhes disponível. Várias regras e linhas de correlação distintas permanecem separadas. |

A exportação padrão mantém todas as projeções de carimbos de data/hora, incluindo metadados repetidos. Ela não
faz ordenação global nem consolidação de duplicatas. Com `--sort`, as projeções duplicadas são consolidadas;
isso pode gerar menos linhas do que uma exportação não ordenada das mesmas entradas. Carimbos de data/hora
iguais com significados ou fontes de evidência diferentes são mantidos nos dois modos. A identidade usada para
reconhecer duplicatas inclui o computador, a origem, o artefato, o registro, o significado do horário e o
estado de recuperação. As linhas consolidadas registram `DuplicateCount` e uma quantidade limitada de
caminhos/campos de origem alternativos. Fontes distintas nunca são mescladas só porque seus caminhos e
carimbos de data/hora parecem iguais. Quando a origem bruta não pode ser resolvida, a identidade do registro
de entrada é mantida para não fundir capturas diferentes.

### Precisão, horários locais e suposições { #precision-local-times-and-assumptions }

Campos sabidamente em UTC e carimbos de data/hora com deslocamento explícito são normalizados para UTC. Os
valores originais e a precisão registrada são mantidos, incluindo FILETIMEs de 100 ns. O analisador de origem
ou um visualizador posterior pode já ter arredondado um horário; este comando não consegue restaurar a
precisão perdida.

Alguns campos de itens de shell/FAT representam horário local, mesmo quando o analisador que os produziu os
serializa com `Z`. A linha do tempo preserva o valor analisado e a origem do campo; ela não consegue recuperar
o fuso horário do endpoint, que não foi registrado, nem validar uma conversão feita nessa etapa anterior.
Verifique o artefato bruto e as configurações do endpoint antes de usar esse horário em uma correlação
precisa entre hosts.

Um carimbo de data/hora ingênuo sem fuso horário definido por artefato é contado como não resolvido, em vez de ser tratado silenciosamente como UTC. Se a evidência justificar, forneça uma compensação fixa assinada:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Valores só com data também exigem essa opção. A meia-noite usada como âncora é rotulada como suposição com
**precisão de dia**; não é um evento observado à meia-noite. Nomes de fusos horários e regras de horário de
verão não são aceitos. Valores ausentes, zerados, inválidos ou fora do intervalo não geram eventos sintéticos
na data do epoch. Uma data válida de 1970 registrada explicitamente é mantida.

Duas categorias opcionais ficam de fora por padrão:

- `--include-collection-times`: metadados dos arquivos de origem LNK/Jump Lists no sistema de arquivos de
  coleta/análise. Esses eventos recebem o rótulo `collection_filesystem`, separados da atividade do endpoint.
- `--include-interpreted-times`: valores QWORD genéricos do Registro que o leitor do Registro exibe como
  FILETIME. Eles recebem o rótulo `interpreted_value` porque muitos desses valores são, na verdade, contadores.

## Registros de saída e ignorados { #output-and-skipped-records }

O comando grava **apenas a linha do tempo** selecionada com `-o`, no formato escolhido com `-t` (`csv` por padrão, ou `jsonl`). Ele não cria arquivos complementares de diagnóstico ou resumo de duração indeterminada, nem spools de diagnóstico por registro. Os arquivos complementares existentes de execuções anteriores permanecem intactos.

O resumo do terminal informa o número de eventos da linha do tempo e registros de duração indeterminada. O carimbo de data/hora inválido diferente de zero e as contagens de fuso horário não resolvido são mostrados separadamente. Com `--sort`, também informa o número de duplicatas consolidadas. Mantenha os resultados originais analisados ​​para registros que não produzem um evento de linha do tempo e para o contexto completo do artefato.

As datas válidas que aguardam uma suposição justificada de fuso horário contam como fusos horários não resolvidos, em vez de carimbos de data e hora inválidos. Nenhum carimbo de data e hora é inventado e registros sem um tempo mapeado utilizável não são inseridos na linha do tempo como eventos de época sintéticos.

Registros com horários válidos fora do intervalo solicitado são contados como filtrados e não sem tempo. Uma exportação bem-sucedida significa que as entradas analisadas selecionadas foram processadas com êxito; isso não significa que todos os artefatos foram coletados, que todos os campos têm um carimbo de data/hora ou que todos os ataques são detectáveis.

As exportações CSV podem achatar metadados aninhados: perfis das Informações do sistema, pontos de montagem de
usuários do USB, itens de shell e outros contextos estruturados podem ser mais completos no JSON/JSONL nativo.
Use a entrada nativa quando esses campos forem importantes. A linha do tempo não consegue reconstruir dados
que uma exportação CSV omitiu. IDs de artefato não suportados e arquivos de entrada malformados causam um
erro, em vez de um sucesso com resultados parciais.

## Desempenho e segurança da saída { #performance-and-output-safety }

As entradas são lidas em streaming e processadas em paralelo. Por padrão, cada worker grava um fragmento de
eventos já formatados; os fragmentos são concatenados na ordem dos arquivos de entrada, sem ordenar nem reler
suas estruturas de eventos. O CSV tem um único cabeçalho mesmo quando vários arquivos são processados. A ordem
da saída é determinística, seja qual for o número de workers; ela não implica cronologia.

`Input files` mostra o tamanho combinado das entradas analisadas selecionadas e dos cronogramas Hayabusa. `Results saved` mostra o tamanho do arquivo de linha do tempo salvo. Ambos usam as mesmas unidades `KiB`/`MiB`/`GiB` legíveis por humanos que `auto-parse`.

Com `--sort`, os buffers são descarregados em um diretório temporário privado e, em seguida, passagens de
mesclagem limitadas produzem a saída cronológica. A ordenação usa segundos inteiros completos e nanossegundos,
incluindo datas anteriores a 1970. A ordem entre horários iguais é determinística, independentemente do número
de workers e do orçamento de ordenação. Uma barra de progresso avança a cada evento processado durante as
mesclagens intermediárias e finais, incluindo passagens repetidas. Ela aparece em um terminal e fica oculta
quando a saída é redirecionada.

`--memory-limit` só se aplica com `--sort`: é o **orçamento total dos buffers de ordenação**, e não um limite
de RSS do processo. Registros de entrada, a sobrecarga de objetos/cadeias de caracteres JSON, buffers CSV e
registros de mesclagem exigem memória adicional. Com orçamentos pequenos, o número de workers é reduzido. `--max-record-size` limita o tamanho de um registro de entrada analisado (padrão: 16 MiB);
um array/relatório JSON formatado (pretty) conta como um único documento de entrada de nível superior.
Documentos maiores podem exigir um limite mais alto ou, de preferência, JSONL. A leitura antecipada do
analisador CSV também conta para esse limite.

O espaço temporário em disco precisa comportar os fragmentos de eventos formatados (padrão) ou os registros de
ordenação expandidos e as passagens de mesclagem (`--sort`), além dos resultados finais. Os dois modos mantêm
a preparação temporária da saída e a proteção que impede publicar resultados de entradas que falharam. Um
registro pode gerar muitos eventos, então a linha do tempo pode ficar muito maior do que a entrada. Filtre o
intervalo de tempo/os artefatos quando fizer sentido e escolha um volume temporário com espaço livre
suficiente.

Ctrl+C cancela cooperativamente, remove os diretórios privados de classificação/preparo e sai com o código `130`. Em Linux e macOS isso também se aplica a SIGTERM e SIGHUP. O cancelamento é verificado durante a leitura, normalização, mesclagem e gravação da linha do tempo, antes do início da publicação. A terminação indetectável ou a perda de energia ainda podem deixar arquivos temporários.

As saídas existentes requerem `-C`. A linha do tempo não deve substituir entradas, manifestos, evidências locais referenciadas ou seus diretórios de evidências protegidos. Erros de entrada ou falhas de classificação deixam a linha do tempo existente intacta. A linha do tempo concluída é encenada ao lado de seu destino e a substitui atomicamente. Use um novo nome de arquivo de saída para cada execução ao reter uma exportação completa anterior.

## Mais detalhes { #further-details }

- [Importação de CSV/JSONL no Timesketch](https://timesketch.org/guides/user/import-from-json-csv/)
- [Expansão de carimbos de data/hora e CSV legado do Plaso](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Saída e formatação do Plaso](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Evidências e interpretação](../overview/evidence-model.md)
- [Formatos de saída](../getting-started/output-formats.md)

As entradas Juntei atuais usam rótulos abreviados como `Comp`, `Filename`, `Args` e `Attr`. MFT CSV também aceita os atuais `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` e Cabeçalhos `HasADS`, mantendo intactas as gerações de números de sequência e as distinções DOS/ADS. Os rótulos exportados mais antigos permanecem aceitos e `SourceField` mantém a ortografia na entrada real. A entrada Hayabusa mantém seu próprio campo `Computer`. Os metadados do setor de inicialização e do descritor de segurança sem registros de data e hora do evento permanecem nos resultados originais analisados; ele não cria eventos na linha do tempo.

## Analisadores mais recentes com várias tabelas { #newer-multi-table-parsers }

O JSON/JSONL nativo traz o nome de cada tabela. Para um CSV avulso, os adaptadores de navegadores, WebCache e Outlook identificam a tabela pelo esquema de colunas, de modo que renomear o CSV ou usar os sufixos de origem da análise automática não muda a interpretação dos carimbos de data/hora. O UAL mantém seus campos de carimbo de data/hora conhecidos e explícitos; contadores de acesso e textos não relacionados que pareçam datas não são interpretados como datas por suposição.

As linhas de propriedades do Outlook só são projetadas quando o tipo declarado é um tipo de horário, simples ou multivalorado. Uma cadeia de caracteres que contém uma data continua sendo uma cadeia de caracteres. As mensagens incorporadas mantêm seus IDs de item e as referências ao item pai, e os horários de extração dos anexos não substituem os carimbos de data/hora armazenados dos anexos. As linhas de arquivos, destinatários, contatos ou propriedades sem horário continuam rastreáveis na saída nativa e no relatório de registros sem horário.

Os mesmos esquemas de saída CSV e JSONL valem para esses analisadores; adicionar um novo artefato não acrescenta centenas de colunas ao CSV. O contexto específico de cada artefato fica em `Details`.
