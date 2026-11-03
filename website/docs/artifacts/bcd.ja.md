# ブート構成データ (BCD) { #boot-configuration-data-bcd }

## BCD ストアとは { #what-the-bcd-store-is }

Windows Vista 以降では、ブート構成がレジストリ ハイブ、つまり BCD ストアに保存されます: BIOS システムでは `\Boot\BCD`、UEFI システムの EFI システム パーティションでは `\EFI\Microsoft\Boot\BCD`。 `bcdedit` および Windows インストーラーはそれを編集します。ブート マネージャーはブートのたびにそれを読み取ります。

ストアにはオブジェクトが保持されており、それぞれの名前は GUID にちなんで付けられます。

- **Windows ブート マネージャー** (`{bootmgr}`)、デフォルトのエントリ、表示順序、メニュー タイムアウト。 UEFI システムでは、**ファームウェア ブート マネージャー** (`{fwbootmgr}`) とファームウェアのブート順序も含まれます。
- Windows インストールごとに 1 つの **Windows ブート ローダー** (回復環境を含む)、**アプリケーションの再開** (休止状態)、および **メモリ テスター**。
- **継承された設定** オブジェクト (`{globalsettings}`、`{bootloadersettings}`、…) の設定が他のオブジェクトに継承されます。
- **device** オブジェクト (回復環境の RAM ディスク オプション)。

各オブジェクトには、説明、デバイスとパス、`recoveryenabled`、`bootstatuspolicy`、`testsigning`、`nointegritychecks`、`safeboot` などの設定といった要素があります。

2 種類の攻撃によって変化します。

- **Ransomware** は回復をオフにするため、システムを修復できません: `bcdedit /set {default} recoveryenabled No` および `bcdedit /set {default} bootstatuspolicy ignoreallfailures`。一部のファミリでは、Windows がセキュリティ製品が実行されないセーフ モードで再起動されるように `safeboot` を設定しています。
- **Rootkits および署名されていないドライバー** は、起動時のチェックを弱めるために `testsigning on` または `nointegritychecks on` を使用する場合があります。これらの設定だけでは、ルートキットが実行されたことを証明することはできません。

<span class="evidence-strength">この証拠から判断できること:</span> 収集時点で、このブートエントリにはこの設定がありました。設定を最後に書き込んだ時刻は、対応する要素のキーの最終書き込み時刻です。

## 場所 { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

要素のタイプは 8 16 進数です。そのクラス (`1` すべてのアプリケーション、`2` 1 種類のアプリケーション、`3` デバイス)、その形式 (`1` デバイス、`2` 文字列、`3` オブジェクト、 `4` オブジェクト リスト、`5` 整数、`6` ブール値、`7` 整数リスト) とその番号。このコマンドは、要素に Microsoft ドキュメント (`BcdLibraryBoolean_AutoRecoveryEnabled`) として名前を付けます。

多くの場合、収集ツールはストアをコピーしません。EFI システム パーティションはマウントされておらず、`\Boot` は非表示になっています。明示的に収集するか、パーティションをマウントします。 `-d` は `BCD-Template` も読み取ります (`HiveType` がどちらであるかを示します)。テンプレートは Windows の起動元であり、起動に使用するストアではありません。

## 分析時の注意点 { #analyst-cautions }

- **Settings は継承されます。** ブート ローダーは、`BcdLibraryObjectList_InheritedObjects` (通常は `{globalsettings}` を継承する `{bootloadersettings}`) 内のオブジェクトを継承します。 `RecoveryEnabled`、`TestSigning`、`NoIntegrityChecks`、`SafeBoot`、および `BootStatusPolicy` は解決された格納値 (独自の値、そうでない場合は継承された値) であり、`Inherited` は別のオブジェクトから取得された値を示します。継承は最大 8 レベルまで追跡され、サイクルは制限されます。参照されるオブジェクトが部分ストアに存在しない場合があります。空のフィールドは、有効な Windows デフォルトを確立しません。ファームウェアとセキュア ブートによって、保存されたオプションが有効にならない場合もあります。
- **未設定と無効は異なります。** `RecoveryEnabled` が空なら、その要素は存在せず、Windows の既定値が適用されます。
- **Times は要素ごとです。** 各要素のキー時刻 (`Elements` の `key_last_written`) は、最後に書き込まれた時刻です。 Windows は、更新およびアップグレード時に一部の要素を書き換えます。他の要素の時間と比較してください。
- **Devices は部分的にデコードされます。** GPT パーティションは、そのパーティションとディスクの GUID として指定されます。ラムディスクをオプションオブジェクトとして使用します。他のデバイスはバイトのみを保持します (`data_raw`)。
- **文書化されていない要素** (`0x15000065`、`0x4…` クラス) には名前がなく、バイトが保持されます。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

概要には、各ストアのデフォルト エントリ、すべてのテスト署名、整合性チェック、リカバリ、ブート ステータス ポリシー (任意の `Ignore…`)、およびセーフ モード設定が、設定された時刻または継承元のオブジェクトとともにリストされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath`, `HiveType` | ホスト、ハイブ、および `BCD` または `BCD-Template`。 |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | オブジェクトの GUID、そのタイプ (`0x10200003`)、およびオブジェクトの内容 (`Windows boot loader`、`inherited settings for Windows boot loader` など)。 |
| `Description`, `Path`, `Device` | 説明、アプリケーションのパス、およびそのデバイス (GPT パーティションとディスク、または RAM ディスクのオプション オブジェクト)。 |
| `IsDefault`, `DisplayOrder` | Windows ブート マネージャーがデフォルトでブートするかどうか、およびブート メニュー内の位置。 |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | ストアから解決された `recoveryenabled`、`testsigning`、`nointegritychecks`。 |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`、`Network`、`DsRepair`) および `bootstatuspolicy` (`IgnoreAllFailures`、...) と番号。 |
| `Inherited` | これらのうちどれが別のオブジェクトからのものであり、どれか。 |
| `Elements` | すべての要素: タイプ、名前、値、バイト数 (デバイス、文書化されていないタイプ、およびデコードされないデータの場合)、キー時間。 |
| `KeyLastWrittenUTC` | オブジェクトのキーの最後の書き込み。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | 型のないオブジェクト、データが形式に適合しない要素。 |

## 調査戦略 { #investigation-strategy }

1. `TestSigning`、`NoIntegrityChecks`、`RecoveryEnabled = false`、`Ignore…`、`BootStatusPolicy`、および `SafeBoot` のデフォルト エントリとすべてのブート ローダーを確認します。
2. 要素 time を使用して変更の日付を確認し、Prefetch、Amcache、プロセス作成イベント (4688、Sysmon 1)、およびシステム イベント ログの `bcdedit.exe` と関連付けます。
3. そこにあるべきではないブート エントリまたはデバイス (別のローダー パス、RAM ディスク)、および変更されたデフォルトまたは表示順序を探します。
4. ストアを `BCD-Template` および同じビルドの別のシステムと比較します。

## さらに詳しい情報 { #further-details }

- [Microsoft: BCD ブート オプションのリファレンス](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (オブジェクト タイプ)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
