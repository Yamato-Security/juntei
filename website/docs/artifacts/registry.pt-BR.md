# Hives do Registro do Windows { #windows-registry-hives }

## O que são os hives do Registro { #what-registry-hives-are }

O Registro do Windows armazena a configuração do sistema e dos usuários em arquivos binários de hive. As chaves formam uma árvore;
os valores sob uma chave contêm dados tipados. Os artefatos do Registro descrevem persistência, software instalado,
serviços, contas, configuração de rede, atividade do usuário, configurações de segurança e quase todas as partes
do estado do Windows.

O leitor genérico do Juntei decodifica qualquer hive primário, reproduz os logs de transação, relata danos e pode
recuperar registros excluídos/de células livres, de slack de célula, remanescentes, anteriores à reprodução e existentes apenas nos logs de transação.

<span class="evidence-strength">Melhor conclusão:</span> o hive continha, ou preservava de forma recuperável,
o estado da chave/valor. O horário da última gravação de uma chave mostra que a chave mudou, não qual valor mudou nem por quê.

## Hives comuns e suas localizações { #common-hives-and-locations }

| Hive | Caminho típico | Exemplos de evidência |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | Serviços, drivers, conjuntos de controles, nome do host, dispositivos montados, ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | Software instalado, TaskCache, perfis, configuração do Windows |
| `SAM` | `C:\Windows\System32\config\SAM` | Contas locais e metadados das contas |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | Metadados da política de segurança local/segredos (dependente de acesso) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | Configuração do perfil padrão do sistema |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | Preferências do usuário, chaves Run, RecentDocs, atividade de aplicativos |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | Classes do shell, atividade de COM/shell do usuário |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | Inventário de aplicativos (use `parse-registry amcache` para obter uma saída semântica) |

Colete os arquivos `.LOG`, `.LOG1` e `.LOG2` adjacentes. Um hive copiado com o Windows em execução pode estar
sujo (dirty); os logs de transação contêm estado confirmado que o arquivo primário, sozinho, ainda não reflete.

## Cuidados para o analista { #analyst-cautions }

- O carimbo de data/hora da última gravação de uma chave se aplica à **chave**, não a um valor individual.
- Os valores do Registro normalmente não têm carimbo de data/hora próprio.
- `CurrentControlSet` é um alias do analista/do Windows resolvido por meio de `SYSTEM\Select`; não é uma
  chave armazenada literalmente.
- Registros excluídos podem estar parciais, desatualizados, duplicados entre fontes de recuperação ou sobrescritos.
- Registros existentes apenas nos logs de transação mostram um estado histórico/intermediário. Explique `recovered_from`.
- Um resultado `absent_value` significa que a chave consultada existe, mas não tem esse valor no estado analisado;
  isso não prova que o valor nunca existiu.
- Cadeias de caracteres, caminhos e linhas de comando são configuração, não prova de que a ação configurada foi executada.
- Um hive sujo sem logs aplicáveis pode omitir ou expor um estado desatualizado.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host` | Nome do host derivado do contexto do sistema disponível; vazio quando o Juntei não consegue identificá-lo com segurança. |
| `HiveType` | Categoria do hive detectada (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT` etc.). Use-a para interpretar os caminhos. |
| `SourcePath` | O hive de entrada exato. Essencial quando o modo de diretório combina hosts/usuários. |
| `RecordType` | `key`, `value`, `default_value` ou `absent_value`. A ausência se aplica apenas à consulta/ao estado. |
| `KeyPath` | Caminho abaixo da raiz do hive. `[unknown]` marca uma ascendência excluída não resolvida. |
| `KeyLastWrittenUTC` | Última modificação da chave como um todo. Não consegue identificar qual valor mudou. |
| `ValueName` | Valor nomeado; vazio para um valor padrão/sem nome. |
| `ValueType` | Tipo do Registro (`REG_SZ`, `REG_DWORD`, `REG_BINARY` etc.). O tipo afeta a interpretação. |
| `ValueData` | Valor renderizado de acordo com o tipo. Multicadeias (multi-strings) continuam sendo listas no JSON. |
| `DataSize` | Tamanho registrado pelo valor. Compare com os bytes brutos e com `Problems`. |
| `ValueDataRaw` | Bytes exatos quando a decodificação falha ou quando `--raw-data` é usado. Preserve-os para validação. |
| `ValueU64` | Representação numérica inteira para DWORD/QWORD sem perda de precisão do JavaScript. |
| `ValueFiletimeUTC` | Renderização opcional de QWORD como FILETIME. Só faz sentido quando o esquema da chave define um FILETIME. |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only` ou `transaction_log_recovered`. Determina o grau de confiança. |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay` ou uma entrada de log como `SYSTEM.LOG1#1234`. |
| `TransactionLogStatus` | `clean`, `replayed`, `unchanged` ou `not_applied` para o contexto deste registro/hive. |
| `LogsApplied` | Nomes dos arquivos de log de transação que o Juntei reproduziu. Registre-os no relatório. |
| `Problems` | Danos, células ausentes, incompatibilidade de tipo, listas/pais não resolvidos ou outras limitações. |

## Interpretação do status de recuperação { #recovery-status-interpretation }

| Status | Significado |
|---|---|
| `deleted_complete` | A chave/o valor excluído foi recuperado com caminho/dados conhecidos completos. Ainda assim é histórico, não estado ativo. |
| `deleted_partial` | Parte do caminho, da lista de valores, do valor ou dos dados foi sobrescrita/está ilegível. Não preencha as lacunas com células reutilizadas. |
| `residual_value_only` | O valor sobreviveu sem contexto de chave suficiente; `KeyPath` pode estar ausente. |
| `transaction_log_recovered` | O estado existe apenas nas versões anteriores à reprodução/nos logs, e não nos registros atuais alocados/livres. |

## Estratégia de investigação { #investigation-strategy }

1. Comece por uma chave semântica exata quando ela for conhecida; use buscas por regex/dados para descoberta.
2. Verifique `TransactionLogStatus`, `AllocationStatus`, `RecoveredFrom` e `Problems` antes do conteúdo.
3. Interprete os dados de acordo com o esquema documentado da chave; `REG_BINARY` não é autodescritivo.
4. Correlacione a configuração com evidências de execução (logs de eventos, Prefetch, SRUM, serviços/tarefas).
5. Preserve os dados brutos para campos novos ou contestados.

## Mais detalhes { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Especificação do formato do Registro, de Maxim Suhanov](https://github.com/msuhanov/regf)
- [Analisador de Registro yarp](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Registry, de Eric Zimmerman](https://github.com/EricZimmerman/Registry)
- [Microsoft: tipos de valores do Registro](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
