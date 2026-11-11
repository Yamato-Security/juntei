# SRUM { #srum }

## O que é o SRUM { #what-srum-is }

O System Resource Usage Monitor (SRUM) armazena um banco de dados ESE do uso de recursos por aplicativos e
serviços. Dependendo da versão do Windows, ele registra bytes de rede, identificadores de interface/perfil,
conexões, tempo de CPU, atividade em primeiro plano/segundo plano, atividade de disco, consumo de energia, notificações
push e dados da linha do tempo de aplicativos. As entradas são associadas a IDs de aplicativo e SIDs de
usuário por meio de `SruDbIdMapTable`.

O SRUM é especialmente útil quando um executável foi excluído: seu ID de aplicativo, seu usuário e suas linhas de
uso de recursos podem sobreviver ao arquivo. Os totais de rede podem sustentar uma hipótese de exfiltração, mas
o SRUM não é uma captura de pacotes e, em geral, não identifica todos os endpoints remotos nem os arquivos transferidos.

<span class="evidence-strength">Melhor conclusão:</span> o Windows atribuiu o uso de recursos registrado
ao aplicativo/à conta durante o período de medição da tabela.

## Localização e coleta { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

O SRUM existe no Windows 8 e posteriores. Copie o `SRUDB.dat` de forma consistente, de preferência a partir de um snapshot ou com
um coletor forense. Colete também o `SOFTWARE` e os arquivos de transação/log do ESE no diretório
`sru`.

O hive SOFTWARE correspondente é separado do banco ESE: quando está sujo, o Juntei reaplica automaticamente em memória seus logs de transações do Registro adjacentes `SOFTWARE.LOG`, `.LOG1` e `.LOG2`. Colete-os também. Os detalhes da recuperação e avisos são exibidos; sem esses logs, nomes de contas ou Wi-Fi podem estar incompletos ou desatualizados. Isso não reaplica os logs do banco SRUM.

!!! warning "Bancos de dados sujos"
    O Juntei lê um `SRUDB.dat` sujo (dirty) como foi coletado e relata essa condição. Atualmente, ele não reproduz
    os `SRU*.log`, de modo que as alterações existentes apenas nesses logs ficam de fora. Não descreva silenciosamente um
    banco de dados sujo como completo.

## Cuidados para o analista { #analyst-cautions }

- Muitas linhas representam agregações periódicas, normalmente em torno de limites de hora cheia — não o horário exato da ação.
- Totais de bytes, por si sós, não identificam conteúdo, intenção, protocolo nem destino.
- Uma identidade de aplicativo pode ser um caminho, um serviço, um aplicativo empacotado ou um objeto; examine `AppType`.
- Os nomes de usuários e de redes Wi-Fi são enriquecimentos a partir do hive `SOFTWARE` correspondente. A ausência de nomes não significa
  que o ID subjacente esteja ausente.
- `null`, zero, colunas ausentes, tipos não suportados e valores danificados são estados diferentes.
- Os esquemas dos provedores variam conforme a versão do Windows. Preserve as tabelas desconhecidas e os nomes de colunas originais.
- Serviços do Windows em segundo plano geram, legitimamente, grandes totais de recursos.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## Tabelas de saída { #output-tables }

A saída CSV cria `SRUM-IdentityMap.csv` para `SruDbIdMapTable` e uma tabela por provedor. Os nomes de saída conhecidos incluem:

| Provedor/tabela | Finalidade investigativa |
|---|---|
| `NetworkUsages` | Bytes enviados/recebidos por aplicativo, usuário, interface e perfil. |
| `NetworkConnections` | Atividade do tipo contagem/duração de conexões e horários de início das conexões. |
| `AppResourceUseInfo` | Uso de CPU, disco e rede em primeiro plano/segundo plano e uso relacionado de aplicativos. |
| `AppTimelineProvider` | Atividade da linha do tempo de aplicativos, com campos de horário de término e de estado de uso. |
| `EnergyUsage` | Medições de energia/do provedor de curto e longo prazo; `IsLt` as diferencia. |
| `PushNotifications` | Atividade de notificações push. |
| `VfuProvider` | O provedor `vfuprov`; sua finalidade detalhada não está bem documentada, portanto interprete seus campos com cautela. |
| `EnergyEstimation`, `TaggedEnergy` | Provedores de energia e uso específicos de cada versão. |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Tabelas de provedores do Windows Server. |
| Nome de tabela desconhecido | O Juntei preserva provedores desconhecidos sob seu GUID/nome de tabela e emite todas as colunas do catálogo. |

Como o SRUM é orientado pelo catálogo e o Windows adiciona colunas aos provedores, intencionalmente não há uma lista
fixa e finita de campos específicos de cada provedor. O Juntei emite **todas as colunas presentes no banco de dados, com os
rótulos de campo em PascalCase do Juntei**. Os campos a seguir constituem o contrato de saída estável/comum.

## Saída categorizada do auto-parse { #categorized-auto-parse-output }

O `auto-parse` coloca cada tabela do SRUM diretamente em sua categoria investigativa. CSV, JSON e JSONL
usam o mesmo prefixo de nome de arquivo `SRUM-`; não há um diretório `SRUM/` envolvente.

| Categoria | Arquivos CSV |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`, `SRUM-EnergyEstimation.csv`, `SRUM-TaggedEnergy.csv`, `SRUM-SizeInBytes.csv`, `SRUM-TotalUsed.csv`, `SRUM-IdentityMap.csv`, `SRUM-VfuProvider.csv` e tabelas de provedores desconhecidos |

Somente as tabelas presentes na captura são exportadas. Os contadores de rede e os registros de notificações são
colocados de acordo com seu principal uso investigativo; isso não significa que toda linha descreva uma conexão
remota. O mapa de identidades serve a várias categorias. O VFU e os provedores desconhecidos permanecem em
`SystemInformation` porque sua presença, por si só, não estabelece execução.

`SystemInformation/SRUM-logs.txt` contém os diagnósticos do analisador e o mapeamento entre a saída original e a
final. Ambos os manifestos listam os caminhos reais das tabelas em `OutputFiles`; o `Output` da tarefa SRUM
é o diretório do computador, e `Category` identifica a localização compartilhada de seu log/metadados.

Com vários bancos de dados, os nomes de arquivo recebem `-Source1`, `-Source2` etc. antes da extensão.
O log preserva os aliases originais da captura, e as linhas JSON/JSONL mantêm sua procedência em `SourceFile`,
`Table`, `Provider`, `Comp` e `Artifact`. Nenhuma tabela é copiada para várias
categorias. A linha do tempo combinada para SIEM (`-j`) inclui cada registro SRUM mantido uma única vez.

O `parse-ese srum` independente mantém o local de saída solicitado. Seus arquivos de tabela CSV também têm
o prefixo `SRUM-`; os nomes de arquivo de saída JSON/JSONL explícitos continuam sendo escolhidos pelo analista.

## Campos de saída comuns { #common-output-fields }

| Campo | Interpretação |
|---|---|
| `SourceFile` | Banco de dados de entrada; presente em JSON/JSONL e útil ao trabalhar com vários hosts. |
| `Table` | Nome/GUID original da tabela ESE. Preserve-o ao comparar versões ou provedores desconhecidos. |
| `Provider` | Nome amigável do provedor/da saída, ou o nome original da tabela, se desconhecido. |
| `ID` | O `AutoIncId` da linha; um identificador de linha do banco de dados, não um ID de processo. |
| `Timestamp` | Quando o SRUM registrou/agregou os valores, arredondado para milissegundos. Normalmente não é o horário exato do evento. |
| `EXE-Info` | Identidade resolvida do aplicativo/serviço/pacote a partir de `AppID`. Principal ponto de pivô para o programa. |
| `EXE-InfoDescription` | Descrição extraída de nomes compostos de aplicativos do SRUM. Apenas contexto. |
| `EXE-Timestamp` | Horário embutido em uma identidade de aplicativo composta, quando presente; não é o horário de uso da linha. |
| `AppType` | `NormalApp`, `Service`, `ModernApp` ou `Object`; determina como `EXE-Info` deve ser interpretado. |
| `SID` | SID da conta resolvido a partir de `UserID`. Forte ponto de pivô para a identidade de usuário/serviço. |
| `User` | Nome amigável do perfil enriquecido a partir do `SOFTWARE`; pode ser nulo enquanto `SID` continua válido. |
| `UserID`, `AppID` | Referências numéricas brutas para `SruDbIdMapTable`; preserve-as para nomes não resolvidos ou duplicados. |
| `IsLt` | `true` para a tabela EnergyUsage de longo prazo, `false` para a de curto prazo. |
| `InterfaceLUID` | LUID bruto da interface de rede, a partir das linhas do provedor. |
| `InterfaceType` | Classe de interface decodificada (por exemplo, `IF_TYPE_IEEE80211`); é uma classe, não um adaptador único. |
| `L2ProfileID` | Referência numérica ao perfil de rede. |
| `ProfileName` | Nome da rede Wi-Fi/do perfil enriquecido a partir do `SOFTWARE`; potencialmente sensível e não é prova de conexão atual. |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | Colunas FILETIME conhecidas de provedores, decodificadas para UTC. Seu significado exato pertence ao provedor, e não é um horário de execução universal. |
| Colunas específicas do provedor | Colunas exportadas do provedor, como `BytesSent`, `BytesReceived` (a coluna `BytesRecvd` do banco de dados), contadores de CPU/disco, durações, contagens, flags ou estimativas. Interprete as unidades de acordo com o provedor e compare linhas dentro do mesmo esquema. |

## Campos de diagnóstico (`--include-value-state`) { #diagnostic-fields-include-value-state }

| Campo | Interpretação |
|---|---|
| `ValueState` | Mapa de cada campo do banco de dados para `parsed`, `null`, `absent`, `unsupported` ou `damaged`. Isso distingue a ausência de evidência de uma falha do analisador. |
| `Raw` | Bytes originais em hexadecimal para os valores que têm bytes. Use-os para validação e decodificação futura. |

`SRUM-IdentityMap.csv` (a partir de `SruDbIdMapTable`) contém `IdIndex`, `IdType` e `IdBlob`: o índice bruto, a categoria da identidade
e o texto decodificado do programa/SID/objeto usado pelas linhas dos provedores.

## Estratégia de investigação { #investigation-strategy }

1. Identifique o programa com `EXE-Info`, `AppType` e `AppID`; não confunda `Id` com PID.
2. Agrupe as linhas de rede por programa, usuário, perfil, interface e intervalo de tempo.
3. Compare os bytes enviados/recebidos com as linhas de base normais e com outros hosts.
4. Correlacione os intervalos suspeitos com Prefetch, logs de eventos, dados de DNS/proxy/firewall e acesso a arquivos.
5. Relate o status de banco de dados sujo e a falta de reprodução dos logs de transação como uma limitação da coleta.

## Mais detalhes { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [SrumECmd, de Eric Zimmerman](https://github.com/EricZimmerman/Srum)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
