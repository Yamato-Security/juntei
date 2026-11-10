# Coleta de evidências { #collecting-evidence }

Juntei analisa arquivos já coletados do Windows. Use uma ferramenta de aquisição confiável e preserve os metadados originais quando possível.

[auto-parse --vhdx](../commands/auto-parse.md#vhdx-disk-images): Linux / macOS / Windows.

## Localizações comuns { #common-locations }

| Artefato | Localização típica |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | Arquivo de metadados do volume NTFS `C:\$MFT` |
| `$LogFile` | Arquivo de metadados do volume NTFS `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| Diário USN | Fluxo de metadados do NTFS `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| Registro de acesso do usuário (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| Internet Explorer e Edge legado (WebCache) | `%LOCALAPPDATA%\Microsoft\Windows\WebCache\` |
| Outlook | `%LOCALAPPDATA%\Microsoft\Outlook\*.ost`, `%USERPROFILE%\Documents\Outlook Files\*.pst` (`%LOCALAPPDATA%\Microsoft\Outlook\*.pst` antes do Outlook 2010), `%APPDATA%\Microsoft\Templates\*.oft`, e os arquivos `.msg` onde quer que tenham sido salvos |
| Navegadores web | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\` e as pastas de perfil dos outros navegadores |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` e outros locais do usuário/do sistema |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` e `CustomDestinations` |
| ShimCache | Hive `SYSTEM`: `...\Session Manager\AppCompatCache\AppCompatCache` |
| Quarentena do Defender | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| Registro da máquina | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| Registro do usuário | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## Arquivos que devem ser coletados juntos { #files-that-must-travel-together }

- Colete os hives do Registro com os arquivos `.LOG`, `.LOG1` e `.LOG2` adjacentes. O Juntei pode reproduzir os logs
  nos comandos de Registro e no ShimCache quando o hive está sujo (dirty).
- Colete o `SRUDB.dat` do SRUM de forma consistente. Atualmente, o Juntei relata um banco de dados ESE sujo, mas
  não reproduz os `SRU*.log`; os registros existentes apenas nesses logs ficam indisponíveis.
- Colete o `$MFT` junto com o `$Extend/$J` sempre que possível. O registro USN armazena referências de arquivo do pai,
  não um caminho completo; o MFT melhora a resolução dos caminhos.
- Colete o `$LogFile` com o `$MFT` do mesmo volume, no momento mais próximo possível. O `$LogFile` se sobrescreve
  rapidamente; uma coleta posterior pode perder totalmente os registros de redo/undo anteriores.
- Colete o SRUM com o hive `SOFTWARE` correspondente para resolver os nomes de contas e de perfis Wi-Fi.
- Colete o diretório `Sum` inteiro do Registro de acesso do usuário: o `SystemIdentity.mdb` fornece os nomes das
  funções e lista os bancos de dados anuais `{GUID}.mdb`. Assim como no SRUM, o Juntei não reproduz os logs de
  transação `Svc*.log` do UAL.
- Colete os bancos de dados dos navegadores com os arquivos `-wal` ao lado deles (`places.sqlite-wal`,
  `cookies.sqlite-wal`): as alterações mais recentes, e as versões mais antigas das páginas das quais as entradas
  excluídas são recuperadas, só ficam ali até o navegador fazer um checkpoint.
- Colete a pasta `WebCache` inteira: o Windows mantém o `WebCacheV01.dat` aberto enquanto o usuário está conectado,
  e suas alterações mais recentes ficam apenas nos logs de transação `V01*.log`, que o Juntei não reproduz.
- Colete os arquivos OST e PST do Outlook com o Outlook fechado, ou a partir de uma imagem de disco ou de uma cópia de
  sombra de volume: o Outlook os mantém abertos enquanto está em execução. Procure PSTs também fora das pastas padrão: os
  usuários movem os PSTs de arquivamento para outras pastas, unidades e compartilhamentos de rede.
- Colete a árvore `Quarantine` inteira do Defender para que Entries, ResourceData e Resources possam ser associados.

## Cuidados com sistemas ativos { #live-system-cautions }

Arquivos abertos podem mudar durante a cópia. Hives do Registro e bancos de dados ESE podem estar internamente sujos.
O ShimCache pode não gravar em disco suas entradas mais recentes em memória até o desligamento. Uma ferramenta de coleta forense
que use snapshots de volume ou acesso bruto é preferível a uma cópia de arquivos comum.

## Segurança { #safety }

As evidências podem conter malware, amostras de exploits, arquivos LNK maliciosos e texto com fórmulas de planilha.
Faça a análise em um sistema isolado, não execute os fluxos recuperados e use `--extract` somente quando você
pretender manipular conteúdo perigoso. Preserve a origem intacta separadamente.
