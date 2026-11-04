# Listas MRU de itens usados mais recentemente (RunMRU, TypedPaths, WordWheelQuery, TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## O que são as listas { #what-the-lists-are }

O Windows e seus programas mantêm listas curtas do que o usuário digitou ou abriu por último, no
`NTUSER.DAT` do usuário:

- **RunMRU:** comandos digitados na caixa de diálogo Executar (Win+R): `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`.
- **TypedPaths:** caminhos digitados na barra de endereços do Explorer.
- **WordWheelQuery:** termos pesquisados na caixa de pesquisa do Explorer.
- **TypedURLs:** endereços digitados na barra de endereços do Internet Explorer e, desde o Windows 8,
  `TypedURLsTime`, quando cada um foi digitado.
- **Paint e WordPad:** os arquivos que cada um abriu por último (`Recent File List`).
- **ACMru (Windows XP):** os termos do Assistente de Pesquisa, por tipo de pesquisa.

<span class="evidence-strength">Melhor conclusão:</span> o usuário digitou este comando, caminho,
termo de pesquisa ou endereço, ou o programa abriu este arquivo. Apenas a entrada mais recente de cada lista
tem data, dada pelo horário da última gravação de sua chave, exceto os endereços digitados, que `TypedURLsTime` data.

## Localização { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

As subchaves de `ACMru` são os tipos de pesquisa: `5603` nomes de arquivos e pastas, `5604` palavras ou frases em
arquivos, `5647` computadores e pessoas, `5001` a Internet.

Colete o `NTUSER.DAT` com seus logs de transação para cada perfil.

## Cuidados para o analista { #analyst-cautions }

- **Um horário por lista.** O horário da última gravação de uma chave muda quando sua lista muda, portanto data apenas a
  entrada mais recente (`IsMostRecent`, `LastUsedUTC`). As demais estão em ordem, sem data.
  `TypedURLsTime` data cada endereço digitado (`TypedTimeUTC`).
- **Digitado, não executado.** O RunMRU registra o que foi digitado na caixa de diálogo Executar, quer tenha sido executado
  com sucesso ou não. O Windows armazena cada comando com um `\1` no final; o Juntei o remove. Correlacione
  com Prefetch, BAM, UserAssist e eventos de criação de processos.
- **As listas são curtas.** Cada uma mantém apenas suas entradas mais recentes, e configurações de privacidade e ferramentas de limpeza
  as apagam. Use `parse-registry read --recover-deleted` para valores excluídos.
- **Os termos são o que foi digitado.** WordWheelQuery e ACMru guardam o texto digitado, não o que foi encontrado.
- **A ordem vem da lista.** O `MRUList` do RunMRU e o `MRUListEx` do WordWheelQuery fornecem a
  ordem; para as demais, quem a fornece é o número do valor (`url1`, `File1` e `000` são os mais recentes).

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

O resumo lista as entradas por lista e as entradas com data mais recentes (endereços digitados e a entrada
mais recente de cada lista), das mais novas para as mais antigas.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, o proprietário do perfil (a partir do caminho ou das pastas do shell), o SID do `ProfileList` do sistema e o hive. |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad` ou `ACMru`. |
| `Subkey` | A subchave de `ACMru`: o tipo de pesquisa (`5603`, `5604`, `5647`, `5001`). |
| `MRU-Position`, `IsMostRecent` | A posição na lista (0 é a mais recente); vazio quando a lista não nomeia o valor. |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | O comando (sem seu `\1`), caminho, termo de pesquisa, endereço ou arquivo. |
| `TypedTimeUTC` | Quando o endereço foi digitado, a partir de `TypedURLsTime`. |
| `LastUsedUTC` | O horário da última gravação da chave, apenas para a entrada mais recente. |
| `KeyPath`, `KeyLastWrittenUTC` | A chave e sua última gravação: o horário de sua entrada mais recente, não de todas as linhas. |
| `DataRaw` | Os bytes do valor (hex) quando não podem ser decodificados. |
| `TransactionLogStatus`, `LogsApplied` | Contexto da recuperação do Registro. |
| `Problems` | Valores que a lista nomeia mas a chave não contém, valores que ela não nomeia, valores com nomes incorretos, tipos errados, termos truncados, horários que não são FILETIMEs. |

## Estratégia de investigação { #investigation-strategy }

1. Leia o RunMRU em busca de comandos, scripts, compartilhamentos de rede e ferramentas de administração digitados pelo usuário.
2. Leia TypedPaths e WordWheelQuery em busca de pastas e arquivos que o usuário procurou:
   credenciais, exportações, arquivos compactados, locais removíveis e de rede.
3. Coloque os endereços digitados na linha do tempo com `TypedTimeUTC`.
4. Correlacione com Prefetch, BAM, UserAssist, ShellBags, arquivos LNK, jump lists e histórico do navegador.

## Mais detalhes { #further-details }

- [libyal winreg-kb: chaves de itens usados mais recentemente (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
