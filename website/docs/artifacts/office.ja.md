# Microsoft Office (最近使用したファイル、信頼できるドキュメント、マクロ設定) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## オフィスが記録するもの { #what-office-records }

Microsoft Office は、`NTUSER.DAT` のユーザーごと、バージョンおよびアプリケーションごとに次のように保持します。

- **最近使用したファイルと場所:** `File MRU`、アプリケーションが開いたドキュメント、および `Place MRU`、アプリケーションが開いたフォルダー。各項目は最後に開かれた時刻を示します。 Microsoft 365 は、サインインしたアカウントごとのリストも保持します (`User MRU\<account>`)。
- **信頼されたドキュメント:** ユーザーがドキュメント上で [編集を有効にする*] または [コンテンツを有効にする] をクリックすると、Office はそれを `TrustRecords` に記憶します: ドキュメントのパス、いつ信頼されたか、マクロが有効になったかどうか。
- **マクロ設定:** `VBAWarnings` (どのマクロを実行するか)、`BlockContentExecutionFromInternet` (インターネットからのファイル内のマクロをブロックするかどうか)、`AccessVBOM` (プログラムが VBA プロジェクトを変更するかどうか)、および **Trusted Locations**、ドキュメントのマクロが要求せずに実行されるフォルダー。グループ ポリシーは、`Software\Policies` で同じように設定できます。

悪意のあるドキュメントでは、通常、ユーザーがマクロを有効にする必要があります。マクロが有効になっている信頼レコードは、ドキュメント、ユーザー、および時間を結び付けます。

<span class="evidence-strength">この証拠から判断できること:</span> ユーザーがこのアプリケーションでこの文書を開き、最後に開いたのがこの時刻です。また、この時刻に文書の編集やマクロを許可したことや、Office のマクロ実行設定がこの状態だったことを示します。

## 場所 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

バージョン: `16.0` は Office 2016 以降 (2019、2021、2024、Microsoft 365)、`15.0` Office 2013、 `14.0` オフィス 2010、`12.0` オフィス 2007。

項目は `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx` のようになります。`T` は最後に開かれた時刻 (16 進数の FILETIME) で、パスは `*` の後に続きます。信頼レコードは 24 バイトです。文書が信頼された FILETIME、次に最後の 4 バイトのフラグ、マクロが有効な場合は `0x7FFFFFFF`、編集のみが有効な場合は `0x00000001` です。

## 分析時の注意点 { #analyst-cautions }

- **Times.** `LastOpenedUTC` は各アイテムの独自の時間です。 `TrustedAtUTC` は、ユーザーがドキュメントを信頼したとき、*編集を有効にする* または *コンテンツを有効にする* をクリックした瞬間であり、最後に開く前である可能性があります。
- **マクロは有効ですが、実行されません。** `MacrosEnabled` は、ユーザーがドキュメントのマクロを有効にしたことを示します。マクロが実行されたか、マクロが何をしたかを証明するものではありません。プロセス作成、Prefetch、PowerShell ログ、およびドキュメント自体と関連付けます。
- Office に保存される **Paths。** 信頼レコードでは、`%USERPROFILE%`、スラッシュ、`file:///`、URL、および URL エンコード (`%20`) を使用する可能性のあるパスでドキュメントに名前を付けます。クラウドドキュメントのMRU項目はURLです。
- **Defaults と有効なポリシー。** Office は、デフォルトの信頼できる場所 (`Templates`、`XLSTART`、`STARTUP` など) を作成します。場所の `Date` は便利な手掛かりであり、誰が作成したかを証明するものではありません。管理者、スクリプト、ポリシー、およびコピーされたレジストリ設定もこの場所に設定される可能性があります。 `VBAWarnings` が欠落しているだけでは、有効なマクロ ポリシーは確立されません。 Office のバージョン、インターネット ファイルのマーキング、その他の管理設定も重要です。 [Microsoft では、インターネット マクロのブロック ](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked) について説明します。
- **削除されたドキュメント。** リストはドキュメントよりも存続します。見つからないファイルもここに表示できます。リストは短いため、Office の *固定されていない項目をクリア* およびクリーンアップ ツールによってクリアされます。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

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
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

この概要には、マクロが有効になっているドキュメント、マクロをより自由に実行できる設定 (`VBAWarnings` 1、`BlockContentExecutionFromInternet` 0、`AccessVBOM` 1、`AllowNetworkLocations` 1)、信頼できる場所のユーザーがリストされます。追加またはポリシー セット、および最後に開かれたドキュメント。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | ホスト、プロファイル所有者 (パスまたはシェル フォルダーから)、システムの `ProfileList` からの SID、およびハイブ。 |
| `OfficeVersion`, `App` | バージョンのキー (`16.0`) とアプリケーション (`Word`、`Excel`、…)。 |
| `List` | `File MRU`、`Place MRU`、`TrustRecords`、`Security`、または `Trusted Locations`。 |
| `Account` | `User MRU` リストの Microsoft 365 アカウント。 |
| `Policy` | 設定または場所がポリシーによって設定されているかどうか。 |
| `MRU-Position`, `IsMostRecent` | リスト内の項目の位置 (`Item 1` は 0、最新)。 |
| `Path` | ドキュメントまたはフォルダー。保存されている信頼できるドキュメントのパス。信頼できる場所の `Path`。 |
| `LastOpenedUTC` | アイテムが最後に開かれたとき。 |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | ドキュメントが信頼された時期、そのマクロが有効かどうか、およびレコードのフラグ。不明なフラグは `MacrosEnabled` を空のままにし、診断を生成します。 |
| `Setting`, `Value` | マクロ設定とその値、およびその意味 (`1 (all macros enabled)`)。信頼できる場所のキー (`Location3`) とその他の値 (`AllowSubFolders=1; Date=…; Description=…`)。 |
| `KeyPath`, `KeyLastWrittenUTC` | キーとその最後の書き込み。 |
| `DataRaw` | デコードされない場合の値のバイト数 (16 進数)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | `*` のないアイテム、または時刻が 1 ではないアイテム、24 バイトではない信頼レコード、`Path` のない場所、間違ったタイプ。 |

## 調査戦略 { #investigation-strategy }

1. `MacrosEnabled` true から始めます。ドキュメント、いつ、誰が作成したか。ファイル (`$MFT`、USN Journal、`Zone.Identifier`)、そのソース (ブラウザーとメールの履歴)、次に実行された内容を見つけます。
2. マクロ設定と信頼できる場所を確認します。すべてのマクロが有効になっている、インターネット マクロがブロックされていない、VBA プロジェクト アクセス、またはユーザーが追加した場所 (`Downloads`、`Temp`、ネットワーク共有内) はデフォルトの機能を弱めます。
3. `LastOpenedUTC` を、LNK ファイル、ジャンプ リスト、およびリムーバブルおよびネットワークの場所を含むドキュメントのパスの RecentDocs とともにタイムラインに配置します。

## さらに詳しい情報 { #further-details }

- [Microsoft: Microsoft 365 ファイル](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6) のマクロを有効または無効にします
- [Microsoft: 信頼できるドキュメント](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: 信頼できる場所の追加、削除、または変更](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
