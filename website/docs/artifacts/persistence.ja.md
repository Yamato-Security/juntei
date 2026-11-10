# レジストリの自動起動 { #registry-autostarts }

## 彼らは何ですか { #what-they-are }

Windows はプログラムを起動し、さまざまなレジストリの場所から DLL をロードします。起動時、ログオン時、プログラムの開始時または終了時、`cmd.exe` が開いたとき、COM クラスの作成時、またはスプーラ、LSA、netsh、またはタイム サービスがそのプラグインをロードしたときです。攻撃者はこれらの場所を永続化、権限昇格、DLL インジェクションに使用し、正規のソフトウェアもこれらの場所を使用します。 Juntei は、`SOFTWARE`、`SYSTEM`、`NTUSER.DAT`、および `UsrClass.dat` からそれらを 1 つのテーブルに読み取ります。プログラム、DLL、またはパッケージごとに 1 つのレコードを書き込み、それぞれに MITRE ATT&CK テクニックでタグ付けします。

<span class="evidence-strength">この証拠から判断できること:</span> このプログラムまたは DLL が起動・読み込みされるよう、ハイブ内に設定されていました。実行そのものを証明するものではありません。実行には、各仕組みが必要とする条件（ログオン、デバッグ対象の起動、クラスの生成など）と、実行を示す別の証拠が必要です。

## 所在地 { #locations }

| ハイブ | 何が読まれているか |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`、`RunOnce` (+`Setup`)、`RunOnceEx`、`RunServices(Once)`、および `Policies\Explorer\Run`、およびそれらの `Wow6432Node` およびターミナル サーバーのインストール モード コピー。 `Explorer\StartupApproved\Run`、`Run32`、および `StartupFolder`。 `Winlogon` (`Shell`、`Userinit`、`Taskman`、`AppSetup`、`GinaDLL`、…および `Notify`); `Windows` `AppInit_DLLs` と `LoadAppInit_DLLs`; `Image File Execution Options\*` `Debugger` (`UseFilter` 付き) および `VerifierDlls`。 `SilentProcessExit\*` `MonitorProcess`; `Active Setup\Installed Components\*` `StubPath`; `NetSh`; `Command Processor` `AutoRun` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`、`SetupExecute`、`Execute`、`PendingFileRenameOperations`、`KnownDLLs`、`AppCertDlls`、`Environment` .NET プロファイラー変数); `Control\Lsa` `Authentication`、`Security`、および `Notification Packages` (+`OSConfig`); `SecurityProviders`; `Control\Print` モニター、プロバイダー、およびプリント プロセッサー。 `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | ユーザーの `Run` キーと `StartupApproved`。 `Windows NT\CurrentVersion\Windows` `Load` および `Run`; `Winlogon` `Shell`; `Environment` (`UserInitMprLogonScript`、.NET プロファイラ); `Control Panel\Desktop` `SCRNSAVE.EXE`; `PackagedAppXDebug`; `Command Processor` `AutoRun`; `Software\Classes` (`UsrClass.dat` として) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` および `LocalServer32` (および `Wow6432Node`)。 `shell\open\command` のプログラム タイプ (`exefile`、`batfile`、`scrfile` など) とどの自動昇格プログラムが開始するか (`ms-settings`、`mscfile`、`Folder` など)。 `ActivatableClasses\…\DebugInformation` |

各ハイブとそのトランザクション ログを収集します。ユーザー ハイブは `Users\<name>\NTUSER.DAT` および `Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat` にあります。

## 分析時の注意点 { #analyst-cautions }

- **Defaults も報告されます。** クリーンなシステム上のほとんどのレコードは Windows デフォルトです。
  - `Userinit` は `userinit.exe` です。
  - `Shell` は `explorer.exe` です。
  - `KnownDLLs`、LSA パッケージ、プリント モニター、netsh ヘルパー、およびアクティブ セットアップ コンポーネントにはすべてストック エントリがあります。

  同じバージョンのクリーン ビルドとの違いを探してください。
- **`StartupApproved` は、タスク マネージャーのスタートアップ リストのみをカバーします。** そこでは、`Run` エントリまたはスタートアップ フォルダー ファイルが無効になっているかどうか、および無効になるのはいつかを示します。一致する `Run` 値がないエントリは、「StartupApproved のみ」の詳細とともに報告されます。削除された自動起動が表示される場合があります。
- **`OverridesMachineRegistration` にはシステムの `SOFTWARE` ハイブが必要です。** ユーザーの COM クラスまたはコマンドがそれに対してチェックされ、ホストの `SYSTEM` ハイブと同じ方法で検出されます。オーバーライドは COM ハイジャック パターン (T1546.015) です。 OneDrive およびその他のユーザーごとのインストールでは、何もオーバーライドしない独自のクラスも登録されます。
- **`ImagePath` と `Args` は、それがわかる場合にのみコマンド ラインから分割されます。**
  - 引用されたプログラムは、終了引用符で分割されます。
  - フォルダのない最初のワードがプログラム (`cmd /c …`) として扱われます。
  - それ以外の場合、分割はプログラムの拡張子で終了する最短の開始位置で行われます。

  スペースを含む引用符で囲まれていないパスでは、別のプログラム (`C:\Program.exe`) が存在する場合、それが実行される可能性があります。
- **`GlobalFlag` は、`VerifierDlls` と `MonitorProcess` に対して `Enabled` を決定します。** `VerifierDlls` は、`0x100` でのみロードされます。 `MonitorProcess` は、`0x200` および `SilentProcessExit\<program>` `ReportingMode` の `0x1` でのみ実行されます。 `2` (ダンプ) または `4` (通知) のモードでは何も実行されず、`Details` はモードのフラグに名前を付けます。
- **`KeyLastWrittenUTC` は、値ではなく、キーの最後の書き込み** です。いくつかの価値観がそれを共有しています。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

概要では、メカニズムごとにレコードがカウントされます。また、`Run` 値なしで残された `StartupApproved\Run` および `Run32` エントリ (スタートアップ フォルダー ファイルには失うものはありません)、およびマシンのものをオーバーライドするユーザー COM クラスとコマンドもリストされます。

すべての `parse-registry` コマンドと同様に、読み込むファイルの 1 つである `-o` は、`-C` であっても拒否します。これには、パス、シンボリック リンク、またはハード リンクごとに、ハイブ、そのトランザクション ログ、システムの SYSTEM および SOFTWARE ハイブが含まれます。伝えられないときは書きません。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath`, `HiveType` | ホスト コンテキスト、ソース ハイブ、およびそのタイプ。 |
| `User`, `UserSID` | ユーザー ハイブの場合: プロファイル フォルダー、および SID。 SID は `UsrClass.dat` のルート キー名から取得され、それ以外の場合はシステムの `ProfileList` から取得されます。 |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | MITRE ATT&CK テクニック。いずれかが当てはまる (`T1547.001`、`T1546.012`、…)。 |
| `KeyPath`, `ValueName` | ハイブのルートの下のキー (名前は書かれているとおり) と値 (空: デフォルト値)。 |
| `Data` | 値のテキスト、またはリストの 1 つの項目。読み取れない場合、またはエントリに `StartupApproved` のみの名前が付けられている場合は空です。 |
| `ImagePath`, `Args` | プログラムまたは DLL と引数。これらは区別できます (上記を参照)。 |
| `Enabled`, `DisabledAtUTC` | `StartupApproved`、`LoadAppInit_DLLs`、`GlobalFlag` (`ReportingMode` を含む)、`UseFilter`、`IsInstalled`、タイムプロバイダーの `Enabled` または `ScreenSaveActive` の内容。何も言われないときは空です。 |
| `Details` | 詳細: 保留中の名前変更の動作、COM クラスの名前、`ReportingMode`、`FilterFullPath`、アクティブ セットアップ `ShellComponent` など |
| `OverridesMachineRegistration` | ユーザーの COM クラスまたはコマンド: マシンが同じものを登録するかどうか。システムの `SOFTWARE` ハイブがなければ空です。 |
| `WOW64` | `Wow6432Node` の下: 32 ビット プログラム用。 |
| `KeyLastWrittenUTC` | キーの最後の書き込み。 |
| `DataRaw` | 値のバイト (16 進数) (テキストでない場合、または有効でない場合) UTF-16。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | 間違った値のタイプ、読み取れないデータ、間違ったサイズの `StartupApproved` エントリ。 |

## 調査戦略 { #investigation-strategy }

1. まずは要約から始めます。
   - マシンのものをオーバーライドする COM クラスとコマンド。
   - `Run` 値のない `StartupApproved` エントリ。
   - エントリがほとんどないメカニズム (`ifeo_debugger`、`silent_process_exit`、`appinit_dlls`、`appcert_dlls`、`uac_bypass_command`、`file_association`、`logon_script`、`dotnet_profiler`)。
2. LSA パッケージ、`BootExecute`、`KnownDLLs`、プリント モニター、および netsh ヘルパーを、同じ Windows バージョンのクリーン ビルドと比較します。
3. ファイル システム ($MFT、ハッシュおよび署名) に対して `ImagePath` を解決し、ユーザー プロファイル、`ProgramData` および `Temp` パスを確認します。
4. `KeyLastWrittenUTC` をセキュリティおよび Sysmon レジストリ イベント、プロセス作成、Prefetch および Amcache と関連付けます。
5. `parse-registry read --recover-deleted` を使用して、これらのキーから削除された自動起動を見つけます。

## さらに詳しい情報 { #further-details }

- [MITRE ATT&CK T1547 ブートまたはログオンの自動開始実行](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 イベントトリガーによる実行](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: 実行および RunOnce レジストリ キー](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
