# Navegadores web { #web-browsers }

## O que são os bancos de dados dos navegadores { #what-the-browser-databases-are }

Os navegadores guardam seu histórico, downloads, cookies e entradas de formulários em bancos de dados SQLite no perfil
de cada usuário. Os navegadores baseados no Chromium (Chrome, Edge, Brave, Opera, Vivaldi) e os baseados no Firefox usam
cada um seus próprios arquivos e tabelas, mas registram os mesmos tipos de atividade:

- **Histórico**: as páginas visitadas, cada visita com seu horário, como o navegador chegou à página (endereço digitado,
  um link, um redirecionamento, um favorito) e, no Chromium, por quanto tempo a página ficou em primeiro plano.
- **Downloads**: a URL de onde veio um arquivo (e os redirecionamentos no caminho), onde ele foi salvo, seu tamanho,
  quando o download começou e terminou e se foi concluído.
- **Cookies**: os sites que os definiram, quando foram definidos e enviados pela última vez, e quando expiram.
- **Histórico de formulários**: valores digitados em formulários web e caixas de pesquisa, quantas vezes, e a primeira e
  a última vez.

A atividade do navegador costuma ser o registro mais direto do que um usuário fez: a página de phishing que ele seguiu,
a ferramenta que baixou, o webmail que usou, os termos de pesquisa que digitou.

<span class="evidence-strength">Melhor conclusão:</span> o perfil do navegador registrou uma visita à URL no horário
registrado, ou salvou o download no caminho registrado; o banco de dados não diz quem estava ao teclado.

## Localização e coleta { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` é `Default`, `Profile 1`, `Profile 2` e assim por diante. O Chromium moveu `Cookies` para `Network` na
versão 96; as versões anteriores o mantêm ao lado de `History`. Até a versão 25, o Firefox guardava seus downloads em
`downloads.sqlite`; as versões posteriores os guardam em `places.sqlite`.

Colete todos os arquivos do perfil, e cada banco de dados com os arquivos ao lado dele que compartilham seu nome:
`places.sqlite-wal`, `cookies.sqlite-wal`, `formhistory.sqlite-wal`, `History-journal`.

!!! warning "Logs de escrita antecipada (WAL)"
    O Firefox mantém seus bancos de dados no modo de log de escrita antecipada (WAL): as alterações mais recentes ficam
    apenas no arquivo `-wal` até que o navegador as copie para o banco de dados (um checkpoint). O Juntei lê cada banco
    de dados com seu arquivo `-wal`, como o navegador faria. Sem ele, faltam visitas, cookies e entradas de formulários
    recentes, e entradas excluídas no log ainda aparecem como ativas.

## Entradas excluídas { #deleted-entries }

Quando um navegador exclui o histórico, as linhas saem das tabelas, mas seus registros podem permanecer no espaço livre
do banco de dados, em sua lista de páginas livres (freelist) ou em versões mais antigas de suas páginas: no arquivo
`-wal`, ou no próprio arquivo do banco de dados quando um `-wal` substituiu essas páginas. O Juntei recupera deles
visitas, páginas, downloads e entradas de formulários excluídos, como linhas cujo `Status` é `Deleted`:

- `Confidence` indica o grau de certeza da correspondência do registro com sua tabela: `High` para um registro completo
  em que todos os valores têm o tipo de sua coluna, `Medium` ou `Low` para registros danificados ou registros que se
  encaixam em várias tabelas.
- `RecoveredFrom` indica onde o registro estava: a área da página (uma célula, um freeblock, espaço não alocado, uma
  página livre), qual página e qual versão dela, e o deslocamento do registro na página. Os frames do WAL são numerados
  a partir de 0.
- `LostColumns` lista as colunas que um registro danificado não contém mais: seus campos ficam vazios.

Os navegadores excluem com o `secure_delete` do SQLite: o espaço liberado no banco de dados é zerado. O que pode ser
recuperado fica então principalmente em versões mais antigas das páginas: as que o arquivo `-wal` ainda contém, e as
cópias que o próprio arquivo do banco de dados guarda das páginas que um `-wal` substituiu, até o próximo checkpoint.
Copie o arquivo `-wal` com seu banco de dados, e faça isso antes que o navegador seja executado novamente.

## Cuidados para o analista { #analyst-cautions }

- Os horários estão em UTC. O Chromium e o Firefox os registram com precisão de microssegundos; o histórico de
  formulários do Chromium, com precisão de segundos.
- Uma visita não significa que o usuário digitou ou clicou: redirecionamentos, frames e páginas abertas por outros
  programas também são visitas. Leia `Transition`, `Typed` e `Hidden`.
- O Chromium criptografa os valores dos cookies: `Value` fica vazio para eles. O Juntei não os descriptografa.
- Uma linha `Removed` do histórico de formulários vem da tabela `moz_deleted_formhistory` do Firefox, que o Firefox para
  Android preenche para o Sync. O Firefox para desktop normalmente a deixa vazia.
- Uma entrada excluída com confiança `Low` pode ser um fragmento de outro registro: confira-a com as linhas ativas e com
  as demais evidências.
- A ausência de linhas excluídas não significa que nada foi excluído: espaço zerado e um checkpoint não deixam nada para
  recuperar. Cookies excluídos ainda não são recuperados.
- O conteúdo determina a família do navegador. Um caminho de perfil compatível especifica `Browser` como Chrome ou Firefox; um caminho conflitante não altera a família. Caso contrário, usa-se `Chromium-based`, `Firefox-based` ou `Safari` (`Unknown` se indeterminado). `User` e `Profile` vêm do caminho e podem ser perdidos ao copiar arquivos.
- Journals de rollback (`-journal`) não são lidos, nem o `WebCacheV01.dat` do Internet Explorer e do Edge legado.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` procura os arquivos pelo nome e identifica cada um pelas suas tabelas: um arquivo `History` de outro programa é
ignorado. `-f` lê um banco de dados independentemente do nome.

## Tabelas de saída { #output-tables }

O CSV cria quatro arquivos, cada um apenas com o cabeçalho quando não tem linhas. As linhas JSON e JSONL trazem `Table`
(`History`, `Downloads`, `Cookies` ou `FormHistory`) depois de `SourceFile`.

| Arquivo CSV | Conteúdo |
|---|---|
| `Browser-History.csv` | Visitas, ativas e excluídas; páginas excluídas cuja URL nenhuma página ativa tem. |
| `Browser-Downloads.csv` | Downloads, ativos e excluídos. |
| `Browser-Cookies.csv` | Cookies. |
| `Browser-FormHistory.csv` | Entradas de formulários, ativas e excluídas; registros de remoção do Firefox. |

## Campos { #fields }

Todas as tabelas começam com estes campos:

| Campo | Interpretação |
|---|---|
| `SourceFile` | O banco de dados do qual a linha foi lida. |
| `User` | O usuário do Windows em cujo perfil está o caminho (a pasta depois de `Users`). |
| `Browser` | Família pelo conteúdo, refinada por um caminho de perfil compatível. |
| `Profile` | O perfil do navegador: a pasta do banco de dados. |
| `Status` | `Live`, `Deleted` (recuperado) ou `Removed` (um registro de remoção do Firefox). |

### Histórico { #history }

| Campo | Interpretação |
|---|---|
| `RecordType` | `Visit`, ou `Page` para uma página excluída cujas visitas também podem ter desaparecido. |
| `VisitTime` | Quando a visita começou (UTC). |
| `URL`, `Title` | A página e seu título, como vistos pela última vez. |
| `Transition` | Como o navegador chegou à página. Chromium: o tipo principal e depois seus qualificadores (`LINK`, `TYPED`, `AUTO_BOOKMARK`, `FORM_SUBMIT`, `RELOAD`, … com `CHAIN_START`, `SERVER_REDIRECT`, `FROM_ADDRESS_BAR`, …); Firefox: `LINK`, `TYPED`, `BOOKMARK`, `EMBED`, `REDIRECT_PERMANENT`, `REDIRECT_TEMPORARY`, `DOWNLOAD`, `FRAMED_LINK`, `RELOAD`. |
| `VisitDuration` | Segundos em que a página ficou em primeiro plano (Chromium; 0 quando não foi medido). |
| `VisitCount` | Total de visitas à página. |
| `Typed`, `TypedCount` | Se a URL chegou a ser digitada, e quantas vezes (Chromium). |
| `Hidden` | Oculta na lista do histórico: subframes, origens de redirecionamentos, erros. |
| `Frecency` | A classificação da página pelo Firefox para as sugestões da barra de endereços. |
| `VisitID`, `FromVisitID` | O ID de linha da visita e o da visita da qual ela veio (a página de referência, a origem do redirecionamento). |
| `LastVisitTime` | A última visita de uma página excluída (UTC). |
| `PageSource` | De onde vêm a URL e o título de uma visita excluída: `Live` (uma página ativa), `Recovered` (uma página excluída) ou `NotFound`. |
| `Confidence`, `RecoveredFrom`, `LostColumns` | Como uma entrada excluída foi recuperada; consulte [Entradas excluídas](#deleted-entries). |

### Downloads { #downloads }

| Campo | Interpretação |
|---|---|
| `StartTime`, `EndTime` | Quando o download começou e terminou (UTC). |
| `URL`, `RedirectChain` | De onde o arquivo veio, e todas as URLs no caminho (Chromium; ` -> ` entre elas no CSV). |
| `TargetPath`, `CurrentPath` | Onde o arquivo foi salvo (um URI `file://` no Firefox), e onde foi gravado durante o download. |
| `ReceivedBytes`, `TotalBytes` | Bytes recebidos, e o tamanho do arquivo. |
| `State` | `Complete`, `InProgress`, `Cancelled`, `Interrupted`, `Paused`, `Blocked` ou `Dirty` (bloqueado como malware). |
| `DangerType` | O veredito do Chromium sobre o arquivo: 0 não perigoso, 1 tipo de arquivo perigoso, 2 URL perigosa, 3 conteúdo perigoso, 4 possivelmente perigoso, 5 incomum, 6 mantido pelo usuário, 7 host perigoso, 8 potencialmente indesejado, … |
| `InterruptReason` | Por que um download interrompido parou (Chromium; 0 quando não há motivo). |
| `Referrer`, `TabURL` | A página de referência, e a página na aba que iniciou o download. |
| `MimeType`, `Opened` | O tipo do arquivo como foi servido, e se ele foi aberto a partir do navegador (Chromium). |
| `FileDeletedByUser` | Se o usuário excluiu do disco o arquivo baixado pela lista de downloads do navegador (Firefox). |
| `DownloadID` | O ID de linha do download. |

### Cookies { #cookies }

| Campo | Interpretação |
|---|---|
| `Created`, `LastAccessed`, `Expires` | Quando o cookie foi definido e enviado pela última vez, e quando expira (UTC; `Expires` vazio para um cookie de sessão). |
| `Host`, `Name`, `Path` | O site a que o cookie se destina, seu nome e seu caminho. |
| `Value` | O valor como armazenado: vazio quando o Chromium o criptografou. |
| `Secure`, `HttpOnly`, `Persistent` | Enviado somente por HTTPS, oculto dos scripts, mantido entre sessões (Chromium). |
| `GoogleAnalytics` | Os campos decodificados de um cookie do Google Analytics (`__utma`, `__utmb`, `__utmt`, `__utmz`): horários de visita, sessões, e a origem da campanha e os termos de pesquisa. |
| `RowID` | O ID de linha do cookie. |

### Histórico de formulários { #form-history }

| Campo | Interpretação |
|---|---|
| `FirstUsed`, `LastUsed` | Quando o valor foi inserido pela primeira e pela última vez (UTC). |
| `RemovedTime` | Quando a entrada foi removida, segundo um registro de remoção do Firefox (UTC). |
| `FieldName`, `Value` | O campo do formulário (`email`, `q`, a barra de pesquisa do Firefox: `searchbar-history`) e o que foi digitado. |
| `Count` | Quantas vezes o valor foi inserido. |
| `GUID` | O identificador da entrada no Firefox, que os registros de remoção citam. |
| `Sources` | De onde o Firefox salvou a entrada, nas versões que guardam essa informação (separadas por uma barra vertical no CSV). |
| `RowID` | ID da linha; vazio/null quando perdido na recuperação. Um zero realmente registrado permanece `0`. |

## Análise automática { #automatic-parsing }

O `auto-parse` grava as tabelas em `BrowserActivity` como `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` e `Browser-FormHistory.csv` (um único arquivo `Browser.json` ou `Browser.jsonl` com `-t json` ou
`-t jsonl`), junto com `BrowserActivity/Browser-logs.txt`. A linha do tempo combinada de `-j` inclui todas as linhas, com
`Comp` e o identificador de `Artifact` `browser`. `-Z` converte `VisitTime`, `LastVisitTime`, `StartTime`, `EndTime`,
`Created`, `LastAccessed`, `Expires`, `FirstUsed`, `LastUsed` e `RemovedTime`.

## Mais detalhes { #further-details }

- [Chromium — tipos de transição de página](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — tipos de perigo de download](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — banco de dados Places](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — log de escrita antecipada](https://www.sqlite.org/wal.html)
- [Plaso — analisadores SQLite de navegadores](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
