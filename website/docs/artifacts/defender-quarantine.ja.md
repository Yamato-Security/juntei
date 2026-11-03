# Microsoft Defender 隔離 { #microsoft-defender-quarantine }

## 隔離の内容 { #what-the-quarantine-contains }

Microsoft Defender 隔離は、検出に関するメタデータと、隔離されたリソースの暗号化されたコンテンツを保存します。 Juntei は、エントリ、ResourceData、およびリソース レコードを関連付けて、脅威ラベル、隔離時間、元の論理パスおよび物理パス、ファイル メタデータ、所有者、ダウンロード元 `Zone.Identifier`、ハッシュ、および整合性の問題を報告します。

このアーティファクトは、最初に検出/修復を記録したイベント ログよりも存続する可能性があります。また、分離されたリバース エンジニアリングのために疑わしいバイトを正確に保存することもできます。

<span class="evidence-strength">この証拠から判断できること:</span> Defender が、記録された時刻に、指定の脅威またはリソースの隔離レコードを作成しました。製品の検出名だけでマルウェアと断定することはできず、隔離されたことだけでは、それ以前の実行も証明できません。

## 保存場所と収集 { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

`Quarantine` ディレクトリ全体を収集します。コンテンツ/レコード ファイルが一致しないエントリでもメタデータとしては役立ちますが、ファイル バイトや計算されたハッシュを生成することはできません。

## 安全性とアナリストへの注意事項 { #safety-and-analyst-cautions }

- 抽出されたコンテンツは悪意のあるものである可能性があります。分離された分析システムを使用し、決して実行しないでください。
- `ThreatName` は、当時の Defender の署名/分類です。誤検知や後で再分類される可能性があります。
- 隔離時間は修復/隔離時間であり、必ずしもダウンロード、作成、実行にかかる時間ではありません。
- 隔離されたファイルは、実行前または実行後に検出された可能性があります。
- 元のパスと Zone.Identifier は貴重ですが、存在しなかったり、攻撃者が制御したりする可能性があります。
- 孤立したレコードは、保持/クリーンアップ、不完全な収集、または破損によって発生する可能性があります。
- Juntei はサイズ、CRC、ID、ハッシュを検証します。回復されたバイトを信頼する前に、`Problems` を確認してください。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

たとえば、`tool.exe` は `tool.exe.quarantined` になります。補助ストリームの名前には、`tool.exe.Zone.Identifier.quarantined` などのタイプが含まれます。 Juntei は実行権限も削除します。拡張機能とアクセス許可により偶発的な実行は減少しますが、回復されたバイトは依然としてマルウェアとして扱う必要があります。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Record` | `resource`、リソースのないエントリ、`orphan-content`、または `orphan-record`。孤立者は不完全なリンク/コレクションを公開します。 |
| `QuarantineTime` | Defender 実行時間ではなく、UTC の隔離/修復時間。 |
| `ThreatName`, `ThreatID` | Defender 分類と数値 ID。ベンダーの判断/コンテキストとして扱います。 |
| `ResourceType` | ファイル、レジストリ キー、サービス、スタートアップ項目、スケジュールされたタスク、またはその他のリソース カテゴリ。 |
| `Path`, `PhysicalPath` | 元の論理的/物理的な場所。一次相関ピボット。正確なスペルを保持します。 |
| `FileSize` | リソースのメタデータによって記録されるサイズ。 `ContentSize` および MFT と比較してください。 |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | 隔離されたリソースに関して取得されたメタデータ。隔離時間から切り離されており、タイムストンピングに対して脆弱です。 |
| `Attr` | 記録されたファイルの属性。非表示/システム ステータスは便利なコンテキストです。 |
| `Owner` | 復元されたセキュリティ記述子からの SID。所有権は必ずしも実行ユーザーであるとは限りません。 |
| `ZoneID`, `ReferrerURL`, `HostURL` | `Zone.Identifier` からのダウンロード ゾーンと URL の値。強い起源が存在する場合はリードします。 ADS は存在しないか削除できます。 |
| `ContentSize` | 実際に復元されたプライマリ コンテンツのサイズ。記録された `FileSize` と比較してください。 |
| `MD5`, `SHA1`, `SHA256` | 回復されたバイトから計算されます。最新のコンテンツ識別には SHA-256 を使用します。相互運用性のためにすべてを保持してください。 |
| `Streams` | 回復されたストリームの種類/名前/サイズ。 ADS、セキュリティ データ、またはレジストリ ハイブを含めることができます。 |
| `ResourceID` | リソースのメタデータ/コンテンツ レコードをリンクする識別子。 |
| `EntryID` | 隔離エントリ GUID; Defender の歴史/イベント証拠へのリンクもあります。 |
| `ScanID` | リソース/アクションをグループ化するための Defender スキャン識別子。 |
| `QuarantineFolder` | ルートソースフォルダー/ホストの区別。 |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | 行の構築に使用される正確なソース ファイル。検証には必須です。 |
| `ExtractedFile` | `--extract` によって書き込まれたプライマリ隔離ファイル。一次抽出が行われていない場合は空です。補助ストリームはその横に書かれています。 `auto-parse -x` は、出力ディレクトリ (`./HOST1/SecurityProducts/DefenderQuarantined/…`) から取得します。 |
| `Problems` | 整合性の不一致、コンテンツの欠落、ID の不一致、損傷、または孤立したコンテキスト。抑圧しないでください。 |

JSON/JSONL は、CSV の結合テキストではなく、構造化リソース、ストリーム リスト、レコード エントリ、およびすべての関係の詳細を保持します。

## 抽出動作 { #extraction-behavior }

Juntei は、隔離エントリに拡張子が記録される場合、元のファイル名に最終拡張子 `.quarantined` を付けて使用します。孤立したリソースまたは非ファイル リソースのコンテンツ識別子を使用し、その識別子を追加してファイル名の衝突を解決します。 Windows で無効なファイル名の文字は置き換えられます。セキュリティおよび代替ストリームは、プライマリ ファイルの横に書き込まれます。各セキュリティ ストリームは、判読可能なテキスト レポートも取得します。 Juntei は実行権限を削除します。 `-C` がなければ、既存の異なるコンテンツは上書きされません。これらの制御は事故を減らしますが、バイトを安全にするものではありません。

### 復元されたセキュリティ記述子の読み取り { #reading-the-recovered-security-descriptor }

`.security.quarantined` ファイルにはバイナリ Windows セキュリティ メタデータが含まれているため、テキスト エディタでは読み取ることができません。これは、実行可能ファイルの別のコピーではなく、別個のバックアップ ストリームです。 Juntei は、抽出時に `parse-file defender-quarantine --extract` または `auto-parse -x` で自動的にデコードされます。

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

テキスト ファイル名は、衝突識別子を含む実際のバイナリ名の後に続きます。生成されるテキストは通常​​の UTF-8 で、他の抽出ファイルと同じ上書き保護が使用されます。これにはバイナリ ファイル名、バイト数、SHA-256 が含まれるため、レポートをソースと照合できます。

| レポートフィールド | それをどう解釈するか |
|---|---|
| 所有者とプライマリグループ | 記述子に記録される SID。既知の SID には、`BUILTIN\Administrators` や `NT AUTHORITY\SYSTEM` などの名前が付けられます。他の SID は未解決のままです。ローカル/ドメイン アカウントに名前を付けるには、使い慣れた RID だけでは不十分です。所有権は、誰がファイルを作成または実行したかを証明するものではありません。 |
| 制御フラグ | 元の数値フラグとその名前 (ACL の存在、継承、保護を含む)。 |
| DACL | 順序付けされたアクセス制御エントリ (ACE): 許可/拒否タイプ、アカウント SID、許可マスク、権限および継承フラグ。これらは記録されたエントリであり、ユーザーのトークンに対する有効なアクセスの計算ではありません。 |
| SACL | 記録された監査エントリまたは必須の整合性エントリ。監査の成功/失敗フラグは、アクセスの履歴ではなく、構成について説明します。 SACL が存在しない場合、すべての監査が無効になったことは証明されません。 |
| オブジェクトの GUID | オブジェクト固有の ACE のオプションの GUID。ファイル権限として誤ってラベル付けされるのではなく、オブジェクト固有の権限が数値的に保持されます。 |
| SDDL | 所有者、グループ、およびサポートされている ACL エントリのセキュリティ記述子定義言語表現。制御フラグを別途参照してください。コンポーネントが破損しているかサポートされていない場合、Juntei は部分的な SDDL 文字列を出力しません。 |
| デコードのステータスと問題 | `complete` または `incomplete` (理由あり)。サポートされていない ACE は、レポートに未加工の 16 進数が残ります。抽出が成功すると、元のバイナリが常に保持されます。デコードの問題は、隔離レコードの `Problems` フィールドにも発生します。 |

存在しない DACL、NULL DACL、および空の DACL は個別に報告されます。存在しない、または NULL DACL には、任意のアクセス制限はありません。空の DACL には、アクセスを許可する ACE がありません。他の Windows アクセス チェックと権限は依然として重要です。継承された ACE と継承専用 ACE には明示的にフラグが付けられたままになります。

たとえば、継承されたフル コントロールを管理者に付与するエントリは次のようになります。

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

デコーダは、Windows API、Python、またはアカウント ルックアップを行わずに、Windows、macOS、および Linux 上でオフラインで実行されます。基本的な許可/拒否/監査/アラーム ACE、オブジェクト ACE、および必須整合性ラベルをデコードします。条件付き、コールバック、リソース属性、およびその他のサポートされていない ACE は明示的に識別され、生の形式で保存されます。アクセス条件は評価されません。記述子が破損しても、他のストリームの抽出は停止されません。

## 調査戦略 { #investigation-strategy }

1. `ThreatName`、`Path`、`QuarantineTime`、`EntryId`、およびハッシュから開始します。
2. `Problems` を検証し、記録されたサイズと復元されたコンテンツを比較します。
3. SHA-256 を他のホスト、EDR、サンドボックス、および脅威インテリジェンス データにピボットします。
4. 元のパス/時刻を `$MFT`、USN ジャーナル、Prefetch、Amcache、およびイベント ログと関連付けます。
5. URL/zone フィールドを使用して初期アクセスを調査しますが、ブラウザ/プロキシ/DNS の証拠で検証します。

## さらに詳しい情報 { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [コートマークディフェンダー](https://github.com/Sootmark/defender)
- [dissect.target Defender プラグイン](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: Microsoft Defender ウイルス対策修復と隔離](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [ディフェンダーダンプ](https://github.com/knez/defender-dump)
- [Microsoft: セキュリティ記述子](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: セキュリティ記述子文字列形式](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: ACE 文字列](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
