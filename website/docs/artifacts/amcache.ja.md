# Amcache { #amcache }

## Amcacheとは { #what-amcache-is }

Amcache は、レジストリ ハイブに保存されている Windows アプリケーション互換性インベントリです。最新のシステムでは、実行可能ファイルは `InventoryApplicationFile` に記録され、プラグ アンド プレイ デバイスは `InventoryDevicePnp` に記録されます。古い Windows バージョンは従来の `Root\File` レイアウトを使用します。

Amcache は、元のファイルが消失した後もフル パス、SHA-1、ファイル サイズ、PE メタデータ、バージョン情報、発行元のテキスト、リンクの日付、およびレジストリのタイムスタンプを保存できるため、実行可能ファイルの調査にとって貴重です。デバイス レコードは、接続されているハードウェア、ドライバー、インストール日、および識別子を識別できます。

<span class="evidence-strength">この証拠から判断できること:</span> Windows が、記録されたファイルまたはデバイスをインベントリに登録しました。実行可能ファイルのレコードは、**存在の証拠であり、実行の証明ではありません**。

## 保存場所と収集 { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

対象の Windows 環境から、ハイブと隣接する `Amcache.hve.LOG`、`.LOG1`、`.LOG2` を収集してください。ハイブがダーティ状態の場合、`parse-registry amcache` は利用可能なトランザクションログをメモリ上で自動再生してから解析し、証拠ファイルは変更しません。復元の詳細はパーサーログまたは端末に記録され、レコードを標準出力に送る場合は標準エラー出力に表示されます。ログが欠落または使用不能の場合、レコードが不完全になったり、整合性の取れていないハイブを解析できなかったりする可能性があります。

## 分析時の注意点 { #analyst-cautions }

- Amcache 実行可能エントリは、ユーザーがファイルを起動したことを証明するものではありません。
- `Publisher` は、Windows によって記録された PE/バージョン リソースのメタデータです。収集時にファイルに有効な Authenticode 署名があったことを証明するものではありません。
- ハッシュが存在しない可能性があり、レガシー/モダンのスキーマは同一の値を公開しません。
- `LinkDate` は PE ヘッダーから取得され、偽造、コピー、または再現可能なビルドによって生成される可能性があります。
- キーの最終書き込み時刻は、レジストリ レコードの更新を表し、必ずしもダウンロード、作成、最初の実行、または最後の実行を表すものではありません。
- パスは、取り外し可能なボリュームまたは存在しないボリュームを参照できます。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

CSV 出力には、概要、結果、ファイル、およびデバイス テーブルが含まれます。 `auto-parse` の場合、これらは各コンピュータの `FileExistence/` ディレクトリの直下にある `Registry-Amcache-Summary.csv`、`Registry-Amcache-Findings.csv`、`Registry-Amcache-Files.csv`、および `Registry-Amcache-Devices.csv` であり、`Registry-Amcache-logs.txt` はテーブルの横にあります。複数のハイブでは、拡張子の前に `-Source1`、`-Source2` などが追加されます。マニフェストの `OutputFiles` には正確なパスがリストされています。スタンドアロン CSV エクスポートでは、ネイティブ テーブル名と要求された出力場所が保持されます。 JSONL レコードは、`RecordType` 値 `summary`、`finding`、`file`、および `device` を使用します。

## ファイル出力フィールド { #file-output-fields }

| フィールド | アナリストはそれをどのように使用すべきか |
|---|---|
| `KeyName`, `SourceKeyPath` | 正確なソース レジストリ レコードとスキーマ ブランチを特定します。再現性のために保存します。 |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | レジストリ レコードの更新時刻 (読み取り可能な生の形式)。実行時間というラベルを付けないでください。 |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | レガシー スキーマの親ボリューム キー時間。ファイルのアクション時間ではなく、コンテキストに沿ったものです。 |
| `FileID` | Amcache の記録されたファイル識別子。正確に保持されます。常に SHA-1 であると想定しないでください。 |
| `SHA1`, `SHA256` | Defender、脅威インテリジェンス、EDR、およびファイル リポジトリへの強力なピボット。 Null は記録されていないことを意味します。 |
| `Name`, `OriginalFileName`, `Description` | PE/バージョン名を記録しました。実際のパスと比較してマスカレードを検出します。メタデータは攻撃者によって制御されます。 |
| `FullPath` | 最も便利なロケーション ピボット。大文字と小文字を慎重に正規化しますが、元の証拠文字列は保持します。 |
| `LongPathHash` | Windows の パス関連の識別子。コンテンツのハッシュではなく、レコードの照合に役立ちます。 |
| `ProgramID` | 利用可能な場合は、ファイルをアプリケーション インベントリ レコードにリンクします。 |
| `Publisher` | 記録された発行元の文字列です。署名の検証結果ではなく、メタデータとして扱ってください。 |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | バージョンリソースコンテキスト。違いやありえない値はマスカレード分析をサポートする可能性があります。 |
| `BinaryType` | 記録されたアーキテクチャ/型コンテキスト。ファイルが復元されたときに PE 分析と比較します。 |
| `LinkDate` | PE 記録されたコンパイル/リンクのタイムスタンプ。懐疑と裏付けがある場合にのみ役立ちます。 |
| `Size` | 記録サイズ。 `$MFT`、Defender コンテンツ、または復元されたファイルと比較します。 |
| `Language` | PE 言語識別子。帰属ではなく、文脈上の手がかりが弱い。 |
| `IsPE-File`, `IsOS-Component` | Windows の には分類フラグが記録されます。偽の値は無害であることを証明するものではありません。 |
| `USN` | NTFS は、Amcache によって記録されたシーケンス値を更新します。残っている USN レコードにピボットするのに役立ちます。 |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | パッケージ化されたアプリケーション/コンポーネントをファイルに関連付けます。 |
| `RegistryValues` | タイプ、デコードされた値、生の 16 進数を含むすべての元のレジストリ値。型付きフィールドが存在しない場合、または将来/従来のフィールドに使用します。 |

## デバイス出力フィールド { #device-output-fields }

| フィールドグループ | フィールドと解釈 |
|---|---|
| ソース | `KeyName`、`SourceKeyPath`、`KeyLastWrittenUTC`、`KeyLastWrittenFiletime`: 正確なデバイス レコードとレジストリの更新時間。 |
| アイデンティティ | `Description`、`BusDescription`、`Manufacturer`、`Model`: 人向けのデバイスの説明。ベンダー文字列は汎用的なものにすることができます。 |
| ハードウェア識別子 | `HWID`、`CompID`、`Class`、`ClassGUID`、`ContainerID`、`DeviceInterfaceClasses`: 最強のデバイス ファミリとインスタンス ピボット。 |
| 州 | `DeviceState`、`ProblemCode`、`ParentID`、`MatchingID`、`Enumerator`: Windows 列挙体と関係コンテキスト。 |
| ドライバー | `DriverID`、`DriverName`、`DriverPackageStrongName`、`DriverVerDate`、`DriverVerVersion`、`Provider`、`Service`、`INF`: ドライバー パッケージとサービスの関連付け。インストールされているファイルとサービスを関連付けます。 |
| インストール | `InstallDate`、`FirstInstallDate`、`InstallState`、`ExtendedInfs`: 記録されたインストール ライフサイクル。日付が欠けていても、デバイスが取り付けられていなかったという証拠にはなりません。 |
| フィルターとスタック | `LowerClassFilters`、`UpperClassFilters`、`LowerFilters`、`UpperFilters`、`StackID`、`LocationPaths`: 高度なドライバー スタック コンテキスト。予期しないフィルター ドライバーは検討する必要があります。 |
| 生保存 | `RegistryValues`: 検証用の完全な型付き/生のソース値とサポートされていないフィールド。 |

CSV の配列および生の値の列では、`JSON` 接尾辞が使用されます (たとえば、`HWID-JSON` および `RegistryValuesJSON`)。セルには JSON が含まれていますが、JSON/JSONL はネイティブ配列を保持します。

## 調査戦略 { #investigation-strategy }

1. `FullPath`、`SHA1`、`Name`、および `USN` を軸にします。
2. 実行をサポートするために、一致する Prefetch を探します。
3. `$MFT` および USN ジャーナルを使用して、ファイルの作成、名前変更、削除、サイズを確立します。
4. 発行元/バージョンのメタデータを、独自に復元された PE と比較し、署名を検証します。
5. Defender 隔離ログとイベント ログで検出または削除を確認してください。

## さらに詳しい情報 { #further-details }

- [Juntei の ys-amcache-parser 出力スキーマ](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [Regipy Amcache プラグイン](https://github.com/mkorman90/regipy)
