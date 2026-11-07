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
- Se a imagem contiver arquivos de Volume Shadow Copies em pastas como `VSS32`, cada pasta será detectada e analisada como outro sistema (`HOST1_2`, …).
- Uma imagem que não pode ser montada é relatada; as demais ainda são analisadas, e a execução termina com
  status de saída 1.

Se um volume NTFS não puder ser aberto, o Juntei informa o problema e mantém os volumes legíveis. Para imagens parcialmente legíveis, `auto-parse --vhdx` retorna o código 1; `mount` também retorna 1 após desmontar.

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
