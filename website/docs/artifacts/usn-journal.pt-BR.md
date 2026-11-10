# Diário USN (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## O que é o Diário USN { #what-the-usn-journal-is }

O NTFS registra as alterações em arquivos e diretórios no diário de alterações USN (Update Sequence Number). Um
registro pode descrever criação, extensão/sobrescrita de dados, alterações de atributos ou de segurança, renomeação,
fechamento e exclusão. Ele armazena uma referência de arquivo, a referência do pai, o nome, os motivos, os atributos,
o carimbo de data/hora e um USN monotonicamente crescente.

O diário é excelente para reconstruir uma rajada de operações de arquivo de um atacante, especialmente quando
os arquivos foram excluídos. Ele registra **que os metadados/o conteúdo mudaram**, não os bytes alterados.

<span class="evidence-strength">Melhor conclusão:</span> o NTFS registrou os motivos de alteração indicados para
o arquivo referenciado no carimbo de data/hora. Os motivos costumam se acumular até um registro `CLOSE`.

## Localização e coleta { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` é um fluxo nomeado de metadados do NTFS. Colete-o com uma ferramenta forense com acesso bruto ao disco. Colete também o
`$MFT` do mesmo volume, porque os registros USN armazenam referências de arquivo do pai, e não caminhos completos.

## Cuidados para o analista { #analyst-cautions }

- O diário é circular e tem tamanho limitado. Os registros antigos são sobrescritos.
- `FILE_DELETE` indica que o NTFS processou a exclusão; o conteúdo ainda pode ser recuperável em outro lugar.
- Um motivo como `DATA_OVERWRITE` não revela os bytes, o processo nem o usuário responsável.
- Vários motivos podem ser combinados em um único registro; `CLOSE` muitas vezes marca o fim das alterações acumuladas.
- Os caminhos são reconstruídos. Os diretórios pais podem ter sido renomeados, excluídos ou reutilizados.
- Os números de sequência das referências de arquivo importam. O número da entrada, sozinho, pode levar à geração errada.
- A recuperação por carving pode recuperar registros válidos sem o contexto original do diário; relate essa distinção.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## Campos de saída CSV { #csv-output-fields }

| Campo | Interpretação |
|---|---|
| `Name` | Nome do arquivo/diretório armazenado neste registro USN. Não é um caminho completo. |
| `Extension` | Extensão extraída de `Name` por conveniência; não a use como tipo de conteúdo confiável. |
| `EntryNumber`, `SequenceNumber` | Referência NTFS do arquivo. Use ambos para identificar a geração do registro. |
| `ParentEntryNumber`, `ParentSequenceNumber` | Referência NTFS do diretório pai no momento da alteração. |
| `ParentPath` | Caminho do pai reconstruído pelo Juntei levando em conta o tempo. Marcadores de desconhecido significam que a ascendência não pôde ser resolvida. |
| `UpdateSequenceNumber` | Valor de ordenação USN. Útil para uma ordenação estável e para pivôs com Amcache/MFT. |
| `UpdateTimestamp` | Carimbo de data/hora armazenado em registros V2/V3. Trate os carimbos de data/hora de contexto recuperado por carving com a mesma validação de qualquer registro. |
| `UpdateReasons` | Um ou mais flags de alteração nomeados, unidos por `|`. Interprete as combinações e o `CLOSE` em conjunto. |
| `FileAttributes` | Atributos NTFS registrados no momento da alteração; os bits desconhecidos permanecem em hexadecimal. |
| `OffsetToData` | Deslocamento em bytes na entrada. Essencial para validação e para registros recuperados por carving. |
| `SourceFile` | Diário/imagem de entrada; distingue hosts e volumes. |
| `SourceInfo` | Flags de origem que descrevem como a alteração entrou no diário. Geralmente contextual, não atribuição. |
| `SecurityID` | Índice do identificador de segurança NTFS do registro, não um SID de usuário. |
| `Version` | Versão do registro USN. O Juntei grava V2/V3 em CSV; o JSON mantém as extensões (extents) de rastreamento de intervalos do V4. |

JSON/JSONL representam corretamente as referências V3 de 128 bits e preservam os registros de rastreamento de intervalos do V4 e
suas extensões. Use-os quando os campos de baixo nível específicos de cada versão forem importantes.

## Estratégia de investigação { #investigation-strategy }

1. Filtre por horário, nome, extensão, caminho do pai ou referência de arquivo.
2. Ordene por `UpdateSequenceNumber`, além do horário, para ver a ordem exata no diário.
3. Associe `RENAME_OLD_NAME` a `RENAME_NEW_NAME` para movimentações/renomeações.
4. Use o `$MFT` para validar o estado dos registros, caminhos, tamanhos e carimbos de data/hora.
5. Correlacione com a atribuição de processos a partir de logs de eventos, EDR, Prefetch, serviços/tarefas ou Defender.

## Mais detalhes { #further-details }

- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: flags de motivo do Change Journal](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [Crate usn do ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
