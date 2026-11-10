# Cache do Agendador de Tarefas { #task-scheduler-cache }

## O que é o TaskCache { #what-taskcache-is }

O Agendador de Tarefas do Windows mantém um cache no Registro, sob `SOFTWARE`, que vincula caminhos de tarefas a GUIDs,
gatilhos/listas, entidades de segurança, ações, metadados de execução e descritores de segurança. O Juntei decodifica
ações de execução e de manipulador COM, além de ações legadas de e-mail/mensagem, horários das tarefas e sinais de
tarefas ocultas da interface normal do Agendador de Tarefas.

Tarefas são um mecanismo comum de persistência e de execução remota. O cache do Registro também é valioso
quando um atacante exclui ou adultera o arquivo XML da tarefa.

<span class="evidence-strength">Melhor conclusão:</span> o TaskCache configurou ou preservou a
tarefa/ação. Campos de execução e logs independentes são necessários para provar uma execução específica.

## Locais { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

Colete o `SOFTWARE` com os logs de transação e a árvore `System32\Tasks` completa. Os logs de eventos Operational do Agendador de
Tarefas e Security acrescentam evidências de criação/atualização/execução.

## Cuidados para o analista { #analyst-cautions }

- O cache do Registro e o XML da tarefa podem divergir após adulteração, exclusão, registro incompleto ou danos.
- O `Hidden` do Juntei identifica ocultação estrutural (vínculo com a Tree/descritor de segurança ausente sob
  condições relevantes), e não simplesmente uma tarefa configurada com a flag de interface “Hidden” normal.
- O Windows 7 armazena as ações das tarefas no XML, e não no valor `Actions` do cache; ali, uma ação vazia
  não é automaticamente suspeita.
- `LastRun` e `LastResult` são úteis, mas não constituem um histórico de execução completo e à prova de adulteração.
- `Context` nomeia a entidade de segurança/o contexto configurado, não o usuário que registrou a tarefa.
- CLSIDs de manipuladores COM exigem resolução separada no Registro/em arquivos.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath` | Contexto do host e hive `SOFTWARE` de origem. |
| `TaskName` | Caminho completo no agendador. Pivô principal para o XML e os logs de eventos. |
| `TaskGUID` | GUID que une as entradas de cache Tree e Tasks. Chave de correlação forte. |
| `TaskIndex` | Metadados de Tree/índice usados pelo agendador; contextual, não é uma contagem de execuções. |
| `TaskLists` | Listas do cache (`Boot`, `Logon`, `Plain`, `Maintenance`) que categorizam o acionamento/tratamento. |
| `Hidden` | Indicador estrutural de tarefa oculta. Revise `InTree`, `InTasks` e o estado do descritor antes de concluir que houve adulteração. |
| `InTree`, `InTasks` | Se ambos os lados do cache existem. A ausência de vínculo pode indicar exclusão/adulteração/registro incompleto. |
| `HasSecurityDescriptor` | Se a entrada da Tree tem `SD`; a ausência pode contribuir para o comportamento de tarefa oculta nas versões relevantes. |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | Alterações no nível da chave para cada lado do cache; não são horários exatos de criação/execução. |
| `ActionIndex` | Ação, numerada a partir de zero, dentro da tarefa. Uma tarefa pode gerar várias linhas. |
| `ActionType` | `exec`, `com_handler`, `send_email` ou `show_message`. |
| `Executable`, `Args`, `WorkingDirectory` | Ação de execução efetiva. Interprete aspas, variáveis, caminhos relativos e LOLBins. |
| `COM-ClassID`, `ActionData` | CLSID e dados do manipulador COM, ou carga útil legada de e-mail/mensagem. Resolva o CLSID separadamente. |
| `Context` | Contexto da entidade de segurança codificado com as ações (`LocalSystem`, `Author` etc.); identidade configurada, não quem registrou. |
| `Author`, `URI`, `Description` | Metadados da tarefa. Facilmente controlados pelo atacante e não constituem evidência de confiança. |
| `LastRegisteredUTC` | Horário de registro/atualização do DynamicInfo, quando presente. |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | Metadados de execução/resultado em cache. Correlacione com logs operacionais e evidências de processos. |
| `ActionsRaw` | Bytes originais da ação, se a decodificação estiver incompleta ou for necessária uma validação. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Divergência entre caminho/GUID, tipo de valor incorreto, estrutura de ação truncada ou partes ausentes. |

## Estratégia de investigação { #investigation-strategy }

1. Revise primeiro as tarefas ocultas/inconsistentes e os caminhos que não são da Microsoft.
2. Reconstrua os comandos efetivos, incluindo argumentos, diretório de trabalho, expansão de variáveis de ambiente e resolução COM.
3. Compare o registro do Registro com o XML da tarefa e calcule o hash/obtenha os horários do executável referenciado.
4. Correlacione `TaskName`/GUID/horário com eventos do Agendador de Tarefas, Security 4698/4702, logs de processos, Prefetch e MFT/USN.
5. Pesquise dados do Registro excluídos/recuperados quando um atacante tiver removido a tarefa.

## Mais detalhes { #further-details }

- [Microsoft: esquema do Agendador de Tarefas](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: log de eventos do Agendador de Tarefas](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
