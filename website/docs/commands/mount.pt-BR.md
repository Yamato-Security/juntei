# Montar imagens de disco { #mounting-disk-images }

O `juntei mount` torna os arquivos de uma imagem de disco VHDX legíveis em um diretório, somente leitura,
enquanto estiver em execução. Assim, as coletas `--vhdx` do KAPE e os discos do Hyper-V podem ser examinados
por qualquer ferramenta, como o Hayabusa ou os próprios comandos do Juntei, sem extrair nada, sem root e sem que
o sistema operacional ofereça suporte a VHDX ou NTFS. Ele usa FUSE e funciona no Linux.

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

- Os arquivos do volume NTFS no ponto de montagem: em uma coleta do KAPE, a pasta `C` que contém os arquivos
  coletados, a pasta `LongFileNames` do KAPE e os próprios arquivos de metadados do volume, como o `$MFT`. Uma
  imagem com vários volumes NTFS os mostra como `Volume1`, `Volume2` e assim por diante.
- Todo arquivo é somente leitura e pertence ao proprietário do ponto de montagem. Os carimbos de data/hora dos
  arquivos são os de `$STANDARD_INFORMATION` do NTFS.
- Os nomes são listados como o NTFS os armazena e são procurados de forma exata. Nomes DOS 8.3 e fluxos de dados
  alternativos não são mostrados.
- Arquivos compactados são descompactados (até 1 GiB cada). Arquivos criptografados com EFS não podem ser lidos.

## As evidências não são alteradas { #the-evidence-is-not-changed }

A imagem é aberta somente para leitura e a montagem recusa gravações. Um VHDX que não foi fechado corretamente é
lido com seu log pendente aplicado na memória, sem que nada jamais seja gravado de volta. O KAPE grava seus
arquivos VHDX com o DiscUtils, que deixa o setor de inicialização NTFS sem a instrução de salto e sem a
assinatura `55 AA`; o Juntei lê esses volumes como estão.

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

## Analisar todas as imagens com o auto-parse { #parsing-every-image-with-auto-parse }

O `auto-parse --vhdx` analisa os sistemas Windows contidos em cada imagem de disco VHDX do diretório de
evidências (ou na única imagem informada com `-d`), sem uma montagem separada:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- Cada imagem é montada somente leitura em um novo diretório temporário, em uma pasta com o nome da imagem
  (`host1-triage`), e os sistemas encontrados ali são analisados como em uma coleta em disco. As imagens são
  desmontadas no final e também ao pressionar Ctrl+C.
- O terminal lista as imagens e a imagem de cada sistema:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- No manifesto, `EvidenceDirectory` é o diretório informado e o campo `Image` de cada sistema indica sua imagem.
  O `EvidenceRoot` e os comandos dos analisadores usam a pasta de montagem (`./host1-triage/C`).
- O KAPE armazena os arquivos que copia das Cópias de Sombra de Volume (Volume Shadow Copies) em pastas como
  `VSS32`. Cada uma delas é encontrada e analisada como mais um sistema (`HOST1_2`, …).
- Uma imagem que não pode ser montada é relatada; as demais ainda são analisadas, e a execução termina com
  status de saída 1.

## Requisitos { #requirements }

- Linux: FUSE 3 (`fusermount3`, o pacote `fuse3`) e acesso a `/dev/fuse`, que usuários comuns normalmente têm.
  Não é necessário ser root.
- Windows e macOS não são suportados. No Windows, em vez disso, anexe a imagem somente leitura em um
  PowerShell com privilégios elevados e informe às ferramentas a letra da unidade dela:

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
