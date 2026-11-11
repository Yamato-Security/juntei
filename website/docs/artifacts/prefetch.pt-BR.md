# Prefetch { #prefetch }

## O que é o Prefetch { #what-prefetch-is }

Os arquivos Prefetch do Windows registram informações usadas para otimizar a inicialização de aplicativos. Um arquivo `.pf`
específico de um programa normalmente armazena o nome e o hash do executável, a contagem de execuções, os horários de execução recentes, informações
de volume, diretórios e arquivos referenciados durante a inicialização.

Nas versões compatíveis do Windows, um registro Prefetch de aplicativo válido costuma ser uma evidência forte
de que o executável foi executado. É um dos artefatos de execução mais úteis fora dos logs de eventos.

<span class="evidence-strength">Melhor conclusão:</span> o executável nomeado foi iniciado no(s)
horário(s) de execução recente(s) registrado(s), sujeito ao comportamento do sistema operacional e à validade do analisador.

## Localização e coleta { #location-and-collection }

```text
C:\Windows\Prefetch\*.pf
```

O Juntei é compatível com as versões de formato 17 a 31 (Windows XP a Windows 11), incluindo os arquivos compactados
do Windows 8+. Colete o diretório inteiro para que os hashes dos nomes de executáveis e os registros vizinhos possam
ser comparados.

## Cuidados para o analista { #analyst-cautions }

- O Windows armazena apenas um horário de execução histórico nas versões mais antigas e até oito nas versões mais novas.
  `RunCount` pode exceder o número de carimbos de data/hora disponíveis.
- O Prefetch pode estar desativado, limpo, excluído ou indisponível em alguns sistemas/funções. A ausência não é
  prova de que um programa não foi executado.
- As listas de arquivos carregados descrevem dependências e acessos durante a inicialização; elas não provam que cada arquivo listado
  era malicioso, foi aberto deliberadamente ou foi lido por completo.
- O nome do executável no cabeçalho do Prefetch tem comprimento limitado. O Juntei distingue
  caminhos carregados com o mesmo nome pelo hash de caminho SCCA. Se vários candidatos continuarem
  sem resolução, ele mantém o nome do cabeçalho e define PathAmbiguous; a lista de arquivos
  carregados continua disponível. Um único candidato cujo hash não corresponde é mantido, com
  PathValidated como false. Processos hospedeiros podem incluir dados da linha de comando no hash,
  portanto um caminho não validado não está necessariamente errado.
- Os carimbos de data/hora do sistema de arquivos de um arquivo de origem `.pf` são distintos dos horários de execução incorporados.
- `Findings` são observações de caça a ameaças. Instaladores e a administração legítima usam frequentemente caminhos de preparação.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.ExecutableName? | test("powershell|rundll32"; "i"))'
```

## Campos CSV padrão { #standard-csv-fields }

| Campo | Interpretação |
|---|---|
| `SourceFile` | `.pf` de entrada; mantenha-o para distinguir hosts e nomes de executáveis duplicados. |
| `ExecutableName` | Nome do executável no cabeçalho. Sem distinção entre maiúsculas e minúsculas para a maior parte da caça a ameaças; preserve a grafia original. |
| `ExecutablePath` | Caminho carregado resolvido pelo Juntei, quando possível. Confirme-o com `Files` e outros artefatos. |
| `PathValidated` | O caminho selecionado corresponde ao hash de caminho SCCA. Isso valida sua atribuição, não o conteúdo ou a confiabilidade do arquivo. |
| `PathAmbiguous` | Vários caminhos carregados permanecem possíveis; `ExecutablePath` mantém o nome do cabeçalho. Confira `Files` e outros artefatos. |
| `PrefetchHash` | Hash que o Windows usa no nome do arquivo `.pf`; não é um hash criptográfico do conteúdo do arquivo. |
| `FormatVersion` | Versão da estrutura do Prefetch; determina os campos disponíveis e a quantidade de carimbos de data/hora. |
| `RunCount` | Contador cumulativo registrado pelo Windows. Útil para frequência, não para o histórico exato. |
| `LastRun` | Horário de execução mantido mais recente. |
| `PreviousRun1` … `PreviousRun7` | Horários de execução anteriores mantidos, do mais recente ao mais antigo. Vazio quando o formato/histórico não os tem. |
| `VolumeCount` | Número de volumes de origem registrados. |
| `VolumeDevicePaths` | Caminhos de dispositivo NT dos volumes usados durante a inicialização. Correlacione com os números de série. |
| `VolumeSerialNumbers` | Identificadores de volume; úteis para vincular evidências de unidades removíveis ou reformatadas. |
| `VolumeCreated` | Horários de criação de volume registrados, não horários de criação do executável. |
| `DirectoryCount` | Número de strings de diretório registradas. |
| `FileCount` | Número de referências a arquivos carregados registradas. |
| `Findings` | Observações do Juntei, como binários do sistema realocados ou caminhos de execução suspeitos. Revise manualmente. |
| `Files` | Arquivos carregados/referenciados com as referências NTFS disponíveis. Excelente pivô de DLL/caminho, não prova de intenção sobre o arquivo. |
| `Dir` | Strings de diretório associadas à inicialização. Úteis quando os caminhos individuais dos arquivos carregados estão incompletos. |

## Campos CSV da linha do tempo (`-T`) { #timeline-csv-fields-t }

| Campo | Interpretação |
|---|---|
| `RunTime` | Um horário de execução mantido pelo Prefetch. |
| `ExecutableName`, `ExecutablePath` | Contexto de identidade/caminho do programa. |
| `PathValidated` | O caminho selecionado corresponde ao hash de caminho SCCA. Isso valida sua atribuição, não o conteúdo ou a confiabilidade do arquivo. |
| `PathAmbiguous` | Vários caminhos carregados permanecem possíveis; `ExecutablePath` mantém o nome do cabeçalho. Confira `Files` e outros artefatos. |
| `PrefetchHash` | Separa variantes/caminhos que compartilham um nome de executável. |
| `RunCount` | Contador do arquivo Prefetch; é repetido em cada linha da linha do tempo. |
| `SourceFile` | `.pf` de origem para validação. |

O JSON/JSONL preserva volumes, diretórios, arquivos carregados e referências de arquivos NTFS como arrays
estruturados, em vez de células CSV unidas por vírgulas.

## Estratégia de investigação { #investigation-strategy }

1. Use `-T` para adicionar as execuções mantidas a uma linha do tempo.
2. Agrupe pelo nome do executável mais o hash do Prefetch; o mesmo nome pode ter caminhos/hashes diferentes.
3. Revise `ExecutablePath`, caminhos de preparação, achados de binários do sistema realocados e DLLs carregadas.
4. Associe as referências MFT de destino e os caminhos de arquivo aos registros do `$MFT`/USN.
5. Correlacione os horários de execução com logs de eventos, SRUM, telemetria de rede, serviços, tarefas e artefatos do usuário.

## Mais detalhes { #further-details }

- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [Security Ronin prefetch-forensic](https://github.com/SecurityRonin/prefetch-forensic)
