# インストールされているプログラム { #installed-programs }

## Windows が保持するもの { #what-windows-keeps }

Windows とインストーラーは、インストールされたソフトウェアをいくつかのレジストリ リストに記録します。各リストには、プログラム、製品、ディストリビューション、またはパッケージごとに 1 つのキーがあります。 Juntei はキーごとに 1 つのレコードを与えます。

| ソース | ハイブとキー | 保持しているもの |
|---|---|---|
| `uninstall` | `SOFTWARE` および `NTUSER.DAT` `…\CurrentVersion\Uninstall\*` および `SOFTWARE\Wow6432Node\…` | 設定と *プログラムと機能* のリスト: 名前、バージョン、発行元、インストール日、フォルダー、アンインストール コマンド、サイズ、非表示かどうか |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | Windows インストーラー製品: 製品名とバージョン、パッケージのファイル名、およびインストール元のフォルダー |
| `app_path` | `SOFTWARE` および `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | プログラム Windows は名前 (`Win+R`、`start tool`) で始まります: プログラムのパスとそのフォルダー |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | Windows Linux ディストリビューションのサブシステム: 名前、そのファイルまたはディスクの場所、WSL 1 または 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | パッケージ化された (ストア) アプリ: すべてのユーザーにプロビジョニングされ、Windows 独自の、ステージングされ、プロビジョニング解除され、各ユーザーの |

<span class="evidence-strength">この証拠から判断できること:</span> このプログラム、製品、パッケージが、コンピュータ全体またはこのユーザー向けにインストール、あるいは登録されました。インストール日はインストーラが記録した値で、キーの最終書き込み時刻は、そのエントリを最後に変更した時刻です。

## 場所 { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

`SOFTWARE`、すべての `NTUSER.DAT`、およびそれらのトランザクション ログを収集します。 `SYSTEM` はホストの名前であり、`SOFTWARE` の `ProfileList` は各ユーザーの SID を示します。

## 分析時の注意点 { #analyst-cautions }

- **インストールの日付とキーの時間。** `InstallDate` は、インストーラーによって現地時間で書き込まれた日付 (時間なし) です。インストーラーによっては、それを書き込まないものもあれば、別の形式 (`InstallDateRaw` に書かれたままにする) を書き込むものもあります。更新により、日付を変更せずにエントリを書き換えたり、プログラムをあまり変更せずに日付を変更したりできます。`InstallDate` と `KeyLastWrittenUTC` を比較してください。
- **アンインストールされたソフトウェア。** 完全に削除されたプログラムはキーを失います。削除された `Uninstall` キーには `parse-registry read --recover-deleted` を使用し、残されたトレースには Amcache、Prefetch、および `$MFT` を使用します。
- **ユーザーごとの登録。** `NTUSER.DAT` エントリは、プロファイルに登録されたソフトウェアを識別します。この範囲では、誰がインストーラーを実行したか、または管理者権限が使用されたかどうかは確立されません。
- **Hiddenエントリー.** `SystemComponent`は、*プログラムと機能*からエントリーを非表示にします。 `DisplayName` のないエントリ (アップデート、コンポーネント) も表示されません。それらはすべて報告されています。
- **製品コード.** GUID 形状の `Uninstall` キーは、`ProductCode` までのパックされた `msi_product` キーと照合できます。 GUID の形をした名前だけでは、Windows インストーラーがエントリを作成したことを証明するものではありません。
- **App Paths.** これらは、実行記録ではなく、プログラムを見つけるために使用される登録です。 `Path` 値は複数の検索ディレクトリを保持できますが、必ずしもインストール ディレクトリである必要はありません。
- **Publisher.** これは、インストーラーによって提供されるレジストリ メタデータであり、検証された署名情報ではありません。
- **Architecture.** `Wow6432Node` は、64 ビット Windows (`Architecture` `x86`、`Wow64`) 上に 32 ビット プログラムを保持します`True`);他のエントリはシステム独自のアーキテクチャ (空) です。パッケージのアーキテクチャは独自です (`x64`、`neutral`、…)。
- **WSL.** `Version` は、ディストリビューションの `Flags` の WSL バージョンです (`0x8`: WSL 2 の仮想で実行されます)マシン）。ディストリビューション自体の `Version` 値 (`Details` 内) はそのファイル システムのバージョンであり、WSL 1 ディストリビューションでも 2 になります。 `BasePath` には、`ext4.vhdx` (WSL 2) または `rootfs` (WSL 1) が保持されています。これを収集して内部を確認します。
- **Packages.** `<SID>` リストは、各ユーザーに登録されたパッケージです。ほとんどは Windows 独自のものです。 `Deprovisioned` パッケージは新規ユーザー用に削除され、`EndOfLife\<SID>` パッケージはそのユーザー用に削除されました。
- **破損した値。** 不正な数値、間違った値のタイプ、および無効な UTF ～ 16 は、`Problems` で報告され、`DataRaw` に保持されます。これには、ゼロバイトの現在値の空の 16 進文字列が含まれます。オプションの数値フィールド内のクリーンな空の文字列は、存在しないものとして扱われます。パッケージ バージョンには、符号なし 16 ビットの範囲内にある 4 つの 10 進数コンポーネントが含まれている必要があります。 `-t` がレコードを標準出力にストリームするとき、ハイブレベルの警告が標準エラー出力に残ります。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

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
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

この概要には、最後に (`InstallDate` によって) インストールされたプログラム、ユーザーが自分でインストールしたもの、および WSL ディストリビューションがリストされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath`, `HiveType` | ホスト、ハイブ ファイル、および `SOFTWARE` または `NTUSER.DAT`。 |
| `User`, `UserSID` | ユーザー ハイブのユーザー (そのプロファイル フォルダーと `ProfileList` SID)、またはパッケージがリストされているユーザー。 |
| `Source`, `Scope` | リスト (`uninstall`、`msi_product`、`app_path`、`wsl`、`package`)、および `machine` または `user`。 |
| `Name`, `Version`, `Publisher` | `DisplayName`、`DisplayVersion`、`Publisher`;製品の `ProductName` および `Version`。アプリパスのプログラムファイル名。ディストリビューションの名前と WSL のバージョン。パッケージの名前とバージョン。 |
| `InstallDate`, `InstallDateRaw` | `YYYYMMDD` が書き込まれたときの日付として `InstallDate`。それ以外は書かれているとおりです。 |
| `KeyLastWrittenUTC` | エントリのキーが最後に変更されたとき。 |
| `InstallLocation`, `InstallSource` | `InstallLocation` (アプリパスの `Path`、ディストリビューションの `BasePath`、パッケージの `Path`)。 `InstallSource` (製品の最後に使用されたソース フォルダー)。 |
| `Program` | App Path のプログラム。 |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`. |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | `EstimatedSize` (KiB);それが隠されているかどうか。 Windows インストーラーがインストールしたかどうか。 |
| `ProductCode` | Windows インストーラーの製品コード (`{GUID}`)。 |
| `Architecture`, `WOW64` | `Wow6432Node` の下の `x86`、またはパッケージのアーキテクチャ。キーが `Wow6432Node` の下にあるかどうか。 |
| `Details` | 詳細: `DisplayIcon`、`ModifyPath`、`ParentKeyName`、`ReleaseType`、`Inno Setup: User` (Inno Setup インストーラーを実行したアカウント)。製品の `PackageCode`、`PackageName`。ディストリビューションの `Flags`、`State`、`DefaultUid`、`Version`、`PackageFamilyName`、`Default`。パッケージの `List`、`ResourceId`、`PublisherId`。 |
| `KeyPath` | エントリのキー。 |
| `DataRaw` | 名前と異なる値のバイト (16 進数)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | 読み取れなかった値、または名前が示すものと異なる値。 |

## 調査戦略 { #investigation-strategy }

1. インシデントを中心に `InstallDate` および `KeyLastWrittenUTC` で並べ替えます。リモート アクセス ツール、トンネリングおよびファイル転送クライアント、アーカイバ、スクリプト ランタイム、セキュリティ ツールを見てみましょう。
2. `Scope` `user` レコードを確認します: 管理者権限なしでインストールされたソフトウェア。
3. ユーザーが書き込み可能なフォルダー (`Downloads`、`AppData`、`Temp`、`ProgramData`) については、`InstallLocation`、`InstallSource`、および `UninstallCommand` を確認します。
4. WSL ディストリビューションを探し、その `BasePath` を収集します。
5. Amcache および Prefetch (インストーラーと実行中のプログラム)、`$MFT` (インストール フォルダーの作成)、およびアプリケーション イベント ログ (MsiInstaller イベント) と関連付けます。

## さらに詳しい情報 { #further-details }

- [Microsoft: レジストリ キー ](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key) をアンインストールします
- [Microsoft: アプリの登録 (アプリのパス)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: パッケージ ID の概要](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
