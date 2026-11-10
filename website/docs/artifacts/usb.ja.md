# USB デバイス { #usb-devices }

## USB の歴史とは { #what-the-usb-history-is }

Windows は、インストールされるすべてのデバイスを記録します。 USB ストレージ、他の USB デバイス、ポータブル デバイス (電話、カメラ、メディア プレーヤー) の場合、`SYSTEM` ハイブは、名前、シリアル番号、Windows がインストールした時間、最後に到着した時間、最後に削除された時間を含むデバイスごとのキーを保持します。 `MountedDevices` は、ボリュームに与えられたボリューム GUID とドライブ文字を記録します。 `SOFTWARE` ハイブには、Explorer がポータブル デバイスとボリュームに付けた名前と、ReadyBoost のボリューム シリアル番号の記録が保存されます。各ユーザーの `NTUSER.DAT` は、ユーザーがマウントしたボリュームを記録します。

`parse-registry usb` は、それらをデバイスおよびボリュームごとに 1 つのレコードに結合します。つまり、どのデバイスであったか、いつインストールされ接続されたか、どのドライブ文字とボリュームを持っていたか、どのユーザーがそのボリュームをマウントしていたかなどです。

<span class="evidence-strength">この証拠から判断できること:</span> このベンダー、製品、シリアル番号のデバイスがシステムに接続され、初回・最終の接続時刻が記録されています。そのボリュームにはこのドライブ文字が付与され、これらのユーザーがマウントしていました。レジストリの記録から、コピーしたファイルは分かりません。

## 場所 { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| デバイスのプロパティ | フィールド | 意味 (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: ドライバーが最後にインストールされた日 |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: デバイスが最初にインストールされたとき |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: 最後に接続した日時 (Windows 8 以降) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: 最後に削除された日時 (Windows 8 以降) |

Windows 8 以降は、毎回 `0064` … `0067` キーのデフォルト値として保持されます。 Windows 7 は `00000064\00000000\Data` に保持されます。 Juntei は両方を読み取ります。

多くの DFIR リファレンスでは、`0064` を最初のインストール、`0065` をインストールとラベル付けしています。 Juntei は、上記の Microsoft の `devpkey.h` 定義に従い、毎回の記録に基づいて列に名前を付けます。

`-f` または `-d` に `SYSTEM` ハイブを与えます。システムの `SOFTWARE` ハイブとそのユーザーの `NTUSER.DAT` ハイブは、コレクション (`Windows\System32\config\SOFTWARE`、`Users\<user>\NTUSER.DAT`) に保持されるため、その隣に見つかります。または、ハイブが一緒にエクスポートされた場合は同じディレクトリの 1 つ下のレベルにあります。すべてのトランザクション ログを収集します。

## レコードの結合方法 { #how-the-records-are-joined }

- **Devices:** デバイス キーごとに 1 つのレコード: `usbstor`、`usb`、および `wpdbusenum` (`DeviceClass`)。 USB ストレージ デバイスは、同じコンテナ ID (それ以外の場合は同じシリアル番号) を持つ USB デバイスからベンダー ID と製品 ID を取得します。
- **Volumes:** USB ストレージ デバイスのボリュームは、データがデバイスに名前を付けるパスである `MountedDevices` の `\??\Volume{GUID}` 値です。 2 つのボリュームを持つデバイスには 2 つのレコードがあります。そのドライブ文字は、同じデータを持つ `\DosDevices\X:` 値です。
- **名とシリアル番号:** ボリューム名は、`Windows Portable Devices` および同じデバイスに名前を付けるポータブル デバイスから取得されます。ボリューム ラベルとシリアル番号は、その名前を表す `EMDMgmt` キーから取得されます。
- **Users:** `MountPoints2` がボリュームの GUID を持ち、そのキーの最終書き込み時刻を持つユーザー。
- **デバイス キーのないボリューム** (`volume`): デバイス キー名のない `MountedDevices` ボリューム (内部ディスク、または `Enum` キーがなくなった削除されたデバイス)、ドライブ文字なしのボリューム GUID 値がデータを共有する、ボリュームのみがユーザーのもの`MountPoints2`があります。
- **デバイスのみ `SOFTWARE` 名** (`portable_device`、`readyboost`)。

`SourceKeys` は、レコードの結合元のすべてのキーと値をそのハイブとともにリストします。

## 分析時の注意点 { #analyst-cautions }

- **ドライブ文字は再利用されます。** `MountedDevices`は、各文字が与えられた最後のデバイスを保持します。文字は、その文字のデータがまだ残っているデバイスにのみ表示されます。
- **シリアル番号は作成できます。** インスタンスの 2 番目の文字が `&` の場合、デバイスがシリアル番号を報告しないため、Windows が作成します。その場合、`Serial` は空になります。このようなインスタンスは、別のコンピューターでは異なる場合があります。
- **USB ストレージ インスタンスは、論理ユニット** (`&0`) で終わります。 `Serial` は省略します。 `Instance` はそれを保持します。
- **Times は Windows のバージョンによって異なります。** Windows 7 にはインストール時間のみがあります。 `LastArrivalUTC` と `LastRemovalUTC` は最後の接続のみであり、すべての接続ではありません。イベント ログ (`Microsoft-Windows-Partition/Diagnostic` 1006、Kernel-PnP、`DriverFrameworks-UserMode`) および `setupapi.dev.log` にはさらに多くの情報が含まれています。
- **コントロール セット:** デフォルトでは、最後に使用されたコントロール セット Windows が読み取られます。 `--all-control-sets` はすべてのコントロール セットを読み取ります。古いものでも、現在のものに存在しないデバイスを一覧表示できます。
- **A キーの最終書き込み時刻は接続時刻ではありません。** Windows はさまざまな理由でデバイス キーを書き換えます。
- **削除されたデバイスは痕跡を残します。** ベンダーと製品があるがデバイス キーがないボリューム レコードは、`Enum` キーが (アンインストールまたはクリーニング ツールによって) 削除されたデバイスで、そのボリュームには `MountedDevices` がまだ残っています。
- **ReadyBoost シリアル番号** はボリュームのシリアル番号 (`dir` では `VolumeSerials`) であり、LNK ファイルおよび Jump Lists にも記録されます。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

この概要には、最後に到着した USB ストレージ デバイス (新しいものから順)、そのドライブ文字、およびデバイス キーのない USB ストレージ デバイスに属するボリュームの数がリストされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host` | コンピュータの名前 (`SYSTEM`)。 |
| `DeviceClass` | `usbstor`、`usb`、`wpdbusenum`、またはデバイス キーなしの場合は、`volume`、`portable_device`、または `readyboost`。 |
| `ControlSet` | デバイスキーのコントロールセット。 |
| `Vendor`, `Product`, `Revision` | USB ストレージ: デバイスの名前 (`Disk&Ven_…&Prod_…&Rev_…`) から、デバイスの場合はボリュームまたは `SOFTWARE` キーの名前も付けられます。 |
| `Vid`, `PID` | USB ベンダー ID と製品 ID。 |
| `Serial`, `Instance` | シリアル番号と、記載されているインスタンス キーの名前。 |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | 書かれたとおりのデバイス キーの値 (`DeviceDesc` は多くの場合 `@<inf>,%…%;<text>`)。ポータブル デバイスの `FriendlyName` は通常、そのボリュームのラベルまたはドライブです。 |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | デバイスのプロパティの時間 (上記を参照)。 |
| `DriveLetters` | `MountedDevices` データがボリュームのドライブ文字。 |
| `VolumeGUID`, `VolumeData` | ボリュームの GUID、およびボリュームが存在する `MountedDevices` データによると、デバイス パス、MBR ディスク署名とパーティション オフセット、または GPT パーティション。 |
| `VolumeNames`, `VolumeSerials` | Explorer は、デバイスまたはそのボリューム、ReadyBoost ラベル、およびボリューム シリアル番号 (`1234-ABCD`) に付けた名前を付けます。 |
| `User` | `MountPoints2` にボリュームがあり、そのキーが最後に書き込まれたユーザー (JSON: `User`、`MountPoints2LastWritten`、`SourcePath`)。 |
| `FoundIn`, `SourceKeys` | 見つかった場所 (`Enum`、`MountedDevices`、`Windows Portable Devices`、`EMDMgmt`、`MountPoints2`)、および結合元のすべてのキーと値。 |
| `KeyPath`, `KeyLastWrittenUTC` | デバイスのインスタンス キーとその最後の書き込み。 |
| `SourcePath`, `SoftwarePath` | `SYSTEM` ハイブと、何かを与えたときの `SOFTWARE` ハイブ。 |
| `TransactionLogStatus`, `LogsApplied` | レコードが結合されたハイブのレジストリ回復コンテキスト。 |
| `Problems` | 読み取れなかったキー、および FILETIME ではないデバイス プロパティ (そのバイトを含む)。 |

## 調査戦略 { #investigation-strategy }

1. USB ストレージ デバイスを `LastArrivalUTC` (それ以外の場合は `DriverLastInstalledUTC`) 順に並べ替え、コンピュータ独自のリーダーとハブを脇に置きます。
2. 各デバイスのベンダー、製品、シリアル番号、ボリューム GUID、ドライブ文字、およびボリューム シリアル番号をメモし、LNK ファイル、Jump Lists、ShellBags、および `$MFT` (そのドライブ文字またはボリューム上のパス) でそれらを探します。
3. `User` を確認して、誰がボリュームをマウントしたか、いつ (`MountPoints2` キーの最後の書き込み) をマウントしたかを確認します。
4. USB ストレージ デバイス (キーが削除されたデバイス) を指定する `volume` レコードを探します。
5. 最後の接続だけでなく、すべての接続のイベント ログと関連付けます。
6. 削除されたデバイス キーには `parse-registry read --recover-deleted` を使用します。

## さらに詳しい情報 { #further-details }

- [Microsoft: デバイスのプロパティ (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: マウント マネージャー リクエストのサポート](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (Wine のヘッダーにあるとおり)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): デバイスのプロパティ番号
- [libyal winreg-kb: USB ストレージ](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md)、[マウントされたデバイス](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) および [マウント ポイント](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)

ユーザー結合では各ハイブのソース ID が使用されるため、同じプロファイル名を持つエクスポートされたハイブは分離されたままになります。 `SourceKeys` は、寄与するユーザー ハイブ パスを保持します。コンパニオン ユーザーの検出では、再帰的なハイブの検出と一致して、シンボリック リンクがスキップされます。 VID/PID 相関には、一致する使用可能なコンテナー ID が必要です。これがなければ、シリアル照合により明確な ID が生成されなければなりません。対立する候補者は依然として不明な問題を抱えている。
