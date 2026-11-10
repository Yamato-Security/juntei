# 監査ポリシー { #audit-policy }

## それは何ですか { #what-it-is }

監査ポリシーには、どのセキュリティ イベント Windows がセキュリティ イベント ログに記録されるかが記載されています。 Windows Vista 以降ではサブカテゴリごと (Windows 10 および 11 では 59)、Windows XP およびサーバー 2003 ではカテゴリごとに設定されます。それぞれが成功、失敗、またはその両方を記録するか、何も記録しません。

LSA は、ポリシーが `auditpol`、ローカル セキュリティ ポリシー、グループ ポリシーのいずれからのものであっても、`SECURITY` ハイブ内で有効なポリシーを保持します。

<span class="evidence-strength">この証拠から判断できること:</span> ハイブの最終書き込み時点で、これらのイベントを記録する、または記録しない設定になっていました。あるサブカテゴリの監査が無効なら、そのイベントがないことから活動の有無は判断できません。

## 場所 { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista 以降では、12 バイトのヘッダー、次にサブカテゴリごとに 1 つの設定、カテゴリごとに、各カテゴリにサブカテゴリの数が保持されます。バージョンの違いはその数のみです。各バージョンはカテゴリの最後にサブカテゴリを追加します (Windows 8.x およびサーバー 2012 R2 ハイブ読み取りでは 56、Windows では 59) 10、11、およびサーバー 2022)。 Windows XP およびサーバー 2003 は、監査がオンかどうか (1 バイト、その後パディング Windows はクリアされません) と 9 つのカテゴリ設定を保持します。このレイアウトは、NIST の CFReDS ハッキング事件の公開 Windows XP イメージでチェックされましたが、何も監査されていません。そのカテゴリ設定の値は、Microsoft のドキュメントに従います。

`SECURITY` とそのトランザクション ログ、およびホストに名前を付けるシステムの `SYSTEM` ハイブを収集します。

## 分析時の注意点 { #analyst-cautions }

- **不明なデータは不明のままです。** サポートされていないリビジョンは診断とともに保存されます。不明な設定フラグは `Success` および `Failure` を空のままにし、監査設定カウントから除外します。カウント テーブルが破損していると、予約語を別の監査設定に昇格させることができません。キャプチャには、name/GUID カタログを超える追加のスロットを含めることができます。それらの設定と数値 `SubcategoryId` は空の名前/GUID で保持されます。マップされていないスロット自体は破損の証拠ではありません。


- **順序は Windows 独自のものです。** カテゴリ内では、設定は GUID ではなく、サブカテゴリのメッセージ ID の順序になります。特殊ログオン (`%%12548`、ログオン/ログオフのインデックス 4) は、IPsec クイック モードの前に来ます。 GUID の順序でこれらを読み取るツールは、Windows 10 のデフォルトを特別ログオン「監査なし」および IPsec クイック モード「成功」として報告します。 `SubcategoryId` は、ID イベント 4719 (システム監査ポリシーが変更されました) が与えるものと同じであるため、この 2 つは一致する可能性があります。
- **ソースではなく、有効なポリシー。** グループ ポリシーとローカル ポリシーは同じ値を書き込みます。グループ ポリシーの `audit.csv` (SYSVOL および `C:\Windows\System32\GroupPolicy` 内) は、適用される内容 (ポリシーの `Machine\Microsoft\Windows NT\Audit\audit.csv`) を示します。
- **Changes over time:** `KeyLastWrittenUTC` は、ポリシーが最後に変更されたときの情報です。イベント 4719 は、各変更をログに記録します (セキュリティ ログに記録されている限り)。攻撃者は監査をオフにしてから再びオンにする可能性があります。
- **サブカテゴリとカテゴリ。** Windows Vista 以降では、サブカテゴリは Windows が使用するものです (`SCENoApplyLegacyAuditPolicy` セキュリティ オプションがオフで、従来のカテゴリ ポリシーがサブカテゴリをオーバーライドしない限り)。
- **Windows XP および 2003:** 監査オフ (`AuditingEnabled` False) の場合、設定に関係なくカテゴリは監査されません。
- **Events:** `Events` は、Microsoft の高度な監査ポリシー設定のドキュメントからの、各サブカテゴリの主要なイベントをリストします。これらは通常、調査で検索されるものであり、サブカテゴリがログに記録するすべてのイベントではありません。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

各ハイブの概要には、そのレイアウト、監査される設定の数、およびポリシーが最後に書き込まれた日時が示されます。ここには、成功と失敗、成功のみと失敗のみについて監査される内容がリストされ、次に監査されないサブカテゴリとその主なイベントがリストされます。これらのイベントはキャプチャされたポリシーで無効になっています。これは、以前のアクティビティ中にポリシーを確立するものではありません。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath` | ホスト (システムの `SYSTEM` ハイブから)、および `SECURITY` ハイブ。 |
| `Layout` | `subcategories` (Windows Vista 以降) または `categories` (XP および 2003)。値がデコードされない場合は空です。 |
| `Category`, `Subcategory` | `auditpol` と名付けられています。 |
| `SubcategoryGUID`, `SubcategoryID` | サブカテゴリの GUID、およびイベント 4719 としてのそのメッセージ ID (`%%12548`)。 |
| `Setting` | `none`、`success`、`failure`、または `success_and_failure`。 |
| `Success`, `Failure` | 成功と失敗をログに記録するかどうか。 |
| `AuditingEnabled` | Windows XP および 2003: 監査がオンになっているかどうか。 |
| `Events` | サブカテゴリのメインイベント ID。 |
| `KeyLastWrittenUTC` | `PolAdtEv` の最後の書き込み: ポリシーが最後に変更されたときについて。 |
| `DataRaw` | 値全体がデコードされない場合の値のバイト数 (16 進数)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | 不明なレイアウトまたはサイズ、合計しないカウント、および 3 を超える値。 |

## 調査戦略 { #investigation-strategy }

1. 何かが起こっていないと結論付ける前に、そのサブカテゴリが監査されているかどうかを確認してください: プロセス作成 (4688)、ログオン (4624、4625)、特殊ログオン (4672)、その他のオブジェクト アクセス イベント (スケジュールされたタスク、4698)、セキュリティ システム拡張 (サービス)インストール、4697)、ユーザー アカウントとセキュリティ グループの管理 (4720、4732)、ファイル共有 (5140)、Kerberos (4768、4769)。
2. ポリシーをセキュリティ ログと比較します。現在監査されていないサブカテゴリのイベントは、ポリシーが変更されたことを意味します。イベント 4719 と最近の `KeyLastWrittenUTC` を探します。
3. グループ ポリシーの `audit.csv` と比較して、ドメイン全体のポリシーとローカルの変更を区別します。
4. `parse-registry read --recover-deleted` と、`PolAdtEv` の以前のバージョンのトランザクション ログを使用します。

## さらに詳しい情報 { #further-details }

- [Microsoft: 高度なセキュリティ監査ポリシー設定](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: 監査定数 (サブカテゴリ GUID)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) システム監査ポリシーが変更されました](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
