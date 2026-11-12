# Histórico do PowerShell { #powershell-history }

## O que o histórico registra { #what-the-history-records }

O PSReadLine salva as linhas de comando digitadas em um host interativo do PowerShell. O arquivo contém texto na ordem em que foi salvo, sem horários por comando. O histórico pode ser editado, apagado, desativado ou filtrado e não comprova que um comando terminou com sucesso. Corrobore a execução e seus horários com logs de eventos do PowerShell, telemetria de processos e outros artefatos de execução.

## Localização e coleta { #location-and-collection }

O local padrão no Windows para Windows PowerShell 5.1 e PowerShell 7+ é:

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

Outros hosts usam nomes como `Visual Studio Code Host_history.txt` nesse diretório. No Linux e macOS, o PowerShell normalmente usa `~/.local/share/powershell/PSReadLine/<Host>_history.txt`. O parâmetro `HistorySavePath` do PSReadLine pode redirecionar o arquivo; esses caminhos padrão são pressupostos de descoberta, não uma lista completa dos locais possíveis.

A entrada por diretório e o `auto-parse` selecionam recursivamente arquivos `<Host>_history.txt` diretamente em uma pasta `PSReadLine`, sem diferenciar maiúsculas e minúsculas ASCII. Não selecionam arquivos `*_history.txt` arbitrários em outros locais. Use explicitamente `-f` para arquivos renomeados ou cópias fora de um diretório `PSReadLine`. Preserve a estrutura original do perfil quando possível para atribuir o usuário.

## Analisar com Juntei { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV é o formato de arquivo padrão. Com `-t csv`, `-t json` ou `-t jsonl` sem `-o`, os registros vão para stdout. Sem ambas as opções, apenas um resumo é exibido. JSON contém objetos separados e formatados; JSONL contém um objeto por linha. Adicione `-C` para sobrescrever uma saída existente; aliases do arquivo de origem são protegidos contra substituição.

O `auto-parse` salva `ShellHistory/PowerShellHistory.csv` (ou o formato JSON escolhido), também para imagens VHDX/E01. O histórico não contribui com eventos para `auto-parse -j` ou `super-timeline`.

## Codificação e tratamento de linhas { #encoding-and-line-handling }

- Uma marca de ordem de bytes UTF-8, UTF-16LE ou UTF-16BE determina a codificação e é removida no início do arquivo. Sem BOM, a entrada é sempre tratada como UTF-8. UTF-16 sem BOM não é inferido; caracteres NUL são preservados e sinalizados como possíveis danos de codificação.
- Sequências UTF-8 inválidas e substitutos UTF-16 sem par tornam-se U+FFFD, com o dano informado em `Problems`. Um byte UTF-16 excedente no final e erros de leitura também são informados; linhas legíveis são mantidas. Confira `Problems` e o log do parser antes de confiar na completude.
- Os arquivos são processados em fluxo. São mantidos no máximo 1,048,576 bytes de entrada por linha física, sem o BOM e o delimitador LF (um CR antes de LF conta para o limite). Linhas maiores são truncadas no limite de um caractere inteiro; os bytes restantes são consumidos e `Problems` informa quantos foram omitidos. O limite se aplica aos bytes de entrada codificados, não à quantidade de caracteres.
- Os finais de linha LF e CRLF são removidos, assim como um CR final de uma linha não terminada. Outros espaços e tabulações são mantidos. Linhas vazias não geram registros, mas contam na numeração e encerram uma cadeia de continuação; linhas apenas com espaços em branco continuam como registros. Erros de leitura podem gerar uma linha de diagnóstico vazia para marcar onde a leitura parou.
- O PSReadLine armazena entradas de várias linhas com um acento grave no final de cada linha física continuada. As linhas ficam separadas, preservando os acentos graves e a indentação. `EntryStartLine` aponta para a primeira linha de uma cadeia consecutiva cujas linhas anteriores terminam com acento grave. Nenhuma sintaxe de shell é interpretada e nenhum comando é reconstruído. O truncamento pode afetar a atribuição de continuações; confira `Problems` antes de agrupar essas linhas.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `SourceFile` | Caminho relativo ao diretório de entrada, usando `/`; com `-f`, apenas o nome do arquivo. Nenhum caminho de origem absoluto é exportado. |
| `User` | Nome do perfil inferido do caminho preservado. A entrada por diretório também reconhece estruturas `Users`/`Documents and Settings` sob a raiz de entrada; `-f` exige o sufixo usual do caminho de perfil do PSReadLine. Valores desconhecidos ficam vazios no CSV e como `null` no JSON. |
| `ShellHost` | Prefixo do nome do arquivo antes de `_history.txt`, como `ConsoleHost`; desconhecido para arquivos renomeados sem esse sufixo. |
| `LineNumber` | Número da linha física a partir de 1, incluindo linhas vazias ignoradas. |
| `EntryStartLine` | Linha física inicial da cadeia de continuação, ou o número da própria linha. |
| `Command` | Linha física decodificada sem seu final de linha, sujeita ao limite documentado e à proteção CSV. |
| `FileLastWriteTime` | Horário de modificação do arquivo de origem, repetido em cada linha. **Metadado do arquivo, nunca horário de execução de comando.** A análise de imagens usa os metadados originais, não o horário de staging. |
| `FileSize` | Tamanho do arquivo de origem em bytes. |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE` ou `UTF-16BE`. |
| `Problems` | Diagnósticos de danos e truncamento das linhas; separados por ponto e vírgula no CSV e como array no JSON/JSONL. Diagnósticos do arquivo também aparecem no log. |

`FileLastWriteTime` usa UTC no formato ISO 8601 por padrão, por exemplo `2026-10-20T08:46:38.5Z`. Somente `auto-parse` aceita `-Z, --timezone` para alterar o deslocamento exibido. O comando independente `parse-file powershell-history` não tem opção de fuso horário.

Cada célula de texto CSV é verificada quanto a prefixos de fórmulas de planilha (`=`, `+`, `-`, `@`, incluindo formas de largura total e prefixos após espaços em branco ou certos caracteres invisíveis/de controle). Células afetadas e células que começam com caracteres de controle recebem um apóstrofo inicial. Isso pode alterar comandos e caminhos no CSV. JSON e JSONL mantêm o texto decodificado sem o apóstrofo CSV; substituições de decodificação e truncamento continuam aplicáveis. Use JSONL ao comparar textos de comandos salvos e sempre preserve os arquivos de histórico originais.
