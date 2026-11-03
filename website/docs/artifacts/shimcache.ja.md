# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## ShimCacheとは { #what-shimcache-is }

Windows ' アプリケーション互換性サブシステムは、互換性/シムの動作を評価する際に、実行可能パスとメタデータをキャッシュします。キャッシュは、ファイルの名前変更、削除、またはリムーバブル ボリュームとの接続が切断されても存続します。そのレイアウトと使用可能なフィールドは、Windows XP から Windows 11 までで異なります。

ShimCache は、履歴パスの検出と、形式に信頼できる実行フラグがある場合、追加の実行サポートに役立ちます。キャッシュされた時間は通常、Windows がファイルを参照または実行した時間ではなく、**ファイルの最終変更時間** です。

<span class="evidence-strength">この証拠から判断できること:</span> Windows が互換性処理のために、このパスを認識しました。エントリ単独では、存在または認識の証拠にとどまります。`Executed` は、対応する形式の場合にのみ使用してください。

## 保存場所と収集 { #location-and-collection }

バイトは `SYSTEM` レジストリ ハイブにあり、通常は次の場所にあります。

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

`SYSTEM`、`SYSTEM.LOG1`、および `SYSTEM.LOG2` を収集します。 Juntei はすべてのコントロール セットを読み取り、可能な場合は `Select\Current` を識別します。

!!! warning "最新のライブエントリがハイブにない可能性があります"
    Windows は、ShimCache の更新をメモリに保持し、シャットダウン/再起動中にそれらをフラッシュできます。実行中のシステムからコピーされたハイブには、最新の観測値が含まれていない可能性があります。キャッシュのレジストリの最終書き込み時間は、キャッシュがいつ保存されたかを説明するのに役立ちますが、エントリごとの実行時間ではありません。

## 分析時の注意点 { #analyst-cautions }

- キャッシュエントリが存在するだけでは、**実行を証明できません**。
- `Executed` は、`Yes`、`No`、または `NA` です。利用可能なセマンティクスは形式/バージョンによって異なります。
- `LastModifiedTimeUTC` は参照ファイルに属し、観察や実行には属しません。
- 順序は便利ですが、普遍的な正確なタイムラインではありません。重複はコントロール セット/スラック間で発生します。
- `FromSlack` エントリはアクティブ キャッシュの外でリカバリされたため、追加の注意が必要です。
- 使用可能なログのないダーティな `SYSTEM` では、現在のレコードが省略される可能性があります。リカバリの概要を読んでください。
- 調査結果は、マルウェアの判定ではなく、疑わしい名前付け/パス パターンを示しています。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `ControlSet` | キャッシュを保持する数値制御セット。どちらかを選択する前に、`IsCurrent` を比較してください。 |
| `CacheEntryPosition` | デコードされたキャッシュ内の位置/順序。時間ではなく、相対的な順序付けに役立ちます。 |
| `Path` | キャッシュされた実行可能ファイル/パッケージのパス。主要な狩猟のピボット。 |
| `LastModifiedTimeUTC` | エントリに保存されるファイルの変更時間 (実行/観察時間ではありません)。 |
| `Executed` | サポートされている形式フラグに応じて、`Yes`、`No`、または `NA`。バージョン コンテキストでのみ使用します。 |
| `Duplicate` | 同じエントリが以前のコントロール セットまたはスラックですでに見られていました。二重カウントの防止に役立ちます。 |
| `SourceFile` | ハイブ/生の値を入力します。ホスト間で不可欠です。 |
| `FromSlack` | エントリはアクティブ リストではなくキャッシュ値のスラックから回復されました。自信が低く、おそらく年齢が高い。 |
| `CacheFormat` | デコードされた Windows/キャッシュ レイアウト。フィールドの可用性とセマンティクスを制御します。 |
| `CacheLastWrittenUTC` | レジストリ キー/キャッシュの保存時間。これは共有コンテキストであり、各エントリの実行時間ではありません。 |
| `InsertFlags`, `ShimFlags` | 生の互換性フラグ。 `Executed` は、サポートされている場合の Juntei の解釈結果です。 |
| `DataSize` | このエントリのデータのサイズ。構造の検証に役立ちます。 |
| `Machine` | 新しい形式 (`x64`、`x86`、`ARM64`) で記録されたアーキテクチャ。 |
| `FileSize`, `LastUpdateTimeUTC` | Windows XP 固有のキャッシュされたメタデータ (存在する場合)。 |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | サポートされているエントリのパッケージ化されたアプリケーション ID。発行元 ID はコンテキストであり、署名証明ではありません。 |
| `Findings` | 再配置されたシステム バイナリや疑わしいステージング パスなどの Juntei 異常コード。リードのみ。 |
| `RegistryPath`, `RegistryValueName` | 正確なハイブのキー/値のソース。検証のために保存します。 |
| `CacheValueSize` | 個々のエントリの `DataSize` ではなく、レジストリ値全体のサイズ。 |
| `IsCurrent` | このエントリのコントロール セットが `SYSTEM\Select\Current` に等しいかどうか。安全に知られていない場合は空です。 |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | コンテキストおよびリカバリ分析用の生の `SYSTEM\Select` コントロール セット番号。 |

JSON/JSONL はさらに、エントリ データ、タグ、パッケージ化されたアプリのサブレコード、構造化された検出結果、キャッシュの問題、および回復の詳細を保存します。

## 調査戦略 { #investigation-strategy }

1. 現在のコントロール セットを優先しますが、他のセットとスラックを履歴証拠として保持します。
2. 検索パスと結果。 `LastModifiedTimeUTC` を実行時間として使用しないでください。
3. ここで、`Executed == Yes` は、Prefetch、SRUM、イベント ログ、またはその他のランタイム トレースで裏付けられます。
4. パスを `$MFT`/USN と照合して、作成、削除、メタデータの履歴を特定します。
5. トランザクション ログが再生されたかどうか、およびライブ キャッシュがフラッシュされなかった可能性があるかどうかを報告します。

## さらに詳しい情報 { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [エリック・ジマーマンの AppCompatCacheParser](https://github.com/EricZimmerman/AppCompatCacheParser)
- [セキュリティ Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: フォレンジック調査におけるアプリケーション互換性キャッシュの活用](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Maxim Suhanov の Windows レジストリ形式仕様](https://github.com/msuhanov/regf)
