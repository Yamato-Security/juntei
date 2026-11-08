# Outlook { #outlook }

## O que são os arquivos do Outlook { #what-outlooks-files-are }

O Outlook para Windows guarda e-mails, contatos, calendários e tarefas em arquivos próprios:

- **OST** (`.ost`): a cópia offline de uma caixa de correio, que o Outlook mantém para contas do Exchange e do
  Microsoft 365 e, desde o Outlook 2013, também para contas IMAP. Os do Outlook 2013 e posteriores têm páginas de
  4 KiB e blocos compactados.
- **PST** (`.pst`): um repositório pessoal: contas POP3, itens arquivados (`archive.pst`), exportações e os
  repositórios que os usuários criam. Os PSTs ANSI são os do Outlook 97 a 2002; os PSTs Unicode, os do Outlook 2003
  e posteriores.
- **MSG** (`.msg`): um item salvo como arquivo, quando é arrastado para fora do Outlook, salvo com **Salvar como**
  ou exportado por outro programa. Mensagens anexadas também são arquivos MSG.
- **OFT** (`.oft`): um modelo do Outlook: um arquivo MSG a partir do qual novos itens são criados.

Cada repositório contém pastas, e cada pasta, seus itens: e-mails, solicitações de reunião e respostas, contatos e
listas de distribuição, itens de calendário, tarefas, anotações, entradas de diário e relatórios. Cada pasta também
guarda itens ocultos, suas informações associadas à pasta (FAI, folder associated information): modos de exibição,
regras, formulários e as configurações do Outlook (itens `IPM.Configuration.*`, como a lista de preenchimento
automático de uma caixa de correio do Exchange). Os anexos de um item podem ser arquivos, referências a arquivos,
objetos OLE ou outros itens (uma mensagem encaminhada como anexo, as exceções de um compromisso recorrente), que têm
seus próprios anexos.

<span class="evidence-strength">Melhor conclusão:</span> o repositório ou arquivo continha o item, com esses
remetentes, destinatários, horários e anexos, quando foi coletado. Ele não diz que o usuário o leu, escreveu ou
enviou, nem que seu remetente é quem o item diz.

## Localização e coleta { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

Os PSTs podem estar em qualquer lugar: os usuários movem os PSTs de arquivamento para outras pastas, unidades
e compartilhamentos de rede. O Outlook guarda os caminhos dos repositórios de cada perfil no Registro do usuário. Os
anexos que um usuário abriu a partir do Outlook são copiados para uma pasta de
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` (`Temporary Internet Files\Content.Outlook` antes do
Windows 8.1), onde podem permanecer.

O Outlook mantém seus repositórios abertos enquanto está em execução, e um OST pode ter dezenas de gigabytes. Colete-os
com o Outlook fechado, ou a partir de uma imagem de disco ou de uma cópia de sombra de volume. O Juntei lê cada
repositório como um fluxo: ele nunca mantém um repositório inteiro na memória.

## Cuidados para o analista { #analyst-cautions }

- O Juntei lê de forma tolerante: ele passa pelo que está danificado, lê o que pode ser lido e relata o restante como
  problemas. Eles são exibidos (os 10 primeiros de cada arquivo), contados no resumo e na tabela de arquivos, e
  listados no campo `Problems` dos itens e anexos a que se referem. Um item que não pode ser lido de forma alguma
  ainda tem uma linha na tabela de itens, com seu `ItemID` e o motivo.
- Os horários estão em UTC, com precisão de 100 nanossegundos. `SubmitTime` é quando o cliente do remetente
  submeteu a mensagem, pelo relógio daquele computador; `DeliveryTime`, quando ela chegou a esta caixa de correio.
  `CreationTime` é quando esta cópia do item foi criada: entregue ou criada na caixa de correio, sincronizada no OST,
  copiada ou importada para o PST, ou salva como arquivo MSG. Não é quando a mensagem foi escrita.
  `LastModifiedTime` é quando o item foi alterado pela última vez no repositório.
- `StartTime` e `DueTime` de uma tarefa são datas no fuso horário do usuário, que o Outlook armazena como
  meia-noite. O Juntei as grava sem deslocamento (`2017-02-19T00:00:00`), e `-Z` as mantém como estão.
- O Outlook grava a meia-noite de 1º de janeiro de 4501 para **Nenhuma** em algumas datas, como a data de conclusão
  de uma tarefa ou o horário de um lembrete. O Juntei a grava como está armazenada: `4501-01-01T00:00:00Z`, sem o `Z`
  em `StartTime` e `DueTime` de uma tarefa.
- `Read` é o sinalizador de lido no momento em que o repositório foi coletado: alguém, ou uma regra, marcou o item
  como lido. Ele não diz nem quem nem quando.
- Os nomes e endereços de um remetente e de um destinatário são o que o programa de envio e os servidores de e-mail
  registraram, e podem ser forjados. Endereços `EX` são nomes distintos (DN) do Exchange
  (`/o=…/ou=…/cn=Recipients/cn=…`); o endereço SMTP fica em `SenderSMTP-Address` e `SMTP-Address` quando o item
  também o armazena. Para verificar de onde uma mensagem veio, leia seus cabeçalhos da Internet: o `0x007D001F` ou o
  `0x007D001E` (`PidTagTransportMessageHeaders`) da tabela de propriedades.
- Os itens excluídos no Outlook ficam em **Itens Excluídos** até que essa pasta seja esvaziada, e são lidos como
  qualquer outro. Itens que não estão mais na tabela de nenhuma pasta, que um repositório pode manter até reutilizar
  seu espaço, ainda não são lidos.
- `ItemID` é o ID de nó da mensagem em seu PST ou OST: o mesmo item pode ter outro em outra cópia do repositório. O
  de um item incorporado é o de seu item pai seguido do índice do anexo (`2097348/0`, `0/0/1`); o item de um arquivo
  `.msg` ou `.oft` é `0`.
- `User` vem apenas do caminho (a pasta depois de `Users`) e se perde quando um arquivo é copiado para outro lugar.
  Um PST pode ser de qualquer usuário.
- O texto que não está em Unicode é lido na página de código do item (a dele mesmo, a de sua localidade ou a sua
  página de código da Internet). `--code-page` informa a página de código dos itens que não informam nenhuma: a
  página de código ANSI do sistema que os gravou, `932` para um sistema japonês.
- A senha de um PST não o criptografa: o Juntei lê PSTs protegidos por senha como qualquer outro. O valor `Encryption`
  de um repositório (`Permute`, a “criptografia compactável” do Outlook, ou `Cyclic`) apenas ofusca seus blocos.
  Mensagens criptografadas com S/MIME guardam seu corpo e seus anexos em um anexo criptografado (`smime.p7m`), que
  não é descriptografado.
- Anexos costumam ser malware. `--extract-attachments` grava seus dados em arquivos nomeados pelo SHA-256, sem
  permissão de execução; nunca os execute e mantenha-os em um sistema de análise isolado.

### Ainda não lidos { #not-read-yet }

Catálogos de endereços offline (OAB), o fluxo de preenchimento automático (`Stream_Autocomplete_*.dat`) e os
arquivos `.nk2`, as configurações de envio/recebimento (`.srs`), os caches de configurações do Outlook (RoamCache,
Autodiscover), os bancos de dados SQLite do Outlook e as mensagens que não estão mais na tabela de nenhuma pasta.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pst, *.ost, *.msg and *.oft
  -f, --file <FILE>      Parse one PST, OST, MSG or OFT file, whatever its name
      --code-page <CP>   Code page of items that don't give theirs (default 1252)

Output:
  -o, --output <PATH>              Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>         csv, json, or jsonl
      --properties                 Add the Properties table: every property as stored
      --extract-attachments <DIR>  Write each attachment's data once, named by its SHA-256
  -C, --clobber                    Overwrite files when saving or extracting
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl \
  | jq 'select(.Table == "Items" and .Hidden == false) | {DeliveryTime, SenderAddress, Subject}'
juntei parse-file outlook -f suspicious.msg -o msg-csv --properties --extract-attachments isolated/
```

`-d` procura arquivos terminados em `.pst`, `.ost`, `.msg` e `.oft`, sem diferenciar maiúsculas de minúsculas. Seja
qual for a forma como um arquivo é informado, seus primeiros bytes dizem o que ele é: `!BDN` para um PST ou OST, a
assinatura de arquivo composto para um MSG ou OFT. Qualquer outra coisa é relatada como um arquivo que não pôde ser
analisado. O código de saída é 1 quando nenhum arquivo pôde ser lido.

`--extract-attachments` grava os dados de cada anexo uma única vez, como `<SHA-256>` com a extensão do anexo quando
ela tem de 1 a 10 letras e dígitos ASCII (`0c7d…1311.jpg`); nenhum nome vindo de um item é usado em um caminho. O
diretório deve estar vazio, a menos que `-C` seja informado; com `-C`, um arquivo de mesmo nome que contenha outra
coisa é substituído.

## Tabelas de saída { #output-tables }

O CSV cria oito arquivos, nove com `--properties`, cada um apenas com o cabeçalho quando não tem linhas. As linhas
JSON e JSONL trazem `Table` (`Files`, `Folders`, `Items`, `Recipients`, `Attachments`, `Contacts`, `Calendar`,
`Tasks` ou `Properties`) depois de `SourceFile`. As linhas de um item vêm logo após ele, e as de seus itens
incorporados vêm logo após elas; a linha de um arquivo na tabela de arquivos vem depois de todas as suas outras
linhas.

| Arquivo CSV | Conteúdo |
|---|---|
| `Outlook-Files.csv` | Uma linha por arquivo lido: o repositório ou arquivo e o que ele contém. |
| `Outlook-Folders.csv` | As pastas dos PSTs e OSTs. |
| `Outlook-Items.csv` | Todos os itens: as mensagens das pastas, seus itens ocultos, os itens dos arquivos `.msg` e `.oft` e os itens incorporados em seus anexos. |
| `Outlook-Recipients.csv` | Os destinatários dos itens. |
| `Outlook-Attachments.csv` | Os anexos dos itens. |
| `Outlook-Contacts.csv` | Os campos de contatos e listas de distribuição. |
| `Outlook-Calendar.csv` | Os campos de itens de calendário e mensagens de reunião. |
| `Outlook-Tasks.csv` | Os campos de tarefas e solicitações de tarefa. |
| `Outlook-Properties.csv` | Com `--properties`: todas as propriedades dos repositórios, pastas, itens, destinatários e anexos, como estão armazenadas. |

## Campos { #fields }

Todas as tabelas começam com estes campos:

| Campo | Interpretação |
|---|---|
| `SourceFile` | O arquivo do qual a linha foi lida. |
| `User` | O usuário do Windows em cujo perfil está o caminho (a pasta depois de `Users`). |

### Arquivos { #files }

| Campo | Interpretação |
|---|---|
| `FileType` | `PST`, `OST`, `MSG`, `OFT` ou `CFB` para um arquivo composto lido como item cujo ID de classe não é o de uma mensagem nem o de um modelo (muitos programas além do Outlook o deixam vazio). |
| `Format` | O layout de um repositório: `ANSI` (Outlook 97 a 2002), `Unicode` (Outlook 2003 e posteriores) ou `Unicode 4K` (os OSTs do Outlook 2013 e posteriores); a versão de arquivo composto de um arquivo MSG ou OFT (`CFB v3`, `CFB v4`). |
| `Version`, `ClientVersion` | Os campos `wVer` e `wVerClient` do cabeçalho de um repositório. |
| `Encryption` | Como os blocos de um repositório são codificados: `None`, `Permute` ou `Cyclic`. |
| `DisplayName` | O nome do repositório (`Personal Folders`, um endereço de e-mail). |
| `RecordKey` | A identidade do repositório (hex), que os IDs de entrada de seus itens contêm. |
| `Folders` | O número de pastas do repositório. |
| `Items` | O número de mensagens de suas pastas, ou 1 para um arquivo MSG ou OFT. |
| `HiddenItems` | O número de itens ocultos (informações associadas à pasta) de suas pastas. |
| `EmbeddedItems` | O número de itens incorporados em anexos, em todos os níveis. |
| `Attachments` | O número de anexos de todos esses itens. |
| `Problems` | O número de problemas encontrados no arquivo. |

### Pastas { #folders }

| Campo | Interpretação |
|---|---|
| `FolderID`, `ParentID` | O ID de nó da pasta e o de sua pasta pai (vazio para a pasta raiz). |
| `FolderPath` | Os nomes das pastas a partir da raiz: `/Top of Personal Folders/Inbox`, `/Root - Mailbox/IPM_SUBTREE/Sent Items`. |
| `Name` | O nome da pasta. |
| `ContainerClass` | O que a pasta contém: `IPF.Note` (e-mail), `IPF.Appointment`, `IPF.Contact`, `IPF.Task`, `IPF.Imap`, …. |
| `ContentCount`, `UnreadCount` | As mensagens, e as mensagens não lidas, que a pasta diz ter. |
| `Messages`, `HiddenItems`, `Subfolders` | As linhas de sua tabela de conteúdo, de sua tabela de conteúdo associado e de sua tabela de hierarquia: o que o Juntei lê. Uma diferença em relação a `ContentCount` merece atenção. |
| `CreationTime`, `LastModifiedTime` | Quando a pasta foi criada e alterada pela última vez (UTC), quando o repositório os guarda. |
| `Problem` | Por que a pasta não pôde ser aberta; nesse caso, suas subpastas não são lidas. |

### Itens { #items }

| Campo | Interpretação |
|---|---|
| `FolderPath` | A pasta do item; vazio em um arquivo MSG ou OFT. |
| `ItemID` | O ID do item: seu ID de nó em um PST ou OST, `0` em um arquivo MSG ou OFT, o ID de seu item pai e o índice do anexo para um item incorporado (`2097348/0`). |
| `Hidden` | Se o item é uma informação associada à pasta: um item oculto. |
| `EmbeddedIn` | O ID do item ao qual ele está anexado; vazio para os demais. |
| `ItemType` | O que a classe de mensagem diz que o item é: `Mail`, `MeetingMessage`, `Appointment`, `Contact`, `DistributionList`, `Task`, `TaskRequest`, `StickyNote`, `Journal`, `Post`, `Report`, `Configuration` ou `Other`. |
| `MessageClass` | A classe de mensagem como está armazenada (`IPM.Note`, `IPM.Note.SMIME`, `IPM.Schedule.Meeting.Request`, `REPORT.IPM.Note.NDR`, …). |
| `SubmitTime` | Quando o cliente do remetente submeteu a mensagem (UTC, pelo relógio do remetente). |
| `DeliveryTime` | Quando ela foi entregue a esta caixa de correio (UTC). |
| `CreationTime`, `LastModifiedTime` | Quando esta cópia foi criada no repositório ou arquivo, e quando foi alterada pela última vez (UTC). |
| `Subject` | O assunto, com seu prefixo (`RE: `). |
| `SenderName`, `SenderAddressType`, `SenderAddress` | Quem a enviou: o nome, o tipo de endereço (`SMTP`, `EX`) e o endereço desse tipo. |
| `SenderSMTP-Address` | O endereço SMTP do remetente, quando está armazenado além de um endereço de outro tipo. |
| `SentRepresentingName`, `SentRepresentingAddress` | Em nome de quem ela foi enviada (um representante envia em nome de um gerente; uma caixa de correio compartilhada), com seu endereço SMTP quando armazenado, senão o endereço de seu tipo. |
| `To`, `Cc`, `Bcc` | Os nomes dos destinatários como o item os exibe (veja a tabela de destinatários para seus endereços). `Bcc` só existe na cópia do remetente. |
| `InternetMessageID`, `InReplyTo` | O `Message-ID` da mensagem, e o da mensagem que ela responde. |
| `ConversationTopic` | O assunto sem seu prefixo, que as mensagens de uma conversa compartilham. |
| `Importance`, `Sensitivity` | `Low`, `Normal` ou `High`; `Normal`, `Personal`, `Private` ou `Confidential`. |
| `Read` | Se o item está marcado como lido. |
| `Flags` | Os sinalizadores da mensagem (`PidTagMessageFlags`, hex): `0x1` lida, `0x2` não modificada, `0x4` submetida, `0x8` não enviada (um rascunho), `0x10` tem anexos, `0x20` de mim, `0x40` oculta, `0x80` reenviada, `0x400` já lida alguma vez, `0x2000` vinda da Internet. |
| `Size` | O tamanho do item no repositório, com seus anexos. |
| `Recipients`, `Attachments` | Quantos destinatários e anexos ele tem. |
| `Categories` | Suas categorias, separadas por `;`. |
| `BodyPreview` | O início de seu corpo em texto sem formatação, com os espaços em branco condensados, no máximo 256 caracteres; vazio quando ele só tem um corpo HTML ou RTF. |
| `Problems` | O que não pôde ser lido no item (no máximo 10, depois quantos mais). |

### Destinatários { #recipients }

| Campo | Interpretação |
|---|---|
| `ItemID` | O ID do item. |
| `Index` | A linha do destinatário na tabela de destinatários do item, a partir de 0. |
| `RecipientType` | `To`, `Cc`, `Bcc`, `Originator` ou o valor como está armazenado (hex). |
| `Name`, `AddressType`, `Address` | O nome, o tipo de endereço e o endereço do destinatário. |
| `SMTP-Address` | O endereço SMTP, quando está armazenado além de um endereço de outro tipo. |

### Anexos { #attachments }

| Campo | Interpretação |
|---|---|
| `ItemID`, `Index` | O ID do item e a linha do anexo em sua tabela de anexos, a partir de 0. |
| `Method` | `ByValue` (um arquivo), `ByReference`, `ByReferenceResolve`, `ByReferenceOnly` (um caminho para um arquivo), `EmbeddedMessage` (um item), `OLE` (um objeto OLE), `ByWebReference` ou o número. |
| `Filename` | O nome do anexo: seu nome de arquivo longo, o curto ou seu nome de exibição. |
| `Extension` | Sua extensão como está armazenada (`.pdf`), senão a de seu nome. |
| `MimeType`, `ContentID` | Seu tipo MIME e o ID de conteúdo pelo qual um corpo HTML faz referência a ele. |
| `Inline` | Se o corpo o exibe (uma imagem do corpo HTML) em vez de listá-lo. |
| `Hidden` | Se o Outlook o oculta da lista de anexos. |
| `Size` | Seu tamanho como o repositório o informa, com suas propriedades. |
| `DataSize`, `SHA256` | O tamanho e o SHA-256 de seus dados: o arquivo, ou o arquivo composto de um objeto OLE em um PST ou OST. Vazio para um item incorporado e para um anexo por referência. |
| `Pathname` | O caminho de um anexo por referência. |
| `CreationTime`, `LastModifiedTime` | Os horários do anexo (UTC): muitas vezes os do arquivo a partir do qual ele foi criado. |
| `EmbeddedSubject` | O assunto de um item incorporado, cujas próprias linhas têm o `ItemID` `<ItemID>/<Index>`. |
| `ExtractedFile` | Com `--extract-attachments`, o arquivo, naquele diretório, em que seus dados foram gravados. |
| `Problems` | O que não pôde ser lido no anexo, ou gravado. |

### Contatos { #contacts }

| Campo | Interpretação |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | Como na tabela de itens: `Contact` ou `DistributionList`. |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | Os nomes do contato. |
| `FileUnder` | O nome sob o qual o contato está arquivado. |
| `Company`, `Title`, `Department` | Onde o contato trabalha. |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | Os três endereços de e-mail do contato, cada um com seu nome de exibição. |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | Os números de telefone do contato. |
| `InstantMessaging` | O endereço de mensagens instantâneas do contato. |
| `CreationTime`, `LastModifiedTime` | Como na tabela de itens. |

### Calendário { #calendar }

| Campo | Interpretação |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Como na tabela de itens: `Appointment` ou `MeetingMessage`. |
| `StartTime`, `EndTime` | Quando o compromisso começa e termina (UTC); os da primeira ocorrência para um compromisso recorrente. |
| `DurationMinutes` | Sua duração em minutos. |
| `Location` | Onde ele acontece. |
| `AllDay`, `Recurring` | Se é um evento de dia inteiro, e se ele é recorrente (suas exceções são itens incorporados). |
| `BusyStatus` | `Free`, `Tentative`, `Busy`, `OutOfOffice` ou `WorkingElsewhere`. |
| `ResponseStatus` | A resposta do usuário: `None`, `Organized`, `Tentative`, `Accepted`, `Declined` ou `NotResponded`. |
| `Organizer` | O nome do remetente do item. |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | Os participantes, como o item os lista. |
| `GlobalObjectID` | A identidade da reunião entre caixas de correio (hex): a mesma reunião nos calendários do organizador e dos participantes. |
| `ReminderSet`, `ReminderTime` | Se um lembrete está definido, e o horário para o qual ele está definido (UTC). |
| `CreationTime`, `LastModifiedTime` | Como na tabela de itens. |

### Tarefas { #tasks }

| Campo | Interpretação |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Como na tabela de itens: `Task` ou `TaskRequest`. |
| `Status` | `NotStarted`, `InProgress`, `Complete`, `Waiting` ou `Deferred`. |
| `PercentComplete` | Quanto dela foi concluído, de 0 a 100. |
| `StartTime`, `DueTime` | Quando ela começa e quando vence: datas no fuso horário do usuário, sem deslocamento. |
| `CompletedTime` | Quando ela foi concluída (UTC). |
| `Complete` | Se ela está concluída. |
| `Owner`, `Assigner` | Quem é seu proprietário, e quem a atribuiu. |
| `ReminderSet`, `ReminderTime` | Como na tabela de calendário. |
| `CreationTime`, `LastModifiedTime` | Como na tabela de itens. |

### Propriedades { #properties }

| Campo | Interpretação |
|---|---|
| `ItemID` | O ID do item; `store` para as propriedades do próprio repositório; `folder:<FolderID>` para as de uma pasta. |
| `Scope` | De quem é a propriedade: `Store`, `Folder`, `Item`, `Recipient` ou `Attachment`. |
| `Index` | O índice do destinatário ou do anexo; vazio para os demais. |
| `PropertyTag` | O ID e o tipo da propriedade, como `0x0037001F` (`PidTagSubject`, uma string). |
| `PropertyName` | Para uma propriedade nomeada (IDs a partir de `0x8000`), seu conjunto de propriedades e seu nome: `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords`, ou o GUID do conjunto quando ele não é um conjunto conhecido. |
| `Type` | O tipo da propriedade: `PtypString`, `PtypInteger32`, `PtypTime`, `PtypBinary`, `PtypMultipleString`, …. |
| `Value` | O valor: texto para strings, ISO 8601 em UTC para horários (que `-Z` não converte), hex para valores binários (os primeiros 4.096 bytes, depois `…(N bytes)`), os valores de uma propriedade multivalorada como uma lista (separados por `, ` no CSV), `(object)` para um item incorporado ou os dados de um anexo, e os bytes em hex com uma observação para um valor danificado ou de um tipo que não é decodificado. |

## Análise automática { #automatic-parsing }

O `auto-parse` grava as tabelas na nova categoria `Communication` como `Outlook-Files.csv`, `Outlook-Folders.csv`,
`Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`, `Outlook-Contacts.csv`,
`Outlook-Calendar.csv` e `Outlook-Tasks.csv` (um único arquivo `Outlook.json` ou `Outlook.jsonl` com `-t json` ou
`-t jsonl`), junto com `Communication/Outlook-logs.txt`. Ele passa ao parser sua `--code-page`, e não acrescenta a
tabela de propriedades nem extrai anexos: execute `parse-file outlook` para isso. A linha do tempo combinada de `-j`
inclui todas as linhas, com `Comp` e o identificador de `Artifact` `outlook`. `-Z` converte todos os campos
terminados em `Time`, exceto `StartTime` e `DueTime` de uma tarefa.

## Mais detalhes { #further-details }

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — formato de arquivo Personal Folder File (PFF)](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
