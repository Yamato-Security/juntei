# Netzwerke { #networks }

## Was sie sind { #what-they-are }

Windows speichert, mit welchen Netzwerken sich ein System verbunden hat und wie sein Netzwerk eingerichtet war.

- **`SOFTWARE`:**
  - **Netzwerkprofile** (`NetworkList\Profiles`): eines pro Netzwerk, mit dem sich das System verbunden hat,
    mit seinem Namen (der SSID eines WLANs), seiner Kategorie (öffentlich, privat, Domäne), dem Verbindungstyp
    (kabelgebunden, drahtlos, VPN, mobiles Breitband) sowie dem Zeitpunkt der ersten und letzten Verbindung des Systems.
  - **Signaturen** (`NetworkList\Signatures`): was jedes Netzwerk identifiziert, etwa die
    MAC-Adresse des Gateways und das DNS-Suffix.
  - **Netzwerkkarten** (`NetworkCards`): die Adapter.
- **`SYSTEM`** (pro Steuerungssatz):
  - **TCP/IP-Schnittstellen:** Adressen, DHCP-Server und Lease, DNS-Server, Domäne.
  - **Freigaben**, die das System anbietet (`LanmanServer\Shares`).
  - **Portproxys** (`PortProxy`): die von `netsh interface portproxy` eingerichteten Tunnel, die
    Angreifer zum Pivoting und zum Erreichen von RDP über einen anderen Port nutzen.
  - **Persistente Routen.**
  - **Die Zeitzone**, mit der die lokalen Zeiten der Profile in UTC umgerechnet werden.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Das System hat sich mit einem Netzwerk
dieses Namens und Gateways verbunden, zum ersten und letzten Mal zu diesen Zeitpunkten. Es hatte diese Adresse von diesem DHCP-Server
und bot beim Speichern des Hives diese Freigaben und Tunnel an. Der Name eines Netzwerks allein
identifiziert nicht, wo es sich befand: Viele Netzwerke heißen „Network“ oder teilen sich eine SSID.

## Speicherort { #location }

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

Sichern Sie beide Hives mit ihren Transaktionsprotokollen. Der SYSTEM-Hive wird benötigt, um die Profilzeiten
in UTC umzurechnen.

## Hinweise für die Analyse { #analyst-cautions }

- **Profilzeiten sind lokal.** `DateCreated` und `DateLastConnected` werden in der lokalen Zeit des Systems
  geschrieben.
  - `FirstConnectedLocal` und `LastConnectedLocal` behalten sie so bei, wie sie vorgefunden wurden.
  - `FirstConnectedUTC` und `LastConnectedUTC` rechnen sie mit der Zeitzone aus dem SYSTEM-Hive des Systems
    (`TimeZone`) und dessen Sommerzeitregel um.
  - Ein Zeitpunkt aus der Zeit vor einer Änderung der Zeitzone oder aus einem Jahr mit einer anderen
    Sommerzeitregel (US-Daten vor 2007) kann um die Differenz abweichen.
  - Ohne lesbaren SYSTEM-Hive und gültigen aktuellen Steuerungssatz werden nur lokale Zeiten ausgegeben. Für Sommerzeitlücken und mehrdeutige, doppelt vorkommende Stunden gibt es keinen eindeutig belegten UTC-Zeitstempel: UTC bleibt leer, und `Problems` erläutert die Einschränkung. Ungültige Offsets oder Datumsangaben bleiben ebenfalls unkonvertiert.
- **Nur erste und letzte Verbindung.** Jedes Profil hat genau eine erste und eine letzte Verbindung, nicht jede
  Verbindung. Der letzte Schreibzugriff auf den Profilschlüssel entspricht meist der letzten Verbindung.
- **Namen wiederholen sich.** Windows benennt nicht verwaltete kabelgebundene Netzwerke „Network“, „Network 2“ usw.
  WLAN-SSIDs sind nicht eindeutig. Verwenden Sie `GatewayMac` und `DnsSuffix`, um Netzwerke zu unterscheiden, und
  korrelieren Sie diese mit DHCP-Protokollen und dem Asset-Inventar. Eine Gateway-MAC ist nicht zwangsläufig die BSSID eines WLAN-Access-Points und belegt allein keinen physischen Standort.
- **Freigaben, Proxys und Routen** sind Konfiguration: Eine Freigabe oder ein Portproxy kann eingerichtet und
  nie verwendet worden sein. Die einzelnen Werte haben keinen eigenen Zeitstempel; der letzte Schreibzugriff auf den Schlüssel gibt an, wann sich einer von ihnen
  geändert hat.
- **Schnittstellen:** Ein Schnittstellenschlüssel unter einer GUID ohne Adressen ist ein Adapter, der nicht eingerichtet
  ist. Bei DHCP stammen die Adressen aus der letzten Lease. Eine aus der Zeit vor DHCP verbliebene statische Adresse wird
  nicht als verwendet angezeigt.
- **Steuerungssätze:** Standardmäßig wird der Steuerungssatz gelesen, den Windows zuletzt verwendet hat. `--all-control-sets`
  liest alle, wodurch eine Freigabe oder ein Proxy sichtbar werden kann, der aus dem aktuellen entfernt wurde.

## Mit Juntei parsen { #parse-with-juntei }

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

Die Zusammenfassung listet die zuletzt verbundenen Netzwerke (neueste zuerst) sowie jede Freigabe, jeden Portproxy und jede
persistente Route auf.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host und der Hive. |
| `ControlSet`, `IsCurrentControlSet` | Für SYSTEM-Datensätze: der Steuerungssatz und ob es derjenige ist, den Windows zuletzt verwendet hat. |
| `Source` | `network_profile`, `network_signature` (eine Signatur, zu der kein Profil gehört), `network_card`, `interface`, `share`, `port_proxy`, `route`. |
| `ProfileName` | Der Name eines Profils (die SSID eines WLANs); die Beschreibung einer Signatur. |
| `Category`, `ConnectionType` | `public`, `private`, `domain`; `wired`, `wireless`, `vpn`, `mobile_broadband` (andernfalls die Nummer). |
| `FirstConnectedUTC`, `LastConnectedUTC` | `DateCreated` und `DateLastConnected`, mit `TimeZone` in UTC umgerechnet. |
| `FirstConnectedLocal`, `LastConnectedLocal` | Dieselben Werte, wie vorgefunden (lokale Zeit, auf die Millisekunde). |
| `TimeZone` | Die für die Umrechnung verwendete Zeitzone. |
| `GatewayMAC`, `DNS-Suffix` | Die MAC-Adresse des Gateways und das DNS-Suffix der Signaturen des Profils (`|` zwischen mehreren). |
| `InterfaceGUID` | Die GUID einer Schnittstelle; der `ServiceName` einer Netzwerkkarte. |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | Die verwendeten Adressen, der DHCP-Server und die Lease einer Schnittstelle. |
| `ShareName`, `SharePath` | Eine Freigabe und der Ordner, den sie anbietet. |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | Ein Portproxy: worauf er lauscht und wohin er weiterleitet. |
| `Route` | Eine persistente Route, wie gespeichert: `destination,netmask,gateway,metric`. |
| `Details` | Weitere Werte, als `Name=Value; …` (CSV) oder als Objekt (JSON): die Beschreibung eines Profils, `Managed`, der erste Name und die Signaturschlüssel; der Adapter einer Schnittstelle (aus `NetworkCards`), Masken, Gateways, DNS-Server und Domäne; Typ, Anmerkung und weitere Felder einer Freigabe; die Art eines Proxys; die Bestandteile einer Route. |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | Der Schlüssel, der Wert für eine Freigabe, einen Proxy oder eine Route und der letzte Schreibzugriff auf den Schlüssel. |
| `DataRaw` | Bytes, die sich nicht dekodieren lassen: ein Zeitpunkt, der keine gültige `SYSTEMTIME` ist, eine Freigabe, die keine Liste von Zeichenfolgen ist. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Werte mit falschem Typ oder falscher Größe, unmögliche Datumsangaben und Namen, die nicht ihrer Form entsprechen. |

## Untersuchungsstrategie { #investigation-strategy }

1. Sortieren Sie die Profile nach `LastConnectedUTC` und prüfen Sie die Netzwerke im zeitlichen Umfeld des Vorfalls: unbekannte
   WLANs, mobile Hotspots, VPNs.
2. Verwenden Sie `GatewayMac` und `DnsSuffix`, um gleichnamige Netzwerke zu unterscheiden und ein
   Netzwerk über mehrere Hosts hinweg zuzuordnen.
3. Prüfen Sie jeden Portproxy und jede Freigabe. Ein Portproxy auf Port 3389 oder 445 eines internen Hosts oder eine Freigabe
   eines Staging-Ordners (`C:\ProgramData`, `C:\Users\Public`) ist ein Ansatzpunkt. Korrelieren Sie mit den Ereignisprotokollen
   (Prozesserstellung von `netsh`, Freigabeereignisse 5142/5144) und den Schlüsselzeitstempeln.
4. Verorten Sie den Host anhand der Adressen und Lease-Zeiten der Schnittstellen im Netzwerk, und gleichen Sie diese
   mit Firewall-, DHCP- und Proxy-Protokollen ab.
5. Verwenden Sie `--all-control-sets` und `parse-registry read --recover-deleted` für entfernte Freigaben,
   Proxys und Profile.

## Weitere Informationen { #further-details }

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
