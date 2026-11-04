# Histórico de aplicativos { #application-history }

## O que as ferramentas deixam para trás { #what-tools-leave-behind }

Muitas ferramentas de terceiros guardam configurações e históricos no `NTUSER.DAT` do usuário. Várias delas estão entre
as ferramentas que atacantes usam, ou usam para preparar dados antes de levá-los: ferramentas Sysinternals, compactadores,
clientes de sincronização em nuvem e ferramentas de acesso remoto. O Juntei lê estas:

| Aplicativo | Chave em `HKCU\Software` | O que mostra |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | Cada ferramenta cuja licença o usuário aceitou (`EulaAccepted`): PsExec, ProcDump, Sysmon, … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | Arquivos compactados abertos; nomes de arquivos compactados e pastas de extração digitados em suas caixas de diálogo |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | Arquivos compactados criados; pastas de extração; pastas navegadas e usadas como destino de cópia em seu gerenciador de arquivos |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | Arquivos compactados no menu Arquivo, pastas de extração, arquivos compactados recentes |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | Endereço de e-mail, pasta local e nomes de cada conta; bibliotecas sincronizadas (`ScopeIdToMountPointPathCache`) e pastas (`Tenants\<tenant>`) |
| Adobe Acrobat e Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | Arquivos PDF e pastas abertos por último, com nomes de arquivo, tamanhos e datas |
| TeamViewer | `TeamViewer` (e `Version<N>` de uma versão mais antiga) | Todos os valores: nomes de conta e de usuário, pastas iniciais de transferência de arquivos (`<partner ID>?<local>|<remote>`), … |
| CCleaner | `Piriform\CCleaner` | Todos os valores: o que está configurado para limpar, `UpdateKey`, … |

<span class="evidence-strength">Melhor conclusão:</span> este usuário executou a ferramenta (ou aceitou sua
licença), ou a ferramenta abriu, criou ou extraiu esses arquivos compactados e pastas, ou esta conta de nuvem
foi configurada para o usuário. A maioria dos históricos não tem horário por entrada: apenas a entrada mais recente recebe
o horário da última gravação da chave da lista.

## Localização { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

Colete o `NTUSER.DAT` de cada usuário com seus logs de transação, e o `SOFTWARE` do sistema para obter
os SIDs dos usuários.

## Cuidados para o analista { #analyst-cautions }

- **Aceitação do EULA do Sysinternals.** `FirstUsedUTC` é uma estimativa do primeiro uso a partir do horário da última
  gravação da chave quando seu único valor é um `EulaAccepted` legível definido como `1`. Um EULA recusado ou
  danificado não recebe essa estimativa. As configurações do Registro também podem ser pré-configuradas,
  copiadas ou editadas: corrobore a execução e seu horário com outros artefatos.
  Ferramentas que guardam configurações ali (Process Explorer, Autoruns, …) regravam a chave mais tarde:
  `FirstUsedUTC` fica então vazio e `KeyLastWrittenUTC` é apenas a última alteração de configuração. Uma ferramenta
  executada com `-accepteula`, ou como outro usuário (o lado remoto do PsExec é executado como SYSTEM), grava a
  chave desse usuário.
- **Posições, não horários.** As entradas da lista são ordenadas (`MruPosition` 0 é a mais recente), mas
  apenas a última gravação da chave da lista data a mais recente. Entradas mais antigas foram usadas antes, em um
  horário desconhecido.
- **Nomes digitados.** O `ArcName` do WinRAR é o nome do arquivo compactado digitado ao criá-lo: geralmente um nome de
  arquivo sem pasta.
- **Os caminhos do Adobe** são mantidos como o Adobe os grava (`/C/Users/alice/a.pdf` é
  `C:\Users\alice\a.pdf`). Seu `sDate` (em `Details`) é uma string de data PDF, mantida como gravada.
- **O `UpdateKey` do CCleaner** é horário local sem fuso; é mantido como gravado.
- **TeamViewer.** Os logs de conexão são arquivos (`Connections_incoming.txt`, `TeamViewer*_Logfile.log`),
  não valores do Registro: colete-os também.
- **Ausência.** Usuários e ferramentas podem limpar esses históricos (o próprio CCleaner pode).

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

O resumo lista as estimativas de primeiro uso mais recentes do Sysinternals, os arquivos compactados e as pastas que cada lista usou
por último, e as contas do OneDrive.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `SourcePath` | Host e `NTUSER.DAT`. |
| `User`, `UserSID` | O nome da pasta do perfil e seu SID a partir do `ProfileList` do sistema. |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`. |
| `Item` | A ferramenta Sysinternals, a lista (`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …), a conta do OneDrive ou a chave de suas pastas sincronizadas, ou o nome do valor. |
| `MRU-Position` | A posição da entrada em sua lista, sendo 0 a mais recente. |
| `Path` | O arquivo compactado, a pasta ou o documento. |
| `Value` | `EulaAccepted`, o endereço de e-mail de uma conta do OneDrive ou o id de uma biblioteca sincronizada, um valor do TeamViewer ou do CCleaner. |
| `FirstUsedUTC` | Estimativa do primeiro uso a partir da aceitação do EULA (veja os cuidados acima). |
| `Details` | `DisplayName`, `UserCID`, `ConfiguredTenantId`, … de uma conta do OneDrive; `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate` de um documento do Adobe. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | De onde o registro vem e quando essa chave foi alterada pela última vez. |
| `DataRaw` | Os bytes do valor (hex), quando não são decodificados. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Dados que não puderam ser decodificados ou lidos. |

## Estratégia de investigação { #investigation-strategy }

1. Procure ferramentas Sysinternals que um usuário comum não executa (PsExec, ProcDump, PsLoggedOn,
   SDelete) e seus carimbos de data/hora de EULA.
2. Revise nomes de arquivos compactados e pastas de extração próximos ao incidente: arquivos compactados criados em
   `ProgramData`, `Temp` ou na pasta de um usuário antes de os dados saírem do sistema.
3. Verifique contas do OneDrive que não pertencem à organização, e nomes de conta e pastas de transferência de arquivos
   do TeamViewer.
4. Correlacione com Prefetch, Amcache, BAM e UserAssist (a execução da ferramenta), o `$MFT` e o Diário
   USN (o arquivo compactado), arquivos LNK e Jump Lists (documentos abertos) e logs de eventos.

## Mais detalhes { #further-details }

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
