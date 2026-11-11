# Jump Lists { #jump-lists }

## O que são Jump Lists { #what-jump-lists-are }

As Jump Lists sustentam os menus exibidos a partir das entradas de aplicativos na barra de tarefas e no menu Iniciar. As Jump
Lists automáticas são históricos gerenciados pelo Windows dos itens usados por um aplicativo. As Jump Lists personalizadas são
categorias e tarefas definidas pelo aplicativo. Ambas podem incorporar estruturas LNK com caminhos de destino, horários,
contexto de volume/rede, referências MFT e argumentos de inicialização.

Os registros automáticos podem revelar arquivos ou pastas abertos por um aplicativo, a ordem/o horário do último uso, a
contagem de interações e o estado de fixação. As listas personalizadas podem expor páginas “fechadas recentemente” ou tarefas
do aplicativo — incluindo argumentos de comando maliciosos.

<span class="evidence-strength">Melhor conclusão:</span> o aplicativo proprietário registrou ou expôs
o item/a tarefa em sua Jump List. O comportamento varia conforme o aplicativo; corrobore a execução/o acesso efetivo.

## Locais e coleta { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

O prefixo hexadecimal do nome do arquivo é um AppID. Colete ambos os diretórios e preserve o caminho do perfil
do usuário. As listas automáticas são contêineres OLE Compound File com um fluxo `DestList` e fluxos LNK
numerados; as listas personalizadas usam seu próprio formato de categorias/links.

## Cuidados para o analista { #analyst-cautions }

- As descrições de AppID vêm de um mapeamento conhecido; um AppID desconhecido não é suspeito por si só.
- Os campos `LastModified` / de último uso pertencem a uma entrada da DestList; os horários do destino incorporado são diferentes.
- As contagens de interações existem apenas em versões compatíveis e não são uma contagem completa de aberturas.
- A fixação preserva itens além da recência normal; limpezas ou configurações do aplicativo podem remover outros.
- Uma lista pode conter fluxos obsoletos/desvinculados ou bytes de uma lista personalizada anterior e mais longa.
- Os argumentos e caminhos de LNK incorporados são controlados pelo atacante. A associação com o aplicativo é contexto, não confiança.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

Com `auto-parse`, as tabelas CSV são `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`
e `FileAndFolderOpening/JumpLists-CustomDestinations.csv`, diretamente sob cada computador.
O log compartilhado é `JumpLists-logs.txt` nessa categoria. Os manifestos listam os caminhos exatos
em `OutputFiles`. As exportações CSV independentes mantêm `AutomaticDestinations.csv` e
`CustomDestinations.csv` no diretório que você escolher. As exportações automáticas em JSON e JSONL são
arquivos únicos `JumpLists.json` ou `JumpLists.jsonl`; a estrutura dos registros não muda.

## Campos CSV de AutomaticDestinations { #automaticdestinations-csv-fields }

| Grupo de campos | Campos e interpretação |
|---|---|
| Origem | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: o contêiner da Jump List e seus horários do sistema de arquivos. |
| Aplicativo | `AppId`, `AppIdDescription`: AppID do nome do arquivo e mapeamento de aplicativo conhecido. Não infira a identidade a partir de um hash desconhecido. |
| Contêiner | `HasSps`, `DestListVersion`, `LastUsedEntryNumber`: estrutura/versão da DestList e contexto do cabeçalho. |
| Ordenação | `MRU`, `EntryNumber`: posição de uso mais recente e identificador hexadecimal da entrada/do fluxo. |
| Metadados de uso | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition`: ciclo de vida/uso/estado de fixação da DestList. Interprete conforme a versão. |
| Host/caminho | `Hostname`, `MacAddress`, `Path`, `DisplayPath`: local do destino e contexto de rastreamento. Host/MAC não é atribuição isolada. |
| IDs de rastreamento | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid`: correlacione destinos movidos e o rastreamento do LNK incorporado. |
| Notas | `Notes`: informa sobre fluxos desvinculados, sobras recuperadas ou outro contexto do analisador. |
| LNK incorporado | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Args`: interprete conforme descrito em [Atalhos LNK](lnk.md). |
| LNK adicional | `NetworkPath`, `Name`, `IconLocation`: campos de rede e de apresentação do atalho incorporado. |
| Validação | `Problems`: fluxos ausentes, truncamento, divergência de contagens ou danos no link. |

## Campos CSV de CustomDestinations { #customdestinations-csv-fields }

A saída personalizada começa com `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription` e `EntryName` (nome da categoria ou `Tasks`), seguidos de todos os campos de LNK incorporado
listados acima, e então `Notes`, `NetworkPath`, `Name`, `IconLocation` e `Problems`.

O JSON/JSONL preserva adicionalmente o cabeçalho da DestList, a pontuação/soma de verificação/repositório de propriedades de cada entrada,
`DestListPropertyStore`, os fluxos do arquivo composto, as categorias personalizadas, os bytes após a última categoria
e os campos aninhados de cada LNK incorporado.

## Estratégia de investigação { #investigation-strategy }

1. Identifique o usuário e o perfil de origem e, em seguida, associe o AppID ao aplicativo com cautela.
2. Ordene as entradas automáticas por `LastModified` e `MRU`; separe os itens fixados.
3. Revise caminhos, compartilhamentos de rede, números de série de volumes removíveis, referências MFT e argumentos.
4. Inspecione `Notes` e links desvinculados/remanescentes — itens excluídos ou anteriores podem sobreviver ali.
5. Correlacione com RecentDocs, LNKs independentes da pasta Recent, Prefetch, SRUM, `$MFT` e logs de eventos.

## Mais detalhes { #further-details }

- [Microsoft: formato Compound Binary File (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: formato Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [Projeto de teste/referência de analisador de JumpList](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
