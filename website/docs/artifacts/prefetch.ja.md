# Prefetch { #prefetch }

## Prefetchとは { #what-prefetch-is }

Windows Prefetch ファイルには、アプリケーションの起動を最適化するために使用される情報が記録されます。プログラム固有の `.pf` ファイルには通常、実行可能ファイルの名前とハッシュ、実行回数、最近の実行時間、ボリューム情報、ディレクトリ、および起動時に参照されるファイルが保存されます。

サポートされている Windows バージョンの場合、有効なアプリケーション Prefetch レコードは、通常、実行可能ファイルが実行されたことの強力な証拠となります。これは、イベント ログ以外の実行アーティファクトの中で最も有用なものの 1 つです。

<span class="evidence-strength">この証拠から判断できること:</span> OS の挙動とパーサーの妥当性を考慮したうえで、指定の実行可能ファイルが、記録された直近の実行時刻に起動したと判断できます。

## 保存場所と収集 { #location-and-collection }

```text
C:\Windows\Prefetch\*.pf
```

Juntei は、圧縮された Windows 8+ ファイルを含む形式バージョン 17 ～ 31 (Windows XP ～ Windows 11) をサポートします。実行可能ファイル名のハッシュと隣接するレコードを比較できるように、ディレクトリ全体を収集します。

## 分析時の注意点 { #analyst-cautions }

- Windows は、古いバージョンでは履歴ランタイムを 1 つだけ保存しますが、新しいバージョンでは最大 8 つまで保存します。 `RunCount` は、使用可能なタイムスタンプの数を超える可能性があります。
- Prefetch は、一部のシステム/役割では無効化、削除、除外、または使用不可にすることができます。存在しないことは、プログラムが実行されなかったという証拠にはなりません。
- ロードされたファイルのリストには、起動時の依存関係とアクセスが記述されています。これらは、リストされたすべてのファイルが悪意のあるもの、意図的に開かれたもの、または完全に読み取られたものであることを証明するものではありません。
- Prefetch ヘッダー内の実行可能ファイル名には長さの制限があります。Juntei は SCCA パスハッシュを使って、同じ名前のロードされたパスを区別します。複数の候補を区別できない場合はヘッダーの名前を保持して PathAmbiguous を設定し、ロードされたファイルの一覧は引き続き確認できます。ハッシュが一致しない候補が 1 つだけの場合は、そのパスを使い、PathValidated を false にします。ホスティング プロセスはコマンドラインのデータをハッシュに含めることがあるため、検証されていないパスが誤りとは限りません。
- `.pf` ソース ファイルのファイル システム タイムスタンプは、埋め込まれたランタイムとは別のものです。
- `Findings` は狩猟観察です。インストーラーと正規の管理者はステージング パスを使用することがよくあります。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.ExecutableName? | test("powershell|rundll32"; "i"))'
```

## 標準の CSV フィールド { #standard-csv-fields }

| フィールド | 解釈 |
|---|---|
| `SourceFile` | `.pf` を入力します。ホストを区別し、実行可能ファイル名を重複させるために保持します。 |
| `ExecutableName` | ヘッダーの実行可能ファイル名。ほとんどの検索では大文字と小文字が区別されません。元のスペルを保持します。 |
| `ExecutablePath` | 可能な場合、Juntei の解決されたロードされたパス。 `Files` および他のアーティファクトに対してそれを確認します。 |
| `PathValidated` | 選択したパスが SCCA パスハッシュと一致します。パスの帰属を確認する値であり、ファイルの内容や安全性を証明するものではありません。 |
| `PathAmbiguous` | ロードされた候補が複数あり、パスを確定できません。`ExecutablePath` にはヘッダーの名前を保持します。`Files` と他のアーティファクトで確認してください。 |
| `PrefetchHash` | ハッシュ Windows は `.pf` ファイル名で使用されます。これは暗号化されたファイルコンテンツのハッシュではありません。 |
| `FormatVersion` | Prefetch 構造バージョン。利用可能なフィールドとタイムスタンプ数を決定します。 |
| `RunCount` | Windows によって記録された累積カウンター。正確な履歴ではなく、頻度に役立ちます。 |
| `LastRun` | 最近保持された実行時間。 |
| `PreviousRun1` … `PreviousRun7` | 以前に保持された実行時間 (新しいものから古いもの)。形式/履歴に不足している場合は空です。 |
| `VolumeCount` | 記録されたソース ボリュームの数。 |
| `VolumeDevicePaths` | 起動時に使用されるボリュームの NT デバイス パス。シリアルと関連付けます。 |
| `VolumeSerialNumbers` | ボリューム識別子。リムーバブルまたは再フォーマットされたドライブの証拠をリンクするのに役立ちます。 |
| `VolumeCreated` | 実行可能ファイルの作成時間ではなく、ボリュームの作成時間を記録しました。 |
| `DirectoryCount` | 記録されたディレクトリ文字列の数。 |
| `FileCount` | 記録されたロードされたファイル参照の数。 |
| `Findings` | 再配置されたシステム バイナリや疑わしい実行パスなどの Juntei の観察。手動で確認します。 |
| `Files` | 利用可能な NTFS 参照を含むロード/参照されたファイル。優れた DLL/パス ピボット、ファイルの意図の証明ではありません。 |
| `Dir` | 起動に関連付けられたディレクトリ文字列。ロードされた個々のファイルのパスが不完全な場合に便利です。 |

## タイムライン CSV フィールド (`-T`) { #timeline-csv-fields-t }

| フィールド | 解釈 |
|---|---|
| `RunTime` | 1 つの保持された Prefetch 実行時間。 |
| `ExecutableName`, `ExecutablePath` | プログラム ID/パス コンテキスト。 |
| `PathValidated` | 選択したパスが SCCA パスハッシュと一致します。パスの帰属を確認する値であり、ファイルの内容や安全性を証明するものではありません。 |
| `PathAmbiguous` | ロードされた候補が複数あり、パスを確定できません。`ExecutablePath` にはヘッダーの名前を保持します。`Files` と他のアーティファクトで確認してください。 |
| `PrefetchHash` | 実行可能ファイル名を共有するバリアント/パスを区切ります。 |
| `RunCount` | Prefetch ファイルからのカウンター。これはタイムラインの各行で繰り返されます。 |
| `SourceFile` | 検証のために `.pf` をソースします。 |

JSON/JSONL は、ボリューム、ディレクトリ、ロードされたファイル、および NTFS ファイル参照を、コンマ結合された CSV セルではなく構造化配列として保存します。

## 調査戦略 { #investigation-strategy }

1. `-T` を使用して、保持された実行をタイムラインに追加します。
2. 実行可能ファイル名と Prefetch ハッシュのグループ。同じ名前に異なるパス/ハッシュを含めることができます。
3. `ExecutablePath`、ステージング パス、再配置されたシステム バイナリの検出結果、およびロードされた DLL を確認します。
4. ターゲットの MFT 参照とファイル パスを `$MFT`/USN レコードと照合します。
5. 実行時間をイベント ログ、SRUM、ネットワーク テレメトリ、サービス、タスク、ユーザー アーティファクトと関連付けます。

## さらに詳しい情報 { #further-details }

- [ys-プリフェッチ-パーサー](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [セキュリティ Ronin プリフェッチ - フォレンジック](https://github.com/SecurityRonin/prefetch-forensic)
- [エリック・ジマーマンの PECmd](https://github.com/EricZimmerman/PECmd)
