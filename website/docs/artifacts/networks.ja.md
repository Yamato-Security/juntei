# ネットワーク { #networks }

## 彼らは何ですか { #what-they-are }

Windows は、システムが参加したネットワークとそのネットワークの設定方法を記憶します。

- **`SOFTWARE`:**
  - **ネットワーク プロファイル** (`NetworkList\Profiles`): システムが接続したネットワークごとに 1 つ、名前 (Wi-Fi ネットワークの SSID)、カテゴリ (パブリック、プライベート、ドメイン)、接続タイプ (有線、無線、VPN、モバイル ブロードバンド)、およびシステムが最初と最後に接続した日時が含まれます。
  - **Signatures** (`NetworkList\Signatures`): ゲートウェイの MAC アドレスや DNS サフィックスなど、各ネットワークを識別するもの。
  - **ネットワーク カード** (`NetworkCards`): アダプター。
- **`SYSTEM`** (コントロール セットごと):
  - **TCP/IP インターフェイス:** アドレス、DHCP サーバーとリース、DNS サーバー、ドメイン。
  - システムが提供する **Shares** (`LanmanServer\Shares`)。
  - **ポートプロキシ** (`PortProxy`): `netsh interface portproxy`がセットアップするトンネル。攻撃者はこれを使用してピボットし、別のポート経由でRDPに到達します。
  - **永続的なルート。**
  - **タイム ゾーン**。プロファイルの現地時間を UTC に変換します。

<span class="evidence-strength">この証拠から判断できること:</span> システムが、この名前とゲートウェイのネットワークに接続し、その初回・最終接続時刻が記録されています。ハイブの保存時点で、この DHCP サーバーからこのアドレスを取得し、これらの共有とトンネルを提供していました。「Network」という名前や同じ SSID は広く使われるため、ネットワーク名だけで場所は特定できません。

## 場所 { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\NetworkList\Profiles\{GUID}
      ProfileName, Description, Category, NameType, Managed, DateCreated, DateLastConnected
  Microsoft\Windows NT\CurrentVersion\NetworkList\Signatures\Managed|Unmanaged\<signature>
      ProfileGuid, DefaultGatewayMac, DnsSuffix, FirstNetwork
  Microsoft\Windows NT\CurrentVersion\NetworkCards\<n>          ServiceName, Description
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Services\Tcpip\Parameters\Interfaces\{GUID}                    IP, DHCP, DNS, lease times
  Services\LanmanServer\Shares                                   one value per share
  Services\PortProxy\v4tov4|v4tov6|v6tov4|v6tov6\tcp             listen = connect
  Services\Tcpip\Parameters\PersistentRoutes                     destination,mask,gateway,metric
  Control\TimeZoneInformation
```

両方のハイブとトランザクション ログを収集します。 SYSTEM ハイブは、プロファイル時間を UTC に変換するために必要です。

## 分析時の注意点 { #analyst-cautions }

- **プロファイル時間はローカル時間です。** `DateCreated` および `DateLastConnected` はシステムのローカル時間で書き込まれます。
  - `FirstConnectedLocal` と `LastConnectedLocal` は、見つかったままになります。
  - `FirstConnectedUTC` および `LastConnectedUTC` は、システムの SYSTEM ハイブ (`TimeZone`) のタイム ゾーンとその夏時間ルールで変換されます。
  - タイムゾーンが変更される前の時間、または別の夏時間ルールが適用された年（2007 より前の US 日付）からの時間は、その差分だけずれている可能性があります。
  - 読み取り可能な SYSTEM ハイブと有効な現在の制御セットがなければ、現地時間のみが与えられます。 DST ギャップとあいまいな繰り返し時間には、証明された単一の UTC タイムスタンプがありません。UTC は空のままで、`Problems` で制限が説明されます。無効なオフセットまたは日付も変換されません。
- **最初と最後のみ。** 各プロファイルには最初と最後の接続が 1 つあり、すべての接続があるわけではありません。通常、プロファイル キーの最後の書き込みは最後の接続と一致します。
- **Namesrepeat.** Windows は、管理対象外の有線ネットワークに「ネットワーク」、「ネットワーク 2」などの名前を付けます。 Wi-Fi SSID は一意ではありません。 `GatewayMac` と `DnsSuffix` を使用してネットワークを区別し、これらを DHCP ログおよび資産インベントリと関連付けます。ゲートウェイ MAC は必ずしも Wi-Fi アクセス ポイント BSSID である必要はなく、単独では物理的な位置を確立しません。
- **共有、プロキシ、およびルート** は構成です。共有またはポート プロキシはセットアップできますが、使用することはできません。それぞれの値には独自の時間がありません。キーの最後の書き込みは、キーのいずれかが変更されたときです。
- **Interfaces:** GUID の下にアドレスのないインターフェイス キーは、セットアップされていないアダプターです。 DHCP の場合、アドレスは最後のリースのものです。 DHCP より前から残っている静的アドレスは使用中として表示されません。
- **コントロール セット:** デフォルトでは、最後に使用されたコントロール セット Windows が読み取られます。 `--all-control-sets` はすべてを読み取り、現在のものから削除された共有またはプロキシを表示することができます。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and SYSTEM hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry networks -d evidence/ -o networks.csv
juntei parse-registry networks -d evidence/ -t jsonl \
  | jq 'select(.Source == "port_proxy" or .Source == "share")'
```

概要には、最後に接続されたネットワーク (新しいものから順)、すべての共有、ポート プロキシ、永続ルートがリストされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath` | 宿主と巣箱。 |
| `ControlSet`, `IsCurrentControlSet` | SYSTEM レコードの場合: コントロール セット、および Windows が最後に使用したものであるかどうか。 |
| `Source` | `network_profile`、`network_signature` (プロファイルにない署名)、`network_card`、`interface`、`share`、`port_proxy`、`route`。 |
| `ProfileName` | プロファイルの名前 (Wi-Fi ネットワークの SSID)。署名の説明。 |
| `Category`, `ConnectionType` | `public`、`private`、`domain`; `wired`、`wireless`、`vpn`、`mobile_broadband` (それ以外の場合は数字)。 |
| `FirstConnectedUTC`, `LastConnectedUTC` | `DateCreated` と `DateLastConnected` は、`TimeZone` を使用して UTC に変換されました。 |
| `FirstConnectedLocal`, `LastConnectedLocal` | 見つかったものと同じです (現地時間、ミリ秒まで)。 |
| `TimeZone` | 変換に使用されるタイムゾーン。 |
| `GatewayMAC`, `DNS-Suffix` | ゲートウェイの MAC アドレスと、プロファイル署名の DNS サフィックス (複数の間は `|`)。 |
| `InterfaceGUID` | インターフェイスの GUID。ネットワークカードの`ServiceName`。 |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | 使用中のインターフェイスのアドレス、DHCP サーバーおよびリース。 |
| `ShareName`, `SharePath` | 共有とそれが提供するフォルダー。 |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | ポート プロキシ: 何をリッスンし、どこに転送するか。 |
| `Route` | 保存された永続ルート: `destination,netmask,gateway,metric`。 |
| `Details` | `Name=Value; …` (CSV) またはオブジェクト (JSON) としてのその他の値: プロファイルの説明、`Managed`、名および署名キー。インターフェイスのアダプター (`NetworkCards` から)、マスク、ゲートウェイ、DNS サーバー、およびドメイン。共有の種類、コメント、その他のフィールド。プロキシの種類。ルートのパーツ。 |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | キー、共有、プロキシ、またはルートの値、およびキーの最後の書き込み。 |
| `DataRaw` | デコードされないバイト: 有効な `SYSTEMTIME` ではない時刻、文字列のリストではない共有。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | 間違ったタイプまたはサイズの値、ありえない日付、形式ではない名前。 |

## 調査戦略 { #investigation-strategy }

1. プロファイルを `LastConnectedUTC` で並べ替え、インシデントの周囲のネットワーク (不明な Wi-Fi ネットワーク、モバイル ホットスポット、VPN) を確認します。
2. `GatewayMac` と `DnsSuffix` を使用して、同じ名前のネットワークを区別し、ホスト間でネットワークを一致させます。
3. すべてのポート プロキシと共有を確認します。内部ホストの 3389 または 445、またはステージング フォルダーの共有 (`C:\ProgramData`、`C:\Users\Public`) へのポート プロキシがリードです。イベント ログ (`netsh` プロセス作成、5142/5144 共有イベント) およびキー時刻と関連付けます。
4. インターフェイスのアドレスとリース時間を使用してネットワーク上にホストを配置し、それらをファイアウォール、DHCP、およびプロキシ ログと照合します。
5. 削除された共有、プロキシ、プロファイルには `--all-control-sets` および `parse-registry read --recover-deleted` を使用します。

## さらに詳しい情報 { #further-details }

- [Microsoft: netsh インターフェイス portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: プロキシ: 内部プロキシ](https://attack.mitre.org/techniques/T1090/001/)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
