# Coleta de evidências { #collecting-evidence }

O Juntei analisa arquivos já coletados do Windows. Use uma ferramenta de aquisição confiável e preserve
os metadados originais sempre que possível.

## Localizações comuns { #common-locations }

| Artefato | Localização típica |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | Arquivo de metadados do volume NTFS `C:\$MFT` |
| `$LogFile` | Arquivo de metadados do volume NTFS `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| Diário USN | Fluxo de metadados do NTFS `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
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
- Colete a árvore `Quarantine` inteira do Defender para que Entries, ResourceData e Resources possam ser associados.

## Cuidados com sistemas ativos { #live-system-cautions }

Arquivos abertos podem mudar durante a cópia. Hives do Registro e bancos de dados ESE podem estar internamente sujos.
O ShimCache pode não gravar em disco suas entradas mais recentes em memória até o desligamento. Uma ferramenta de coleta forense
que use snapshots de volume ou acesso bruto é preferível a uma cópia de arquivos comum.

## Segurança { #safety }

As evidências podem conter malware, amostras de exploits, arquivos LNK maliciosos e texto com fórmulas de planilha.
Faça a análise em um sistema isolado, não execute os fluxos recuperados e use `--extract` somente quando você
pretender manipular conteúdo perigoso. Preserve a origem intacta separadamente.
