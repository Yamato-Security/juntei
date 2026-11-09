# Lista de comandos { #command-list }

Os analisadores individuais são agrupados pelo tipo de fonte. Execute `juntei --help`,
`juntei <SOURCE> --help` ou `juntei <SOURCE> <ARTIFACT> --help` para explorá-los:

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Para uma coleta de evidências do Windows com vários computadores, use
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md). Ele invoca os mesmos comandos agrupados
e organiza os resultados e os logs por computador e por categoria investigativa.

## [Imagens de disco](mount.md) { #disk-images }

| Comando | Entrada | Guia detalhado |
|---|---|---|
| `mount` | Imagem de disco VHDX ou E01 | [Montar imagens de disco](mount.md) |

## [Comandos ESE](user-activity.md) { #ese-commands }

| Comando | Artefato de entrada | Guia detalhado |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [Registro de acesso do usuário (UAL)](../artifacts/ual.md) |
| `parse-ese webcache` | ESE `WebCacheV01.dat` | [Internet Explorer e Edge legado](../artifacts/webcache.md) |

## [Comandos de arquivos](execution.md) { #file-commands }

| Comando | Artefato de entrada | Guia detalhado |
|---|---|---|
| `parse-file defender-quarantine` | Árvore ou entrada de quarentena do Defender | [Quarentena do Defender](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [Atalhos LNK](../artifacts/lnk.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [Comandos NTFS](filesystem.md) { #ntfs-commands }

| Comando | Artefato de entrada | Guia detalhado |
|---|---|---|
| `parse-ntfs boot` | `$Boot`, ou uma imagem de volume NTFS | [Setor de inicialização ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`), ou uma imagem de disco | [Índices de diretórios ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | `$LogFile` do NTFS | [Log de transações NTFS](../artifacts/logfile.md) |
| `parse-ntfs mft` | `$MFT` do NTFS | [Tabela Mestre de Arquivos](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [Descritores de segurança ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [Diário USN](../artifacts/usn-journal.md) |

## [Comandos do Registro](security-registry.md) { #registry-commands }

| Comando | Artefato de entrada | Guia detalhado |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [Redes](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [Segredos LSA](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (e `BCD-Template`) | [Dados de configuração de inicialização (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [Compatibilidade de aplicativos](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [Política de auditoria](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage, RecentApps e itens fixados na barra de tarefas](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [Histórico de aplicativos](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [Listas MRU](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (com `SAM`, `SOFTWARE` para nomes de usuário) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (`NTUSER.DAT` no Windows XP) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [Programas instalados](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [Caixas de diálogo Abrir e Salvar como](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [Inicializações automáticas do Registro](../artifacts/persistence.md) |
| `parse-registry read` | Qualquer hive primário do Registro | [Hives do Registro](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (com `SOFTWARE` para os SIDs de usuário) | [Acesso remoto (lado do usuário)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [Contas locais SAM](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [Configurações de segurança](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Serviços do Windows](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` ou um valor AppCompatCache exportado | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` de um sistema | [Informações do sistema](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Cache do Agendador de Tarefas](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`, com `SOFTWARE` e `NTUSER.DAT` | [Dispositivos USB](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## [Comandos SQLite](sqlite.md) { #sqlite-commands }

| Comando | Artefato de entrada | Guia detalhado |
|---|---|---|
| `parse-sqlite browser` | `History`, `Cookies`, `Web Data`, `places.sqlite`, `cookies.sqlite`, `formhistory.sqlite` | [Navegadores web](../artifacts/browser.md) |

## Tipos de fonte e mudanças de formato { #source-types-and-format-changes }

Os grupos descrevem os formatos que o Juntei suporta atualmente. `parse-registry amcache` lê o
`Amcache.hve`; isso não implica suporte aos bancos de dados de inventário Amcache em SQLite. Os crates de analisadores
de artefatos permanecem separados para que o suporte a outro formato de armazenamento possa ser adicionado de forma independente.

Os antigos nomes planos de analisadores foram substituídos pelos caminhos de fonte e artefato acima. Mantenha as
opções existentes do analisador após o nome do artefato. `auto-parse` e os subcomandos de Registro existentes
mantêm seus nomes.

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` faz buscas em vários formatos de fonte, por isso é um comando de análise de
nível superior. Ele complementa o inventário completo de configurações de `parse-registry persistence` com
131 verificações, classificação por lista de permissão/bloqueio, metadados da carga útil, cobertura e correlação de carimbos de data/hora.

## Uso no nível superior { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX or E01 disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  parse-sqlite        Parse SQLite databases
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

As páginas de fontes reproduzem o uso dos comandos e dão exemplos curtos. Os guias de artefatos explicam
as evidências, os campos de saída, as limitações e a estratégia de investigação.
