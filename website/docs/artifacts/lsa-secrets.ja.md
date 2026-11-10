# LSA の秘密 { #lsa-secrets }

## 彼らは何ですか { #what-they-are }

ローカル セキュリティ機関 (LSA) は、`SECURITY` ハイブに秘密を保管します。これらには、自動ログオン パスワード、サービスを実行するアカウントのパスワード、コンピューター アカウントのドメイン パスワード、DPAPI システム キー、キャッシュされたドメイン ログオンの暗号化に使用されるキーが含まれます。また、キャッシュされたドメイン ログオンも保持されるため、ドメイン ユーザーはドメイン コントローラーなしでサインインできます。

`parse-registry lsa-secrets` には、システムが保持するシークレットとそのシークレットがいつ設定されたか、およびシステムが保持および使用するキャッシュされたログオンの数がリストされます。 **名前と時刻のみを読み取ります。何も復号化されず、シークレットのバイトの読み取りも書き込みも行われません。** Juntei は証拠を読み取ります。資格情報は抽出されません。

<span class="evidence-strength">この証拠から判断できること:</span> システムがこの名前のシークレットを保持し、この時刻に設定していました。サービスのシークレット（`_SC_<service>`）は、LSA がそのサービスの実行アカウントのパスワードを保存していることを意味します。

## 場所 { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| 名前 | 種類 | それは何ですか |
|---|---|---|
| `DefaultPassword` | `autologon` | Winlogon が自動的にサインインするパスワード。 |
| `_SC_<service>` | `service_account` | アカウントのパスワードは `<service>` として実行されます。 |
| `$MACHINE.ACC` | `machine_account` | ドメイン内のコンピュータ アカウントのパスワード。 |
| `DPAPI_SYSTEM` | `dpapi_system` | システムの DPAPI キー。 |
| `NL$KM` | `cached_logon_key` | キャッシュされたログオンが暗号化されるキー。 |
| `L$_SQSA_<SID>` | `security_questions` | ローカル アカウントのセキュリティの質問 (Windows 10 以降)。 |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | ダイヤルアップおよび VPN 資格情報。 |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | その他のプライベート データには、プレフィックス `LsaStorePrivateData` が付けられます。 |

`SECURITY` とそのトランザクション ログ、およびホストに名前を付けるシステムの `SYSTEM` ハイブを収集します。

## 分析時の注意点 { #analyst-cautions }

- **値ではなく名前と時間。** 値にデータが保持されるかどうかは、そのサイズによって決まります。その内容は読まれていません。
- **`DefaultPassword` は自動ログオンを意味しません。** シークレットは、自動的にサインインしないシステムを含む、ほとんどのシステムに存在します。 `SOFTWARE` の `Microsoft\Windows NT\CurrentVersion\Winlogon` (`AutoAdminLogon`、`DefaultUserName`) は、自動ログオンが構成されているかどうかを示します。最近の `CurrentSetUTC` は説明する価値のある変更です。
- **サービス シークレット:** `_SC_<service>` は、サービスがパスワード付きのアカウントとして実行されるように設定されている場合に表示されます。 `parse-registry services` (`ObjectName`) と比較し、最近インストールされたサービスや不明なサービスを探します。シークレットはサービスが終了した後も残る可能性があります。
- **Times:** `CurrentSetUTC` および `OldSetUTC` は、Windows が現在値と以前の値を設定したときです。 `$MACHINE.ACC` パスワードはデフォルトでは 30 日ごとに変更されるため、その時間は変化します。 `KeyLastWrittenUTC` および `CurrVal`/`OldVal` キーの時刻は、シークレットが最後に変更されたときの時刻です。
- **Cキャッシュされたログオン:** レコードにユーザーが指定されている場合、スロットは使用中です。レコードのヘッダーは暗号化されません。ユーザー名の長さのみが読み取られます。 `CachedLogonsKept` は `NL$Control` のカウントで、`CachedLogonsCount` (デフォルトでは 10) に続きます。スロットが使用中であるということは、ドメイン ユーザーが対話的にサインインしていることを意味し、その資格情報がハイブからオフラインで解読される可能性があります。ユーザーの名前は暗号化され、デコードされません。完全なヘッダーと一貫したユーザー名の長さを持つ検証済みのバイナリ レコードのみが使用中にカウントされます。 `Problems` が破損したスロットを報告した場合、使用中の数は検証されたスロットのみを対象とします。破損したスロットを空であると解釈しないでください。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

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
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

各ハイブの概要には、各シークレットがその種類と設定時期とともにリストされます。次に、アカウント パスワード LSA が保存されているサービスに名前を付け、使用中のキャッシュされたログオン スロットをカウントします。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath` | ホスト (システムの `SYSTEM` ハイブから)、および `SECURITY` ハイブ。 |
| `Secret`, `Kind` | シークレットの名前とその内容 (上の表を参照)。キャッシュされたログオンのレコードの `Cache` および `cached_logons`。 |
| `ServiceName`, `UserSID` | `_SC_` シークレットが対象となるサービス、およびユーザーの SID の名前は (`L$_SQSA_`、`RasDialParams`) です。 |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` および `OupdTime`: 現在の値と以前の値が設定されたとき。 |
| `HasCurrentValue`, `HasOldValue` | `CurrVal` および `OldVal` がデータを保持するかどうか。 |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | `CurrVal`、`OldVal`、および秘密鍵の最後の書き込み。 |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | キャッシュされたログオンのレコード: スロット、使用中のスロット、および `NL$Control` の数。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | 不正な形式または範囲外の時間 (時間メタデータのみが表示される)、破損したキャッシュ ヘッダー、間違ったキャッシュ値のタイプ、および読み取りできないキー。 |

## 調査戦略 { #investigation-strategy }

1. 不明なサービスまたは最近インストールされたサービスの `_SC_` シークレットを探し、それらを `parse-registry services` およびイベント 7045 (サービスがインストールされました) と照合します。
2. `DefaultPassword` の時間を `Winlogon` の自動ログオン設定と比較して確認します。
3. ドメイン メンバーでは、使用中のキャッシュされたログオン スロットと、`Cache` が最後に書き込まれた日時を記録します。
4. シークレットの時間をタイムラインと比較します。侵入ウィンドウ中に設定されたシークレットは、新しいサービス アカウント、自動ログオン、またはマシン アカウントの変更をマークすることができます。

## さらに詳しい情報 { #further-details }

- [Microsoft: プライベート データ オブジェクト](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: キャッシュされたドメイン ログオン情報](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: キャッシュされた資格情報](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
