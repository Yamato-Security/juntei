# セキュリティ設定 { #security-settings }

## 彼らは何ですか { #what-they-are }

攻撃者が変わる Windows の 非表示のままにして資格情報にアクセスするためのセキュリティ設定。彼らはオフになります Microsoft Defender またはツールをそこから除外するか、 Windows 平文のパスワードを保持し、リモート デスクトップを開き、ファイアウォールをオフにし、沈黙させます。 PowerShell ロギング。レジストリには次の設定が保存されます。

- **`SOFTWARE`:** Defender、その除外とポリシー。 UAC;リモート デスクトップとファイアウォールのポリシー。 PowerShell の実行ポリシー、ロギング ポリシー、および AMSI プロバイダー。システムの復元。イベントログチャネル。
- **`SYSTEM`** (コントロール セットごと): `WDigest`、LSA 保護、Credential Guard、制限付き管理者、NTLM。リモートデスクトップ。ファイアウォールプロファイル。 SMBv1;最終アクセス時間。各イベント ログのサイズ。
- **`NTUSER.DAT`:** ユーザーの PowerShell 実行ポリシーとログ ポリシー、および Windows スクリプト ホストの AMSI。

`parse-registry security-settings` は、検出した各設定をレコードとして報告します。各レコードには、設定の値、値の意味、および **がデフォルトの** を弱めるかどうか (保護または証拠ソースをオフにするか、値を指定しない場合の現在の Windows (10 および 11) よりも低くするかどうか) が含まれます。

<span class="evidence-strength">この証拠から判断できること:</span> ハイブの保存時点で、この設定はこの値であり、キーの最終書き込み時刻はこの時刻でした。保護を弱める設定は調査の手掛かりであり、侵害の証明ではありません。管理者、グループポリシー、他のセキュリティ製品、古い Windows バージョンでも設定される値があり、一般的な場合は説明に記載しています。

## 場所 { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

トランザクション ログを含む 3 つのハイブすべてを収集します。収集の直前に変更された設定は、ログにのみ存在する可能性があります。

## 分析時の注意点 { #analyst-cautions }

- **Stored 構成は有効な保護状態ではありません。** 収集されたレジストリ値と、破損した値リストの診断行が報告されます。欠落したデータは、デフォルト、未収集のソース、または別の管理メカニズムを反映している可能性があります。保護が有効か無効かを証明するものではありません。 `WeakensDefault` フラグ設定が適用されると、指定されたベースラインが削減される可能性があります。
- **レガシー Defender フラグは無視される場合があります。** `DisableAntiSpyware` および `DisableAntivirus` は、Defender が停止したことを証明しません。適用可能性は、プラットフォームのバージョン、オンボーディング、および改ざん保護によって異なります。 [Microsoft では、これらの従来の設定 ](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware) について説明します。
- **最新の管理対象除外はレジストリ キャプチャでは使用できない場合があります。** プラットフォーム 4.18.25110.6 以降、エンドポイント構成管理に Defender を使用するデバイスでは、除外構成を取得するためにサポートされている Defender PowerShell コマンドレットが必要です。したがって、オフラインのレジストリが存在しないことは、除外が存在しないことを証明するものではありません。 [Microsoft 構成ガイダンス](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings)。
- **Weakened は悪意のあるものではありません。** 例:
  - `DisableAntiSpyware` = 1 は、別のウイルス対策プログラムが登録されているときに Windows によっても設定されます。
  - リモート デスクトップはサーバー上で意図的に開かれます。
  - Windows 7 は、より小さなイベント ログを保持し、最終アクセス時刻を更新しませんでした。
- **デフォルトはバージョンによって異なります。** 一部の値は以前はデフォルトでした。
  - `NtfsDisableLastAccessUpdate` = 1 は、Windows 10 1803 より前のデフォルトでした。評決なしで報告されていますが、後のバージョンで手動で設定された `0x80000001` は弱体化されています。
  - `RunAsPPL`、`LsaCfgFlags`、`UseLogonCredential` を現在の Windows と比較します。
- **Defender 除外:** 各パス、拡張子、プロセス、またはアドレスはレコードです。除外するたびに Defender が弱まります。 Defender 自体を追加する `TemporaryPaths` には評決がありません。ユーザーが書き込み可能なフォルダー、`.ps1`/`.exe` 拡張機能、および LOLBins (`rundll32.exe`、`powershell.exe`) を探します。
- **ポリシー キー** (`Policy` = True) は、グループ ポリシー、ツール、およびポリシーがローカル設定に影響を与えることを知っている攻撃者によって書き込まれます。それらをドメインの GPO と関連付けます。
- **イベント ログ チャネル:** のみ 25 セキュリティ関連チャネルが報告され、Windows がデフォルトで各チャネルをオンにするかどうかが報告されます。たとえば、`TaskScheduler/Operational` と `DNS-Client/Operational` はデフォルトではオフになっています。その他には `parse-registry read` を使用してください。
- **Key 回:** `KeyLastWrittenUTC` はキーのものなので、この値がいつ変更されたかというと必ずしもそうではありません。 `Policies` キー全体が一度に書き込まれるのが一般的です。
- **コントロール セット:** デフォルトでは、最後に使用されたコントロール セット Windows が読み取られます。 `--all-control-sets` はすべてを読み取ります。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

概要では、エリアごとに設定がカウントされます。ここには、デフォルトとすべての Defender の除外を弱める設定がリストされています。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath`, `HiveType` | ホスト、ハイブ、およびそのタイプ (`SOFTWARE`、`SYSTEM`、`NTUSER.DAT`)。 |
| `ControlSet`, `IsCurrentControlSet` | `SYSTEM` の場合: コントロール セット、および Windows が最後に使用したものであるかどうか。 `Select\Current` が欠落しているか無効である場合は不明です。 |
| `User`, `UserSID` | `NTUSER.DAT` の場合: プロファイル フォルダーの名前、およびシステムの `ProfileList` からのユーザーの SID。 |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`. |
| `Setting` | それは何ですか: `defender_disable_realtime_monitoring`、`defender_exclusion_path`、`uac_local_account_token_filter_policy`、`wdigest_use_logon_credential`、`rdp_deny_connections`、`powershell_script_block_logging`、`event_channel_enabled`、… |
| `Value` | 値: 10 進数の数値、文字列、または除外そのもの。値が設定のタイプではない場合は空です。 |
| `Meaning` | 値の意味。 |
| `WeakensDefault` | `True` 値が弱まる場合は現在の Windows がそれなしで行うこと、`False` がそうでない場合、空である場合はそれが適用されない (サイズ、ポート、AMSI プロバイダー)、または伝えることができない。 |
| `Policy` | キーが `Policies` キーの下にあるかどうか。 |
| `KeyPath`, `ValueName` | キーと値。除外の値の名前は除外される項目です。 AMSI プロバイダーには何もありません。 |
| `KeyLastWrittenUTC` | キーの最後の書き込み。 |
| `DataRaw` | 値のバイト (16 進数) (設定のタイプではない場合)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | 間違ったタイプの値と読み取れないキー。 |

## 調査戦略 { #investigation-strategy }

1. すべてのホストで `--weakened-only` を実行し、ホスト間を比較すると、1 つのホストのみの設定が目立ちます。
2. ホスト上および Defender のログで見つかったツールに対して各 Defender 除外を確認します (イベント 5007 は構成変更を記録します)。
3. 重要な時間をログオン、サービスのインストール、プロセスの作成と関連付けます。 `reg.exe`、`Set-MpPreference`、`netsh advfirewall`、および `powershell` コマンド ラインを探します (イベント 4688、Sysmon 1 および 13、PowerShell) 4104）。
4. 資格情報設定 (`WDigest`、`RunAsPPL`、`LocalAccountTokenFilterPolicy`、`DisableRestrictedAdmin`) については、資格情報のダンプを探し、その後ハッシュを渡します。
5. デフォルトを復元するために削除された値には `parse-registry read --recover-deleted` を使用し、古いコントロール セットには `--all-control-sets` を使用します。

## さらに詳しい情報 { #further-details }

- [Microsoft: Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus) の除外を構成および検証する
- [Microsoft: ユーザー アカウント制御の設定と構成](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: 追加された LSA 保護を構成します](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: 防御力の低下: ツールの無効化または変更](https://attack.mitre.org/techniques/T1562/001/)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
