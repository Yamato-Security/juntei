# MuiCache { #muicache }

## MuiCacheとは { #what-muicache-is }

ユーザーがプログラムを開始すると、Windows は、シェルが表示する名前 (`FileDescription`、たとえば、`cmd.exe` の場合は「Windows コマンド プロセッサ」) のプログラムのバージョン リソースを読み取り、ユーザーごとに `MuiCache` にキャッシュします。 Windows 8 以降も会社 (`CompanyName`) をキャッシュします。キャッシュはプログラムのフルパスによってキー化されます。

- **Windows 8 以降:** プログラムごとに 2 つの値、`<path>.FriendlyAppName` および `<path>.AppCompany`。
- **Windows Vista および 7:** プログラムごとに 1 つの値 `<path>` がフレンドリ名を保持します。
- **Windows XP:** `NTUSER.DAT` の同じ形式。

MuiCache は、ユーザーのシェル セッションにプログラム実行リードを提供します。これには、リムーバブル ドライブ、ネットワーク共有、存在しないフォルダー上のプログラムなど、ファイル自体が付けた名前と会社が含まれます。

<span class="evidence-strength">この証拠から判断できること:</span> このシェルプロファイルに、ファイルのパス、説明、会社名がキャッシュされていました。他の実行アーティファクトと関連付けてください。このプロファイルは、プロセスの実行アカウントを特定するものではありません。MuiCache にプログラムごとの時刻や実行回数はありません。

## 場所 { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (キャッシュされた文字列の言語) はプログラムではないため、省略されます。 `@<file>,-<number>` (`@shell32.dll,-22019`) という名前の値は、プログラムではなく、シェルが独自のメニューとファイル タイプ用にキャッシュしたリソース文字列です。それらは省略され、要約ではカウントされます。 Windows Vista 以降では、`NTUSER.DAT` には MuiCache がありません。

すべてのプロファイルのトランザクション ログとともに `UsrClass.dat` (および `NTUSER.DAT`) を収集し、ホスト名とユーザー名についてはシステムの `SYSTEM` および `SOFTWARE` ハイブを収集します。

## 分析時の注意点 { #analyst-cautions }

- **プロファイルの所有者は必ずしもプロセス アカウントである必要はありません。** 別のアカウントで起動されたプログラムは、このユーザーのセッションにウィンドウを表示し、このユーザーのキャッシュにメタデータを残すことができます。 `User` および `UserSid` は、プロセス セキュリティ コンテキストではなく、ハイブの所有者を識別します。
- **プログラムごとの時間はありません。** `KeyLastWrittenUTC` はキーの最後の書き込みです。つまり、特定のプログラムが実行されたときではなく、そのプログラムに対する最新の変更です。 Prefetch、BAM、UserAssist およびイベント ログに時間が表示されます。
- **ユーザーが手動で起動したプログラムだけではありません。** `Open with`（「プログラムから開く」）、シェル拡張、インストーラの子プロセスなど、シェル自身が読み込んだ DLL やプログラムも現れることがあります。例として `explorerframe.dll` や `shell32.dll` があります。
- **名前はファイルに由来します。** `FriendlyAppName` および `AppCompany` は、実行時にファイルのバージョン リソースが示したものです。マルウェアはあらゆる説明や会社名を主張できます。名前を変更したツールは元の説明を保持するため、その内容が明らかになる可能性があります (フレンドリ名がリモート アクセス ツールである `svchost.exe` という名前のファイル)。
- **パスごとに 1 つのエントリ。** 同じプログラムを再度実行しても、エントリは追加されません。別のパスにあるコピーでも同様です。プログラムのエントリは、ファイルが削除された後も残ることがあります。
- **両方の形式が 1 つのパスに対応します。** パスに両方の形式のフレンドリ名がある場合 (アップグレードされたプロファイル)、それぞれが独自のレコードを取得するため、どちらの値も失われません。
- **破損した値。** 文字列ではない値、無効な値、UTF-16、または全体を読み取ることができない値は、そのバイト (`FriendlyAppNameRaw`、`AppCompanyRaw`) を保持し、`Problems` で報告されます。無効な UTF-16 によっても、表示されるテキストに置換文字が生成される可能性があります。元のバイトには生の列を使用します。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

この概要には、Windows および Program Files フォルダー (ユーザー フォルダー、他のドライブ、ネットワーク共有) の外側にあるプログラムとそのユーザー、および取り残されたリソース文字列の数がリストされます。他の名前の Program Files フォルダー (他の言語では Windows) もリストされます。相対パスと環境変数フォルダーをエスケープするパスはリストされたままになります。通常のドライブ パスは、検査者のファイル システムにアクセスせずに `.` および `..` コンポーネントを解決した後にチェックされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | ホスト、プロファイル フォルダーの名前、ユーザーの SID (`UsrClass.dat` のルート キーによって名前が付けられます。それ以外の場合はシステムの `ProfileList`)、およびハイブ。 |
| `HiveType` | `UsrClass.dat` または `NTUSER.DAT`。 |
| `Path` | プログラムのパス: `.FriendlyAppName` または `.AppCompany` を除いた値の名前。 |
| `FriendlyAppName` | プログラムの説明 (`<path>.FriendlyAppName` または `<path>`)。 |
| `AppCompany` | プログラムの会社、`<path>.AppCompany` (Windows 8 以降)。 |
| `KeyPath`, `KeyLastWrittenUTC` | MuiCache キーとその最後の書き込み: そのプログラムに対する最新の変更。 |
| `FriendlyAppNameRaw`, `AppCompanyRaw` | 値のバイト (16 進数) (きれいにデコードされるテキストではない場合)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト: `replayed` (ログにプログラムの値の 1 つが書き込まれたとき)。 |
| `Problems` | 値が文字列ではないか、有効な UTF-16 ではないか、読み取ることができません。 |

## 調査戦略 { #investigation-strategy }

1. ユーザーが書き込み可能な場所 (`\Users\`、`Downloads`、`Temp`、`ProgramData`)、他のドライブ、ネットワーク共有、管理ツール、リモート アクセス ツール、およびアーカイブ ツールにあるプログラムを確認します。
2. ファイル名をそのフレンドリ名および会社名と比較します。不一致があると、名前が変更されたツールまたはマスカレード ファイルが明らかになる可能性があります。
3. Prefetch、BAM、および UserAssist (実行時)、Amcache および ShimCache (ファイルのハッシュと存在)、および `$MFT` (ファイルがまだ存在するかどうか) と相関します。
4. 削除された MuiCache 値には、`UsrClass.dat` で `parse-registry read --recover-deleted` を使用します。

## さらに詳しい情報 { #further-details }

- [Microsoft: VERSIONINFO リソース](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cサイバートリアージ: MuiCache 実験、GUI セッション、およびプロセスアカウント制限](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
