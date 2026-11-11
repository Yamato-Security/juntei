# Amcache { #amcache }

## O que é o Amcache { #what-amcache-is }

O Amcache é um inventário de compatibilidade de aplicativos do Windows armazenado em um hive do Registro. Sistemas modernos
registram arquivos executáveis em `InventoryApplicationFile` e dispositivos Plug and Play em
`InventoryDevicePnp`; versões mais antigas do Windows usam um layout legado `Root\File`.

Na investigação de executáveis, o Amcache é valioso porque pode preservar o caminho completo, o SHA-1,
o tamanho do arquivo, metadados PE, informações de versão, o texto do editor, a data de link e horários do Registro
depois que o arquivo original desapareceu. Os registros de dispositivos podem identificar hardware conectado, drivers,
datas de instalação e identificadores.

<span class="evidence-strength">Melhor conclusão:</span> o Windows inventariou o arquivo ou
dispositivo registrado. Um registro de executável é **evidência de presença, não prova de execução**.

## Localização e coleta { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

Colete o hive e os arquivos adjacentes `Amcache.hve.LOG`, `.LOG1` e `.LOG2`. Se o hive estiver sujo, `parse-registry amcache` reaplica automaticamente em memória os logs de transações utilizáveis antes da análise, sem modificar as evidências. Os detalhes aparecem no log do parser ou no terminal; ao enviar registros para stdout, os diagnósticos vão para stderr. Logs ausentes ou inutilizáveis podem deixar registros incompletos ou impedir a leitura de um hive inconsistente.

## Cuidados para o analista { #analyst-cautions }

- Uma entrada de executável no Amcache não prova que um usuário iniciou o arquivo.
- `Publisher` são metadados PE/de recurso de versão registrados pelo Windows. Não provam que o arquivo tinha
  uma assinatura Authenticode válida no momento da coleta.
- Os hashes podem estar ausentes, e os esquemas legado e moderno não expõem valores idênticos.
- `LinkDate` vem do cabeçalho PE e pode ser forjado, copiado ou produzido por builds reproduzíveis.
- O horário de última gravação de uma chave descreve a atualização do registro no Registro, não necessariamente o download, a criação,
  a primeira execução ou a última execução.
- Os caminhos podem se referir a volumes removíveis ou que não estão mais presentes.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

A saída CSV contém as tabelas de resumo, achados, arquivos e dispositivos. Com `auto-parse`, elas são
`Registry-Amcache-Summary.csv`, `Registry-Amcache-Findings.csv`, `Registry-Amcache-Files.csv` e
`Registry-Amcache-Devices.csv`, diretamente no diretório `FileExistence/` de cada computador, com
`Registry-Amcache-logs.txt` ao lado das tabelas. Vários hives acrescentam `-Source1`, `-Source2` etc.
antes da extensão; `OutputFiles` nos manifestos lista os caminhos exatos. As exportações CSV
independentes mantêm seus nomes de tabela nativos e o local de saída solicitado. Os registros JSONL usam
os valores de `RecordType` `summary`, `finding`, `file` e `device`.

## Campos de saída de arquivos { #file-output-fields }

| Campo | Como o analista deve usá-lo |
|---|---|
| `KeyName`, `SourceKeyPath` | Identificam o registro exato de origem no Registro e o ramo do esquema. Preserve para garantir a reprodutibilidade. |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | Horário de atualização do registro no Registro, em formato legível e bruto. Não o rotule como horário de execução. |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | Horário da chave de volume pai nos esquemas legados; contextual, não é o horário de uma ação sobre o arquivo. |
| `FileID` | Identificador de arquivo registrado pelo Amcache. Mantido exatamente como está; não presuma que seja sempre um SHA-1. |
| `SHA1`, `SHA256` | Pivôs fortes para Defender, inteligência de ameaças, EDR e repositórios de arquivos. Nulo significa não registrado. |
| `Name`, `OriginalFileName`, `Description` | Nomes PE/de versão registrados. Compare com o caminho real para detectar mascaramento; os metadados são controlados pelo atacante. |
| `FullPath` | Pivô de localização mais útil. Normalize maiúsculas/minúsculas com cuidado, mas mantenha a string de evidência original. |
| `LongPathHash` | Identificador do Windows relacionado ao caminho; útil para correlacionar registros, não é um hash de conteúdo. |
| `ProgramID` | Vincula o arquivo a um registro de inventário de aplicativos, quando disponível. |
| `Publisher` | Texto do editor registrado. Trate como metadado, não como validação de assinatura. |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | Contexto do recurso de versão. Diferenças ou valores implausíveis podem sustentar a análise de mascaramento. |
| `BinaryType` | Contexto de arquitetura/tipo registrado; compare com a análise PE quando o arquivo for recuperado. |
| `LinkDate` | Horário de compilação/link do PE conforme registrado. Útil apenas com ceticismo e corroboração. |
| `Size` | Tamanho registrado; compare com o `$MFT`, o conteúdo do Defender ou um arquivo recuperado. |
| `Language` | Identificador de idioma do PE; uma pista contextual fraca, não atribuição. |
| `IsPE-File`, `IsOS-Component` | Flags de classificação registradas pelo Windows. Um valor falso não prova que o arquivo é inofensivo. |
| `USN` | Valor de sequência de atualização do NTFS registrado pelo Amcache; pode ajudar a pivotar para registros USN remanescentes. |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | Associa aplicativos/componentes empacotados ao arquivo. |
| `RegistryValues` | Todos os valores originais do Registro, com tipo, valor decodificado e hex bruto. Use quando um campo tipado estiver ausente ou para campos futuros/legados. |

## Campos de saída de dispositivos { #device-output-fields }

| Grupo de campos | Campos e interpretação |
|---|---|
| Origem | `KeyName`, `SourceKeyPath`, `KeyLastWrittenUTC`, `KeyLastWrittenFiletime`: registro exato do dispositivo e horário de atualização no Registro. |
| Identidade | `Description`, `BusDescription`, `Manufacturer`, `Model`: descrições do dispositivo voltadas ao usuário; as strings do fabricante podem ser genéricas. |
| Identificadores de hardware | `HWID`, `CompID`, `Class`, `ClassGUID`, `ContainerID`, `DeviceInterfaceClasses`: os pivôs mais fortes para família e instância do dispositivo. |
| Estado | `DeviceState`, `ProblemCode`, `ParentID`, `MatchingID`, `Enumerator`: contexto de enumeração e de relacionamentos do Windows. |
| Driver | `DriverID`, `DriverName`, `DriverPackageStrongName`, `DriverVerDate`, `DriverVerVersion`, `Provider`, `Service`, `INF`: pacote de driver e associações de serviço; correlacione com arquivos e serviços instalados. |
| Instalação | `InstallDate`, `FirstInstallDate`, `InstallState`, `ExtendedInfs`: ciclo de vida de instalação registrado. A ausência de data não prova que o dispositivo nunca foi conectado. |
| Filtros e pilha | `LowerClassFilters`, `UpperClassFilters`, `LowerFilters`, `UpperFilters`, `StackID`, `LocationPaths`: contexto avançado da pilha de drivers; drivers de filtro inesperados merecem revisão. |
| Preservação bruta | `RegistryValues`: valores de origem completos, tipados/brutos, para validação e campos não suportados. |

As colunas de arrays e de valores brutos no CSV usam o sufixo `JSON` (por exemplo, `HWID-JSON` e
`RegistryValuesJSON`); suas células contêm JSON, enquanto JSON/JSONL mantêm arrays nativos.

## Estratégia de investigação { #investigation-strategy }

1. Pivote em `FullPath`, `SHA1`, `Name` e `USN`.
2. Procure o Prefetch correspondente para sustentar a execução.
3. Use o `$MFT` e o Diário USN para estabelecer criação, renomeação, exclusão e tamanho do arquivo.
4. Compare os metadados de editor/versão com um PE recuperado de forma independente e com a validação da assinatura.
5. Verifique a quarentena do Defender e os logs de eventos em busca de detecção ou remoção.

## Mais detalhes { #further-details }

- [Esquema de saída do ys-amcache-parser do Juntei](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [amcache-forensic da Security Ronin](https://github.com/SecurityRonin/amcache-forensic)
- [Plugin Amcache do Regipy](https://github.com/mkorman90/regipy)
