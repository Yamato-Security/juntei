# Montar imagens de disco { #mounting-disk-images }

`juntei mount` disponibiliza os arquivos de uma imagem de disco VHDX / E01 / Ex01 em um diretório, somente para leitura, durante sua execução. Ferramentas como Hayabusa e Juntei podem analisá-los sem extração nem privilégios de root. Usa FUSE e funciona no Linux. O leitor e a biblioteca de montagem são mantidos em [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

Em outro terminal:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

Pressione Ctrl+C no primeiro terminal para desmontar ou execute `fusermount3 -u /mnt/host1`.

## O que a montagem mostra { #what-the-mount-shows }

- Os arquivos do volume NTFS aparecem no ponto de montagem. Uma imagem coletada pode conter diretórios como `C` e `LongFileNames`, além de metadados do volume como `$MFT`. Vários volumes NTFS aparecem como `Volume1`, `Volume2` etc.
- Todo arquivo é somente leitura e pertence ao proprietário do ponto de montagem. Os carimbos de data/hora dos
  arquivos são os de `$STANDARD_INFORMATION` do NTFS.
- Os nomes são listados como o NTFS os armazena e são procurados de forma exata. Nomes DOS 8.3 e fluxos de dados
  alternativos não são mostrados.
- Arquivos compactados são descompactados (até 1 GiB cada). Arquivos criptografados com EFS não podem ser lidos.

## As evidências não são alteradas { #the-evidence-is-not-changed }

A imagem é aberta somente para leitura e a montagem recusa gravações. Em um VHDX que não foi fechado corretamente, o log pendente é aplicado na memória, sem gravá-lo na imagem. Alguns componentes DiscUtils deixam zeradas a instrução de salto e a assinatura `55 AA` do setor de inicialização NTFS; Juntei lê esses volumes como estão.

## Uso { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Análise nativa de imagens { #parsing-every-image-with-auto-parse }

Com `--vhdx`, `auto-parse` lê VHDX nativamente no **Linux, macOS e Windows**, usando [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser). Não exige montagem pelo sistema, FUSE ou privilégios de administrador.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Os parsers precisam de caminhos: o Juntei copia os arquivos que os parsers leem para um espaço temporário privado com buffers limitados: hives do registro e seus logs de transação, arquivos de metadados NTFS, bancos de dados, arquivos de prefetch, atalhos, Jump Lists, arquivos do Outlook e pastas de quarentena do Defender. Qualquer outro arquivo vira um marcador vazio com o horário de modificação original, de modo que diretórios e caminhos aparecem como no disco. O persistence-hunter lê o conteúdo de um marcador a partir da imagem só quando precisa, por exemplo um executável indicado por uma entrada de persistência. **Reserve espaço para os arquivos copiados**, além dos resultados. Regiões nulas são gravadas como lacunas quando permitido. Nomes, estrutura, hard links e horários de modificação são preservados. Os horários de origem de LNK e Jump Lists vêm dos metadados NTFS originais, nunca dos arquivos temporários. O término normal, mesmo com erros de parser, remove as cópias; uma interrupção forçada pode deixá-las. A origem permanece somente leitura; logs VHDX pendentes são aplicados apenas na memória.

A descoberta de hosts, CSV/JSON/JSONL e `-j` usam o fluxo habitual. `Image` em `manifest.json` identifica a imagem original; `EvidenceRoot`, o caminho relativo interno. Erros de leitura e nomes inseguros aparecem em `ImageErrors` e nas linhas com falha de `manifest.csv`. Resultados legíveis são preservados; entradas incompletas retornam 1. Nomes perigosos ou ambíguos não são renomeados. Streams NTFS nomeados não são extraídos como arquivos adicionais; artefatos coletados como arquivos comuns continuam disponíveis.

O [`mount` separado](mount.md) expõe o sistema de arquivos a outros programas e ainda exige Linux FUSE. `auto-parse --vhdx` não o utiliza.

A preparação usa por padrão um diretório privado no sistema de arquivos de saída. Use `--temp-dir` para escolher um diretório existente em outro disco. Os metadados originais ficam separados no volume de saída, permitindo analisar arquivos já legíveis mesmo se o disco de preparação encher. Reserve espaço para cópias e resultados; se o volume de saída estiver cheio, os resultados não podem ser salvos. Ctrl+C interrompe os processos, limpa a área privada e sai com código 130. SIGKILL ou falta de energia ainda podem deixar cópias. Subdiretórios inacessíveis são ignorados na busca; uma raiz de entrada ilegível continua sendo um erro.

O diretório de preparação deve ficar fora do diretório de saída. No Linux e no macOS, SIGTERM e SIGHUP também interrompem os processos de análise e limpam a área de trabalho.

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## Imagens de disco E01 e Ex01 { #e01-and-ex01-disk-images }

`--e01` lê imagens físicas EWF1 (`.E01`) e EWF2 (`.Ex01`) nativamente no **Linux, macOS e Windows**, sem FUSE, montagem do sistema ou privilégios de administrador. Usa [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) e o mesmo processamento NTFS descrito acima.

Informe o primeiro segmento de uma imagem dividida. Mantenha todos os arquivos associados, incluindo `.E02`, ao lado dele com seus nomes originais. Há suporte para discos inteiros (MBR/GPT) e volumes NTFS individuais. Use `--vhdx --e01` para diretórios mistos.

Valem os mesmos requisitos de espaço temporário e metadados de origem. Imagens E01 costumam conter discos inteiros, e todas as imagens são preparadas antes do início da análise, mas só são copiados os arquivos que os parsers leem, normalmente uma pequena parte do espaço usado. As imagens são somente leitura. Lacunas de aquisição e danos encontrados ao abrir ou ler aparecem em `ImageErrors` e `manifest.csv`; os resultados legíveis são preservados com código de saída 1. Apenas os dados acessados são verificados, não todos os blocos nem o hash da mídia completa. Arquivos lógicos `.L01` e `.Lx01` são recusados; não há opção de senha.

O comando separado `mount` ainda exige Linux FUSE; a análise automática nativa não.

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## Requisitos da montagem visível ao sistema { #os-visible-mount-requirements }

`juntei mount` exige Linux FUSE 3 (`fusermount3`) e acesso a `/dev/fuse`. No Windows e macOS, use a análise nativa acima.
