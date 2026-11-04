# Informações do sistema { #system-information }

## O que é { #what-it-is }

Os quatro hives de máquina de um sistema informam o que um analista precisa saber antes de ler sua linha do tempo:

- seus nomes (NetBIOS, nome de host DNS, domínio DNS) e o domínio ou grupo de trabalho a que pertence;
- sua versão, build e revisão de atualização do Windows, e quando o Windows foi instalado ou atualizado pela última vez;
- seu fuso horário, que indica os horários locais mantidos pelo sistema;
- quando foi desligado corretamente pela última vez;
- sua arquitetura de processador e suas páginas de código;
- o último usuário que entrou no sistema e os perfis de todos os usuários que já entraram;
- seus SIDs: o do computador (as contas locais são esse SID mais um RID) e o do seu domínio;
- as versões do Windows a partir das quais foi atualizado.

`parse-registry system-info` grava um registro por sistema com todas essas informações.

<span class="evidence-strength">Melhor conclusão:</span> a configuração como estava quando os hives
foram coletados (ou gravados pela última vez). Os horários das chaves indicam quando partes dela mudaram pela última vez.

## Localização { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

Colete os quatro hives com seus logs de transação. Qualquer um deles, sozinho, fornece o que contém.

## Cuidados para o analista { #analyst-cautions }

- **Os horários do Registro estão em UTC; o fuso horário serve para todo o resto.** Os horários das chaves, os FILETIMEs e a maioria
  dos horários de eventos estão em UTC. Logs de texto, alguns logs de aplicativos e datas gravadas em nomes ou cadeias de caracteres
  (como `Source OS (Updated on …)`) podem estar em horário local. `Bias` é o número de minutos do horário padrão local
  até o UTC (`UTC = local + Bias`). `UtcOffset` e `DaylightUtcOffset` fornecem os dois deslocamentos,
  e `TimeZoneDescription` informa quando o horário de verão começa e termina.
- **O fuso horário pode mudar.** `TimeZoneLastWrittenUTC` é a última gravação da chave. O Windows também
  regrava a chave por conta própria quando atualiza `ActiveTimeBias` (o deslocamento em vigor naquele momento), então um
  horário recente não prova que alguém mudou o fuso. Compare com os logs de eventos: Security 4616
  (a hora do sistema foi alterada) e os eventos `Microsoft-Windows-Kernel-General` do log System.
- **`StandardStart` e `DaylightStart` são `TIME_FIELDS` do kernel, não `SYSTEMTIME`s.** O
  dia da semana é o último campo, não o terceiro (como no valor `TZI` do fuso em `SOFTWARE`). Nos
  fusos dos EUA, os dois layouts são lidos da mesma forma, porque o dia da semana é domingo (0). O Juntei os decodifica
  como `TIME_FIELDS`: um sistema público da Ilha de Páscoa (primeiro sábado às 22:00) mostra a
  diferença. Um mês igual a 0 significa ausência de horário de verão.
- **O Windows 11 diz "Windows 10".** `ProductName` mantém `Windows 10 …` no Windows 11. As builds 22000
  e posteriores são Windows 11, e o resumo indica isso. `CurrentVersion` é `6.3` no Windows 8.1, 10
  e 11; `Version` usa `CurrentMajorVersionNumber` (10) quando ele existe.
- **`InstallDate` muda com as atualizações de recursos.** Uma atualização in-place para uma nova versão (de 1809 para
  21H2, por exemplo) o redefine. A versão que cada atualização substituiu, com seu próprio `InstallDate`, está
  em `PreviousVersions` (`Setup\Source OS`). O `InstallDate` mais antigo entre eles é o mais próximo da
  instalação original. A data no nome de uma chave `Source OS` não indica seu fuso horário.
- **`ShutdownTime` é o último desligamento correto.** Uma falha ou uma queda de energia não o atualiza.
  Compare-o com os logs de eventos (6006, 6008, 1074) e com os horários das últimas gravações nos hives.
- **`LastLoggedOnUser` é o último usuário da tela de entrada,** não todos os logons. Área de Trabalho Remota,
  `runas`, serviços e tarefas agendadas nem sempre o definem. `LogonUiLastWrittenUTC` indica aproximadamente quando
  ele foi definido pela última vez.
- **Perfis:** `ProfileList` lista todos os usuários (e serviços) que tiveram um perfil no
  sistema, incluindo contas excluídas até que seu perfil seja removido. O Windows 10 e posteriores gravam
  `LocalProfileLoadTime` e `LocalProfileUnloadTime`: aproximadamente a última entrada e a última saída.
- **SIDs:** as contas locais são `MachineSid-<RID>`, e as contas de domínio, `DomainSid-<RID>`.
  `MachineSid` vem do `SAM` (ou, na falta dele, de `PolAcDmS` do `SECURITY`). Os dois devem coincidir;
  quando não coincidem, `Problems` indica isso, como aconteceria com hives de sistemas diferentes. `DomainSid` fica vazio
  em um grupo de trabalho.
- **Um diretório, um sistema.** Com `-d`, os hives de um diretório são de um único sistema.
  Cópias de `RegBack` e um segundo hive do mesmo tipo (`SYSTEM.old`) geram registros próprios.
  Com `-f`, os hives fornecidos são de um único sistema, e os demais são procurados ao lado deles.
- **Conjuntos de controles:** apenas o conjunto de controles que o Windows usou por último é lido. `parse-registry read` mostra
  os outros.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

O resumo apresenta, para cada sistema: sua versão do Windows, quando foi instalado, seu fuso horário por
extenso, o deslocamento em vigor quando a chave foi gravada pela última vez, seu último desligamento, seu domínio ou
grupo de trabalho, o último usuário que entrou, seu SID, quantos perfis ele tem e as versões a partir das quais
foi atualizado. O `auto-parse` também exibe o fuso horário de cada sistema ao listar os sistemas que
encontrou.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | Host e os hives lidos (`SourcePath` é o hive `SYSTEM` ou, na falta dele, o primeiro lido). |
| `CompName`, `Hostname`, `Domain` | O nome NetBIOS (`SYSTEM` ou, na falta dele, `PolAcDmN` do `SECURITY`), o nome de host DNS e o domínio DNS (`Tcpip\Parameters`). |
| `PrimaryDomain`, `DNS-Domain` | O domínio ao qual o computador está ingressado, ou seu grupo de trabalho (`PolPrDmN`), e o nome DNS do domínio (`PolDnDDN`). |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: o nome do produto como gravado, `EditionID`, `DisplayVersion` (ou, na falta dele, `ReleaseId`), a versão principal e secundária (ou, na falta delas, `CurrentVersion`), a build, a revisão de build de atualização, `CSDVersion` e `Client`, `Server` ou `Server Core`. |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (com precisão de segundos) e `InstallTime` (com precisão de 100 ns, Windows 10 e posteriores): a instalação ou a última atualização de recursos. |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | Como gravados. |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (ou, na falta dele, `StandardName`) e o fuso por extenso, com seus deslocamentos e as datas do horário de verão. |
| `UTC-Offset`, `DaylightUTC-Offset` | O deslocamento do horário padrão local em relação ao UTC e o do horário de verão (vazio sem horário de verão). |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`, `ActiveTimeBias` (em vigor quando a chave foi gravada), `StandardBias`, `DaylightBias`. |
| `StandardName`, `DaylightName` | Como gravados, muitas vezes uma referência a recurso (`@tzres.dll,-112`). |
| `StandardStart`, `DaylightStart` | Quando o horário padrão e o horário de verão começam, por extenso (`first Sunday of November, 02:00`). |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | Horário de verão dinâmico desativado; o relógio de hardware mantém o UTC. |
| `TimeZoneLastWrittenUTC` | A última gravação da chave do fuso horário. |
| `LastShutdownUTC` | `ShutdownTime`: o último desligamento correto. |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | O conjunto de controles lido, `PROCESSOR_ARCHITECTURE`, `ACP` e `OEMCP`. |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | O último usuário da tela de entrada e a última gravação de sua chave. |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | O SID do computador, o SID do seu domínio (`PolPrDmS`) e o SID da conta de computador no domínio (`PolMachineAccountS`). |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | Aproximadamente quando o computador foi nomeado pela última vez, ingressou pela última vez em um domínio ou grupo de trabalho e foi atualizado pela última vez (a chave de `CurrentVersion`). |
| `PreviousVersions` | Cada chave `Source OS`: a versão, sua data de instalação e a data no nome da chave (JSON: um objeto para cada). |
| `Profiles` | Cada perfil: SID, pasta, horário da chave, último carregamento e descarregamento (JSON: um objeto para cada). |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro, considerando os quatro hives. |
| `Problems` | Valores do tipo ou tamanho errado (com seus bytes mantidos), e um `SAM` e um `SECURITY` cujos SIDs de computador diferem. |

## Estratégia de investigação { #investigation-strategy }

1. Leia primeiro o fuso horário: anote `UtcOffset`, as datas do horário de verão e se o fuso foi
   alterado recentemente. Use-o para converter os logs em horário local.
2. Anote a versão, a build e a data de instalação. Verifique `PreviousVersions` em busca de atualizações e compare
   a data de instalação com os horários mais antigos do sistema de arquivos e dos logs de eventos.
3. Anote `LastShutdownUTC`: atividade posterior a ele sem um novo horário de desligamento sugere uma falha, uma queda
   de energia ou uma coleta ao vivo.
4. Use `MachineSid` e `DomainSid` para distinguir contas locais de contas de domínio nos SIDs encontrados em outros lugares (BAM,
   `ProfileList`, logs de eventos), e os horários de carregamento e descarregamento dos perfis para as últimas entradas.
5. Correlacione os nomes do computador e do domínio com os logs de eventos, e com `CompNameLastWrittenUTC`
   e `PrimaryDomainLastWrittenUTC` para renomeações e ingressos em domínio.

## Mais detalhes { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: identificadores de segurança](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: fusos horários](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: informações do produto Windows](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
