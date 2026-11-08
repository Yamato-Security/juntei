# Análise automática de coletas { #automatic-collection-parsing }

O `auto-parse` é a maneira mais rápida de transformar uma coleta de triagem do Windows com vários computadores em um
diretório de análise estruturado. Informe a raiz das evidências e um novo diretório de saída. Ele descobre
os sistemas coletados, identifica seus nomes de computador e executa em paralelo todos os analisadores compatíveis do
Juntei.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

A saída padrão é CSV. Adicione uma linha do tempo JSONL para `jq` ou para importação em SIEM, mantendo esses resultados
CSV:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "Mantenha as árvores de saída e de evidências separadas"
    O Juntei recusa um diretório de saída que esteja dentro de `<EVIDENCE>` ou que contenha `<EVIDENCE>`. Isso
    impede que os dados gerados sejam redescobertos como evidência e impede que `--clobber`
    toque em arquivos da coleta de origem.

## Imagens de disco VHDX { #vhdx-disk-images }

Com `--vhdx`, `auto-parse` analisa os sistemas em cada imagem de disco VHDX do diretório de evidências. Cada imagem é montada somente para leitura com FUSE durante a análise, sem extrair arquivos. Consulte [Montar imagens de disco](mount.md).

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

Se um volume NTFS não puder ser aberto, o Juntei informa o problema e mantém os volumes legíveis. Para imagens parcialmente legíveis, `auto-parse --vhdx` retorna o código 1; `mount` também retorna 1 após desmontar.

## Fuso horário da saída { #output-time-zone }

Por padrão, os carimbos de data/hora analisados ficam em UTC, independentemente do fuso horário local do computador do analista. O UTC
usa a forma dos logs de eventos do Windows `2026-10-20T08:46:38.2001668Z`: uma data/hora ISO 8601, `T`
entre a data e a hora, e `Z` para UTC. Os dígitos fracionários disponíveis são preservados.
O `GeneratedUTC` do `manifest.json` também usa essa forma. O `Start time` e o `End time` do console
estão no horário local do computador do analista, como `2026-11-04 09:15:02`, qualquer que seja o fuso horário
da saída.

Escolha um deslocamento fixo de exibição com `-Z, --timezone`:

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

Um deslocamento é um valor `HH:MM` com sinal e dois dígitos (horas `00–23`, minutos `00–59`). Nomes como
`JST`, valores abreviados como `+9` e intervalos inválidos produzem um erro com um exemplo de sintaxe.
Omita a opção ou use `--timezone "+00:00"` para UTC. Um deslocamento fixo não aplica as regras de horário
de verão.

A opção se aplica aos carimbos de data/hora analisados nos resultados CSV, JSON e JSONL de cada artefato e
à exportação JSONL adicional de `-j`. Por exemplo, `2026-10-20T08:46:38.2001668Z` passa a ser
`2026-10-20T17:46:38.2001668+09:00`; ambos descrevem o mesmo instante.

Os nomes dos campos permanecem estáveis, incluindo os nomes existentes terminados em `UTC`. Os valores convertidos trazem
explicitamente o deslocamento selecionado. O `OutputTimeZone` de ambos os manifestos registra o deslocamento de exibição
solicitado, enquanto `GeneratedUTC` continua sendo metadado da execução em UTC. Os logs de cada analisador mantêm os resumos nativos em UTC dos analisadores.

Números brutos de FILETIME/epoch, dicionários de valores do Registro, a configuração de fuso horário da origem, metadados
de texto, valores apenas de data e valores de relógio local sem deslocamento conhecido são preservados. O Juntei
não adiciona `Z` a um horário local desconhecido. Os relógios de itens de shell derivados de FAT (`Item*UTC` em
ShellBags/OpenSaveMRU e `TargetItems` de LNK/Jump Lists) não têm deslocamento registrado; seus rótulos legados
UTC ou o sufixo `Z` exibido não provam que estejam em UTC, por isso a opção de deslocamento os preserva.
As colunas CSV que contêm resumos em texto livre mantêm
o texto registrado; JSON/JSONL preserva os carimbos de data/hora analisados aninhados como campos estruturados.

Em UTC, a saída existente do analisador é usada diretamente. Um deslocamento explícito diferente de zero adiciona uma passagem de
formatação em streaming dentro de cada worker do analisador; ela altera os valores exibidos sem alterar as evidências.

Se um deslocamento levar um carimbo de data/hora para fora dos anos `0000–9999`, esse valor mantém seu deslocamento
explícito original. Valores com mais de nove dígitos fracionários também permanecem inalterados, em vez de
serem arredondados. O Juntei registra um aviso no arquivo `*-logs.txt` do analisador e converte os demais
carimbos de data/hora. Um CSV/JSON malformado ou um erro de saída ainda faz a tarefa falhar e deixa inalterado o
arquivo de resultado original afetado.

## O que ele descobre { #what-it-discovers }

O Juntei procura recursivamente diretórios chamados `Windows` que contenham um diretório `System32`, `Prefetch`
ou `AppCompat`, sem diferenciar maiúsculas de minúsculas. O reconhecimento dos dois últimos também dá suporte a coletas
direcionadas que contêm apenas Prefetch ou Amcache. Isso exclui diretórios de perfis de usuário como
`AppData\Roaming\Microsoft\Windows`. Instalações legadas aninhadas, como
`C:\Windows.old\Windows`, são atribuídas ao volume coletado externo, em vez de se tornarem computadores
duplicados. A profundidade máxima padrão é 10. Isso abrange caminhos comuns do Velociraptor, como:

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

Altere o limite somente quando a coleta estiver aninhada em um nível mais profundo:

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

Para cada sistema descoberto, o nome do computador é escolhido nesta ordem:

1. `hostname` no `client_info.json` do ancestral mais próximo (com seu valor mantido como `FQDN` no manifesto).
2. `ComputerName` em um hive `SYSTEM` do Registro coletado.
3. O nome útil mais próximo de um diretório da coleta, ignorando a unidade e a estrutura `uploads/auto` do
   Velociraptor e removendo um sufixo como `_F.…`.

Os nomes são ajustados para uso seguro como nomes de diretório. Nomes duplicados recebem um sufixo numérico tanto no
diretório de saída quanto no valor `Comp`, para que sistemas distintos continuem distinguíveis no JSON.
O manifesto mantém o nome do host exatamente como foi descoberto em um campo `Hostname` separado.

Depois de listar os sistemas, o `auto-parse` exibe uma linha `Timezone:` logo acima de
`Parser tasks`. Ela mostra o fuso horário de saída selecionado: `UTC` por padrão, ou um valor como
`UTC+09:00` quando `--timezone "+09:00"` é usado. O rótulo é verde e o valor usa a
cor de texto padrão do terminal, como os outros valores do resumo. `-K, --no-color` mantém a linha sem cores.
A configuração de fuso horário dos sistemas coletados está disponível nos resultados de
[Informações do sistema](../artifacts/system-info.md).

## Analisadores que ele executa { #parsers-it-runs }

A falha de um analisador não interrompe os outros. O `auto-parse` executa estas tarefas para cada computador
descoberto. As tarefas usam os mesmos comandos agrupados por origem da análise manual, como
`parse-ntfs mft`, `parse-registry amcache`, `parse-ese srum` e `parse-file prefetch`:

- Amcache
- `$MFT`
- Registros de redo/undo do NTFS no `$LogFile`
- Registros do Prefetch e uma linha do tempo de execução do Prefetch separada
- Diário USN
- Índices de diretórios `$I30`
- Setores de inicialização `$Boot` e descritores de segurança `$Secure:$SDS`
- SRUM
- Registro de acesso do usuário (UAL)
- Histórico, downloads, cookies e histórico de formulários de navegadores web (navegadores baseados no Chromium e no Firefox)
- Atalhos LNK
- Jump Lists
- ShimCache
- Persistence Hunter (131 verificações de persistência entre fontes, com perfis de permissão/bloqueio)
- Metadados da quarentena do Defender
- Histórico das caixas de diálogo Abrir e Salvar como
- todas as chaves e valores do Registro, incluindo dados excluídos recuperáveis
- serviços, RecentDocs, TaskCache, contas SAM, locais de inicialização automática do Registro, UserAssist, BAM,
  programas instalados, MuiCache, ShellBags e dispositivos USB
- informações do sistema: nomes, versão do Windows, fuso horário, último desligamento, perfis e SIDs
- a política de auditoria: quais eventos de segurança o sistema estava configurado para registrar
- configuração de inicialização (BCD e BCD-Template)
- perfis de rede, interfaces, compartilhamentos, proxies de porta e rotas
- nomes e horários dos segredos LSA, e logons de domínio em cache em uso (nada é descriptografado)

O Persistence Hunter mantém todas as severidades (`--min-severity info`), incluindo correspondências parciais com a lista de permissões.
`LOW` é uma classificação a ser investigada; a ausência de metadados de assinatura também pode produzi-la para comandos
maliciosos. O inventário automático não descarta essas entradas usando o limite padrão médio do
comando manual.

O hunter usa um worker dentro de cada processo de analisador automático. Isso evita que `auto-parse -T <N>`
multiplique seu orçamento de processos por outro pool do tamanho do número de CPUs em cada processo filho. O
`persistence-hunter -T <N>` independente controla as próprias threads de worker do hunter.

A saída CSV do `$MFT` inclui todos os carimbos de data/hora e os nomes curtos. A saída do Registro inclui os dados brutos e uma
interpretação como FILETIME dos valores `REG_QWORD`. A saída JSON e JSONL do SRUM inclui metadados de estado
dos valores. Os arquivos do Defender e os fluxos residentes do `$MFT` **não são extraídos** automaticamente.

Com `--mft-slack`, a saída do `$MFT` também inclui os nomes de arquivo que restaram no slack dos registros FILE, como [`parse-ntfs mft -x`](../artifacts/mft.md) os grava: linhas marcadas com `FromSlack` em CSV e `SlackFileNames` em JSON e JSONL. `--code-page` se aplica a arquivos LNK, Jump Lists, ShellBags e OpenSaveMRU.

Um artefato que não foi coletado gera uma tarefa `skipped` normal, não uma falha.

## Layout da saída { #output-layout }

Os resultados são organizados por computador e por categoria investigativa. Os diretórios de categoria usam
PascalCase, como `Accounts` e `FileAndFolderOpening`. As tabelas do Amcache e das Jump Lists ficam
diretamente em suas categorias, com prefixos nos nomes de arquivo. Os nomes dos diretórios de computador mantêm a grafia descoberta. Os diretórios de saída
existentes não são migrados; escolha um novo diretório de saída ao deixar a nomenclatura antiga. Cada analisador grava um arquivo
`<artifact>-logs.txt` ao lado de seus resultados, inclusive quando o artefato não foi coletado.
O CSV mantém tabelas separadas porque cada artefato tem colunas diferentes:

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── BrowserActivity/
│   │   ├── Browser-Cookies.csv
│   │   ├── Browser-Downloads.csv
│   │   ├── Browser-FormHistory.csv
│   │   ├── Browser-History.csv
│   │   └── Browser-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| Categoria | Artefatos |
|---|---|
| `Accounts` | Contas locais SAM, nomes e horários dos segredos LSA |
| `Applications` | Histórico de aplicativos (vestígios de ferramentas de terceiros) |
| `Applications` | Programas instalados |
| `BrowserActivity` | Histórico, downloads, cookies e histórico de formulários de navegadores web, com as entradas excluídas recuperadas |
| `ExternalDevices` | Dispositivos e volumes USB |
| `FileAndFolderOpening` | Jump Lists, atalhos LNK, Microsoft Office, listas MRU, caixas de diálogo Abrir e Salvar como, RecentDocs, ShellBags |
| `FileExistence` | Amcache, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, Diário USN |
| `NetworkActivity` | Perfis de rede, interfaces, compartilhamentos, proxies de porta e rotas; conexões, tráfego e notificações push do SRUM |
| `LateralMovement` | Conexões de saída dos usuários por Área de Trabalho Remota, PuTTY e WinSCP, unidades mapeadas e compartilhamentos; Registro de acesso do usuário (os clientes das funções de um servidor) |
| `Persistence` | Configuração de inicialização (BCD), inicializações automáticas do Registro, serviços, TaskCache |
| `ProcessExecution` | Compatibilidade de aplicativos, BAM e DAM, FeatureUsage e itens fixados, MuiCache, Prefetch, linha do tempo do Prefetch, atividade de aplicativos/CPU do SRUM, UserAssist |
| `Registry` | Chaves e valores gerais dos hives, incluindo registros recuperados |
| `SecurityProducts` | Metadados da quarentena do Defender e conteúdo extraído opcional; configurações de segurança (Defender, UAC, proteção de credenciais, RDP, firewall, registro em log) |
| `SystemInformation` | Nomes, versão do Windows, fuso horário, último desligamento, perfis, SIDs e política de auditoria; energia/capacidade, mapas de identidade e provedores desconhecidos do SRUM |

As categorias ajudam a organizar as investigações; não são prova de uma atividade específica. Por exemplo,
entradas do Amcache e do ShimCache, por si sós, não estabelecem execução. O SRUM registra o uso de recursos;
consulte o guia de artefatos antes de tirar conclusões sobre a execução de um programa.

CSV, JSON e JSONL usam as mesmas categorias. Um arquivo `<artifact>-logs.txt` por tarefa de analisador contém seu
comando exato e suas mensagens; por exemplo, `HOST1/FileSystem/MFT-logs.txt` e
`HOST1/ProcessExecution/Prefetch-logs.txt`. JSON e JSONL usam os mesmos nomes de arquivo de log.
O Amcache e as Jump Lists têm, cada um, um log compartilhado ao lado de suas tabelas prefixadas. As tabelas dos provedores do SRUM são colocadas diretamente
em `NetworkActivity`, `ProcessExecution` e `SystemInformation`, com um arquivo compartilhado
`SystemInformation/SRUM-logs.txt`. Nenhum subdiretório `logs/` é criado.

Os nomes de arquivo CSV padrão são `Registry-SAM.csv`, `LNK.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv` e
`DefenderQuarantine.csv`. As tabelas derivadas do Registro usam o prefixo `Registry-`, incluindo
`Registry-ShimCache.csv`, `Registry-UserAssist.csv`, `Registry-BAM.csv` e `Registry-SystemInfo.csv`.
As chaves e valores gerais dos hives ficam em `Registry-Dump.csv`. As tabelas do Amcache são
`FileExistence/Registry-Amcache-Devices.csv`, `Registry-Amcache-Files.csv`,
`Registry-Amcache-Findings.csv` e `Registry-Amcache-Summary.csv`. As Jump Lists usam
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` e
`JumpLists-CustomDestinations.csv`. Não há subdiretórios para o Amcache nem para as Jump Lists.
As tabelas do SRUM usam nomes de arquivo `SRUM-` diretamente sob sua categoria, por exemplo
`NetworkActivity/SRUM-NetworkConnections.csv`.
As tabelas do Registro de acesso do usuário ficam em `LateralMovement`, um arquivo por tabela, como
`UAL-Clients.csv` e `UAL-SystemIdentity.csv`, com `UAL-logs.txt`; consulte
[Registro de acesso do usuário (UAL)](../artifacts/ual.md#automatic-parsing).
As tabelas dos navegadores web ficam em `BrowserActivity`: `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` e `Browser-FormHistory.csv`, com `Browser-logs.txt`; consulte
[Navegadores web](../artifacts/browser.md#automatic-parsing).
Consulte a [tabela completa de posicionamento do SRUM](../artifacts/srum.md#categorized-auto-parse-output).

Os logs correspondem ao nome do artefato com iniciais maiúsculas, como `Registry-SAM-logs.txt`, `PrefetchTimeline-logs.txt`
e `Registry-UserAssist-logs.txt`. Os arquivos JSON/JSONL usam as mesmas raízes de nome, por exemplo `MFT.jsonl`; o
identificador `Artifact` do registro é `mft` para consultas. As saídas com várias tabelas têm
`Registry-Amcache-logs.txt` ou `JumpLists-logs.txt` ao lado de suas tabelas. O log compartilhado do SRUM fica
em `SystemInformation`.


O `manifest.json` registra o `Hostname` original descoberto, a identidade `Comp` única da saída/dos registros,
a procedência do nome do host, os caminhos das evidências, a `Category` do artefato, as linhas de comando, a duração, o código
de saída, o caminho do resultado, o caminho do log, a linha do tempo JSONL solicitada e qualquer erro de exportação, se a extração do Defender foi solicitada,
e se cada tarefa
foi concluída com sucesso, ignorada ou falhou.

O auto-parse não grava nenhum caminho absoluto, de modo que os resultados podem ser compartilhados sem mostrar onde estão as evidências,
os resultados ou o Juntei no sistema de análise:

- Os caminhos das evidências são relativos ao diretório de evidências. O terminal lista cada computador com seu
  diretório nas evidências (`HOST1: ./HOST1_F`), e os analisadores são executados no diretório de evidências e
  recebem esses caminhos. As colunas de caminho de origem dos resultados (`SourceFile`, `SourcePath`, …), os logs
  e os campos `EvidenceRoot` e `WindowsDirectories` dos manifestos mostram onde cada arquivo está nas
  evidências (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`).
- Os caminhos de resultados dos manifestos e dos logs (`Output`, `OutputFiles`, `Log`, `JSON-Timeline`, os
  caminhos de saída em `Command`) e o `ExtractedFile` do Defender são relativos ao diretório de saída
  (`./HOST1/ProcessExecution/Prefetch.csv`, `../siem.jsonl` para uma linha do tempo ao lado dele). `Command`
  começa com o nome do programa (`juntei`); seus caminhos de evidências são relativos ao diretório de
  evidências, e seus caminhos de saída, ao diretório de saída.
- `EvidenceDirectory` e `OutputDirectory` são os nomes dos dois diretórios. `ManifestVersion` 2
  marca os manifestos que registram os caminhos de resultados a partir do diretório de saída; manifestos anteriores não têm
  `ManifestVersion` e os registram como foram fornecidos a `-o`. Uma nova execução com `-C` lê ambos.
- O terminal mostra um caminho de saída ou de linha do tempo como foi fornecido quando ele é relativo, e a partir do
  diretório atual quando é absoluto (`../results/`).

Depois do número de processos paralelos, o terminal mostra o tamanho total dos arquivos dos
sistemas Windows encontrados (`Total filesize: 26.1 MiB`).

A mensagem de progresso mostra o computador e o artefato ativos, seguidos de um tamanho aproximado
da entrada, por exemplo `HOST1 / prefetch (~2.0 MiB)`. Um grupo de arquivos usa o tamanho combinado; cada
tarefa de MFT, LogFile ou Diário USN mostra o tamanho individual de sua entrada. As estimativas excluem os logs de transação
auxiliares e os arquivos de enriquecimento e não preveem o tamanho da saída analisada.

O `manifest.csv` fornece o mesmo índice de tarefas como uma tabela adequada para planilhas, com uma linha por
tarefa de analisador. Ele inclui `Comp`, o `Hostname` original, `FQDN`, `HostnameSource`, `EvidenceRoot`,
`Artifact`, `Category`, `OutputType`, `Purpose`, `Status`, `DurationMs`, `ExitCode`, `Output`, `OutputFiles`, `Log`,
`Error` e `Command`. `Purpose` é `artifact` para os resultados categorizados ou `json_timeline` para
a passagem JSONL nativa adicional usada com CSV. `OutputType` é o formato real de saída do analisador.
`Command` é um array JSON com o executável e seus argumentos, preservando espaços e aspas.
Para o SRUM, `Output` é o diretório do computador, `OutputFiles` é um array JSON com os caminhos de saída
exatos de seus provedores, e `Category` indica a localização do log compartilhado (`SystemInformation`).
Os arquivos são categorizados individualmente; `OutputFiles` também aparece no manifesto JSON.

Filtre `Status` por `skipped` ou `failed` para encontrar as tarefas que exigem análise; o `Error` de uma tarefa ignorada
pode estar vazio, com a explicação em seu arquivo `Log`.

O resumo do terminal inclui o número total e o tamanho dos arquivos regulares existentes no momento sob o
diretório de saída, incluindo os dois manifestos, os logs dos analisadores e os arquivos extraídos. Links simbólicos
não são seguidos. Os tamanhos são escalonados automaticamente com o mesmo formatador de `Total filesize`, com
unidades como `KiB`, `MiB` e `GiB`:

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

As tabelas do Amcache e do SRUM usam `-Source1`, `-Source2` etc. quando vários bancos de dados de origem
foram coletados. O log compartilhado do analisador registra os caminhos originais e finais, e os registros JSON/JSONL
mantêm a procedência da origem. As tarefas CSV do Amcache/das Jump Lists listam seus caminhos exatos de tabela em
`OutputFiles` do manifesto; seu `Output` é o diretório da categoria investigativa. As exportações JSON e JSONL
do Amcache e das Jump Lists já são arquivos únicos chamados `Registry-Amcache.json[l]` e
`JumpLists.json[l]`. A linha do tempo combinada de `-j` mantém o mesmo conteúdo dos registros e os mesmos valores de `Artifact`.
`--clobber` usa o manifesto anterior para limpar as saídas registradas, incluindo nomes anteriores,
preservando os arquivos de categoria não relacionados. Uma coleta normal de um único volume grava os registros do `$MFT` e do Diário USN diretamente em
`FileSystem/MFT.csv` e `FileSystem/UsnJournal.csv`. Volumes coletados adicionais usam arquivos irmãos
numerados, como `FileSystem/MFT-2.csv` e `FileSystem/MFT-2-logs.txt`.

## Ingestão de JSON e em SIEM { #json-and-siem-ingestion }

Todo registro JSON ou JSONL recebe dois campos de nível superior:

- `Comp`: o sistema de origem.
- `Artifact`: o analisador do Juntei que produziu o registro, como `mft` ou `prefetch-timeline`.

`-j, --json-timeline <FILE>` cria um arquivo JSONL adicional, independentemente do formato dos resultados
individuais. Todas as saídas categorizadas e seus logs são mantidos:

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

O nome do arquivo é usado como fornecido; caminhos relativos são relativos ao diretório de trabalho atual.
Ele pode estar na raiz da saída, em outro subdiretório de saída não reservado ou fora do diretório
de saída. Evidências, manifestos, resultados/logs por computador e diretórios temporários dos analisadores não podem
ser usados como destino. Um arquivo existente exige `-C`; uma exportação que falha o mantém intacto.

No **modo CSV**, o Juntei executa uma passagem JSONL nativa adicional para que o arquivo do SIEM preserve os tipos
numéricos e booleanos, os dados aninhados, os campos brutos e os metadados que as tabelas CSV podem omitir. As
tarefas extras compartilham o mesmo limite de concorrência `-T`, não extraem novamente o conteúdo da quarentena e
têm logs auxiliares `*-json-timeline-logs.txt` separados. Ambos os manifestos distinguem sua finalidade e
seu formato de saída. Os arquivos JSONL temporários são removidos após uma execução bem-sucedida; eles são mantidos se um
analisador ou a exportação falhar. Planeje o tempo de processamento extra e o espaço temporário em disco.

No **modo JSON**, o Juntei transmite os resultados existentes para um JSONL compacto sem analisar novamente as
evidências. Os relatórios do Amcache são expandidos nos mesmos registros individuais de resumo, constatação, arquivo e dispositivo
do JSONL nativo. No **modo JSONL**, os registros anotados existentes são combinados diretamente.
Todos os modos usam a mesma estrutura de registro para SIEM. O conjunto de dados mantém os carimbos de data/hora e os campos
nativos dos artefatos; ele não inventa um carimbo de data/hora de evento comum nem ordena globalmente artefatos heterogêneos.

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

O `manifest.json` registra o caminho `JSON-Timeline` solicitado e qualquer `JSON-TimelineError`. Seus registros
de tarefa, e o `manifest.csv`, continuam apontando para as saídas de artefatos mantidas. As tarefas CSV bem-sucedidas exclusivas da linha do tempo
referenciam o arquivo JSONL final em vez dos arquivos temporários excluídos. Se algum analisador falhar, a
execução termina sem sucesso e a linha do tempo contém apenas os resultados bem-sucedidos; inspecione os manifestos
e os logs das categorias antes de tratá-la como completa. O resumo de contagem/tamanho de arquivos abrange os arquivos dentro
do diretório de saída; uma linha do tempo externa é relatada separadamente como `JSONL timeline: <FILE>`.

## Desempenho { #performance }

As tarefas dos analisadores são processos separados, de modo que artefatos que exigem muita CPU podem ser executados ao mesmo tempo, e um dano em
um analisador não consegue encerrar a execução inteira. Por padrão, o Juntei usa o número de CPUs lógicas. Limite a
concorrência quando as evidências estiverem em um disco lento ou o sistema tiver pouca memória:

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## Uso { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse the files inside every VHDX disk image in the evidence directory, each mounted read-only with FUSE while it is parsed

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Use um novo diretório de saída sempre que for viável. `-C` remove ou sobrescreve apenas os destinos exatos
gerados por esta execução, para que um artefato agora ausente não deixe um resultado desatualizado. Ele não exclui
arquivos não relacionados nem a árvore de saída inteira. Ele também remove o diretório
`SecurityProducts/DefenderQuarantined/` de cada computador, e o diretório reservado `defender-quarantined/`
do layout plano anterior, antes da análise, quer a nova execução use `-x` ou não.

Use um diretório de saída novo ao deixar um layout plano mais antigo. Os arquivos CSV/JSON anteriores
não são realocados nem excluídos automaticamente.

!!! danger "Trate o conteúdo extraído do Defender como malware"
    `-x, --extract-defender-quarantined` grava o conteúdo em quarentena em disco, no diretório
    `SecurityProducts/DefenderQuarantined/` de cada computador. Os fluxos binários recuperados terminam em `.quarantined`, e o Juntei
    remove a permissão de execução. Cada descritor de segurança binário também recebe um relatório legível `.security_descriptor.txt`
    com proprietário, permissões e SDDL. Nunca execute os payloads recuperados. Use um sistema de análise
    isolado e espere que o antivírus ou o EDR gere alertas sobre o conteúdo recuperado ou o remova.

Consulte [como ler o descritor de segurança recuperado](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
para conhecer os campos do relatório e suas limitações forenses.

O auto-parse também lê as conexões remotas dos usuários (RDP, PuTTY, WinSCP e unidades mapeadas).
