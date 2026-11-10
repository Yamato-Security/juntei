# MuiCache { #muicache }

## O que é o MuiCache { #what-muicache-is }

Quando um usuário inicia um programa, o Windows lê os recursos de versão do programa para obter o nome que o
shell deve exibir (o `FileDescription`, por exemplo "Windows Command Processor" para
`cmd.exe`) e o armazena em cache por usuário no `MuiCache`. O Windows 8 e posteriores também armazenam em cache a empresa
(`CompanyName`). O cache é indexado pelo caminho completo do programa:

- **Windows 8 e posteriores:** dois valores por programa, `<path>.FriendlyAppName` e
  `<path>.AppCompany`.
- **Windows Vista e 7:** um valor por programa, `<path>`, contendo o nome amigável.
- **Windows XP:** o mesmo formato, no `NTUSER.DAT`.

O MuiCache pode fornecer pistas de execução de programas na sessão do shell de um usuário, incluindo programas
em unidades removíveis, compartilhamentos de rede e pastas que não existem mais, com o nome e a empresa
que o próprio arquivo informou.

<span class="evidence-strength">Melhor conclusão:</span> este perfil do shell armazenou em cache o caminho,
a descrição e a empresa de um arquivo. Correlacione com outros artefatos de execução. O perfil não identifica
a conta sob a qual o processo foi executado. O MuiCache não tem horário por programa nem contagem de execuções.

## Localização { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (o idioma das strings em cache) não é um programa e é deixado de fora. Valores com nomes
`@<file>,-<number>` (`@shell32.dll,-22019`) são strings de recurso que o shell armazenou em cache para seus próprios
menus e tipos de arquivo, não programas; eles são deixados de fora e contados no resumo. No Windows Vista
e posteriores, o `NTUSER.DAT` não tem MuiCache.

Colete o `UsrClass.dat` (e o `NTUSER.DAT`) com seus logs de transação de cada perfil, além dos hives
`SYSTEM` e `SOFTWARE` do sistema, para o host e os nomes de usuário.

## Cuidados para o analista { #analyst-cautions }

- **O dono do perfil não é necessariamente a conta do processo.** Um programa iniciado com outra
  conta pode exibir uma janela na sessão deste usuário e deixar metadados no cache deste usuário.
  `User` e `UserSid` identificam o dono do hive, não o contexto de segurança do processo.
- **Sem horário por programa.** `KeyLastWrittenUTC` é a última gravação da chave: a alteração mais recente em
  qualquer um de seus programas, não quando um programa específico foi executado. Prefetch, BAM, UserAssist e logs de eventos fornecem
  horários.
- **Não apenas programas que o usuário iniciou manualmente.** Uma DLL ou programa que o próprio shell carregou (para
  `Open with`, uma extensão do shell, um processo filho de um instalador) pode aparecer, como `explorerframe.dll` ou
  `shell32.dll`.
- **O nome vem do arquivo.** `FriendlyAppName` e `AppCompany` são o que os recursos de versão do arquivo
  informavam quando ele foi executado. Um malware pode declarar qualquer descrição e empresa; uma ferramenta renomeada
  mantém sua descrição original, o que pode revelá-la (um arquivo chamado `svchost.exe` cujo
  nome amigável é uma ferramenta de acesso remoto).
- **Uma entrada por caminho.** Executar o mesmo programa novamente não adiciona uma entrada; uma cópia em outro
  caminho, sim. A entrada de um programa pode permanecer depois que o arquivo é excluído.
- **Ambos os formatos para um caminho.** Quando um caminho tem um nome amigável nos dois formatos (um perfil
  atualizado), cada um recebe seu próprio registro, para que nenhum valor seja perdido.
- **Valores danificados.** Um valor que não é string, não é UTF-16 válido ou não pode ser lido por inteiro mantém
  seus bytes (`FriendlyAppNameRaw`, `AppCompanyRaw`) e é relatado em `Problems`.
  UTF-16 inválido também pode produzir caracteres de substituição no texto exibido; use a coluna bruta
  para obter os bytes originais.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

O resumo lista os programas fora das pastas Windows e Program Files (pastas de usuário,
outras unidades, compartilhamentos de rede) com seus usuários, e o número de strings de recurso deixadas de fora.
Pastas Program Files com outros nomes (no Windows em outros idiomas) também são listadas.
Caminhos relativos e caminhos que escapam de uma pasta de variável de ambiente continuam listados. Caminhos comuns
de unidades são verificados após a resolução dos componentes `.` e `..`, sem acessar o sistema de arquivos
do examinador.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, nome da pasta de perfil, SID do usuário (a chave raiz de um `UsrClass.dat` o nomeia; caso contrário, o `ProfileList` do sistema) e o hive. |
| `HiveType` | `UsrClass.dat` ou `NTUSER.DAT`. |
| `Path` | O caminho do programa: o nome do valor sem `.FriendlyAppName` ou `.AppCompany`. |
| `FriendlyAppName` | A descrição do programa, de `<path>.FriendlyAppName` ou `<path>`. |
| `AppCompany` | A empresa do programa, de `<path>.AppCompany` (Windows 8 e posteriores). |
| `KeyPath`, `KeyLastWrittenUTC` | A chave MuiCache e sua última gravação: a alteração mais recente em qualquer um de seus programas. |
| `FriendlyAppNameRaw`, `AppCompanyRaw` | Os bytes do valor (hex), quando não são um texto decodificado sem problemas. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro: `replayed` quando os logs gravaram um dos valores do programa. |
| `Problems` | Valores que não são strings, não são UTF-16 válido ou não podem ser lidos. |

## Estratégia de investigação { #investigation-strategy }

1. Examine programas em locais graváveis pelo usuário (`\Users\`, `Downloads`, `Temp`, `ProgramData`), em
   outras unidades e em compartilhamentos de rede, além de ferramentas de administração, acesso remoto e compactação.
2. Compare o nome do arquivo com seu nome amigável e empresa: uma divergência pode revelar uma ferramenta
   renomeada ou um arquivo mascarado.
3. Correlacione com Prefetch, BAM e UserAssist (quando foi executado), Amcache e ShimCache (o hash
   e a presença do arquivo) e o `$MFT` (se o arquivo ainda existe).
4. Use `parse-registry read --recover-deleted` no `UsrClass.dat` para valores excluídos do MuiCache.

## Mais detalhes { #further-details }

- [Microsoft: recurso VERSIONINFO](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage: experimentos com MuiCache, sessões de GUI e limitações da conta do processo](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
