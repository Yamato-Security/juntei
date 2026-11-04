# Setor de inicialização (`$Boot`) { #boot-sector-boot }

## O que é o `$Boot` { #what-boot-is }

`$Boot` é a entrada 7 da MFT de todo volume NTFS: os primeiros 8 KiB do volume. Seu primeiro setor, o
**registro de inicialização do volume** (volume boot record), contém o bloco de parâmetros do BIOS que descreve o sistema de arquivos:

- os tamanhos de setor e de cluster, o número de setores do volume e os **setores ocultos**
  (os setores do disco antes do volume, ou seja, onde sua partição começa);
- onde o `$MFT` e o `$MFTMirr` começam, e os tamanhos dos registros FILE e INDX;
- o **número de série do volume** de 64 bits, definido quando o volume é formatado.

O restante do primeiro setor e os 15 setores seguintes contêm o **código de inicialização** que carrega
o Windows.

Os 32 bits inferiores do número de série são o que o Windows exibe como `XXXX-XXXX` (`vol`, `dir`) e o que
[atalhos LNK](lnk.md), [Jump Lists](jump-lists.md) e [Prefetch](prefetch.md) registram sobre o
volume em que um arquivo estava. O `$Boot` vincula esses registros a um volume de um host coletado.

<span class="evidence-strength">Melhor conclusão:</span> um número de série de volume em um LNK, Jump List ou Prefetch
igual ao `VolumeSerialNumber` de um `$Boot` aponta para esse volume, ou para outro volume que
recebeu o mesmo número de série ou foi copiado com ele. Números de série podem ser alterados e não são
únicos entre hosts.

## Localização e coleta { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

Colete-o com uma ferramenta forense com acesso bruto ao disco, ou leia o primeiro setor de uma imagem de volume: ambos são
os mesmos bytes. O Juntei lê qualquer arquivo que comece com um setor de inicialização NTFS, portanto uma imagem de volume funciona
como entrada. Uma imagem de disco começa com a tabela de partições; forneça ao Juntei o volume da partição.

## Cuidados para o analista { #analyst-cautions }

- O número de série muda quando o volume é formatado. Um arquivo LNK cujo número de série não corresponde
  a nenhum volume atual pode apontar para uma mídia removível, o servidor de um compartilhamento de rede ou um volume formatado
  desde então.
- O código de inicialização é o mesmo em volumes formatados pelo mesmo Windows: ele é idêntico nos arquivos `$Boot`
  do Windows 10, 11 e Server 2012 R2 do DFIR Artifact Museum. Um bootkit de registro de inicialização de volume
  o altera, ou altera o campo de setores ocultos, que o código de inicialização usa para encontrar o restante do
  `$Boot`. Compare `BootCodeSHA256`, `JumpInstruction` e `HiddenSectors` com outros hosts e com a tabela de
  partições antes de concluir qualquer coisa: um gerenciador de inicialização ou um software de criptografia também pode alterar o
  código de inicialização.
- Uma imagem de um volume criptografado com BitLocker começa com o próprio setor de inicialização do BitLocker (OEM ID
  `-FVE-FS-`). Fornecida com `-f`, o Juntei a relata, e `Problems` informa que não é NTFS; o setor de inicialização
  NTFS está nos dados criptografados. Um `$Boot` coletado do volume desbloqueado é o NTFS.
- `Problems` lista inconsistências estruturais reconhecidas e truncamentos, em vez de fornecer um teste completo de montabilidade pelo Windows. Use-o para avaliar setores recuperados por carving e cópias, não como um achado de ataque por si só. O hash do código de inicialização exclui o salto inicial, o bloco de parâmetros do BIOS e a assinatura final; inspecione esses campos também.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

O resumo lista o número de série do volume de cada setor de inicialização (como o Windows o exibe e como os campos de LNK e
Prefetch o exibem), o tamanho, o tamanho do cluster e o cluster do `$MFT`. Quando vários hosts são analisados, ele
os agrupa pelo hash do código de inicialização, de modo que um volume cujo código de inicialização difere dos outros se destaca.

## Campos de saída CSV { #csv-output-fields }

| Campo | Interpretação |
|---|---|
| `SourcePath` | Arquivo de entrada. |
| `VolumeSerialNumber` | O número de série de 32 bits, como a saída de LNK, Jump List e Prefetch o exibe (`180B1E67`). O Windows o exibe como `180B-1E67`. |
| `VolumeSerialNumber64` | O número de série completo de 64 bits. |
| `OEM-ID` | `NTFS` em um volume NTFS. |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | A geometria do volume. Valores de setores por cluster acima de 128 são registrados como potência de dois, decodificada aqui. Vazio quando não é válido. |
| `TotalSectors`, `VolumeSize` | Setores do volume e seu tamanho em bytes, sem a cópia do setor de inicialização no final. |
| `HiddenSectors` | Setores do disco antes do volume: onde sua partição começa. |
| `MFT-Cluster`, `MFT-Offset` | Onde o `$MFT` começa, em clusters e em bytes a partir do início do volume. |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | Onde o `$MFTMirr` começa. |
| `FileRecordSize`, `IndexRecordSize` | Bytes por registro FILE (geralmente 1.024) e por registro INDX (geralmente 4.096). |
| `MediaDescriptor` | `F8` para um disco rígido. |
| `JumpInstruction` | Os três primeiros bytes (`EB 52 90`): um salto sobre o bloco de parâmetros do BIOS até o código de inicialização. |
| `EndSignature` | `55 AA` no final de um setor de inicialização válido. |
| `BootCodeSHA256` | SHA-256 do código de inicialização: os bytes 0x54 a 0x1FD do primeiro setor e o restante dos 8 KiB. |
| `Problems` | O que o Windows não montaria: um OEM ID diferente de `NTFS`, ausência de `55 AA`, tamanhos inválidos, campos FAT que o NTFS mantém zerados preenchidos, o `$MFT` além do fim do volume; e um arquivo menor que 8 KiB. |

JSON/JSONL têm os mesmos campos e nomes, com números como números, `null` para valores desconhecidos e
`Problems` como uma lista.

## Estratégia de investigação { #investigation-strategy }

1. Coloque o `VolumeSerialNumber` de cada host ao lado dos números de série de volume de seus arquivos LNK, Jump
   Lists e arquivos Prefetch: os que não correspondem a nenhum volume coletado são outros volumes (unidades
   USB, compartilhamentos, volumes formatados desde então).
2. Procure os outros números de série na saída de [dispositivos USB](usb.md), que lista os números de série
   de volume dos volumes removíveis montados.
3. Compare `BootCodeSHA256` e `HiddenSectors` entre hosts. Investigue um código de inicialização divergente
   com um disassembler e a tabela de partições do disco, e procure entradas de log de eventos relacionadas à inicialização.
4. Use `MFT-Offset` e `ClusterSize` para encontrar o `$MFT` em uma imagem de volume.

## Mais detalhes { #further-details }

- [Microsoft: Visão geral do NTFS](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: documentação do formato NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (o cabeçalho do volume)
- [The Sleuth Kit: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (seu crate `boot`)
