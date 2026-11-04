# Fernzugriff (Benutzerseite) { #remote-access-user-side }

## Was es ist { #what-it-is }

Die `NTUSER.DAT` eines Benutzers bewahrt Spuren der Remoteverbindungen, die der Benutzer von diesem Computer aus hergestellt hat.
Sie bilden die Quellseite lateraler Bewegung:

- **Remotedesktop (`mstsc`):**
  - die in den Client eingegebenen Computer, die jüngsten zuerst;
  - ein Schlüssel pro Computer, bei dem sich der Benutzer angemeldet hat, mit dem verwendeten Kontonamen und dem Fingerabdruck
    des vom Benutzer akzeptierten Serverzertifikats;
  - die Computer, denen der Benutzer die Nutzung lokaler Geräte erlaubt hat.
- **PuTTY:**
  - gespeicherte Sitzungen: Host, Port, Konto, Protokoll, Schlüsseldatei, Proxy und Portweiterleitungen;
  - die zuletzt gestarteten Sitzungen;
  - jeder vom Benutzer akzeptierte SSH-Serverschlüssel.
- **WinSCP:** gespeicherte Sitzungen und akzeptierte SSH-Serverschlüssel, in derselben Form wie bei PuTTY.
- **Verbundene Laufwerke:** jedes Laufwerk, das bei der Anmeldung wiederhergestellt werden soll, und die in
  *Netzlaufwerk verbinden* (Map Network Drive) eingegebenen Pfade.
- **Freigaben:** die vom Explorer eingebundenen Freigaben (`MountPoints2`).

Sie zeigen, wohin ein Konto als Nächstes ging und mit welchen Anmeldeinformationen, selbst wenn die eigenen Protokolle des Remote-
computers nicht mehr vorhanden sind.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Das Profil dieses Benutzers hat sich mit diesem Ziel verbunden oder
eine Verbindung dorthin gespeichert. Eine gespeicherte Sitzung oder ein eingegebener Name allein beweist nicht, dass eine
Verbindung erfolgreich war. Ein akzeptierter Hostschlüssel oder ein `Servers`-Schlüssel mit einem `UsernameHint` ist stärker:
Der Client hat den Server erreicht.

## Speicherort { #location }

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

Sichern Sie `NTUSER.DAT` mit seinen Transaktionsprotokollen für jedes Profil. Sichern Sie auch den `SOFTWARE`-Hive des Systems:
Er liefert die SID jedes Benutzers.

## Hinweise für die Analyse { #analyst-cautions }

- **Schlüsselzeiten sind Listenzeiten.** `Default`, `SshHostKeys` und `Map Network Drive MRU` enthalten Listen,
  und der letzte Schreibzugriff auf ihren Schlüssel ist der Zeitpunkt der letzten Änderung der Liste. Diese Zeit gehört zum ersten Element
  (`MruPosition` 0). Für die anderen ist sie nur eine Obergrenze. Ein Schlüssel `Servers\<host>`, ein Sitzungs-,
  `Network\<letter>`- oder Freigabeschlüssel hat seine eigene Zeit.
- **Gespeichert heißt nicht verwendet.** Eine PuTTY- oder WinSCP-Sitzung kann gespeichert werden, ohne jemals eine Verbindung herzustellen. PuTTY
  schreibt `Recent sessions` neu, wenn eine gespeicherte Sitzung gestartet wird, und fügt einen Hostschlüssel hinzu, wenn ein Server
  erstmals erreicht wird.
- **Kennwörter:**
  - WinSCP kann Kennwörter für Sitzungen, Tunnel und Proxys speichern.
  - PuTTY speichert nur das Kennwort eines Proxys, im Klartext.
  - `PasswordStored` und `StoredPasswords` in `Details` geben an, welche Werte gesetzt sind. Sie werden nie
    dekodiert oder ausgegeben.
- **WinSCP lässt Standardwerte weg.** Eine Sitzung ohne `FSProtocol` ist SFTP, und eine ohne
  `PortNumber` verwendet den Standardport des Protokolls. `Protocol` und `Port` bleiben dann leer, statt
  zu raten.
- **Fingerabdrücke von Hostschlüsseln** sind `SHA256:…` im OpenSSH-Format, wie `ssh-keygen -l`, `ssh` und PuTTY ab 0.75
  sie anzeigen. Juntei berechnet sie aus dem Schlüssel, den PuTTY und WinSCP speichern. Vergleichen Sie sie mit
  dem eigenen Schlüssel des Servers, `known_hosts`-Dateien und Protokollen, um den Server hinter einem Namen oder einer IP-
  Adresse zu identifizieren. Ein SSH-1-Schlüssel (`rsa`) hat keinen.
- **Verbundene Laufwerke:** `RemoteUsername` ist nur gesetzt, wenn das Laufwerk mit einem anderen
  Konto verbunden wurde; Windows schreibt ein `REG_DWORD` 0, wenn das eigene Konto des Benutzers verwendet wurde.
- **Freigaben:** `MountPoints2` enthält auch Schlüssel für Volumes und Laufwerksbuchstaben. Hier werden nur Netzwerkfreigaben
  (`##server#share`) gemeldet; `\\server@SSL@443\DavWWWRoot` ist eine WebDAV-Freigabe.
- **Entfernung:** Das Löschen der Listen oder die Verwendung eines portablen Clients hinterlässt hier keine Spur. Vergleichen Sie mit
  Jump Lists, Prefetch, `.rdp`-Dateien, dem RDP-Bitmap-Cache und den Protokollen des Ziels.

## Mit Juntei parsen { #parse-with-juntei }

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

Die Zusammenfassung zählt Verbindungen nach Client. Sie listet die Verbindungen auf, deren Schlüssel zuletzt geschrieben wurden,
die neuesten zuerst, sowie die Sitzungen, die ein Kennwort speichern.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, Name des Profilordners, SID des Benutzers aus der `ProfileList` des Systems (mit dem `SOFTWARE`-Hive des Systems) und der Hive. |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive` oder `network_share`. |
| `Kind` | Herkunft des Datensatzes: `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point`. |
| `Target`, `Port` | Der Computer (Hostname oder IP-Adresse) und Port oder die Freigabe (`\\server\share`). Ein Port wird nur angegeben, wo Windows oder der Client einen geschrieben hat. |
| `RemoteUsername` | Das auf dem Ziel verwendete Konto: `UsernameHint`, der `UserName` einer Sitzung (oder das `user@` aus PuTTYs `HostName`), der `UserName` eines verbundenen Laufwerks. |
| `Protocol` | `Protocol` von PuTTY (`ssh`, `telnet`, `raw`, …), `FSProtocol` von WinSCP (`sftp`, `scp`, `ftp`, `webdav`, `s3`), der Anbieter eines verbundenen Laufwerks (`Microsoft Windows Network`). |
| `MRU-Position` | Die Position in der jeweiligen Liste (0 ist die jüngste): `MRU<n>`, `MRUList`, `Recent sessions` von PuTTY. |
| `SessionName` | Name einer gespeicherten Sitzung, mit dekodiertem `%20` und anderen Escape-Sequenzen. |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | Typ eines akzeptierten SSH-Hostschlüssels (`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …), sein OpenSSH-Fingerabdruck und seine Zahlen wie gespeichert. |
| `DriveLetter` | Laufwerksbuchstabe eines verbundenen Laufwerks. |
| `PasswordStored` | Ob die Sitzung ein Kennwort speichert; wird nie dekodiert. |
| `Details` | Weitere Einstellungen als `Name=Value; …` (CSV) oder als Objekt (JSON): `CertHash` des RDP-Zertifikats, `LocalDevices`, Schlüsseldatei, Proxy, SSH-Tunnel, Portweiterleitungen, Remotebefehl und Verzeichnisse einer Sitzung, `StoredPasswords`, `Label` einer Freigabe, `ConnectionType` eines Laufwerks. |
| `KeyPath`, `ValueName` | Der Schlüssel und – bei einem Datensatz, der aus einem einzelnen Wert besteht – der Wert. |
| `KeyLastWrittenUTC` | Letzter Schreibzugriff auf den Schlüssel (siehe die Hinweise zu Listen). |
| `DataRaw` | Die Bytes des Werts (hexadezimal), wenn er keine Zeichenfolge ist oder sich nicht dekodieren lässt. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Werte des falschen Typs (mit ihren Bytes), nicht lesbare Hostschlüssel, `MRUList`-Buchstaben ohne zugehörigen Wert und Werte, die nicht aufgelistet werden. |

## Untersuchungsstrategie { #investigation-strategy }

1. Listen Sie die Ziele jedes Benutzers nach Client auf und gleichen Sie sie mit den Systemen ab, die dieser Benutzer
   erreichen sollte. Achten Sie auf administrative Freigaben (`C$`, `ADMIN$`) und IP-Adressen.
2. Notieren Sie die verwendeten Konten (`RemoteUsername`): Domänenadministratoren und lokale Konten auf anderen
   Rechnern.
3. Gleichen Sie `HostKeyFingerprint` mit den Schlüsseln der Server und `CertHash` mit den Zertifikaten der RDP-Server
   ab.
4. Korrelieren Sie mit den Anmeldeereignissen der Ziele (4624 Typ 10 und 3, 1149), Jump Lists und LNK-Dateien
   (`mstsc.exe`, `putty.exe`, von Freigaben geöffnete Dateien), Prefetch und dem RDP-Bitmap-Cache.
5. Verwenden Sie `parse-registry read --recover-deleted` für gelöschte Einträge und ältere Kopien der
   `NTUSER.DAT` des Profils (Volumeschattenkopien, Sicherungen) für geleerte Listen.

## Weitere Informationen { #further-details }

- [Microsoft: Einträge aus dem Feld „Computer“ der Remotedesktopverbindung entfernen](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [PuTTY-FAQ: Wo speichert PuTTY seine Daten?](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: Speicherung der Konfiguration](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
