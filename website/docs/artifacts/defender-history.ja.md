# Microsoft Defender の検出履歴 { #microsoft-defender-detection-history }

## 検出履歴の内容 { #what-the-history-records }

Microsoft Defender は、検出ごとに検出履歴ファイルを 1 つ書き込みます。各ファイルには次の内容が記録されます。

- 脅威の識別子、名前、重大度、カテゴリ
- Defender が行った処理と、その成否
- 脅威が実行中だったかどうか
- 関係したユーザーとプロセス、および検出した仕組み
- 隔離エントリ (ある場合)
- 脅威が見つかったすべてのリソース: ファイル、アーカイブ内のファイル、ダウンロード、プロセス、レジストリ値、サービス、コマンドライン、振る舞い

Defender は、隔離しなかった脅威についてもこのファイルを書き込みます。検出しただけの脅威、許可された脅威、削除に失敗した脅威も含まれます。Windows セキュリティの保護の履歴と、WMI の `MSFT_MpThreatDetection` クラス (`Get-MpThreatDetection`) は、これらのレコードを表示します。Orange Cyberdefense がファイルの値を変更したところ、WMI は変更後の値を返しました。

<span class="evidence-strength">この証拠から判断できること:</span> Defender が、記録された状態と時刻で、指定の脅威を記載のリソースで検出したことを記録しました。脅威名は製品の判定です。検出レコードだけでは、脅威が実行されたことも、完全に削除されたことも示せません。

## 保存場所と収集 { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

各ファイルの名前は、その `DetectionID` です。`DetectionHistory` フォルダー全体を収集してください。ディレクトリ入力と `auto-parse` は、`DetectionHistory` フォルダー内 (深さは問いません) のファイルのうち、検出履歴ファイルとして始まるものをすべて読み取ります。そのようなフォルダーの外に置かれたコピーには `-f` を使用してください。Defender は、設定された日数が過ぎるとスキャン履歴から項目を削除します (`Set-MpPreference` の `ScanPurgeItemsAfterDelay`)。そのため、古い検出は残っていない場合があります。

隔離は別のアーティファクトです。[Defender の隔離データ](defender-quarantine.md) を参照してください。検出の `QuarantineID` は、隔離の出力の `EntryID` と同じ表記 (大文字、波かっこなし) なので、両者を結合できます。

## アナリストへの注意事項 { #analyst-cautions }

- 検出、修復の試行、削除の成功は別々の状態です。`ThreatStatus` (`Detected`、`Quarantined`、`Removed`、`Allowed`、`RemoveFailed` など) は、`ThreatStatusErrorCode` と合わせて読んでください。最後の状態変更が成功した場合、このエラー コードは 0 です。
- `ExecutionStatus` は、検出時点での Defender の見方です (`Blocked`、`Allowed`、`Executing`、`NotExecuting`、`Unknown`)。脅威が実行された証拠にも、実行されなかった証拠にもなりません。Prefetch、Amcache、イベント ログ、プロセスのテレメトリで裏付けてください。
- `ThreatName` は、その時点のシグネチャ名です。誤検知や、後での再分類があり得ます。EICAR などのテストファイルも、実際の脅威と同じように報告されます。
- `User` は、Defender が記録した `DomainUser` の値です (`UserSource` は `DomainUser`)。空の場合や、サービス アカウントの場合、ファイルを作成したユーザーとは別のユーザーの場合があります。
- Defender は 1 つの脅威について複数のファイルを書き込むことがあり、Juntei はそのすべてを保持します。コピーや復元されたファイルのように、2 つのファイルが同じ `DetectionID` を持つこともあります。`DetectionID` と `SourceFile` を組み合わせてグループ化してください。
- 意味がわかっていない値は、解釈せずに保持します。そのセクションと位置は `UnknownValues` にあります。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

既定のファイル形式は CSV です。`-t csv`、`-t json`、`-t jsonl` を指定して `-o` を指定しない場合、レコードは標準出力に書き込まれます。どちらも指定しない場合は、概要だけが表示されます。既存の出力を上書きするには `-C` を追加してください。`-C` を指定しても、入力ファイルの上に出力が書き込まれることはありません。

Juntei は、各検出のリソースごとに 1 つのレコードを書き込みます。リソースのない検出には、リソースのフィールドが空のレコードを 1 つ書き込みます。1 つの検出のレコードは同じ `DetectionID` を持ち、保存順に `ResourceIndex` で番号が付きます。

破損があっても処理は止まりません。途中で切れたファイルは、破損箇所より前に読み取れた値を保持し、`Problems` に何が欠けているかを示します。`DetectionHistory` フォルダー内の検出履歴ファイルではないファイルは、報告したうえでスキップします。ほかのファイルは引き続き読み取ります。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Record` | `resource`、またはリソースのない検出の場合は `detection`。 |
| `InitialDetectionTime` | Defender が脅威を最初に検出した時刻 (UTC)。ファイルの値が 0 の場合は空です。時刻ではない値は `UnknownValues` に保持します。 |
| `LastThreatStatusChangeTime` | 脅威の状態が最後に変わった時刻 (UTC)。 |
| `RemediationTime` | Defender が脅威を修復した時刻 (UTC)。試行の時刻であり、削除の証拠ではありません。`ThreatStatus` を確認してください。 |
| `ThreatName`, `ThreatID` | Defender のシグネチャ名と数値の識別子 (`Get-MpThreatCatalog`)。 |
| `Severity`, `SeverityID` | `Low`、`Moderate`、`High`、`Severe`、または `Unknown` と、保存されている数値。 |
| `Category`, `CategoryID` | `TROJAN`、`TOOL`、`BEHAVIOR` などの脅威のカテゴリと、保存されている数値。 |
| `ThreatStatus`, `ThreatStatusID` | 脅威に対する Defender の状態。文書化された名前のない数値は、`ThreatStatus` を空にして保持します。 |
| `ThreatStatusErrorCode` | 最後の状態変更のエラー コード。0 は成功を意味します。 |
| `ExecutionStatus`, `ExecutionStatusID` | 検出時に Defender が脅威の実行を認識していたかどうか。 |
| `DetectionSource`, `DetectionSourceTypeID` | 検出した仕組み: `Real-time`、`IOAV` (ダウンロードと添付ファイル)、`User`、`System`、`AMSI` など。 |
| `User`, `UserSource` | Defender が記録した `DomainUser` の値と、その根拠 (`DomainUser`)。ファイルにユーザーの記載がない場合は空です。 |
| `Process` | 関係したプロセス (`ProcessName`)。`Unknown` の場合もあります。 |
| `ResourceIndex`, `ResourceCount` | このリソースの位置と、検出に含まれるリソースの数。 |
| `ResourceType` | `file`、`containerfile`、`webfile`、`process`、`regkeyvalue`、`service`、`CmdLine`、`behavior`、`amsi` など、Defender が付けた名前。 |
| `Path` | リソース: パス、アーカイブ内のファイル (`archive.zip->member`)、レジストリ値、コマンドライン、またはプロセス。`webfile` は、URL とダウンロードしたプロセス (`pid:…,ProcessStart:…`) も示します。正確な表記を保持してください。 |
| `ResourceFlags` | リソースのフラグ (16 進数)。意味は文書化されていません。 |
| `TrackingStartTime` | リソースの `ThreatTrackingStartTime`。plaso が検出の日時に使う値で、整数を FILETIME (UTC) として読み取ったものです。 |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | リソースの追跡データにあるハッシュとサイズ。Defender がスキャンした時点のファイルを表します。 |
| `TrackingID` | リソースの `ThreatTrackingId`。 |
| `OtherTracking` | そのほかのすべての追跡値。保存順に `Key=Value` の形式です。振る舞いの検出の `ThreatTrackingScanSource` や `ThreatTrackingBMContextRich` などがあります。 |
| `QuarantineID` | 隔離エントリ (`Quarantine\Entries\{GUID}`)。ファイルの値がゼロの GUID の場合は空です。 |
| `DetectionID` | 検出の GUID で、ファイルの名前でもあります。一致しない場合は `Problems` で報告します。 |
| `MagicVersion` | ヘッダーの `Magic.Version:1.2`。Windows Server 2016 の一部のファイルなど、これを含まずに書き込まれたファイルでは空です。 |
| `UnknownValues` | 意味がわかっていない値。`position=value` の形式です (例: `6.13=NT AUTHORITY\SYSTEM`)。状態の後のカウント付きリストは `2.list` です。 |
| `SourceFile` | 読み取ったファイル。入力ディレクトリからの相対パス、`-f` の場合はファイル名です。 |
| `Problems` | 破損、欠けている値、予期しない型、または一致しないファイル名。省略しないでください。 |

JSON と JSONL は、同じフィールドを型付きの値で保持します。数値は数値のままで、ない値は `null` です。`OtherTracking` の代わりに `Tracking` があり、すべての追跡値を Defender のキー、型、値とともに保持します。`UnknownValues` は、型付きの位置のリストになります。`CountedList` はカウント付きリストを保持します。保存されている FILETIME は、読み取った時刻と並べて元のティック数を保持します。

## タイムラインでの扱い { #timeline-behavior }

`auto-parse` は、レコードを `SecurityProducts/DefenderHistory.csv` (または `.json`/`.jsonl`) に保存し、統合された `-j` タイムラインに追加します。`super-timeline` は、各リソースのレコードの `InitialDetectionTime`、`LastThreatStatusChangeTime`、`RemediationTime` からイベントを作成し、状態、エラー コード、実行状態、検出の仕組みをイベントの詳細に含めます。`TrackingStartTime` は整数として保存されているため、`--include-interpreted-times` を指定した場合にだけイベントになります。ファイル自体の時刻は使用しません。使用できる時刻がないレコードは解析済みの出力に残り、時刻なしとして数えられます。

## 調査の進め方 { #investigation-strategy }

1. `ThreatName`、`Path`、`SHA256`、`InitialDetectionTime`、`ThreatStatus` から始めます。
2. 脅威が削除されたと結論付ける前に、`ThreatStatusErrorCode` と `Problems` を確認します。
3. `QuarantineID` を隔離の `EntryID` と結合し、隔離されたバイトを復元します。
4. ハッシュとパスを `$MFT`、USN ジャーナル、Prefetch、Amcache、ブラウザーの履歴、イベント ログ(Defender の操作ログ: イベント ID 1116、1117、1118、1119) に展開します。
5. `webfile` リソースについては、URL とプロセスの開始を使って初期侵入を調査し、ブラウザー、プロキシ、DNS の証拠で検証します。

## テストしたバージョン { #tested-versions }

テストには、ほかのツールが独立して読み取った結果のある公開ファイルを使用しています。

- Orange Cyberdefense の Windows 10 のサンプル (Defender 4.18.24090.11)。WMI が返した値と照合しています。
- plaso の Windows のテスト ファイル 2 つ。plaso の `windefender_history` パーサーと照合しています。
- DFIRArtifactMuseum の APT シミュレーター VM のファイル 14 個。
- Windows Server 2016 の一部のファイルのように、ヘッダーのバージョンなしで書き込んだコピー。
- 途中で切れたファイル、空のファイル、別の種類のファイル。

## 詳細情報 { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [Orange Cyberdefense wddh-parser](https://github.com/cert-orangecyberdefense/wddh-parser)
- [plaso Defender history parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: MSFT_MpThreatDetection class](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: Microsoft Defender Antivirus event IDs](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
