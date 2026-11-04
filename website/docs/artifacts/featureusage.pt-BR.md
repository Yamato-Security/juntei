# FeatureUsage, RecentApps e itens fixados na barra de tarefas { #featureusage-recentapps-and-taskbar-pins }

## O que são { #what-they-are }

O Explorer mantém vários registros de como um usuário utiliza aplicativos no `NTUSER.DAT` desse usuário:

- **FeatureUsage** (Windows 10 e posteriores): contadores por aplicativo, por tipo de uso.
  - `AppSwitched`: quantas vezes o usuário alternou para a janela do aplicativo (clique na barra de tarefas,
    Alt+Tab).
  - `AppLaunch`: quantas vezes ele foi iniciado a partir da barra de tarefas (um item fixado).
  - `ShowJumpView`: quantas vezes sua jump list foi aberta (clique com o botão direito na barra de tarefas).
  - `AppBadgeUpdated`: quantas vezes seu selo na barra de tarefas (uma contagem de não lidos) mudou.
  - `TrayButtonClicked`: quantas vezes cada botão da barra de tarefas (`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …) foi clicado.

  Um aplicativo é um ID de modelo de usuário de aplicativo (`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`), um caminho cuja primeira parte pode ser o GUID de uma
  pasta conhecida (`{1AC14E77-…}\cmd.exe` é `%windir%\system32\cmd.exe`) ou um ID de processo
  (`*PID00001d74`) para janelas que o Explorer não conseguiu associar a um aplicativo.
- **RecentApps** (Windows 10 até a versão 1709): os aplicativos que um usuário executou recentemente, com uma contagem
  de inicializações e o último horário; sob cada um, os arquivos que ele abriu (`RecentItems`).
- **Itens fixados na barra de tarefas** (`Taskband`): os programas fixados na barra de tarefas, como itens do shell
  (`Favorites`) e como o link de cada atalho fixado (`FavoritesResolve`). No Windows 10 e
  posteriores, cada item fixado indica se foi o Windows que o fixou (`SystemPinned`) ou o usuário (`UserPinned`).
- **ProgramsCache** (`StartPage`, `StartPage2`; do Windows XP ao 7): os programas em cache do Menu Iniciar
  (`ProgramsCache`) e, no Windows 7, os itens fixados no Menu Iniciar e na barra de tarefas
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`).

<span class="evidence-strength">Melhor conclusão:</span> este usuário usou este aplicativo este
número de vezes desta forma (FeatureUsage), executou-o neste horário (RecentApps) ou o fixou. Os
contadores não têm horário próprio; somente a última gravação da chave e os horários do RecentApps datam
algo.

## Localização { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

O `KeyCreationTime` de `FeatureUsage` é o momento em que a contagem começou; o resumo o exibe. Colete o
`NTUSER.DAT` com seus logs de transação de cada perfil, além dos hives `SYSTEM` e
`SOFTWARE` do sistema, para o host e o SID do usuário.

## Cuidados para o analista { #analyst-cautions }

- **Contagens sem horários.** Uma contagem do FeatureUsage diz com que frequência, não quando. O horário de última gravação
  da chave do tipo (`KeyLastWrittenUTC`) é a alteração mais recente em qualquer uma de suas contagens.
- **Nem toda execução.** `AppSwitched` conta alternâncias de janela, não inicializações; um aplicativo executado
  sem janela, ou para o qual o usuário nunca alternou, não é contado. `AppLaunch` conta apenas
  inicializações a partir da barra de tarefas.
- **Aplicativos empacotados e caminhos.** Um aplicativo é nomeado da forma como o Windows agrupou suas janelas: o ID de um
  aplicativo empacotado, um ID que um programa definiu para si mesmo (`Chrome`, `VMware.Workstation.vmui`) ou um
  caminho. `AppPath` fornece o caminho com o GUID de uma pasta conhecida substituído pelo caminho padrão
  da pasta.
- **Itens fixados padrão.** Perfis novos recebem itens fixados padrão (Explorador de Arquivos, Edge, a Store); no Windows 10
  e posteriores, eles indicam `SystemPinned`. Um item `UserPinned` é escolha do usuário.
- **Itens fixados em duas listas.** `Favorites` e `FavoritesResolve` descrevem os mesmos itens fixados na mesma
  ordem (`Index`), mas são apresentados como registros separados.
- **Os caminhos do ProgramsCache** são relativos: uma pasta (`Programs\Accessories`) é seguida pelos
  atalhos que contém (`Notepad.lnk`).
- **Valores danificados.** Uma contagem que não é `REG_DWORD`, um horário que não é `REG_QWORD`, um item fixado
  que não é compreendido ou um link que não é um link mantém seus bytes (`DataRaw`) e é relatado em
  `Problems`.

## Analisar com o Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

O resumo lista os registros por fonte, quando o FeatureUsage de cada hive começou a contar, os
aplicativos para os quais o usuário mais alternou, os aplicativos e itens recentes (do mais novo ao mais antigo) e os itens fixados na barra de tarefas.

## Campos de saída { #output-fields }

| Campo | Interpretação |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, nome da pasta de perfil, SID do usuário obtido do `ProfileList` do sistema e o hive. |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked` (FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` ou `ProgramsCache`. |
| `App` | O nome de um valor do FeatureUsage, o `AppId` de um aplicativo recente ou o ID de modelo de usuário de aplicativo de um item fixado. |
| `AppPath` | `App` como caminho, com o GUID de uma pasta conhecida substituído pelo seu caminho padrão; vazio para IDs. |
| `Count` | Uma contagem do FeatureUsage ou o `LaunchCount` de um aplicativo recente. |
| `LastAccessedUTC` | O `LastAccessedTime` de um aplicativo ou item recente. |
| `ItemPath` | O `Path` de um item recente (ou o `AppPath` de um aplicativo recente); o caminho do item do shell de um item fixado ou programa em cache (`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`); o caminho de um link de `FavoritesResolve`. |
| `DisplayName` | O `DisplayName` de um item recente. |
| `PinType` | `SystemPinned` ou `UserPinned`, para um item fixado de `Favorites`. |
| `Index` | A posição de um item fixado ou programa em cache em sua lista. |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | O valor e a chave de origem, e a última gravação da chave. |
| `DataRaw` | Bytes (hex) que não puderam ser decodificados. |
| `TransactionLogStatus`, `LogsApplied` | Contexto de recuperação do Registro. |
| `Problems` | Valores do tipo errado, itens fixados ou listas não compreendidos, links que não são links. |

## Estratégia de investigação { #investigation-strategy }

1. Ordene `AppSwitched` por `Count` para ver os aplicativos em que o usuário mais trabalhou e procure
   ferramentas de administração, acesso remoto, compactação e transferência, além de caminhos em locais graváveis pelo usuário
   (`\Users\`, `Downloads`, `Temp`).
2. Verifique os itens fixados: uma ferramenta `UserPinned` ou um item fixado cujo atalho está em um local incomum.
3. Use os horários do RecentApps, com UserAssist, Prefetch, BAM e Jump Lists, para datar o uso.
4. Compare `KeyCreationTime` com a criação do perfil: as contagens começam a partir desse momento.
5. Use `parse-registry read --recover-deleted` para contadores e itens fixados excluídos.

## Mais detalhes { #further-details }

- [Microsoft: IDs de modelo de usuário de aplicativo](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: cache de programas](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
