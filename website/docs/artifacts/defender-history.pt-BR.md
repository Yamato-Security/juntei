# Histórico de detecções do Microsoft Defender { #microsoft-defender-detection-history }

## O que o histórico registra { #what-the-history-records }

O Microsoft Defender grava um arquivo de histórico de detecções para cada detecção. Cada arquivo registra:

- o identificador, o nome, a gravidade e a categoria da ameaça;
- o que o Defender fez e se teve sucesso;
- se a ameaça estava em execução;
- o usuário e o processo envolvidos, e o que detectou a ameaça;
- a entrada de quarentena, se houver;
- cada recurso em que a ameaça foi encontrada: arquivos, membros de arquivos compactados, downloads, processos, valores do Registro,
  serviços, linhas de comando e comportamentos.

O Defender também grava esses arquivos para ameaças que nunca colocou em quarentena: as que ele apenas detectou, as que
foram permitidas e as que ele não conseguiu remover. O histórico de proteção da Segurança do Windows e a classe WMI
`MSFT_MpThreatDetection` (`Get-MpThreatDetection`) mostram esses registros. Quando a Orange
Cyberdefense alterou valores em um arquivo, o WMI informou os valores alterados.

<span class="evidence-strength">Melhor conclusão:</span> o Defender registrou uma detecção da ameaça nomeada
nos recursos listados, com o status e os horários registrados. O nome da ameaça é o veredito do produto. Um
registro de detecção, por si só, não mostra que a ameaça foi executada nem que ela foi removida por
completo.

## Localização e coleta { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

Cada arquivo tem o nome do seu `DetectionID`. Colete toda a pasta `DetectionHistory`. A entrada por diretório e o
`auto-parse` leem todo arquivo dentro de uma pasta `DetectionHistory`, em qualquer profundidade, que comece como um
arquivo de histórico de detecções. Use `-f` para uma cópia mantida fora de uma pasta assim. O Defender remove itens do
seu histórico de verificação após um número configurável de dias (`ScanPurgeItemsAfterDelay` em
`Set-MpPreference`), então detecções mais antigas podem ter desaparecido.

A quarentena é um artefato separado: consulte [Quarentena do Defender](defender-quarantine.md). O
`QuarantineID` de uma detecção usa a mesma grafia que o `EntryID` da saída da quarentena (maiúsculas,
sem chaves), de modo que os dois podem ser vinculados.

## Cuidados para o analista { #analyst-cautions }

- Detecção, tentativa de remediação e remoção bem-sucedida são estados distintos. Leia
  `ThreatStatus` (`Detected`, `Quarantined`, `Removed`, `Allowed`, `RemoveFailed`, …) junto com
  `ThreatStatusErrorCode`, que é 0 quando a última mudança de status teve sucesso.
- `ExecutionStatus` é a visão do Defender no momento da detecção (`Blocked`, `Allowed`, `Executing`,
  `NotExecuting`, `Unknown`). Não é prova de que a ameaça foi executada, nem de que nunca foi executada.
  Corrobore com Prefetch, Amcache, logs de eventos e telemetria de processos.
- `ThreatName` é o nome da assinatura naquele momento. Falsos positivos e reclassificações posteriores são
  possíveis. Arquivos de teste como o EICAR são relatados como ameaças reais.
- `User` é o valor `DomainUser` registrado pelo Defender (`UserSource` é `DomainUser`). Ele pode
  estar vazio, ser uma conta de serviço ou um usuário diferente daquele que criou o arquivo.
- O Defender pode gravar vários arquivos para uma mesma ameaça, e o Juntei mantém cada um deles. Dois arquivos podem
  até conter o mesmo `DetectionID`, como acontece com um arquivo copiado ou restaurado. Agrupe por `DetectionID` e
  `SourceFile` em conjunto.
- Valores cujo significado não é conhecido são mantidos, não interpretados: sua seção e sua posição ficam em
  `UnknownValues`.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

CSV é o formato de arquivo padrão. Com `-t csv`, `-t json` ou `-t jsonl` sem `-o`, os registros vão para
stdout. Sem nenhuma das duas opções, apenas um resumo é exibido. Adicione `-C` para sobrescrever uma saída
existente. A saída nunca é gravada sobre um dos arquivos de entrada, mesmo com `-C`.

O Juntei grava um registro por recurso de cada detecção. Uma detecção sem recursos recebe um registro com os
campos de recurso vazios. Os registros de uma detecção compartilham o seu `DetectionID` e são numerados por
`ResourceIndex` na ordem armazenada.

Danos não interrompem a execução. Um arquivo truncado mantém os valores lidos antes do dano, e
`Problems` informa o que está faltando. Um arquivo em uma pasta `DetectionHistory` que não é um arquivo de histórico
de detecções é relatado e ignorado. Os demais arquivos continuam sendo lidos.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Record` | `resource`, ou `detection` para uma detecção sem recursos. |
| `InitialDetectionTime` | Quando o Defender detectou a ameaça pela primeira vez (UTC). Vazio quando o arquivo contém 0; um valor que não é um horário é mantido em `UnknownValues`. |
| `LastThreatStatusChangeTime` | Quando o status da ameaça mudou pela última vez (UTC). |
| `RemediationTime` | Quando o Defender remediou a ameaça (UTC). É o horário de uma tentativa, não prova de remoção; verifique `ThreatStatus`. |
| `ThreatName`, `ThreatID` | Nome da assinatura e identificador numérico do Defender (`Get-MpThreatCatalog`). |
| `Severity`, `SeverityID` | `Low`, `Moderate`, `High`, `Severe` ou `Unknown`, com o número armazenado. |
| `Category`, `CategoryID` | Categoria da ameaça, como `TROJAN`, `TOOL` ou `BEHAVIOR`, com o número armazenado. |
| `ThreatStatus`, `ThreatStatusID` | Status da ameaça segundo o Defender. Um número sem nome documentado é mantido com `ThreatStatus` vazio. |
| `ThreatStatusErrorCode` | Código de erro da última mudança de status; 0 significa que ela teve sucesso. |
| `ExecutionStatus`, `ExecutionStatusID` | Se o Defender viu a ameaça em execução quando ela foi detectada. |
| `DetectionSource`, `DetectionSourceTypeID` | O que detectou a ameaça: `Real-time`, `IOAV` (downloads e anexos), `User`, `System`, `AMSI` e outros. |
| `User`, `UserSource` | O valor `DomainUser` registrado pelo Defender e sua base (`DomainUser`). Vazio quando o arquivo não nomeia nenhum usuário. |
| `Process` | O processo envolvido (`ProcessName`); pode ser `Unknown`. |
| `ResourceIndex`, `ResourceCount` | A posição deste recurso e o número de recursos na detecção. |
| `ResourceType` | `file`, `containerfile`, `webfile`, `process`, `regkeyvalue`, `service`, `CmdLine`, `behavior`, `amsi` e outros, como o Defender os nomeia. |
| `Path` | O recurso: um caminho, um membro de arquivo compactado (`archive.zip->member`), um valor do Registro, uma linha de comando ou um processo. Um `webfile` também indica sua URL e o processo que fez o download (`pid:…,ProcessStart:…`). Preserve a grafia exata. |
| `ResourceFlags` | Flags do recurso em hexadecimal; seu significado não é documentado. |
| `TrackingStartTime` | O `ThreatTrackingStartTime` do recurso, um inteiro lido como FILETIME (UTC), da mesma forma que o plaso data uma detecção. |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | Hashes e tamanho dos dados de rastreamento do recurso. Eles descrevem o arquivo no momento em que o Defender o verificou. |
| `TrackingID` | O `ThreatTrackingId` do recurso. |
| `OtherTracking` | Todos os outros valores de rastreamento, como `Key=Value` na ordem armazenada, por exemplo `ThreatTrackingScanSource` ou `ThreatTrackingBMContextRich` em detecções comportamentais. |
| `QuarantineID` | A entrada de quarentena (`Quarantine\Entries\{GUID}`); vazio quando o arquivo contém um GUID zero. |
| `DetectionID` | O GUID da detecção; também é o nome do arquivo. Uma divergência é relatada em `Problems`. |
| `MagicVersion` | O `Magic.Version:1.2` do cabeçalho; vazio em arquivos gravados sem ele, como alguns do Windows Server 2016. |
| `UnknownValues` | Valores cujo significado não é conhecido, como `position=value` (por exemplo, `6.13=NT AUTHORITY\SYSTEM`), com a lista contada que vem após o status como `2.list`. |
| `SourceFile` | O arquivo lido, relativo ao diretório de entrada, ou o seu nome com `-f`. |
| `Problems` | Danos, valores ausentes, tipos inesperados ou um nome de arquivo que não corresponde. Não suprima. |

JSON e JSONL mantêm os mesmos campos com valores tipados: números continuam sendo números e valores ausentes são
`null`. Em vez de `OtherTracking`, eles têm `Tracking`, cada valor de rastreamento com a chave do Defender,
seu tipo e seu valor. `UnknownValues` vira uma lista de posições com tipos. `CountedList`
contém a lista contada. Um FILETIME armazenado mantém sua contagem bruta de ticks ao lado do horário que ele representa.

## Comportamento na linha do tempo { #timeline-behavior }

O `auto-parse` salva os registros como `SecurityProducts/DefenderHistory.csv` (ou `.json`/`.jsonl`) e
os adiciona à linha do tempo combinada `-j`. O `super-timeline` gera eventos a partir de
`InitialDetectionTime`, `LastThreatStatusChangeTime` e `RemediationTime` no registro de cada recurso,
com o status, o código de erro, o status de execução e a origem da detecção nos detalhes do evento.
`TrackingStartTime` é armazenado como inteiro, então só vira evento com
`--include-interpreted-times`. Os horários do próprio arquivo nunca são usados. Um registro sem horário utilizável
permanece na saída analisada e é contado como sem horário.

## Estratégia de investigação { #investigation-strategy }

1. Comece por `ThreatName`, `Path`, `SHA256`, `InitialDetectionTime` e `ThreatStatus`.
2. Verifique `ThreatStatusErrorCode` e `Problems` antes de concluir que uma ameaça foi removida.
3. Vincule `QuarantineID` ao `EntryID` da quarentena para recuperar os bytes em quarentena.
4. Pivote hashes e caminhos para `$MFT`, Diário USN, Prefetch, Amcache, histórico do navegador e logs
   de eventos (o log operacional do Defender: IDs de evento 1116, 1117, 1118 e 1119).
5. Para recursos `webfile`, use a URL e o início do processo para investigar o acesso inicial, e
   valide-os com evidências de navegador, proxy e DNS.

## Versões testadas { #tested-versions }

Os testes usam arquivos públicos com resultados que outras ferramentas leram de forma independente:

- A amostra do Windows 10 da Orange Cyberdefense (Defender 4.18.24090.11), conferida com o que o WMI
  informou.
- Os dois arquivos de teste do Windows do plaso, conferidos com o analisador `windefender_history` do plaso.
- 14 arquivos da VM do simulador de APT do DFIRArtifactMuseum.
- Uma cópia gravada sem a versão do cabeçalho, como alguns arquivos do Windows Server 2016.
- Arquivos truncados, vazios ou de outro tipo.

## Mais detalhes { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [defender da Sootmark](https://github.com/Sootmark/defender)
- [wddh-parser da Orange Cyberdefense](https://github.com/cert-orangecyberdefense/wddh-parser)
- [Analisador de histórico do Defender do plaso](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: classe MSFT_MpThreatDetection](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: IDs de evento do Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
