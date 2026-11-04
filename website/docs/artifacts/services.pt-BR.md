# Serviços do Windows { #windows-services }

## O que a configuração de serviços registra { #what-service-configuration-records }

Os serviços e drivers de kernel do Windows são configurados em cada conjunto de controle do `SYSTEM`. Os registros nomeiam
o executável/driver ou a DLL do serviço, o modo de inicialização, o tipo de serviço, a conta, o grupo de ordem de carregamento, a
inicialização atrasada, o comportamento em caso de erro e as ações que o Windows executa após uma falha.

Atacantes criam ou modificam serviços para persistência, escalonamento de privilégios, carregamento de drivers e
movimentação lateral. Softwares legítimos usam exatamente o mesmo mecanismo, portanto caminhos, contas, assinatura,
contexto de criação e evidências de execução são importantes.

<span class="evidence-strength">Melhor conclusão:</span> o conjunto de controle analisado configurava o
serviço/driver conforme mostrado. A configuração por si só não prova que o serviço foi iniciado com sucesso.

## Localização { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

Colete o `SYSTEM` com seus logs de transação. IDs de evento como System 7045 e Security 4697 podem
registrar a instalação; eventos de controle de serviços e de processos podem sustentar a inicialização/execução efetiva.

## Cuidados para o analista { #analyst-cautions }

- `ImagePath` pode conter variáveis de ambiente, aspas, argumentos, caminhos de dispositivo ou um caminho
  sem aspas com espaços. Preserve-o exatamente antes de normalizá-lo.
- `ServiceDll` é fundamental para serviços hospedados em `svchost.exe`; `ImagePath` sozinho pode nomear apenas o host.
- `StartType=automatic` não prova que o serviço foi iniciado nem que permaneceu habilitado.
- `ObjectName` é a conta configurada, não necessariamente o usuário que instalou o serviço.
- Drivers e serviços em modo de usuário usam flags `ServiceType` e convenções de caminho diferentes.
- As ações de falha podem executar um comando como a conta do serviço após uma falha; essa é uma superfície de persistência
  fácil de passar despercebida.
- Outros conjuntos de controle podem preservar configurações anteriores; use `IsCurrentControlSet`.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath` | Identidade do sistema e hive de entrada. Preserve entre coletas. |
| `ControlSet`, `IsCurrentControlSet` | Conjunto de controle de origem e se `Select\Current` o nomeia. Prefira o atual para afirmações sobre o estado ativo. |
| `ServiceName` | Nome da chave do Registro/nome interno do serviço. Pivô forte para eventos e comandos do Service Control Manager. |
| `DisplayName`, `Description` | Texto exibido ao usuário; pode ser referência a recursos ou controlado pelo atacante. |
| `ImagePath` | Comando do executável/driver conforme configurado. Analise aspas, argumentos, variáveis de ambiente e caminhos de dispositivo. |
| `ServiceDLL` | DLL carregada por um serviço hospedeiro. Frequentemente é a carga útil real em serviços de processo compartilhado. |
| `StartType`, `StartValue` | Modo de inicialização legível/bruto: boot (0), system (1), automatic (2), manual (3), disabled (4). |
| `DelayedAutostart` | O serviço automático inicia após a fase inicial de inicialização; ainda é capaz de garantir persistência. |
| `ServiceType`, `TypeValue` | Flags decodificadas/brutas: driver de kernel, driver de sistema de arquivos, processo próprio/compartilhado, serviço de usuário etc. |
| `ErrorControl` | Nível de tratamento de erros de boot/inicialização. Principalmente contexto operacional. |
| `ObjectName` | Conta de serviço configurada ou nome do objeto do driver. Não é a identidade de quem instalou. |
| `Group` | Grupo de ordem de carregamento, especialmente importante para drivers de boot/sistema. |
| `FailureResetPeriod` | Segundos até a contagem de falhas ser zerada; `4294967295` significa nunca. |
| `FailureCommand` | Comando configurado para uma ação de falha `run_command`. Campo de persistência de alto valor. |
| `FailureActions` | Lista completa de ações decodificada (`restart`, `reboot`, `run_command`, `none`) e atrasos. |
| `FailureActionType`, `FailureActionDelayMs` | Primeira ação achatada para filtragem; inspecione também a lista completa. |
| `FailureActionsRaw` | Bytes originais quando a decodificação está incompleta/danificada. |
| `KeyLastWrittenUTC` | Última alteração da chave do serviço, não necessariamente o horário de instalação/inicialização. |
| `TransactionLogStatus`, `LogsApplied` | Contexto da reprodução de logs do Registro. |
| `Problems` | Incompatibilidade de tipo, estrutura de falha malformada ou campos ausentes/ilegíveis. |

## Estratégia de investigação { #investigation-strategy }

1. Revise caminhos fora do padrão, diretórios graváveis, contas incomuns, horários de chaves novos e comandos de falha.
2. Expanda as variáveis de ambiente usando a configuração do host afetado, não da máquina do analista.
3. Calcule o hash/verifique a assinatura do executável ou da DLL real, se coletados.
4. Correlacione o nome/caminho do serviço com os eventos 7045/4697, criação de processos, Prefetch, `$MFT` e USN.
5. Compare os conjuntos de controle atual e não atuais em busca de persistência alterada/excluída.

## Mais detalhes { #further-details }

- [Microsoft: árvore do Registro para serviços de dispositivos e drivers](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / tipos e estados de serviços](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: ações de falha de ChangeServiceConfig2](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
