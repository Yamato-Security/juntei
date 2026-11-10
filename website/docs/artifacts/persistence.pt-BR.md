# Inicializações automáticas do Registro { #registry-autostarts }

## O que são { #what-they-are }

O Windows inicia programas e carrega DLLs a partir de muitos locais do Registro: na inicialização, no logon, quando um
programa inicia ou termina, quando o `cmd.exe` é aberto, quando uma classe COM é criada ou quando o spooler,
a LSA, o netsh ou o serviço de horário carrega seus plug-ins. Atacantes usam esses locais para persistência,
escalonamento de privilégios e injeção de DLL, e softwares legítimos também os usam. O Juntei os lê
de `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` e `UsrClass.dat` em uma única tabela. Ele grava um registro por
programa, DLL ou pacote, e marca cada um com sua técnica do MITRE ATT&CK.

<span class="evidence-strength">Melhor conclusão:</span> o hive configurou este programa ou DLL
para iniciar ou ser carregado. Isso não prova que o programa foi executado. Executá-lo requer as condições que o
mecanismo especifica (um logon, o início do programa depurado, a criação da classe) e outras
evidências de execução.

## Locais { #locations }

| Hive | O que é lido |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce` (+`Setup`), `RunOnceEx`, `RunServices(Once)` e `Policies\Explorer\Run`, com suas cópias em `Wow6432Node` e no modo de instalação do Terminal Server; `Explorer\StartupApproved\Run`, `Run32` e `StartupFolder`; `Winlogon` (`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … e `Notify`); `Windows` `AppInit_DLLs` com `LoadAppInit_DLLs`; `Image File Execution Options\*` `Debugger` (com `UseFilter`) e `VerifierDlls`; `SilentProcessExit\*` `MonitorProcess`; `Active Setup\Installed Components\*` `StubPath`; `NetSh`; `Command Processor` `AutoRun` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, variáveis de profiler .NET em `Environment`); `Control\Lsa` `Authentication`, `Security` e `Notification Packages` (+`OSConfig`); `SecurityProviders`; monitores, provedores e processadores de impressão de `Control\Print`; `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | as chaves `Run` e `StartupApproved` do usuário; `Load` e `Run` de `Windows NT\CurrentVersion\Windows`; `Shell` de `Winlogon`; `Environment` (`UserInitMprLogonScript`, profiler .NET); `SCRNSAVE.EXE` de `Control Panel\Desktop`; `PackagedAppXDebug`; `AutoRun` de `Command Processor`; `Software\Classes` (como `UsrClass.dat`) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` e `LocalServer32` (e os de `Wow6432Node`); `shell\open\command` de tipos de programa (`exefile`, `batfile`, `scrfile`, …) e do que os programas com elevação automática iniciam (`ms-settings`, `mscfile`, `Folder`, …); `ActivatableClasses\…\DebugInformation` |

Colete cada hive com seus logs de transação. Os hives de usuário ficam em `Users\<name>\NTUSER.DAT` e
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat`.

## Cuidados para o analista { #analyst-cautions }

- **Os padrões também são relatados.** A maioria dos registros em um sistema limpo são padrões do Windows:
  - `Userinit` é `userinit.exe`;
  - `Shell` é `explorer.exe`;
  - `KnownDLLs`, os pacotes LSA, os monitores de impressão, os auxiliares do netsh e os componentes do Active Setup têm todos entradas de fábrica.

  Procure o que difere de uma instalação limpa da mesma versão.
- **`StartupApproved` abrange apenas a lista de Inicialização do Gerenciador de Tarefas.** Ele indica se uma entrada `Run`
  ou um arquivo da pasta Inicializar está desativado ali, e quando. Uma entrada sem um valor `Run`
  correspondente é relatada, com os detalhes "only in StartupApproved". Ela pode mostrar uma inicialização automática que
  foi removida.
- **`OverridesMachineRegistration` requer o hive `SOFTWARE` do sistema.** A classe COM ou o
  comando de um usuário é comparado com ele, e ele é localizado da mesma forma que o hive `SYSTEM` do host. Uma
  substituição é o padrão de sequestro de COM (T1546.015). O OneDrive e outras instalações por usuário também
  registram classes próprias que não substituem nada.
- **`ImagePath` e `Args` são separados da linha de comando somente quando isso pode ser determinado.**
  - Um programa entre aspas é separado na aspa de fechamento.
  - Uma primeira palavra sem pasta é considerada o programa (`cmd /c …`).
  - Caso contrário, a separação ocorre no início mais curto que termina com a extensão de um programa.

  Um caminho sem aspas com espaços pode executar um programa diferente (`C:\Program.exe`) se ele existir.
- **`GlobalFlag` determina `Enabled` para `VerifierDlls` e `MonitorProcess`.** `VerifierDlls` é carregado
  somente com `0x100`. `MonitorProcess` é executado somente com `0x200` e o `0x1` de `ReportingMode` em
  `SilentProcessExit\<program>`. Um modo `2` (um dump) ou `4` (uma notificação) não executa nada, e
  `Details` nomeia as flags do modo.
- **`KeyLastWrittenUTC` é a última gravação da chave**, não do valor. Vários valores o compartilham.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

O resumo conta os registros por mecanismo. Ele também lista as entradas de `StartupApproved\Run` e `Run32`
que ficaram sem seu valor `Run` (os arquivos da pasta Inicializar não têm nenhum a perder), e as classes COM
e comandos do usuário que substituem os da máquina.

Como todo comando `parse-registry`, ele recusa um `-o` que seja um dos arquivos que ele lê, mesmo
com `-C`. Isso abrange o hive, seus logs de transação e os hives SYSTEM e SOFTWARE do sistema,
por caminho, link simbólico ou hard link. Quando não consegue determinar, ele não grava.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Contexto do host, hive de origem e seu tipo. |
| `User`, `UserSID` | Para hives de usuário: a pasta do perfil e o SID. O SID vem do nome da chave raiz de `UsrClass.dat`; caso contrário, do `ProfileList` do sistema. |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | Técnica do MITRE ATT&CK, quando alguma se aplica (`T1547.001`, `T1546.012`, …). |
| `KeyPath`, `ValueName` | A chave sob a raiz do hive (nomes como gravados) e o valor (vazio: o valor padrão). |
| `Data` | O texto do valor, ou um item de uma lista. Vazio quando não pode ser lido, ou quando apenas `StartupApproved` nomeia a entrada. |
| `ImagePath`, `Args` | O programa ou a DLL, e os argumentos, quando podem ser distinguidos (veja acima). |
| `Enabled`, `DisabledAtUTC` | O que `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag` (com `ReportingMode`), `UseFilter`, `IsInstalled`, o `Enabled` do provedor de horário ou `ScreenSaveActive` indicam. Vazio quando nada indica. |
| `Details` | Mais informações: o que uma renomeação pendente faz, o nome da classe COM, `ReportingMode`, `FilterFullPath`, um `ShellComponent` do Active Setup, … |
| `OverridesMachineRegistration` | Para a classe COM ou o comando de um usuário: se a máquina registra o mesmo. Vazio sem o hive `SOFTWARE` do sistema. |
| `WOW64` | Em `Wow6432Node`: para programas de 32 bits. |
| `KeyLastWrittenUTC` | A última gravação da chave. |
| `DataRaw` | Os bytes do valor (hex), quando não são texto ou não são UTF-16 válido. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Tipos de valor incorretos, dados que não podem ser lidos, entradas de `StartupApproved` com tamanho incorreto. |

## Estratégia de investigação { #investigation-strategy }

1. Comece pelo resumo:
   - classes COM e comandos que substituem os da máquina;
   - entradas de `StartupApproved` sem seu valor `Run`;
   - os mecanismos que raramente têm entradas (`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`).
2. Compare os pacotes LSA, `BootExecute`, `KnownDLLs`, os monitores de impressão e os auxiliares do netsh com uma instalação
   limpa da mesma versão do Windows.
3. Resolva `ImagePath` em relação ao sistema de arquivos ($MFT, hashes e assinaturas) e verifique caminhos de perfil
   de usuário, `ProgramData` e `Temp`.
4. Correlacione `KeyLastWrittenUTC` com eventos de Registro do Security e do Sysmon, criação de processos,
   Prefetch e Amcache.
5. Use `parse-registry read --recover-deleted` para encontrar inicializações automáticas removidas dessas chaves.

## Mais detalhes { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: Chaves do Registro Run e RunOnce](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
