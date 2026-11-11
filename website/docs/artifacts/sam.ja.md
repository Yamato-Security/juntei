# SAM ローカル アカウント { #sam-local-accounts }

## SAM の内容 { #what-the-sam-contains }

セキュリティ アカウント マネージャー (`SAM`) ハイブには、ローカル アカウント レコードが保存されます。 Juntei は、アカウント名、RID、プロファイル/コメント フィールド、アカウント制御フラグ、最終ログオン/パスワードのタイムスタンプ、カウンター、パスワードのヒント、Microsoft アカウントのリンク、プロファイル パス、および Windows がサインイン UI からアカウントを非表示にするかどうかをデコードします。

これは、新しく作成されたローカル管理者、有効な休眠アカウント、パスワードの変更、非表示のアカウント、水平移動ステージング アカウントを見つけるのに役立ちます。

<span class="evidence-strength">この証拠から判断できること:</span> ローカル SAM に、その状態のアカウントが記録されていました。これは完全な認証履歴ではなく、ドメインアカウントを記録するものでもありません。

## 保存場所と収集 { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

十分な権限/raw アクセスを使用してハイブとログを収集します。ホスト ID は、コレクション内の対応する `SYSTEM` ハイブから強化できます。

## 分析時の注意点 { #analyst-cautions }

- このコマンドは、Active Directory ドメイン ユーザーではなく、**local** アカウントを解析します。
- Juntei は意図的にパスワード ハッシュを抽出しません。
- RID 500 は、名前が変更されても組み込みの管理者アカウントです。名前だけでは不十分です。
- `LastLogon` とカウンターはレジストリの状態であり、セッションごとの完全な監査証跡ではありません。
- `HiddenFromLogonUI` は、サインイン画面からアカウントを非表示にします。アカウントは無効になりません。
- 組み込みの `DefaultAccount` および `WDAGUtilityAccount` は合法的に非表示にすることができます。
- 破損または欠落している `F` 値からの Null アカウント フラグを有効として解釈してはなりません。
- `NamesKeyLastWrittenUTC` はアカウント名の登録コンテキストに近似しますが、保証された作成時間ではありません。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath` | ホスト コンテキストと正確な SAM ハイブ。 |
| `RID` | 相対識別子。強力なアカウント ID ピボット。 500 はビルトイン管理者です。 |
| `User` | 現在のローカルアカウント名。名前は変更できます。 RID で関連付けます。 |
| `FullName`, `Comment`, `UserComment` | 説明文。攻撃者によって制御されており、レビューを偽装するのに役立ちます。 |
| `AccountFlags` | 無効、ロックアウト、パスワード不要、通常アカウント、パスワード無期限などのデコードされた状態。 |
| `AccountFlagsValue` | 検証用の生の数値フラグとサポートされていないビット。 |
| `CountryCode`, `CodePage` | アカウントのロケール/コードページのメタデータ。通常、属性ではなく文脈が重要です。 |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | 完全な精度と整数秒間の互換性で、前回のログオン状態と同じ状態になります。完全な歴史ではありません。 |
| `PasswordLastSetUTC` | パスワード変更時間を記録。予期しない最近/古い値を調査します。 |
| `AccountExpirationUTC` | 有効期限が有限の場合に設定されます。 Null は、「設定しない」または「設定しない」ことを意味します。 |
| `LastFailedLogonUTC` | 最後に記録された失敗したローカル ログオンは SAM 状態です。セキュリティ イベントを裏付けます。 |
| `FailedLogonCount`, `LogonCount` | Windows 固有の更新動作を伴う累積/状態カウンター。正確なイベント数ではなく、コンテキストとして使用します。 |
| `PasswordHint` | ユーザーが提供する機密性の高いヒント。パスワードのパターンが公開される可能性があります。適切に扱ってください。 |
| `InternetUserName` | リンクされた Microsoft アカウント ID (存在する場合)。 |
| `HiddenFromLogonUI` | UI 非表示状態。高い RID/予期しないカスタム アカウントを確認します。障害者という意味ではありません。 |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | ローカルアカウント環境/ログオン構成。異常なスクリプト/UNC パスは永続性を示している可能性があります。 |
| `KeyLastWrittenUTC` | RID アカウント キーの変更時間。フィールド固有ではなくキーレベル。 |
| `NamesKeyLastWrittenUTC` | ユーザー名マッピングキーの変更時間。便利なアカウントのライフサイクルの概算。 |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | 生の 64 ビット FILETIME 値は、検証および特別なセンチネル処理のために保持されます。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリの再生コンテキスト。 |
| `Problems` | F/V/名前の不一致、エントリの欠落、オフセットの不正、またはデコードの損傷。 |

## 調査戦略 { #investigation-strategy }

1. RID、名前、フラグ、プロファイル パス、および `NamesKeyLastWrittenUTC` をホスト ベースラインと比較します。
2. 有効な RID 1000+ アカウント、パスワード不要、無期限、非表示、または最近変更されたアカウントを確認します。
3. セキュリティ 4720/4722/4724/4732 イベント、ログオン イベント、プロファイル作成、およびリモート アクティビティと関連付けます。
4. 収集前にアカウントが削除された場合は、レジストリの回復履歴を検索します。
5. 正規のプロビジョニング/デフォルト アカウントと、攻撃者が作成した対話型アカウントを区別します。

## さらに詳しい情報 { #further-details }

- [Microsoft オープン仕様: SAM リモート プロトコル](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
- [dissect.target SAM プラグイン](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
