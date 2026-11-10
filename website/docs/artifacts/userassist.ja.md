# UserAssist { #userassist }

## UserAssistとは { #what-userassist-is }

Windows エクスプローラーは、ユーザーがエクスプローラー シェルから起動するプログラムとショートカット (スタート メニュー、タスクバー、デスクトップ、フォルダー内のダブルクリック) をカウントします。ユーザーの `NTUSER.DAT` にカウントが保存されます。各項目には次のような特徴があります。

- 実行カウント。
- Windows 7 以降では、フォーカスがあった回数と長さ (ミリ秒単位)。
- 最後に実行されたとき。

値の名前は ROT13 でエンコードされます。通常、プログラム名は既知のフォルダーの GUID で始まります (`{1AC14E77-…}` は `System32`)。

UserAssist は、特定のユーザーが対話的に開始した内容を示しますが、他のほとんどの実行証拠はユーザーに帰属しません。

<span class="evidence-strength">この証拠から判断できること:</span> ユーザーが Explorer を通じて、このプログラムまたはショートカットを、この回数起動し、最後に起動したのがこの時刻です。コマンドライン、サービス、スケジュールタスク、別のプログラムなど、他の方法による起動は数えられません。

## 場所 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | ホールド |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | プログラム (`executable`)、Windows 7 以降 |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | ショートカット (`shortcut`)、Windows 7 以降 |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | Vista へのプログラムとショートカット (`active_desktop`)、Windows 2000 |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | インターネット ツールバー (`internet_toolbar`)、Windows 2000 ～ Vista |

他の GUID は `other` として報告されます。すべてのプロファイルのトランザクション ログを含む `NTUSER.DAT` と、ユーザーの SID を提供するシステムの `SOFTWARE` ハイブを収集します。

## 分析時の注意点 { #analyst-cautions }

- **新しいプロファイルがシードされます。** Windows 7 以降では、新しいプロファイルはデフォルトの項目 (「はじめに」、「電卓」、「ペイント」、「付箋」など) で始まります。それらの実行カウントは 14 からカウントダウンされ、プロファイルが作成された前後の 1 つの最終実行時刻をすべて共有します。それらはユーザーのアクティビティではありません。
- **Version 3 は、5** (Windows 2000 から Vista) までカウントされます。保存されたカウントは 5 から始まるため、そこから 5 が減算されます。 5 に保存されているカウントは、`Problems` にレポートされます。
- **Paths は、既知のフォルダーのデフォルト パス** (`%windir%\system32`、`%ProgramFiles%`、`%APPDATA%\…`) を使用します。別の場所にリダイレクトされたフォルダーには、引き続きデフォルトが表示されます。アプリ ID (`Microsoft.Windows.Explorer`、`Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) とコントロール パネル項目 (`::{…}`) にはパスがありません。
- **追跡はオフにすることができます** と `Explorer\Advanced` `Start_TrackProgs` = 0, `Policies\Explorer` `NoInstrumentation` = 1、 または `UserAssist\Settings` `NoLog` = 1。概要には、オフになっているハイブがリストされます。空または欠落している UserAssist アクティブなプロファイルでは、反フォレンジックを意味する場合があります。
- **`NoEncrypt` = 1** では、Windows は ROT13 なしで名前を書き込みます。このような名前は誤ってデコードされるため、そのハイブのすべての項目は `Problems` (およびレコードが stdout に送られるときの stderr) でそのように表示されます。
- **最終実行時刻とカウントは項目ごとに保持されます (**)。ただし、キーの最終書き込み時刻 (`KeyLastWrittenUTC`) は、項目に対する最新の更新の時刻にすぎません。
- フォーカスのみを持った項目の **R 実行回数は 0** になる可能性があります。フォーカス数とフォーカス時間は、バージョン 5 (Windows 7 以降) にのみ存在します。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

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
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

概要には、ユーザーが最後に実行したアイテム (新しいものから順) と、UserAssist がオフになっているハイブがリストされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | ホスト、プロファイル フォルダーの名前、システムの `ProfileList` (システムの `SOFTWARE` ハイブを含む) からのユーザーの SID、およびハイブ。 |
| `GUID`, `Kind` | `{GUID}` キーとそのキーが保持するもの (`executable`、`shortcut`、`active_desktop`、`internet_toolbar`、`other`)。 |
| `NameRot13`, `Name` | 保存およびデコードされた値の名前。 |
| `Path` | パスとしてデコードされた名前。既知のフォルダーの GUID はデフォルトのパスに置き換えられます。アプリ ID とシェル項目の場合は空です。 |
| `RunCount` | エクスプローラーから起動された回数 (バージョン 3: 保存された回数から 5 を引いた値)。 |
| `FocusCount`, `FocusTimeMs` | フォーカスがあった回数とその時間 (ミリ秒単位) (バージョン 5)。 |
| `LastRunUTC` | 最後に開始したとき。 Windows が 0 を格納した場合、または格納された時間が 9999 を超えた場合は空になります (`Problems` がそれを与え、`DataRaw` がバイトを保持します)。 |
| `Format`, `SessionID` | データのレイアウト (5: 72 バイト、3: 16 バイト) およびデータが最後に書き込まれたセッション番号。 |
| `KeyLastWrittenUTC` | `Count` キーの最後の書き込み: その項目の最新の更新。 |
| `DataRaw` | デコードされない場合の値のバイト数 (16 進数)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | サイズが不明なデータ、キーの `Version`、5 の下のバージョン 3 カウント、9999、`NoEncrypt` 以降の最後の実行と一致しないサイズ。 |

## 調査戦略 { #investigation-strategy }

1. `LastRunUTC` でソートし、シードされたアイテムを脇に置きます。これらのアイテムは、プロファイルの作成時に 1 回共有されます。
2. ユーザーが書き込み可能な場所 (`%USERPROFILE%`、`Downloads`、`Temp`、`ProgramData`)、リムーバブル ドライブとネットワーク パス、および管理ツールとリモート アクセス ツールにあるプログラムを探します。
3. Prefetch (ランタイムを伴う実行)、Amcache および ShimCache (存在)、Jump Lists および LNK ファイル (これらのファイルで開かれたもの)、およびプログラム ファイルの `$MFT` と相関させます。
4. 追跡がオフになっているハイブと、UserAssist が空であるプロファイルをクリアするためにチェックしてください。
5. 削除された UserAssist 値には `parse-registry read --recover-deleted` を使用します。

## さらに詳しい情報 { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: ユーザー アシスト キー](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
