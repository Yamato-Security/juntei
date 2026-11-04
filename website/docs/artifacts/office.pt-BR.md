# Microsoft Office (arquivos recentes, documentos confiáveis, configurações de macro) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## O que o Office registra { #what-office-records }

O Microsoft Office mantém, por usuário em `NTUSER.DAT` e por versão e aplicativo:

- **Arquivos e locais recentes:** `File MRU`, os documentos que um aplicativo abriu, e `Place MRU`,
  as pastas a partir das quais os abriu, cada item com o horário em que foi aberto pela última vez. O Microsoft 365
  também mantém uma lista por conta conectada (`User MRU\<account>`).
- **Documentos confiáveis:** quando um usuário clica em *Habilitar Edição* (*Enable Editing*) ou *Habilitar Conteúdo* (*Enable Content*) em um documento,
  o Office o guarda em `TrustRecords`: o caminho do documento, quando ele foi considerado confiável e se
  suas macros foram habilitadas.
- **Configurações de macro:** `VBAWarnings` (quais macros são executadas), `BlockContentExecutionFromInternet`
  (se as macros em arquivos vindos da Internet são bloqueadas), `AccessVBOM` (se programas podem
  alterar projetos VBA) e os **Locais Confiáveis**, pastas cujos documentos têm as macros executadas sem
  perguntar. A Política de Grupo pode definir o mesmo em `Software\Policies`.

Um documento malicioso geralmente precisa que o usuário habilite suas macros: um registro de confiança cujas macros
foram habilitadas vincula o documento, o usuário e um horário.

<span class="evidence-strength">Melhor conclusão:</span> o usuário abriu este documento neste
aplicativo (e pela última vez neste horário); o usuário habilitou a edição ou as macros deste documento neste
horário; o Office estava configurado para executar macros desta forma.

## Localização { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

Versões: `16.0` é o Office 2016 e posteriores (2019, 2021, 2024, Microsoft 365), `15.0` o Office 2013,
`14.0` o Office 2010, `12.0` o Office 2007.

Um item tem a forma `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx`: `T`
é o horário da última abertura (um FILETIME em hexadecimal), e o caminho vem depois de `*`. Um registro de confiança tem 24
bytes: o FILETIME em que o documento foi considerado confiável e, em seguida, seus flags nos últimos 4 bytes, `0x7FFFFFFF`
quando suas macros foram habilitadas e `0x00000001` quando apenas sua edição foi.

## Cuidados para o analista { #analyst-cautions }

- **Horários.** `LastOpenedUTC` é o horário próprio de cada item. `TrustedAtUTC` é quando o usuário considerou
  o documento confiável, o momento em que *Habilitar Edição* ou *Habilitar Conteúdo* foi clicado, o que pode ser anterior
  à abertura mais recente.
- **Macros habilitadas, não executadas.** `MacrosEnabled` indica que o usuário habilitou as macros do documento; isso
  não prova que uma macro foi executada nem o que ela fez. Correlacione com a criação de processos, o Prefetch, os logs do PowerShell
  e o próprio documento.
- **Caminhos como o Office os armazena.** Os registros de confiança identificam o documento por um caminho que pode usar
  `%USERPROFILE%`, barras normais, uma URL `file:///` e codificação de URL (`%20`). Os itens MRU de
  documentos na nuvem são URLs.
- **Padrões e política efetiva.** O Office cria locais confiáveis padrão (`Templates`,
  `XLSTART`, `STARTUP`, …). A `Date` de um local é uma pista útil, não uma prova de quem o criou:
  administradores, scripts, políticas e configurações de Registro copiadas também podem preenchê-lo. A ausência de
  `VBAWarnings`, por si só, não estabelece a política de macros efetiva. A versão do Office, a marcação de arquivos
  vindos da Internet e outras configurações de gerenciamento também importam.
  [A Microsoft explica o bloqueio de macros da Internet](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked).
- **Documentos excluídos.** As listas sobrevivem aos documentos; um arquivo ausente ainda pode aparecer
  aqui. As listas são curtas e são limpas pela opção *Limpar itens não fixados* (*Clear unpinned items*) do Office e por ferramentas de limpeza.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

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
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

O resumo lista os documentos cujas macros foram habilitadas, as configurações que permitem que as macros sejam executadas com mais
liberdade (`VBAWarnings` 1, `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1,
`AllowNetworkLocations` 1), os locais confiáveis adicionados por usuários ou definidos por política, e os documentos
abertos por último.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, o dono do perfil (a partir do caminho ou das pastas do shell), o SID a partir do `ProfileList` do sistema e o hive. |
| `OfficeVersion`, `App` | A chave da versão (`16.0`) e o aplicativo (`Word`, `Excel`, …). |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security` ou `Trusted Locations`. |
| `Account` | A conta do Microsoft 365 de uma lista `User MRU`. |
| `Policy` | Se a configuração ou o local é definido por política. |
| `MRU-Position`, `IsMostRecent` | A posição de um item em sua lista (`Item 1` é 0, o mais recente). |
| `Path` | O documento ou a pasta; o caminho de um documento confiável conforme armazenado; o `Path` de um local confiável. |
| `LastOpenedUTC` | Quando o item foi aberto pela última vez. |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | Quando o documento foi considerado confiável, se suas macros foram habilitadas e os flags do registro. Flags desconhecidos deixam `MacrosEnabled` vazio e geram um diagnóstico. |
| `Setting`, `Value` | Uma configuração de macro e seu valor com o respectivo significado (`1 (all macros enabled)`); a chave de um local confiável (`Location3`) e seus outros valores (`AllowSubFolders=1; Date=…; Description=…`). |
| `KeyPath`, `KeyLastWrittenUTC` | A chave e sua última gravação. |
| `DataRaw` | Os bytes do valor (hex) quando não são decodificados. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Itens sem `*` ou com um horário inválido, registros de confiança que não têm 24 bytes, locais sem `Path`, tipos errados. |

## Estratégia de investigação { #investigation-strategy }

1. Comece pelos registros com `MacrosEnabled` verdadeiro: quais documentos, quando e por quem. Encontre os arquivos (`$MFT`,
   Diário USN, `Zone.Identifier`), sua origem (histórico do navegador e de e-mail) e o que foi executado em seguida.
2. Verifique as configurações de macro e os locais confiáveis: todas as macros habilitadas, macros da Internet não
   bloqueadas, acesso a projetos VBA ou um local adicionado pelo usuário (em `Downloads`, `Temp`, um compartilhamento de rede)
   enfraquecem os padrões.
3. Coloque `LastOpenedUTC` na linha do tempo com arquivos LNK, Jump Lists e RecentDocs para os
   caminhos dos documentos, incluindo locais removíveis e de rede.

## Mais detalhes { #further-details }

- [Microsoft: Habilitar ou desabilitar macros em arquivos do Microsoft 365](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: Documentos confiáveis](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: Adicionar, remover ou alterar um local confiável](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
