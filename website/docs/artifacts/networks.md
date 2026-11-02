# Networks

## What they are

Windows remembers the networks a system joined and how its network was set up.

- **`SOFTWARE`:**
  - **Network profiles** (`NetworkList\Profiles`): one per network the system connected to,
    with its name (a Wi-Fi network's SSID), category (public, private, domain), connection type
    (wired, wireless, VPN, mobile broadband), and when the system first and last connected to it.
  - **Signatures** (`NetworkList\Signatures`): what identifies each network, such as the
    gateway's MAC address and the DNS suffix.
  - **Network cards** (`NetworkCards`): the adapters.
- **`SYSTEM`** (per control set):
  - **TCP/IP interfaces:** addresses, DHCP server and lease, DNS servers, domain.
  - **Shares** the system offers (`LanmanServer\Shares`).
  - **Port proxies** (`PortProxy`): the tunnels `netsh interface portproxy` sets up, which
    attackers use to pivot and to reach RDP through another port.
  - **Persistent routes.**
  - **The time zone**, which turns the profiles' local times into UTC.

<span class="evidence-strength">Best conclusion:</span> the system connected to a network of
this name and gateway, first and last at these times. It had this address from this DHCP server,
and it offered these shares and tunnels when the hive was saved. A network's name alone doesn't
identify where it was: many networks are called "Network" or share an SSID.

## Location

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

Collect both hives with their transaction logs. The SYSTEM hive is needed to convert the profile
times to UTC.

## Analyst cautions

- **Profile times are local.** `DateCreated` and `DateLastConnected` are written in the system's
  local time.
  - `FirstConnectedLocal` and `LastConnectedLocal` keep them as found.
  - `FirstConnectedUTC` and `LastConnectedUTC` convert them with the time zone of the system's
    SYSTEM hive (`TimeZone`) and its daylight saving rule.
  - A time from before the time zone was changed, or from a year with another daylight saving
    rule (US dates before 2007), can be off by the difference.
  - Without a readable SYSTEM hive and a valid current control set, only local times are given. DST gaps and ambiguous repeated hours have no single proven UTC timestamp: UTC is left empty and `Problems` explains the limitation. Invalid offsets or dates also remain unconverted.
- **First and last only.** Each profile has one first and one last connection, not every
  connection. The profile key's last write usually matches the last connection.
- **Names repeat.** Windows names unmanaged wired networks "Network", "Network 2", and so on.
  Wi-Fi SSIDs are not unique. Use `GatewayMac` and `DnsSuffix` to tell networks apart, and
  correlate these with DHCP logs and asset inventory. A gateway MAC is not necessarily a Wi-Fi access point BSSID and alone does not establish physical location.
- **Shares, proxies and routes** are configuration: a share or a port proxy can be set up and
  never used. Each value has no time of its own; the key's last write is when any of them
  changed.
- **Interfaces:** an interface key under a GUID with no addresses is an adapter that isn't set
  up. With DHCP, the addresses are the last lease's. A static address left from before DHCP is
  not shown as in use.
- **Control sets:** by default the control set Windows used last is read. `--all-control-sets`
  reads every one, which can show a share or a proxy that was removed from the current one.

## Parse with Juntei

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

The summary lists the networks connected last (newest first), and every share, port proxy and
persistent route.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host, and the hive. |
| `ControlSet`, `IsCurrentControlSet` | For SYSTEM records: the control set, and whether it is the one Windows used last. |
| `Source` | `network_profile`, `network_signature` (a signature no profile has), `network_card`, `interface`, `share`, `port_proxy`, `route`. |
| `ProfileName` | A profile's name (a Wi-Fi network's SSID); a signature's description. |
| `Category`, `ConnectionType` | `public`, `private`, `domain`; `wired`, `wireless`, `vpn`, `mobile_broadband` (else the number). |
| `FirstConnectedUTC`, `LastConnectedUTC` | `DateCreated` and `DateLastConnected` converted to UTC with `TimeZone`. |
| `FirstConnectedLocal`, `LastConnectedLocal` | The same, as found (local time, to the millisecond). |
| `TimeZone` | The time zone used for the conversion. |
| `GatewayMAC`, `DNS-Suffix` | The gateway's MAC address and the DNS suffix of the profile's signatures (`|` between several). |
| `InterfaceGUID` | An interface's GUID; a network card's `ServiceName`. |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | An interface's addresses in use, DHCP server and lease. |
| `ShareName`, `SharePath` | A share and the folder it offers. |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | A port proxy: what it listens on and where it forwards to. |
| `Route` | A persistent route as stored: `destination,netmask,gateway,metric`. |
| `Details` | Other values, as `Name=Value; …` (CSV) or an object (JSON): a profile's description, `Managed`, first name and signature keys; an interface's adapter (from `NetworkCards`), masks, gateways, DNS servers and domain; a share's type, remark and other fields; a proxy's kind; a route's parts. |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | The key, the value for a share, proxy or route, and the key's last write. |
| `DataRaw` | Bytes that don't decode: a time that isn't a valid `SYSTEMTIME`, a share that isn't a list of strings. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Values of the wrong type or size, impossible dates, and names that aren't of their form. |

## Investigation strategy

1. Sort the profiles by `LastConnectedUTC` and check the networks around the incident: unknown
   Wi-Fi networks, mobile hotspots, VPNs.
2. Use `GatewayMac` and `DnsSuffix` to tell networks with the same name apart, and to match a
   network across hosts.
3. Check every port proxy and share. A port proxy to an internal host's 3389 or 445, or a share
   of a staging folder (`C:\ProgramData`, `C:\Users\Public`), is a lead. Correlate with event
   logs (`netsh` process creation, 5142/5144 share events) and the key times.
4. Use the interfaces' addresses and lease times to place the host on the network, and match them
   with firewall, DHCP and proxy logs.
5. Use `--all-control-sets` and `parse-registry read --recover-deleted` for removed shares,
   proxies and profiles.

## Further Details

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
