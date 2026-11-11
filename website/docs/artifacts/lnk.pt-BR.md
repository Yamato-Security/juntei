# Atalhos LNK { #lnk-shortcuts }

## O que são arquivos LNK { #what-lnk-files-are }

Os arquivos Windows Shell Link (`.lnk`) descrevem um destino, além de argumentos de inicialização, diretório de trabalho, ícone,
estado da janela, metadados do destino, informações de volume/rede, itens do namespace do shell e
identificadores opcionais de rastreamento de links distribuídos. O Windows e os aplicativos os criam em Itens recentes,
Área de Trabalho, Menu Iniciar e outros locais; atacantes também entregam atalhos maliciosos diretamente.

Um LNK da pasta Recent pode sustentar a interação do usuário com um destino que não existe mais, incluindo uma
unidade removível ou um compartilhamento de rede. Um LNK arbitrário pode existir sem nunca ter sido clicado.

<span class="evidence-strength">Melhor conclusão:</span> o atalho foi criado/atualizado com o
contexto de destino registrado. Sua localização e os artefatos ao redor determinam se ele sustenta o uso.

## Locais e coleta { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

Pesquise todo o perfil do usuário e os arquivos compactados/conteúdos de e-mail coletados, não apenas `Recent`. Preserve
os horários e o caminho do próprio arquivo de origem, pois a localização altera a interpretação.

## Cuidados para o analista { #analyst-cautions }

- Os horários, o tamanho, a referência MFT e os dados de volume do destino são instantâneos armazenados no LNK; eles não são
  os metadados do próprio arquivo de atalho e podem estar desatualizados.
- `MachineID`, o MAC e o horário de criação do rastreador descrevem o contexto de rastreamento de links distribuídos, normalmente
  a máquina que criou/atualizou o link — não necessariamente a máquina onde você o encontrou.
- Um nó UUID semelhante a um MAC pode vir de virtualização, randomização ou identificadores não relacionados à rede.
- `Args` pode tornar perigoso um executável benigno (`powershell.exe`, `rundll32.exe`, `mshta.exe`).
- Strings ANSI exigem a página de código do sistema de origem; use `--code-page 932` para muitos links em japonês.
- Um atalho pode ser forjado ou copiado. Não equipare seus campos a execução sem o apoio de Prefetch,
  Jump List, RecentDocs, log de eventos ou outras evidências.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## Campos de saída CSV { #csv-output-fields }

| Grupo de campos | Campos e interpretação |
|---|---|
| Origem | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: caminho do atalho e horários do sistema de arquivos hospedeiro. São distintos dos horários do destino. |
| Horários do destino | `TargetCreated`, `TargetModified`, `TargetAccessed`: instantâneos registrados no LNK; pistas históricas úteis, não metadados atuais do destino. |
| Dados básicos do destino | `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`: contexto registrado do destino/inicialização. Os argumentos podem mudar o significado. |
| Volume | `DriveType`, `VolumeSerialNumber`, `VolumeLabel`: identificam o contexto fixo/removível/de rede e permitem correlacionar outros atalhos. |
| Caminhos | `LocalPath`, `NetworkPath`, `CommonPath`, `TargetIDAbsolutePath`: representações complementares do caminho. Revise divergências e caminhos do namespace do shell. |
| Comportamento de inicialização | `Args`, `ShowWindow`, `HotKey`, `ShimLayer`, `DarwinId`: como o shell inicia ou configura o destino. `Args` é fundamental. |
| Identidade NTFS | `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`: referência do destino no momento da criação do link; compare ambos com o `$MFT`. |
| Rastreamento | `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `VolumeDroid`, `FileDroid`, `VolumeDroidBirth`, `FileDroidBirth`: rastreamento de links distribuídos. Use para correlação, não como atribuição isolada. |
| Blocos extras | `ExtraBlocksPresent`: lista das estruturas opcionais disponíveis; explica quais outros campos são sustentados. |
| Apresentação | `Name`, `IconLocation`, `IconIndex`, `IconEnvironment`, `EnvironmentVariables`: dados de exibição e do destino com variáveis de ambiente expandidas. Atacantes podem usar nomes/ícones enganosos. |
| Pastas do shell | `KnownFolder`, `SpecialFolder`: identificadores de pastas do shell do Windows decodificados. |
| Rede | `NetworkDevice`, `NetworkProvider`: contexto do compartilhamento/dispositivo de rede e do provedor. |
| Codificação | `ConsoleCodePage`: página de código do console registrada, quando presente; distinta do `--code-page` do analisador. |
| Dados finais | `AppendedBytes`: bytes após o fim lógico. Dados inesperados podem indicar contrabando de conteúdo ou arquivos malformados. |
| Validação | `Problems`: danos, truncamento, blocos inconsistentes ou limitações de decodificação. Inclua na revisão. |

O JSON/JSONL preserva adicionalmente cada item de ID de destino (tipo, nome, GUID, horários FAT e referência
MFT), todos os blocos de dados extras, propriedades do repositório de propriedades, configurações do console e as relações
estruturais brutas. Use-o para um atalho suspeito em vez de depender apenas da linha CSV plana.
Os horários aninhados de `TargetItems` vêm de relógios FAT de itens do shell sem deslocamento armazenado.
Sua representação legada com `Z` não comprova UTC; `auto-parse --timezone` os preserva.
Os FILETIMEs do destino no cabeçalho do LNK e os horários do rastreador continuam sendo carimbos de data/hora absolutos conversíveis.

## Estratégia de investigação { #investigation-strategy }

1. Examine a localização e os horários de origem antes de interpretar os campos do destino.
2. Reconstrua a inicialização efetiva a partir do caminho de destino, `Args`, `WorkingDirectory` e dos blocos de ambiente.
3. Use a referência MFT do destino, o número de série do volume, o caminho de rede, os droids, o MachineID e o MAC como pivôs entre as evidências.
4. Verifique Prefetch e os logs de eventos para execução; verifique Jump Lists/RecentDocs para interação do usuário.
5. Para anexos maliciosos, preserve os dados anexados e calcule o hash/analise o próprio LNK.

## Mais detalhes { #further-details }

- [Microsoft Open Specifications: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
