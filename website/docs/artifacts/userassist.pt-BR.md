# UserAssist { #userassist }

## O que é o UserAssist { #what-userassist-is }

O Windows Explorer conta os programas e atalhos que um usuário inicia a partir do shell do Explorer: o
menu Iniciar, a barra de tarefas, a área de trabalho e cliques duplos em pastas. Ele guarda as contagens no
`NTUSER.DAT` do usuário. Cada item tem:

- uma contagem de execuções;
- no Windows 7 e posteriores, quantas vezes e por quanto tempo (em milissegundos) ele teve o foco;
- quando foi executado pela última vez.

Os nomes dos valores são codificados em ROT13. O nome de um programa geralmente começa com o GUID de uma pasta conhecida
(`{1AC14E77-…}` é `System32`).

O UserAssist mostra o que um usuário específico iniciou interativamente, algo que a maioria das outras evidências
de execução não atribui a um usuário.

<span class="evidence-strength">Melhor conclusão:</span> o usuário iniciou este programa ou atalho
pelo Explorer, esta quantidade de vezes, a última neste horário. Programas iniciados de outra forma (uma linha de
comando, um serviço, uma tarefa agendada, outro programa) não são contados.

## Localização { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | Contém |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | Programas (`executable`), Windows 7 e posteriores |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | Atalhos (`shortcut`), Windows 7 e posteriores |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | Programas e atalhos (`active_desktop`), Windows 2000 a Vista |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | Barra de ferramentas da Internet (`internet_toolbar`), Windows 2000 a Vista |

Outros GUIDs são relatados como `other`. Colete o `NTUSER.DAT` com seus logs de transação para todos os
perfis, e o hive `SOFTWARE` do sistema, que fornece o SID do usuário.

## Cuidados para o analista { #analyst-cautions }

- **Novos perfis são pré-preenchidos.** No Windows 7 e posteriores, um novo perfil começa com itens padrão
  (Introdução, Calculadora, Paint, Notas Autoadesivas, …). Suas contagens de execuções diminuem a partir de 14,
  e todos compartilham um único horário de última execução, próximo de quando o perfil foi criado. Eles não são atividade
  do usuário.
- **A versão 3 conta a partir de 5** (Windows 2000 a Vista). A contagem armazenada começa em 5, então 5 é
  subtraído dela. Uma contagem armazenada abaixo de 5 é relatada em `Problems`.
- **Os caminhos usam o caminho padrão da pasta conhecida** (`%windir%\system32`, `%ProgramFiles%`,
  `%APPDATA%\…`). Uma pasta redirecionada para outro lugar ainda mostra seu padrão. IDs de aplicativos
  (`Microsoft.Windows.Explorer`, `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) e itens do Painel de Controle
  (`::{…}`) não têm caminho.
- **O rastreamento pode ser desativado** com `Explorer\Advanced` `Start_TrackProgs` = 0,
  `Policies\Explorer` `NoInstrumentation` = 1 ou `UserAssist\Settings` `NoLog` = 1. O resumo
  lista os hives em que ele está desativado. Um UserAssist vazio ou ausente em um perfil ativo pode indicar
  antiforense.
- **`NoEncrypt` = 1** faz o Windows gravar os nomes sem ROT13. Esses nomes são decodificados incorretamente, então
  cada item desse hive informa isso em `Problems` (e em stderr quando os registros vão para stdout).
- **O horário da última execução e as contagens são mantidos por item,** mas o horário da última gravação da chave
  (`KeyLastWrittenUTC`) é apenas o horário da atualização mais recente de qualquer item.
- **As contagens de execuções podem ser 0** para itens que apenas tiveram o foco. As contagens e os tempos de foco existem apenas na
  versão 5 (Windows 7 e posteriores).

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

O resumo lista os itens executados por último (os mais recentes primeiro) com seus usuários, e os hives em que o
UserAssist está desativado.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, o nome da pasta do perfil, o SID do usuário a partir do `ProfileList` do sistema (com o hive `SOFTWARE` do sistema) e o hive. |
| `GUID`, `Kind` | A chave `{GUID}` e o que ela contém (`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`). |
| `NameRot13`, `Name` | O nome do valor conforme armazenado, e decodificado. |
| `Path` | O nome decodificado como caminho, com o GUID de uma pasta conhecida substituído por seu caminho padrão; vazio para IDs de aplicativos e itens do shell. |
| `RunCount` | Quantas vezes foi iniciado pelo Explorer (versão 3: a contagem armazenada menos 5). |
| `FocusCount`, `FocusTimeMs` | Quantas vezes teve o foco, e por quanto tempo em milissegundos (versão 5). |
| `LastRunUTC` | Quando foi iniciado pela última vez. Vazio quando o Windows armazenou 0, ou quando o horário armazenado ultrapassa 9999 (nesse caso `Problems` o informa e `DataRaw` mantém os bytes). |
| `Format`, `SessionID` | O layout dos dados (5: 72 bytes, 3: 16 bytes) e o número da sessão em que foi gravado pela última vez. |
| `KeyLastWrittenUTC` | A última gravação da chave `Count`: a atualização mais recente de qualquer um de seus itens. |
| `DataRaw` | Os bytes do valor (hex), quando não são decodificados. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Dados de tamanho desconhecido, um tamanho que diverge do `Version` da chave, uma contagem da versão 3 abaixo de 5, uma última execução além de 9999, `NoEncrypt`. |

## Estratégia de investigação { #investigation-strategy }

1. Ordene por `LastRunUTC` e separe os itens pré-preenchidos: eles compartilham um único horário, próximo da
   criação do perfil.
2. Procure programas em locais graváveis pelo usuário (`%USERPROFILE%`, `Downloads`, `Temp`,
   `ProgramData`), unidades removíveis e caminhos de rede, e ferramentas de administração e de acesso
   remoto.
3. Correlacione com Prefetch (execução com horários de execução), Amcache e ShimCache (presença),
   Jump Lists e arquivos LNK (o que foi aberto com eles) e o `$MFT` para os arquivos dos programas.
4. Verifique os hives em que o rastreamento está desativado, e os perfis cujo UserAssist está vazio, em busca de limpeza.
5. Use `parse-registry read --recover-deleted` para valores do UserAssist excluídos.

## Mais detalhes { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: User Assist keys](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
