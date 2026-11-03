# システム情報 { #system-information }

## それは何ですか { #what-it-is }

システムの 4 つのマシン ハイブは、アナリストがタイムラインを読む前に必要なことを示します。

- その名前 (NetBIOS、DNS ホスト名、DNS ドメイン) およびそのドメインまたはワークグループ。
- Windows のバージョン、ビルドおよび更新リビジョン、Windows がインストールまたは最後にアップグレードされた時期。
- タイムゾーン。システムが保持している現地時間を示します。
- 最後に正常にシャットダウンしたとき。
- プロセッサ アーキテクチャとコード ページ。
- 最後にサインインしたユーザーと、サインインしたすべてのユーザーのプロファイル。
- SID: コンピュータ (ローカル アカウントに RID を加えたもの) とそのドメイン。
- アップグレード元の Windows バージョン。

`parse-registry system-info` は、これらすべてを使用してシステムごとに 1 つのレコードを書き込みます。

<span class="evidence-strength">この証拠から判断できること:</span> ハイブを収集、または最後に書き込んだ時点でのシステム構成です。キーの時刻は、その構成の各部分を最後に変更した時刻を示します。

## 場所 { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

トランザクション ログを含む 4 つのハイブを収集します。それらのどれか一つだけでも、それが持つものを与えます。

## 分析時の注意点 { #analyst-cautions }

- **レジストリ時間は UTC です。** キー時刻、FILETIME、およびほとんどのイベント時刻は UTC です。テキスト ログ、一部のアプリケーション ログ、および名前または文字列 (`Source OS (Updated on …)` など) に書き込まれた日付はローカルである可能性があります。 `Bias` は、現地標準時から UTC (`UTC = local + Bias`) までの分です。 `UtcOffset` と `DaylightUtcOffset` は 2 つのオフセットを示し、`TimeZoneDescription` は夏時間の開始時刻と終了時刻を示します。
- **タイム ゾーンは変更される可能性があります。** `TimeZoneLastWrittenUTC` はキーの最後の書き込みです。 Windows は、`ActiveTimeBias` (その時点で有効なオフセット) を更新するときに独自にキーを書き換えるため、最近誰かがゾーンを変更したことを証明するものではありません。イベント ログと比較します: セキュリティ 4616 (変更されたシステム時刻) およびシステム ログの `Microsoft-Windows-Kernel-General` イベント。
- **`StandardStart` と `DaylightStart` はカーネルの `TIME_FIELDS` であり、`SYSTEMTIME`s ではありません。** 曜日は最後のフィールドであり、3 番目のフィールドではありません (次のゾーンの `TZI` 値のように) `SOFTWARE`）。 US ゾーンでは、平日が日曜日 (0) であるため、どちらのレイアウトも同じように読み取られます。 Juntei はそれらを `TIME_FIELDS` としてデコードします。イースター島の公共システム (第 1 土曜日の 22:00) では違いが示されています。 0 の月は、夏時間がないことを意味します。
- **Windows 11 は「Windows 10」と言っています。** `ProductName` は `Windows 10 …` を Windows 11 に保持します。 22000 以降のビルドは Windows 11 であり、概要にはそのように記載されています。 `CurrentVersion` は、Windows、8.1、10、および 11 上の `6.3` です。 `Version` は、`CurrentMajorVersionNumber` (10) が存在する場合はそれを使用します。
- **`InstallDate` は機能アップグレードに伴い移動します。** 新しいリリースへのインプレース アップグレード (1809 から 21H2 など) によりリセットされます。各アップグレードが独自の `InstallDate` に置き換えられたリリースは、`PreviousVersions` (`Setup\Source OS`) にあります。最初の `InstallDate` は、元のインストールに最も近いものです。 `Source OS` キーの名前の日付はタイムゾーンを示しません。
- **`ShutdownTime` は最後の正常なシャットダウンです。** クラッシュまたは電源喪失により更新されません。これをイベント ログ (6006、6008、1074) およびハイブへの最後の書き込み時刻と比較します。
- **`LastLoggedOnUser` はサインイン画面の最後のユーザーですが、** はすべてのログオンではありません。リモート デスクトップ、`runas`、サービス、スケジュールされたタスクすべてが設定するわけではありません。 `LogonUiLastWrittenUTC` は最後に設定されたときのものです。
- **Profiles:** `ProfileList` には、システム上にプロファイルを持っていたすべてのユーザー (およびサービス) がリストされます。プロファイルが削除されるまで、削除されたアカウントも含まれます。 Windows 10 以降は、`LocalProfileLoadTime` と `LocalProfileUnloadTime` を書き込みます: 最後のサインインとサインアウトについて。
- **SID:** ローカル アカウントは `MachineSid-<RID>`、ドメイン アカウントは `DomainSid-<RID>` です。 `MachineSid` は `SAM` から取得されます (または `SECURITY` の `PolAcDmS` から取得されます)。 2 つは一致する必要があります。そうでない場合は、別のシステムのハイブと同様に、`Problems` がそうします。 `DomainSid` はワークグループ内で空です。
- **1 つのディレクトリ、1 つのシステム。** `-d` では、1 つのディレクトリ内のハイブが 1 つのシステムになります。 `RegBack` コピーと同じ種類の 2 番目のハイブ (`SYSTEM.old`) は、それぞれ独自のレコードです。 `-f` では、指定されたハイブは 1 つのシステムのものであり、他のハイブはその隣で検索されます。
- **コントロール セット:** 最後に使用されたコントロール セット Windows のみが読み取られます。 `parse-registry read` はその他を示します。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

概要には、システムごとに、Windows バージョン、インストール日、タイム ゾーン (単語)、キーが最後に書き込まれたときに有効なオフセット、最後のシャットダウン、ドメインまたはワークグループ、最後にサインインしたユーザー、SID、プロファイルの数、およびアップグレード元のバージョンが表示されます。 `auto-parse` は、見つかったシステムをリストするときに、各システムのタイムゾーンも出力します。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | ホスト、およびハイブの読み取り (`SourcePath` は `SYSTEM` ハイブ、それ以外の場合は最初に読み取られます)。 |
| `CompName`, `Hostname`, `Domain` | NetBIOS 名 (`SYSTEM`、それ以外の場合は `SECURITY` の `PolAcDmN`)、DNS ホスト名、および DNS ドメイン (`Tcpip\Parameters`)。 |
| `PrimaryDomain`, `DNS-Domain` | コンピューターが参加しているドメインまたはそのワークグループ (`PolPrDmN`)、およびドメインの DNS 名 (`PolDnDDN`)。 |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: 表記されている製品名、`EditionID`、`DisplayVersion` (その他の場合は `ReleaseId`)、メジャー バージョンとマイナー バージョン (その他の場合は `CurrentVersion`)、ビルド、アップデートのビルド リビジョン、`CSDVersion`、および `Client`、 `Server` または `Server Core`。 |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (2 番目まで) および `InstallTime` (100 ns、Windows 10 以降): インストールまたは最後の機能アップグレード。 |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | 書いてある通り。 |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (その他の場合は `StandardName`)、およびそのオフセットと夏時間の日付を含むワードでのゾーン。 |
| `UTC-Offset`, `DaylightUTC-Offset` | 現地標準時間の UTC からのオフセット、および夏時間 (夏時間なしの空白)。 |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`、`ActiveTimeBias` (キーが書き込まれたときに有効)、`StandardBias`、`DaylightBias`。 |
| `StandardName`, `DaylightName` | 書かれているように、多くの場合、リソース参照 (`@tzres.dll,-112`) です。 |
| `StandardStart`, `DaylightStart` | 標準時間と夏時間の開始時刻、言葉で表すと (`first Sunday of November, 02:00`)。 |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | 動的夏時間はオフになります。ハードウェアクロックはUTCを維持します。 |
| `TimeZoneLastWrittenUTC` | タイムゾーンキーの最後の書き込み。 |
| `LastShutdownUTC` | `ShutdownTime`: 最後のクリーンなシャットダウン。 |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | コントロール セットは、`PROCESSOR_ARCHITECTURE`、`ACP`、および `OEMCP` を読み取りました。 |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | サインイン画面の最後のユーザーとそのキーの最後の書き込み。 |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | コンピューターの SID、そのドメインの SID (`PolPrDmS`)、およびドメイン内のコンピューター アカウントの SID (`PolMachineAccountS`)。 |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | コンピューターが最後に名前を付けられた時期、ドメインまたはワークグループに最後に参加した時期、および最後に更新された時期 (`CurrentVersion` のキー)。 |
| `PreviousVersions` | 各 `Source OS` キー: バージョン、インストール日、およびキーの名前に含まれる日付 (JSON: それぞれ 1 つのオブジェクト)。 |
| `Profiles` | 各プロファイル: SID、フォルダー、キー時刻、最後のロードおよびアンロード (JSON: それぞれ 1 つのオブジェクト)。 |
| `TransactionLogStatus`, `LogsApplied` | 4 つのハイブすべてにわたるレジストリ回復コンテキスト。 |
| `Problems` | 間違ったタイプまたはサイズの値 (バイト数は保持される)、およびコンピューターの SID が異なる `SAM` と `SECURITY`。 |

## 調査戦略 { #investigation-strategy }

1. まずタイム ゾーンを読みます。`UtcOffset`、夏時間の日付、およびゾーンが最近変更されたかどうかに注意してください。これを使用して現地時間のログを変換します。
2. バージョン、ビルド、インストール日をメモします。 `PreviousVersions` でアップグレードを確認し、インストール日を最も古いファイル システムおよびイベント ログの時刻と比較します。
3. 注 `LastShutdownUTC`: 新しいシャットダウン時間がないその後のアクティビティは、クラッシュ、電源喪失、またはライブ コレクションを示唆しています。
4. `MachineSid` および `DomainSid` を使用して、他の SID (BAM、`ProfileList`、イベント ログ) のローカル アカウントとドメイン アカウント、および最後のサインインのプロファイルのロード時間とアンロード時間を区別します。
5. コンピューター名とドメイン名をイベント ログと関連付け、名前変更とドメイン参加のために `CompNameLastWrittenUTC` および `PrimaryDomainLastWrittenUTC` を関連付けます。

## さらに詳しい情報 { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: セキュリティ識別子](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: タイムゾーン](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: Windows 製品情報](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
