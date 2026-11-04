# BAM / DAM { #bam-and-dam }

## O que são BAM e DAM { #what-bam-and-dam-are }

O Background Activity Moderator (BAM) e o Desktop Activity Moderator (DAM) são serviços do Windows
(Windows 10 1709 e posteriores) que gerenciam o que os aplicativos podem fazer em segundo plano.
Eles mantêm, por usuário, o último horário em que cada programa foi executado, no hive `SYSTEM`:

- uma chave por usuário, nomeada com o SID do usuário;
- um valor por programa: seu nome é o caminho do programa em seu dispositivo
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) ou o ID de um aplicativo empacotado
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`);
- os primeiros 8 bytes dos dados do valor são o horário da última execução (FILETIME).

O BAM associa um programa a um usuário e a um horário sem precisar do hive do próprio usuário, e lista programas executados
a partir de dispositivos removíveis e de rede.

<span class="evidence-strength">Melhor conclusão:</span> um processo deste programa foi executado na
sessão deste usuário; ele foi executado pela última vez neste horário. O BAM não conta execuções, e uma entrada pode permanecer depois que o
programa foi excluído.

## Localização { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

A chave de cada usuário também tem os valores `Version` e `SequenceNumber`; eles não são programas e são
ignorados. Um sistema atualizado a partir do 1709 ou do 1803 pode ter ambos os layouts.

Colete o `SYSTEM` com seus logs de transação, e o `SAM` e o `SOFTWARE` do sistema: eles nomeiam os
usuários.

## Cuidados para o analista { #analyst-cautions }

- **Um horário por programa e usuário.** Apenas a execução mais recente é mantida. Prefetch, Amcache e
  logs de eventos fornecem execuções anteriores.
- **Atividade recente.** Há relatos de que as entradas de programas que não são executados há algum tempo são
  removidas quando o Windows inicia, então o BAM mostra principalmente atividade recente: um programa ausente nele
  ainda pode ter sido executado há muito tempo.
- **Caminhos de dispositivo, não letras de unidade.** `\Device\HarddiskVolume3` é o terceiro volume que o Windows
  numerou, não `C:`. `DevicePath` e `Path` separam o dispositivo do caminho nele; associe o
  volume com `MountedDevices`, o `$MFT` ou outros artefatos antes de escrever uma letra de unidade.
- **Nem todos os programas.** BAM e DAM rastreiam programas nas sessões dos usuários; serviços geralmente não
  aparecem. Entradas de `S-1-5-18` são tipicamente processos de instalação e da tela de entrada
  (`FirstLogonAnim.exe`), e as de `S-1-5-90-0-<n>`, do Desktop Window Manager (`DWM-<n>`).
- **Conjuntos de controle.** Por padrão, é lido o conjunto de controle que o Windows usou por último. `--all-control-sets`
  lê todos: um conjunto de controle mais antigo pode manter uma entrada que o atual não tem mais.
- **Nomes de usuário.** O nome de uma conta local vem do `SAM` (o SID do computador e o RID da
  conta); o de uma conta de domínio, de sua pasta de perfil em `ProfileList`. `UserSource`
  indica qual (`sam`, `profile_list`, `well_known`). Sem eles, apenas o SID é informado. Um RID
  que o `SAM` não tem mais (uma conta excluída) permanece sem nome.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

O resumo lista cada usuário com seu SID, nome e número de entradas, e os programas executados por último
(os mais recentes primeiro).

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath` | Host e o hive `SYSTEM`. |
| `ControlSet`, `IsCurrentControlSet` | O conjunto de controle em que a entrada está e se é aquele que o Windows usou por último. |
| `Moderator` | `bam` ou `dam`. |
| `UserSID`, `User`, `UserSource` | O SID que dá nome à chave, o nome do usuário e de onde ele vem (`sam`, `profile_list`, `well_known`). |
| `Program` | O nome do valor como o Windows o gravou: um caminho de dispositivo ou o ID de um aplicativo empacotado. |
| `DevicePath`, `Path` | Para um caminho de dispositivo, o dispositivo (`\Device\HarddiskVolume3`) e o caminho nele (`\Windows\System32\cmd.exe`); vazio para um aplicativo empacotado. |
| `LastRunUTC` | Quando o programa foi executado pela última vez (com precisão de 100 ns). |
| `KeyPath`, `KeyLastWrittenUTC` | A chave do usuário e sua última gravação: a atualização mais recente de qualquer uma de suas entradas. |
| `DataRaw` | Os bytes do valor (hex), quando não contêm um horário: não é `REG_BINARY` ou tem menos de 8 bytes. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Dados que não contêm um horário e valores que não podem ser lidos. |

## Estratégia de investigação { #investigation-strategy }

1. Ordene por `LastRunUTC` e examine programas em locais graváveis pelo usuário (`\Users\`, `Downloads`,
   `Temp`, `ProgramData`), em dispositivos diferentes do volume do sistema, e ferramentas de administração e
   de acesso remoto.
2. Verifique qual usuário os executou, e contas que não correspondem a um perfil atual.
3. Correlacione com Prefetch (contagens de execução e execuções anteriores), UserAssist (inicializações pelo Explorer), Amcache
   e ShimCache (presença), o `$MFT` (o arquivo do programa) e logs de eventos.
4. Use `--all-control-sets` e `parse-registry read --recover-deleted` para valores do BAM excluídos.

## Mais detalhes { #further-details }

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: Identificadores de segurança (SIDs conhecidos)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

FILETIMEs de última execução fora do intervalo deixam `LastRunUTC` vazio e preservam o valor original em `DataRaw`, com uma explicação em `Problems`. O resumo de atividade recente identifica cada host e mantém distintas as observações de hives/usuários separados.
