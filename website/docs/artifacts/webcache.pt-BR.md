# Internet Explorer e Edge legado { #internet-explorer-and-legacy-edge }

## O que é o banco de dados WebCache { #what-the-webcache-database-is }

O `WebCacheV01.dat` é o banco de dados das funções de Internet do Windows (WinINet) no perfil de cada usuário, um banco
de dados ESE. O Internet Explorer 10 e 11 guardam nele seu histórico, downloads, cache e cookies, assim como o Edge
legado, o Edge do Windows 10 de antes de o Edge passar a ser baseado no Chromium. Cada tipo de registro é um contêiner,
uma tabela própria:

- **Histórico** (`History`, e `MSHist01…` para um dia ou uma semana): as URLs visitadas, cada uma com a conta para a
  qual foi registrada, quando foi visitada pela última vez e quantas vezes. O Explorador de Arquivos também registra ali
  os arquivos locais que abre, como URLs `file:///`.
- **Downloads** (`iedownload`): a URL de onde veio um arquivo, onde ele foi salvo e onde foi gravado durante o
  download, seu tamanho, quando o download começou, a página de referência e o tipo MIME.
- **Cookies**: no Windows 10 e posteriores, os próprios cookies (tabelas `CookieEntryEx`), com host, nome, valor e
  horários; antes, os arquivos de cookies que os continham.
- **Cache** (`Content`): os arquivos baixados para exibir páginas, cada um com sua URL, onde está a cópia no disco e os
  cabeçalhos da resposta HTTP. Outros programas que usam o WinINet também guardam seus downloads em cache aqui.
- Armazenamento web (`DOMStore`), listas de compatibilidade (`iecompat`) e outros.

O banco de dados sobrevive aos navegadores: o Internet Explorer foi descontinuado e o Edge baseado no Chromium mantém
seus próprios bancos de dados SQLite (consulte [Navegadores web](browser.md)), mas o banco de dados continua em todos os
perfis, com o que foi registrado antes e o que o Explorador de Arquivos e outros programas ainda registram.

<span class="evidence-strength">Melhor conclusão:</span> o perfil do usuário registrou a URL, o download ou o arquivo
em cache no horário registrado; o banco de dados não diz quem estava ao teclado, nem que um arquivo em cache foi aberto.

## Localização e coleta { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

Os arquivos que os contêineres citam (arquivos em cache, arquivos de cookies) ficam em suas próprias pastas:
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\`, `INetCookies\`, `History\`, e em `%LOCALAPPDATA%\Packages\` para os
aplicativos.

Colete a pasta `WebCache` inteira. O Windows mantém o banco de dados aberto enquanto o usuário está conectado, então uma
cópia de um sistema em execução costuma estar suja (dirty): suas alterações mais recentes ficam apenas nos logs de
transação `V01*.log`.

!!! warning "Logs de transação"
    O Juntei não reproduz os logs de transação do ESE. Um banco de dados que não foi fechado corretamente é contado no
    resumo como sujo e relatado como um problema: as alterações que ainda estão apenas em seus logs ficam de fora. Para
    incluí-las, reproduza primeiro os logs em uma cópia da pasta, com o comando `esentutl /r V01 /d` do Windows
    executado nessa cópia.

## Cuidados para o analista { #analyst-cautions }

- Os horários estão em UTC, com precisão de 100 nanossegundos.
- Uma entrada do histórico é uma URL, não uma visita: `AccessedTime` é sua última visita e `AccessCount`, quantas
  houve. O contêiner `History` guarda uma entrada por URL; o contêiner de um dia ou de uma semana guarda as daquele
  período, e seu nome indica seu primeiro dia e o dia seguinte ao último (`MSHist012026110120261102`: 1º de novembro de
  2026).
- URLs `file:///` no histórico são arquivos locais abertos no Explorador de Arquivos e em outros programas, não páginas
  web.
- `Account` é a conta para a qual uma entrada foi registrada (`Visited: alice@…`); `User` é a pasta de perfil em que
  está o caminho do banco de dados. Eles podem ser diferentes: uma pasta de perfil nem sempre tem o nome de sua conta, e
  o caminho de um banco de dados copiado para outro lugar não contém uma pasta de perfil.
- `Package` indica de quem é uma entrada: do Edge legado (`microsoft.microsoftedge_8wekyb3d8bbwe`), do Internet
  Explorer no Modo Protegido Aprimorado (`windows_ie_ac_001`) ou de outro aplicativo. Vazio, a entrada é do próprio
  usuário: Internet Explorer, Explorador de Arquivos e outros programas.
- A Microsoft não documenta o registro de download. O Juntei lê seu horário de início e seu tamanho apenas quando seu
  cabeçalho cabe dentro do registro, e seus caminhos e demais strings apenas quando as duas últimas são uma URL e um
  caminho do Windows. Caso contrário, a linha traz a URL da própria entrada, e seu horário de acesso quando o início não
  pode ser lido, com um problema. `Publisher` é o nome que o navegador registrou, não uma verificação da assinatura do
  arquivo.
- Uma entrada de cache diz que o arquivo foi baixado e guardado em cache. A cópia no disco pode ter desaparecido, ter
  sido substituída ou nunca ter sido aberta.
- Entradas excluídas ainda não são recuperadas: o ESE pode manter um registro excluído em suas páginas até limpá-las,
  mas o Juntei lê apenas os registros ativos.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` procura `WebCacheV01.dat` pelo nome; `-f` lê um banco de dados independentemente do nome.

## Tabelas de saída { #output-tables }

O CSV cria cinco arquivos, cada um apenas com o cabeçalho quando não tem linhas. As linhas JSON e JSONL trazem `Table`
(`History`, `Downloads`, `Cookies`, `Entries` ou `Containers`) depois de `SourceFile`.

| Arquivo CSV | Conteúdo |
|---|---|
| `WebCache-History.csv` | As entradas dos contêineres de histórico. |
| `WebCache-Downloads.csv` | Downloads. |
| `WebCache-Cookies.csv` | Cookies (Windows 10 e posteriores). |
| `WebCache-Entries.csv` | As entradas de todos os outros contêineres: arquivos em cache, arquivos de cookies, armazenamento web, listas de compatibilidade e o restante. |
| `WebCache-Containers.csv` | Os contêineres. |

## Campos { #fields }

Todas as tabelas começam com estes campos:

| Campo | Interpretação |
|---|---|
| `SourceFile` | O banco de dados do qual a linha foi lida. |
| `User` | O usuário do Windows em cujo perfil está o caminho (a pasta depois de `Users`). |
| `Package` | O aplicativo a que pertencem os dados da linha: a pasta sob `Packages` do diretório de seu contêiner (ou de sua tabela de cookies). Vazio para os dados do próprio usuário. |

### Histórico { #history }

| Campo | Interpretação |
|---|---|
| `Container` | O contêiner: `History`, ou `MSHist01` seguido do primeiro dia de seu período e do dia seguinte ao último. |
| `AccessedTime` | A última visita à URL (UTC); no contêiner de um dia ou de uma semana, a última daquele período. |
| `Account` | A conta para a qual a entrada foi registrada. |
| `URL` | A página ou o arquivo, sem o prefixo `Visited: account@`. |
| `AccessCount` | Quantas vezes a URL foi visitada. |
| `EntryID`, `ContainerID` | O ID da entrada em seu contêiner, e o ID do contêiner. |

### Downloads { #downloads }

| Campo | Interpretação |
|---|---|
| `StartTime` | Quando o download começou (UTC). |
| `URL` | De onde o arquivo veio. |
| `TargetPath`, `CurrentPath` | Onde o arquivo foi salvo, e onde foi gravado durante o download: um arquivo `.partial` no Edge legado, a cópia do cache no Internet Explorer. |
| `TotalBytes` | O tamanho do arquivo. |
| `Referrer` | A página de referência. |
| `MimeType` | O tipo do arquivo como foi servido (Internet Explorer). |
| `Publisher` | O nome que o Edge legado registrou para o editor do arquivo. |
| `AccessedTime` | O horário de acesso da própria entrada (UTC), na maioria das vezes o início. |
| `DownloadID`, `ContainerID` | O ID da entrada em seu contêiner, e o ID do contêiner. |

### Cookies { #cookies }

| Campo | Interpretação |
|---|---|
| `LastModified`, `Expires` | Quando o cookie foi definido, e quando expira (UTC). |
| `Host`, `Name`, `Path` | O site a que o cookie se destina, seu nome e seu caminho. |
| `Value` | O valor como armazenado. |
| `Flags` | Os flags do cookie, como armazenados. |
| `Directory` | O diretório que o banco de dados atribui à tabela de cookies: de quem são os cookies (`…\INetCookies\ESE\`, `…\MicrosoftEdge\Cookies\ESE\` do Edge legado, o de um aplicativo). |
| `CookieTable`, `EntryID` | A tabela (`CookieEntryEx_<n>`) e o ID do cookie nela. |

### Entradas { #entries }

| Campo | Interpretação |
|---|---|
| `Container` | O contêiner: `Content` (arquivos em cache), `Cookies` (arquivos de cookies), `DOMStore` (armazenamento web), `iecompat`, …. |
| `AccessedTime` | Quando a entrada foi usada pela última vez (UTC). |
| `ModifiedTime` | A última modificação do recurso, como informada pelo servidor (UTC). |
| `CreationTime` | Quando a entrada foi criada (UTC). |
| `ExpiryTime`, `SyncTime` | Quando ela expira, e quando foi verificada com o servidor pela última vez (UTC). |
| `URL` | A URL como armazenada: uma URL para um arquivo em cache, `Cookie:alice@example.com/` para um arquivo de cookie, `DOMStore:…`, `iecompat:…`. |
| `Account` | A conta de um arquivo de cookie. |
| `Filename`, `FilePath` | O arquivo da entrada, e onde ele está: o diretório do contêiner e a subpasta que a entrada indica. |
| `FileSize` | O tamanho do arquivo. |
| `AccessCount`, `SyncCount` | Quantas vezes a entrada foi usada, e verificada com o servidor. |
| `EntryType`, `Flags` | O tipo e os flags da entrada, como armazenados. |
| `RedirectURL` | Para onde a URL redirecionou. |
| `ResponseHeaders`, `RequestHeaders` | Os cabeçalhos da resposta HTTP (`HTTP/1.1 200 OK`, `Content-Type`, `Server`, …) e os da solicitação, como armazenados, com todas as suas linhas. |
| `EntryID`, `ContainerID` | O ID da entrada em seu contêiner, e o ID do contêiner. |

### Contêineres { #containers }

| Campo | Interpretação |
|---|---|
| `ContainerID`, `Name` | O ID e o nome do contêiner. |
| `Directory` | Onde seus arquivos estão no disco. |
| `SecureDirectories` | As subpastas de `Directory` em que seus arquivos estão (separadas por vírgula no CSV). |
| `PartitionID` | `M` para os contêineres do próprio usuário, `L` para os do Modo Protegido do Internet Explorer (as pastas `Low`), o SID de um contêiner de aplicativo para os de um aplicativo. |
| `LastAccessTime`, `LastScavengeTime` | Quando o contêiner foi usado pela última vez, e limpo pela última vez (UTC). |

## Análise automática { #automatic-parsing }

O `auto-parse` grava as tabelas em `BrowserActivity` como `WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv` e `WebCache-Containers.csv` (um único arquivo `WebCache.json` ou
`WebCache.jsonl` com `-t json` ou `-t jsonl`), junto com `BrowserActivity/WebCache-logs.txt`. A linha do tempo
combinada de `-j` inclui todas as linhas, com `Comp` e o identificador de `Artifact` `webcache`. `-Z` converte todos os
horários: os campos terminados em `Time`, `LastModified` e `Expires`.

## Mais detalhes { #further-details }

- [Microsoft — Sobre o WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — formato de arquivo de banco de dados ESE](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — analisador de MSIE WebCache](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
