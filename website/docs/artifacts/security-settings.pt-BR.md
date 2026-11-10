# Configurações de segurança { #security-settings }

## O que são { #what-they-are }

Atacantes alteram as configurações de segurança do Windows para permanecer ocultos e alcançar credenciais. Eles
desativam o Microsoft Defender ou excluem suas ferramentas dele, fazem o Windows manter senhas em texto claro,
abrem a Área de Trabalho Remota, desativam o firewall e silenciam o registro em log do PowerShell. O Registro mantém
essas configurações:

- **`SOFTWARE`:** o Defender, suas exclusões e políticas; o UAC; as políticas da Área de Trabalho Remota e do
  firewall; a política de execução do PowerShell, as políticas de log e os provedores AMSI; a Restauração do Sistema;
  os canais do log de eventos.
- **`SYSTEM`** (por conjunto de controle): `WDigest`, proteção da LSA, Credential Guard, Restricted Admin,
  NTLM; Área de Trabalho Remota; os perfis do firewall; SMBv1; horários de último acesso; o tamanho de cada log de eventos.
- **`NTUSER.DAT`:** a política de execução e as políticas de log do PowerShell do usuário, e o AMSI para o
  Windows Script Host.

`parse-registry security-settings` relata cada configuração que encontra como um registro. Cada registro tem o
valor da configuração, o que o valor significa e se ele **enfraquece o padrão**: se desativa uma
proteção ou uma fonte de evidências, ou a reduz abaixo do que o Windows atual (10 e 11) faz
sem o valor.

<span class="evidence-strength">Melhor conclusão:</span> esta configuração tinha este valor quando o hive
foi salvo, e a chave foi gravada pela última vez neste horário. Uma configuração enfraquecida é uma pista, não prova de
comprometimento. Administradores, a Política de Grupo, outros produtos de segurança e versões mais antigas do Windows também definem
alguns desses valores, e o significado informa isso quando é comum.

## Localização { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

Colete os três hives com seus logs de transação. Uma configuração alterada pouco antes da coleta
pode estar apenas nos logs.

## Cuidados para o analista { #analyst-cautions }

- **A configuração armazenada não é o estado efetivo da proteção.** São relatados os valores do Registro coletados e as linhas de diagnóstico de listas de valores
  danificadas. A ausência de dados pode refletir padrões, fontes não coletadas ou outro mecanismo de
  gerenciamento; não é prova de que a proteção esteja habilitada ou desabilitada. `WeakensDefault` sinaliza
  configurações que podem reduzir a linha de base indicada quando são respeitadas.
- **As flags legadas do Defender podem ser ignoradas.** `DisableAntiSpyware` e `DisableAntivirus` não
  provam que o Defender parou. A aplicabilidade depende da versão da plataforma, da integração (onboarding) e da proteção
  contra adulteração. [A Microsoft explica essas configurações legadas](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware).
- **Exclusões gerenciadas modernas podem não estar disponíveis em capturas do Registro.** A partir da plataforma
  4.18.25110.6, dispositivos que usam o gerenciamento de configuração do Defender for Endpoint exigem cmdlets do PowerShell
  do Defender suportados para obter a configuração de exclusões. Portanto, a ausência no Registro offline
  não estabelece a inexistência de exclusões.
  [Orientação de configuração da Microsoft](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings).
- **Enfraquecido não significa malicioso.** Exemplos:
  - `DisableAntiSpyware` = 1 também é definido pelo Windows enquanto outro antivírus está registrado.
  - A Área de Trabalho Remota é aberta deliberadamente em servidores.
  - O Windows 7 mantinha logs de eventos menores e não atualizava os horários de último acesso.
- **Os padrões dependem da versão.** Alguns valores eram o padrão anteriormente:
  - `NtfsDisableLastAccessUpdate` = 1 era o padrão antes do Windows 10 1803. Ele é relatado
    sem veredito, mas `0x80000001`, definido manualmente em uma versão posterior, é considerado enfraquecido.
  - `RunAsPPL`, `LsaCfgFlags` e `UseLogonCredential` são comparados com o Windows atual.
- **Exclusões do Defender:** cada caminho, extensão, processo ou endereço é um registro. Toda exclusão
  enfraquece o Defender. `TemporaryPaths`, que o próprio Defender adiciona, não tem veredito. Procure pastas graváveis pelo usuário,
  extensões `.ps1`/`.exe` e LOLBins (`rundll32.exe`, `powershell.exe`).
- **Chaves de política** (`Policy` = True) são gravadas pela Política de Grupo, por ferramentas e por atacantes que
  sabem que as políticas prevalecem sobre as configurações locais. Correlacione-as com os GPOs do domínio.
- **Canais do log de eventos:** apenas 25 canais relevantes para a segurança são relatados, indicando se o Windows
  ativa cada um por padrão. `TaskScheduler/Operational` e `DNS-Client/Operational`, por
  exemplo, ficam desativados por padrão. Use `parse-registry read` para os demais.
- **Horários das chaves:** `KeyLastWrittenUTC` pertence à chave, portanto não é necessariamente o momento em que este valor
  foi alterado. É comum que uma chave `Policies` inteira seja gravada de uma só vez.
- **Conjuntos de controle:** por padrão, é lido o conjunto de controle que o Windows usou por último. `--all-control-sets`
  lê todos eles.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

O resumo conta as configurações por área. Ele lista as configurações que enfraquecem o padrão e todas as
exclusões do Defender.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, o hive e seu tipo (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`). |
| `ControlSet`, `IsCurrentControlSet` | Para `SYSTEM`: o conjunto de controle e se é aquele que o Windows usou por último; desconhecido quando `Select\Current` está ausente ou é inválido. |
| `User`, `UserSID` | Para `NTUSER.DAT`: o nome da pasta de perfil e o SID do usuário obtido do `ProfileList` do sistema. |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`. |
| `Setting` | O que é: `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | O valor: um número em decimal, uma string ou a própria exclusão. Vazio quando o valor não é do tipo da configuração. |
| `Meaning` | O que o valor significa. |
| `WeakensDefault` | `True` quando o valor enfraquece o que o Windows atual faz sem ele, `False` quando não enfraquece, vazio quando isso não se aplica (um tamanho, uma porta, um provedor AMSI) ou não pode ser determinado. |
| `Policy` | Se a chave está sob uma chave `Policies`. |
| `KeyPath`, `ValueName` | A chave e o valor. O nome do valor de uma exclusão é o item excluído; um provedor AMSI não tem nenhum. |
| `KeyLastWrittenUTC` | A última gravação da chave. |
| `DataRaw` | Os bytes do valor (hex), quando não são do tipo da configuração. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Valores do tipo errado e chaves que não podem ser lidas. |

## Estratégia de investigação { #investigation-strategy }

1. Execute com `--weakened-only` em todos os hosts e compare os hosts entre si: uma configuração
   que apenas um host tem se destaca.
2. Verifique cada exclusão do Defender em relação às ferramentas encontradas no host e nos logs do Defender
   (o Evento 5007 registra alterações de configuração).
3. Correlacione os horários das chaves com logons, instalações de serviços e criação de processos. Procure
   linhas de comando de `reg.exe`, `Set-MpPreference`, `netsh advfirewall` e `powershell` (evento
   4688, Sysmon 1 e 13, PowerShell 4104).
4. Para configurações de credenciais (`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`), procure dumping de credenciais e pass-the-hash posteriores.
5. Use `parse-registry read --recover-deleted` para valores excluídos para restaurar um padrão, e
   `--all-control-sets` para conjuntos de controle mais antigos.

## Mais detalhes { #further-details }

- [Microsoft: configurar e validar exclusões do Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: configurações e configuração do Controle de Conta de Usuário](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: configurar a proteção adicional da LSA](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
