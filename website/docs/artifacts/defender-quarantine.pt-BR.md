# Quarentena do Microsoft Defender { #microsoft-defender-quarantine }

## O que a quarentena contém { #what-the-quarantine-contains }

A quarentena do Microsoft Defender preserva metadados sobre uma detecção e o conteúdo criptografado dos
recursos colocados em quarentena. O Juntei associa os registros de Entries, ResourceData e Resources para relatar
o rótulo da ameaça, o horário da quarentena, os caminhos lógicos e físicos originais, metadados do arquivo, o proprietário,
o `Zone.Identifier` de origem do download, hashes e problemas de integridade.

Este artefato pode sobreviver aos logs de eventos que originalmente registraram a detecção/remediação. Ele também
pode preservar os bytes exatos do arquivo suspeito para engenharia reversa em ambiente isolado.

<span class="evidence-strength">Melhor conclusão:</span> o Defender criou o registro de quarentena para
a ameaça/o recurso nomeado no horário registrado. O rótulo do produto não é prova independente de malware,
e a quarentena, por si só, não prova execução anterior.

## Localização e coleta { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

Colete todo o diretório `Quarantine`. Uma entrada sem os arquivos de conteúdo/registro correspondentes ainda é
um metadado útil, mas não permite obter os bytes do arquivo nem calcular hashes.

## Segurança e cuidados para o analista { #safety-and-analyst-cautions }

- O conteúdo extraído provavelmente é malicioso. Use um sistema de análise isolado e nunca o execute.
- `ThreatName` é a assinatura/classificação do Defender naquele momento. Falsos positivos e
  reclassificações posteriores são possíveis.
- O horário da quarentena é o horário da remediação/quarentena — não necessariamente do download, da criação ou da execução.
- Um arquivo em quarentena pode ter sido detectado antes da execução ou depois de ter sido executado.
- O caminho original e o Zone.Identifier são valiosos, mas podem estar ausentes ou ser controlados pelo atacante.
- Registros órfãos podem resultar de retenção/limpeza, coleta incompleta ou corrupção.
- O Juntei verifica tamanhos, CRCs, IDs e hashes; revise `Problems` antes de confiar nos bytes recuperados.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

Por exemplo, `tool.exe` se torna `tool.exe.quarantined`. Os fluxos auxiliares incluem seu tipo no
nome, como `tool.exe.Zone.Identifier.quarantined`. O Juntei também remove a permissão de execução.
A extensão e as permissões reduzem a execução acidental, mas os bytes recuperados ainda devem ser
tratados como malware.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Record` | `resource`, entrada sem recursos, `orphan-content` ou `orphan-record`. Os órfãos revelam vínculos/coleta incompletos. |
| `QuarantineTime` | Horário da quarentena/remediação do Defender em UTC, não o horário de execução. |
| `ThreatName`, `ThreatID` | Classificação e ID numérico do Defender. Trate como veredito/contexto do fornecedor. |
| `ResourceType` | Arquivo, chave do Registro, serviço, item de inicialização, tarefa agendada ou outra categoria de recurso. |
| `Path`, `PhysicalPath` | Local lógico/físico original. Pivô principal de correlação; preserve a grafia exata. |
| `FileSize` | Tamanho registrado pelos metadados do recurso. Compare com `ContentSize` e com a MFT. |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | Metadados capturados do recurso em quarentena; distintos do horário da quarentena e vulneráveis a timestomping. |
| `Attr` | Atributos de arquivo registrados. O status oculto/de sistema pode ser um contexto útil. |
| `Owner` | SID do descritor de segurança recuperado; o proprietário não é necessariamente o usuário que executou o arquivo. |
| `ZoneID`, `ReferrerURL`, `HostURL` | Zona de download e valores de URL do `Zone.Identifier`. Pistas fortes de origem quando presentes; o ADS pode estar ausente/ter sido removido. |
| `ContentSize` | Tamanho real do conteúdo principal recuperado. Compare com o `FileSize` registrado. |
| `MD5`, `SHA1`, `SHA256` | Calculados a partir dos bytes recuperados. Use o SHA-256 para identificação moderna de conteúdo; mantenha todos para interoperabilidade. |
| `Streams` | Tipos/nomes/tamanhos dos fluxos recuperados; podem incluir ADS, dados de segurança ou um hive do Registro. |
| `ResourceID` | Identificador que vincula os metadados do recurso aos registros de conteúdo. |
| `EntryID` | GUID da entrada de quarentena; também vincula a evidências do histórico/de eventos do Defender. |
| `ScanID` | Identificador da verificação do Defender, para agrupar recursos/ações. |
| `QuarantineFolder` | Pasta de origem raiz/distinção entre hosts. |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | Arquivos de origem exatos usados para montar a linha. Essenciais para validação. |
| `ExtractedFile` | Arquivo principal em quarentena gravado por `--extract`; vazio quando nenhuma extração principal ocorreu. Os fluxos auxiliares são gravados ao lado dele. `auto-parse -x` o informa a partir do seu diretório de saída (`./HOST1/SecurityProducts/DefenderQuarantined/…`). |
| `Problems` | Divergência de integridade, conteúdo ausente, desacordo de IDs, danos ou contexto de órfão. Não suprima. |

JSON/JSONL preserva o recurso estruturado, a lista de fluxos, as entradas de registro e todos os detalhes de
relacionamento, em vez do texto concatenado do CSV.

## Comportamento da extração { #extraction-behavior }

O Juntei usa o nome de arquivo original com uma extensão final `.quarantined` quando a entrada de quarentena
registra um. Ele usa o identificador de conteúdo para recursos órfãos ou que não são arquivos e acrescenta o
identificador para resolver colisões de nomes de arquivo. Caracteres inválidos em nomes de arquivo do Windows são substituídos. Os fluxos
de segurança e alternativos são gravados ao lado do arquivo principal. Cada fluxo de segurança também recebe um relatório
de texto legível. O Juntei remove a permissão de execução;
conteúdo diferente já existente não é sobrescrito sem `-C`. Esses controles reduzem acidentes, mas não
tornam os bytes seguros.

### Leitura do descritor de segurança recuperado { #reading-the-recovered-security-descriptor }

Um arquivo `.security.quarantined` contém metadados de segurança binários do Windows, portanto não é legível em um
editor de texto. É um fluxo de backup separado, não outra cópia do executável. O Juntei o decodifica automaticamente
durante a extração, tanto com `parse-file defender-quarantine --extract` quanto com `auto-parse -x`:

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

O nome do arquivo de texto segue o nome real do binário, incluindo qualquer identificador de colisão. O texto
gerado é UTF-8 comum e usa as mesmas proteções contra sobrescrita dos demais arquivos extraídos. Ele inclui
o nome do arquivo binário, a contagem de bytes e o SHA-256, para que o relatório possa ser associado à sua origem.

| Campo do relatório | Como interpretá-lo |
|---|---|
| Proprietário e grupo primário | SIDs registrados no descritor. SIDs conhecidos recebem nomes como `BUILTIN\Administrators` ou `NT AUTHORITY\SYSTEM`. Os demais SIDs permanecem sem resolução; um RID familiar, por si só, não basta para nomear uma conta local/de domínio. O proprietário não prova quem criou ou executou um arquivo. |
| Flags de controle | Flags numéricas originais e seus nomes, incluindo presença de ACL, herança e proteção. |
| DACL | Entradas de controle de acesso (ACEs) em ordem: tipo permitir/negar, SID da conta, máscara de permissões, direitos e flags de herança. São entradas registradas, não um cálculo do acesso efetivo para o token de um usuário. |
| SACL | Entradas registradas de auditoria ou de integridade obrigatória. As flags de auditoria de sucesso/falha descrevem a configuração, não um histórico de acessos. A ausência de SACL não estabelece que toda a auditoria estava desabilitada. |
| GUIDs de objeto | GUIDs opcionais para ACEs específicas de objeto, com os direitos específicos de objeto mantidos em forma numérica em vez de rotulados incorretamente como direitos de arquivo. |
| SDDL | Representação em Security Descriptor Definition Language do proprietário, do grupo e das entradas de ACL suportadas. Consulte as flags de controle separadamente. O Juntei não emite uma string SDDL parcial se algum componente estiver danificado ou não for suportado. |
| Status da decodificação e problemas | `complete` ou `incomplete`, com os motivos. ACEs não suportadas mantêm seu hex bruto no relatório. O binário original é sempre mantido quando a extração é bem-sucedida. Os problemas de decodificação também aparecem no campo `Problems` do registro de quarentena. |

Uma DACL ausente, uma DACL NULL e uma DACL vazia são relatadas separadamente. Uma DACL ausente ou NULL não impõe
nenhuma restrição de acesso discricionário; uma DACL vazia não tem ACEs que concedam acesso. Outras verificações de acesso
e privilégios do Windows continuam relevantes. ACEs herdadas e somente de herança permanecem explicitamente sinalizadas.

Por exemplo, uma entrada que concede controle total herdado aos Administradores pode aparecer como:

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

O decodificador funciona offline no Windows, no macOS e no Linux, sem APIs do Windows, Python ou consulta de contas.
Ele decodifica ACEs básicas de permissão/negação/auditoria/alarme, ACEs de objeto e rótulos de integridade obrigatória. ACEs condicionais,
de callback, de atributos de recurso e outras não suportadas são identificadas explicitamente e preservadas em
forma bruta; suas condições de acesso não são avaliadas. Um descritor danificado não interrompe a extração
dos demais fluxos.

## Estratégia de investigação { #investigation-strategy }

1. Comece por `ThreatName`, `Path`, `QuarantineTime`, `EntryId` e os hashes.
2. Valide `Problems` e compare o tamanho registrado com o conteúdo recuperado.
3. Pivote o SHA-256 para outros hosts, EDR, sandbox e dados de inteligência de ameaças.
4. Correlacione o caminho/horário original com `$MFT`, Diário USN, Prefetch, Amcache e logs de eventos.
5. Use os campos de URL/zona para investigar o acesso inicial, mas valide com evidências de navegador/proxy/DNS.

## Mais detalhes { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [defender da Sootmark](https://github.com/Sootmark/defender)
- [Plugin Defender do dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: remediação e quarentena do Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft: descritores de segurança](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: formato de string do descritor de segurança](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: strings de ACE](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
