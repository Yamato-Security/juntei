# Persistence Hunter { #persistence-hunter }

A persistência permite que um programa seja executado novamente após uma reinicialização, uma entrada na sessão, um gatilho agendado ou outro
evento. Um atacante pode usar a configuração normal do Windows para esse fim. Muitos programas legítimos
usam exatamente os mesmos mecanismos, portanto encontrar uma entrada de persistência é um ponto de partida para a análise.

`juntei persistence-hunter` é um port nativo em Rust do
[PyrsistenceSniper da Hexastrike](https://github.com/hexastrike/PyrsistenceSniper), com 131 verificações: o
catálogo de 130 verificações do PyrsistenceSniper mais uma verificação nativa de COM do menu Iniciar.

Para uma explicação de cada mecanismo, de seus locais de evidência e de como investigar uma constatação,
consulte o [guia de persistência com 131 verificações](../persistence-hunter/index.md). Os
[agradecimentos sobre persistência](../persistence-hunter/acknowledgements.md) descrevem os projetos em Python e
PowerShell por trás do catálogo.

## Uso { #usage }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Primeira varredura { #first-scan }

```bash
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

As evidências podem ser um volume do Windows montado, uma coleta do Velociraptor ou um diretório que contenha
vários computadores coletados. Os computadores são mantidos separados durante a análise, e todo resultado inclui
`Comp`. O escape de nomes de arquivo coletados é decodificado ao resolver caminhos do Windows.

Para um único hive:

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

Um único hive não pode alimentar verificações que precisam de outro hive ou de evidências do sistema de arquivos. Leia o relatório
de cobertura antes de tratar um resultado vazio como significativo.

Liste os IDs das verificações e seus mapeamentos ATT&CK:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

Use os IDs de verificação impressos por `--list-checks`, ou um ID ATT&CK como `T1547.001`.
Um ID pai como `T1547` seleciona suas subtécnicas. `--technique` é um alias de `--check`.
Cada verificação inclui sua técnica, descrição e links de referência nas constatações.

## Varreduras em paralelo { #parallel-scans }

O comando usa um pool de workers compartilhado, cujo padrão é o número de CPUs lógicas. Defina
`-T, --threads <N>` para escolher a quantidade de workers, ou `-T 1` para uma varredura com uma única thread:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

A descoberta de evidências, as verificações independentes, os metadados da carga útil e as fontes de linha do tempo MFT/EVTX são executados em paralelo. Os
hives do Registro carregados de um host são compartilhados somente para leitura, e os metadados em cache da mesma carga útil são inicializados uma única vez.
Os hosts são carregados um de cada vez para evitar multiplicar a memória dos hives em toda a coleta. As constatações,
a cobertura, a seleção de carimbos de data/hora e a ordem dos avisos permanecem as mesmas com diferentes quantidades de threads.

O `auto-parse` já executa processos de análise em paralelo, por isso dá a cada processo filho `persistence-hunter`
um único worker. Sua própria opção `-T` controla o número de processos de análise simultâneos.

## Evidências a coletar { #evidence-to-collect }

| Fonte | Exemplos de verificações |
|---|---|
| `SYSTEM` e logs de transação adjacentes | Serviços, comandos de recuperação de serviços, pacotes LSA, configurações de inicialização, provedores de autenticação |
| `SOFTWARE` e logs de transação adjacentes | Chaves Run, Winlogon, IFEO, registros COM, TaskCache, configurações do Office |
| `NTUSER.DAT` e `UsrClass.dat` de cada usuário | Inicializações automáticas do usuário, variáveis de ambiente, substituições de COM, configurações do Office |
| `SAM` e `SOFTWARE` | Anomalias de RID, contas ocultas, associação incomum ao grupo local Administradores |
| `Windows/System32/Tasks` | Comandos e manipuladores COM de tarefas agendadas; comparação com o TaskCache |
| Arquivos `.library-ms` do menu Iniciar de cada usuário e pastas nomeadas com CLSID | Candidatos a gatilhos de ativação COM, unidos aos registros de classe correspondentes do usuário/da máquina |
| Pastas de inicialização, perfis do PowerShell, modelos do Office, configurações do Terminal | Arquivos e comandos executados por gatilhos do usuário ou do sistema |
| `OBJECTS.DATA` do WMI | Instâncias de consumidores de eventos e vínculos entre filtros e consumidores |
| `qmgr.db` ou `qmgr*.dat` do BITS | Comandos de notificação recuperados e SIDs dos proprietários |
| `$MFT` e arquivos EVTX relevantes | Correlação independente de carimbos de data/hora |
| Executáveis, DLLs e arquivos de catálogo referenciados | Hashes SHA256 e metadados descritivos de PE/certificado |

Os logs de transação do Registro são reproduzidos quando disponíveis. `--no-logs` lê os hives como foram coletados;
um hive sujo não reproduzido pode omitir alterações recentes e é relatado na cobertura. O conjunto de controle
atual é usado nas verificações do sistema ativo. A verificação do backdoor DSRM examina adicionalmente
`ControlSet001`, `ControlSet002` e `CurrentControlSet` quando presentes; confira o caminho relatado
em relação a `SYSTEM\\Select` antes de descrever essa configuração como ativa. Outros conjuntos de controle históricos
podem ser examinados com os comandos individuais de análise do Registro.

As leituras de hives compartilham um orçamento de entrada de 1 GiB por computador. Os hives da máquina são tentados antes dos hives de usuário;
entradas ou logs de transação que não cabem são nomeados na cobertura parcial. Os demais hives legíveis e as
verificações baseadas em arquivos continuam. Somente os locais canônicos de hives ou exportações fornecidas explicitamente estabelecem a
identidade do hive; uma cópia dentro do diretório temporário de um usuário não pode substituir o hive real desse usuário.
A unidade de sistema original registrada no SOFTWARE é mapeada para o volume de sistema capturado, de modo que montar
um volume `C:` original sob um rótulo de unidade de evidência diferente não descarta os metadados da carga útil.

O `auto-parse` também executa o hunter e salva `persistence/PersistenceHunter.csv` (ou `.json`/`.jsonl`)
dentro do diretório de cada computador, com `PersistenceHunter-logs.txt` ao lado. Suas constatações também entram
na linha do tempo opcional para SIEM de `-j <FILE>`. A análise automática usa `--min-severity info` para manter o
inventário completo de configurações, incluindo as entradas correspondidas por regras de permissão.

## Registros COM e gatilhos de ativação { #com-registrations-and-activation-triggers }

A verificação `scheduled_task_files` seleciona o principal de cada ação usando o `Actions Context` do XML da tarefa
e o `Principal id` correspondente. Um principal de grupo é mantido como grupo;
ele não pode determinar quais registros de classe de um usuário individual seriam usados.

Os candidatos a manipuladores COM são resolvidos separadamente nas visões de classes nativa e de 32 bits. Somente o
`UsrClass.dat`/`NTUSER.DAT` capturado do usuário correspondente e o `SOFTWARE` da máquina são considerados.
Os principais por SID são unidos por meio dos caminhos capturados de `ProfileList`. Os nomes de conta, incluindo
`HOST\user` e `DOMAIN\user`, preferem um perfil `user.DOMAIN` correspondente a um perfil
`user` simples. Aliases do host local e `.\user` usam os nomes de host capturados para a correspondência por sufixo;
um sufixo exato tem precedência sobre outros aliases de host. O perfil simples é considerado
somente quando nenhum perfil qualificado corresponde. Toda correspondência desse tipo é rotulada explicitamente como uma
**heurística por nome de pasta** em `Description`; ela não prova a propriedade da conta nem a associação ao domínio. Correspondências ambíguas
permanecem sem resolução e são relatadas tanto na descrição da constatação quanto na cobertura; a ambiguidade
entre perfis qualificados não aciona um fallback para o perfil simples. Isso não depende
de `Volatile Environment`, que normalmente existe apenas na memória e não é salvo com os hives do Registro. Um CLSID de usuário duplicado
oculta os filhos do CLSID da máquina. Os principais de grupo usam candidatos COM da máquina e mantêm
essa limitação na descrição da constatação; um principal de grupo normal não torna, por si só,
a cobertura parcial nem consome as entradas do resumo de erros.

O resolvedor segue `InprocServer32` e `LocalServer32`, usa caminhos `CodeBase` gerenciados
(incluindo subchaves de versão) e une o `LocalService` do AppID ao `ImagePath` do serviço coletado
no SYSTEM. As associações de nomes de executáveis do AppID são mantidas, mas um nome base não estabelece
o caminho completo de um executável. As URLs `file:` locais de CodeBase são decodificadas; as URLs remotas são registradas
sem serem acessadas. Os registros ausentes mantêm o CLSID com
`ResolutionStatus: unresolved_com_class`, campos de caminho/hash vazios e `Exists` desconhecido.

A verificação `start_menu_com` lê as URLs XML dos arquivos `.library-ms` do menu Iniciar de cada usuário e coleta as
pastas nomeadas com CLSID, incluindo diretórios vazios. Ela relata o caminho do gatilho, o CLSID, o servidor
candidato e a proveniência no Registro. Uma aquisição somente de arquivos não consegue preservar um diretório vazio,
e a ausência de hives de usuário impede uma busca completa de registros por usuário. Links simbólicos
não são seguidos. Estes são candidatos de configuração: arquitetura, elevação, UAC, política e
contexto de ativação podem afetar se um registro é realmente usado. Corrobore a execução
com evidências de processos, de carregamento de módulos ou outras evidências independentes.

## Classificação e perfis { #classification-and-profiles }

A classificação mínima padrão é `medium`. Para inspecionar o inventário completo, incluindo as
entradas correspondidas pela lista de permissões incluída:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "Use todas as severidades ao revisar coletas incompletas"
    Uma regra de permissão ampla pode corresponder parcialmente ao comando de um atacante mesmo quando seu assinante está ausente
    ou é diferente. O projeto original classifica correspondências parciais como `LOW`, que o limite padrão `medium`
    do comando manual oculta. Isso pode afetar tarefas agendadas maliciosas em diretórios do Windows.
    Use `--min-severity info` para uma investigação completa e revise `PolicyReason`, `SignerStatus`,
    o comando e as evidências de origem em conjunto. O `auto-parse` já mantém essas entradas.

| Classificação | Significado |
|---|---|
| `HIGH` | Uma regra de bloqueio configurada correspondeu totalmente |
| `MEDIUM` | Nenhuma regra de permissão ou de bloqueio correspondeu |
| `LOW` | As condições rígidas de uma regra de permissão corresponderam, mas os metadados do assinante não corresponderam ou não estavam disponíveis |
| `INFO` | Uma regra de permissão correspondeu totalmente |

Essas são classificações de política, não probabilidades medidas de comprometimento. Uma entrada `MEDIUM` padrão
não é prova de malware. `INFO` não estabelece que um arquivo tem uma assinatura válida nem que
um atacante não abusou dele. Uma correspondência parcial de permissão fica abaixo do limite padrão, portanto inclua
`--min-severity low` ou `info` ao revisar coletas esparsas ou ajustar a lista de permissões.

Forneça um perfil YAML com `--profile profile.yaml`. Ele **substitui** o perfil incluído. As regras globais
e por verificação dentro do arquivo selecionado são combinadas; qualquer regra de bloqueio totalmente correspondida prevalece.
Por exemplo:

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

As condições de regra compatíveis são `path_matches`, `value_matches`, `hash` (64 dígitos hexadecimais de SHA256),
`not_lolbin` e `signer` (substring sem distinção entre maiúsculas e minúsculas). Todas as condições rígidas declaradas devem
corresponder. Os metadados do assinante são a condição flexível descrita acima. As regexes usam a sintaxe de expressões regulares
do Rust: lookarounds e referências retroativas não são compatíveis. IDs de verificação desconhecidos, expressões inválidas
e campos de perfil desconhecidos produzem um erro em vez de enfraquecer silenciosamente a política.

## Campos de saída { #output-fields }

CSV é o formato de arquivo padrão. Use `-t json` para objetos JSON legíveis ou `-t jsonl` para uma saída compacta,
com um objeto por linha. Sem `-o`, um `-t` explícito grava apenas dados em stdout; os avisos de cobertura
vão para stderr. Sem nenhuma das duas opções, o Juntei imprime um resumo e até 30 candidatos.

| Campos | Como interpretá-los |
|---|---|
| `Comp`, `CheckID`, `Technique`, `MitreID` | Qual computador e qual mecanismo de persistência a linha descreve |
| `Path`, `Value` | O local do Registro, arquivo, tarefa ou registro recuperado e seu conteúdo configurado |
| `Description`, `References` | Por que o mecanismo é importante e onde ler mais |
| `AccessGained` | O escopo usual do mecanismo (`USER` ou `SYSTEM`; o BITS também pode identificar `LOCAL_SERVICE`/`NETWORK_SERVICE`); não é prova de que um comando específico obteve esse privilégio |
| `Severity`, `PolicyReason` | Classificação do perfil, explicação da política correspondente e qualquer ressalva de correspondência parcial ou de assinante não verificado |
| `SourceFile`, `RecordStatus` | A fonte coletada exata, seu caminho começando pela entrada conforme fornecida (`./HOST/...` a partir do auto-parse), e o contexto de alocação/reprodução/recuperação; siga-o de volta até a evidência original |
| `KeyLastWrite` | O carimbo de data/hora da **chave** do Registro. Ele não data um valor individual |
| `LastChange`, `ChangeSource`, `ChangeEvidence` | Horário de alteração correlacionado de forma independente, a prova de origem e se a correlação foi bem-sucedida |
| `ResolvedPath`, `ResolutionStatus`, `Exists` | Caminho da carga útil e resultado da resolução offline; evidências não coletadas são desconhecidas, não `false` |
| `SHA256` | SHA256 dos bytes da carga útil coletada, quando legível |
| `Signer`, `SignerStatus` | Metadados descritivos de certificado incorporado/catálogo e sua disponibilidade; a confiança no certificado e a validade da assinatura não são verificadas |
| `IsLolbin`, `Launcher` | Se o inicializador ou a carga útil configurados estão na lista LOLBAS incluída, e o inicializador usado para uma carga útil executada por proxy |
| `IsBuiltin`, `IsInOS-Directory` | Contexto para comandos comuns do Windows e diretórios do sistema; nenhum dos campos estabelece legitimidade |

Um comando `rundll32.exe` pode nomear uma carga útil DLL. O Juntei preserva o inicializador enquanto resolve e
calcula o hash da carga útil. Uma referência fora do computador coletado, uma unidade ausente, uma correspondência ambígua
sem distinção entre maiúsculas e minúsculas ou uma variável de ambiente não resolvida não é substituída por um arquivo da
máquina do analista.

## Carimbos de data/hora e limitações { #timestamps-and-limitations }

O hunter não usa os horários de modificação dos arquivos copiados na VM do analista como carimbos de data/hora do ataque.
Quando disponíveis, ele correlaciona gravações no Registro com o evento 13 do Sysmon, a instalação de tarefas com o
Security 4698 ou o Task Scheduler 106, a instalação de serviços com o System 7045, e configurações baseadas em arquivos
com os carimbos de data/hora originais do `$MFT`. A proveniência aparece em `ChangeSource`.
Um evento pode datar uma alteração de configuração sem provar que a carga útil configurada foi executada.
O evento 6011 do log System local pode provar um nome de computador anterior. O hunter segue cadeias de renomeação
verificadas ancoradas no computador coletado, preservando seu valor atual de `Comp` enquanto registra
o computador original do evento e a prova do registro de renomeação em `ChangeSource`. Nomes estrangeiros não comprovados,
logs arquivados, logs encaminhados e outra instalação do Windows não são aceitos por meio dessa regra.

ComputerName/ActiveComputerName, Hostname/NV Hostname do TCP/IP e o hostname/FQDN de client-info capturados
fornecem aliases do host, cobrindo diferenças de DNS/NetBIOS e renomeações pendentes. Sem nenhuma identidade capturada,
um único nome nos logs primários pode sustentar a datação com proveniência explícita de identidade não verificada
e um aviso de cobertura. Vários nomes não verificados não são atribuídos ao host. Os nomes de Computer rejeitados
são relatados como uma limitação de cobertura da correlação; seus canais continuam marcados como coletados.

Use `--mft <FILE>` para selecionar uma MFT coletada externamente para um único computador. Use `--no-timeline`
para ignorar a correlação de carimbos de data/hora. `NOT_RUN`, `NOT_APPLICABLE`, `NO_ARTIFACT`, `NO_MATCH` e `REJECTED`
explicam um `LastChange` vazio; um campo vazio não é evidência de que a entrada é inofensiva.

A recuperação de WMI e BITS é um carving limitado, e não um analisador completo de transações de banco de dados.
Strings ou vínculos recuperados podem incluir dados antigos/excluídos; esses registros precisam ser corroborados.
Os esquemas de classes WMI sozinhos não são contados como assinaturas. Uma entrada do TaskCache sem um arquivo XML
de tarefa coletado é uma discrepância a ser revisada; uma coleta de triagem pode simplesmente ter omitido esse arquivo.
Os metadados de catálogo e PE não são verificados e podem estar ausentes de uma coleta. As verificações de substituição de
arquivos de acessibilidade dependem de metadados de assinatura de referência adequados; a ausência de evidências de catálogo
não deve fazer com que todos os executáveis de acessibilidade sejam sinalizados.

A cobertura é registrada para todas as verificações, inclusive as que têm zero constatações. `complete` significa que a
fonte coletada aplicável foi lida; isso não prova que a coleta está completa nem que
o computador não tem outra persistência. `partial`, `not_collected`, `failed` e `disabled` explicam
os limites. Guarde o `--coverage Coverage.csv` e o log do analisador junto com suas constatações.

O port usa os formatos CSV/JSON/JSONL do Juntei. Ele não chama o programa Python original, não consulta
serviços de enriquecimento online nem reproduz a interface HTML/XLSX original.

## Mais detalhes { #further-details }

- [Implementação e perfis do PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper)
- [Técnicas de persistência do MITRE ATT&CK](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft: Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [Inventário de inicializações automáticas do Registro](../artifacts/persistence.md)
- [Serviços](../artifacts/services.md) e [TaskCache](../artifacts/taskcache.md)
