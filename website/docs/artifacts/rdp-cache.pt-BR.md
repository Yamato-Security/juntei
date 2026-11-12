# Cache de bitmaps do RDP { #rdp-bitmap-cache }

## O que o cache de bitmaps do RDP registra { #what-the-rdp-bitmap-cache-records }

Quando o cliente da Área de Trabalho Remota (`mstsc.exe`) se conecta com o cache persistente de bitmaps ativado, que
é o padrão, ele salva em disco pedaços da tela remota para que as sessões seguintes não precisem recebê-los de novo.
Cada pedaço é um bloco (tile) de no máximo 64 × 64 pixels. Juntos, os blocos são pequenos fragmentos do que o usuário
viu nos computadores remotos: janelas, textos, nomes de arquivos, comandos e áreas de trabalho.

| Cliente | Arquivos de cache | Blocos |
|---|---|---|
| Windows 7 e posteriores | `Cache0000.bin`, `Cache0001.bin`, …, de até 100 MB cada | 32 bits, um após o outro |
| Windows XP, Server 2003 e anteriores | `bcache2.bmc` (8 bits por pixel), `bcache22.bmc` (16), `bcache23.bmc` (24) e `bcache24.bmc` (32), de até 10 MB cada | Um por slot de 64 × 64; alguns compactados |

O Windows 10 e posteriores também deixam um `bcache24.bmc` vazio ao lado dos arquivos `.bin`.

<span class="evidence-strength">Melhor conclusão:</span> o usuário cujo perfil contém o cache se conectou, com o
cliente da Área de Trabalho Remota, a um computador que exibiu o conteúdo em cache. O cache não diz qual computador
foi, nem quando.

## Locais e coleta { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

O valor `BitmapPersistCacheLocation` de `HKCU\Software\Microsoft\Terminal Server Client` pode mudar o local do
cache. Colete a pasta `Cache` inteira de cada perfil, também das cópias de sombra de volume: uma cópia mais antiga de
um arquivo de cache pode conter blocos que o arquivo atual já não tem.

## Cuidados para o analista { #analyst-cautions }

- Os blocos não têm horários, e o cache não nomeia os computadores de onde eles vieram. Relacione-os às conexões RDP
  por meio de outras evidências: as chaves de Registro `Terminal Server Client` do cliente
  (`parse-registry remote-access`), o `Default.rdp` e outros arquivos `.rdp`, as Jump Lists do `mstsc.exe`, os
  horários dos arquivos de cache no `$MFT` e os eventos de logon dos computadores remotos.
- Um arquivo de cache mistura blocos de muitas sessões e de vários computadores remotos. O cliente substitui blocos
  antigos à medida que um arquivo se enche; assim, um bloco pode ter dias ou meses.
- Os blocos são pedaços de telas, não telas inteiras. Blocos que estavam lado a lado na tela costumam estar lado a
  lado no arquivo, mas nem sempre. Reconstrua as telas com o
  [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher), ou visualmente, a partir das colagens.
- A maioria dos blocos tem 64 × 64 pixels. Os blocos das bordas direita e inferior de uma tela são mais estreitos ou
  mais baixos, dependendo da resolução: 58 ou 62 pixels de largura, ou 28 ou 35 de altura, por exemplo. O Juntei lê
  cada bloco no seu próprio tamanho. Ferramentas que leem todos os blocos como se tivessem 64 pixels de largura
  mostram os estreitos distorcidos.
- A maioria dos blocos de cor única (`SingleColor`) é fundo. Exclua-os com um filtro para encontrar o conteúdo.
- Nos arquivos `.bmc`, os blocos de 8 bits são cores da paleta da sessão, que o cache não guarda. O Juntei usa a
  paleta padrão do Windows; por isso, as cores deles podem estar erradas. Os blocos compactados usam o RLE
  intercalado (interleaved RLE) do protocolo da Área de Trabalho Remota (MS-RDPBCGR 3.1.9), que o Juntei
  descompacta conforme a especificação o define.
- Um bloco `.bmc` menor que o seu slot de 64 × 64 deixa depois de si o final de um bloco anterior. O Juntei
  salva essas linhas restantes como uma imagem separada (`RemnantImage`). Elas pertencem a um bloco mais antigo, não
  ao bloco que está no slot.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```

```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```

Com `-d`, o Juntei lê arquivos cujo nome é `Cache` seguido de dígitos e `.bin`, e `bcache` seguido de dígitos e
`.bmc`. Um arquivo `Cache????.bin` que não começa com a assinatura `RDP8bmp` pertence a outro programa: ele é
contado no resumo e ignorado. Com `-f`, o Juntei lê o arquivo qualquer que seja o nome.

### Imagens { #images }

O `--extract` cria uma pasta para cada arquivo de cache, `<DIR>/<user>/<cache file>/`. Quando dois arquivos de
cache ficariam na mesma pasta, como um arquivo de cache e a sua cópia vinda de uma cópia de sombra de volume, a
segunda pasta é numerada (`Cache0001.bin_2`). A pasta contém:

- `<cache file>_<index>.bmp`: cada bloco, no seu próprio tamanho. Os nomes são os mesmos que o bmc-tools usa, de
  modo que o RdpCacheStitcher consegue abrir a pasta.
- `<cache file>_old_<index>.bmp`: apenas `.bmc`; o final de um bloco anterior que ficou depois de um bloco
  menor.
- `<cache file>_collage.png`: todos os blocos na ordem do arquivo, 64 por linha (`--collage-width`), cada um em uma
  célula de 64 × 64 a partir do canto superior esquerdo. O bloco de índice *i* fica na linha *i* / 64 e na coluna
  *i* % 64, contadas a partir de 0. A cor magenta preenche as partes vazias das células: ao lado de um bloco
  estreito, abaixo de um bloco baixo e onde um bloco não pôde ser lido.

O `--collage-only` salva apenas as colagens. Uma imagem que já existe não é substituída sem `-C`, e o `Problems` do
bloco correspondente informa isso. Os próprios arquivos de cache nunca são substituídos, nem mesmo com `-C`.

## Campos de saída CSV { #csv-output-fields }

| Campo | Significado |
|---|---|
| `SourceFile` | O arquivo de cache. |
| `User` | O usuário em cuja pasta de perfil o arquivo de cache está, obtido do caminho. |
| `TileIndex` | O número do bloco no arquivo, contado a partir de 0, como no nome da sua imagem. |
| `Width`, `Height` | O tamanho do bloco em pixels. |
| `SingleColor` | `true` quando todos os pixels do bloco têm a mesma cor. |
| `TileImage` | Com `--extract`: o arquivo BMP do bloco. |
| `Collage` | Com `--extract`: a colagem do arquivo de cache. |
| `RemnantImage` | `.bmc` com `--extract`: o final de um bloco anterior que ficou no slot. |
| `Key` | A chave de 64 bits do bloco no protocolo da Área de Trabalho Remota, em hexadecimal. |
| `Format`, `FormatVersion` | `bin`, com a versão que vem após a assinatura (6 no Windows 10 e posteriores), ou `bmc`. |
| `BitsPerPixel` | 32 para `.bin`; 8, 16, 24 ou 32 para `.bmc`. |
| `Compressed`, `Flags` | `.bmc`: se o bloco estava compactado e os flags do seu cabeçalho. |
| `Offset` | Onde o cabeçalho do bloco começa no arquivo. |
| `Problems` | Danos: um arquivo que termina no meio de um bloco, um cabeçalho de bloco que nenhum bloco poderia ter, dados que não cabem no tamanho do bloco, dados compactados que não podem ser descompactados, uma imagem que não pôde ser salva. |

JSON e JSONL têm os mesmos campos, com números e booleanos tipados, `null` para valores que um bloco não tem e
`Problems` como uma lista. Danos fora de qualquer bloco, como dados após o último bloco, são relatados nas mensagens
da execução.

## Análise automática { #automatic-parsing }

O `auto-parse` grava `LateralMovement/RdpCache.csv` (`RdpCache.json` ou `RdpCache.jsonl` com
`-t json` ou `-t jsonl`), junto com `LateralMovement/RdpCache-logs.txt`, e salva a colagem de cada arquivo de cache
em uma pasta `LateralMovement/RdpCache/`. Adicione `--extract-rdp-tiles` para salvar também todos os blocos, o que
pode chegar a dezenas de milhares de arquivos para um único usuário. A partir de imagens de disco, os arquivos de
cache são copiados. A linha do tempo combinada de `-j` inclui todas as linhas, com o identificador de `Artifact`
`rdp-cache`. Os blocos não têm horários; por isso, o [`super-timeline`](../commands/super-timeline.md) não gera
eventos a partir deles.

## Estratégia de investigação { #investigation-strategy }

1. Abra as colagens e procure textos: nomes de computadores e de usuários, caminhos de arquivos, comandos, janelas
   de console, barras de endereço de navegadores e caixas de diálogo.
2. Reconstrua as telas com o RdpCacheStitcher a partir de uma pasta de blocos salvos com `--extract`.
3. Descubra a quais computadores o usuário se conectou, a partir do Registro do cliente (`parse-registry
   remote-access`), dos arquivos `.rdp` e das Jump Lists do `mstsc.exe`. Nesses computadores, procure os logons RDP
   correspondentes e as entradas do Registro de acesso do usuário (UAL).
4. Use os horários do `$MFT` dos arquivos de cache, e os das cópias deles nas cópias de sombra de volume, para
   restringir quando os blocos foram gravados.

## Mais detalhes { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: o cache de bitmaps do RDP (em francês)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — RDP cache plugin](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — persistent bitmap cache](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
