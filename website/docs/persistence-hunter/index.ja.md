# Persistence Hunter { #persistence-hunter }

永続化は、コードを再度実行する方法です。Windows の開始時、ユーザーのサインイン時、アプリケーションの起動時、スケジュールされた時刻の到来時、または別のイベントの発生時です。攻撃者は、正規のソフトウェアが更新、デバイスのサポート、セキュリティ、管理に使用するのと同じ設定を悪用する可能性があります。

`juntei persistence-hunter` は、これらのメカニズムおよび関連するアカウント/保護の異常をカバーする **131 チェック** について、取得された Windows の証拠をオフラインで検査します。レジストリ ハイブ、タスク ファイル、スクリプト、アプリケーション設定、および選択されたデータベース レコードを読み取ります。

このページでは、実装されたすべてのチェックについて説明します。完全なオプション、出力フィールド、プロファイル、およびタイムスタンプの処理については、[ コマンド ガイド](../commands/persistence-hunter.md) を参照してください。 [acknowledgements](acknowledgements.md) は、Python PyrsistenceSniper プロジェクトと元の PowerShell PersistenceSniper ツールをリンクします。

## 調査を開始する { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

完全な候補インベントリには `--min-severity info` を使用します。手動コマンドのデフォルトは `medium` です。そのプロファイルは、ペイロードの欠落や署名者の証拠が完全な比較を妨げる場合でも、部分的な許可一致を `LOW` として分類できます。許可一致や見慣れないエントリのいずれも、何かが悪意のあるものであるかどうかを決定するものではありません。 `auto-parse` はすでにすべての重大度を保持しており、各コンピューターの `persistence/` カテゴリに結果を書き込みます。

メカニズムに注目するには、以下からその **Check ID** をコピーします。

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

このコマンドは、デフォルトで共有ワーカー プールを使用します。 4 つのワーカーを選択するには `-T 4` を追加し、単一スレッド スキャンの場合は `-T 1` を追加します。入力には複数のコンピュータが含まれる場合があります。彼らの証拠は別々のままです。

## 調査結果を証拠として読む { #read-findings-as-evidence }

| 質問 | 何を検査するか |
|---|---|
| 何がどこで設定されましたか? | `CheckId`、`Path`、および引数を含む完全な `Value`。 `SourceFile` に従って、キャプチャされたハイブ、ファイル、またはデータベースに移動します。 |
| ターゲットはコレクションに存在しますか? | `ResolvedPath`、`ResolutionStatus`、および `Exists`。収集されていない、あいまいな、またはサポートされていない参照は不明です。 Juntei は、アナリストのマシンのファイルを置き換えません。 |
| 設定は有効ですか? | 実際のトリガー、有効フラグ、選択したコントロール セット、Windows/アプリケーションのバージョン、ユーザー、およびアーキテクチャ。一部のチェックでは、非アクティブな構成を意図的に保持します。 |
| これは攻撃ですか? | `Severity` と `PolicyReason` は、判定ではなく、プロファイルの一致について説明します。予想されるソフトウェア、完全なコマンド、ファイル ハッシュ、独立した実行の証拠、および周囲のタイムラインと比較します。 |
| 誰がそれを実行できるでしょうか？ | `AccessGained` はスコープ/コンテキストのヒントです。マシン全体の構成には `SYSTEM` と表示されることがよくありますが、これは起動されたプロセスが SYSTEM として実行されたことを証明するものではありません。タスク プリンシパルと BITS 所有者 SID は、利用可能な場合にはより具体的なコンテキストを提供します。 |
| いつから変わりましたか？ | `KeyLastWrite` はレジストリ キー全体の日付を示します。 `LastChange`、`ChangeSource`、および `ChangeEvidence` は、使用可能な場合、個別の MFT/EVTX 相関を記述します。どちらのフィールドにも、ペイロードの実行日付が自動的に記録されることはありません。 |
| 回復された状態はどの程度信頼できますか? | `RecordStatus`、`Coverage.csv`、および再生された証拠、刻まれた証拠、破損した証拠、欠落した証拠、または限定的な証拠に関する警告をお読みください。回復された履歴レコードはアクティブではなくなる可能性があります。 |
| 指名された発行元は信頼できますか? | `Signer` および `SignerStatus` は、説明的な PE/証明書/カタログ メタデータです。 Juntei は、Authenticode 署名または証明書の信頼性を検証しません。 |

所見が見つからないからといって、持続性がないことは証明されません。 `complete` カバレッジ行は、該当する収集ソースが読み取られたことを意味し、取得にすべての関連ファイル、すべてのユーザー、またはすべての可能な Windows 永続化メカニズムが含まれていることを意味しません。他のカバレッジ状態は、チェックの欠落、部分的、無効、または失敗を説明します。ハンターは、ライブ メモリ、ネットワーク、またはクラウド ID スキャンではありません。

## カタログと所在地の表記 { #catalogue-and-location-notation }

カタログは PyrsistenceSniper に加えて、ネイティブ `start_menu_com` チェックに基づいています。以下の説明は、Juntei のネイティブの動作を証拠の制限も含めて文書化しています。 **116 レジストリベースのチェックと 15 ファイル/リポジトリ チェック** があります。一部のファイル チェックではレジストリ データも結合されるため、これらは相互に排他的な取得要件ではなく実装数です。メカニズムは以下の調査目的ごとにグループ化されており、各グループ内では Check ID の順にアルファベット順に並べられています。

- `HKLM\SYSTEM`、`HKLM\SOFTWARE`、および `HKLM\SAM` は、キャプチャされたマシン ハイブを指します。通常は `Windows\System32\config` から収集され、利用可能な場合はトランザクション ログも含まれます。
- `HKU\<user>` は、関連するキャプチャされたユーザー ハイブを意味します。ほとんどのユーザー設定は NTUSER.DAT から取得されます。クラス登録は、前述のように `AppData\Local\Microsoft\Windows\UsrClass.dat` から取得される場合があります。
- `<ControlSet>` は、SYSTEM で選択されたアクティブ コントロール セットを意味します。 DSRM チェックの追加の履歴の場所は、以下に明示的に記載されています。レビューするときは `SYSTEM\Select` を確認してください。
- `<name>` は可変パス コンポーネントです。レジストリ ターゲット テーブルでは、`<child>` は直接の子の登録を意味します。それは無制限の再帰的ウォークを意味するものではありません。明示的な再帰チェックでは、それらの走査を個別に記述します。 Office パス内の括弧は、オプションのバージョンを示します。
- 表には、チェックによって選択された実際のマシンとユーザーのビューがリストされています。 32 ビット ビューが実装されている場所にリストされます。すべてのチェックで考えられるすべてのアーキテクチャ/ビューが検査されると想定しないでください。
- 汎用値チェックでは、複数の文字列要素や数値のゼロ (該当する場合) など、空ではない構成済みのエントリが出力されます。これら自体は、悪意のあるペイロード、有効なトリガー、または実行の成功を必要としません。より選択的な述語がチェックごとに記述されます。

ATT&CK ID は、さらなる調査のために継承されたカタログ マッピングとリンクです。これらは、攻撃者がそのテクニックを実行したという発見ではなく、潜在的な悪用について説明しています。保護設定および診断チェックでは、スタンドアロンの永続化メカニズムではなく、有効化条件が記述される場合があります。

| 機構グループ | 小切手 |
|---|---:|
| [ブート、サインイン、およびスタートアップ](#boot-sign-in-and-startup) | 32 |
| [サービスとシステム拡張機能](#services-and-system-extensions) | 13 |
| [スケジュールされたタスク](#scheduled-tasks) | 3 |
| [WMI、BITS、およびその他のイベント トリガー](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorer、COM、およびファイルまたは URL ハンドラー](#explorer-com-and-file-or-url-handlers) | 15 |
| [Office アプリケーションとアドイン](#office-applications-and-add-ins) | 7 |
| [デバッガー、クラッシュ ハンドラー、および互換性シム](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [.NET ランタイムとプロファイラー設定](#net-runtime-and-profiler-settings) | 7 |
| [DLL および実行可能ルックアップのオーバーライド ](#dll-and-executable-lookup-overrides) | 18 |
| [アカウントとサインインの可視性](#accounts-and-sign-in-visibility) | 4 |
| [認証コンポーネントと資格情報の保護](#authentication-components-and-credential-protection) | 8 |
| **合計** | **131** |

## ブート、サインイン、およびスタートアップ { #boot-sign-in-and-startup }

これらの設定は、ブート、サインイン、またはデスクトップの初期化中に実行されるコマンドまたはライブラリを調整します。 Windows バージョンと該当するユーザーを確認します。設定が廃止されているか、無効になっているか、攻撃者ではなくキオスクを対象としている可能性があります。

### アクティブセットアップ {#active_setup}

**ID:** `active_setup` · **カタログATT&CK:**をチェック[T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**Juntei がチェックする内容:** ネイティブ ビューおよび 32 ビット ビューのマシン全体のアクティブ セットアップ コンポーネント登録から `StubPath` を読み取ります。 Windows は、ユーザーのプロファイルを準備するときにこれらのコマンドを使用できます。このチェックでは、リテラルのプレースホルダー値 `U` および `/UserInstall` はスキップされます。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` および対応する `Wow6432Node` ツリー。

**調査方法:** 各コンポーネントとインストールされている製品を比較し、コマンド引数を検査します。マシン全体の登録は、サインインしているユーザーのセッションで実行される可能性があります。出力の `SYSTEM` ラベルは、結果として得られるプロセス トークンの証拠ではなく、構成スコープを説明します。

### ブート実行 {#boot_execute}

**ID:** `boot_execute` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** セッション マネージャー `BootExecute` 複数文字列を読み取り、空ではない個々のエントリを保持します。初期ブートフェーズ用に設定されたコマンドを記録します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**調査方法:** `autocheck autochk *` はよく知られたデフォルトであり、ディスク ツールは正当なエントリを追加できます。デフォルト以外のエントリがすべて実行された、または悪意があると想定せずに、追加されたコマンドとそのファイルを調査します。

### ブート検証プログラム {#boot_verification_program}

**ID:** `boot_verification_program` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** `BootVerificationProgram` の `ImagePath` (ブート検証に関連付けられた実行可能場所) を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**調査方法:** この古いメカニズムが、キャプチャされた Windows バージョンとブート構成に適用できるかどうかを確認します。ターゲット ファイルを解決し、その目的を予想されるブート管理ソフトウェアと比較します。

### エクスプローラーの負荷値 {#explorer_load}

**ID:** `explorer_load` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** Windows サインイン初期化構成に関連付けられた `Load` 値をマシンおよびユーザー ハイブから読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**調査方法:** コンテンツをフェッチまたはデコードする未知のコマンド、ユーザーが書き込み可能な宛先、および引数を調査します。適用性と実行を個別に確認します。空の結果は、他のサインイン メカニズムをカバーしません。

### フォントドライバー {#font_drivers}

**ID:** `font_drivers` · **カタログATT&CK:**をチェック[T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei がチェックする内容:** 従来の `Font Drivers` レジストリ キー内の空でない値を潜在的なライブラリ登録として列挙します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | 空ではないすべての値 |

**調査方法:** これを異常な従来の構成サーフェスとして扱います。ロードできると結論付ける前に、Windows バージョンのサポートと参照ライブラリをチェックし、プロセスまたはイメージ ロードの証拠で実際の使用を裏付けてください。

### グループ ポリシー拡張 DLL {#gp_extension_dlls}

**ID:** `gp_extension_dlls` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** Winlogon での各即時のグループ ポリシー拡張登録から `DllName` を読み取ります。これらの DLL はポリシー処理を拡張します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**調査方法:** 管理製品とセキュリティ製品は拡張機能をインストールできます。拡張子 GUID および DLL を予期される製品と照合し、実行をアサートする前にポリシー処理とイメージロードの証拠を調べます。

### グループポリシースクリプト {#gp_scripts}

**ID:** `gp_scripts` · **カタログATT&CK:**をチェック[T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei がチェックする内容:** キャプチャされたローカル グループ ポリシー `scripts.ini` および `psscripts.ini` 登録ファイル (ユーザーごとのローカル ポリシー ストアを含む) からコマンドとパラメーターのペアを解析します。スタートアップ/シャットダウン セクションとログオン/ログオフ セクションは、設定されたコマンドと引数を保持し、マシン セクションはユーザー セクションと区別されます。

**調査された証拠:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` および同等の `GroupPolicy\User\Scripts` ファイル。
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`.

**調査方法:** 実装されたローカル ポリシーの登録場所のみが調査されます。これは完全な SYSVOL/GPO 監査ではありません。ポリシー ディレクトリ内の別の場所にあるスクリプト ファイルだけでは十分ではありません。その登録をポリシーの処理および実行の証拠と関連付けます。

### ログオン スクリプト (UserInitMprLogonScript) {#logon_scripts}

**ID:** `logon_scripts` · **カタログATT&CK:**をチェック[T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei がチェックする内容:** ユーザーの `UserInitMprLogonScript` 環境値を読み取ります。これにより、そのユーザーのサインイン用のスクリプトを選択できます。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**調査方法:** 承認されたログオン スクリプトと比較し、実際のスクリプトを検査します。この値はその後のサインインまたは実行を証明するものではなく、ドメイン提供のスクリプトには追加のポリシー証拠が必要です。

### プラットフォームの実行 {#platform_execute}

**ID:** `platform_execute` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** セッション マネージャーの空ではない各 `PlatformExecute` エントリを読み取ります。プラットフォーム関連のブート初期化のために選択されたプログラムを識別します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**調査方法:** ハードウェアまたはプロビジョニングのワークフローでエントリが説明される場合があります。この Windows ビルドのブート パスがそれを使用しているかどうかを確認し、値の名前から攻撃を推測するのではなく、参照されたイメージを調査します。

### RDP クライアント拡張 DLL {#rdp_clx_dll}

**ID:** `rdp_clx_dll` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** ターミナル サーバーの既定のユーザー構成内の `ClxDllPath` を、RDP 関連の拡張子の場所として読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**調査方法:** Windows/RDPのバージョンとインストールされている拡張機能を確認します。構成された DLL は、イメージ読み込み分析のリードとなります。レジストリ値だけでは、RDP 接続または DLL ロードが発生したことはわかりません。

### RDP 仮想チャネル DLL {#rdp_virtual_channel}

**ID:** `rdp_virtual_channel` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** マシン全体のターミナル サーバー クライアント `Default\AddIns` 登録の下にある値を列挙します。可能な場合、バイナリ値は UTF-16LE としてデコードされ、使用可能な拡張パスが維持されます。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: すべての値。

**調査方法:** RDP クライアントとデバイス リダイレクト製品はアドインを正当に登録します。実際の DLL またはコマンド、および関連するクライアント セッションを検査します。アドイン内のすべての値が実行可能パスであるわけではありません。

### RDP WDS スタートアップ プログラム {#rdp_wds_startup}

**ID:** `rdp_wds_startup` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** RDP Windows ステーション ドライバー構成から `StartupPrograms` を読み取ります。これは、リモート セッションの起動用に構成されたプログラムを識別します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**調査方法:** `rdpclip` はおなじみのエントリです。追加内容とホストの RDP 構成を調査し、セッション/プロセス レコードを使用して、構成されたプログラムが実際に開始したかどうかを判断します。

### レジストリ実行キー {#run_keys}

**ID:** `run_keys` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** は、Run、RunOnce、RunEx、RunOnceEx、およびポリシー エクスプローラーのマシン/ユーザー ハイブからの Run 値と、リストされたマシンの 32 ビット キーを列挙します。 RunEx/RunOnceEx セクションは、境界とサイクル チェックを使用して走査されます。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | 空ではないすべての値 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | 空ではないすべての値 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | 空ではないすべての値 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | 空ではないすべての値 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | 空ではないすべての値 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | 空ではないすべての値 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | 空ではないすべての値 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | 空ではないすべての値 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | 空ではないすべての値 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | 空ではないすべての値 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | 空ではないすべての値 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | 空ではないすべての値 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | 空ではないすべての値 |

**調査方法:** ソフトウェア アップデーターとトレイ アプリケーションは通常、ここに表示されます。実行可能ファイル、コマンド引数、およびユーザー コンテキストを比較します。 RunOnce エントリは使用後に消えることがあります。キャプチャされた状態は、以前のサインインの完全な履歴ではありません。

### RunServices (レガシー) {#run_services}

**ID:** `run_services` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** 従来のマシン全体の `RunServices` キーの値を列挙します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | 空ではないすべての値 |

**調査方法:** これは過去の構成サーフェイスであり、最新の Windows ビルドで実行されることを保証するものではありません。ターゲットとオペレーティング システムの適用性を調査し、独立した実行証拠を探します。

### RunServicesOnce (レガシー) {#run_services_once}

**ID:** `run_services_once` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** 従来のマシン全体の `RunServicesOnce` キーの値を列挙します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | 空ではないすべての値 |

**調査方法:** 空でない値を構成リードとして扱います。ソース Windows バージョンがメカニズムをサポートしているかどうか、また展開または互換性ソフトウェアがエントリを説明しているかどうかを確認してください。

### S0 初期コマンド {#s0_initial_command}

**ID:** `s0_initial_command` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** 潜在的なセッション 0 初期化コマンドとしてセッション マネージャーから `S0InitialCommand` を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**調査方法:** キャプチャされた Windows のビルドおよびブート状態への適用性を確立します。設定されたコマンドは、ブート実行のタイムスタンプ付き記録ではありません。そのファイルを検査し、ブート/プロセス テレメトリと関連付けます。

### セッションマネージャーの実行 {#session_manager_execute}

**ID:** `session_manager_execute` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** セッション マネージャーから空ではない `Execute` エントリを読み取り、セッションの初期化用に選択されたプログラムを公開します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**調査方法:** 予想されるイメージ ベースラインと比較し、異常なネイティブ実行可能ファイルまたはパスを検査します。実行を要求する前に、変更後に関連するブート/セッションが発生したかどうかを確認します。

### セッションマネージャー SubSystems {#session_manager_subsystems}

**ID:** `session_manager_subsystems` · **カタログATT&CK:**をチェック[T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei がチェックする内容:** セッション マネージャーの `SubSystems` キーの `Windows` 値を読み取り、Windows サブシステム コマンドを構成します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**調査方法:** 通常のコマンドは長く、バージョンに依存します。完全な値と引数を同じビルドの信頼できるベースラインと比較します。重要なサブシステム設定を変更すると、起動が中断される可能性があります。

### セットアップ実行 {#setup_execute}

**ID:** `setup_execute` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** セッション マネージャーの `SetupExecute` エントリを読み取り、起動用に構成されたセットアップ関連のコマンドをカバーします。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**調査方法:** インストール、アップグレード、および回復アクティビティにより、正当なエントリが作成される可能性があります。構成が永続化によるものであると考える前に、セットアップ ログと参照イメージを比較してください。

### シェルフォルダーの起動リダイレクト {#shell_folders_startup}

**ID:** `shell_folders_startup` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** シェル フォルダーとユーザー シェル フォルダーの両方にある `Startup` と `Common Startup` をデフォルトの起動場所と比較します。デフォルト以外のリダイレクトが報告され、収集されたフォルダーの内容が検査されます。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` および `User Shell Folders`: `Common Startup`。
- NTUSER.DAT の同等の `Software\Microsoft\Windows\CurrentVersion\Explorer` 設定: `Startup`、およびリダイレクトされた場所にキャプチャされたファイル。

**調査方法:** リダイレクトされたスタートアップ フォルダーは自動起動を隠す可能性がありますが、リダイレクトは正当である可能性があります。パスはキャプチャされた証拠内で解決される必要があります。未収集の宛先は不明ですが、フォルダーが空だったという証拠にはなりません。

### シェルランチャーオーバーライド {#shell_launcher}

**ID:** `shell_launcher` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** ポリシー `Shell` 値と `system.ini` ブート シェル マッピングを SOFTWARE から読み取ります。これらは代替シェル選択サーフェスです。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**調査方法:** キオスクとアプリケーション アプライアンスは、エクスプローラーを意図的に置き換えることができます。選択したシェルとマシンの役割を比較し、設定が関連するセッションに対して有効であることを確認します。

### スタートアップフォルダー {#startup_folder}

**ID:** `startup_folder` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** 最初に使用可能なスタートアップ フォルダーのリダイレクトに続いて、既定のユーザーごとのおよび共通のスタートアップ フォルダー内の直接ファイルを検査します。収集された `.lnk` が解析できれば、ショートカット ターゲットが回復されます。

**調査された証拠:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` と `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`。
- マシン/ユーザー シェル フォルダーおよびユーザー シェル フォルダー設定で適用可能な `Common Startup` / `Startup` リダイレクト。

**調査方法:** スタートアップ リンクとファイルにより、サインイン時に正規のアプリケーションが起動される可能性があります。ファイル名だけでなくターゲットの引数も検査します。共通フォルダーは共有構成の場所です。起動されたプログラムが SYSTEM トークンを使用したことを証明するものではありません。

### ターミナル サービスの初期プログラム {#ts_initial_program}

**ID:** `ts_initial_program` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** ターミナル サービス ポリシー `InitialProgram` およびマシンのターミナル サーバー インストールの Run/RunOnce/RunOnceEx 構成 (境界付きネストされた RunOnceEx セクションを含む) を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | 空ではないすべての値 |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | 空ではないすべての値 |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | 空ではないすべての値 |

**調査方法:** リモート アプリケーションの展開では、これらの設定が意図的に使用される場合があります。影響を受けるセッションを確立し、選択したコマンドに従って証拠を処理します。登録だけではリモート ログオンは表示されません。

### Winlogon AppSetup {#winlogon_appsetup}

**ID:** `winlogon_appsetup` · **カタログATT&CK:**をチェック[T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei がチェックする内容:** Winlogon から `AppSetup` を読み取り、カンマ区切りのエントリを個別の候補に分割します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**調査方法:** 各コンポーネントを予想される Windows セットアップ/サインイン ソフトウェアと比較します。リスト エントリは個別に観察される実行ではなく、その関連性は Windows のバージョンと展開状態に依存する可能性があります。

### Winlogon GinaDLL {#winlogon_gina_dll}

**ID:** `winlogon_gina_dll` · **カタログATT&CK:**をチェック[T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei がチェックする内容:** 従来の Winlogon `GinaDLL` 置換設定を読み取ります。 GINA は、古い Windows サインイン拡張モデルでした。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**調査方法:** これは主に Vista 以前のシステムに関係します。新しいシステムに残っているキーは、有効なログオン拡張機能の証拠にはなりません。ソース バージョンと参照されている DLL を検査します。

**さらに詳しい情報:** [Microsoft ドキュメント](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina)。

### Winlogon MPNotify {#winlogon_mpnotify}

**ID:** `winlogon_mpnotify` · **カタログATT&CK:**をチェック[T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei がチェックする内容:** Winlogon の `mpnotify` コマンド/ライブラリ設定、サインイン関連の拡張サーフェスを読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**調査方法:** ターゲットと引数を、一致する Windows ベースラインと比較します。コードが実行されたことを確認する前に、バージョン固有の動作を確認し、後続のサインインと関連付けます。

### Winlogon 通知パッケージ {#winlogon_notify_packages}

**ID:** `winlogon_notify_packages` · **カタログATT&CK:**をチェック[T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei がチェックする内容:** 各即時 Winlogon `Notify` パッケージ登録から `DllName` を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**調査方法:** 通知パッケージは従来の拡張メカニズムです。キャプチャされた Windows バージョンがそれらをロードするかどうかを確認し、承認された認証ソフトウェアと説明されていないライブラリを区別します。

### Winlogon シェル {#winlogon_shell}

**ID:** `winlogon_shell` · **カタログATT&CK:**をチェック[T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei がチェックする内容:** マシン全体およびユーザーごとの Winlogon `Shell` 値を読み取り、デスクトップ シェルの選択を公開します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**調査方法:** エクスプローラーが一般的ですが、キオスクでは別のシェルを使用することもできます。追加されたコマンドまたは異常なパスを調査し、サインイン時にどのユーザーの有効な構成が適用されたかを判断します。

### Winlogon システム {#winlogon_system}

**ID:** `winlogon_system` · **カタログATT&CK:**をチェック[T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei がチェックする内容:** Winlogon の `System` 値を読み取り、カンマ区切りのエントリを個々の候補に区切ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**調査方法:** これをバージョン依存のサインイン構成リードとして扱います。リストを実行履歴として扱うのではなく、参照されているすべてのコンポーネントを検査し、実行証拠で裏付けます。

### Winlogon タスクマン {#winlogon_taskman}

**ID:** `winlogon_taskman` · **カタログATT&CK:**をチェック[T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei がチェックする内容:** 代替タスク管理コマンドのマシン/ユーザー Winlogon `Taskman` 値を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**調査方法:** どの Windows 動作が設定を消費するか、またターゲットが予期されるかどうかを判断します。構成された置換は、ユーザーがその機能を呼び出したという証拠にはなりません。

### Winlogon ユーザー単位 {#winlogon_userinit}

**ID:** `winlogon_userinit` · **カタログATT&CK:**をチェック[T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei がチェックする内容:** マシン/ユーザー Winlogon `Userinit` 値を読み取り、空の末尾項目を無視しながら、プログラムをカンマで区切って分割します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**調査方法:** 通常のエントリは、多くの場合 `userinit.exe` を指します。追加のエントリと変更されたパスを検査します。該当するユーザーを確認し、後でサインインします。 `KeyLastWrite` は、特にこの値ではなく、キー全体に属します。

### Winlogon VMアプレット {#winlogon_vmapplet}

**ID:** `winlogon_vmapplet` · **カタログATT&CK:**をチェック[T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei がチェックする内容:** Winlogon の `VMApplet` コマンド設定を読み取り、古いシステム アプレット起動面をカバーします。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**調査方法:** オペレーティング システムの適用性と予期されるアプレットを確認します。予期しないコマンドは調査する価値がありますが、トリガーとなるインターフェイスが開かれたことは証明されません。

## サービスとシステム拡張 { #services-and-system-extensions }

Windows サービスと拡張可能なシステム コンポーネントは、実行可能ファイルや DLL をロードするための多くの正当な方法を提供します。構成されたターゲットを、予想される製品、サービス アカウント、開始設定、およびソースに基づくインストール履歴と比較します。

### DiagTrack DLL {#diagtrack_dll}

**ID:** `diagtrack_dll` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** DiagTrack サービスの `ImagePath` を読み取ります。カタログの名前は DLL に基づいていますが、このチェックでは、設定されたサービス イメージ コマンドが検査されます。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**調査方法:** サービス ベースラインおよび一般的なサービス チェックと比較します。同じ構成で複数のチェック結果が生成される場合があります。候補数は個別の攻撃の数ではありません。

### DiagTrack リスナー DLL {#diagtrack_listener_dll}

**ID:** `diagtrack_listener_dll` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** DiagTrack リスナー自動ロガー設定から `FileName` を読み取ります。これは ETW 自動ロガー設定であり、DLL がロードされた証拠ではありません。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**調査方法:** 生の値と Windows ログ構成を検査します。カタログではこれを DLL テクニックとしてラベル付けしていますが、ルックアップではロード可能な DLL を検証したり、この値を通じて実行を証明したりすることはありません。

### Netsh ヘルパー DLL {#netsh_helper}

**ID:** `netsh_helper` · **カタログATT&CK:**をチェック[T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**Juntei がチェックする内容:** マシン `NetSh` ヘルパー登録キーの下に空でない値を列挙します。ヘルパーは、DLL ベースのコマンドを使用して `netsh.exe` を拡張できます。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | 空ではないすべての値 |

**調査方法:** ネットワーク製品にはヘルパーがインストールされる場合があります。各値を期待される製品と照合し、解決されたライブラリを検査して、関連する netsh の実行またはイメージのロードを探します。

### プリントモニター {#print_monitors}

**ID:** `print_monitors` · **カタログATT&CK:**をチェック[T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**Juntei がチェックする内容:** アクティブなコントロール セット内の各即時の印刷モニター登録の下にある `Driver` を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**調査方法:** 印刷ソフトウェアとポート モニターはスプーラーを正当に拡張します。 DLL および製品メタデータをプリンター インベントリと比較し、スプーラー/イメージ ロードの証拠でロードを確認します。

### プリントプロセッサ {#print_processors}

**ID:** `print_processors` · **カタログATT&CK:**をチェック[T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**Juntei がチェックする内容:** x64 および x86 プリント プロセッサ登録ツリーの下で `Driver` を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**調査方法:** サードパーティのプリンタ スイートでは、使い慣れた Windows のデフォルト以外にもプロセッサを追加できます。悪意を割り当てる前に、両方のアーキテクチャ、実際の DLL、およびインストールされているプリンタ ソフトウェアを確認してください。

### サービス コントロール マネージャー拡張機能 {#scm_extension}

**ID:** `scm_extension` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** サービス マネージャー拡張サーフェスである `ServiceControlManagerExtension` 内の空でない値を列挙します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | 空ではないすべての値 |

**調査方法:** 名前付きライブラリを調べ、この Windows ビルドが設定を使用するかどうかを調べます。ブート/サービスアクティビティと関連付けます。チェックでは、ロードされた拡張機能を証明するのではなく、登録のインベントリがチェックされます。

### DNS サーバー レベル プラグイン DLL {#server_level_plugin_dll}

**ID:** `server_level_plugin_dll` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** アクティブな SYSTEM コントロール セット内の DNS サービスの `ServerLevelPluginDll` 設定を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**調査方法:** これは、一部のドメイン コントローラーを含む、DNS サーバーの役割を実行しているシステムに関連します。 DLL、サービス アカウント、およびサービスの再起動/負荷の証拠を調査します。役割が非アクティブまたは不在であると、結論が変わります。

### サービス失敗コマンド {#service_failure_command}

**ID:** `service_failure_command` · **カタログATT&CK:**をチェック[T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei がチェックする内容:** アクティブな制御セット内の各サービスの `FailureCommand` を読み取り、設定された回復コマンドを公開します。

**調査された証拠:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`.

**調査方法:** このコマンドは、サービスの回復アクションによって選択され、該当する障害が発生した場合にのみ実行されます。リカバリ構成と障害/プロセスの記録を検査します。値だけでは、アクティブな回復アクションは確立されません。

### SNMP 拡張エージェント DLL {#snmp_extension_agent}

**ID:** `snmp_extension_agent` · **カタログATT&CK:**をチェック[T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei がチェックする内容:** SNMP サービスの `ExtensionAgents` 登録キーの値を列挙します。値は、直接の DLL パスではなく、別のレジストリの場所を識別する場合があります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | 空ではないすべての値 |

**調査方法:** ライブラリへの間接登録に従って、SNMP がインストールされ、使用されていることを確認します。この出力内の未解決の文字列は、DLL が見つからない、または悪意があるという証拠にはなりません。

### タイムプロバイダー {#time_providers}

**ID:** `time_providers` · **カタログATT&CK:**をチェック[T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**Juntei がチェックする内容:** 即時の Windows 時間 `TimeProviders` 登録から `DllName` を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**調査方法:** プロバイダーの有効化設定と Windows タイム サービスの状態を個別に確認します。 DLL パス チェックでは、有効なプロバイダーは必要ありません。時刻同期製品は正規の代替品を提供する場合があります。

### Windows サービス (ServiceDll) {#windows_service_dll}

**ID:** `windows_service_dll` · **カタログATT&CK:**をチェック[T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei がチェックする内容:** アクティブな SYSTEM コントロール セット内のサービスの下にある `Parameters\ServiceDll` を読み取り、利用可能な場合はサービス固有のタイムスタンプ証拠を添付します。

**調査された証拠:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`.

**調査方法:** `svchost.exe` などの共有ホストはペイロード DLL ではありません。ライブラリ、サービス アカウント、開始状態、および一致するサービスのインストール/イメージ読み込みの証拠を検査します。登録はサービスが実行されたことを証明するものではありません。

**さらに詳しい情報:** [Microsoft ドキュメント](https://docs.microsoft.com/en-us/windows/win32/services/service-programs)。

### Windows サービス (ImagePath) {#windows_service_image_path}

**ID:** `windows_service_image_path` · **カタログATT&CK:**をチェック[T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei がチェックする内容:** アクティブな制御セット内のサービス `ImagePath` コマンドを読み取ります。 ImagePath のないドライバー タイプ 1、2、または 8 は、暗黙の `System32\drivers\<service>.sys` ターゲットを保持します。

**調査された証拠:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath`、`Type`、およびサポート サービス メタデータ。

**調査方法:** サービス アカウントを確認し、出力のマシン スコープ ラベルとは別に設定を開始します。無効化された登録またはデマンド開始登録は引き続きインベントリです。タイムスタンプのあいまいな共有実行可能パスよりも、サービス名の証拠が優先されます。

**さらに詳しい情報:** [Microsoft ドキュメント](https://docs.microsoft.com/en-us/windows/win32/services/service-programs)。

### Windows アップデートサービス起動 DLL {#wu_service_startup_dll}

**ID:** `wu_service_startup_dll` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** Windows 更新サービスの `Parameters\ServiceDll` をアクティブなコントロール セットから読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**調査方法:** 同じ Windows ビルドおよびインストールされたサービス状態のライブラリを比較します。これは一般的な ServiceDll インベントリと重複しており、2 番目の明確な侵害を示すものではありません。

## スケジュールされたタスク { #scheduled-tasks }

タスク スケジューラは、XML ファイルと SOFTWARE ハイブの両方に構成を保存します。これらの情報源を一緒に読んでください。タスクは実行せずに登録することができ、不完全なコレクションは非表示タスクまたはゴースト タスクに似ている可能性があります。

### ゴーストスケジュールされたタスク {#ghost_task}

**ID:** `ghost_task` · **カタログATT&CK:**をチェック[T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei がチェックする内容:** SOFTWARE TaskCache `Tree` と `Tasks` 登録を比較し、タスク ファイルがコレクション内に存在する場合は、対応するタスク XML ファイルを検索します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` と `TaskCache\Tasks`。
- ファイルシステム比較のためにキャプチャされた `Windows\System32\Tasks\<task path>` ファイル。

**調査方法:** 対応するものが見つからない場合は、削除された/古いタスク、組み込みの登録、または不完全なコレクションが反映されている可能性があります。 XML が収集されない場合は、ファイルシステムの比較が利用できないことを意味します。タスクが非表示または悪意があると判断する前に、カバレッジと正確な不一致を検査してください。

### 非表示のスケジュールされたタスク {#hidden_scheduled_task}

**ID:** `hidden_scheduled_task` · **カタログATT&CK:**をチェック[T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei がチェックする内容:** TaskCache ツリー登録で欠落している `SD` 値または空のバイナリ セキュリティ記述子、通常の列挙からタスクを非表示にすることに関連する条件をチェックします。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: `SD` のプレゼンス/データ、TaskCache の対応するメタデータ。

**調査方法:** が見つからない/空である SD は、特定の構成異常であり、タスクが実行されたことや、タスクが悪意のあるものであることを証明するものではありません。 XML `Hidden` 設定と任意の ACL 問題は別の条件であり、ここで使用される述語ではありません。

**さらに詳しい情報:** [Microsoft ドキュメント](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/)。

### スケジュールされたタスク (XML ファイル) {#scheduled_task_files}

**ID:** `scheduled_task_files` · **カタログATT&CK:**をチェック[T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei がチェックする内容:** キャプチャされたタスク XML を解析して、Exec コマンド/引数および COM ハンドラー クラス ID/データを取得します。 `Actions Context`/`Principal id` によってプリンシパルを選択し、ユーザーとグループの ID を個別に保持し、SYSTEM、USER、または UNKNOWN を報告します。未解決の COM アクションは表示されたままになります。

**調査された証拠:**

- `Windows\System32\Tasks\<task path>` XML ファイル (Exec、ComHandler、および使用可能なプリンシパル フィールドを含む)。
- ネイティブ ビューと 32 ビット ビューで、ユーザーのキャプチャされた `UsrClass.dat` および NTUSER `Software\Classes` とマシン SOFTWARE `Classes\CLSID` を照合します。
- SID プリンシパルの SOFTWARE `ProfileList` をキャプチャしました。名前付きプリンシパルは、一意に一致する `user.DOMAIN` フォルダーを好み、次にキャプチャされたローカル ホスト サフィックス、最後にプレーンな `user` フォルダーを好みます。一致はヒューリスティックとして明示的にラベル付けされます。本当の曖昧さは未解決のままであり、調査結果と報道で指摘されています。揮発性レジストリ キーは必要ありません。
- 管理対象 `CodeBase` (バージョン サブキーを含む)、AppID 関連付け、および AppID `LocalService` の SYSTEM サービス `ImagePath`。

COM サーバー候補には、`Description` にレジストリ ソースが含まれています。実行可能ファイル名のみの AppID の関連付けは、推測されたディレクトリに割り当てられるのではなく、未解決のままになります。ローカル `file:` URL はデコードされます。リモート URL はネットワークにアクセスしなくても保持されます。ユーザー クラスのオーバーライドの範囲はそのプロファイルに限定され、不完全なユーザー CLSID はシャドウされたマシンの登録には失敗しません。グループのみのタスクでは、カバレッジ エラーの概要に日常的な通知を追加せずに、説明に個別ユーザーの制限を付けてマシン候補を保持します。

**調査方法:** ソース XML およびイベント ログ内のトリガー、有効な状態、プリンシパル、および実行履歴を検査します。作成されたタスクは決して実行されない可能性があります。未解決の COM ハンドラーは依然として候補であり、欠落している/グループのみの主データが SYSTEM を意味するとみなしてはなりません。

## WMI、BITS、およびその他のイベント トリガー { #wmi-bits-and-other-event-triggers }

Windows Management Instrumentation (WMI) はシステム管理をサポートします。バックグラウンド インテリジェント転送サービス (BITS) はバックグラウンド転送をサポートします。どちらもイベントトリガーのコマンドを保持できます。一部のメカニズムは、管理イベント、転送通知、アプリケーションの起動、またはユーザーのアクションを待機します。保存されたトリガーは構成の証拠として役立ちますが、その存在によってトリガーが起動されたことが証明されるわけではありません。

### アクセシビリティ機能 バックドア {#accessibility_tools}

**ID:** `accessibility_tools` · **カタログATT&CK:**をチェック[T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei がチェックする内容:** 収集された参照バイナリから使用可能な Microsoft カタログ メタデータを要求した後、PE OriginalFilename と証明書/カタログ メタデータを使用して、収集された 7 つの System32 アクセシビリティ バイナリを比較します。

**調査された証拠:**

- `Windows\System32\sethc.exe`、`osk.exe`、`narrator.exe`、`magnify.exe`、`utilman.exe`、`atbroker.exe`、および `DisplaySwitch.exe`。
- 収集された `Windows\System32\winlogon.exe`、`cmd.exe`、または `notepad.exe` 参照メタデータおよび該当する収集されたカタログ。

**調査方法:** メタデータは名前変更された置換を提案できますが、Juntei は署名または証明書の信頼性を検証しません。必要なカタログのない疎なコレクションでは、署名されていないように見えるすべてのファイルをバックドアとして扱うのではなく、不完全な比較カバレッジが生成されます。

### アクセシビリティ機能 {#assistive_technology}

**ID:** `assistive_technology` · **カタログATT&CK:**をチェック[T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei がチェックする内容:** 登録されたアクセシビリティ `StartExe` とネイティブ/32 ビット SOFTWARE ビューの `StartParams` コマンドを読み取り、ユーザーおよびユーザーからの有効なアクセシビリティ `Configuration` エントリを読み取ります。 DEFAULT 蕁麻疹。数値/文字列以外の StartExe プレースホルダーはスキップされます。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` および `StartParams`;マシン `Wow6432Node` ビューも表示されます。
- NTUSER.DAT の `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` および DEFAULT ハイブの同等の設定。

**調査方法:** アクセシビリティ ソフトウェアは正規のものです。有効な機能名と実際の実行可能登録を区別し、ターゲットとサインイン/デスクトップ実行の証拠を検査します。これは、アクセシビリティ実行可能ファイル自体を置き換えることとは別のものです。

**さらに詳細:** [テクニックリファレンス (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/)。

### BITS ジョブ通知コマンド {#bits_notify_command}

**ID:** `bits_notify_command` · **カタログATT&CK:**をチェック[T1197](https://attack.mitre.org/techniques/T1197/)

**Juntei がチェックする内容:** キャプチャされた `qmgr.db`、`qmgr0.dat`、または `qmgr1.dat` からカウントされた文字列 BITS ジョブ レコードを回復し、空ではない通知プログラム、パラメーター、ジョブ ID、および所有者 SID を保持します。

**調査された証拠:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`、`qmgr0.dat`、および `qmgr1.dat`。

**調査方法:** これらは刻まれたレコードです: 割り当てられた/現在のジョブの状態は検証されておらず、古い/削除されたジョブが表示される可能性があります。データベースの MFT 書き込み時間はコンテナーの弱い証拠であり、通知の実行時間ではありません。所有者 SID は、SYSTEM、ローカル サービス、ネットワーク サービス、およびユーザーを区別します。

### コマンドプロセッサ AutoRun {#cmd_autorun}

**ID:** `cmd_autorun` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** マシン/ユーザーのコマンド プロセッサ `AutoRun` コマンドを読み取ります。これは、コマンド プロンプトの起動に影響を与える可能性がある設定です。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**調査方法:** 開発ツールとシェルのカスタマイズがその価値を説明している可能性があります。 `cmd.exe /d` バイパスする AutoRunしたがって、構成だけでは cmd.exe が表示されるたびに実行されたことを証明することはできません。コマンドの内容とトリガープロセスの引数を検査します。

**さらに詳しい情報:** [Microsoft ドキュメント](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd)。

### ErrorHandler.cmd の永続化 {#error_handler_cmd}

**ID:** `error_handler_cmd` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** 収集された `Windows\System32\ErrorHandler.cmd` スクリプトの存在をエラー ハンドラー永続化候補として報告します。

**調査された証拠:**

- `Windows\System32\ErrorHandler.cmd`.

**調査方法:** の存在だけでは、該当する Windows リカバリ/エラー パスがそれを呼び出したという証明にはなりません。スクリプトを読み、その出所を調べ、セットアップ/エラー/プロセスの証拠と関連付けます。

### Power Automate デスクトップ フロー {#power_automate}

**ID:** `power_automate` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** 各ユーザーの Power Automate Desktop `flows` および `scripts` ストアの下にあるキャプチャされたフロー/スクリプト ディレクトリを検索し、検出された各フローを 1 回レポートします。

**調査された証拠:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` と `scripts\<flow>\...`。

**調査方法:** これは、スケジュールされたすべてのトリガーや実行されたフローではなく、保存されたオートメーションを識別します。自動または悪意のある永続化として扱う前に、フローの内容と外部起動構成を検査してください。

### PowerShell プロフィール {#powershell_profiles}

**ID:** `powershell_profiles` · **カタログATT&CK:**をチェック[T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**Juntei がチェックする内容:** 認識された Windows PowerShell および PowerShell インストール/プロファイル ディレクトリ内の profile.ps1 スタイル ファイルを検索します。これには、ユーザー ドキュメント、検出された OneDrive ドキュメント、および収集されたドキュメント リダイレクトが含まれます。

**調査された証拠:**

- `profile.ps1` で終わるファイルは、`Windows\System32\WindowsPowerShell\v1.0` および `Windows\SysWOW64\WindowsPowerShell\v1.0` に直接含まれます。
- ユーザーごとの `Documents\WindowsPowerShell` および `Documents\PowerShell` (キャプチャされたドキュメント リダイレクトと検出された OneDrive ドキュメント フォルダーを含む)。
- `profile.ps1` で終わるファイルは、`Program Files\PowerShell` または `Program Files (x86)\PowerShell` の下のバージョン ディレクトリに直接あります。

**調査方法:** プロファイルには無害なカスタマイズが含まれている可能性があります。スクリプトを読み、呼び出されたペイロードに従います。 PowerShell を `-NoProfile` とともに起動すると、それらはスキップされます。ファイルの存在だけでは、プロファイルの実行や悪意のあるコンテンツを証明することはできません。

### スクリーンセーバーハイジャック {#screensaver}

**ID:** `screensaver` · **カタログATT&CK:**をチェック[T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**Juntei がチェックする内容:** 各ユーザーのデスクトップ `SCRNSAVE.EXE` 値を読み取り、構成されたスクリーンセーバー画像を識別します。

**調査された証拠:**

- NTUSER.DATの`HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE`。

**調査方法:** 承認されたスクリーンセーバーと比較し、対象を検査します。他のデスクトップ設定、セッション状態、タイムアウトによって、トリガーできるかどうかが決まります。保存されたパスは、スクリーンセーバーが開始されたことを証明するものではありません。

### テレメトリーコントローラーコマンド {#telemetry_controller}

**ID:** `telemetry_controller` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** TelemetryController およびその直接の子登録から `Binary`、`Command`、および `MaintenanceCommand` を読み取ります。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` とその直接の子: `Binary`、`Command`、`MaintenanceCommand`。

**調査方法:** 選択した実行可能ファイルと引数を、同じ Windows ビルドおよびメンテナンス ソフトウェアと比較します。レジストリ コマンドは、メンテナンス トリガーが起動されたことを示しません。

### Windows 端末カスタム プロファイル {#windows_terminal}

**ID:** `windows_terminal` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** パッケージ化、プレビュー、およびパッケージ化されていない Windows 端末の JSON/JSONC 設定を解析し、空ではないプロファイル/デフォルトの `commandline` 設定をレポートします。

**調査された証拠:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` および WindowsTerminalPreview と同等のもの。
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: 従来のプロファイル配列、`profiles.list`、および `profiles.defaults` コマンドライン フィールド。

**調査方法:** ここには普通の貝殻が自然に出現します。選択された/デフォルトのプロファイルを特定し、異常なコマンドまたは起動引数を検査します。プロファイルを保存するだけでは起動しません。

### WMI イベント サブスクリプション {#wmi_event_subscription}

**ID:** `wmi_event_subscription` · **カタログATT&CK:**をチェック[T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**Juntei がチェックする内容:** キャプチャされた WMI OBJECTS.DATA リポジトリから、認識されたイベント コンシューマ インスタンスとフィルタとコンシューマのバインディング参照を切り分けます。認識されるクラスには、コマンド ライン、アクティブ スクリプト、ログ ファイル、イベント ログ、および SMTP コンシューマが含まれます。

**調査された証拠:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` と `Repository\FS\OBJECTS.DATA`。

**調査方法:** 生のリポジトリ ヒットは、古い、バインドされていない、または履歴である可能性があります。出力ではその資格が保持されます。 Carving は、完全なアクティブなサブスクリプション グラフを再構築したり、トリガーが起動されたことを証明したりしません。リポジトリのファイル書き込み時間は、サブスクリプションの作成/実行時間ではありません。

## エクスプローラー、COM、およびファイルまたは URL ハンドラー { #explorer-com-and-file-or-url-handlers }

コンポーネント オブジェクト モデル (COM) を使用すると、アプリケーションは登録されたソフトウェア コンポーネントをアクティブ化できます。 Windows シェルは、登録されたハンドラーを使用して、ファイル、URL、フォルダー、および特殊オブジェクトを開きます。 COM 識別子 (CLSID) はコンポーネントを識別します。 ProgID は、一部のコンポーネントに読み取り可能な名前を付けます。各登録に従って、コマンドまたは選択した DLL を選択します。

### アプリパスのハイジャック {#app_paths}

**ID:** `app_paths` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** マシン全体の `App Paths` 登録の下にあるデフォルト値を読み取ります。これにより、シェルベースの起動のためにアプリケーション検索をリダイレクトできます。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>` のデフォルト値。

**調査方法:** アプリ パスの登録は、インストールされているアプリケーションに共通であり、すべてのプロセス起動を一般的に書き換えるわけではありません。登録された名前とイメージを予想されるソフトウェアと比較し、関連するシェル/プロセスの証拠を調べます。

### COM オブジェクト ハイジャック {#com_treat_as}

**ID:** `com_treat_as` · **カタログATT&CK:**をチェック[T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei がチェックする内容:** サポートされているマシン CLSID をシャドウする COM `TreatAs` リダイレクトとユーザーごとの InprocServer32/LocalServer32 登録を検索しますネイティブ/32 ビット クラス ストア。 TreatAs ターゲットは、同じユーザーのクラスとマシン クラスを使用して解決されます。

**調査された証拠:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` およびマシン `Classes\Wow6432Node` ビュー。
- NTUSER.DAT 内のユーザーごとの `Software\Classes\CLSID` および UsrClass.dat 内の `CLSID` (実装された `Wow6432Node` ビューを含む): `TreatAs`、`InprocServer32`、および`LocalServer32` のデフォルト値。

**調査方法:** ユーザーごとの上書きには管理者権限は必要ありませんが、正規のソフトウェアでも上書きを登録できます。選択したクラスと DLL/実行可能ファイルに従い、アーキテクチャとアクティベーション コンテキストを評価し、実際の COM アクティベーションを個別に確認します。

### ディスク クリーンアップ ハンドラー ハイジャック {#disk_cleanup_handler}

**ID:** `disk_cleanup_handler` · **カタログATT&CK:**をチェック[T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei がチェックする内容:** ディスク クリーンアップ VolumeCaches ハンドラー CLSID を読み取り、解決可能なマシン COM InprocServer32 のライブラリへの登録を追跡します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` デフォルトの CLSID および `Classes\CLSID\<CLSID>\InprocServer32`。

**調査方法:** ハンドラーを実行するには、関連するクリーンアップ ワークフローによって呼び出される必要があります。未解決のハンドラーはこの特定のチェックでは出力されないため、キャプチャが不完全な場合は、レジストリ パーサーを使用して生の登録を検査します。

### エクスプローラー AppKey オーバーライド {#explorer_app_key}

**ID:** `explorer_app_key` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** Explorer の直接の子 `AppKey` の下にある `ShellExecute` および `Association` の値を読み取り、アプリケーション キーによって選択されたコマンド/アクションをカバーします。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` および `Association`。

**調査方法:** キーボード ユーティリティと OEM ツールでこれらの値を定義できます。関連付け名には、ファイル パスではなく別のレジストリ ピボットが必要な場合があります。それをトリガーするユーザーアクションを確認してください。

### ブラウザヘルパーオブジェクト {#explorer_bho}

**ID:** `explorer_bho` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** ブラウザー ヘルパー オブジェクト CLSID 登録を読み取り、マシンの InprocServer32 ライブラリが利用可能な場合は解決します。未解決のクラス ID は表示されたままになります。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` およびマシンのデフォルト値 `Classes\CLSID\<CLSID>\InprocServer32`。

**調査方法:** これは互換性のあるブラウザ/エクスプローラーの動作に関係しており、特にレガシー環境に関連します。承認されたブラウザ/セキュリティ拡張機能が表示される場合があります。登録された BHO は、現在のブラウザーがそれをロードしたことを証明するものではありません。

### Explorer デスクトップ CLSID ハイジャック {#explorer_clsid_hijack}

**ID:** `explorer_clsid_hijack` · **カタログATT&CK:**をチェック[T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei がチェックする内容:** リストされている 3 つの特別な Explorer 名前空間 CLSID に対して構成されたシェル オープン コマンドをマシン/ユーザー クラス ストアから読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**調査方法:** デスクトップ、マイ コンピューター、およびマイ ドキュメントのコマンド ハンドラーを一致するベースラインと比較します。これは、エクスプローラーの名前空間ハイジャックの可能性をすべてチェックするわけではなく、選択した CLSID とパスをチェックします。

### エクスプローラーのコンテキスト メニュー ハンドラー {#explorer_context_menu}

**ID:** `explorer_context_menu` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** ディレクトリ、ディレクトリ背景、フォルダー、ドライブ、AllFilesystemObjects、および `*` のコンテキスト メニュー ハンドラーを列挙します。可能な場合は、収集された DLL 登録のクラス ID を追跡します。

**調査された証拠:**

- マシン `HKLM\SOFTWARE\Classes` およびユーザー UsrClass.dat クラスには、`<subject>\shellex\ContextMenuHandlers\<handler>` デフォルト値および対応する COM ライブラリ登録が格納されます。

**調査方法:** アーカイブ ツール、セキュリティ製品、クラウド クライアントからの右クリック拡張機能が一般的です。実際のハンドラーとトリガーとなる対話を検査します。未解決の GUID は、DLL が存在しなかったという証拠ではなく、有用な手がかりとして残ります。

### ファイル関連付けのハイジャック {#file_association_hijack}

**ID:** `file_association_hijack` · **カタログATT&CK:**をチェック[T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei がチェックする内容:** `.txt`、`.pdf`、`.doc`、`.docx`、の直接拡張および解決された ProgID `shell\open\command` 登録を検査します。 `.html`、`.htm`、`.js`、`.vbs`、`.hta`、`.exe`、`.bat`、`.cmd`、および `.ps1`。

**調査された証拠:**

- マシン `HKLM\SOFTWARE\Classes` およびユーザー UsrClass.dat クラス ストア: 選択された `<extension>` デフォルトと `<extension-or-ProgID>\shell\open\command` デフォルト。

**調査方法:** 予想されるアプリケーションと比較し、コマンド引数を検査します。これは定義された拡張子セットであり、すべてのファイル タイプや完全な有効な UserChoice 解像度ではありません。保存されたコマンドは、ドキュメントが開かれたことを証明するものではありません。

### プロトコルハンドラーのハイジャック {#protocol_handler_hijack}

**ID:** `protocol_handler_hijack` · **カタログATT&CK:**をチェック[T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei がチェックする内容:** http、https、mailto、ms-msdt、および ms-officecmd の URL オープン コマンド、および検出された `URL Protocol` 登録と該当するユーザー オーバーライドを読み取ります。

**調査された証拠:**

- マシン `HKLM\SOFTWARE\Classes` およびユーザー UsrClass.dat クラスには、`URL Protocol` マーカーと `<scheme>\shell\open\command` のデフォルト値が格納されます。

**調査方法:** ブラウザーとアプリケーション インストーラーはスキームを正当に登録します。コマンドと引数に従い、スキームの予想される目的を評価し、URL の開始/プロセスの証拠と関連付けます。

### ごみ箱 COM 拡張ハンドラ {#recycle_bin_com_extension}

**ID:** `recycle_bin_com_extension` · **カタログATT&CK:**をチェック[T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei がチェックする内容:** リストされたごみ箱を読み取ります。CLSID open/empty/explore コマンドはオーバーライドされ、コンテキスト メニュー/ドラッグ ドロップ ハンドラーの登録が即時に行われます。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**調査方法:** 承認されたシェル拡張機能と比較します。報告される値の一部は、直接解決されたライブラリではなく、COM 識別子です。コードの実行をアサートする前に、参照されるクラスとユーザーのアクションを調査してください。

### 検索プロトコル ハンドラー ハイジャック {#search_protocol_handler}

**ID:** `search_protocol_handler` · **カタログATT&CK:**をチェック[T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei がチェックする内容:** マシン/ユーザーの `search-ms\shell\open\command` デフォルト ハンドラーを読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**調査方法:** ハンドラーの変更により、search-ms URL が開いたときの動作が変わる可能性がありますが、その存在はエクスプロイト配信の証拠ではありません。 Windows のデフォルトと比較し、それを消費する URL/プロセス アクティビティを調査します。

### SharedTaskScheduler {#shared_task_scheduler}

**ID:** `shared_task_scheduler` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** 空ではない Explorer `SharedTaskScheduler` 値を登録されたシェル コンポーネントとして列挙します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | 空ではないすべての値 |

**調査方法:** 値は、実行可能パスではなく COM クラスを参照している可能性があります。サーバー登録を追跡し、オペレーティング システムの動作、拡張機能の来歴、関連するシェル読み込みの証拠を確認します。

### ShellExecuteHooks {#shell_execute_hooks}

**ID:** `shell_execute_hooks` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** 空ではない Explorer `ShellExecuteHooks` 値を列挙し、従来のシェル実行フック登録をカバーします。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | 空ではないすべての値 |

**調査方法:** 参照されている CLSID を解決し、Windows ビルドとポリシーがフックを許可しているかどうかを確認します。レガシー登録は使用されない場合があります。値自体は観察されたシェル実行ではありません。

### [スタート] メニュー ライブラリと CLSID フォルダー COM の永続性 {#start_menu_com}

**ID:** `start_menu_com` · **カタログATT&CK:**をチェック[T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei がチェックする内容:** `.library-ms` ライブラリ `simpleLocation` URL が COM CLSID を参照する XML ファイル、および名前付きディレクトリを検索します。 `<name>.{CLSID}`、`Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu` の下。エクスプローラーとこれらのオブジェクトの対話により、COM コードがアクティブ化される可能性があります。チェックには、キャプチャされた空のディレクトリが含まれます。シンボリックリンクをたどりません。 [スタート] メニューの外にある通常のライブラリ ファイルと XML コメント内の GUID は、このチェックでは報告されません。

**調査された証拠:**

- ユーザーごとのスタート メニュー `.library-ms` ファイルおよび CLSID という名前のディレクトリ エントリ。
- ユーザーのキャプチャされたクラス登録とマシン SOFTWARE をアーキテクチャ別に照合します。
- スケジュールされたタスクと同じリゾルバーを通じて、CodeBase/AppID/サービス構成を管理します。

**調査方法:** `Path` および `SourceFile` は、アクティベーション トリガー アーティファクトを識別します。 `Value` には候補サーバー コマンドが含まれます。解決できない場合は CLSID が含まれます。 `Description` は、トリガーを CLSID およびキャプチャされたレジストリ ソースにリンクします。 `ResolvedPath`/`Sha256` は、解決され、収集されたペイロードのみを記述します。実行可能ファイル名のみの AppID の関連付けは未解決のままです。トリガーは正当な場合もあります。存在は、サインイン、アクティブ化、実行、または攻撃を証明するものではありません。ライブラリ URL、クラス登録、ペイロード、ユーザー コンテキスト、および独立した実行ログを検査します。

**Collection の注意:** ファイルのみのコレクションでは、空のディレクトリを省略できます。標準ユーザーのスタート メニューのパスが検査されます。リダイレクトされたスタート メニューおよびその他の Explorer 名前空間の場所は、このチェックの対象になりません。じんましんの欠落または未解決の登録により、候補者を保持しながら部分的なカバレッジが生成されます。タスクのアーキテクチャ、昇格、およびポリシーによって、効果的な COM ルックアップ動作が変更される可能性があります。

**詳細:** [原案および研究者のクレジット](https://github.com/last-byte/PersistenceSniper/pull/32)、[COMユーザー/マシンごとの統合ビュー](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root)、 [MITRE COM ハイジャック](https://attack.mitre.org/techniques/T1546/015/)。

### TypeLib COM ハイジャック {#typelib_hijack}

**ID:** `typelib_hijack` · **カタログATT&CK:**をチェック[T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei がチェックする内容:** ユーザーごとの UsrClass.dat TypeLib GUID/version/locale win32/win64 登録を読み取り、パスとスクリプト モニカーの値を保持します。末尾の数値リソース インデックスは、ターゲット ファイルの解決のためにのみ削除されます。

**調査された証拠:**

- キャプチャされた各 UsrClass.dat の `TypeLib\<GUID>\<version>\<locale>\win32` および `win64` のデフォルト値。

**調査方法:** タイプ ライブラリは、正規の DLL、代替ファイル、またはスクリプト モニカを指すことができます。小切手はこれらの登録の一覧表を作成します。デフォルト以外のパスがすべて攻撃であると想定するのではなく、影響を受けるアプリケーションを評価し、タイプ ライブラリの読み込みを確認します。

## Office アプリケーションとアドイン { #office-applications-and-add-ins }

Office は、正規のアドイン、テンプレート、アプリケーション固有の拡張機能をサポートしています。不審な登録は、特定の Office バージョン、アプリケーション、読み込み設定、またはユーザー操作に依存する可能性があります。構成だけではマクロの実行は確立されません。

### Office アドインの登録 {#office_addins}

**ID:** `office_addins` · **カタログATT&CK:**をチェック[T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei がチェックする内容:** バージョン管理されていない Word、Excel、PowerPoint、Outlook、Access アドインおよび 14.0/15.0/16.0 Office ルート (実装されたマシンを含む) を列挙します。ストアとユーザーのルート。明示的なマニフェスト/FileName/パス値を保持するか、LoadBehavior コンテキストを使用して適格な COM ベースのエントリを解決します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` および実装されたマシンのクイック実行と同等のもの。
- NTUSER.DATの`HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>`; SOFTWARE、NTUSER.DAT、および UsrClass.dat からのマシン/ユーザー COM クラスの一致。

**調査方法:** アドインは一般的なビジネス ソフトウェアです。 LoadBehavior、アプリケーション/バージョン、マニフェスト、実際のペイロード、および予想されるベンダーを確認します。このチェックでは、インストールされているすべてのアドイン形式または登録ビューが収集またはロードされていることは保証されません。

### Office AI アドイン ハイジャック {#office_ai_hijack}

**ID:** `office_ai_hijack` · **カタログATT&CK:**をチェック[T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei がチェックする内容:** Office 16.0 クイック実行共通 AI 登録場所の値を列挙します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: すべての値。

**調査方法:** 異常な値とパスを、インストールされている Office ビルドと比較します。値の存在は、どの Office 機能がその値を消費したかを示したり、その値が実行可能コードを選択したことを証明したりするものではありません。

### Office DLL オーバーライド {#office_dll_override}

**ID:** `office_dll_override` · **カタログATT&CK:**をチェック[T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei がチェックする内容:** バージョン管理されていない Word/PowerPoint ルートおよび 14.0/15.0/16.0 の下にある `WwlibtDll` および `PPCoreTDLL` を読み取ります。実装されたクイック実行およびユーザー ルートを含む設定。

**調査された証拠:**

- マシン/ユーザーの `Microsoft\Office\[<version>\]Word` および `PowerPoint` 設定、および実装されたマシンのクイック実行 Office ツリー: `WwlibtDll` および `PPCoreTDLL`。

**調査方法:** どのアプリケーション/ビルドがオーバーライドを認識するかを特定し、ライブラリと出所を調べて、Office の起動またはイメージの読み込みと関連付けます。設定されたオーバーライドだけでは実行の証明にはなりません。

### Office のデフォルトのテンプレート {#office_templates}

**ID:** `office_templates` · **カタログATT&CK:**をチェック[T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**Juntei がチェックする内容:** Normal.dotm/NormalEmail.dotm ファイルと、Office インストールのスタートアップ フォルダーを含む、認識された Word スタートアップ フォルダーおよび Excel の XLSTART フォルダーに直接収集されたインベントリ。

**調査された証拠:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` と `NormalEmail.dotm`。
- ローミング AppData の下のユーザー `Microsoft\Word\STARTUP` および `Microsoft\Excel\XLSTART` の直接ファイル。
- Program Files および Program Files (x86) の下にある認識された Office `<version>\STARTUP` / `XLSTART` および `root\<version>\STARTUP` / `XLSTART` フォルダー内のファイルを直接指定します。

**調査方法:** テンプレートは正規のものである場合もあれば、実行可能なコンテンツが含まれていない場合もあります。 Juntei は、このチェックではマクロの抽出または実行を行いません。ドキュメントの内部、マクロ ポリシー、およびアプリケーションのアクティビティを検査します。マシンのインストール パスは SYSTEM プロセス トークンを意味しません。

### オフィス テスト DLL {#office_test_dll}

**ID:** `office_test_dll` · **カタログATT&CK:**をチェック[T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**Juntei がチェックする内容:** リストされたマシン/ユーザーおよび 32 ビット マシン ビューから Office Test Special Perf キーの値を列挙します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | 空ではないすべての値 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | 空ではないすべての値 |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | 空ではないすべての値 |

**調査方法:** 予期しないオーバーライドを、Office バージョンとペイロードの検証が必要なリードとして扱います。このチェックでは、特定の Office インストールが値を読み込むかどうかをテストするのではなく、構成が報告されます。

### Outlook ホームページ攻撃 {#outlook_home_page}

**ID:** `outlook_home_page` · **カタログATT&CK:**をチェック[T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**Juntei がチェックする内容:** Office バージョン 14.0、15.0、および 16.0 のユーザー Outlook WebView の子の下にある `URL` を読み取ります。

**調査された証拠:**

- NTUSER.DATの`HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL`。

**調査方法:** フォルダーのホーム ページは正当である可能性がありますが、動作は Outlook のバージョンとセキュリティ更新プログラムによって異なります。 URL/コンテンツと有効なポリシーを検査し、Outlook/web/プロセスの証拠と関連付けます。

### VBA モニター DLL ハイジャック {#vba_monitors}

**ID:** `vba_monitors` · **カタログATT&CK:**をチェック[T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei がチェックする内容:** マシンおよびユーザーごとのクラス ストアに実装された 2 つの VBA モニター関連 CLSID の InprocServer32 値を読み取ります。

**調査された証拠:**

- マシン SOFTWARE 内の `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` および `{0002E157-0000-0000-C000-000000000046}`、および同等のユーザー UsrClass.dat 登録。

**調査方法:** 選択したライブラリを、インストールされている Office/VBA 環境と比較します。これは対象を絞った CLSID チェックであり、すべての VBA 拡張機能のインベントリやマクロ実行の証拠ではありません。

## デバッガー、クラッシュ ハンドラー、互換性シム { #debuggers-crash-handlers-and-compatibility-shims }

デバッガーまたはエラー ハンドラーは、アプリケーションの起動、クラッシュ、ハング、または終了時に実行できます。アプリケーション互換性データベースも起動に影響を与える可能性があります。報告されたコマンドとともに、前提条件の設定とトリガー プログラムを調査します。

### AeDebug デバッガー ハイジャック {#ae_debug}

**ID:** `ae_debug` · **カタログATT&CK:**をチェック[T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei がチェックする内容:** ジャストインタイム クラッシュ デバッグ構成で使用されるネイティブ コマンドと 32 ビット AeDebug `Debugger` コマンドを読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**調査方法:** 開発/デバッグ ソフトウェアがこれを正当に設定している可能性があります。デバッガー コマンドとクラッシュ トリガーの前提条件をすべて調べて、アプリケーション エラーとプロセス イベントを関連付けます。

### AeDebug 保護されたプロセス デバッガー {#ae_debug_protected}

**ID:** `ae_debug_protected` · **カタログATT&CK:**をチェック[T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei がチェックする内容:** 保護されたプロセスのデバッグ サーフェスのネイティブ コマンドと 32 ビット AeDebugProtected `Debugger` コマンドを読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**調査方法:** オペレーティング システムのビルドおよび保護ポリシーで設定されたコマンドを使用できるかどうかを確認します。保存されたデバッガーは、保護されたプロセスがクラッシュしたこと、またはコマンドが実行されたことを証明するものではありません。

### イメージファイル実行オプションデバッガ {#ifeo_debugger}

**ID:** `ifeo_debugger` · **カタログATT&CK:**をチェック[T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei がチェックする内容:** ネイティブ/32 ビットのイメージ ファイル実行オプションの登録 (ネストされたフィルター キーを含む) で `Debugger` 値を検索します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger` (ネストされたフィルター キーとマシン `Wow6432Node` ビューを含む)。

**調査方法:** 名前付きイメージの起動時にデバッガーを選択できます。イメージ/フィルターのスコープとデバッガーの引数を検査します。開発者ツールでも IFEO を使用できます。ターゲットの起動をトレースして実行を確立します。

### IFEO 委任された NTDLL {#ifeo_delegated_ntdll}

**ID:** `ifeo_delegated_ntdll` · **カタログATT&CK:**をチェック[T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei がチェックする内容:** ネイティブ/32 ビットのイメージ ファイル実行オプションの登録 (ネストされたキーを含む) で `VerifierDlls` オーバーライドを検索します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls` (ネストされたキーとマシン `Wow6432Node` ビューを含む)。

**調査方法:** チェックの述語は、構成された VerifierDlls 値です。すべての検証者/委任の前提条件が有効であることを証明するものではありません。有効な注入パスを推測する前に、該当するフラグ、Windows バージョン、およびペイロードを確認してください。

### サイレントプロセス終了モニター {#ifeo_silent_process_exit}

**ID:** `ifeo_silent_process_exit` · **カタログATT&CK:**をチェック[T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei がチェックする内容:** ネストされたキーを含む、`MonitorProcess` のネイティブ/32 ビット SilentProcessExit 登録を検索します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess` (ネストされたキーとマシン `Wow6432Node` ビューを含む)。

**調査方法:** サイレント終了監視には、関連するレポート/IFEO 構成も必要です。これらの設定と監視対象イメージの終了イベントを検査します。正規の診断ツールを使用すると、モニターの原因が判明する可能性があります。

### アプリケーションシミング {#installed_sdb}

**ID:** `installed_sdb` · **カタログATT&CK:**をチェック[T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei がチェックする内容:** インストールされているアプリケーション互換性データベースの即時 InstalledSDB 登録から `DatabasePath` を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**調査方法:** 互換性修正は正規のものである可能性があります。 SDB の内容、対象の実行可能ファイル、およびインストール履歴を検査します。データベース登録では、その shim は識別されず、一致するアプリケーションが実行されたことも証明されません。

### LSM デバッガー ハイジャック {#lsm_debugger}

**ID:** `lsm_debugger` · **カタログATT&CK:**をチェック[T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei がチェックする内容:** ネイティブ マシン ビューおよび 32 ビット マシン ビューの `lsm.exe` の SilentProcessExit の下にある `MonitorProcess` を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**調査方法:** これは、より広範な SilentProcessExit インベントリと重複します。構成を実行された永続性として扱う前に、前提条件フラグと関連する LSM 終了を確認してください。異なる結果を評価する場合は、同じエントリを 1 回カウントします。

### MiniDump 補助 DLL {#minidump_auxiliary_dlls}

**ID:** `minidump_auxiliary_dlls` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** MiniDumpAuxiliaryDlls 内の名前付きの空でない値を列挙し、値の名前をトリガー モジュール コンテキストとして、値のデータを選択されたターゲットとして保持します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: トリガーモジュールの値の名前とターゲットデータ。

**調査方法:** トリガー モジュールと補助 DLL の両方を検査します。クラッシュ/ミニダンプ ツールによってエントリが説明される場合があり、登録ではダンプ操作によってライブラリがロードされたことが示されません。

### アプリケーションシミング {#shim_custom}

**ID:** `shim_custom` · **カタログATT&CK:**をチェック[T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei がチェックする内容:** 直接 AppCompatFlags カスタム アプリケーション キーの下にある `.sdb` という名前の値を読み取り、アプリケーションとデータベースの関連付けを保持します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: `.sdb` で終わる値の名前。

**調査方法:** これらの関連付けを、InstalledSDB および実際の互換性データベースと組み合わせます。ファイル名の関連付けだけでは、データベースの動作が明らかになったり、一致するアプリケーションの起動が示されたりすることはありません。

### WER デバッガー ハイジャック {#wer_debugger}

**ID:** `wer_debugger` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** ネイティブ/32 ビットを読み取ります Windows エラー `Debugger` 値を報告します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**調査方法:** 正規の診断ソフトウェアによってデバッガが構成されている可能性があります。完全なコマンドと関連するクラッシュ イベントを調査します。登録は、Windows エラー報告が開始したことを証明するものではありません。

### WER デバッガー ハイジャックがハングする {#wer_hangs}

**ID:** `wer_hangs` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** ネイティブ/32 ビットの読み取り Windows エラー報告 `Debugger` 値がハングします。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**調査方法:** ハング処理ワークフローがこの設定を呼び出し、ターゲット プログラムを検査するかどうかを確認します。レジストリの書き込みをデバッガーの実行として扱うのではなく、アプリケーションのハングと関連付けてレコードを処理します。

### WER ReflectDebugger ハイジャック {#wer_reflect_debugger}

**ID:** `wer_reflect_debugger` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** ネイティブ/32 ビットを読み取ります Windows エラー `ReflectDebugger` 値を報告します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**調査方法:** 承認された診断構成と比較し、Windows のバージョン/トリガーの適用性を確立します。実行を特定する前に、コマンド引数を検査し、クラッシュ/プロセスの確証を求めます。

### WER ランタイム例外モジュール {#wer_runtime_exception}

**ID:** `wer_runtime_exception` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** ネイティブ/32-bit RuntimeExceptionHelperModules の下に値名を列挙します。ここでは、ライブラリ パスは値の名前に含まれており、必ずしもそのデータに含まれているわけではありません。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` およびマシン `Wow6432Node` ビュー: 値の名前。

**調査方法:** 指定されたモジュールとアプリケーション/ランタイム コンテキストを検査します。データ内の小さな数値はペイロード パスではないため、登録だけではヘルパーがロードされたことを証明できません。

## .NET ランタイムとプロファイラーの設定 { #net-runtime-and-profiler-settings }

マネージド ランタイムは、アプリケーション コードの前にプロファイラー、スタートアップ アセンブリ、またはデバッグ ライブラリを読み込むことができます。これらの設定は、開発者や監視製品によっても使用されます。ランタイム、プロセス アーキテクチャ、有効フラグ、および新しく開始されたプロセスが設定を継承できるかどうかを確認します。

### AppDomainManager インジェクション {#appdomain_manager}

**ID:** `appdomain_manager` · **カタログATT&CK:**をチェック[T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**Juntei がチェックする内容:** マシン/ユーザーの `APPDOMAIN_MANAGER_ASM` および `APPDOMAIN_MANAGER_TYPE` 環境変数を読み取り、カスタム .NET AppDomain マネージャーのアセンブリとタイプの選択をカバーします。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**調査方法:** アプリケーションは意図的にカスタム マネージャーを使用する可能性があります。アセンブリと型を一緒に解決し、実行時の互換性とプロセスの継承をチェックします。単一の変数もそのキーのタイムスタンプも、マネージド アプリケーションがその変数をロードしたことを証明するものではありません。

### .NET CLR プロファイラー ハイジャック {#cor_profiler}

**ID:** `cor_profiler` · **カタログATT&CK:**をチェック[T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei がチェックする内容:** マシン/ユーザーの COR プロファイリング環境設定を読み取り、個々のサービスの環境複数文字列内の名前付きエントリと一致します。有効フラグ、プロファイラー識別子、およびアーキテクチャ固有のパスが保持されます。

**調査された証拠:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` とキャプチャされた各ユーザー NTUSER.DAT `Environment`。
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: 複数文字列の名前付き COR 設定。

**調査方法:** プロファイラは正当な監視をサポートしています。有効化フラグ、CLSID、および DLL パスは、構成の異なる部分です。各行を実行されたプロファイラーとして扱うのではなく、効果的な有効化と関連する .NET プロセスを確認します。

### .NET コア CLR プロファイラー ハイジャック {#coreclr_profiler}

**ID:** `coreclr_profiler` · **カタログATT&CK:**をチェック[T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei がチェックする内容:** 対応するマシン/ユーザー CORECLR プロファイリング環境設定および一致するサービス環境エントリ (アーキテクチャ固有のパスを含む) を読み取ります。

**調査された証拠:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` とキャプチャされた各ユーザー NTUSER.DAT `Environment`。
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: 複数文字列の名前付き CORECLR 設定。

**調査方法:** .NET ランタイム、アーキテクチャ、有効フラグ、およびライブラリを検証します。承認された可観測性製品はこれらの設定を構成する場合があり、既存のプロセスは新しく変更された環境を継承しない場合があります。

### .NET DbgManagedDebugger ハイジャック {#dotnet_dbg_managed_debugger}

**ID:** `dotnet_dbg_managed_debugger` · **カタログATT&CK:**をチェック[T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei がチェックする内容:** リストされたマシン/ユーザー `.NETFramework` `DbgManagedDebugger` 設定 (明示的なマシン 32 ビット パスを含む) を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**調査方法:** 該当する .NET ランタイム/デバッグ ポリシーでこのコマンドが使用されているかどうかを確認します。開発者ツールがそれを説明してくれるかもしれません。管理対象のクラッシュとプロセスの作成を関連付けて、実行を確立します。

### .NET フレームワーク COR_PROFILER レジストリ キー {#dotnet_framework_profiler}

**ID:** `dotnet_framework_profiler` · **カタログATT&CK:**をチェック[T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei がチェックする内容:** リストされた `.NETFramework` レジストリ ビューから `COR_PROFILER`、`COR_PROFILER_PATH`、および `COR_ENABLE_PROFILING` を読み取ります。これは、環境ベースのプロファイラー チェックを補完します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**調査方法:** 値をまとめて評価し、ランタイムがそのレジストリの場所を尊重するかどうかを判断します。構成されたプロファイリング ツールは正規のものである可能性があります。スタンドアロン有効フラグを、識別された DLL ロードとして解釈しないでください。

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**ID:** `dotnet_startup_hooks` · **カタログATT&CK:**をチェック[T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei がチェックする内容:** マシン/ユーザーの `DOTNET_STARTUP_HOOKS` 環境値を読み取ります。これにより、サポートされている .NET ランタイムの起動フック アセンブリを選択できます。

**調査された証拠:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` と各ユーザー NTUSER.DAT `Environment\DOTNET_STARTUP_HOOKS`。

**調査方法:** 複数のアセンブリの値全体を検査し、ランタイム サポートを確認します。このチェックでは、サービス固有の起動フック環境を列挙したり、プロセスが値を継承して使用したことを証明したりすることはありません。

### 既知のマネージド デバッグ DLL {#known_managed_debugging_dlls}

**ID:** `known_managed_debugging_dlls` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** `.NETFramework` `KnownManagedDebuggingDlls` 登録をマネージド デバッガー ライブラリ オーバーライドの可能性として読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**調査方法:** インストールされている開発/ランタイム ツールと比較し、トリガーとなるデバッガーを特定します。レジストリ エントリは、デバッガー セッションまたはライブラリのロードを証明するものではありません。

## DLL および実行可能ルックアップのオーバーライド { #dll-and-executable-lookup-overrides }

これらのチェックでは、コンポーネントがコードを検索する方法に影響を与える明示的なライブラリの場所または設定が検査されます。パスのオーバーライドまたは検索ポリシーの弱体化は、調査の手がかりとなります。実行にはペイロードと互換性のあるトリガー コンポーネントが引き続き必要です。

### AppCert DLL {#appcert_dlls}

**ID:** `appcert_dlls` · **カタログATT&CK:**をチェック[T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**Juntei がチェックする内容:** 空ではないセッション マネージャー AppCertDlls 登録、プロセス作成 API に関連付けられたライブラリ拡張サーフェスを列挙します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | 空ではないすべての値 |

**調査方法:** 各ライブラリとそのインストールの出所を調べます。適用性と実行時の負荷を個別に判断します。タイムラインでの通常のプロセスの作成自体は、構成された DLL がロードされたことを証明するものではありません。

### AppInit DLL {#appinit_dlls}

**ID:** `appinit_dlls` · **カタログATT&CK:**をチェック[T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**Juntei がチェックする内容:** ネイティブ/32 ビット `AppInit_DLLs` を読み取り、ライブラリ リストを分割し、利用可能な `LoadAppInit_DLLs` および `RequireSignedAppInit_DLLs` コンテキストを各候補に追加します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs`、`LoadAppInit_DLLs`、および `RequireSignedAppInit_DLLs`。マシン `Wow6432Node` ビューも表示されます。

**調査方法:** ゼロの LoadAppInit_DLLs フラグは、明示的に非アクティブとしてマークされます。 Windows のバージョン、セキュア ブート、署名要件も読み込みに影響します。 AppInit DLL の実行を要求する前に、有効な設定を確認してください。

**さらに詳しい情報:** [Microsoft ドキュメント](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls)。

### AutodialDLL オーバーライド {#autodial_dll}

**ID:** `autodial_dll` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** WinSock2 パラメータ `AutodialDLL` オーバーライドを読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**調査方法:** この設定を使用するネットワーク機能とソース Windows のバージョンを確認します。実際の DLL とネットワーク/プロセス アクティビティを確認します。オーバーライドだけでは、アクティブなロードや悪意のあるロードは確立されません。

### CHM ヘルパー DLL {#chm_helper_dll}

**ID:** `chm_helper_dll` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** HtmlHelp Author の下の `Location` を読み取ります。ヘルプ関連のライブラリ/場所オーバーライド サーフェスです。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**調査方法:** 関連するヘルプ コンポーネントがこのシステム上の値を使用しているかどうかを確認し、選択したファイルを調べます。レジストリ オーバーライドは、CHM ドキュメントが開かれたことを証明するものではありません。

### NL6 ContentIndex DLL オーバーライド {#content_index_dll}

**ID:** `content_index_dll` · **カタログATT&CK:**をチェック[T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei がチェックする内容:** 即時アクティブ コントロール セット ContentIndex 言語登録から `DLLOverridePath` を読み取ります。

**調査された証拠:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`.

**調査方法:** 言語/検索コンポーネントは Windows ビルドによって異なる場合があります。ライブラリと一致するインストール済みコンポーネントを検査します。実際のインデックス作成プロセスに従って、読み込みを確認します。

### 暗号 ExpoOffload DLL {#crypto_expo_offload}

**ID:** `crypto_expo_offload` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** 暗号化オフロード `ExpoOffload` 値を暗号化ライブラリ オーバーライド候補として読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**調査方法:** 機能/ビルドの適用性を確認し、選択したライブラリの起源を調査します。珍しい値の存在を、資格情報の盗難または実行されたコードの証拠として扱わないでください。

### Direct3D ソフトウェア ラスタライザー DLL {#direct3d_dll}

**ID:** `direct3d_dll` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** Direct3D ドライバーの `SoftwareRasterizer` ライブラリ オーバーライドを読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**調査方法:** グラフィックスおよび互換性ソフトウェアは、ラスタライザーの構成に影響を与える可能性があります。 DLL と互換性のある Direct3D アプリケーションのアクティビティを検査します。パスを保存しても、パスがロードされたことは証明されません。

### DLL 検索順序の弱体化 {#dll_search_mode}

**ID:** `dll_search_mode` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** アクティブなセッション マネージャーの値が解析されてゼロになった場合にのみ、`SafeDllSearchMode` または `CWDIllegalInDllSearch` を報告します。これらは検索ポリシー条件であり、選択されたペイロードではありません。

**調査された証拠:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` および `CWDIllegalInDllSearch`。

**調査方法:** 検索ポリシーが弱まると、別の検索順序攻撃が容易になる可能性があります。植え付けられた DLL を特定したり、実行を証明したりするものではありません。ゼロ以外の値または存在しない値は、このチェックでは出力されません。実質的なデフォルトとポリシーを個別に評価します。

**さらに詳しい情報:** [Microsoft ドキュメント](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security)。

### 既知のDLLの除外 {#exclude_from_known_dlls}

**ID:** `exclude_from_known_dlls` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** 保護された/既知の DLL 検索動作の変更として、空ではない `ExcludeFromKnownDlls` エントリをセッション マネージャーから読み取ります。

**調査された証拠:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`.

**調査方法:** 除外は、置換ライブラリ自体ではなく、検索条件を識別します。影響を受けるアプリケーションと潜在的な DLL 検索場所を見つけて、非標準のイメージのロードを裏付けます。

**さらに詳しい情報:** [Microsoft ドキュメント](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order)。

### hhctrl.ocx DLL オーバーライド {#hhctrl_ocx_dll}

**ID:** `hhctrl_ocx_dll` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** ネイティブ/32 ビット マシンとサポートされているユーザー クラス ストアを読み取ります。実装された HTML の InprocServer32 オーバーライドは、CLSID を制御するのに役立ちます。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**調査方法:** コントロールをアクティブ化しているアプリケーションとアーキテクチャを特定します。登録された DLL およびヘルプ関連アクティビティを検査します。 COM 登録は、HTML ヘルプがコンテンツをレンダリングしたことを証明するものではありません。

### 既知の DLL {#known_dlls}

**ID:** `known_dlls` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** アクティブなセッション マネージャーの KnownDLLs キーの名前付き値を列挙します。

**調査された証拠:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: 名前付きの値。

**調査方法:** 多くのエントリは Windows ライブラリであると予想され、一部の値はペイロード パスではなく構成をサポートしています。完全なキーを信頼できる一致ビルドと比較します。このチェックでは、代替アーキテクチャ固有のすべての KnownDLL ツリーがスキャンされるわけではありません。

### LSA 拡張子 DLL {#lsa_extensions}

**ID:** `lsa_extensions` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** LsaExtensionConfig LsaSrv 構成から `Extensions` を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**調査方法:** セキュリティ ソフトウェアは認証コンポーネントを拡張できます。実際のライブラリとソース Windows バージョンとの互換性を検査し、LSASS イメージ ロードまたはその他の独立した使用の証拠を探します。

### MAPI32 DLL パス オーバーライド {#mapi32_dll_path}

**ID:** `mapi32_dll_path` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** ネイティブ マシン ビューおよび 32 ビット マシン ビューでの即時メール クライアント登録の下にある `DLLPathEx` および `DLLPath` を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**調査方法:** メール クライアントは正規の MAPI プロバイダーを登録します。どのクライアント/プロバイダーが選択されたかを判断し、実際のライブラリを検査します。インストールされているプロバイダーは、インシデント中にアプリケーションが MAPI を使用したことを証明しません。

### MSDTC XA DLL {#msdtc_xa_dll}

**ID:** `msdtc_xa_dll` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** MSDTC MTxOCI 構成から `OracleXaLib` および `OracleOciLib` を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**調査方法:** データベース統合にはこれらのライブラリが正当に必要になる場合があります。トランザクション コーディネーターの構成とインストールされているデータベース ソフトウェアを確認し、構成されたパスが実行されたと仮定するのではなく、サービス/モジュールの使用と関連付けます。

### NLDP DLL オーバーライド {#nldp_dll}

**ID:** `nldp_dll` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** NlsData `DllOverridePath` 構成値を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**調査方法:** オーバーライドを使用する可能性のあるターゲット ライブラリ、Windows ビルド、および自然言語コンポーネントを確認します。この値は、インデックス作成または言語処理が発生したことを示しません。

### RDP テストDVCPプラグイン DLL {#rdp_test_dvc_plugin}

**ID:** `rdp_test_dvc_plugin` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** マシンのターミナル サーバー クライアント `TestDVCPlugin` ライブラリ設定を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**調査方法:** 見慣れないテスト プラグインとバージョン固有の動作を調査します。実行を確立するには、互換性のあるクライアント セッションとライブラリのロードが必要です。レジストリ値だけでは表示されません。

### 検索インデクサー DLL オーバーライド {#search_indexer_dll}

**ID:** `search_indexer_dll` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** Windows を読み取り、SOFTWARE から `DllPath` オーバーライドを検索します。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**調査方法:** 一致する検索コンポーネント/バージョンを特定し、ターゲット ファイルを検査します。非アクティブな Windows 検索サービスまたは未使用の設定は、観察された悪意のあるライブラリのロードとは異なります。

### ウィンソック AutoProxy DLL {#winsock_auto_proxy}

**ID:** `winsock_auto_proxy` · **カタログATT&CK:**をチェック[T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei がチェックする内容:** 直接の NameSpace_Catalog5 Catalog_Entries および Catalog_Entries64 登録の下にある `LibraryPath` を読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**調査方法:** ネットワーク プロバイダーは正規のものである可能性があります。両方のアーキテクチャ、プロバイダーの来歴、および関連するネットワーク アプリケーション/イメージ読み込みの証拠を確認します。この対象を絞った名前空間プロバイダーの検索は、考えられるすべての Winsock 拡張機能ではありません。

## アカウントとサインインの可視性 { #accounts-and-sign-in-visibility }

ローカル アカウント レコードとサインインの可視性設定により、ID ベースの永続性が明らかになります。これらのチェックは、キャプチャされた SAM および SOFTWARE ハイブを検査します。これらは、Active Directory アカウント、ドメイン グループ メンバーシップ、または過去のログオンのインベントリではありません。

### 管理者グループのメンバーシップ {#admin_group_membership}

**ID:** `admin_group_membership` · **カタログATT&CK:**をチェック[T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei がチェックする内容:** SAM からローカル管理者のエイリアス メンバー リストをデコードします。キャプチャされた SAM に非表示のアカウント、またはアカウント/グループを指定していないローカル マシンの SID が報告されます。

**調査された証拠:**

- `SAM\Domains\Builtin\Aliases\00000220\C` そして機械 SID/user/group レコードの下にあります `SAM\Domains\Account` で SAM 巣箱。
- 隠し名の相関の場合は `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`。

**調査方法:** これは、対象を絞った異常チェックであり、すべての管理者のリストではありません。外部/ドメイン SID は、欠落しているローカル ユーザーとして評価されません。削除されたアカウント、キャプチャのタイミング、または不完全なハイブについては、悪意のあるメンバーシップを締結する前に調査が必要です。

### 非表示のローカルアカウント {#hidden_account}

**ID:** `hidden_account` · **カタログATT&CK:**をチェック[T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**Juntei がチェックする内容:** Winlogon SpecialAccounts UserList の数値ゼロ値を報告し、通常のサインイン画面に表示されない名前を識別します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: アカウント名の値。

**調査方法:** サービス アカウントまたはメンテナンス アカウントは意図的に非表示にすることができ、その値は、指定されたアカウントが存在すること、または管理者であることを証明するものではありません。 SAM アカウント レコードとログオン アクティビティを使用して参加します。

### RID ハイジャック {#rid_hijacking}

**ID:** `rid_hijacking` · **カタログATT&CK:**をチェック[T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei がチェックする内容:** SAM アカウント サブキーの 16 進数 RID と、その F 値のオフセット 0x30 に格納されている RID を比較し、不一致を報告します。

**調査された証拠:**

- SAM ハイブ内の `SAM\Domains\Account\Users\<hexadecimal RID>\F`。

**調査方法:** 不一致はアカウント レコードの重大な異常ですが、破損/キャプチャの分析が必要になる場合もあります。ソースバイト、トランザクションログステータス、アカウント/ログオンレコードを検査します。どの攻撃者が記録を変更したかは証明されません。

### RID サブナー (非表示の管理者アカウント) {#rid_suborner}

**ID:** `rid_suborner` · **カタログATT&CK:**をチェック[T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei のチェック内容:** サブキー RID が 500 ではないが、F 値 RID が 500 (より具体的な管理者 ID 置換) である SAM アカウントをレポートします。パターン。

**調査された証拠:**

- SAM ハイブ内の `SAM\Domains\Account\Users\<hexadecimal RID>\F`。

**調査方法:** これは、一般的な RID 不一致チェックと重複します。カタログ タイトルに関係なく、非表示のサインイン設定を個別に必要とするわけではありません。アカウントの可視性、権限、実際のアカウントの使用状況を個別に確認します。

## 認証コンポーネントと資格情報の保護 { #authentication-components-and-credential-protection }

ローカル セキュリティ機関 (LSA) は、LSASS および関連コンポーネントを通じてセキュリティ機能を処理します。認証パッケージ、パスワード フィルター、ネットワーク プロバイダーは、正規のセキュリティ ソフトウェアまたは永続的な資格情報アクセスへのルートになる可能性があります。 2 つは、実行可能ファイルの自動起動ではなく、インベントリ保護設定をチェックします。ホストの役割、Windows バージョン、および有効なポリシーを使用してそれらを解釈します。

### AMSI プロバイダー DLL {#amsi_providers}

**ID:** `amsi_providers` · **カタログATT&CK:**をチェック[T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei がチェックする内容:** マシン AMSI プロバイダー CLSID を列挙し、使用可能な場合はネイティブ/32 ビット InprocServer32 ライブラリを解決します。未解決の登録はクラス ID を保持します。

**調査された証拠:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` および解決可能な `Classes\CLSID` / `Classes\Wow6432Node\CLSID` InprocServer32 登録。

**調査方法:** ウイルス対策製品とセキュリティ製品は、AMSI プロバイダーを合法的に登録しています。プロバイダー/ライブラリの出所と実際の読み込みを確認します。登録は、AMSI がバイパスされたこと、またはプロバイダーが悪意のあるものであることを証明するものではありません。

### 認証パッケージ {#authentication_packages}

**ID:** `authentication_packages` · **カタログATT&CK:**をチェック[T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**Juntei がチェックする内容:** アクティブな制御セットから空ではない個々の LSA `Authentication Packages` エントリを読み取ります。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**調査方法:** ソース Windows ビルドおよびインストールされている認証ソフトウェアと比較します。デフォルト以外のパッケージは正規のものである可能性があります。傍受を終了する前に、DLL を解決し、読み込み/認証情報のアクティビティを確認します。

### DSRM 管理者ログオン動作 {#dsrm_backdoor}

**ID:** `dsrm_backdoor` · **カタログATT&CK:**をチェック[T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei がチェックする内容:** 実装された ControlSet001、ControlSet002、または CurrentControlSet の場所にある数値 `DsrmAdminLogonBehavior=2` を報告します。この設定により、DSRM アカウントの通常の回復コンテキスト外での使用が許可されます。

**調査された証拠:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior`、さらに ControlSet002 および CurrentControlSet (存在する場合)。

**調査方法:** 主にドメイン コントローラーに関連します。 SYSTEM Select を使用して、どのコントロール セットがアクティブであるかを決定します。過去のコントロールセットの一致が必ずしも有効であるとは限りません。この値は、攻撃者が DSRM パスワードを知っていたこと、またはログオンしていたことを証明するものではありません。

### Credential Guard の構成 {#lsa_cfg_flags}

**ID:** `lsa_cfg_flags` · **カタログATT&CK:**をチェック[T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei がチェックする内容:** 設定された LSA `LsaCfgFlags` 値を読み取ります (通常の有効な値とゼロを含みます)。これは、実行可能ファイルの自動起動ではなく、Credential Guard 構成をインベントリします。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**調査方法:** Windows のバージョン、ファームウェア/セキュリティの前提条件、および有効なポリシーを使用して値を解釈します。保存されたゼロまたは値が存在しないだけでは、攻撃者による変更、有効な保護状態、または資格情報の盗難は確立されません。

### LSA パスワードフィルター {#lsa_password_filter}

**ID:** `lsa_password_filter` · **カタログATT&CK:**をチェック[T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**Juntei がチェックする内容:** LSA `Notification Packages` を読み取ります。ベア ライブラリ名は、キャプチャされた Windows System32 ディレクトリに対して解決され、必要に応じて `.dll` が追加されます。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**調査方法:** パスワード ポリシー製品では通常、フィルターがインストールされます。ライブラリと読み込みの証拠を検査し、設定されたフィルタとパスワードの変更または資格情報の傍受が発生した証拠を区別します。

### LSASS PPL 保護ステータス {#lsa_run_as_ppl}

**ID:** `lsa_run_as_ppl` · **カタログATT&CK:**をチェック[T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei がチェックする内容:** 設定された LSA `RunAsPPL` 値 (ゼロ以外の値を含む) を読み取ります。これは保護設定のインベントリであり、LSASS を保護せずに実行したテストではありません。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**調査方法:** 効果的な LSA 保護は、Windows のバージョン、ポリシー、ファームウェアのロック、および再起動の状態によって異なります。値が存在しない行がない場合は、測定された保護結果ではありません。独立したシステム/ランタイムの証拠を使用して、効果的な保護を評価します。

### ネットワークプロバイダー DLL {#network_provider_dll}

**ID:** `network_provider_dll` · **カタログATT&CK:**をチェック[T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**Juntei がチェックする内容:** アクティブな SYSTEM コントロール セット内のサービスの下にある `NetworkProvider\ProviderPath` を読み取ります。

**調査された証拠:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`.

**調査方法:** ネットワーク ログオン プロバイダーは正規である可能性があります。プロバイダーの順序/適用性、インストールされているネットワーク/セキュリティ ソフトウェア、および実際のライブラリのロードを確認します。 ProviderPath 値だけでは、資格情報の傍受を証明することはできません。

### セキュリティパッケージ {#security_packages}

**ID:** `security_packages` · **カタログATT&CK:**をチェック[T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**Juntei がチェックする内容:** 空ではない LSA `Security Packages` エントリを読み取り、設定されたセキュリティ サポート プロバイダーをカバーします。

**調査された証拠:**

| レジストリキー | 調べた値 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**調査方法:** 予想される Windows/security-product ベースラインと比較し、各ライブラリを検査します。この構成では、プロバイダーがインシデント中にロードされ、資格情報が取得され、追加されたことは証明されません。

## 候補から結論まで取り組む { #work-from-a-candidate-to-a-conclusion }

1. `SourceFile` で識別される元のソースを開きます。完全な値、XML アクション、スクリプト、または回復されたデータベース レコードと、それが属する正確なユーザー/ホストを確認します。
2. メカニズムが動作できるかどうかを判断します。有効フラグ、ランタイムまたは Windows のバージョン、サービス/タスクの状態、必要なトリガー、ユーザー、アーキテクチャ、およびアクティブなコントロール セットを確認します。
3. 選択したプログラム、DLL、スクリプト、またはアセンブリを検査します。収集したバイトをハッシュし、引数と内容を確認します。認識可能な Windows ランチャーまたは発行元の文字列は正当性を証明しません。
4. 独立した証拠を使用して、実際に何が起こったのかを立証します。例には、プロセスの作成、Sysmon イメージのロード、タスク スケジューラのランタイム レコード、サービス アクティビティ、Prefetch、および関連ファイルが含まれます。構成変更イベントは、ペイロードの実行を証明しなくても、設定が変更されたことを証明できます。
5. オリジナルのソースのタイムスタンプと記録された相関の出所を使用してタイムラインを構築します。キーの書き込み、インストールの変更、ファイル/コンテナの書き込み、および実行を別の事実として保持します。
6. レコードの制限および検出結果: ハイブの欠落、ターゲット ファイルの欠落、不明なタスク プリンシパル、未検証の署名者のメタデータ、刻まれた履歴レコード、または裏付けできないトリガー。

コマンド構文、プロファイル、すべての出力フィールド、およびカバレッジの状態については、[Persistence Hunter コマンド ガイド](../commands/persistence-hunter.md) に戻ってください。 [アーティファクト ガイド](../artifacts/index.md)では基礎となるソースについて説明し、[acknowledgements](acknowledgements.md)ではアップストリーム永続化ツールを特定します。
