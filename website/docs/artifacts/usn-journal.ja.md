# USN ジャーナル (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## USN ジャーナルとは { #what-the-usn-journal-is }

NTFS は、ファイルとディレクトリの変更を更新シーケンス番号 (USN) 変更ジャーナルに記録します。レコードには、作成、データ拡張/上書き、属性またはセキュリティの変更、名前変更、閉じる、および削除を記述することができます。ファイル参照、親参照、名前、理由、属性、タイムスタンプ、および単調増加する USN が保存されます。

このジャーナルは、特にファイルが削除された場合に、攻撃者のファイル操作のバーストを再構築するのに優れています。変更されたバイトではなく、メタデータ/コンテンツが変更された **** が記録されます。

<span class="evidence-strength">この証拠から判断できること:</span> NTFS が、そのタイムスタンプに、参照されたファイルについて、記載の変更理由を記録しました。理由フラグは、`CLOSE` レコードまで蓄積されることがあります。

## 保存場所と収集 { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` は、NTFS メタデータの名前付きストリームです。 raw 対応のフォレンジック ツールを使用して収集します。 USN レコードには完全なパスではなく親ファイル参照が保存されるため、同じボリュームの `$MFT` も収集します。

## 分析時の注意点 { #analyst-cautions }

- ジャーナルは円形であり、サイズが制限されています。古い記録は上書きされます。
- `FILE_DELETE` は、NTFS が削除を処理したことを示しています。コンテンツは他の場所で回復可能なままになる可能性があります。
- `DATA_OVERWRITE` などの理由では、バイト、プロセス、またはユーザーの責任が明らかになりません。
- いくつかの理由を 1 つのレコードに組み合わせることができます。 `CLOSE` は、多くの場合、蓄積された変更の終了を示します。
- パスが再構築されます。親ディレクトリの名前が変更、削除、または再利用されている可能性があります。
- ファイル参照のシーケンス番号は重要です。エントリ番号だけでは、間違った世代に接続する可能性があります。
- Carving は、元のジャーナル コンテキストなしで有効なレコードを復元できます。その違いを報告します。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## CSV 出力フィールド { #csv-output-fields }

| フィールド | 解釈 |
|---|---|
| `Name` | この USN レコードに格納されているファイル/ディレクトリ名。これは完全なパスではありません。 |
| `Extension` | `Name` から解析された便利な拡張機能。これを信頼できるコンテンツ タイプとして使用しないでください。 |
| `EntryNumber`, `SequenceNumber` | ファイルの NTFS 参照。両方を使用してレコードの世代を識別します。 |
| `ParentEntryNumber`, `ParentSequenceNumber` | 変更時の親ディレクトリの NTFS 参照。 |
| `ParentPath` | Juntei の時間に応じて再構築された親パス。マーカーが不明な場合は、祖先を解決できなかったことを意味します。 |
| `UpdateSequenceNumber` | USN の順序値。安定したソートと Amcache/MFT ピボットに役立ちます。 |
| `UpdateTimestamp` | V2/V3 レコードに保存されるタイムスタンプ。刻まれたコンテキストのタイムスタンプを、他のレコードと同じ検証で扱います。 |
| `UpdateReasons` | `|` で結合された 1 つ以上の名前付き変更フラグ。組み合わせと `CLOSE` を一緒に解釈します。 |
| `FileAttributes` | 変更時に記録された NTFS 属性。不明なビットは 16 進数のままです。 |
| `OffsetToData` | 入力内のバイト オフセット。検証と記録の彫刻に不可欠です。 |
| `SourceFile` | 日記/画像を入力します。ホストとボリュームを区別します。 |
| `SourceInfo` | 変更がジャーナルにどのように入力されたかを説明するソース フラグ。通常、帰属ではなく文脈に沿ったものです。 |
| `SecurityID` | NTFS レコードのセキュリティ識別子インデックス。ユーザー SID ではありません。 |
| `Version` | USN レコードのバージョン。 Juntei は V2/V3 を CSV に書き込みます。 JSON は、V4 範囲追跡エクステントを保持します。 |

JSON/JSONL は、128 ビットの V3 参照を正しく表し、V4 範囲追跡レコードとそのエクステントを保持します。下位レベルのバージョン固有のフィールドが重要な場合に使用します。

## 調査戦略 { #investigation-strategy }

1. 時間、名前、拡張子、親パス、またはファイル参照でフィルターします。
2. `UpdateSequenceNumber` および時間で並べ替えると、正確なジャーナルの順序が表示されます。
3. 移動/名前変更のために `RENAME_OLD_NAME` と `RENAME_NEW_NAME` をペアにします。
4. `$MFT` を使用して、レコードの状態、パス、サイズ、タイムスタンプを検証します。
5. イベント ログ、EDR、Prefetch、サービス/タスク、または Defender からのプロセスの属性を関連付けます。

## さらに詳しい情報 { #further-details }

- [Microsoft: 変更仕訳帳](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: 変更ジャーナル理由フラグ](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [ys-ntfs-parser の usn クレート](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
