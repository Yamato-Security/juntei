# Caixas de diálogo Abrir e Salvar como (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## O que é o histórico das caixas de diálogo { #what-the-dialog-history-is }

Quando um programa exibe a caixa de diálogo comum Abrir ou Salvar como, o Windows registra no
`NTUSER.DAT` do usuário:

- **qual arquivo foi aberto ou salvo:** `OpenSavePidlMRU` tem uma subchave por extensão (`docx`,
  `ps1`, …) e `*` para todas as extensões, cada uma com uma lista dos itens usados mais recentemente;
- **qual programa usou a caixa de diálogo e a pasta em que ela estava:** `LastVisitedPidlMRU` (e
  `LastVisitedPidlMRULegacy` para programas que usam as caixas de diálogo mais antigas);
- **programas que exibiram a caixa de diálogo:** `CIDSizeMRU` (o tamanho da caixa de diálogo por programa) e
  `FirstFolder` (o caminho de um programa e uma pasta);
- no Windows 2000 e XP, `OpenSaveMRU` (caminhos por extensão) e `LastVisitedMRU` (programas e
  suas pastas como caminhos).

Desde o Windows Vista, os arquivos e pastas são listas de itens de shell, as mesmas estruturas do
destino de um atalho: cada item é uma parte do caminho (Este Computador, uma unidade, uma pasta, o arquivo), e
um item de arquivo ou pasta pode trazer seus próprios horários e a referência de arquivo NTFS.

<span class="evidence-strength">Melhor conclusão:</span> um programa usou a caixa de diálogo Abrir ou Salvar como
para abrir ou salvar este arquivo, ou nesta pasta, na sessão deste usuário. Apenas a entrada mais recente
de cada lista é datada: pelo horário da última gravação de sua chave.

## Localização { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

Colete o `NTUSER.DAT` com seus logs de transação para todos os perfis.

## Cuidados para o analista { #analyst-cautions }

- **Um horário por lista.** O horário da última gravação de uma chave muda quando uma de suas entradas passa para a
  frente da lista, por isso ele data apenas a entrada na posição 0 (`IsMostRecent`, `LastUsedUTC`).
  As outras entradas estão em ordem, da mais nova para a mais antiga ao longo da lista, mas sem data.
- **O mesmo arquivo em duas listas.** Um arquivo está na lista de sua extensão e em `*`; não o conte
  como dois usos. `*` fornece a ordem entre extensões.
- **Aberto ou salvo.** As listas não dizem qual. Correlacione com os registros do `$MFT` e do Diário
  USN do arquivo, com arquivos LNK e com Jump Lists.
- **Os horários dos itens são do arquivo, não da caixa de diálogo.** `ItemModifiedUTC`, `ItemCreatedUTC` e
  `ItemAccessedUTC` são os horários do arquivo quando o item foi gravado, com precisão de 2 segundos (FAT). Itens de
  pastas conhecidas (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …) muitas vezes não têm nenhum. Esses carimbos de data/hora FAT
  não têm deslocamento de fuso horário. O sufixo `UTC` da coluna é uma convenção de exibição, não uma
  evidência de que o relógio original estava em UTC; correlacione com o fuso horário do sistema de origem e com
  outros carimbos de data/hora antes de compará-los com os FILETIMEs do Registro. Consulte as
  [orientações sobre horários de arquivos](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times) da Microsoft.
- **Os caminhos são caminhos de shell.** Eles começam na área de trabalho: `This PC\C:\Users\…`, uma pasta conhecida, uma
  biblioteca (`UsersLibraries\Documents\…`) ou um local de rede. Um caminho que começa com uma pasta
  ou um arquivo está na área de trabalho. Um caminho vazio é a própria área de trabalho.
- **Itens não decodificados.** Um item de um tipo que o analisador de itens de shell não conhece é exibido pela sua
  classe (`[class 0x00]`), seu tipo é `unknown` e os bytes da lista ficam em `DataRaw`.
- **Os programas são nomes.** `LastVisitedPidlMRU` e `CIDSizeMRU` guardam um nome de arquivo
  (`WINWORD.EXE`), não um caminho. `FirstFolder` guarda um caminho.
- **As listas são curtas e são limpas.** Cada lista mantém um número limitado de entradas, e ferramentas de limpeza
  excluem as chaves. Use `parse-registry read --recover-deleted` para valores excluídos.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

O resumo lista as entradas por lista, os programas que usaram as caixas de diálogo e o arquivo mais
recente de cada hive (a primeira entrada de `*`).

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, o dono do perfil (a partir do caminho ou das pastas do shell), o SID a partir do `ProfileList` do sistema e o hive. |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU` ou `LastVisitedMRU`. |
| `Extension` | A subchave da extensão (`docx`, `*`); vazio para as outras listas. |
| `MRU-Position`, `IsMostRecent` | A posição na lista da chave (0 é a mais recente); vazio quando a lista não menciona o valor. |
| `ValueName` | O número (`MRUListEx`) ou a letra (`MRUList`) do valor. |
| `Program` | O programa que exibiu a caixa de diálogo. |
| `Path` | O arquivo ou a pasta: os nomes dos itens de shell unidos, ou o caminho armazenado como texto. |
| `ItemType` | O tipo do último item de shell: `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | Os horários do arquivo ou da pasta, conforme registrados pelo seu item de shell. |
| `MFT-Entry`, `MFT-Sequence` | A referência de arquivo NTFS registrada pelo item de shell. |
| `LastUsedUTC` | O horário da última gravação da chave, apenas para a entrada mais recente. |
| `KeyPath`, `KeyLastWrittenUTC` | A chave e sua última gravação: o horário de sua entrada mais recente, não de cada linha. |
| `DataRaw` | Os bytes do valor (hex) quando não são decodificados, ou a lista de itens de shell quando o tipo de um item é desconhecido. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Valores que a lista menciona, mas a chave não tem, valores que ela não menciona, tipos errados, nomes ou listas truncados. |

## Estratégia de investigação { #investigation-strategy }

1. Leia `*` em ordem, por usuário, e `LastUsedUTC` para o arquivo mais recente.
2. Procure scripts, arquivos compactados, executáveis e documentos em locais graváveis pelo usuário, removíveis ou de
   rede, e arquivos com nomes que sugiram credenciais ou exportações.
3. Use `LastVisitedPidlMRU` para vincular um programa à pasta que ele usou por último, e `CIDSizeMRU` e
   `FirstFolder` para identificar os programas que chegaram a usar as caixas de diálogo.
4. Correlacione os caminhos e o `MftEntry` com o `$MFT`, o Diário USN, arquivos LNK, Jump Lists,
   ShellBags e RecentDocs.

## Mais detalhes { #further-details }

- [libyal winreg-kb: chaves Most recently used (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: formato Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
