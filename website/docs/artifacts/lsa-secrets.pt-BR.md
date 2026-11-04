# Segredos LSA { #lsa-secrets }

## O que são { #what-they-are }

A Autoridade de Segurança Local (LSA) mantém segredos no hive `SECURITY`. Eles incluem a
senha de logon automático, as senhas das contas sob as quais os serviços são executados, a senha de domínio da conta de computador,
as chaves de sistema do DPAPI e a chave com a qual os logons de domínio em cache são criptografados. Ela
também mantém logons de domínio em cache, para que usuários do domínio possam entrar sem um controlador de domínio.

`parse-registry lsa-secrets` lista quais segredos um sistema contém e quando foram definidos, e quantos
logons em cache ele mantém e usa. **Ele lê apenas nomes e horários: nada é descriptografado, e
nenhum byte de segredo é lido ou gravado.** O Juntei lê evidências; ele não extrai credenciais.

<span class="evidence-strength">Melhor conclusão:</span> o sistema continha um segredo com este nome, definido
neste horário. O segredo de um serviço (`_SC_<service>`) significa que a LSA armazena a senha da conta
sob a qual esse serviço é executado.

## Localização { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| Nome | Tipo | O que é |
|---|---|---|
| `DefaultPassword` | `autologon` | A senha com a qual o Winlogon entra automaticamente. |
| `_SC_<service>` | `service_account` | A senha da conta sob a qual `<service>` é executado. |
| `$MACHINE.ACC` | `machine_account` | A senha da conta de computador em seu domínio. |
| `DPAPI_SYSTEM` | `dpapi_system` | As chaves DPAPI do sistema. |
| `NL$KM` | `cached_logon_key` | A chave com a qual os logons em cache são criptografados. |
| `L$_SQSA_<SID>` | `security_questions` | As perguntas de segurança de uma conta local (Windows 10 e posteriores). |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | Credenciais de conexão discada e de VPN. |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | Outros dados privados, pelo prefixo que `LsaStorePrivateData` lhes atribui. |

Colete o `SECURITY` com seus logs de transação, além do hive `SYSTEM` do sistema, que nomeia o
host.

## Cuidados para o analista { #analyst-cautions }

- **Nomes e horários, não valores.** Se um valor contém dados é determinado pelo seu tamanho. Seu conteúdo
  não é lido.
- **`DefaultPassword` não significa logon automático.** O segredo existe na maioria dos sistemas, inclusive
  naqueles que nunca entram automaticamente. A chave
  `Microsoft\Windows NT\CurrentVersion\Winlogon` do `SOFTWARE` (`AutoAdminLogon`, `DefaultUserName`) indica
  se o logon automático está configurado. Um `CurrentSetUTC` recente é uma alteração que merece explicação.
- **Segredos de serviço:** `_SC_<service>` aparece quando um serviço é configurado para ser executado como uma conta com
  senha. Compare com `parse-registry services` (`ObjectName`) e procure serviços
  instalados recentemente ou desconhecidos. O segredo pode permanecer depois que o serviço deixa de existir.
- **Horários:** `CurrentSetUTC` e `OldSetUTC` indicam quando o Windows definiu os valores atual e
  anterior. A senha `$MACHINE.ACC` muda a cada 30 dias por padrão, portanto seus horários mudam.
  `KeyLastWrittenUTC` e os horários das chaves `CurrVal`/`OldVal` indicam quando o segredo foi alterado
  pela última vez.
- **Logons em cache:** um slot está em uso quando seu registro nomeia um usuário. O cabeçalho do registro não é
  criptografado; apenas o comprimento do nome de usuário é lido. `CachedLogonsKept` é a contagem de `NL$Control`,
  que segue `CachedLogonsCount` (10 por padrão). Slots em uso significam que usuários do domínio entraram
  interativamente, e suas credenciais podem ser quebradas offline a partir do hive. Os nomes dos usuários
  são criptografados e não são decodificados. Somente registros binários validados, com cabeçalho completo e comprimento de nome de usuário consistente, contam como em uso. Se `Problems` relatar slots danificados, a contagem de slots em uso abrange apenas os slots validados; não interprete os slots danificados como vazios.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

Para cada hive, o resumo lista cada segredo com seu tipo e quando foi definido. Em seguida, nomeia
os serviços cujas senhas de conta a LSA armazena e conta os slots de logon em cache em uso.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath` | Host (do hive `SYSTEM` do sistema) e o hive `SECURITY`. |
| `Secret`, `Kind` | O nome do segredo e o que ele é (consulte a tabela acima); `Cache` e `cached_logons` para o registro dos logons em cache. |
| `ServiceName`, `UserSID` | O serviço ao qual se refere um segredo `_SC_` e o SID do usuário fornecido por um nome (`L$_SQSA_`, `RasDialParams`). |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` e `OupdTime`: quando os valores atual e anterior foram definidos. |
| `HasCurrentValue`, `HasOldValue` | Se `CurrVal` e `OldVal` contêm dados. |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | As últimas gravações das chaves `CurrVal`, `OldVal` e do segredo. |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | O registro dos logons em cache: slots, slots em uso e a contagem de `NL$Control`. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Horários malformados ou fora do intervalo (apenas os metadados de horário são exibidos), cabeçalhos de cache danificados, tipos de valor de cache incorretos e chaves que não podem ser lidas. |

## Estratégia de investigação { #investigation-strategy }

1. Procure segredos `_SC_` de serviços desconhecidos ou instalados recentemente e associe-os a
   `parse-registry services` e ao evento 7045 (um serviço foi instalado).
2. Compare os horários de `DefaultPassword` com as configurações de logon automático do `Winlogon`.
3. Em membros de domínio, anote os slots de logon em cache em uso e quando `Cache` foi gravada pela última vez.
4. Compare os horários dos segredos com a linha do tempo: um segredo definido durante a janela da intrusão pode
   indicar uma nova conta de serviço, logon automático ou alteração da conta de computador.

## Mais detalhes { #further-details }

- [Microsoft: objeto de dados privados](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: informações de logon de domínio em cache](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: credenciais em cache](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
