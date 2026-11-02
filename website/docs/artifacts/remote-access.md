# Remote Access (User Side)

## What it is

A user's `NTUSER.DAT` keeps traces of the remote connections the user made from the computer.
These are the source side of lateral movement:

- **Remote Desktop (`mstsc`):**
  - the computers typed into the client, the most recent first;
  - one key per computer the user signed in to, with the account name used and the thumbprint
    of the server certificate the user accepted;
  - the computers the user let use local devices.
- **PuTTY:**
  - saved sessions: host, port, account, protocol, key file, proxy and port forwardings;
  - the sessions started last;
  - every SSH server key the user accepted.
- **WinSCP:** saved sessions and accepted SSH server keys, in the same form as PuTTY's.
- **Mapped drives:** each drive mapped to be restored at sign-in, and the paths typed into
  *Map Network Drive*.
- **Shares:** the shares Explorer mounted (`MountPoints2`).

They show where an account went next and with which credentials, even after the remote
computer's own logs are gone.

<span class="evidence-strength">Best conclusion:</span> this user's profile connected to, or saved
a connection to, this target. A saved session or a typed name alone doesn't prove that a
connection succeeded. An accepted host key or a `Servers` key with a `UsernameHint` is stronger:
the client reached the server.

## Location

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

Collect `NTUSER.DAT` with its transaction logs for every profile. Collect the system's `SOFTWARE`
too: it gives each user's SID.

## Analyst cautions

- **Key times are list times.** `Default`, `SshHostKeys` and `Map Network Drive MRU` hold lists,
  and their key's last write is when the list last changed. That time belongs to the first item
  (`MruPosition` 0). For the others it is only an upper bound. A `Servers\<host>`, session,
  `Network\<letter>` or share key has its own time.
- **Saved isn't used.** A PuTTY or WinSCP session can be saved without ever connecting. PuTTY
  rewrites `Recent sessions` when a saved session is started, and adds a host key when a server is
  first reached.
- **Passwords:**
  - WinSCP can store session, tunnel and proxy passwords.
  - PuTTY stores only a proxy's password, in plain text.
  - `PasswordStored` and `Details`' `StoredPasswords` say which values are set. They are never
    decoded or written out.
- **WinSCP leaves out defaults.** A session without `FSProtocol` is SFTP, and one without
  `PortNumber` uses the protocol's default port. `Protocol` and `Port` stay empty then rather
  than guessing.
- **Host key fingerprints** are OpenSSH's `SHA256:…`, as `ssh-keygen -l`, `ssh` and PuTTY 0.75
  and later show them. Juntei computes them from the key PuTTY and WinSCP keep. Compare them with
  the server's own key, `known_hosts` files and logs to identify the server behind a name or IP
  address. An SSH-1 key (`rsa`) has none.
- **Mapped drives:** `RemoteUsername` is set only when the drive was mapped with another
  account; Windows writes a `REG_DWORD` 0 when it used the user's own.
- **Shares:** `MountPoints2` also has keys for volumes and drive letters. Only network shares
  (`##server#share`) are reported here; `\\server@SSL@443\DavWWWRoot` is a WebDAV share.
- **Removal:** clearing the lists, or using a portable client, leaves no trace here. Compare with
  Jump Lists, Prefetch, `.rdp` files, the RDP bitmap cache and the target's logs.

## Parse with Juntei

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

The summary counts connections by client. It lists the connections whose keys were written last,
newest first, and the sessions that keep a password.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `Username`, `UserSID`, `SourcePath` | Host, the profile folder's name, the user's SID from the system's `ProfileList` (with the system's `SOFTWARE` hive), and the hive. |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive` or `network_share`. |
| `Kind` | Where the record comes from: `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point`. |
| `Target`, `Port` | The computer (host name or IP address) and port, or the share (`\\server\share`). A port is given only where Windows or the client wrote one. |
| `RemoteUsername` | The account used on the target: `UsernameHint`, a session's `UserName` (or the `user@` of PuTTY's `HostName`), a mapped drive's `UserName`. |
| `Protocol` | PuTTY's `Protocol` (`ssh`, `telnet`, `raw`, …), WinSCP's `FSProtocol` (`sftp`, `scp`, `ftp`, `webdav`, `s3`), a mapped drive's provider (`Microsoft Windows Network`). |
| `MRU-Position` | The place in its list (0 is the most recent): `MRU<n>`, `MRUList`, PuTTY's `Recent sessions`. |
| `SessionName` | A saved session's name, with `%20` and other escapes decoded. |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | An accepted SSH host key's type (`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …), its OpenSSH fingerprint, and its numbers as stored. |
| `DriveLetter` | A mapped drive's letter. |
| `PasswordStored` | Whether the session keeps a password; never decoded. |
| `Details` | Other settings, as `Name=Value; …` (CSV) or an object (JSON): the RDP certificate's `CertHash`, `LocalDevices`, a session's key file, proxy, SSH tunnel, port forwardings, remote command and directories, `StoredPasswords`, a share's `Label`, a drive's `ConnectionType`. |
| `KeyPath`, `ValueName` | The key, and the value for a record that is one value. |
| `KeyLastWrittenUTC` | The key's last write (see the cautions for lists). |
| `DataRaw` | The value's bytes (hex), when it isn't a string or doesn't decode. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Values of the wrong type (with their bytes), host keys that can't be read, `MRUList` letters that aren't there, and values it doesn't list. |

## Investigation strategy

1. List each user's targets by client, and check them against the systems that user should
   reach. Look for administrative shares (`C$`, `ADMIN$`) and IP addresses.
2. Note the accounts used (`RemoteUsername`): domain administrators and local accounts on other
   machines.
3. Match `HostKeyFingerprint` with the servers' keys, and `CertHash` with the RDP servers'
   certificates.
4. Correlate with the targets' logon events (4624 type 10 and 3, 1149), Jump Lists and LNK files
   (`mstsc.exe`, `putty.exe`, files opened from shares), Prefetch and the RDP bitmap cache.
5. Use `parse-registry read --recover-deleted` for deleted entries, and older copies of the
   profile's `NTUSER.DAT` (Volume Shadow Copies, backups) for cleared lists.

## Further Details

- [Microsoft: Remove entries from the Remote Desktop Connection Computer box](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [PuTTY FAQ: Where does PuTTY store its data?](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: Configuration storage](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
