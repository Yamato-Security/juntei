# ShellBags { #shellbags }

## O que são ShellBags { #what-shellbags-are }

O Explorer lembra como cada pasta que um usuário abre é exibida: seu modo de exibição, o tamanho dos ícones, a ordem de classificação e
a posição da janela. Para encontrar novamente as configurações de uma pasta, ele guarda a identidade da pasta no Registro
do usuário como uma árvore de itens do shell, os mesmos itens que a lista de IDs de destino de um link contém: uma unidade e, em seguida, cada
pasta até a que foi aberta. Essas árvores são as ShellBags.

Elas abrangem pastas abertas no Explorer e nas caixas de diálogo de abrir e salvar, incluindo pastas em unidades
removíveis, compartilhamentos de rede, dispositivos portáteis (telefones, câmeras), arquivos ZIP e servidores FTP, e
páginas do Painel de Controle. Elas permanecem depois que a pasta, a unidade ou o compartilhamento de rede deixa de existir.

<span class="evidence-strength">Melhor conclusão:</span> o Explorer deste usuário exibiu esta pasta
(ou uma pasta abaixo dela). Uma ShellBag não mostra o que foi feito na pasta, e não data
cada visita: veja os horários abaixo.

## Localização { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

Uma chave `BagMRU` contém valores numerados, cada um sendo um item do shell, e um valor `MRUListEx` que indica sua
ordem, o aberto mais recentemente primeiro. Cada valor tem uma subchave de mesmo nome para os itens
abaixo dele, de modo que um caminho são os itens de `BagMRU` até um valor. O `NodeSlot` de uma subchave é o
número de sua chave `Bags\<N>`, onde estão as configurações de exibição da pasta.

Colete `UsrClass.dat` e `NTUSER.DAT` com seus logs de transação (`.LOG1`, `.LOG2`) para todos os
perfis, e o hive `SOFTWARE` do sistema, que fornece o SID do usuário de um `NTUSER.DAT`.

## O que cada horário indica { #which-times-say-what }

- **Os horários do item do shell** (`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`) são os
  horários do próprio sistema de arquivos da pasta (precisão FAT: 2 segundos) quando o Explorer gravou o item. Eles
  datam a pasta, não a visita, e não são atualizados em visitas posteriores.
  Os relógios FAT não têm deslocamento UTC armazenado. Os títulos legados `UTC` e o sufixo `Z` exibido são
  convenções de exibição, não prova do fuso horário de origem. `auto-parse --timezone` preserva
  esses campos; interprete-os com o contexto do sistema de origem.
- **O horário da última gravação de uma chave** muda quando seu `MRUListEx` é regravado, ou seja, quando um de seus
  itens é aberto e movido para a frente. Assim, o horário da chave em que está o valor de um item
  (`ParentKeyLastWrittenUTC`) é quando o item foi aberto pela última vez **somente quando ele é o item mais recente
  dessa chave** (`IsMostRecentInParent` verdadeiro, `MruPosition` 0). Para os demais itens, é um
  horário posterior, de quando um item irmão foi aberto.
- A própria chave de um item (`KeyLastWrittenUTC`) data da mesma forma a abertura de seu filho mais recente.

## Cuidados para o analista { #analyst-cautions }

- **A ausência prova pouco.** O Explorer grava ShellBags quando o modo de exibição de uma pasta é salvo; uma pasta
  vista apenas brevemente, ou aberta de outra forma (uma linha de comando, um script, um programa), pode não ter nenhuma.
  Um usuário pode excluir as chaves.
- **Uma referência NTFS é a da pasta** (`MftEntry`, `MftSequence`) no momento em que o item foi gravado.
  Procure-a no `$MFT` para ver se a mesma pasta ainda existe, ou se outro arquivo reutilizou a
  entrada.
- **As unidades removíveis** são nomeadas por sua letra de unidade. A letra é a que a unidade tinha na época;
  associe-a ao histórico de USB e de dispositivos montados para identificar o dispositivo.
- **Os nomes vêm do item.** Uma pasta renomeada depois mantém seu nome antigo aqui. As pastas raiz e as pastas
  conhecidas são nomeadas como as ferramentas forenses costumam nomeá-las (`This PC`, `Quick Access`, `Downloads`);
  `Guid` fornece seu identificador.
- **Resultados de pesquisa, bibliotecas e páginas do Painel de Controle** são modos de exibição, não pastas em disco
  (`property_view`, `control_panel_item`, `control_panel_category`).
- **Um item que nenhum analisador conhece** mantém seus bytes em `DataRaw`, com `ItemType` `unknown` e seu
  byte de classe em `ItemClass`. Itens danificados ou incomuns são relatados em `Problems`, e seus bytes
  também são mantidos.
- **Um hive sujo** pode não ter seus itens mais recentes sem seus logs. O Juntei reproduz os logs ao lado do
  hive, a menos que `--no-logs` seja usado; `TransactionLogStatus` indica se um registro veio deles.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

Os itens do shell são decodificados com o ys-lnk-parser, o analisador de itens do shell do `parse-file lnk`. O
resumo lista as pastas abertas por último (as mais recentes primeiro), a partir do horário da última gravação da chave pai,
com seus usuários.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | Host, o nome da pasta do perfil, o SID do usuário (a chave raiz de um `UsrClass.dat` o nomeia; para `NTUSER.DAT`, o `ProfileList` do sistema), o hive e seu tipo. |
| `BagKey` | A chave do item, sob a raiz do hive (`…\BagMRU\4\3\0`): seu valor está na chave acima dela. |
| `MRU-Position`, `IsMostRecentInParent` | Sua posição no `MRUListEx` da chave pai (0: o aberto mais recentemente); vazio quando a lista não o contém. |
| `NodeSlot` | O número de sua chave `Bags\<N>` (as configurações de exibição da pasta). |
| `Path` | Os itens do topo de sua árvore até ele, unidos com `\`. `[unknown]` representa um item cujo valor está ausente. |
| `ItemType`, `ItemClass` | O que o item é (`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`) e seu byte de classe. |
| `ItemName`, `GUID` | Seu nome (o nome longo de uma pasta) e o identificador de classe de uma pasta raiz, item do Painel de Controle ou pasta delegada. |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **Do item do shell:** os horários da pasta quando o item foi gravado (FAT, precisão de 2 segundos). |
| `MFT-Entry`, `MFT-Sequence` | **Do item do shell:** a referência de arquivo NTFS da pasta. |
| `ExtensionBlocks` | Quantos blocos de extensão o item tem. |
| `KeyLastWrittenUTC` | **Do Registro:** a última gravação da própria chave do item: quando seu filho mais recente foi aberto. |
| `ParentKeyLastWrittenUTC` | **Do Registro:** a última gravação da chave em que está seu valor: quando ele foi aberto pela última vez, se `IsMostRecentInParent` for verdadeiro. |
| `DataRaw` | Os bytes do valor (hex), para um item que não é totalmente compreendido ou que tem problemas. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Um valor que o `MRUListEx` de sua chave nomeia mas a chave não tem, um valor que a lista não nomeia, uma subchave sem seu valor, um valor que não é binário, um `NodeSlot` do tipo errado, uma lista de itens do shell danificada. |

## Estratégia de investigação { #investigation-strategy }

1. Liste as unidades removíveis (`volume` com o `Guid` de Unidades Removíveis), os locais de rede, as pastas
   FTP e os dispositivos portáteis, e as pastas abaixo deles.
2. Procure pastas de interesse: locais de preparação graváveis pelo usuário (`Temp`, `ProgramData`, `Public`),
   pastas de ferramentas e de arquivos compactados, perfis de outros usuários e pastas que não existem mais (verifique a
   referência NTFS no `$MFT`).
3. Date as visitas somente com o `ParentKeyLastWrittenUTC` dos itens em que `IsMostRecentInParent` é verdadeiro;
   correlacione o restante com arquivos LNK, Jump Lists e RecentDocs, que datam aberturas de arquivos.
4. Use `parse-registry read --recover-deleted` para chaves e valores `BagMRU` excluídos.

## Mais detalhes { #further-details }

- [libyal libfwsi: formato Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

A busca do SID a partir de `ProfileList` exige um SID único e sintaticamente válido. Mapeamentos ambíguos de pastas de perfil e chaves de backup como `S-1-...bak` deixam `UserSid` desconhecido.
