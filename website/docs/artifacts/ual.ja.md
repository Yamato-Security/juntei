# ユーザーアクセスログ (UAL) { #user-access-logging-ual }

## UAL とは { #what-ual-is }

ユーザーアクセスログ (UAL) は Windows Server の機能で、Windows Server 2012 以降は既定でインストールされ、有効になっています。クライアントがサーバーの役割に対して行った要求を、ESE データベースに集計して記録します。役割、クライアントのアドレス、アカウントの組み合わせごとに、その年にクライアントが最初と最後に確認された日時、要求の総数、および年内の各日の要求数を保持します。対象となる役割には、File Server (SMB)、Active Directory Domain Services、Active Directory Certificate Services、DHCP Server、Print and Document Services、Remote Access、Web Server (IIS)、Windows Server Update Services などがあります。

UAL は横展開の調査に役立ちます。クライアントの行からは、どのアドレスとアカウントがどの日にサーバーを利用したかがわかり、その記録はイベント ログが上書きされた後も長く残っていることが少なくありません。

<span class="evidence-strength">この証拠から判断できること:</span> 記録されたアドレスのクライアントが、記録されたアカウントとして認証され、要求数が記録されている各日にその役割を利用しました。

## 保存場所と収集 { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` には最新のアクティビティが格納されます。`SystemIdentity.mdb` はサーバーについて記述し、役割の名前を定め、各年のデータを保持する `{GUID}.mdb` ファイルを一覧にしています (`CHAINED_DATABASES`)。UAL サービスはデータベースを開いたままにしているため、`Sum` ディレクトリ全体をフォレンジック コレクターで、またはボリューム スナップショットから収集してください。

!!! warning "ダーティなデータベース"
    稼働中のサーバーから収集した `Current.mdb` は、ダーティな状態であることがよくあります。Juntei は収集されたままの状態で読み取り、その状態を報告しますが、`Svc*.log` は再生しません。トランザクション ログにしか存在しないアクティビティは含まれません。

## 分析時の注意点 { #analyst-cautions }

- UAL は要求を日ごとに集計するだけで、個々の要求を記録するわけではありません。クライアント行が持つ時刻は `InsertDate` と `LastAccess` だけです。
- `DailyAccesses` の日付は UTC 基準の日です。
- `Address` はサーバーから見えたアドレスです。サーバーと実際のクライアントの間に NAT ゲートウェイ、プロキシ、VPN が介在している可能性があります。`::1` と `127.0.0.1` はサーバー自身です。
- `Sum` ディレクトリのすべてのデータベースを解析してください。同じクライアントが `Current.mdb` と、その年の `{GUID}.mdb` の両方に行を持つことがあります。
- 行がないことは、何も接続しなかったことの証明にはなりません。UAL は無効にできます (`Disable-Ual`、または User Access Logging Service の停止による)。また、ダーティなデータベースには最新の変更が含まれていません。
- `SystemIdentity.mdb` がない場合、`RoleName` には Windows Server が各役割に付けている名前が使われます。
- `LastAccess` がゼロまたは不正な場合、その値から年を判断しません。有効な `InsertDate`、次に年別データベースのメタデータを使用し、年が不明な場合は `DayOfYear` を保持します。
- 集計値はすべての収集元の行を合算した値です。`Current.mdb`、年別データベース、スナップショットには重複があり得るため、重複を除いた接続数やログオン数として扱わないでください。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Current.mdb, {GUID}.mdb and SystemIdentity.mdb
  -f, --file <FILE>      Parse one UAL database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl \
  | jq 'select(.Table == "CLIENTS") | {Address, AuthenticatedUserName, RoleName, LastAccess}'
```

## 出力テーブル { #output-tables }

CSV では、テーブルごとに 1 つのファイルが作成されます。テーブルが空の場合は、ヘッダーだけのファイルになります。JSON と JSONL の各行には、`SourceFile` (データベース) と `Table` (データベース内でのテーブル名) が含まれます。

| CSV ファイル | テーブル | 内容 |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | 各役割のクライアント: アドレス、アカウント、その年の最初と最後のアクセス、要求の総数と日ごとの要求数。 |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | 各役割が最初と最後に使用された日時。 |
| `UAL-DNS.csv` | `DNS` | アドレスとホスト名の組と、それぞれが最後に確認された日時。 |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Hyper-V 仮想マシン: 識別子、シリアル番号、作成日時と最終アクティブ日時。 |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | サーバー自体: 名前、ドメイン、Windows のバージョン、製品の種類、ハードウェア、タイム ゾーン、最終起動。新しい行は、名前の変更、ドメインへの参加、ドメイン コントローラーへの昇格を示している場合があります。 |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | 各年の `{GUID}.mdb` ファイル。 |
| `UAL-Roles.csv` | `ROLE_IDS` | 役割の GUID とその名前。 |
| `UAL-<table>.csv` | その他のテーブル | Juntei が認識しないテーブルのすべての列。 |

## フィールド { #fields }

| フィールド | 解釈 |
|---|---|
| `SourceFile` | 行の読み取り元のデータベース。 |
| `RoleGUID`, `RoleName` | 役割と、データベースと同じ場所にある `SystemIdentity.mdb` から得たその名前 (ない場合は組み込みの名前)。 |
| `TenantID` | クライアントのテナントの GUID (役割がテナントを報告する場合)。 |
| `TotalAccesses` | データベースの年におけるクライアントの要求数。`DailyAccesses` の合計です。 |
| `InsertDate` | 行が作成された日時。つまり、このデータベースにおけるクライアントのその年最初の要求です (UTC)。 |
| `LastAccess` | クライアントの最新の要求 (UTC)。 |
| `Address` | クライアントの IPv4 または IPv6 アドレス。 |
| `AuthenticatedUserName` | クライアントが認証に使用したアカウント (`DOMAIN\user`。コンピューター アカウントは末尾が `$`)。 |
| `ClientName` | クライアントの名前 (役割が報告する場合)。 |
| `DailyAccesses` | 要求のあった日とその要求数。`Day1` … `Day366` 列から取得し、`LastAccess` の年の日付として表します。CSV では `2023-01-12: 5, 2023-01-13: 2`、JSON では `Date` と `Accesses` を持つオブジェクトになります。年が不明な日は、`Date` の代わりに `DayOfYear` を持ちます。 |
| `FirstSeen`, `LastSeen` | 役割の最初と最後の使用 (`ROLE_ACCESS`)、DNS の組が最後に確認された日時 (`DNS`)。いずれも UTC です。 |
| `CreationTime`, `LastSeenActive` | 仮想マシンまたは `SYSTEM_IDENTITY` 行が作成された日時と、仮想マシンが最後にアクティブだった日時 (UTC)。 |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`、2 `Domain Controller`、3 `Server`。 |
| `OS-CurrentTimeZone` | サーバーの UTC からのオフセット (分単位。`-420` は UTC-07:00)。 |
| `OS-LastBootUpTime` | サーバーの最終起動日時 (UTC)。読み取れない場合は、保存されている WMI のテキストのままです。 |
| その他の列 | テーブルのその他すべての列を、Juntei の PascalCase フィールド ラベルで出力します。例: `SystemDNS-HostName`、`SystemDomainName`、`OS-BuildNumber`、`SystemSerialNumber`。 |

## 自動解析 { #automatic-parsing }

`auto-parse` は、テーブルを `LateralMovement` に `UAL-Clients.csv`、`UAL-RoleAccess.csv` などとして書き出し (`-t json` または `-t jsonl` の場合は 1 つの `UAL.json` または `UAL.jsonl` ファイル)、あわせて `LateralMovement/UAL-logs.txt` を作成します。結合された `-j` タイムラインにはすべての行が含まれ、`Comp` と `Artifact` 識別子 `ual` が付きます。`-Z` は `InsertDate`、`LastAccess`、`FirstSeen`、`LastSeen`、`CreationTime`、`LastSeenActive`、`OS-LastBootUpTime` を変換します。

## さらに詳しい情報 { #further-details }

- [Microsoft — User Access Logging](https://learn.microsoft.com/en-us/windows-server/administration/user-access-logging/get-started-with-user-access-logging)
- [Plaso — User Access Logging parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/user_access_logging.py)
- [Velociraptor — Windows.Forensics.UserAccessLogs](https://docs.velociraptor.app/artifact_references/pages/windows.forensics.useraccesslogs/)
- [CrowdStrike — How to Leverage User Access Logging for Forensic Investigations](https://www.crowdstrike.com/en-us/blog/user-access-logging-ual-overview/)
