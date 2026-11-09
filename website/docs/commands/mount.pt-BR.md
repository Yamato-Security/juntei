# Montar imagens de disco { #mounting-disk-images }

`juntei mount` disponibiliza os arquivos de uma imagem de disco VHDX em um diretório, somente para leitura, durante sua execução. Ferramentas como Hayabusa e Juntei podem analisá-los sem extração nem privilégios de root. Usa FUSE e funciona no Linux. O leitor e a biblioteca de montagem são mantidos em [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

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
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Análise nativa de imagens { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` lê imagens VHDX nativamente no **Linux, macOS e Windows**, sem montagem pelo sistema, FUSE ou privilégios de administrador.

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Os parsers precisam de caminhos de arquivo: o Juntei copia os arquivos expostos pelo leitor NTFS para um diretório temporário privado. **Reserve espaço temporário suficiente para todos esses arquivos.** A imagem original permanece somente leitura e as cópias são removidas ao terminar normalmente.

Falhas de leitura e nomes inseguros são registrados em `ImageErrors` e nas linhas com falha de `manifest.csv`. Os resultados legíveis são preservados; entradas incompletas retornam o código 1.

O comando separado `mount`, que expõe arquivos a outros programas, ainda exige Linux, FUSE 3 e acesso a `/dev/fuse`.

[English: implementation details and limitations](https://yamato-security.github.io/juntei/commands/auto-parse/#vhdx-disk-images).
