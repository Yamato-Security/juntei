# BAM および DAM { #bam-and-dam }

## BAM および DAM とは { #what-bam-and-dam-are }

バックグラウンド アクティビティ モデレータ (BAM) およびデスクトップ アクティビティ モデレータ (DAM) は、アプリケーションがバックグラウンドで実行できる内容を管理する Windows サービス (Windows 10 1709 以降) です。ユーザーごとに、各プログラムが最後に実行された時刻が `SYSTEM` ハイブに保存されます。

- ユーザーごとに 1 つのキー。ユーザーの SID にちなんで名付けられます。
- プログラムごとに 1 つの値。その名前は、デバイス上のプログラムのパス (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`)、またはパッケージ化されたアプリの ID (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`) です。
- 値のデータの最初の 8 バイトは、最後に実行された時刻 (FILETIME) です。

BAM は、プログラムをユーザーとユーザー自身のハイブなしの時間に結び付け、リムーバブル デバイスやネットワーク デバイスから実行されるプログラムをリストします。

<span class="evidence-strength">この証拠から判断できること:</span> このプログラムのプロセスが、このユーザーのセッションで実行され、最後に実行されたのがこの時刻です。BAM は実行回数を記録せず、プログラムの削除後もエントリが残る場合があります。

## 場所 { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

各ユーザーのキーには `Version` 値と `SequenceNumber` 値もあります。それらはプログラムではないので省略されます。 1709 または 1803 からアップグレードされたシステムは、両方のレイアウトを持つことができます。

`SYSTEM` とそのトランザクション ログ、およびシステムの `SAM` と `SOFTWARE` を収集します。これらはユーザーに名前を付けます。

## 分析時の注意点 { #analyst-cautions }

- **プログラムおよびユーザーごとに 1 回。** 最新の実行のみが保持されます。 Prefetch、Amcache およびイベント ログにより、以前の実行が記録されます。
- **最近のアクティビティ.** しばらく実行されていないプログラムのエントリは、Windows の起動時に削除されると報告されているため、BAM には主に最近のアクティビティが表示されます。そこに存在しないプログラムは、ずっと前に実行されていた可能性があります。
- **ドライブ文字ではなくデバイス パスです。** `\Device\HarddiskVolume3` は、`C:` ではなく、Windows 番号が付けられた 3 番目のボリュームです。 `DevicePath` と `Path` は、デバイスをその上のパスから分割します。ドライブ文字を書き込む前に、ボリュームを `MountedDevices`、`$MFT`、またはその他のアーティファクトと照合してください。
- **すべてのプログラムが対象ではありません。** BAM と DAM は、ユーザーのセッション内のプログラムを追跡します。サービスは通常、記録されません。`S-1-5-18` のエントリはセットアップやサインイン画面（`FirstLogonAnim.exe`）、`S-1-5-90-0-<n>` は Desktop Window Manager（`DWM-<n>`）のプロセスであることが一般的です。
- **Control set.** デフォルトでは、最後に使用されたコントロール セット Windows が読み取られます。 `--all-control-sets` はすべてを読み取ります。古いコントロール セットは、現在のコントロール セットが持たなくなったエントリを保持できます。
- **ユーザー名.** ローカル アカウントの名前は、`SAM` (コンピューターの SID とアカウントの RID) に由来します。ドメイン アカウントは、`ProfileList` のプロファイル フォルダーから取得されます。 `UserSource` は (`sam`、`profile_list`、`well_known`) を示します。これらがなければ、SID のみが与えられます。 `SAM` が持たなくなった RID (アカウントが削除された) は、名前が付けられないままになります。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

概要には、各ユーザーとその SID、名前とエントリ数、および最後に実行されたプログラム (新しいものから順) がリストされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath` | ホスト、および `SYSTEM` ハイブ。 |
| `ControlSet`, `IsCurrentControlSet` | エントリが含まれるコントロール セット、および最後に使用された Windows かどうか。 |
| `Moderator` | `bam` または `dam`。 |
| `UserSID`, `User`, `UserSource` | キーの名前の由来となっている SID、ユーザーの名前、およびキーの出所 (`sam`、`profile_list`、`well_known`)。 |
| `Program` | Windows が書き込んだ値の名前: デバイス パスまたはパッケージ化されたアプリの ID。 |
| `DevicePath`, `Path` | デバイス パスの場合、デバイス (`\Device\HarddiskVolume3`) とその上のパス (`\Windows\System32\cmd.exe`)。パッケージ化されたアプリの場合は空です。 |
| `LastRunUTC` | プログラムが最後に実行されたとき (100 ns まで)。 |
| `KeyPath`, `KeyLastWrittenUTC` | ユーザーのキーとその最後の書き込み: そのエントリの最新の更新。 |
| `DataRaw` | 値のバイト数 (16 進数) (時刻を保持しない場合: `REG_BINARY` ではない、または 8 バイト未満)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | 時間を保持しないデータや、読み取れない値。 |

## 調査戦略 { #investigation-strategy }

1. `LastRunUTC` で並べ替えて、システム ボリューム以外のデバイス上のユーザー書き込み可能な場所 (`\Users\`、`Downloads`、`Temp`、`ProgramData`) にあるプログラム、および管理ツールとリモート アクセス ツールを調べます。
2. どのユーザーがそれらを実行したか、および現在のプロファイルと一致しないアカウントを確認してください。
3. Prefetch (実行回数と以前の実行)、UserAssist (エクスプローラーの起動)、Amcache および ShimCache (存在)、`$MFT` (プログラムのファイル)、およびイベント ログと関連付けます。
4. 削除された BAM 値には、`--all-control-sets` および `parse-registry read --recover-deleted` を使用します。

## さらに詳しい情報 { #further-details }

- [libyal winreg-kb: バックグラウンド アクティビティ モデレーター (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: セキュリティ識別子 (既知の SID)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)

範囲外の最後に実行された FILETIME は、`LastRunUTC` を空のままにし、`Problems` に説明を含めて `DataRaw` に元の値を保持します。最近のアクティビティの概要は各ホストを識別し、個別のハイブ/ユーザーの観察を明確に保ちます。
