# リモートアクセス (ユーザー側) { #remote-access-user-side }

## それは何ですか { #what-it-is }

ユーザーの `NTUSER.DAT` は、ユーザーがコンピュータから行ったリモート接続のトレースを保持します。これらは横展開のソース側です。

- **リモート デスクトップ (`mstsc`):**
  - クライアントに入力されたコンピュータ (最新のものから順)。
  - ユーザーがサインインしたコンピューターごとに 1 つのキー。使用されたアカウント名とユーザーが受け入れたサーバー証明書の拇印が含まれます。
  - ユーザーがローカルデバイスの使用を許可したコンピュータ。
- **PuTTY:**
  - 保存されたセッション: ホスト、ポート、アカウント、プロトコル、キー ファイル、プロキシ、およびポート転送。
  - 最後に開始されたセッション。
  - ユーザーが受け入れたすべての SSH サーバー キー。
- **WinSCP:** はセッションを保存し、PuTTY と同じ形式で SSH サーバー キーを受け入れました。
- **マップされたドライブ:** サインイン時に復元されるようにマップされた各ドライブ、および *ネットワーク ドライブのマップ* に入力されたパス。
- **Shares:** Explorer がマウントした共有 (`MountPoints2`)。

これらは、リモート コンピューター自体のログがなくなった後でも、アカウントが次にどこに移動し、どの資格情報を使用したかを示します。

<span class="evidence-strength">この証拠から判断できること:</span> このユーザーのプロファイルに、対象への接続または接続設定が記録されています。保存されたセッションや入力した名前だけでは、接続の成功は証明できません。承認済みホスト鍵、または `UsernameHint` を持つ `Servers` キーは、クライアントがサーバーに到達したという、より強い証拠です。

## 場所 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Terminal Server Client\Default            MRU0 … MRU9
Software\Microsoft\Terminal Server Client\Servers\<host>     UsernameHint, CertHash
Software\Microsoft\Terminal Server Client\LocalDevices       <host> = devices
Software\SimonTatham\PuTTY\Sessions\<session>
Software\SimonTatham\PuTTY\Jumplist                          Recent sessions
Software\SimonTatham\PuTTY\SshHostKeys                       <type>@<port>:<host> = key
Software\Martin Prikryl\WinSCP 2\Sessions\<session>
Software\Martin Prikryl\WinSCP 2\SshHostKeys
Network\<letter>                                             RemotePath, UserName, ProviderName
Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU
Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##server#share
```

プロファイルごとに `NTUSER.DAT` とそのトランザクション ログを収集します。システムの `SOFTWARE` も収集します。これにより、各ユーザーの SID が得られます。

## 分析時の注意点 { #analyst-cautions }

- **Key 時間はリスト時間です。** `Default`、`SshHostKeys`、および `Map Network Drive MRU` はリストを保持し、それらのキーの最後の書き込みはリストが最後に変更されたときです。その時間は最初の項目 (`MruPosition` 0) に属します。他のものについては、それは単なる上限にすぎません。 `Servers\<host>`、セッション、`Network\<letter>`、または共有キーには独自の時間があります。
- **Saved は使用されません。** PuTTY または WinSCP セッションは、接続しなくても保存できます。 PuTTY は、保存されたセッションが開始されるときに `Recent sessions` を書き換え、サーバーに最初に到達するときにホスト キーを追加します。
- **パスワード:**
  - WinSCP は、セッション、トンネル、プロキシのパスワードを保存できます。
  - PuTTY は、プロキシのパスワードのみをプレーン テキストで保存します。
  - `PasswordStored` および `Details`' `StoredPasswords` は、どの値が設定されているかを示します。それらは決してデコードされたり書き出されたりすることはありません。
- **WinSCP はデフォルトを省略します。** `FSProtocol` のないセッションは SFTP で、`PortNumber` のないセッションはプロトコルのデフォルト ポートを使用します。 `Protocol` と `Port` は推測ではなく空のままになります。
- **ホスト キー フィンガープリント** は、`ssh-keygen -l`、`ssh`、および PuTTY 0.75 以降で示されるように、OpenSSH の `SHA256:…` です。 Juntei は、PuTTY と WinSCP が保持するキーからそれらを計算します。これらをサーバー独自のキー、`known_hosts` ファイルおよびログと比較して、名前または IP アドレスに基づいてサーバーを特定します。 SSH-1 キー (`rsa`) には何もありません。
- **マップされたドライブ:** `RemoteUsername` は、ドライブが別のアカウントにマップされた場合にのみ設定されます。 Windows は、ユーザー独自のものを使用した場合、`REG_DWORD` 0 を書き込みます。
- **Shares:** `MountPoints2` にもボリュームとドライブ文字のキーがあります。ここではネットワーク共有 (`##server#share`) のみが報告されます。 `\\server@SSL@443\DavWWWRoot` は WebDAV 共有です。
- **Removal:** リストをクリアするか、ポータブル クライアントを使用すると、ここには痕跡が残りません。 Jump Lists、Prefetch、`.rdp` ファイル、RDP ビットマップ キャッシュ、およびターゲットのログと比較します。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry remote-access -d evidence/ -o remote.csv
juntei parse-registry remote-access -f NTUSER.DAT -t jsonl \
  | jq 'select(.Client == "rdp" or .HostKeyFingerprint != null)'
```

この概要では、クライアントごとの接続数がカウントされます。これには、キーが最後に書き込まれた接続、新しいものから順、およびパスワードを保持しているセッションがリストされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | ホスト、プロファイル フォルダーの名前、システムの `ProfileList` (システムの `SOFTWARE` ハイブを含む) からのユーザーの SID、およびハイブ。 |
| `Client` | `rdp`、`putty`、`winscp`、`mapped_drive`、または `network_share`。 |
| `Kind` | レコードの取得元: `rdp_mru`、`rdp_server`、`rdp_local_devices`、`putty_session`、`putty_host_key`、`winscp_session`、`winscp_host_key`、`network_drive`、`map_network_drive_mru`、 `mount_point`。 |
| `Target`, `Port` | コンピューター (ホスト名または IP アドレス) とポート、または共有 (`\\server\share`)。ポートは、Windows またはクライアントがポートを書き込んだ場所にのみ指定されます。 |
| `RemoteUsername` | ターゲットで使用されるアカウント: `UsernameHint`、セッションの `UserName` (または PuTTY の `HostName` の `user@`)、マップされたドライブの `UserName`。 |
| `Protocol` | PuTTY の `Protocol` (`ssh`、`telnet`、`raw`、…)、WinSCP の `FSProtocol` (`sftp`、`scp`、`ftp`、 `webdav`、`s3`)、マップされたドライブのプロバイダー (`Microsoft Windows Network`)。 |
| `MRU-Position` | リスト内の場所 (0 が最新): `MRU<n>`、`MRUList`、PuTTY の `Recent sessions`。 |
| `SessionName` | `%20` およびその他のエスケープがデコードされた、保存されたセッションの名前。 |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | 受け入れられた SSH ホスト キーのタイプ (`rsa2`、`ssh-ed25519`、`ecdsa-sha2-nistp256` など)、その OpenSSH フィンガープリント、および保存されているその番号。 |
| `DriveLetter` | マップされたドライブの文字。 |
| `PasswordStored` | セッションがパスワードを保持するかどうか。決してデコードされませんでした。 |
| `Details` | `Name=Value; …` (CSV) またはオブジェクト (JSON) としてのその他の設定: RDP 証明書の `CertHash`、`LocalDevices`、セッションのキー ファイル、プロキシ、SSH トンネル、ポート転送、リモート コマンドおよびディレクトリ、`StoredPasswords`、共有の `Label`、ドライブの `ConnectionType`。 |
| `KeyPath`, `ValueName` | キーと、1 つの値であるレコードの値。 |
| `KeyLastWrittenUTC` | キーの最後の書き込み (リストに関する注意を参照)。 |
| `DataRaw` | 値が文字列でない場合、またはデコードされない場合の値のバイト数 (16 進数)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | 間違ったタイプの値 (そのバイト数)、読み取れないホスト キー、存在しない `MRUList` 文字、およびリストにない値。 |

## 調査戦略 { #investigation-strategy }

1. 各ユーザーのターゲットをクライアントごとにリストし、ユーザーが到達すべきシステムと照らし合わせてチェックします。管理共有 (`C$`、`ADMIN$`) および IP アドレスを探します。
2. 使用されているアカウント (`RemoteUsername`) に注目してください: ドメイン管理者と他のマシンのローカル アカウント。
3. `HostKeyFingerprint` をサーバーのキーと照合し、`CertHash` を RDP サーバーの証明書と照合します。
4. ターゲットのログオン イベント (4624 タイプ 10 および 3、1149)、Jump Lists および LNK ファイル (`mstsc.exe`、`putty.exe`、共有から開かれたファイル) と関連付けます。 Prefetch および RDP ビットマップ キャッシュ。
5. 削除されたエントリには `parse-registry read --recover-deleted` を使用し、クリアされたリストにはプロファイルの `NTUSER.DAT` の古いコピー (ボリューム シャドウ コピー、バックアップ) を使用します。

## さらに詳しい情報 { #further-details }

- [Microsoft: リモート デスクトップ接続コンピュータ ボックスからエントリを削除します](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [PuTTY FAQ: PuTTY はデータをどこに保存しますか?](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: 構成ストレージ](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: ターミナル サーバー クライアント](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: マウント ポイント](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
