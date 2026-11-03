# アプリケーションの互換性 { #application-compatibility }

## Windows が保持するもの { #what-windows-keeps }

ShimCache (`parse-registry shimcache`) および Amcache (`parse-registry amcache`) に加えて、Windows のアプリケーション互換性機能により、これらは `Microsoft\Windows NT\CurrentVersion\AppCompatFlags` の下に保持されます。

| ソース | ハイブとキー | それが示すもの |
|---|---|---|
| `layers` | `SOFTWARE` (および `Wow6432Node`) および `NTUSER.DAT` `Layers` | 互換性レイヤーで実行するように設定されたプログラム: `RUNASADMIN` (常に管理者として)、`WIN7RTM`、`HIGHDPIAWARE` など |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 以降) | プログラム互換性アシスタントがユーザーの実行を確認したプログラム |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | Windows 7 でも同じです |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | 追跡対象モジュールをロードしたプログラムと、最後にロードした時期 |
| `cit_system` | `SOFTWARE` `CIT\System` (Windows 7 ～ 初期 Windows 10) | 圧縮された CIT データベース内のプログラム パスとコマンド ライン |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | `sdbinst` でインストールされるカスタム シム データベース: ファイル、説明、インストール時間 |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | 各プログラムに適用されるカスタム shim データベースとそのインストール時間 |

<span class="evidence-strength">この証拠から判断できること:</span> 互換性レイヤーと互換性データベースは、プログラムの動作を変更する設定です。Program Compatibility Assistant と CIT のエントリは、このユーザーまたはシステムでプログラムが実行された、あるいはモジュールが読み込まれたことを示します。

カスタム shim データベースは永続化および注入手法 (MITRE ATT&CK T1546.011) です。shim は DLL をプログラムにロードしたり、プログラムが実行されるたびに別の shim を開始したりできます。

## 場所 { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

`SOFTWARE`、すべての `NTUSER.DAT` とそのトランザクション ログを収集し、`.sdb` ファイルに shim データベースのパス名 (通常は `C:\Windows\AppPatch\Custom\`) を収集します。

## 分析時の注意点 { #analyst-cautions }

- **CIT 拡張は制限されています。** 64 MiB を超える非圧縮を宣言する値は、診断付きの生バイトとして保持されます。不正な圧縮とサイズの不一致が報告されます。


- **プログラム互換性アシスタントのエントリの実行時間はありません。** `Store` データ (`SACP`、`DataRaw` に保持) には 1 回の時間がありますが、Windows 8.1、10 および11 これは、ハイブのすべてのプログラムと他のシステムで同じです。プログラムの実行時ではありません。 `Store` キーの最後の書き込みのみが、最新の変更の日付を示します。
- **リムーバブル メディア上のプログラム** には、`SIGN.MEDIA=<id> <path>` という名前が付けられます。`Path` はメディア上のパスで、`Details` はメディア ID を持ちます。
- **CIT データベース** は LZNT1 圧縮されており、その構造は公的に文書化されていません。Juntei は、それぞれで見つかったプログラム パスとコマンド ラインをデータベースごとに 1 回、カウントや回数なしでリストします。コマンド ラインはそれ自体のレコードであり、プログラムに関連付けられていません。 Windows 10 以降はここにほとんど保存されません。
- **Shim データベース。** すべての `custom_sdb` および `installed_sdb` レコードを確認します。正規のソフトウェアがレコードをインストールすることはほとんどありません。 `.sdb` ファイル自体に、シムの機能が記載されています。
- ユーザーによって設定された **Layers** は、そのユーザーにのみ適用されます。 `SOFTWARE` を皆さんに。 `RUNASADMIN` は、プログラムが毎回昇格を要求するようにします。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

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
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

この概要には、カスタム シム データベース、レイヤーを含むプログラム、および CIT モジュールの最後の用途がリストされています。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath`, `HiveType` | ホスト、ハイブ ファイル、および `SOFTWARE` または `NTUSER.DAT`。 |
| `User`, `UserSID` | ユーザー ハイブのプロファイル フォルダー名、およびシステムの `ProfileList` からの SID。 |
| `Source`, `Scope` | `layers`、`pca_store`、`pca_persisted`、`cit_module`、`cit_system`、`installed_sdb`、`custom_sdb`; `machine` または `user`。 |
| `Path` | プログラム (値の名前)、CIT データベース内のパスまたはコマンド ライン、または shim データベースが適用されるプログラム。 |
| `Layers` | 互換性レイヤー (`~ RUNASADMIN`) と書かれています。 |
| `LastSeenUTC` | プログラムが最後に CIT モジュールをロードしたとき。 |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | シム データベースの GUID、ファイル、およびインストール時間。 |
| `Details` | CIT モジュールの名前、CIT データベースの値の名前、プログラムのリムーバブル メディア ID (`SignMedia`)、shim データベースの `DatabaseDescription` および `DatabaseType`。 |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | レコードの出所、キーが最後に変更されたとき、およびキーが `Wow6432Node` の下にあるかどうか。 |
| `DataRaw` | `Store` 値の `SACP` データ、または本来の値 (16 進数) ではないデータ。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | デコードまたは読み取りができなかったデータ。 |

## 調査戦略 { #investigation-strategy }

1. すべてのカスタム shim データベース、つまり `.sdb` ファイル、インストール時間、および適用されるプログラムを確認します。 Prefetch、Amcache の `sdbinst.exe` と関連付けて、作成イベントを処理します。
2. プログラム互換性アシスタントのエントリ内のユーザーが書き込み可能なフォルダー内のプログラムと、リムーバブル メディアからのプログラムを探します。
3. 通常とは異なるプログラム上の `RUNASADMIN` およびその他のレイヤーを確認してください。
4. 実行時間については、ShimCache、Amcache、BAM、Prefetch、および UserAssist と相関させます。

## さらに詳しい情報 { #further-details }

- [Microsoft: アプリケーション互換性データベース](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: アプリケーション Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
