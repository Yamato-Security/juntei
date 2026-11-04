# Persistence Hunter { #persistence-hunter }

Persistência é uma forma de fazer um código ser executado novamente: quando o Windows é iniciado, um usuário entra na sessão, um aplicativo
é aberto, um horário agendado chega ou outro evento ocorre. Atacantes podem abusar das mesmas configurações
que softwares legítimos usam para atualizações, suporte a dispositivos, segurança e administração.

`juntei persistence-hunter` examina offline as evidências do Windows capturadas com **131 verificações** que cobrem
esses mecanismos e anomalias relacionadas de contas/proteção. Ele lê hives do Registro, arquivos de tarefas,
scripts, configurações de aplicativos e registros selecionados de bancos de dados.

Esta página explica cada verificação implementada. Para ver todas as opções, campos de saída, perfis e o
tratamento de carimbos de data/hora, abra o [guia do comando](../commands/persistence-hunter.md). Os
[agradecimentos](acknowledgements.md) trazem links para o projeto PyrsistenceSniper em Python e para a ferramenta original
PersistenceSniper em PowerShell.

## Iniciar uma investigação { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

Use `--min-severity info` para obter o inventário completo de candidatos. O comando manual usa
`medium` por padrão; seu perfil pode classificar correspondências parciais com a lista de permissões como `LOW` mesmo quando a ausência de evidências da carga útil ou do
assinante impede uma comparação completa. Nem uma correspondência com a lista de permissões nem uma entrada desconhecida determinam
se algo é malicioso. O `auto-parse` já mantém todas as severidades e grava as constatações
na categoria `persistence/` de cada computador.

Para focar em um mecanismo, copie seu **ID da verificação** abaixo:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

Por padrão, o comando usa um pool de workers compartilhado. Adicione `-T 4` para selecionar quatro workers, ou `-T 1`
para uma varredura com uma única thread. A entrada pode conter vários computadores; as evidências de cada um permanecem separadas.

## Ler as constatações como evidências { #read-findings-as-evidence }

| Pergunta | O que inspecionar |
|---|---|
| O que foi configurado, e onde? | `CheckId`, `Path` e o `Value` completo, incluindo os argumentos. Siga `SourceFile` até o hive, arquivo ou banco de dados capturado. |
| O destino existe na coleta? | `ResolvedPath`, `ResolutionStatus` e `Exists`. Referências não coletadas, ambíguas ou não compatíveis são desconhecidas; o Juntei não substitui por um arquivo da máquina do analista. |
| A configuração é efetiva? | O gatilho real, os flags de habilitação, o conjunto de controle selecionado, a versão do Windows/aplicativo, o usuário e a arquitetura. Algumas verificações mantêm deliberadamente configurações inativas. |
| Isto é um ataque? | `Severity` e `PolicyReason` explicam a correspondência com o perfil, não um veredito. Compare com o software esperado, o comando completo, os hashes dos arquivos, evidências independentes de execução e a linha do tempo ao redor. |
| Quem poderia executá-lo? | `AccessGained` é uma indicação de escopo/contexto. Configurações de toda a máquina frequentemente indicam `SYSTEM`, mas isso não prova que o processo iniciado foi executado como SYSTEM. Principais de tarefas e SIDs de proprietários do BITS fornecem um contexto mais específico, quando disponíveis. |
| Quando mudou? | `KeyLastWrite` data a chave do Registro inteira. `LastChange`, `ChangeSource` e `ChangeEvidence` descrevem uma correlação separada com MFT/EVTX, quando disponível. Nenhum desses campos data automaticamente a execução da carga útil. |
| Quão confiável é o estado recuperado? | Leia `RecordStatus`, `Coverage.csv` e os avisos sobre evidências reproduzidas, recuperadas por carving, danificadas, ausentes ou limitadas. Um registro histórico recuperado pode não estar mais ativo. |
| Um editor nomeado é confiável? | `Signer` e `SignerStatus` são metadados descritivos de PE/certificado/catálogo. O Juntei não valida assinaturas Authenticode nem a confiança de certificados. |

A ausência de constatações não prova a ausência de persistência. Uma linha de cobertura `complete` significa que
a fonte coletada aplicável foi lida, não que a aquisição contenha todos os arquivos relevantes,
todos os usuários ou todos os mecanismos de persistência possíveis do Windows. Os outros estados de cobertura explicam verificações ausentes,
parciais, desabilitadas ou com falha. O hunter não é uma varredura de memória em execução, de rede ou de identidades em nuvem.

## Catálogo e notação de locais { #catalogue-and-location-notation }

O catálogo se baseia no PyrsistenceSniper, mais a verificação nativa `start_menu_com`;
as descrições abaixo documentam
o comportamento nativo do Juntei, incluindo as limitações de suas evidências. Há **116 verificações baseadas no
Registro e 15 verificações de arquivos/repositórios**. Algumas verificações de arquivos também combinam dados do Registro, portanto esses números são
contagens de implementação, e não requisitos de aquisição mutuamente exclusivos. Os mecanismos
estão agrupados abaixo por finalidade investigativa e em ordem alfabética de ID da verificação dentro de cada grupo.

- `HKLM\SYSTEM`, `HKLM\SOFTWARE` e `HKLM\SAM` referem-se aos hives da máquina capturados, normalmente
  coletados de `Windows\System32\config`, com os logs de transação quando disponíveis.
- `HKU\<user>` significa o hive de usuário capturado relevante. A maioria das configurações de usuário vem do NTUSER.DAT;
  registros de classes podem vir de `AppData\Local\Microsoft\Windows\UsrClass.dat`, conforme indicado.
- `<ControlSet>` significa o conjunto de controle ativo selecionado no SYSTEM. Os locais históricos adicionais da
  verificação DSRM são informados explicitamente abaixo. Consulte `SYSTEM\Select` ao revisá-los.
- `<name>` é um componente variável do caminho. Nas tabelas de destinos do Registro, `<child>` significa registros
  filhos imediatos; não significa uma travessia recursiva ilimitada. As verificações explicitamente recursivas
  descrevem sua travessia separadamente. Os colchetes nos caminhos do Office indicam uma versão opcional.
- As tabelas listam as visões reais de máquina e de usuário selecionadas pela verificação. Uma visão de 32 bits é listada
  onde implementada; não presuma que todas as verificações examinam todas as arquiteturas/visões possíveis.
- As verificações genéricas de valores emitem as entradas configuradas não vazias, incluindo elementos de multi-strings e
  zero numérico, quando aplicável. Elas não exigem, por si só, uma carga útil maliciosa, um
  gatilho habilitado ou uma execução bem-sucedida. Predicados mais seletivos são informados em cada verificação.

Os IDs do ATT&CK são mapeamentos herdados do catálogo e links para estudo adicional. Eles descrevem um possível
abuso, não uma constatação de que um atacante executou aquela técnica; verificações de configurações de proteção e de diagnóstico
podem descrever condições facilitadoras em vez de um mecanismo de persistência independente.

| Grupo de mecanismos | Verificações |
|---|---:|
| [Inicialização, entrada na sessão e startup](#boot-sign-in-and-startup) | 32 |
| [Serviços e extensões do sistema](#services-and-system-extensions) | 13 |
| [Tarefas agendadas](#scheduled-tasks) | 3 |
| [WMI, BITS e outros gatilhos de eventos](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorer, COM e manipuladores de arquivos ou URLs](#explorer-com-and-file-or-url-handlers) | 15 |
| [Aplicativos e suplementos do Office](#office-applications-and-add-ins) | 7 |
| [Depuradores, manipuladores de falhas e shims de compatibilidade](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [Configurações do runtime .NET e de profilers](#net-runtime-and-profiler-settings) | 7 |
| [Substituições na localização de DLLs e executáveis](#dll-and-executable-lookup-overrides) | 18 |
| [Contas e visibilidade na entrada da sessão](#accounts-and-sign-in-visibility) | 4 |
| [Componentes de autenticação e proteção de credenciais](#authentication-components-and-credential-protection) | 8 |
| **Total** | **131** |

## Inicialização, entrada na sessão e startup { #boot-sign-in-and-startup }

Essas configurações fazem com que um comando ou biblioteca seja executado durante a inicialização, a entrada na sessão ou a inicialização da área de trabalho. Revise a versão do Windows e o usuário aplicável: uma configuração pode estar obsoleta, desabilitada ou destinada a um quiosque, e não a um atacante.

### Active Setup {#active_setup}

**ID da verificação:** `active_setup` · **ATT&CK do catálogo:** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**O que o Juntei verifica:** lê `StubPath` dos registros de componentes do Active Setup de toda a máquina nas visões nativa e de 32 bits. O Windows pode usar esses comandos ao preparar o perfil de um usuário; a verificação ignora os valores de espaço reservado literais `U` e `/UserInstall`.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` e a árvore `Wow6432Node` correspondente.

**Como investigar:** compare cada componente com o produto instalado e inspecione os argumentos do comando. Um registro de toda a máquina pode levar à execução na sessão de um usuário que está entrando; o rótulo `SYSTEM` da saída descreve o escopo da configuração, não uma prova do token do processo resultante.

### Boot Execute {#boot_execute}

**ID da verificação:** `boot_execute` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê a multi-string `BootExecute` do Session Manager, preservando cada entrada não vazia. Ela registra comandos configurados para uma fase inicial do boot.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**Como investigar:** `autocheck autochk *` é um padrão conhecido, e ferramentas de disco podem adicionar entradas legítimas. Investigue os comandos adicionados e seus arquivos sem presumir que toda entrada fora do padrão foi executada ou é maliciosa.

### Boot Verification Program {#boot_verification_program}

**ID da verificação:** `boot_verification_program` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê o `ImagePath` de `BootVerificationProgram`, um local de executável associado à verificação do boot.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**Como investigar:** verifique se esse mecanismo mais antigo é aplicável à versão do Windows e à configuração de boot capturadas. Resolva o arquivo de destino e compare sua finalidade com o software de gerenciamento de boot esperado.

### Valor Load do Explorer {#explorer_load}

**ID da verificação:** `explorer_load` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê o valor `Load` associado à configuração de inicialização da entrada na sessão do Windows nos hives de máquina e de usuário.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**Como investigar:** investigue comandos desconhecidos, destinos graváveis pelo usuário e argumentos que baixam ou decodificam conteúdo. Confirme a aplicabilidade e a execução separadamente; um resultado vazio não cobre outros mecanismos de entrada na sessão.

### Drivers de fonte (Font Drivers) {#font_drivers}

**ID da verificação:** `font_drivers` · **ATT&CK do catálogo:** [T1547](https://attack.mitre.org/techniques/T1547/)

**O que o Juntei verifica:** enumera os valores não vazios da chave legada do Registro `Font Drivers` como possíveis registros de bibliotecas.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | Todos os valores não vazios |

**Como investigar:** trate isso como uma superfície de configuração legada incomum. Verifique a compatibilidade com a versão do Windows e a biblioteca referenciada antes de concluir que ela poderia ser carregada, e corrobore o uso real com evidências de processos ou de carregamento de imagens.

### DLLs de extensões de Política de Grupo {#gp_extension_dlls}

**ID da verificação:** `gp_extension_dlls` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê `DllName` de cada registro imediato de extensão de Política de Grupo sob o Winlogon. Essas DLLs estendem o processamento de políticas.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**Como investigar:** produtos de gerenciamento e de segurança podem instalar extensões. Associe o GUID da extensão e a DLL ao produto esperado e examine evidências de processamento de políticas e de carregamento de imagens antes de afirmar que houve execução.

### Scripts de Política de Grupo {#gp_scripts}

**ID da verificação:** `gp_scripts` · **ATT&CK do catálogo:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**O que o Juntei verifica:** analisa os pares comando/parâmetro dos arquivos de registro de Política de Grupo local `scripts.ini` e `psscripts.ini` capturados, incluindo os repositórios de política local por usuário. As seções de inicialização/desligamento e de logon/logoff mantêm seus comandos e argumentos configurados, com as seções de máquina diferenciadas das seções de usuário.

**Evidências examinadas:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` e os arquivos equivalentes em `GroupPolicy\User\Scripts`.
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`.

**Como investigar:** apenas os locais de registro de política local implementados são examinados; esta não é uma auditoria completa de SYSVOL/GPO. Um arquivo de script localizado em outro ponto de um diretório de políticas não é suficiente. Correlacione seu registro com evidências de processamento de políticas e de execução.

### Scripts de logon (UserInitMprLogonScript) {#logon_scripts}

**ID da verificação:** `logon_scripts` · **ATT&CK do catálogo:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**O que o Juntei verifica:** lê o valor de ambiente `UserInitMprLogonScript` do usuário, que pode selecionar um script para a entrada na sessão desse usuário.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**Como investigar:** compare com os scripts de logon aprovados e inspecione o script real. O valor não prova uma entrada na sessão ou uma execução posterior, e scripts fornecidos pelo domínio exigem evidências adicionais de política.

### Platform Execute {#platform_execute}

**ID da verificação:** `platform_execute` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê cada entrada `PlatformExecute` não vazia no Session Manager. Ela identifica programas selecionados para a inicialização de boot relacionada à plataforma.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**Como investigar:** fluxos de trabalho de hardware ou de provisionamento podem explicar uma entrada. Determine se o caminho de boot nesta build do Windows a utiliza e investigue a imagem referenciada, em vez de inferir um ataque a partir do nome do valor.

### DLL de extensão do cliente RDP {#rdp_clx_dll}

**ID da verificação:** `rdp_clx_dll` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê `ClxDllPath` na configuração de usuário padrão do Terminal Server como um possível local de extensão relacionada ao RDP.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**Como investigar:** verifique a versão do Windows/RDP e as extensões instaladas. Uma DLL configurada é uma pista para a análise de carregamento de imagens; o valor do Registro, por si só, não mostra que uma conexão RDP ou um carregamento de DLL ocorreu.

### DLL de canal virtual RDP {#rdp_virtual_channel}

**ID da verificação:** `rdp_virtual_channel` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** enumera os valores sob os registros `Default\AddIns` do Terminal Server Client de toda a máquina. Valores binários são decodificados como UTF-16LE quando possível, preservando caminhos de extensão utilizáveis.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: todos os valores.

**Como investigar:** clientes RDP e produtos de redirecionamento de dispositivos registram suplementos legitimamente. Inspecione a DLL ou o comando real e a sessão de cliente relevante; nem todo valor sob um suplemento é um caminho de executável.

### Programas de inicialização do RDP WDS {#rdp_wds_startup}

**ID da verificação:** `rdp_wds_startup` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê `StartupPrograms` da configuração do driver de Windows Station do RDP. Ele identifica programas configurados para a inicialização de sessões remotas.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**Como investigar:** `rdpclip` é uma entrada conhecida. Investigue as adições e a configuração RDP do host, e use registros de sessões/processos para determinar se o programa configurado foi de fato iniciado.

### Chaves Run do Registro {#run_keys}

**ID da verificação:** `run_keys` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** enumera os valores Run, RunOnce, RunEx, RunOnceEx e Run de políticas do Explorer dos hives de máquina/usuário, além das chaves de 32 bits da máquina listadas. As seções RunEx/RunOnceEx são percorridas com limites e verificações de ciclos.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Todos os valores não vazios |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Todos os valores não vazios |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Todos os valores não vazios |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Todos os valores não vazios |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Todos os valores não vazios |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Todos os valores não vazios |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Todos os valores não vazios |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Todos os valores não vazios |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Todos os valores não vazios |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Todos os valores não vazios |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | Todos os valores não vazios |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | Todos os valores não vazios |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | Todos os valores não vazios |

**Como investigar:** atualizadores de software e aplicativos da bandeja do sistema aparecem aqui com frequência. Compare o executável, os argumentos do comando e o contexto do usuário. Uma entrada RunOnce pode desaparecer após o uso; o estado capturado não é um histórico completo das entradas na sessão anteriores.

### RunServices (legado) {#run_services}

**ID da verificação:** `run_services` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** enumera os valores da chave legada `RunServices` de toda a máquina.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | Todos os valores não vazios |

**Como investigar:** esta é uma superfície de configuração histórica, não uma garantia de que uma build moderna do Windows a execute. Investigue o destino e a aplicabilidade ao sistema operacional e, em seguida, busque evidências independentes de execução.

### RunServicesOnce (legado) {#run_services_once}

**ID da verificação:** `run_services_once` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** enumera os valores da chave legada `RunServicesOnce` de toda a máquina.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | Todos os valores não vazios |

**Como investigar:** trate os valores não vazios como pistas de configuração. Verifique se a versão do Windows de origem é compatível com o mecanismo e se um software de implantação ou de compatibilidade explica a entrada.

### Comando inicial da Sessão 0 (S0 Initial Command) {#s0_initial_command}

**ID da verificação:** `s0_initial_command` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê `S0InitialCommand` do Session Manager como um possível comando de inicialização da Sessão 0.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**Como investigar:** estabeleça a aplicabilidade à build do Windows e ao estado de boot capturados. Um comando configurado não é um registro com carimbo de data/hora da execução no boot; inspecione seu arquivo e correlacione com a telemetria de boot/processos.

### Session Manager Execute {#session_manager_execute}

**ID da verificação:** `session_manager_execute` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê as entradas `Execute` não vazias do Session Manager, expondo programas selecionados para a inicialização da sessão.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**Como investigar:** compare com a linha de base de imagens esperada e inspecione executáveis nativos ou caminhos incomuns. Estabeleça se um boot/sessão relevante ocorreu após a alteração antes de afirmar que houve execução.

### Session Manager SubSystems {#session_manager_subsystems}

**ID da verificação:** `session_manager_subsystems` · **ATT&CK do catálogo:** [T1547](https://attack.mitre.org/techniques/T1547/)

**O que o Juntei verifica:** lê o valor `Windows` da chave `SubSystems` do Session Manager, que configura o comando do subsistema Windows.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**Como investigar:** o comando normal é longo e depende da versão. Compare o valor completo e os argumentos com uma linha de base confiável da mesma build; alterar uma configuração crítica de subsistema também pode impedir a inicialização.

### Setup Execute {#setup_execute}

**ID da verificação:** `setup_execute` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê as entradas `SetupExecute` do Session Manager, cobrindo comandos relacionados à instalação configurados para a inicialização.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**Como investigar:** atividades de instalação, atualização e recuperação podem criar entradas legítimas. Compare com os logs de instalação e a imagem referenciada antes de atribuir a configuração à persistência.

### Redirecionamento de Startup em Shell Folders {#shell_folders_startup}

**ID da verificação:** `shell_folders_startup` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** compara `Startup` e `Common Startup` tanto em Shell Folders quanto em User Shell Folders com os locais padrão de Startup. Redirecionamentos fora do padrão são relatados, e o conteúdo coletado das pastas é inspecionado.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` e `User Shell Folders`: `Common Startup`.
- Configurações equivalentes de `Software\Microsoft\Windows\CurrentVersion\Explorer` no NTUSER.DAT: `Startup`, além dos arquivos capturados nos locais redirecionados.

**Como investigar:** uma pasta Startup redirecionada pode ocultar inicializações automáticas, mas o redirecionamento pode ser legítimo. O caminho precisa ser resolvido dentro das evidências capturadas; um destino não coletado é desconhecido, não uma prova de que a pasta estava vazia.

### Substituição do shell (Shell Launcher) {#shell_launcher}

**ID da verificação:** `shell_launcher` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê o valor `Shell` de políticas e o mapeamento Shell de boot do `system.ini` no SOFTWARE. Essas são superfícies alternativas de seleção do shell.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**Como investigar:** quiosques e appliances de aplicativos podem substituir o Explorer intencionalmente. Compare o shell selecionado com a função da máquina e confirme que a configuração estava em vigor para a sessão relevante.

### Pasta Startup {#startup_folder}

**ID da verificação:** `startup_folder` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** inspeciona os arquivos diretos nas pastas Startup padrão por usuário e comum, seguindo o primeiro redirecionamento de pasta Startup disponível. Os destinos dos atalhos são recuperados quando um `.lnk` coletado pode ser analisado.

**Evidências examinadas:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` e `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`.
- Redirecionamentos `Common Startup` / `Startup` aplicáveis nas configurações Shell Folders e User Shell Folders de máquina/usuário.

**Como investigar:** links e arquivos de Startup podem iniciar aplicativos legítimos na entrada da sessão. Inspecione os argumentos dos destinos, além dos nomes de arquivos. A pasta comum é um local de configuração compartilhado; isso não prova que o programa iniciado usou um token SYSTEM.

### Programa inicial dos Terminal Services {#ts_initial_program}

**ID da verificação:** `ts_initial_program` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê o `InitialProgram` das políticas dos Terminal Services e a configuração Run/RunOnce/RunOnceEx de instalação do Terminal Server da máquina, incluindo seções RunOnceEx aninhadas com limites.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | Todos os valores não vazios |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | Todos os valores não vazios |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | Todos os valores não vazios |

**Como investigar:** implantações de aplicativos remotos podem usar essas configurações intencionalmente. Estabeleça as sessões afetadas e siga os comandos selecionados até as evidências de processos; o registro por si só não mostra um logon remoto.

### Winlogon AppSetup {#winlogon_appsetup}

**ID da verificação:** `winlogon_appsetup` · **ATT&CK do catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**O que o Juntei verifica:** lê `AppSetup` do Winlogon, dividindo as entradas separadas por vírgulas em candidatos separados.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**Como investigar:** compare cada componente com o software de instalação/entrada na sessão do Windows esperado. Uma entrada da lista não é uma execução observada separada, e sua relevância pode depender da versão do Windows e do estado da implantação.

### Winlogon GinaDLL {#winlogon_gina_dll}

**ID da verificação:** `winlogon_gina_dll` · **ATT&CK do catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**O que o Juntei verifica:** lê a configuração legada de substituição `GinaDLL` do Winlogon. O GINA era um modelo de extensão de entrada na sessão de versões mais antigas do Windows.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**Como investigar:** isso é relevante principalmente para sistemas anteriores ao Vista. Uma chave remanescente em um sistema mais recente não é prova de uma extensão de logon efetiva; inspecione a versão de origem e qualquer DLL referenciada.

**Mais detalhes:** [documentação da Microsoft](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina).

### Winlogon MPNotify {#winlogon_mpnotify}

**ID da verificação:** `winlogon_mpnotify` · **ATT&CK do catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**O que o Juntei verifica:** lê a configuração de comando/biblioteca `mpnotify` do Winlogon, uma superfície de extensão relacionada à entrada na sessão.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**Como investigar:** compare o destino e os argumentos com uma linha de base do Windows correspondente. Verifique o comportamento específico da versão e correlacione com uma entrada na sessão posterior antes de afirmar que o código foi executado.

### Pacotes Notify do Winlogon {#winlogon_notify_packages}

**ID da verificação:** `winlogon_notify_packages` · **ATT&CK do catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**O que o Juntei verifica:** lê `DllName` de cada registro imediato de pacote `Notify` do Winlogon.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**Como investigar:** os pacotes de notificação são um mecanismo de extensão legado. Determine se a versão do Windows capturada os carrega e diferencie softwares de autenticação aprovados de bibliotecas sem explicação.

### Winlogon Shell {#winlogon_shell}

**ID da verificação:** `winlogon_shell` · **ATT&CK do catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**O que o Juntei verifica:** lê os valores `Shell` do Winlogon de toda a máquina e por usuário para expor a seleção do shell da área de trabalho.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**Como investigar:** o Explorer é comum, mas quiosques podem usar outro shell. Investigue comandos adicionados ou caminhos incomuns e determine qual configuração efetiva de usuário se aplicava na entrada na sessão.

### Winlogon System {#winlogon_system}

**ID da verificação:** `winlogon_system` · **ATT&CK do catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**O que o Juntei verifica:** lê o valor `System` do Winlogon e separa as entradas delimitadas por vírgulas em candidatos individuais.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**Como investigar:** trate isso como uma pista de configuração de entrada na sessão sensível à versão. Inspecione cada componente referenciado e corrobore com evidências de execução, em vez de tratar a lista como um histórico de execuções.

### Winlogon Taskman {#winlogon_taskman}

**ID da verificação:** `winlogon_taskman` · **ATT&CK do catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**O que o Juntei verifica:** lê os valores `Taskman` do Winlogon de máquina/usuário em busca de um comando alternativo de gerenciamento de tarefas.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**Como investigar:** determine qual comportamento do Windows consumiria a configuração e se o destino é esperado. Uma substituição configurada não prova que um usuário acionou esse recurso.

### Winlogon Userinit {#winlogon_userinit}

**ID da verificação:** `winlogon_userinit` · **ATT&CK do catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**O que o Juntei verifica:** lê os valores `Userinit` do Winlogon de máquina/usuário, dividindo os programas separados por vírgulas e ignorando itens finais vazios.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**Como investigar:** uma entrada normal geralmente aponta para `userinit.exe`. Inspecione entradas adicionais e caminhos alterados. Confirme o usuário aplicável e uma entrada na sessão posterior; `KeyLastWrite` pertence à chave inteira, não especificamente a este valor.

### Winlogon VMApplet {#winlogon_vmapplet}

**ID da verificação:** `winlogon_vmapplet` · **ATT&CK do catálogo:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**O que o Juntei verifica:** lê a configuração de comando `VMApplet` do Winlogon, cobrindo uma superfície mais antiga de inicialização de applets do sistema.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**Como investigar:** verifique a aplicabilidade ao sistema operacional e o applet esperado. Um comando inesperado merece investigação, mas não estabelece que a interface que o aciona foi aberta.

## Serviços e extensões do sistema { #services-and-system-extensions }

Os serviços do Windows e os componentes extensíveis do sistema oferecem muitas formas legítimas de carregar executáveis ou DLLs. Compare o destino configurado com o produto esperado, a conta de serviço, as configurações de inicialização e o histórico de instalação respaldado pelas fontes.

### DLL do DiagTrack {#diagtrack_dll}

**ID da verificação:** `diagtrack_dll` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê o `ImagePath` do serviço DiagTrack. Apesar do nome orientado a DLL no catálogo, esta verificação examina o comando de imagem do serviço configurado.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**Como investigar:** compare com a linha de base do serviço e com as verificações gerais de serviços. A mesma configuração pode produzir mais de um resultado de verificação; as contagens de candidatos não são contagens de ataques distintos.

### DLL do DiagTrack Listener {#diagtrack_listener_dll}

**ID da verificação:** `diagtrack_listener_dll` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê `FileName` da configuração do autologger DiagTrack-Listener. Esta é uma configuração de autologger do ETW, não uma evidência de que uma DLL foi carregada.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**Como investigar:** inspecione o valor bruto e a configuração de logs do Windows. O catálogo classifica isso como uma técnica de DLL, mas a consulta não valida uma DLL carregável nem prova execução por meio deste valor.

### DLL auxiliar do Netsh {#netsh_helper}

**ID da verificação:** `netsh_helper` · **ATT&CK do catálogo:** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**O que o Juntei verifica:** enumera os valores não vazios sob a chave de registro de auxiliares `NetSh` da máquina. Os auxiliares podem estender o `netsh.exe` com comandos baseados em DLLs.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | Todos os valores não vazios |

**Como investigar:** produtos de rede podem instalar auxiliares. Associe cada valor a um produto esperado e inspecione a biblioteca resolvida; em seguida, procure execuções relevantes do netsh ou carregamentos de imagens.

### Monitores de impressão {#print_monitors}

**ID da verificação:** `print_monitors` · **ATT&CK do catálogo:** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**O que o Juntei verifica:** lê `Driver` sob cada registro imediato de Print Monitor no conjunto de controle ativo.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**Como investigar:** softwares de impressão e monitores de porta estendem legitimamente o spooler. Compare a DLL e os metadados do produto com o inventário de impressoras e confirme o carregamento com evidências do spooler/de carregamento de imagens.

### Processadores de impressão {#print_processors}

**ID da verificação:** `print_processors` · **ATT&CK do catálogo:** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**O que o Juntei verifica:** lê `Driver` sob as árvores de registro de processadores de impressão x64 e x86.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**Como investigar:** pacotes de impressoras de terceiros podem adicionar processadores além dos padrões conhecidos do Windows. Revise ambas as arquiteturas, a DLL real e o software de impressora instalado antes de atribuir intenção maliciosa.

### Extensão do Service Control Manager {#scm_extension}

**ID da verificação:** `scm_extension` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** enumera os valores não vazios de `ServiceControlManagerExtension`, uma superfície de extensão do gerenciador de serviços.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | Todos os valores não vazios |

**Como investigar:** examine as bibliotecas nomeadas e se esta build do Windows consome a configuração. Correlacione com a atividade de boot/serviços; a verificação inventaria registros, em vez de provar que uma extensão foi carregada.

### DLL de plugin de nível de servidor do DNS {#server_level_plugin_dll}

**ID da verificação:** `server_level_plugin_dll` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê a configuração `ServerLevelPluginDll` do serviço DNS no conjunto de controle ativo do SYSTEM.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**Como investigar:** isso é relevante para sistemas que executam a função de Servidor DNS, incluindo alguns controladores de domínio. Investigue a DLL, a conta de serviço e as evidências de reinicialização/carregamento do serviço; uma função inativa ou ausente muda a conclusão.

### Comando de falha de serviço {#service_failure_command}

**ID da verificação:** `service_failure_command` · **ATT&CK do catálogo:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**O que o Juntei verifica:** lê o `FailureCommand` de cada serviço no conjunto de controle ativo, expondo um comando de recuperação configurado.

**Evidências examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`.

**Como investigar:** o comando só é executado quando as ações de recuperação do serviço o selecionam e a falha aplicável ocorre. Inspecione a configuração de recuperação e os registros de falhas/processos; o valor por si só não estabelece uma ação de recuperação ativa.

### DLL de agente de extensão SNMP {#snmp_extension_agent}

**ID da verificação:** `snmp_extension_agent` · **ATT&CK do catálogo:** [T1574](https://attack.mitre.org/techniques/T1574/)

**O que o Juntei verifica:** enumera os valores sob a chave de registro `ExtensionAgents` do serviço SNMP. Os valores podem identificar outro local do Registro em vez de um caminho direto de DLL.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | Todos os valores não vazios |

**Como investigar:** siga manualmente um registro indireto até sua biblioteca e verifique se o SNMP está instalado e em uso. Uma string não resolvida nesta saída não prova que a DLL esteja ausente ou seja maliciosa.

### Provedores de horário (Time Providers) {#time_providers}

**ID da verificação:** `time_providers` · **ATT&CK do catálogo:** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**O que o Juntei verifica:** lê `DllName` dos registros imediatos `TimeProviders` do Windows Time.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**Como investigar:** revise separadamente as configurações de habilitação do provedor e o estado do serviço Windows Time; a verificação do caminho da DLL não exige um provedor habilitado. Produtos de sincronização de horário podem fornecer substitutos legítimos.

### Serviço do Windows (ServiceDll) {#windows_service_dll}

**ID da verificação:** `windows_service_dll` · **ATT&CK do catálogo:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**O que o Juntei verifica:** lê `Parameters\ServiceDll` sob os serviços no conjunto de controle ativo do SYSTEM e anexa evidências de carimbos de data/hora específicas do serviço, quando disponíveis.

**Evidências examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`.

**Como investigar:** um host compartilhado como `svchost.exe` não é a DLL da carga útil. Inspecione a biblioteca, a conta de serviço, o estado de inicialização e as evidências correspondentes de instalação do serviço/carregamento de imagens; um registro não prova que o serviço foi executado.

**Mais detalhes:** [documentação da Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Serviço do Windows (ImagePath) {#windows_service_image_path}

**ID da verificação:** `windows_service_image_path` · **ATT&CK do catálogo:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**O que o Juntei verifica:** lê os comandos `ImagePath` dos serviços no conjunto de controle ativo. Drivers dos tipos 1, 2 ou 8 sem ImagePath mantêm um destino implícito `System32\drivers\<service>.sys`.

**Evidências examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath`, `Type` e os metadados de serviço de apoio.

**Como investigar:** revise a conta de serviço e as configurações de inicialização separadamente do rótulo de escopo de máquina da saída. Registros desabilitados ou de inicialização sob demanda continuam fazendo parte do inventário. Para os carimbos de data/hora, evidências pelo nome do serviço são preferíveis a caminhos de executáveis compartilhados ambíguos.

**Mais detalhes:** [documentação da Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### DLL de inicialização do serviço Windows Update {#wu_service_startup_dll}

**ID da verificação:** `wu_service_startup_dll` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê o `Parameters\ServiceDll` do serviço Windows Update no conjunto de controle ativo.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**Como investigar:** compare a biblioteca com a mesma build do Windows e o estado de manutenção instalado. Isto se sobrepõe ao inventário genérico de ServiceDll e não representa um segundo comprometimento distinto.

## Tarefas agendadas { #scheduled-tasks }

O Agendador de Tarefas armazena a configuração tanto em arquivos XML quanto no hive SOFTWARE. Leia essas fontes em conjunto. Uma tarefa pode estar registrada sem ter sido executada, e uma coleta incompleta pode se parecer com uma tarefa oculta ou fantasma.

### Tarefa agendada fantasma {#ghost_task}

**ID da verificação:** `ghost_task` · **ATT&CK do catálogo:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**O que o Juntei verifica:** compara os registros `Tree` e `Tasks` do TaskCache no SOFTWARE e, quando há arquivos de tarefas na coleta, procura os arquivos XML de tarefa correspondentes.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` e `TaskCache\Tasks`.
- Arquivos `Windows\System32\Tasks\<task path>` capturados, para a comparação com o sistema de arquivos.

**Como investigar:** contrapartes ausentes podem refletir tarefas excluídas/obsoletas, registros internos ou uma coleta incompleta. A ausência de XML coletado significa que a comparação com o sistema de arquivos não está disponível. Inspecione a cobertura e a discrepância exata antes de chamar uma tarefa de oculta ou maliciosa.

### Tarefa agendada oculta {#hidden_scheduled_task}

**ID da verificação:** `hidden_scheduled_task` · **ATT&CK do catálogo:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**O que o Juntei verifica:** verifica nos registros Tree do TaskCache a ausência do valor `SD` ou um descritor de segurança binário vazio, condições associadas a ocultar uma tarefa da enumeração comum.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: presença/dados de `SD`, com os metadados da contraparte no TaskCache.

**Como investigar:** um SD ausente/vazio é uma anomalia de configuração específica, não uma prova de que a tarefa foi executada ou de que é maliciosa. A configuração `Hidden` do XML e problemas arbitrários de ACL são condições diferentes e não são o predicado usado aqui.

**Mais detalhes:** [documentação da Microsoft](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/).

### Tarefa agendada (arquivos XML) {#scheduled_task_files}

**ID da verificação:** `scheduled_task_files` · **ATT&CK do catálogo:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**O que o Juntei verifica:** analisa o XML de tarefas capturado em busca de comandos/argumentos Exec e de IDs de classe/dados de manipuladores COM. Ele seleciona o principal por `Actions Context`/`Principal id`, mantém separadas as identidades de usuário e de grupo e relata SYSTEM, USER ou UNKNOWN. Ações COM não resolvidas permanecem visíveis.

**Evidências examinadas:**

- Arquivos XML `Windows\System32\Tasks\<task path>`, incluindo Exec, ComHandler e os campos de principal disponíveis.
- O `UsrClass.dat` capturado do usuário correspondente e `Software\Classes` do NTUSER, além de `Classes\CLSID` do SOFTWARE da máquina, nas visões nativa e de 32 bits.
- O `ProfileList` do SOFTWARE capturado para principais com SID. Principais nomeados preferem uma pasta `user.DOMAIN` correspondente única, depois sufixos do host local capturados e, por fim, a pasta `user` simples. As correspondências são explicitamente rotuladas como heurísticas; uma ambiguidade real permanece sem resolução e é registrada na constatação e na cobertura. Chaves voláteis do Registro não são necessárias.
- `CodeBase` gerenciado (incluindo subchaves de versão), associações de AppID e o `ImagePath` do serviço no SYSTEM para um `LocalService` de AppID.

Os candidatos a servidor COM incluem sua fonte no Registro em `Description`. Associações de AppID
apenas com o nome do executável permanecem sem resolução, em vez de receberem um diretório presumido. URLs `file:`
locais são decodificadas; URLs remotas são mantidas sem acesso à rede. As substituições de classes do usuário
têm escopo limitado a esse perfil, e um CLSID de usuário incompleto não recorre ao registro da máquina
que ele sombreia. Tarefas apenas de grupo mantêm os candidatos da máquina com a limitação de usuário
individual na descrição, sem acrescentar avisos rotineiros ao resumo de erros de cobertura.

**Como investigar:** inspecione os gatilhos, o estado de habilitação, o principal e o histórico de execução no XML de origem e nos logs de eventos. Uma tarefa criada pode nunca ser executada. Um manipulador COM não resolvido continua sendo um candidato, e dados de principal ausentes/apenas de grupo não devem ser interpretados como SYSTEM.

## WMI, BITS e outros gatilhos de eventos { #wmi-bits-and-other-event-triggers }

O Windows Management Instrumentation (WMI) oferece suporte ao gerenciamento do sistema. O Background Intelligent Transfer Service (BITS) oferece suporte a transferências em segundo plano. Ambos podem manter comandos acionados por eventos. Alguns mecanismos aguardam um evento de gerenciamento, uma notificação de transferência, a inicialização de um aplicativo ou uma ação do usuário. Um gatilho armazenado é uma evidência de configuração útil, mas sua presença não estabelece que o gatilho foi disparado.

### Backdoor dos recursos de acessibilidade {#accessibility_tools}

**ID da verificação:** `accessibility_tools` · **ATT&CK do catálogo:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**O que o Juntei verifica:** compara sete binários de acessibilidade coletados do System32 usando o OriginalFilename do PE e os metadados de certificado/catálogo, depois de exigir metadados de catálogo da Microsoft utilizáveis de um binário de referência coletado.

**Evidências examinadas:**

- `Windows\System32\sethc.exe`, `osk.exe`, `narrator.exe`, `magnify.exe`, `utilman.exe`, `atbroker.exe` e `DisplaySwitch.exe`.
- Metadados de referência de `Windows\System32\winlogon.exe`, `cmd.exe` ou `notepad.exe` coletados e os catálogos coletados aplicáveis.

**Como investigar:** os metadados podem sugerir um substituto renomeado, mas o Juntei não valida assinaturas nem a confiança de certificados. Uma coleta esparsa, sem os catálogos necessários, produz uma cobertura de comparação incompleta, em vez de tratar todo arquivo aparentemente não assinado como um backdoor.

### Recursos de acessibilidade {#assistive_technology}

**ID da verificação:** `assistive_technology` · **ATT&CK do catálogo:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**O que o Juntei verifica:** lê os comandos `StartExe` mais `StartParams` dos recursos de acessibilidade registrados nas visões nativa/de 32 bits do SOFTWARE, e as entradas `Configuration` de acessibilidade habilitadas dos hives de usuário e DEFAULT. Espaços reservados StartExe numéricos/não string são ignorados.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` e `StartParams`; também a visão `Wow6432Node` da máquina.
- `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` no NTUSER.DAT e a configuração equivalente no hive DEFAULT.

**Como investigar:** softwares de acessibilidade são legítimos. Diferencie o nome de um recurso habilitado do registro de seu executável real e, em seguida, inspecione o destino e as evidências de execução na entrada na sessão/área de trabalho. Isso é diferente de substituir o próprio executável de acessibilidade.

**Mais detalhes:** [referência da técnica (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/).

### Comando de notificação de trabalho do BITS {#bits_notify_command}

**ID da verificação:** `bits_notify_command` · **ATT&CK do catálogo:** [T1197](https://attack.mitre.org/techniques/T1197/)

**O que o Juntei verifica:** recupera registros de trabalhos do BITS em strings com contagem de `qmgr.db`, `qmgr0.dat` ou `qmgr1.dat` capturados, mantendo os programas de notificação não vazios, os parâmetros, os IDs de trabalho e os SIDs dos proprietários.

**Evidências examinadas:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`, `qmgr0.dat` e `qmgr1.dat`.

**Como investigar:** estes são registros recuperados por carving: o estado alocado/atual do trabalho não é validado, e trabalhos obsoletos/excluídos podem aparecer. O horário de gravação do banco de dados no MFT é uma evidência fraca do contêiner, não o horário de execução da notificação. Os SIDs dos proprietários distinguem SYSTEM, serviço local, serviço de rede e usuários.

### AutoRun do Command Processor {#cmd_autorun}

**ID da verificação:** `cmd_autorun` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** lê os comandos `AutoRun` do Command Processor de máquina/usuário, uma configuração que pode afetar a inicialização do prompt de comando.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**Como investigar:** ferramentas de desenvolvimento e personalizações do shell podem explicar um valor. `cmd.exe /d` ignora o AutoRun, portanto a configuração por si só não prova que ele foi executado sempre que o cmd.exe apareceu. Inspecione o conteúdo do comando e os argumentos do processo que o acionou.

**Mais detalhes:** [documentação da Microsoft](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd).

### Persistência via ErrorHandler.cmd {#error_handler_cmd}

**ID da verificação:** `error_handler_cmd` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** relata a presença do script `Windows\System32\ErrorHandler.cmd` coletado como candidato a persistência por manipulador de erros.

**Evidências examinadas:**

- `Windows\System32\ErrorHandler.cmd`.

**Como investigar:** a presença por si só não prova que o caminho de recuperação/erro aplicável do Windows o invocou. Leia o script, inspecione sua procedência e correlacione com evidências de instalação/erros/processos.

### Fluxos do Power Automate Desktop {#power_automate}

**ID da verificação:** `power_automate` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** encontra os diretórios de fluxos/scripts capturados sob os repositórios `flows` e `scripts` do Power Automate Desktop de cada usuário, relatando cada fluxo descoberto uma única vez.

**Evidências examinadas:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` e `scripts\<flow>\...`.

**Como investigar:** isso identifica automações armazenadas, não todos os gatilhos agendados nem um fluxo executado. Inspecione o conteúdo do fluxo e a configuração de inicialização externa antes de tratá-lo como persistência automática ou maliciosa.

### Perfil do PowerShell {#powershell_profiles}

**ID da verificação:** `powershell_profiles` · **ATT&CK do catálogo:** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**O que o Juntei verifica:** encontra arquivos do tipo profile.ps1 nos diretórios de instalação/perfil reconhecidos do Windows PowerShell e do PowerShell, incluindo os Documentos do usuário, os Documentos do OneDrive descobertos e os redirecionamentos de Documentos coletados.

**Evidências examinadas:**

- Arquivos terminados em `profile.ps1` diretamente em `Windows\System32\WindowsPowerShell\v1.0` e `Windows\SysWOW64\WindowsPowerShell\v1.0`.
- `Documents\WindowsPowerShell` e `Documents\PowerShell` por usuário, incluindo os redirecionamentos de Documentos capturados e as pastas de Documentos do OneDrive descobertas.
- Arquivos terminados em `profile.ps1` diretamente nos diretórios de versão sob `Program Files\PowerShell` ou `Program Files (x86)\PowerShell`.

**Como investigar:** os perfis podem conter personalizações inofensivas. Leia o script e siga as cargas úteis invocadas. Iniciar o PowerShell com `-NoProfile` os ignora; a presença do arquivo por si só não prova a execução do perfil nem conteúdo malicioso.

### Sequestro da proteção de tela {#screensaver}

**ID da verificação:** `screensaver` · **ATT&CK do catálogo:** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**O que o Juntei verifica:** lê o valor `SCRNSAVE.EXE` de Desktop de cada usuário, identificando a imagem da proteção de tela configurada.

**Evidências examinadas:**

- `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE` no NTUSER.DAT.

**Como investigar:** compare com as proteções de tela aprovadas e inspecione o destino. Outras configurações da área de trabalho, o estado da sessão e o tempo limite determinam se ela pode ser acionada; um caminho salvo não prova que uma proteção de tela foi iniciada.

### Comando do TelemetryController {#telemetry_controller}

**ID da verificação:** `telemetry_controller` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** lê `Binary`, `Command` e `MaintenanceCommand` do TelemetryController e de seus registros filhos imediatos.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` e filhos imediatos: `Binary`, `Command`, `MaintenanceCommand`.

**Como investigar:** compare o executável e os argumentos selecionados com a mesma build do Windows e o software de manutenção. Um comando no Registro não demonstra que um gatilho de manutenção foi disparado.

### Perfis personalizados do Windows Terminal {#windows_terminal}

**ID da verificação:** `windows_terminal` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** analisa as configurações JSON/JSONC do Windows Terminal empacotado, preview e não empacotado e relata as configurações `commandline` não vazias de perfis/padrões.

**Evidências examinadas:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` e o equivalente do WindowsTerminalPreview.
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: arrays de perfis legados e campos commandline de `profiles.list` e `profiles.defaults`.

**Como investigar:** shells comuns aparecem aqui naturalmente. Determine o perfil selecionado/padrão e inspecione comandos ou argumentos de inicialização incomuns. Apenas salvar um perfil não o inicia.

### Assinatura de eventos WMI {#wmi_event_subscription}

**ID da verificação:** `wmi_event_subscription` · **ATT&CK do catálogo:** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**O que o Juntei verifica:** recupera por carving instâncias reconhecidas de consumidores de eventos e referências de vinculação filtro-consumidor do repositório WMI OBJECTS.DATA capturado. As classes reconhecidas incluem consumidores de linha de comando, de script ativo, de arquivo de log, de log de eventos e SMTP.

**Evidências examinadas:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` e `Repository\FS\OBJECTS.DATA`.

**Como investigar:** uma ocorrência bruta no repositório pode ser obsoleta, não vinculada ou histórica; a saída preserva essa ressalva. O carving não reconstrói um grafo completo de assinaturas ativas nem prova que o gatilho foi disparado. O horário de gravação do arquivo do repositório não é um horário de criação/execução da assinatura.

## Explorer, COM e manipuladores de arquivos ou URLs { #explorer-com-and-file-or-url-handlers }

O Component Object Model (COM) permite que aplicativos ativem componentes de software registrados. O shell do Windows usa manipuladores registrados para abrir arquivos, URLs, pastas e objetos especiais. Os identificadores COM (CLSIDs) identificam componentes; os ProgIDs dão a alguns componentes nomes legíveis. Siga cada registro até o comando ou DLL que ele seleciona.

### Sequestro de App Paths {#app_paths}

**ID da verificação:** `app_paths` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** lê os valores padrão sob os registros `App Paths` de toda a máquina, que podem redirecionar a localização de aplicativos em inicializações feitas pelo shell.

**Evidências examinadas:**

- Valor padrão de `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>`.

**Como investigar:** um registro em App Paths é comum para aplicativos instalados e não é uma reescrita geral de todas as inicializações de processos. Compare o nome registrado e a imagem com o software esperado e, em seguida, examine as evidências relevantes do shell/processos.

### Sequestro de objetos COM {#com_treat_as}

**ID da verificação:** `com_treat_as` · **ATT&CK do catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**O que o Juntei verifica:** encontra redirecionamentos COM `TreatAs` e registros InprocServer32/LocalServer32 por usuário que sombreiam um CLSID da máquina, nos repositórios de classes nativos/de 32 bits compatíveis. Os destinos de TreatAs são resolvidos usando as classes do mesmo usuário e as classes da máquina.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` e a visão `Classes\Wow6432Node` da máquina.
- `Software\Classes\CLSID` por usuário no NTUSER.DAT e `CLSID` no UsrClass.dat, incluindo suas visões `Wow6432Node` implementadas: valores padrão de `TreatAs`, `InprocServer32` e `LocalServer32`.

**Como investigar:** substituições por usuário podem não exigir direitos de administrador, mas softwares legítimos também podem registrá-las. Siga a classe selecionada e a DLL/executável, avalie a arquitetura e o contexto de ativação e corrobore a ativação COM real de forma independente.

### Sequestro de manipulador da Limpeza de Disco {#disk_cleanup_handler}

**ID da verificação:** `disk_cleanup_handler` · **ATT&CK do catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**O que o Juntei verifica:** lê os CLSIDs dos manipuladores VolumeCaches da Limpeza de Disco e segue os registros COM InprocServer32 resolvíveis da máquina até suas bibliotecas.

**Evidências examinadas:**

- CLSID padrão de `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` e `Classes\CLSID\<CLSID>\InprocServer32`.

**Como investigar:** o manipulador precisa ser invocado pelo fluxo de limpeza relevante para ser executado. Manipuladores não resolvidos não são emitidos por esta verificação específica; portanto, use o analisador de Registro para inspecionar seus registros brutos se a captura estiver incompleta.

### Substituição de AppKey do Explorer {#explorer_app_key}

**ID da verificação:** `explorer_app_key` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê os valores `ShellExecute` e `Association` sob os filhos imediatos de `AppKey` do Explorer, cobrindo comandos/ações selecionados por teclas de aplicativo.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` e `Association`.

**Como investigar:** utilitários de teclado e ferramentas de OEM podem definir esses valores. Um nome de associação pode exigir outro pivô no Registro, em vez de ser um caminho de arquivo. Verifique a ação do usuário que o acionaria.

### Browser Helper Objects {#explorer_bho}

**ID da verificação:** `explorer_bho` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** lê os registros de CLSID de Browser Helper Objects e resolve as bibliotecas InprocServer32 da máquina quando disponíveis; IDs de classe não resolvidos permanecem visíveis.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` e o valor padrão `Classes\CLSID\<CLSID>\InprocServer32` da máquina.

**Como investigar:** isso está vinculado ao comportamento compatível do navegador/Explorer e é especialmente relevante para ambientes legados. Extensões aprovadas de navegador/segurança podem aparecer. Um BHO registrado não é prova de que um navegador atual o carregou.

### Sequestro de CLSID da área de trabalho do Explorer {#explorer_clsid_hijack}

**ID da verificação:** `explorer_clsid_hijack` · **ATT&CK do catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**O que o Juntei verifica:** lê os comandos shell-open configurados nos três CLSIDs especiais de namespace do Explorer listados, nos repositórios de classes de máquina/usuário.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**Como investigar:** compare os manipuladores de comando de Área de Trabalho, Meu Computador e Meus Documentos com uma linha de base correspondente. Isto verifica CLSIDs e caminhos selecionados, não todos os sequestros possíveis de namespace do Explorer.

### Manipuladores de menu de contexto do Explorer {#explorer_context_menu}

**ID da verificação:** `explorer_context_menu` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** enumera os manipuladores de menu de contexto de Directory, Directory Background, Folder, Drive, AllFilesystemObjects e `*`, seguindo os IDs de classe até os registros de DLL coletados quando possível.

**Evidências examinadas:**

- Repositórios de classes `HKLM\SOFTWARE\Classes` da máquina e UsrClass.dat do usuário: valor padrão de `<subject>\shellex\ContextMenuHandlers\<handler>` e os registros de bibliotecas COM correspondentes.

**Como investigar:** extensões do menu de clique com o botão direito de ferramentas de compactação, produtos de segurança e clientes de nuvem são comuns. Inspecione o manipulador real e a interação que o aciona. GUIDs não resolvidos continuam sendo pistas úteis, e não prova de que a DLL não existia.

### Sequestro de associações de arquivos {#file_association_hijack}

**ID da verificação:** `file_association_hijack` · **ATT&CK do catálogo:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**O que o Juntei verifica:** examina os registros `shell\open\command` diretos das extensões e dos ProgIDs resolvidos para `.txt`, `.pdf`, `.doc`, `.docx`, `.html`, `.htm`, `.js`, `.vbs`, `.hta`, `.exe`, `.bat`, `.cmd` e `.ps1`.

**Evidências examinadas:**

- Repositórios de classes `HKLM\SOFTWARE\Classes` da máquina e UsrClass.dat do usuário: padrões de `<extension>` selecionados e padrões de `<extension-or-ProgID>\shell\open\command`.

**Como investigar:** compare com o aplicativo esperado e inspecione os argumentos do comando. Este é um conjunto definido de extensões, não todos os tipos de arquivo nem a resolução efetiva completa de UserChoice. Um comando salvo não prova que um documento foi aberto.

### Sequestro de manipuladores de protocolo {#protocol_handler_hijack}

**ID da verificação:** `protocol_handler_hijack` · **ATT&CK do catálogo:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**O que o Juntei verifica:** lê os comandos de abertura de URL para http, https, mailto, ms-msdt e ms-officecmd, além dos registros `URL Protocol` descobertos e das substituições de usuário aplicáveis.

**Evidências examinadas:**

- Repositórios de classes `HKLM\SOFTWARE\Classes` da máquina e UsrClass.dat do usuário: marcadores `URL Protocol` e valores padrão de `<scheme>\shell\open\command`.

**Como investigar:** navegadores e instaladores de aplicativos registram esquemas legitimamente. Siga o comando e os argumentos, avalie a finalidade esperada do esquema e correlacione com evidências de abertura de URLs/processos.

### Manipulador de extensão COM da Lixeira {#recycle_bin_com_extension}

**ID da verificação:** `recycle_bin_com_extension` · **ATT&CK do catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**O que o Juntei verifica:** lê as substituições de comandos open/empty/explore do CLSID da Lixeira listadas e os registros imediatos de manipuladores de menu de contexto/arrastar e soltar.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**Como investigar:** compare com as extensões de shell aprovadas. Alguns valores relatados são identificadores COM, e não uma biblioteca resolvida diretamente; investigue a classe referenciada e a ação do usuário antes de afirmar que houve execução de código.

### Sequestro do manipulador do protocolo de pesquisa {#search_protocol_handler}

**ID da verificação:** `search_protocol_handler` · **ATT&CK do catálogo:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**O que o Juntei verifica:** lê o manipulador padrão `search-ms\shell\open\command` de máquina/usuário.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**Como investigar:** uma alteração do manipulador pode mudar o que acontece quando uma URL search-ms é aberta, mas sua presença não é prova de entrega de um exploit. Compare com os padrões do Windows e investigue a atividade de URLs/processos que a consumiria.

### SharedTaskScheduler {#shared_task_scheduler}

**ID da verificação:** `shared_task_scheduler` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** enumera os valores não vazios de `SharedTaskScheduler` do Explorer como componentes de shell registrados.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | Todos os valores não vazios |

**Como investigar:** os valores podem referenciar classes COM em vez de um caminho de executável. Siga seus registros de servidor e verifique o comportamento do sistema operacional, a procedência da extensão e as evidências relevantes de carregamento pelo shell.

### ShellExecuteHooks {#shell_execute_hooks}

**ID da verificação:** `shell_execute_hooks` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** enumera os valores não vazios de `ShellExecuteHooks` do Explorer, cobrindo registros legados de hooks de execução do shell.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | Todos os valores não vazios |

**Como investigar:** resolva qualquer CLSID referenciado e verifique se a build do Windows e as políticas permitem o hook. Um registro legado pode estar sem uso; o valor em si não é uma execução observada pelo shell.

### Persistência COM por bibliotecas e pastas CLSID do Menu Iniciar {#start_menu_com}

**ID da verificação:** `start_menu_com` · **ATT&CK do catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**O que o Juntei verifica:** encontra arquivos XML de biblioteca `.library-ms` cuja URL `simpleLocation` referencia
um CLSID COM, e diretórios nomeados `<name>.{CLSID}`, sob
`Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu`. A interação do Explorer com esses
objetos pode ativar código COM. A verificação inclui diretórios capturados vazios; ela não segue
links simbólicos. Arquivos de biblioteca comuns fora do Menu Iniciar e GUIDs em comentários XML não são
relatados por esta verificação.

**Evidências examinadas:**

- Arquivos `.library-ms` e entradas de diretório nomeadas com CLSID no Menu Iniciar de cada usuário.
- Os registros de classes capturados do usuário correspondente e o SOFTWARE da máquina, separadamente por arquitetura.
- Configuração gerenciada de CodeBase/AppID/serviço, por meio do mesmo resolvedor usado para tarefas agendadas.

**Como investigar:** `Path` e `SourceFile` identificam o artefato que aciona a ativação. `Value`
contém o comando do servidor candidato, ou o CLSID se a resolução não estiver disponível. `Description`
vincula o gatilho ao CLSID e à fonte capturada no Registro. `ResolvedPath`/`Sha256` descrevem apenas
uma carga útil resolvida e coletada. Uma associação de AppID apenas com o nome do executável permanece sem resolução.
Um gatilho pode ser legítimo; sua existência não prova uma entrada na sessão, ativação, execução ou ataque.
Inspecione a URL da biblioteca, o registro da classe, a carga útil, o contexto do usuário e logs de execução independentes.

**Cuidados na coleta:** coletas apenas de arquivos podem omitir diretórios vazios. Os caminhos padrão do Menu Iniciar
do usuário são examinados; Menus Iniciar redirecionados e outros locais de namespace do Explorer não são cobertos
por esta verificação. Hives ausentes ou registros não resolvidos produzem cobertura parcial, mantendo
o candidato. A arquitetura da tarefa, a elevação e as políticas podem alterar o comportamento efetivo da localização COM.

**Mais detalhes:** [proposta original e crédito ao pesquisador](https://github.com/last-byte/PersistenceSniper/pull/32),
[visões mescladas COM por usuário/máquina](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root),
[sequestro de COM no MITRE](https://attack.mitre.org/techniques/T1546/015/).

### Sequestro de COM via TypeLib {#typelib_hijack}

**ID da verificação:** `typelib_hijack` · **ATT&CK do catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**O que o Juntei verifica:** lê os registros win32/win64 de GUID/versão/localidade de TypeLib no UsrClass.dat de cada usuário, mantendo caminhos e valores de script moniker. Os índices numéricos de recursos no final são removidos apenas para a resolução do arquivo de destino.

**Evidências examinadas:**

- Valores padrão de `TypeLib\<GUID>\<version>\<locale>\win32` e `win64` em cada UsrClass.dat capturado.

**Como investigar:** uma biblioteca de tipos pode apontar para uma DLL legítima, um arquivo alternativo ou um script moniker. A verificação inventaria esses registros; avalie o aplicativo afetado e corrobore o carregamento da biblioteca de tipos, em vez de presumir que todo caminho fora do padrão é um ataque.

## Aplicativos e suplementos do Office { #office-applications-and-add-ins }

O Office oferece suporte a suplementos, modelos e extensões específicas de aplicativos legítimos. Um registro suspeito pode depender de uma versão específica do Office, de um aplicativo, de uma configuração de carregamento ou da interação do usuário. A configuração por si só não estabelece a execução de macros.

### Registro de suplementos do Office {#office_addins}

**ID da verificação:** `office_addins` · **ATT&CK do catálogo:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**O que o Juntei verifica:** enumera os suplementos do Word, Excel, PowerPoint, Outlook e Access nas raízes do Office sem versão e 14.0/15.0/16.0, incluindo o repositório Click-to-Run da máquina implementado e as raízes de usuário. Ele mantém os valores explícitos Manifest/FileName/Path ou resolve as entradas elegíveis baseadas em COM, com o contexto de LoadBehavior.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` e o equivalente Click-to-Run da máquina implementado.
- `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` no NTUSER.DAT; classes COM correspondentes de máquina/usuário do SOFTWARE, do NTUSER.DAT e do UsrClass.dat.

**Como investigar:** suplementos são softwares empresariais comuns. Revise LoadBehavior, aplicativo/versão, manifestos, cargas úteis reais e o fornecedor esperado. A verificação não garante que todos os formatos de suplementos instalados ou todas as visões de registro tenham sido coletados ou carregados.

### Sequestro de suplemento de IA do Office {#office_ai_hijack}

**ID da verificação:** `office_ai_hijack` · **ATT&CK do catálogo:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**O que o Juntei verifica:** enumera os valores no local de registro Common AI do Office 16.0 Click-to-Run.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: todos os valores.

**Como investigar:** compare valores e caminhos incomuns com a build do Office instalada. A presença de um valor não mostra qual recurso do Office o consumiu nem prova que ele selecionou código executável.

### Substituição de DLL do Office {#office_dll_override}

**ID da verificação:** `office_dll_override` · **ATT&CK do catálogo:** [T1137](https://attack.mitre.org/techniques/T1137/)

**O que o Juntei verifica:** lê `WwlibtDll` e `PPCoreTDLL` sob as raízes do Word/PowerPoint para as configurações sem versão e 14.0/15.0/16.0, incluindo as raízes Click-to-Run e de usuário implementadas.

**Evidências examinadas:**

- Configurações `Microsoft\Office\[<version>\]Word` e `PowerPoint` de máquina/usuário, além da árvore do Office Click-to-Run da máquina implementada: `WwlibtDll` e `PPCoreTDLL`.

**Como investigar:** determine qual aplicativo/build reconhece a substituição, inspecione a biblioteca e sua procedência e correlacione com uma inicialização do Office ou um carregamento de imagem. Uma substituição configurada, por si só, não é prova de execução.

### Modelos padrão do Office {#office_templates}

**ID da verificação:** `office_templates` · **ATT&CK do catálogo:** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**O que o Juntei verifica:** inventaria os arquivos Normal.dotm/NormalEmail.dotm coletados e os arquivos localizados diretamente nas pastas reconhecidas Startup do Word e XLSTART do Excel, incluindo as pastas de inicialização da instalação do Office.

**Evidências examinadas:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` e `NormalEmail.dotm`.
- Arquivos diretamente em `Microsoft\Word\STARTUP` e `Microsoft\Excel\XLSTART` do usuário, sob o AppData Roaming.
- Arquivos diretamente nas pastas reconhecidas do Office `<version>\STARTUP` / `XLSTART` e `root\<version>\STARTUP` / `XLSTART` sob Program Files e Program Files (x86).

**Como investigar:** um modelo pode ser legítimo ou não conter nenhum conteúdo executável. O Juntei não extrai nem executa macros nesta verificação. Inspecione a estrutura interna do documento, a política de macros e a atividade do aplicativo; um caminho de instalação da máquina não implica um token de processo SYSTEM.

### DLL do Office Test {#office_test_dll}

**ID da verificação:** `office_test_dll` · **ATT&CK do catálogo:** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**O que o Juntei verifica:** enumera os valores da chave Office Test Special Perf nas visões de máquina/usuário e de máquina de 32 bits listadas.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | Todos os valores não vazios |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | Todos os valores não vazios |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | Todos os valores não vazios |

**Como investigar:** trate substituições inesperadas como uma pista que exige validação da versão do Office e da carga útil. A verificação relata a configuração, em vez de testar se uma instalação específica do Office carrega o valor.

### Ataque à página inicial do Outlook {#outlook_home_page}

**ID da verificação:** `outlook_home_page` · **ATT&CK do catálogo:** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**O que o Juntei verifica:** lê `URL` sob os filhos de WebView do Outlook do usuário para as versões 14.0, 15.0 e 16.0 do Office.

**Evidências examinadas:**

- `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL` no NTUSER.DAT.

**Como investigar:** a página inicial de uma pasta pode ser legítima, e o comportamento varia conforme a versão do Outlook e as atualizações de segurança. Inspecione a URL/conteúdo e a política efetiva e, em seguida, correlacione com evidências do Outlook/web/processos.

### Sequestro de DLL do VBA Monitor {#vba_monitors}

**ID da verificação:** `vba_monitors` · **ATT&CK do catálogo:** [T1137](https://attack.mitre.org/techniques/T1137/)

**O que o Juntei verifica:** lê os valores InprocServer32 dos dois CLSIDs relacionados ao VBA monitor implementados, nos repositórios de classes da máquina e por usuário.

**Evidências examinadas:**

- `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` e `{0002E157-0000-0000-C000-000000000046}` no SOFTWARE da máquina, além dos registros equivalentes no UsrClass.dat do usuário.

**Como investigar:** compare as bibliotecas selecionadas com o ambiente Office/VBA instalado. Esta é uma verificação direcionada de CLSIDs, não um inventário de todas as extensões VBA nem uma prova de que uma macro foi executada.

## Depuradores, manipuladores de falhas e shims de compatibilidade { #debuggers-crash-handlers-and-compatibility-shims }

Um depurador ou manipulador de erros pode ser executado quando um aplicativo inicia, falha, trava ou é encerrado. Os bancos de dados de compatibilidade de aplicativos também podem afetar a inicialização. Investigue as configurações pré-requisitos e o programa que aciona o mecanismo junto com o comando relatado.

### Sequestro do depurador AeDebug {#ae_debug}

**ID da verificação:** `ae_debug` · **ATT&CK do catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**O que o Juntei verifica:** lê os comandos `Debugger` do AeDebug nativo e de 32 bits usados pela configuração de depuração just-in-time de falhas.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**Como investigar:** softwares de desenvolvimento/depuração podem definir isso legitimamente. Examine o comando completo do depurador e os pré-requisitos para acionamento por falhas e, em seguida, correlacione com eventos de erro de aplicativo e de processos.

### Depurador de processos protegidos do AeDebug {#ae_debug_protected}

**ID da verificação:** `ae_debug_protected` · **ATT&CK do catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**O que o Juntei verifica:** lê os comandos `Debugger` do AeDebugProtected nativo e de 32 bits, referentes à superfície de depuração de processos protegidos.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**Como investigar:** determine se a build do sistema operacional e a política de proteção podem usar o comando configurado. Um depurador salvo não prova que um processo protegido falhou nem que o comando foi executado.

### Depurador do Image File Execution Options {#ifeo_debugger}

**ID da verificação:** `ifeo_debugger` · **ATT&CK do catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**O que o Juntei verifica:** pesquisa os registros do Image File Execution Options nativos/de 32 bits, incluindo chaves de filtro aninhadas, em busca de valores `Debugger`.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`, incluindo chaves de filtro aninhadas e a visão `Wow6432Node` da máquina.

**Como investigar:** um depurador pode ser selecionado quando a imagem nomeada é iniciada. Inspecione o escopo da imagem/filtro e os argumentos do depurador; ferramentas de desenvolvimento também podem usar o IFEO. Rastreie a inicialização do destino para estabelecer a execução.

### NTDLL delegada do IFEO {#ifeo_delegated_ntdll}

**ID da verificação:** `ifeo_delegated_ntdll` · **ATT&CK do catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**O que o Juntei verifica:** pesquisa os registros do Image File Execution Options nativos/de 32 bits, incluindo chaves aninhadas, em busca de substituições `VerifierDlls`.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`, incluindo chaves aninhadas e a visão `Wow6432Node` da máquina.

**Como investigar:** o predicado da verificação é o valor VerifierDlls configurado; ela não prova que todos os pré-requisitos de verificador/delegação estejam habilitados. Revise os flags aplicáveis, a versão do Windows e a carga útil antes de inferir um caminho de injeção funcional.

### Monitor de encerramento silencioso de processos {#ifeo_silent_process_exit}

**ID da verificação:** `ifeo_silent_process_exit` · **ATT&CK do catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**O que o Juntei verifica:** pesquisa os registros SilentProcessExit nativos/de 32 bits em busca de `MonitorProcess`, incluindo chaves aninhadas.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`, incluindo chaves aninhadas e a visão `Wow6432Node` da máquina.

**Como investigar:** o monitoramento de encerramento silencioso também exige a configuração de relatório/IFEO relevante. Inspecione essas configurações e os eventos de encerramento da imagem monitorada; ferramentas de diagnóstico legítimas podem explicar o monitor.

### Shimming de aplicativos {#installed_sdb}

**ID da verificação:** `installed_sdb` · **ATT&CK do catálogo:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**O que o Juntei verifica:** lê `DatabasePath` dos registros imediatos de InstalledSDB referentes a bancos de dados de compatibilidade de aplicativos instalados.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**Como investigar:** correções de compatibilidade podem ser legítimas. Inspecione o conteúdo do SDB, os executáveis visados e o histórico de instalação. O registro de um banco de dados não identifica seus shims nem prova que um aplicativo correspondente foi executado.

### Sequestro do depurador do LSM {#lsm_debugger}

**ID da verificação:** `lsm_debugger` · **ATT&CK do catálogo:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**O que o Juntei verifica:** lê `MonitorProcess` especificamente sob SilentProcessExit para `lsm.exe`, nas visões de máquina nativa e de 32 bits.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**Como investigar:** isto se sobrepõe ao inventário mais amplo de SilentProcessExit. Confirme os flags pré-requisitos e um encerramento relevante do LSM antes de tratar a configuração como persistência executada; conte a mesma entrada uma única vez ao avaliar constatações distintas.

### DLLs auxiliares do MiniDump {#minidump_auxiliary_dlls}

**ID da verificação:** `minidump_auxiliary_dlls` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** enumera os valores nomeados e não vazios em MiniDumpAuxiliaryDlls, preservando o nome do valor como contexto do módulo acionador e os dados do valor como o destino selecionado.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: nomes de valores dos módulos acionadores e dados de destino.

**Como investigar:** inspecione tanto o módulo acionador quanto a DLL auxiliar. Ferramentas de falhas/minidump podem explicar uma entrada, e um registro não mostra que uma operação de despejo carregou a biblioteca.

### Shimming de aplicativos {#shim_custom}

**ID da verificação:** `shim_custom` · **ATT&CK do catálogo:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**O que o Juntei verifica:** lê os valores com nome `.sdb` sob as chaves imediatas de aplicativos Custom do AppCompatFlags, preservando a associação entre aplicativo e banco de dados.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: nomes de valores terminados em `.sdb`.

**Como investigar:** combine essas associações com o InstalledSDB e o banco de dados de compatibilidade real. Uma associação por nome de arquivo, por si só, não revela o comportamento do banco de dados nem demonstra a inicialização de um aplicativo correspondente.

### Sequestro do depurador do WER {#wer_debugger}

**ID da verificação:** `wer_debugger` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** lê os valores `Debugger` do Relatório de Erros do Windows nativo/de 32 bits.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**Como investigar:** softwares de diagnóstico legítimos podem configurar um depurador. Investigue o comando completo e os eventos de falha relevantes; o registro não prova que o Relatório de Erros do Windows o iniciou.

### Sequestro do depurador de travamentos do WER {#wer_hangs}

**ID da verificação:** `wer_hangs` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** lê os valores `Debugger` de Hangs do Relatório de Erros do Windows nativo/de 32 bits.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**Como investigar:** verifique se um fluxo de tratamento de travamentos invocaria essa configuração e inspecione o programa de destino. Correlacione com registros de travamento de aplicativos e de processos, em vez de tratar uma gravação no Registro como uma execução do depurador.

### Sequestro do ReflectDebugger do WER {#wer_reflect_debugger}

**ID da verificação:** `wer_reflect_debugger` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** lê os valores `ReflectDebugger` do Relatório de Erros do Windows nativo/de 32 bits.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**Como investigar:** compare com a configuração de diagnóstico aprovada e estabeleça a aplicabilidade quanto à versão do Windows/ao gatilho. Inspecione os argumentos do comando e busque corroboração em falhas/processos antes de atribuir a execução.

### Módulos de exceção em tempo de execução do WER {#wer_runtime_exception}

**ID da verificação:** `wer_runtime_exception` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** enumera os nomes de valores sob RuntimeExceptionHelperModules nativo/de 32 bits; aqui, o caminho da biblioteca está no nome do valor, não necessariamente em seus dados.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` e a visão `Wow6432Node` da máquina: nomes de valores.

**Como investigar:** inspecione o módulo nomeado e o contexto do aplicativo/runtime. Um pequeno valor numérico nos dados não é o caminho da carga útil, e o registro por si só não prova que o auxiliar foi carregado.

## Configurações do runtime .NET e de profilers { #net-runtime-and-profiler-settings }

Runtimes gerenciados podem carregar profilers, assemblies de inicialização ou bibliotecas de depuração antes do código do aplicativo. Essas configurações também são usadas por desenvolvedores e produtos de monitoramento. Verifique o runtime, a arquitetura do processo, os flags de habilitação e se um processo recém-iniciado poderia herdar a configuração.

### Injeção via AppDomainManager {#appdomain_manager}

**ID da verificação:** `appdomain_manager` · **ATT&CK do catálogo:** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**O que o Juntei verifica:** lê as variáveis de ambiente `APPDOMAIN_MANAGER_ASM` e `APPDOMAIN_MANAGER_TYPE` de máquina/usuário, cobrindo a seleção de assembly e de tipo para um gerenciador de AppDomain .NET personalizado.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**Como investigar:** aplicativos podem usar gerenciadores personalizados intencionalmente. Resolva o assembly e o tipo em conjunto e verifique a compatibilidade com o runtime e a herança entre processos; nem uma única variável nem o carimbo de data/hora de sua chave provam que um aplicativo gerenciado a carregou.

### Sequestro do profiler do .NET CLR {#cor_profiler}

**ID da verificação:** `cor_profiler` · **ATT&CK do catálogo:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**O que o Juntei verifica:** lê as configurações de ambiente de profiling COR de máquina/usuário e as entradas nomeadas correspondentes nas multi-strings Environment de serviços individuais. Ele mantém os flags de habilitação, os identificadores de profiler e os caminhos específicos de cada arquitetura.

**Evidências examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` e o `Environment` de cada NTUSER.DAT de usuário capturado.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: configurações COR nomeadas na multi-string.

**Como investigar:** profilers dão suporte a monitoramento legítimo. Um flag de habilitação, um CLSID e um caminho de DLL são partes diferentes da configuração; confirme a habilitação efetiva e o processo .NET relevante, em vez de tratar cada linha como um profiler executado.

### Sequestro do profiler do .NET Core CLR {#coreclr_profiler}

**ID da verificação:** `coreclr_profiler` · **ATT&CK do catálogo:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**O que o Juntei verifica:** lê as configurações de ambiente de profiling CORECLR correspondentes de máquina/usuário e as entradas Environment de serviços correspondentes, incluindo os caminhos específicos de cada arquitetura.

**Evidências examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` e o `Environment` de cada NTUSER.DAT de usuário capturado.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: configurações CORECLR nomeadas na multi-string.

**Como investigar:** valide o runtime .NET, a arquitetura, o flag de habilitação e a biblioteca. Produtos de observabilidade aprovados podem configurar essas definições, e um processo já existente pode não herdar um ambiente recém-alterado.

### Sequestro do DbgManagedDebugger do .NET {#dotnet_dbg_managed_debugger}

**ID da verificação:** `dotnet_dbg_managed_debugger` · **ATT&CK do catálogo:** [T1546](https://attack.mitre.org/techniques/T1546/)

**O que o Juntei verifica:** lê as configurações `DbgManagedDebugger` de `.NETFramework` de máquina/usuário listadas, incluindo o caminho explícito de 32 bits da máquina.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**Como investigar:** determine se o runtime .NET/a política de depuração aplicáveis usam esse comando. Ferramentas de desenvolvimento podem explicá-lo; correlacione com falhas gerenciadas e com a criação de processos para estabelecer a execução.

### Chave do Registro COR_PROFILER do .NET Framework {#dotnet_framework_profiler}

**ID da verificação:** `dotnet_framework_profiler` · **ATT&CK do catálogo:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**O que o Juntei verifica:** lê `COR_PROFILER`, `COR_PROFILER_PATH` e `COR_ENABLE_PROFILING` das visões do Registro de `.NETFramework` listadas. Isso complementa as verificações de profilers baseadas em variáveis de ambiente.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**Como investigar:** avalie os valores em conjunto e determine se o runtime respeita esse local do Registro. Uma ferramenta de profiling configurada pode ser legítima; não interprete um flag de habilitação isolado como o carregamento de uma DLL identificada.

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**ID da verificação:** `dotnet_startup_hooks` · **ATT&CK do catálogo:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**O que o Juntei verifica:** lê os valores de ambiente `DOTNET_STARTUP_HOOKS` de máquina/usuário, que podem selecionar assemblies de hooks de inicialização para runtimes .NET compatíveis.

**Evidências examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` e `Environment\DOTNET_STARTUP_HOOKS` de cada NTUSER.DAT de usuário.

**Como investigar:** inspecione o valor inteiro em busca de vários assemblies e verifique a compatibilidade com o runtime. Esta verificação não enumera ambientes de hooks de inicialização específicos de serviços nem prova que algum processo herdou e usou o valor.

### DLLs de depuração gerenciada conhecidas {#known_managed_debugging_dlls}

**ID da verificação:** `known_managed_debugging_dlls` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê os registros `KnownManagedDebuggingDlls` de `.NETFramework` como possíveis substituições de bibliotecas do depurador gerenciado.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**Como investigar:** compare com as ferramentas de desenvolvimento/runtime instaladas e determine o depurador que aciona o mecanismo. A entrada do Registro não prova uma sessão de depuração nem o carregamento de uma biblioteca.

## Substituições na localização de DLLs e executáveis { #dll-and-executable-lookup-overrides }

Estas verificações examinam locais explícitos de bibliotecas ou configurações que influenciam a forma como um componente encontra código. Uma substituição de caminho ou uma política de pesquisa enfraquecida é uma pista investigativa; ainda são necessários uma carga útil e um componente acionador compatível para que haja execução.

### DLLs AppCert {#appcert_dlls}

**ID da verificação:** `appcert_dlls` · **ATT&CK do catálogo:** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**O que o Juntei verifica:** enumera os registros AppCertDlls não vazios do Session Manager, uma superfície de extensão de bibliotecas associada às APIs de criação de processos.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | Todos os valores não vazios |

**Como investigar:** inspecione cada biblioteca e a procedência de sua instalação. Determine a aplicabilidade e o carregamento em tempo de execução separadamente; a criação normal de processos em uma linha do tempo não é, por si só, prova de que a DLL configurada foi carregada.

### DLLs AppInit {#appinit_dlls}

**ID da verificação:** `appinit_dlls` · **ATT&CK do catálogo:** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**O que o Juntei verifica:** lê `AppInit_DLLs` nativo/de 32 bits, divide sua lista de bibliotecas e acrescenta a cada candidato o contexto disponível de `LoadAppInit_DLLs` e `RequireSignedAppInit_DLLs`.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs`, `LoadAppInit_DLLs` e `RequireSignedAppInit_DLLs`; também a visão `Wow6432Node` da máquina.

**Como investigar:** um flag LoadAppInit_DLLs igual a zero é explicitamente marcado como inativo. A versão do Windows, o Secure Boot e os requisitos de assinatura também afetam o carregamento. Revise as configurações efetivas antes de afirmar que uma DLL AppInit foi executada.

**Mais detalhes:** [documentação da Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls).

### Substituição de AutodialDLL {#autodial_dll}

**ID da verificação:** `autodial_dll` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê a substituição `AutodialDLL` dos Parameters do WinSock2.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**Como investigar:** verifique o recurso de rede e a versão do Windows de origem que consomem essa configuração. Revise a DLL real e a atividade de rede/processos; uma substituição por si só não estabelece um carregamento ativo ou malicioso.

### DLL auxiliar de CHM {#chm_helper_dll}

**ID da verificação:** `chm_helper_dll` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê `Location` sob HtmlHelp Author, uma superfície de substituição de biblioteca/local relacionada à ajuda.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**Como investigar:** determine se o componente de ajuda relevante usa o valor neste sistema e inspecione o arquivo selecionado. Uma substituição no Registro não é prova de que um documento CHM foi aberto.

### Substituição de DLL do ContentIndex NL6 {#content_index_dll}

**ID da verificação:** `content_index_dll` · **ATT&CK do catálogo:** [T1574](https://attack.mitre.org/techniques/T1574/)

**O que o Juntei verifica:** lê `DLLOverridePath` dos registros imediatos de ContentIndex Language do conjunto de controle ativo.

**Evidências examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`.

**Como investigar:** os componentes de idioma/pesquisa podem diferir conforme a build do Windows. Inspecione a biblioteca e o componente instalado correspondente; siga o processo de indexação real para corroborar o carregamento.

### DLL ExpoOffload de criptografia {#crypto_expo_offload}

**ID da verificação:** `crypto_expo_offload` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê o valor `ExpoOffload` de Cryptography Offload como candidato a substituição de biblioteca criptográfica.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**Como investigar:** confirme a aplicabilidade ao recurso/build e investigue a origem da biblioteca selecionada. Não trate a presença de um valor incomum como prova de roubo de credenciais ou de código executado.

### DLL do rasterizador de software do Direct3D {#direct3d_dll}

**ID da verificação:** `direct3d_dll` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê a substituição de biblioteca `SoftwareRasterizer` de Direct3D Drivers.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**Como investigar:** softwares gráficos e de compatibilidade podem influenciar a configuração do rasterizador. Inspecione a DLL e a atividade de um aplicativo Direct3D compatível; salvar o caminho não prova que ela foi carregada.

### Enfraquecimento da ordem de pesquisa de DLLs {#dll_search_mode}

**ID da verificação:** `dll_search_mode` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** relata `SafeDllSearchMode` ou `CWDIllegalInDllSearch` somente quando o valor ativo do Session Manager é interpretado como zero. Essas são condições da política de pesquisa, não cargas úteis selecionadas.

**Evidências examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` e `CWDIllegalInDllSearch`.

**Como investigar:** uma política de localização enfraquecida pode facilitar um ataque separado de ordem de pesquisa. Ela não identifica uma DLL plantada nem prova execução. Valores diferentes de zero ou ausentes não são emitidos por esta verificação; avalie os padrões efetivos e as políticas separadamente.

**Mais detalhes:** [documentação da Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security).

### Exclusão de KnownDLLs {#exclude_from_known_dlls}

**ID da verificação:** `exclude_from_known_dlls` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê as entradas `ExcludeFromKnownDlls` não vazias do Session Manager como alterações no comportamento de localização de DLLs protegidas/conhecidas.

**Evidências examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`.

**Como investigar:** uma exclusão identifica uma condição de localização, não a própria biblioteca substituta. Encontre o aplicativo afetado e os possíveis locais de pesquisa de DLLs e, em seguida, corrobore um carregamento de imagem fora do padrão.

**Mais detalhes:** [documentação da Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order).

### Substituição de DLL do hhctrl.ocx {#hhctrl_ocx_dll}

**ID da verificação:** `hhctrl_ocx_dll` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê as substituições InprocServer32 da máquina nativa/de 32 bits e dos repositórios de classes de usuário compatíveis para o CLSID do controle HTML Help implementado.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**Como investigar:** determine qual aplicativo e arquitetura ativam o controle. Inspecione a DLL registrada e a atividade relacionada à ajuda; um registro COM não prova que o HTML Help renderizou conteúdo.

### Known DLLs {#known_dlls}

**ID da verificação:** `known_dlls` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** enumera os valores nomeados na chave KnownDLLs ativa do Session Manager.

**Evidências examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: valores nomeados.

**Como investigar:** muitas entradas são bibliotecas esperadas do Windows, e alguns valores são configurações de apoio, e não um caminho de carga útil. Compare a chave completa com uma build correspondente confiável; esta verificação não varre todas as árvores KnownDLLs alternativas específicas de cada arquitetura.

### DLL de extensões do LSA {#lsa_extensions}

**ID da verificação:** `lsa_extensions` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê `Extensions` da configuração LsaSrv de LsaExtensionConfig.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**Como investigar:** softwares de segurança podem estender componentes de autenticação. Inspecione as bibliotecas reais e a compatibilidade com a versão do Windows de origem e, em seguida, busque evidências de carregamento de imagem no LSASS ou outras evidências independentes de uso.

### Substituição do caminho da DLL MAPI32 {#mapi32_dll_path}

**ID da verificação:** `mapi32_dll_path` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê `DLLPathEx` e `DLLPath` sob os registros imediatos de clientes de Mail nas visões de máquina nativa e de 32 bits.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**Como investigar:** clientes de e-mail registram provedores MAPI legítimos. Determine qual cliente/provedor foi selecionado e inspecione a biblioteca real; um provedor instalado não prova que um aplicativo usou MAPI durante o incidente.

### DLL XA do MSDTC {#msdtc_xa_dll}

**ID da verificação:** `msdtc_xa_dll` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê `OracleXaLib` e `OracleOciLib` da configuração MTxOCI do MSDTC.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**Como investigar:** integrações com bancos de dados podem exigir legitimamente essas bibliotecas. Verifique a configuração do coordenador de transações e o software de banco de dados instalado, e correlacione com o uso de serviços/módulos, em vez de presumir que um caminho configurado foi executado.

### Substituição de DLL do NLDP {#nldp_dll}

**ID da verificação:** `nldp_dll` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê o valor de configuração `DllOverridePath` de NlsData.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**Como investigar:** revise a biblioteca de destino, a build do Windows e o componente de linguagem natural que poderia consumir a substituição. O valor não mostra que houve indexação ou processamento de idioma.

### DLL TestDVCPlugin do RDP {#rdp_test_dvc_plugin}

**ID da verificação:** `rdp_test_dvc_plugin` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê a configuração de biblioteca `TestDVCPlugin` do Terminal Server Client da máquina.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**Como investigar:** investigue um plugin de teste desconhecido e o comportamento específico da versão. São necessários uma sessão de cliente compatível e o carregamento da biblioteca para estabelecer a execução; o valor do Registro por si só não mostra nenhum dos dois.

### Substituição de DLL do Search Indexer {#search_indexer_dll}

**ID da verificação:** `search_indexer_dll` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê a substituição `DllPath` do Windows Search no SOFTWARE.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**Como investigar:** determine o componente/versão do Search correspondente e inspecione o arquivo de destino. Um serviço Windows Search inativo ou uma configuração não consumida é diferente de um carregamento observado de biblioteca maliciosa.

### DLL AutoProxy do Winsock {#winsock_auto_proxy}

**ID da verificação:** `winsock_auto_proxy` · **ATT&CK do catálogo:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**O que o Juntei verifica:** lê `LibraryPath` sob os registros imediatos Catalog_Entries e Catalog_Entries64 de NameSpace_Catalog5.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**Como investigar:** provedores de rede podem ser legítimos. Revise ambas as arquiteturas, a procedência do provedor e as evidências relevantes de aplicativos de rede/carregamento de imagens. Esta consulta direcionada a provedores de namespace não abrange todas as extensões possíveis do Winsock.

## Contas e visibilidade na entrada da sessão { #accounts-and-sign-in-visibility }

Os registros de contas locais e as configurações de visibilidade na entrada da sessão podem revelar persistência baseada em identidade. Estas verificações inspecionam os hives SAM e SOFTWARE capturados; não são um inventário de contas do Active Directory, de associações a grupos de domínio ou de logons anteriores.

### Associação ao grupo Administradores {#admin_group_membership}

**ID da verificação:** `admin_group_membership` · **ATT&CK do catálogo:** [T1098](https://attack.mitre.org/techniques/T1098/)

**O que o Juntei verifica:** decodifica a lista de membros do alias local Administradores no SAM. Ele relata SIDs da máquina local que correspondem a uma conta oculta ou a nenhuma conta/grupo no SAM capturado.

**Evidências examinadas:**

- `SAM\Domains\Builtin\Aliases\00000220\C` e os registros de SID da máquina/usuários/grupos sob `SAM\Domains\Account` no hive SAM.
- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList` para correlação de nomes ocultos.

**Como investigar:** esta é uma verificação direcionada de anomalias, não uma lista de todos os administradores. SIDs externos/de domínio não são avaliados como usuários locais ausentes. Contas excluídas, o momento da captura ou hives incompletos exigem investigação antes de se concluir por uma associação maliciosa.

### Conta local oculta {#hidden_account}

**ID da verificação:** `hidden_account` · **ATT&CK do catálogo:** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**O que o Juntei verifica:** relata valores numéricos iguais a zero em UserList de SpecialAccounts do Winlogon, identificando nomes ocultos da tela comum de entrada na sessão.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: valores com nomes de contas.

**Como investigar:** contas de serviço ou de manutenção podem ser ocultadas intencionalmente, e o valor não estabelece que a conta nomeada existe ou é administradora. Combine com os registros de contas do SAM e a atividade de logon.

### Sequestro de RID {#rid_hijacking}

**ID da verificação:** `rid_hijacking` · **ATT&CK do catálogo:** [T1098](https://attack.mitre.org/techniques/T1098/)

**O que o Juntei verifica:** compara o RID hexadecimal da subchave de uma conta no SAM com o RID armazenado no deslocamento 0x30 de seu valor F e relata as divergências.

**Evidências examinadas:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` no hive SAM.

**Como investigar:** uma divergência é uma anomalia grave no registro da conta, mas também pode exigir análise de corrupção/captura. Inspecione os bytes de origem, o status dos logs de transação e os registros de contas/logons; ela não prova qual agente modificou o registro.

### RID Suborner (conta de administrador oculta) {#rid_suborner}

**ID da verificação:** `rid_suborner` · **ATT&CK do catálogo:** [T1098](https://attack.mitre.org/techniques/T1098/)

**O que o Juntei verifica:** relata contas do SAM cujo RID da subchave não é 500, mas cujo RID no valor F é 500, um padrão mais específico de substituição da identidade do administrador.

**Evidências examinadas:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` no hive SAM.

**Como investigar:** isto se sobrepõe à verificação geral de divergência de RID. Apesar do título no catálogo, ela não exige de forma independente uma configuração de ocultação na entrada da sessão; confirme separadamente a visibilidade da conta, os privilégios e o uso real da conta.

## Componentes de autenticação e proteção de credenciais { #authentication-components-and-credential-protection }

A Autoridade de Segurança Local (LSA) trata funções de segurança por meio do LSASS e de componentes relacionados. Pacotes de autenticação, filtros de senha e provedores de rede podem ser softwares de segurança legítimos ou uma rota para acesso persistente a credenciais. Duas verificações inventariam configurações de proteção, e não inicializações automáticas de executáveis. Interprete-as usando a função do host, a versão do Windows e a política efetiva.

### DLL de provedor AMSI {#amsi_providers}

**ID da verificação:** `amsi_providers` · **ATT&CK do catálogo:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**O que o Juntei verifica:** enumera os CLSIDs de provedores AMSI da máquina e resolve suas bibliotecas InprocServer32 nativas/de 32 bits quando disponíveis. Registros não resolvidos mantêm o ID de classe.

**Evidências examinadas:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` e os registros InprocServer32 resolvíveis em `Classes\CLSID` / `Classes\Wow6432Node\CLSID`.

**Como investigar:** antivírus e produtos de segurança registram provedores AMSI legitimamente. Revise a procedência do provedor/biblioteca e o carregamento real; um registro não prova que o AMSI foi contornado nem que o provedor era malicioso.

### Pacotes de autenticação {#authentication_packages}

**ID da verificação:** `authentication_packages` · **ATT&CK do catálogo:** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**O que o Juntei verifica:** lê cada entrada não vazia de `Authentication Packages` do LSA no conjunto de controle ativo.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**Como investigar:** compare com a build do Windows de origem e o software de autenticação instalado. Um pacote fora do padrão pode ser legítimo. Resolva a DLL e corrobore o carregamento/a atividade de credenciais antes de concluir que houve interceptação.

### Comportamento de logon do administrador DSRM {#dsrm_backdoor}

**ID da verificação:** `dsrm_backdoor` · **ATT&CK do catálogo:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**O que o Juntei verifica:** relata `DsrmAdminLogonBehavior=2` numérico nos locais implementados ControlSet001, ControlSet002 ou CurrentControlSet. Essa configuração pode permitir o uso da conta DSRM fora de seu contexto habitual de recuperação.

**Evidências examinadas:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior`, além de ControlSet002 e CurrentControlSet, quando presentes.

**Como investigar:** relevante principalmente para controladores de domínio. Determine qual conjunto de controle está ativo usando o Select do SYSTEM; uma correspondência em um conjunto de controle histórico não está necessariamente em vigor. O valor não prova que um atacante conhecia a senha DSRM ou fez logon.

### Configuração do Credential Guard {#lsa_cfg_flags}

**ID da verificação:** `lsa_cfg_flags` · **ATT&CK do catálogo:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**O que o Juntei verifica:** lê o valor `LsaCfgFlags` configurado do LSA, incluindo tanto valores comuns de habilitação quanto zero. Isso inventaria a configuração do Credential Guard, e não uma inicialização automática de executável.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**Como investigar:** interprete o valor considerando a versão do Windows, os pré-requisitos de firmware/segurança e a política efetiva. Um zero armazenado ou um valor ausente, por si só, não estabelece uma alteração feita por um atacante, o estado efetivo de proteção nem roubo de credenciais.

### Filtro de senha do LSA {#lsa_password_filter}

**ID da verificação:** `lsa_password_filter` · **ATT&CK do catálogo:** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**O que o Juntei verifica:** lê `Notification Packages` do LSA. Nomes de bibliotecas sem caminho são resolvidos em relação ao diretório System32 do Windows capturado, acrescentando `.dll` quando apropriado.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**Como investigar:** produtos de política de senhas frequentemente instalam filtros. Inspecione a biblioteca e as evidências de carregamento, e diferencie filtros configurados de uma prova de que ocorreu uma alteração de senha ou interceptação de credenciais.

### Status da proteção PPL do LSASS {#lsa_run_as_ppl}

**ID da verificação:** `lsa_run_as_ppl` · **ATT&CK do catálogo:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**O que o Juntei verifica:** lê o valor `RunAsPPL` configurado do LSA, incluindo valores diferentes de zero. É um inventário de configurações de proteção, não um teste de que o LSASS foi executado sem proteção.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**Como investigar:** a proteção efetiva do LSA depende da versão do Windows, da política, do bloqueio por firmware e do estado de reinicialização. A ausência de uma linha para um valor ausente não é um resultado de proteção medido. Use evidências independentes do sistema/de tempo de execução para avaliar a proteção efetiva.

### DLL de provedor de rede {#network_provider_dll}

**ID da verificação:** `network_provider_dll` · **ATT&CK do catálogo:** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**O que o Juntei verifica:** lê `NetworkProvider\ProviderPath` sob os serviços no conjunto de controle ativo do SYSTEM.

**Evidências examinadas:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`.

**Como investigar:** provedores de logon de rede podem ser legítimos. Verifique a ordem/aplicabilidade dos provedores, o software de rede/segurança instalado e o carregamento real da biblioteca; um valor ProviderPath por si só não prova interceptação de credenciais.

### Pacotes de segurança {#security_packages}

**ID da verificação:** `security_packages` · **ATT&CK do catálogo:** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**O que o Juntei verifica:** lê cada entrada não vazia de `Security Packages` do LSA, cobrindo os provedores de suporte de segurança configurados.

**Evidências examinadas:**

| Chave do Registro | Valor examinado |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**Como investigar:** compare com a linha de base esperada do Windows/dos produtos de segurança e inspecione cada biblioteca. A configuração não prova que um provedor foi carregado, capturou credenciais ou foi adicionado durante o incidente.

## Do candidato à conclusão { #work-from-a-candidate-to-a-conclusion }

1. Abra a fonte original identificada por `SourceFile`. Confirme o valor completo, a ação XML,
   o script ou o registro de banco de dados recuperado e o usuário/host exato a que pertence.
2. Determine se o mecanismo poderia funcionar: verifique os flags de habilitação, a versão do runtime ou do Windows,
   o estado do serviço/tarefa, o gatilho necessário, o usuário, a arquitetura e o conjunto de controle ativo.
3. Inspecione o programa, a DLL, o script ou o assembly selecionado. Calcule o hash dos bytes coletados e revise os argumentos
   e o conteúdo. Um iniciador conhecido do Windows ou uma string de editor reconhecível não resolve a questão da legitimidade.
4. Estabeleça o que de fato aconteceu usando evidências independentes. Exemplos incluem criação de processos,
   carregamentos de imagens do Sysmon, registros de execução do Agendador de Tarefas, atividade de serviços, Prefetch e arquivos relacionados.
   Um evento de alteração de configuração pode provar que a configuração mudou sem provar que sua carga útil foi executada.
5. Monte uma linha do tempo usando os carimbos de data/hora das fontes originais e a procedência de correlação registrada.
   Mantenha gravações de chaves, alterações de instalação, gravações de arquivos/contêineres e execuções como fatos separados.
6. Registre os limites, além das constatações: hives ausentes, arquivos de destino ausentes, principais de tarefas desconhecidos,
   metadados de assinante não verificados, registros históricos recuperados por carving ou um gatilho que não pode ser corroborado.

Para a sintaxe do comando, os perfis, todos os campos de saída e os estados de cobertura, volte ao
[guia do comando Persistence Hunter](../commands/persistence-hunter.md). Os
[guias de artefatos](../artifacts/index.md) explicam as fontes subjacentes, e os
[agradecimentos](acknowledgements.md) identificam as ferramentas de persistência de origem.
