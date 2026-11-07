# Registro de acesso do usuário (UAL) { #user-access-logging-ual }

## O que é o UAL { #what-ual-is }

O Registro de acesso do usuário (User Access Logging, UAL) é um recurso do Windows Server, instalado e
habilitado por padrão desde o Windows Server 2012, que agrega em bancos de dados ESE as solicitações que os
clientes fazem às funções do servidor. Para cada função, endereço de cliente e conta, ele guarda quando o
cliente foi visto pela primeira e pela última vez no ano, o número total de solicitações e uma contagem para
cada dia do ano. As funções incluem File Server (SMB), Active Directory Domain Services, Active Directory
Certificate Services, DHCP Server, Print and Document Services, Remote Access, Web Server (IIS) e Windows
Server Update Services.

O UAL é valioso em investigações de movimentação lateral: suas linhas de clientes mostram quais endereços e
contas usaram um servidor, e em quais dias, muitas vezes bem depois que os logs de eventos já foram
sobrescritos.

<span class="evidence-strength">Melhor conclusão:</span> um cliente no endereço registrado, autenticado com
a conta registrada, usou a função em cada dia que tem uma contagem.

## Localização e coleta { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

O `Current.mdb` contém a atividade mais recente. O `SystemIdentity.mdb` descreve o servidor, dá nome às
funções e indica o arquivo `{GUID}.mdb` que guarda cada ano (`CHAINED_DATABASES`). O serviço do UAL mantém os
bancos de dados abertos: colete o diretório `Sum` inteiro com um coletor forense ou a partir de um snapshot
de volume.

!!! warning "Bancos de dados sujos"
    O `Current.mdb` costuma estar sujo (dirty) quando é coletado de um servidor em execução. O Juntei o lê como
    foi coletado e relata essa condição, mas não reproduz os `Svc*.log`: a atividade existente apenas nos logs
    de transação fica de fora.

## Cuidados para o analista { #analyst-cautions }

- O UAL conta as solicitações por dia; ele não registra cada solicitação individualmente. Os únicos horários
  de uma linha de cliente são `InsertDate` e `LastAccess`.
- Os dias de `DailyAccesses` são dias UTC.
- `Address` é o endereço que o servidor viu: um gateway NAT, um proxy ou uma VPN pode estar entre ele e o
  cliente real. `::1` e `127.0.0.1` são o próprio servidor.
- Analise todos os bancos de dados do diretório `Sum`: o mesmo cliente pode ter linhas no `Current.mdb` e no
  `{GUID}.mdb` do seu ano.
- A ausência de linhas não prova que nada se conectou: o UAL pode ser desativado (`Disable-Ual`, ou parando o
  serviço User Access Logging), e um banco de dados sujo não contém suas alterações mais recentes.
- Sem o `SystemIdentity.mdb`, `RoleName` vem dos nomes que o Windows Server dá às suas funções.
- Um `LastAccess` igual a zero ou inválido não determina o ano. O Juntei usa então um `InsertDate` válido e depois os metadados do banco anual; sem um ano válido, preserva `DayOfYear`.
- Os totais do resumo somam todas as linhas de origem. `Current.mdb`, bancos anuais e instantâneos podem se sobrepor; esses valores não são contagens de conexões ou logons sem duplicatas.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Current.mdb, {GUID}.mdb and SystemIdentity.mdb
  -f, --file <FILE>      Parse one UAL database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl \
  | jq 'select(.Table == "CLIENTS") | {Address, AuthenticatedUserName, RoleName, LastAccess}'
```

## Tabelas de saída { #output-tables }

O CSV cria um arquivo por tabela, apenas com o cabeçalho quando a tabela está vazia. As linhas JSON e JSONL
trazem `SourceFile` (o banco de dados) e `Table` (o nome da tabela no banco de dados).

| Arquivo CSV | Tabela | Conteúdo |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | Os clientes de cada função: endereço, conta, primeiro e último acesso do ano, contagem total de solicitações e contagens diárias. |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | Quando cada função foi usada pela primeira e pela última vez. |
| `UAL-DNS.csv` | `DNS` | Pares de endereço e nome de host, com a última vez em que cada um foi visto. |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Máquinas virtuais do Hyper-V: identificadores, número de série, horários de criação e da última atividade. |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | O servidor: nome, domínio, versão do Windows, tipo de produto, hardware, fuso horário e última inicialização; uma nova linha pode revelar uma renomeação, o ingresso em um domínio ou a promoção a controlador de domínio. |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | O arquivo `{GUID}.mdb` de cada ano. |
| `UAL-Roles.csv` | `ROLE_IDS` | Os GUIDs das funções com seus nomes. |
| `UAL-<table>.csv` | Outra tabela | Todas as colunas de uma tabela que o Juntei não conhece. |

## Campos { #fields }

| Campo | Interpretação |
|---|---|
| `SourceFile` | O banco de dados do qual a linha foi lida. |
| `RoleGUID`, `RoleName` | A função e seu nome, obtido do `SystemIdentity.mdb` ao lado do banco de dados (nomes internos na ausência dele). |
| `TenantID` | O GUID do locatário (tenant) do cliente, quando a função informa um. |
| `TotalAccesses` | Solicitações do cliente no ano do banco de dados; a soma de `DailyAccesses`. |
| `InsertDate` | Quando a linha foi criada: a primeira solicitação do cliente no ano neste banco de dados (UTC). |
| `LastAccess` | A solicitação mais recente do cliente (UTC). |
| `Address` | O endereço IPv4 ou IPv6 do cliente. |
| `AuthenticatedUserName` | A conta com a qual o cliente se autenticou (`DOMAIN\user`; contas de computador terminam com `$`). |
| `ClientName` | O nome do cliente, quando a função informa um. |
| `DailyAccesses` | Os dias com solicitações e suas contagens, a partir das colunas `Day1` … `Day366`, datados no ano de `LastAccess`: `2023-01-12: 5, 2023-01-13: 2` em CSV, objetos com `Date` e `Accesses` em JSON. Um dia cujo ano é desconhecido tem `DayOfYear` em vez de `Date`. |
| `FirstSeen`, `LastSeen` | Primeiro e último uso de uma função (`ROLE_ACCESS`), última observação de um par DNS (`DNS`), em UTC. |
| `CreationTime`, `LastSeenActive` | Quando uma máquina virtual ou uma linha `SYSTEM_IDENTITY` foi criada, e quando uma máquina virtual esteve ativa pela última vez, em UTC. |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server`. |
| `OS-CurrentTimeZone` | A diferença do servidor em relação ao UTC, em minutos (`-420` é UTC-07:00). |
| `OS-LastBootUpTime` | A última inicialização do servidor em UTC (o texto WMI como foi armazenado, quando não pode ser lido). |
| Outras colunas | Todas as demais colunas da tabela, com os rótulos de campo em PascalCase do Juntei, por exemplo `SystemDNS-HostName`, `SystemDomainName`, `OS-BuildNumber` e `SystemSerialNumber`. |

## Análise automática { #automatic-parsing }

O `auto-parse` grava as tabelas em `LateralMovement` como `UAL-Clients.csv`, `UAL-RoleAccess.csv` e assim
por diante (um único arquivo `UAL.json` ou `UAL.jsonl` com `-t json` ou `-t jsonl`), junto com
`LateralMovement/UAL-logs.txt`. A linha do tempo combinada de `-j` inclui todas as linhas, com `Comp` e o
identificador de `Artifact` `ual`. `-Z` converte `InsertDate`, `LastAccess`, `FirstSeen`, `LastSeen`,
`CreationTime`, `LastSeenActive` e `OS-LastBootUpTime`.

## Mais detalhes { #further-details }

- [Microsoft — User Access Logging](https://learn.microsoft.com/en-us/windows-server/administration/user-access-logging/get-started-with-user-access-logging)
- [Plaso — User Access Logging parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/user_access_logging.py)
- [Velociraptor — Windows.Forensics.UserAccessLogs](https://docs.velociraptor.app/artifact_references/pages/windows.forensics.useraccesslogs/)
- [CrowdStrike — How to Leverage User Access Logging for Forensic Investigations](https://www.crowdstrike.com/en-us/blog/user-access-logging-ual-overview/)
