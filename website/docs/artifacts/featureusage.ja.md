# FeatureUsage、RecentApps およびタスクバー ピン { #featureusage-recentapps-and-taskbar-pins }

## 彼らは何ですか { #what-they-are }

Explorer は、ユーザーがアプリケーションをどのように使用したかに関するいくつかの記録をユーザーの `NTUSER.DAT` に保持します。

- **FeatureUsage** (Windows 10 以降): 用途ごとのアプリケーションごとのカウンター。
  - `AppSwitched`: ユーザーがアプリケーションのウィンドウに切り替えた回数 (タスクバーのクリック、Alt+Tab)。
  - `AppLaunch`: タスクバー (ピン留めされた項目) から起動された回数。
  - `ShowJumpView`: ジャンプ リストが開かれた回数 (タスクバーを右クリック)。
  - `AppBadgeUpdated`: タスク バー バッジ (未読数) が変更された回数。
  - `TrayButtonClicked`: 各タスク バー ボタン (`StartButton`、`SearchButton`、`TaskViewButton`、`ClockButton` など) がクリックされた回数。

  アプリケーションは、アプリケーション ユーザー モデル ID (`Microsoft.Windows.Explorer`、`Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`)、最初の部分が既知のフォルダーの GUID (`{1AC14E77-…}\cmd.exe` は `%windir%\system32\cmd.exe`) である可能性があるパス、またはプロセス ID です。 (`*PID00001d74`) Windows エクスプローラーはアプリケーションと一致しませんでした。
- **RecentApps** (Windows 10 から 1709): ユーザーが最近実行したアプリケーション、起動回数と前回の時刻。それぞれの下に、開いたファイル (`RecentItems`) が表示されます。
- **Tタスクバーピン** (`Taskband`): シェル項目(`Favorites`)およびピン留めされた各ショートカットのリンク(`FavoritesResolve`)として、タスクバーにピン留めされたプログラム。 Windows 10 以降では、各ピンには、Windows がピンを固定したのか (`SystemPinned`)、ユーザーがピンを固定したのか (`UserPinned`) が示されます。
- **ProgramsCache** (`StartPage`、`StartPage2`; Windows XP ～ 7): [スタート] メニューのキャッシュされたプログラム (`ProgramsCache`) などWindows 7 スタート メニューとタスク バーのピン留めされたもの (`ProgramsCacheSMP`、`ProgramsCacheTBP`)。

<span class="evidence-strength">この証拠から判断できること:</span> このユーザーが、このアプリケーションを、この方法でこの回数使用したこと（FeatureUsage）、この時刻に実行したこと（RecentApps）、またはピン留めしたことを示します。カウンタ自体に時刻はありません。日時の判断に使えるのは、キーの最終書き込み時刻と RecentApps の時刻です。

## 場所 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

`FeatureUsage` の `KeyCreationTime` はカウントを開始したときです。概要がそれを示しています。すべてのプロファイルのトランザクション ログを含む `NTUSER.DAT`、ホストおよびユーザーの SID のシステムの `SYSTEM` および `SOFTWARE` ハイブを収集します。

## 分析時の注意点 { #analyst-cautions }

- **回数を含まないカウント。** FeatureUsage カウントは、いつではなく、どのくらいの頻度で行われるかを示します。種類のキーの最終書き込み時刻 (`KeyLastWrittenUTC`) は、そのカウントに対する最新の変更です。
- **すべての実行が記録されるわけではありません。** `AppSwitched` は起動回数ではなく、ウィンドウの切り替え回数です。ウィンドウを持たないアプリや、ユーザーが切り替えなかったアプリは数えられません。`AppLaunch` は、タスクバーからの起動だけを数えます。
- **パッケージ化されたアプリとパス。** アプリケーションは、そのウィンドウを Windows グループ化して名前が付けられます。パッケージ化されたアプリの ID、ID、それ自体に設定されたプログラム (`Chrome`、`VMware.Workstation.vmui`)、またはパスです。 `AppPath` は、既知のフォルダーの GUID をフォルダーのデフォルト パスに置き換えたパスを提供します。
- **Default pins.** 新しいプロファイルはデフォルトのピンを取得します (ファイル エクスプローラー、エッジ、ストア)。 Windows 10 で、その後は `SystemPinned` と言われます。 `UserPinned` ピンはユーザーが選択します。
- 2 つのリストの **ピン。** `Favorites` および `FavoritesResolve` は、同じピンを同じ順序 (`Index`) で記述しますが、別個のレコードとして指定されます。
- **ProgramsCache パス** は相対パスです。フォルダー (`Programs\Accessories`) の後にそのフォルダー内のショートカット (`Notepad.lnk`) が続きます。
- **破損した値。** `REG_DWORD` ではないカウント、`REG_QWORD` ではない時間、理解されないピン、または 1 ではないリンクはバイト (`DataRaw`) を保持し、`Problems` で報告されます。

## Juntei で解析する { #parse-with-juntei }

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

概要には、ソースごとにレコードがリストされ、各ハイブの FeatureUsage がカウントを開始したとき、ほとんどのアプリケーション、最近のアプリと項目 (新しい順)、およびタスク バーのピンが切り替わりました。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | ホスト、プロファイル フォルダーの名前、システムの `ProfileList` からのユーザーの SID、およびハイブ。 |
| `Source` | `AppSwitched`、`AppLaunch`、`ShowJumpView`、`AppBadgeUpdated`、`TrayButtonClicked` (FeatureUsage)、`RecentApps`、`RecentItems`、`Taskband` または`ProgramsCache`。 |
| `App` | FeatureUsage 値の名前、最近のアプリの `AppId`、またはピンのアプリケーション ユーザー モデル ID。 |
| `AppPath` | `App` をパスとして使用し、既知のフォルダーの GUID をデフォルトのパスに置き換えます。 ID の場合は空です。 |
| `Count` | FeatureUsage カウント、または最近のアプリの `LaunchCount`。 |
| `LastAccessedUTC` | 最近のアプリまたはアイテムの `LastAccessedTime`。 |
| `ItemPath` | 最近のアイテムの `Path` (または最近のアプリの `AppPath`)。ピンまたはキャッシュされたプログラムのシェル項目パス (`User Pinned\TaskBar\File Explorer.lnk`、`Applications\<app ID>`)。 `FavoritesResolve` リンクのパス。 |
| `DisplayName` | 最近のアイテムの `DisplayName`。 |
| `PinType` | `SystemPinned` または `UserPinned`、`Favorites` ピンの場合。 |
| `Index` | リスト内のピンまたはキャッシュされたプログラムの位置。 |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | 値とその取得元のキー、およびキーの最後の書き込み。 |
| `DataRaw` | デコードされないバイト (16 進数)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | 間違ったタイプの値、理解できないピンまたはリスト、リンクではないリンク。 |

## 調査戦略 { #investigation-strategy }

1. ユーザーが最もよく使用したアプリケーションの `AppSwitched` を `Count` で並べ替え、管理、リモート アクセス、アーカイブおよび転送ツール、およびユーザーが書き込み可能な場所 (`\Users\`、`Downloads`、`Temp`) のパスを探します。
2. ピンを確認してください: `UserPinned` ツール、またはショートカットが通常とは異なる場所にあるピン。
3. UserAssist、Prefetch、BAM、および Jump Lists を使用して、RecentApps 回を使用して、使用日を特定します。
4. `KeyCreationTime` をプロファイルの作成と比較します。カウントが開始されます。
5. 削除されたカウンターとピンには `parse-registry read --recover-deleted` を使用します。

## さらに詳しい情報 { #further-details }

- [Microsoft: アプリケーション ユーザー モデル ID](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: プログラム キャッシュ](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
