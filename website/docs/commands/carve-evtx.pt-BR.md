# Carving de logs de eventos { #carve-evtx }

Recupere chunks de logs de eventos que restam depois que um log foi limpo ou excluído: de uma imagem de disco, do espaço não alocado, de um arquivo de paginação ou de uma imagem de memória. Cada chunk EVTX de 64 KiB traz suas próprias tabelas de strings e de templates, de modo que seus registros podem ser lidos sem o restante do log. O Juntei grava os chunks recuperados como arquivos `.evtx` comuns, um por canal, para abrir no Hayabusa ou no Visualizador de Eventos.

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
```

Sem `-o`, apenas o resumo é exibido.

## Busca de chunks { #finding-chunks }

- A assinatura de chunk é procurada em cada deslocamento de byte, e cada cabeçalho precisa ser plausível (tamanho do cabeçalho, números de registro, deslocamentos). As somas de verificação são informadas, mas não exigidas: um chunk parcialmente sobrescrito ainda contém registros legíveis.
- Os chunks de logs com compactação NTFS são descompactados (LZNT1, unidades de compactação de 16 clusters). Em uma imagem de disco, é usado o tamanho de cluster do volume; para outros arquivos, `--cluster-size <BYTES>` (padrão: 4096).
- Os registros são percorridos pelo tamanho até o deslocamento do último registro indicado no cabeçalho, e um ID de registro repetido nunca interrompe o percurso. Depois de um registro danificado, o percurso continua no próximo registro válido. Registros antigos no slack do chunk também são listados.

## Imagens de disco e volumes NTFS { #disk-images }

Com uma imagem E01 ou VHDX, ou um disco ou volume bruto com NTFS, cada chunk é classificado pelo cluster em que está:

| Local | Significado |
|---|---|
| `FreeSpace` | Um cluster que nenhum arquivo usa. |
| `DeletedLog` | Um cluster livre que o registro do `$MFT` de um log excluído ainda lista, ou um log reconstruído a partir desses clusters. |
| `LiveLog` | Um cluster de um log de eventos em uso. Omitido, a menos que se use `--keep-live`. |
| `AllocatedFile` | Um cluster de outro arquivo, como `pagefile.sys` ou o arquivo de armazenamento de uma cópia de sombra. |
| `AllocatedUnknown` | Um cluster alocado que nenhum registro do `$MFT` lista. |
| `OutsideVolumes` | Fora de todos os volumes NTFS do disco. |
| `Unclassified` | A entrada não tem volume NTFS (um arquivo de paginação ou uma imagem de memória). |

Logs excluídos cujo registro do `$MFT` lista apenas clusters livres são reconstruídos por inteiro, inclusive os compactados, e salvos em `Recovered/`. `--free-space-only` procura apenas em clusters livres: é mais rápido, mas ignora o arquivo de paginação, as cópias de sombra e outros arquivos.

## Deduplicação { #deduplication }

Cada chunk é gravado uma única vez. Um chunk é omitido quando pertence a um log ativo, quando é idêntico a outro chunk ou quando todos os seus registros já estão nos logs ativos ou em um chunk já gravado. Chunks com mais registros são considerados primeiro, de modo que uma cópia mais antiga e mais curta de um chunk não acrescenta nada. Para uma entrada sem NTFS, informe os logs ativos com `--live-logs <DIR>`.

## Resultados { #results }

- `Carved/<CHANNEL>.evtx`: Um log por canal, com o nome que o Windows dá aos arquivos de log (`Microsoft-Windows-Sysmon%4Operational.evtx`). Os números de registro nos cabeçalhos dos chunks são renumerados e as somas de verificação, recalculadas. Um registro danificado e tudo o que vem depois dele viram slack. Fora isso, cada chunk é mantido inteiro, inclusive o slack, para que a recuperação de registros, como o `-x` do Hayabusa, continue funcionando nele. A opção `-x` do Hayabusa pesquisa os blocos marcados como vazios. Consulte o `records.csv` do Juntei para localizar registros no espaço residual de blocos parcialmente ocupados.
- `Recovered/Volume<N>/...`: Logs excluídos reconstruídos a partir de seus registros do `$MFT`.
- `chunks.csv` / `chunks.jsonl`: Todos os chunks encontrados: onde estavam, o que foi feito deles (`Status`), seu canal, IDs de registro, carimbos de data/hora e estado da soma de verificação.
- `records.csv` / `records.jsonl`: Todos os registros dos chunks gravados. `OnlyInCarvedData` marca os registros que os logs ativos não contêm.

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

O carving encontra apenas chunks contíguos. Um chunk dividido entre clusters não contíguos, ou entre páginas físicas não contíguas na memória, fica incompleto, e seus registros finais podem estar danificados ou ausentes. Descompacte o `hiberfil.sys` antes. O código de saída é 0 quando a busca foi concluída e 1 em caso de erro.

## Mais detalhes { #further-details }

- [Uso detalhado e referência de campos (inglês)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.md)
- [Uso detalhado e referência de campos (japonês)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.ja.md)
- [Formato EVTX (documentação do libevtx)](https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc)
