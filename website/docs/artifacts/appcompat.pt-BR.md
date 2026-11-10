# Compatibilidade de aplicativos { #application-compatibility }

## O que o Windows mantém { #what-windows-keeps }

Além do ShimCache (`parse-registry shimcache`) e do Amcache (`parse-registry amcache`),
os recursos de compatibilidade de aplicativos do Windows mantêm os seguintes dados em
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags`:

| Fonte | Hive e chave | O que mostra |
|---|---|---|
| `layers` | `SOFTWARE` (e `Wow6432Node`) e `NTUSER.DAT` `Layers` | Programas configurados para executar com camadas de compatibilidade: `RUNASADMIN` (sempre como administrador), `WIN7RTM`, `HIGHDPIAWARE`, … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 e posteriores) | Programas que o Assistente de Compatibilidade de Programas viu o usuário executar |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | O mesmo, no Windows 7 |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | Programas que carregaram um módulo rastreado e quando o fizeram pela última vez |
| `cit_system` | `SOFTWARE` `CIT\System` (do Windows 7 às primeiras versões do Windows 10) | Caminhos de programas e linhas de comando nos bancos de dados CIT compactados |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | Bancos de dados de shim personalizados instalados com `sdbinst`: arquivo, descrição, horário de instalação |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | Os bancos de dados de shim personalizados aplicados a cada programa, com seus horários de instalação |

<span class="evidence-strength">Melhor conclusão:</span> camadas e bancos de dados de shim são
configurações que alteram a forma como um programa é executado; as entradas do Assistente de Compatibilidade de Programas e do CIT
mostram que um programa foi executado (ou carregou um módulo) como este usuário ou neste sistema.

Bancos de dados de shim personalizados são uma técnica de persistência e injeção (MITRE ATT&CK T1546.011): um shim
pode carregar uma DLL em um programa ou iniciar outro sempre que o programa é executado.

## Localização { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

Colete o `SOFTWARE`, todos os `NTUSER.DAT` e seus logs de transação, além dos arquivos `.sdb` cujos caminhos
os bancos de dados de shim nomeiam (geralmente `C:\Windows\AppPatch\Custom\`).

## Cuidados para o analista { #analyst-cautions }

- **A expansão do CIT é limitada.** Valores que declaram mais de 64 MiB descompactados são mantidos
  como bytes brutos, com um diagnóstico. Compactação malformada e divergências de tamanho são relatadas.


- **Não há horário de execução nas entradas do Assistente de Compatibilidade de Programas.** Os dados de `Store` (`SACP`, mantidos em
  `DataRaw`) têm um horário, mas no Windows 8.1, 10 e 11 ele é o mesmo para todos os programas de um
  hive, assim como em outros sistemas: não é o momento em que o programa foi executado. Somente a última gravação da chave `Store`
  data a alteração mais recente.
- **Programas em mídia removível** são nomeados `SIGN.MEDIA=<id> <path>`: `Path` é o caminho na
  mídia, e `Details` contém o ID da mídia.
- **Os bancos de dados CIT** são compactados com LZNT1 e sua estrutura não é documentada publicamente: o Juntei
  lista os caminhos de programas e as linhas de comando encontrados em cada um, uma vez por banco de dados, sem contagens nem
  horários. Uma linha de comando é um registro próprio, não vinculado ao seu programa. O Windows 10 e posteriores mantêm
  pouca coisa aqui.
- **Bancos de dados de shim.** Verifique todos os registros `custom_sdb` e `installed_sdb`: softwares legítimos
  raramente instalam um. O próprio arquivo `.sdb` informa o que o shim faz.
- **Camadas** definidas por um usuário se aplicam somente a esse usuário; as do `SOFTWARE`, a todos. `RUNASADMIN` faz um
  programa solicitar elevação toda vez.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

O resumo lista os bancos de dados de shim personalizados, os programas com camadas e os últimos usos dos
módulos CIT.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, arquivo do hive e `SOFTWARE` ou `NTUSER.DAT`. |
| `User`, `UserSID` | Nome da pasta de perfil de um hive de usuário e seu SID obtido do `ProfileList` do sistema. |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb`; `machine` ou `user`. |
| `Path` | O programa (o nome do valor), um caminho ou linha de comando em um banco de dados CIT ou o programa ao qual um banco de dados de shim se aplica. |
| `Layers` | As camadas de compatibilidade, como foram gravadas (`~ RUNASADMIN`). |
| `LastSeenUTC` | Quando um programa carregou um módulo CIT pela última vez. |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | GUID, arquivo e horário de instalação de um banco de dados de shim. |
| `Details` | Nome de um módulo CIT, nome do valor de um banco de dados CIT, ID da mídia removível de um programa (`SignMedia`), `DatabaseDescription` e `DatabaseType` de um banco de dados de shim. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | De onde vem o registro, quando essa chave foi alterada pela última vez e se está sob `Wow6432Node`. |
| `DataRaw` | Os dados `SACP` do valor `Store`, ou dados que não são o que deveriam ser (hex). |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Dados que não puderam ser decodificados ou lidos. |

## Estratégia de investigação { #investigation-strategy }

1. Revise todos os bancos de dados de shim personalizados: o arquivo `.sdb`, o horário de instalação e os programas aos quais
   se aplicam; correlacione com `sdbinst.exe` no Prefetch, no Amcache e nos eventos de criação de processos.
2. Procure, entre as entradas do Assistente de Compatibilidade de Programas, programas em pastas graváveis pelo usuário
   e programas executados a partir de mídia removível.
3. Verifique `RUNASADMIN` e outras camadas em programas incomuns.
4. Correlacione com ShimCache, Amcache, BAM, Prefetch e UserAssist para obter os horários de execução.

## Mais detalhes { #further-details }

- [Microsoft: banco de dados de compatibilidade de aplicativos](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
