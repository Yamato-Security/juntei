# Comandos do Registro { #registry-commands }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry <COMMAND> [OPTIONS]

Commands:
  amcache             Parse Amcache inventories
  app-compat          Parse application compatibility data
  app-history         Parse third-party tools' traces
  auditpol            Parse the audit policy
  bam                 Parse BAM and DAM last runs
  bcd                 Parse boot configuration
  feature-usage       Parse Explorer use counts and pins
  help                Print this message or the help of the given subcommand(s)
  installed-programs  Parse installed programs
  lsa-secrets         Parse LSA secrets' names and times
  mru                 Parse most-recently-used lists
  muicache            Parse MuiCache program names
  networks            Parse network profiles, interfaces, shares, port proxies and routes
  office              Parse Office recent files, trusted documents and macro settings
  open-save-mru       Parse Open and Save As dialog history
  persistence         Parse autostart locations
  read                Read keys and values
  recent-docs         Parse recent documents
  remote-access       Parse outbound RDP, PuTTY, WinSCP and mapped drive connections
  sam                 Parse local accounts
  security-settings   Parse security settings: Defender, UAC, credentials, RDP, firewall, logging
  services            Parse services
  shell-bags          Parse ShellBags (folders opened in Explorer)
  shimcache           Parse ShimCache entries
  system-info         Parse system information
  taskcache           Parse scheduled tasks
  usb                 Parse USB devices and volumes
  user-assist         Parse UserAssist entries

Options:
  -h, --help  Print help
```

Os comandos canônicos são `app-compat`, `feature-usage`, `open-save-mru`, `recent-docs`,
`user-assist` e `shell-bags`. Suas grafias anteriores (`appcompat`, `featureusage`,
`opensavemru`, `recentdocs`, `userassist` e `shellbags`) continuam aceitas como aliases ocultos
para scripts existentes. A análise automática usa os nomes canônicos; seus identificadores de artefatos,
esquemas de resultados e nomes de arquivos de saída não mudam.

Os comandos do Registro aceitam `-f <HIVE>` ou `-d <COLLECTION>` e opções de saída estruturada.
Atualmente, `amcache` lê hives do Registro do Amcache sem reproduzir os logs de transação. `shimcache`
reproduz os logs do SYSTEM adjacentes por padrão e também aceita um valor AppCompatCache bruto exportado.
Os demais comandos do Registro reproduzem os logs adjacentes automaticamente; use `--log1` / `--log2` para indicar
os logs explicitamente ou `--no-logs` para inspecionar o hive primário sem reprodução.

## `parse-registry amcache` { #parse-registry-amcache }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Amcache hives (Amcache.hve)
  -f, --file <FILE>      File path to one Amcache.hve

Records:
      --files    Include every inventoried executable (InventoryApplicationFile)
      --devices  Include every inventoried PnP device (InventoryDevicePnp)
  -a, --all      Include executables and devices (default for -o and for json, jsonl, csv)

Output:
  -o, --output <PATH>       Save the results to a file (text, json, jsonl) or a directory (csv)
  -t, --output-type <TYPE>  Output type: csv (default with -o), json, jsonl, text (default on the terminal)
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-registry amcache -f Amcache.hve --files
juntei parse-registry amcache -d evidence/ -o amcache-csv
juntei parse-registry amcache -f Amcache.hve -t jsonl | jq 'select(.RecordType == "file")'
```

Consulte [Amcache](../artifacts/amcache.md).

## `parse-registry bam` { #parse-registry-bam }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives
  -f, --file <FILE>      File path to one SYSTEM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Os usuários são nomeados a partir do `SAM` e do `SOFTWARE` (`ProfileList`) do sistema, localizados da mesma forma que o host.
Consulte [BAM / DAM](../artifacts/bam.md).

## `parse-registry installed-programs` { #parse-registry-installed-programs }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      File path to one SOFTWARE or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Entradas `Uninstall`, produtos do Windows Installer, App Paths, distribuições WSL e aplicativos empacotados.
O usuário de um hive de usuário vem da sua pasta de perfil e do `ProfileList` do sistema. Consulte
[Programas instalados](../artifacts/installed-programs.md).

## `parse-registry muicache` { #parse-registry-muicache }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      File path to one UsrClass.dat (Windows Vista and later) or NTUSER.DAT (Windows XP) hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Os usuários são nomeados a partir da posição do hive na coleta, e seus SIDs a partir da chave raiz de um `UsrClass.dat`
ou do `SOFTWARE` (`ProfileList`) do sistema. Consulte [MuiCache](../artifacts/muicache.md).

## `parse-registry open-save-mru` { #parse-registry-open-save-mru }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (8.3 names, network shares): the system's ANSI code page (932 for Japanese) [default: 1252]

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Cada lista de itens do shell é decodificada com o ys-lnk-parser e unida em um caminho. Consulte
[Caixas de diálogo Abrir e Salvar como](../artifacts/opensavemru.md).

## `parse-registry remote-access` { #parse-registry-remote-access }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Os SIDs dos usuários vêm do `SOFTWARE` (`ProfileList`) do sistema, localizado da mesma forma que o host. Senhas
armazenadas são relatadas como presentes, nunca decodificadas. Consulte
[Acesso remoto (lado do usuário)](../artifacts/remote-access.md).


## `parse-registry system-info` { #parse-registry-system-info }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM, SOFTWARE, SAM and SECURITY hives (those in one directory are one system's)
  -f, --file <FILE>      File path to a SYSTEM, SOFTWARE, SAM or SECURITY hive of one system; repeat for each (the others are looked for next to them)
      --no-logs          Read dirty hives as they are, without their transaction logs (default: <hive>.LOG, .LOG1 and .LOG2 next to each are replayed)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Um registro por sistema: seus nomes, domínio ou grupo de trabalho, versão e data de instalação do Windows, fuso
horário, último desligamento, último usuário, perfis e SIDs. Os hives de um diretório pertencem a um único sistema;
com `-f`, os hives informados e os demais dos quatro encontrados ao lado deles.
Consulte [Informações do sistema](../artifacts/system-info.md).


## `parse-registry mru` { #parse-registry-mru }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --list <LISTS>  Only these lists (comma separated): runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Consulte [Listas de itens usados mais recentemente (MRU)](../artifacts/mru.md).


## `parse-registry app-history` { #parse-registry-app-history }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Ferramentas Sysinternals, histórico do WinRAR, do 7-Zip e do WinZip, contas do OneDrive, arquivos recentes do Adobe,
TeamViewer e CCleaner. Consulte [Histórico de aplicativos](../artifacts/app-history.md).


## `parse-registry feature-usage` { #parse-registry-feature-usage }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry feature-usage <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Lê `FeatureUsage`, `RecentApps`, `Taskband` e `ProgramsCache`. Consulte
[FeatureUsage, RecentApps e itens fixados na barra de tarefas](../artifacts/featureusage.md).


## `parse-registry security-settings` { #parse-registry-security-settings }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      File path to one SOFTWARE, SYSTEM or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive (ControlSet001, ControlSet002, ...), not only the one Windows used last
      --weakened-only     Only settings whose value weakens what Windows does by default

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Cada configuração encontrada é um registro com seu valor, seu significado e se ela enfraquece o que o Windows
faz por padrão; `--weakened-only` mantém apenas as que enfraquecem. Consulte
[Configurações de segurança](../artifacts/security-settings.md).


## `parse-registry office` { #parse-registry-office }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Consulte [Microsoft Office](../artifacts/office.md).

### Exemplos { #examples }

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --service ExampleSvc -t jsonl
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry office -d collection/ -o office.csv
juntei parse-registry taskcache -f SOFTWARE --task ScheduledDefrag -t jsonl
juntei parse-registry sam -f SAM --rid 500 -t jsonl
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry user-assist -d collection/ -o userassist.csv
```

As opções completas e a semântica dos campos estão em [Hives do Registro](../artifacts/registry.md),
[Serviços do Windows](../artifacts/services.md), [RecentDocs](../artifacts/recentdocs.md),
[Microsoft Office](../artifacts/office.md),
[Cache do Agendador de Tarefas](../artifacts/taskcache.md), [Contas locais SAM](../artifacts/sam.md),
[Inicializações automáticas do Registro](../artifacts/persistence.md) e [UserAssist](../artifacts/userassist.md).


## `parse-registry auditpol` { #parse-registry-auditpol }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SECURITY hives
  -f, --file <FILE>      File path to one SECURITY hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Um registro por subcategoria (Windows Vista e posteriores) ou categoria (XP e 2003) da política de
auditoria em `Policy\PolAdtEv`: o que ela registra, seu GUID e ID de mensagem, e seus principais eventos. Consulte
[Política de auditoria](../artifacts/auditpol.md).


## `parse-registry app-compat` { #parse-registry-app-compat }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      File path to one SOFTWARE or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Camadas de compatibilidade, `Store` e `Persisted` do Assistente de Compatibilidade de Programas, CIT e bancos de dados
de shim personalizados. O ShimCache fica a cargo de `parse-registry shimcache`. Consulte
[Compatibilidade de aplicativos](../artifacts/appcompat.md).


## `parse-registry bcd` { #parse-registry-bcd }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for BCD hives (and BCD-Template)
  -f, --file <FILE>      File path to one BCD hive (\Boot\BCD, \EFI\Microsoft\Boot\BCD)
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Um registro por objeto do repositório, com assinatura de teste, verificações de integridade, recuperação, política de status
de inicialização e modo de segurança resolvidos por meio de herança. Consulte
[Dados de configuração de inicialização (BCD)](../artifacts/bcd.md).


## `parse-registry lsa-secrets` { #parse-registry-lsa-secrets }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SECURITY hives
  -f, --file <FILE>      File path to one SECURITY hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Um registro por segredo LSA (`Policy\Secrets\<name>`) com seu tipo, o serviço ou usuário que ele
nomeia, quando seus valores foram definidos e se contêm dados, e um para os logons em cache
(`Cache`). Apenas nomes e horários: nada é descriptografado. Consulte [Segredos LSA](../artifacts/lsa-secrets.md).


## `parse-registry networks` { #parse-registry-networks }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE and SYSTEM hives
  -f, --file <FILE>      File path to one SOFTWARE or SYSTEM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Os horários dos perfis são convertidos para UTC com o fuso horário do hive `SYSTEM` do sistema, localizado da mesma forma que o
host, e também mantidos como encontrados. Consulte [Redes](../artifacts/networks.md).


## `parse-registry persistence` { #parse-registry-persistence }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      File path to one SOFTWARE, SYSTEM, NTUSER.DAT or UsrClass.dat hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry read` { #parse-registry-read }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for registry hives (every file that is one, logs aside)
  -f, --file <FILE>      File path to one registry hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
  -k, --key <PATH>             Only the key at this path (in any case; ex: Microsoft\Windows\CurrentVersion\Run). It may start with HKLM\ and the hive's name, and CurrentControlSet is the control set Windows used last
      --key-regex <REGEX>      Only keys whose path matches REGEX (ignoring case)
      --value <NAME>           Only the value named NAME (in any case; "" is the key's default value). With --key or --key-regex, a key without it gives an absent_value record
      --value-regex <REGEX>    Only values whose name matches REGEX (ignoring case; the default value's name is empty)
      --data-regex <REGEX>     Only values whose data, as written, matches REGEX (ignoring case, so a GUID matches in any case; binary data in hex)
      --timeline-start <DATE>  Only keys last written on or after DATE, in UTC unless an offset is given (ex: 2025-10-19, 2025-10-19T13:45:30Z)
      --timeline-end <DATE>    Only keys last written on or before DATE
      --recover-deleted        Also deleted keys and values: in free cells, the slack of cells in use and the data past the hive bins, and what only the transaction logs kept (allocation_status says how complete; recovered_from where found)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
      --raw-data            Write every value's bytes in hex (value_data_raw), not only those that don't decode as their type
      --qword-as-filetime   Also write REG_QWORD values as FILETIMEs (value_filetime_utc), keeping the number (value_data, value_u64)
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-registry read -f SYSTEM --key 'CurrentControlSet\Services' -o services-raw.csv
juntei parse-registry read -f NTUSER.DAT --recover-deleted -t jsonl | jq .
```

## `parse-registry recent-docs` { #parse-registry-recent-docs }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry sam` { #parse-registry-sam }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SAM hives
  -f, --file <FILE>      File path to one SAM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --rid <RID>  Only the account with this RID (ex: 500, the built-in Administrator)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry services` { #parse-registry-services }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives
  -f, --file <FILE>      File path to one SYSTEM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --service <NAME>    Only the service of this name (its key's name, in any case)
      --all-control-sets  Every control set (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry shell-bags` { #parse-registry-shell-bags }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      File path to one UsrClass.dat or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Parsing:
      --code-page <NUMBER>  Windows code page for the ANSI names of shell items (8.3 names, network shares) [default: 1252]

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry shimcache` { #parse-registry-shimcache }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives
  -f, --file <FILE>      File path to one SYSTEM hive, or an AppCompatCache value saved to a file
      --no-logs          Read a dirty hive as it is, without its transaction logs (SYSTEM.LOG1, SYSTEM.LOG2)

Output:
  -o, --output <FILE>       Save every entry to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, the entries are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-registry shimcache -f SYSTEM
juntei parse-registry shimcache -d evidence/ -o shimcache.csv
```

Consulte [ShimCache](../artifacts/shimcache.md).

## `parse-registry taskcache` { #parse-registry-taskcache }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE hives
  -f, --file <FILE>      File path to one SOFTWARE hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --task <NAME>  Only the task of this name, in any case: its whole name (\Microsoft\Windows\Defrag\ScheduledDefrag) or its last part (ScheduledDefrag)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry usb` { #parse-registry-usb }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives (each system's SOFTWARE and users' NTUSER.DAT hives are read with it)
  -f, --file <FILE>      File path to one SYSTEM hive (its system's SOFTWARE and users' NTUSER.DAT hives are read with it)
      --log1 <FILE>      The SYSTEM hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The SYSTEM hive's second transaction log (.LOG2), for -f
      --no-logs          Read dirty hives as they are, without their transaction logs

Filtering:
      --all-control-sets  Every control set (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry user-assist` { #parse-registry-user-assist }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry user-assist <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### Exemplos { #examples_1 }

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --service ExampleSvc -t jsonl
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry taskcache -f SOFTWARE --task ScheduledDefrag -t jsonl
juntei parse-registry sam -f SAM --rid 500 -t jsonl
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry usb -d collection/ -o usb.csv
juntei parse-registry user-assist -d collection/ -o userassist.csv
juntei parse-registry open-save-mru -d collection/ -o opensavemru.csv
juntei parse-registry bam -d collection/ --all-control-sets -o bam.csv
juntei parse-registry installed-programs -d collection/ -o installed.csv
juntei parse-registry muicache -d collection/ -o muicache.csv
juntei parse-registry shell-bags -d collection/ -o shellbags.csv
```

As opções completas e a semântica dos campos estão em [Hives do Registro](../artifacts/registry.md),
[Serviços do Windows](../artifacts/services.md), [RecentDocs](../artifacts/recentdocs.md),
[Cache do Agendador de Tarefas](../artifacts/taskcache.md), [Contas locais SAM](../artifacts/sam.md),
[Inicializações automáticas do Registro](../artifacts/persistence.md), [UserAssist](../artifacts/userassist.md),
[BAM / DAM](../artifacts/bam.md), [Programas instalados](../artifacts/installed-programs.md),
[MuiCache](../artifacts/muicache.md),
[ShellBags](../artifacts/shellbags.md) e
[Dispositivos USB](../artifacts/usb.md).

Consulte também [Caixas de diálogo Abrir e Salvar como](../artifacts/opensavemru.md).

Consulte também [remote-access](../artifacts/remote-access.md).

Consulte também [system-info](../artifacts/system-info.md).

Consulte também [mru](../artifacts/mru.md).

Consulte também [app-history](../artifacts/app-history.md).

Consulte também [featureusage](../artifacts/featureusage.md).

Consulte também [security-settings](../artifacts/security-settings.md).

Consulte também [office](../artifacts/office.md).

Consulte também [auditpol](../artifacts/auditpol.md).

Consulte também [appcompat](../artifacts/appcompat.md).

Consulte também [bcd](../artifacts/bcd.md).

Consulte também [lsa-secrets](../artifacts/lsa-secrets.md).

Consulte também [networks](../artifacts/networks.md).
