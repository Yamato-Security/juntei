# Índice do Windows Search { #windows-search-index }

## O que o índice registra { #what-the-index-records }

O Windows Search (o serviço `WSearch`) rastreia os perfis de usuário, o menu Iniciar, as caixas de correio do
Outlook, o histórico do Internet Explorer, os arquivos offline e tudo o mais que estiver em seu escopo, e guarda o
que leu no `Windows.edb`, um banco de dados ESE. Duas partes dele importam para uma investigação:

- o **repositório de propriedades** (property store; `SystemIndex_PropertyStore`, ou `SystemIndex_0A` no
  Windows 7), com uma linha por item indexado: seu caminho, nome, tipo e tamanho, seu proprietário, os horários
  de criação, modificação e acesso que o indexador copiou do arquivo, quando o indexador o leu, o título e o autor
  de um documento, o início do seu texto, o nome do computador e centenas de outras propriedades;
- as tabelas do **coletor** (gatherer; `SystemIndex_Gthr`, `SystemIndex_GthrPth`), com o registro próprio do
  rastreador para cada item: seu caminho em URL, seu horário de modificação quando foi colocado na fila, o número
  e os flags do rastreamento, e um contador de exclusões.

Um item permanece no índice depois que o arquivo, a mensagem ou a página desaparece, até que o rastreador perceba a
exclusão. O índice, portanto, nomeia arquivos que não existem mais, com seus metadados e um extrato do seu texto
como eram quando foram indexados.

<span class="evidence-strength">Melhor conclusão:</span> o indexador leu o item no caminho registrado, com os
metadados registrados, em `GatherTime`, no computador nomeado. O índice mostra que o arquivo existia e o que ele
continha. Ele não mostra que um usuário o abriu.

## Localização e coleta { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Do Windows 7 ao Windows 11 21H2, e as versões do Windows Server dos mesmos anos, mantêm o índice no
`Windows.edb`. O Windows 11 22H2 e posteriores o mantêm em bancos de dados SQLite na mesma pasta
(`Windows.db`, `Windows-gather.db`, `Windows-usn.db`); o Juntei os encontra e nomeia, e os relata como o formato
mais novo não suportado.

Colete a pasta `Applications\Windows` inteira a partir de uma imagem de disco, de uma cópia de sombra de volume ou
após parar o serviço `WSearch`: o banco de dados fica aberto enquanto o serviço está em execução, e suas alterações
mais recentes ficam apenas nos logs de transação `MSS*.log`, que o Juntei não reproduz. Um índice copiado de um
sistema em execução fica sujo; a saída informa isso e lê o que está no disco. A entrada por diretório e o
`auto-parse` leem todo `Windows.edb` em qualquer profundidade; use `-f` para um único arquivo.

## Cuidados para o analista { #analyst-cautions }

- `GatherTime` é quando o indexador leu o item, não quando um usuário o criou, alterou ou abriu. A indexação roda
  em segundo plano, muitas vezes de minutos a horas após uma alteração, e pausa enquanto o computador está ocupado
  ou na bateria.
- `DateCreated`, `DateModified` e `DateAccessed` são cópias que o indexador tirou do arquivo. Elas são atualizadas
  quando o item é indexado de novo, então descrevem o arquivo como estava em `GatherTime`.
- `ItemPath` é o caminho de exibição. `ItemURL` e `GatherPath` são as URLs do rastreador: `file:` para arquivos
  (com o volume como `[GUID]` no Windows 8 e posteriores), `iehistory://{SID}/` para o histórico do Internet
  Explorer, `mapi://` para itens do Outlook, `csc://{SID}/` para arquivos offline e `winrt://{SID}/` para o
  conteúdo de aplicativos do Windows.
- `User` é o perfil nomeado no caminho do item (`C:\Users\<name>\…`). É onde o item estava, não quem o criou;
  `FileOwner` é o proprietário do arquivo como o indexador o viu.
- `AutoSummary` é o início do texto do item. Ele pode citar um documento que desde então foi excluído ou
  criptografado, e é um extrato truncado, não o documento.
- Um banco de dados sujo pode conter páginas que suas páginas de ramo ainda não alcançam. O Juntei segue a cadeia
  própria das páginas folha e informa no resumo quantas leu dessa forma. Suas alterações mais recentes continuam
  nos logs de transação.
- Um item nas tabelas do coletor sem linha no repositório de propriedades (`InTables` é `Gthr`) foi colocado na
  fila ou rastreado, e suas propriedades ainda não foram gravadas ou já foram removidas. Seu caminho, seu nome e
  seus horários do coletor continuam lá.
- O Windows 7 armazena strings em uma codificação própria, comprimida e ofuscada. O Juntei a decodifica; um valor
  que não decodifica permanece em hexadecimal, com uma nota em `Problems`.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

CSV é o formato de arquivo padrão. Com `-t csv`, `-t json` ou `-t jsonl` sem `-o`, os registros vão para
stdout. Sem nenhuma das duas opções, apenas um resumo é exibido. Adicione `-C` para sobrescrever uma saída
existente. A saída nunca é gravada sobre um dos arquivos de entrada, mesmo com `-C`.

O Juntei grava um registro por item indexado, unindo o repositório de propriedades e as tabelas do coletor pelo
identificador do item. O resumo conta os bancos de dados por esquema, os sujos, os itens presentes nas duas
tabelas, apenas no coletor e apenas no repositório de propriedades, as pastas, os itens marcados como excluídos, os
itens por repositório, os itens sem horário armazenado e os problemas.

Danos não interrompem a execução. Um banco de dados truncado mantém os itens que suas páginas restantes contêm, e
o resumo informa o que não pôde ser lido. Um arquivo chamado `Windows.edb` que não é um índice do Windows Search é
relatado e ignorado. Os demais bancos de dados continuam sendo lidos.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `DocumentID` | O identificador do item no índice: o `WorkID` do repositório de propriedades (`DocID` no Windows 7) e o `DocumentID` da linha do coletor. |
| `InTables` | `Gthr+PropertyStore`, `Gthr` (um item na fila ou rastreado sem propriedades) ou `PropertyStore`. |
| `ItemPath` | `System_ItemPathDisplay`: o caminho como o Windows o exibe. Vazio quando o repositório de propriedades não tem linha. |
| `ItemURL` | `System_ItemUrl`: a URL do rastreador para o item (`file:`, `iehistory://`, `mapi://`, `csc://`, `winrt://`). |
| `GatherPath` | O caminho do coletor, montado a partir da árvore de escopos e do nome de arquivo da linha. |
| `Name` | `System_ItemName`, ou o nome de arquivo da linha do coletor. |
| `ItemType`, `ItemTypeText` | `System_ItemType` (a extensão, ou `Directory`) e seu nome de exibição. |
| `Kind` | `System_Kind`: `document`, `folder`, `link`, `program`, `email`, `picture`, …; vários são unidos com `\|`. |
| `MimeType` | `System_MIMEType`. |
| `Store` | `System_Search_Store`: `file`, `mapi` (Outlook), `iehistory`, `csc` (arquivos offline) ou `winrt` (aplicativos do Windows). |
| `IsFolder`, `IsDeleted` | `System_IsFolder` e `System_IsDeleted`, como armazenados. |
| `DeletedCount` | O contador de exclusões da linha do coletor. |
| `Size` | `System_Size` em bytes. Vazio para pastas, que contêm um valor de preenchimento. |
| `FileAttributes` | `System_FileAttributes` como número (16 diretório, 32 arquivo morto, 2 oculto, …). |
| `FileOwner` | `System_FileOwner` (`DOMAIN\user`) como o indexador o leu. |
| `DateCreated`, `DateModified`, `DateAccessed` | Os horários do arquivo como o indexador os copiou (UTC). |
| `ItemDate` | `System_ItemDate`: a data própria do item, como o horário de envio de uma mensagem ou a data em que uma foto foi tirada; para um arquivo, normalmente seu horário de modificação com precisão de segundos. |
| `GatherTime` | `System_Search_GatherTime`: quando o indexador leu o item (UTC). |
| `GatherLastModified` | O `LastModified` da linha do coletor: o horário de modificação do item quando o rastreador o colocou na fila (UTC). |
| `Title`, `Author` | `System_Title` e `System_Author`. |
| `AutoSummary` | `System_Search_AutoSummary`: o início do texto do item. |
| `CompName` | `System_ComputerName`: o computador ao qual o índice pertence. |
| `TargetURL`, `DateVisited` | A URL e o horário de visita de um item do histórico do Internet Explorer (`System_Link_TargetUrl`, `System_Link_DateVisited`). |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | Um item do histórico de atividades do Windows 10 (`System_Activity_*`, `System_ActivityHistory_*`). |
| `User`, `UserSource` | O perfil nomeado no caminho do item, e de qual caminho ele foi lido (`ItemPath`, `ItemURL` ou `GatherPath`). Vazio fora de um perfil. |
| `ScopeID`, `SDID` | Os identificadores de escopo e de descritor de segurança da linha do coletor. |
| `CrawlNumber` | O `CrawlNumberCrawled` da linha do coletor. |
| `GatherFlags` | Os `TransactionFlags` da linha do coletor. |
| `Priority` | A prioridade de rastreamento de um item ainda na fila. O ESE a marca como NULL assim que o item é rastreado, então ela fica vazia na maioria dos itens. |
| `PropertyCount` | Quantas propriedades o repositório de propriedades contém para o item. |
| `OtherProperties` | Todas as outras propriedades, como `Name=Value` separadas por `; `. |
| `SourceFile` | O banco de dados lido, relativo ao diretório de entrada, ou o seu nome com `-f`. |
| `Problems` | Valores que não decodificaram, horários que não são horários, linhas duplicadas e danos. Não suprima. |

JSON e JSONL mantêm os mesmos campos com valores tipados: números continuam sendo números e valores ausentes são
`null`. Em vez de `OtherProperties`, eles têm `Properties`, cada propriedade do item com seu nome, identificador
numérico, tipo de coluna e valor, e, para um horário, sua contagem bruta de ticks FILETIME ao lado do horário que
ele representa. `Gather` contém as colunas da linha do coletor.

## Comportamento na linha do tempo { #timeline-behavior }

O `auto-parse` salva os registros como `FileExistence/WindowsSearch.csv` (ou `.json`/`.jsonl`) e os adiciona à
linha do tempo combinada `-j`. O `super-timeline` gera eventos a partir de `DateCreated`, `DateModified` e
`DateAccessed` (os horários do arquivo como registrados pelo Windows Search), de `GatherTime` (quando o item foi
indexado), do `DateVisited` de um item do histórico do Internet Explorer e de `ActivityStartTime` e
`ActivityEndTime` de uma atividade. `GatherLastModified` só vira evento para um item para o qual o repositório de
propriedades não contém horário de modificação. Um item sem nenhum desses horários permanece na saída analisada e
é contado como sem horário.

## Estratégia de investigação { #investigation-strategy }

1. Filtre `Store` por `file` e procure em `ItemPath`, `Name` e `AutoSummary` os arquivos, nomes e palavras do
   caso. O índice nomeia arquivos que desde então foram excluídos.
2. Leia `GatherTime` como o último horário em que o arquivo existia com os metadados registrados, e
   `DateModified` como seu horário de modificação naquele momento.
3. Use `FileOwner` e `User` para associar um arquivo a uma conta, e depois confirme com a `$MFT`, o Diário USN e
   `$Secure:$SDS`.
4. Para e-mail (`Store` `mapi`) e histórico do Internet Explorer (`iehistory`), pivote para os arquivos do Outlook
   e para o WebCache a fim de obter os registros completos.
5. Compare o índice entre hosts: `CompName` nomeia o computador ao qual o índice pertence, e um índice coletado de
   uma imagem nomeia os arquivos dessa imagem.

## Versões testadas { #tested-versions }

Os testes usam índices públicos com resultados que outras ferramentas leram de forma independente:

- O índice do Windows 7 do plaso (`SystemIndex_0A`), conferido com o `esedbexport` da libyal, incluindo as
  codificações de strings do Windows 7.
- O índice do Windows 10 do DFIRArtifactMuseum, deixado sujo, conferido com o SIDR da Stroz Friedberg, e seu
  índice do Windows 11 (revisão 200 do formato ESE), conferido com o `esedbexport`.
- O índice SQLite do Windows 11 22H2 do DFIRArtifactMuseum, relatado como o formato mais novo não suportado.
- Bancos de dados truncados, vazios ou de outro tipo.

## Mais detalhes { #further-details }

- [Microsoft: visão geral do Windows Search](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb: formato de arquivo do banco de dados ESE](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb: Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg: SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft: código-fonte do Extensible Storage Engine](https://github.com/microsoft/Extensible-Storage-Engine)
