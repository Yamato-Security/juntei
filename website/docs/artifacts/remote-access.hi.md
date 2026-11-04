# रिमोट एक्सेस (उपयोगकर्ता पक्ष) { #remote-access-user-side }

## यह क्या है { #what-it-is }

किसी उपयोगकर्ता का `NTUSER.DAT` उन remote connections के निशान रखता है जो उपयोगकर्ता ने उस computer से किए।
ये lateral movement का स्रोत पक्ष हैं:

- **Remote Desktop (`mstsc`):**
  - client में टाइप किए गए computers, सबसे हाल वाला पहले;
  - हर उस computer के लिए एक key जिसमें उपयोगकर्ता ने sign in किया, उपयोग किए गए account नाम और
    उपयोगकर्ता द्वारा स्वीकार किए गए server certificate के thumbprint के साथ;
  - वे computers जिन्हें उपयोगकर्ता ने local devices उपयोग करने दिए।
- **PuTTY:**
  - सहेजे गए sessions: host, port, account, protocol, key फ़ाइल, proxy और port forwardings;
  - सबसे हाल में शुरू किए गए sessions;
  - हर SSH server key जिसे उपयोगकर्ता ने स्वीकार किया।
- **WinSCP:** सहेजे गए sessions और स्वीकार की गई SSH server keys, PuTTY जैसे ही रूप में।
- **Mapped drives:** sign-in पर restore होने के लिए map किया गया हर drive, और
  *Map Network Drive* में टाइप किए गए paths।
- **Shares:** वे shares जिन्हें Explorer ने mount किया (`MountPoints2`)।

ये दिखाते हैं कि कोई account आगे कहाँ गया और किन credentials के साथ, तब भी जब remote
computer के अपने logs मिट चुके हों।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> इस उपयोगकर्ता की profile ने इस target से connect किया, या
उससे connection सहेजा। केवल सहेजा गया session या टाइप किया गया नाम यह सिद्ध नहीं करता कि
connection सफल हुआ। स्वीकार की गई host key या `UsernameHint` वाली `Servers` key अधिक मज़बूत है:
client server तक पहुँचा।

## स्थान { #location }

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

हर profile के लिए `NTUSER.DAT` को उसके transaction logs के साथ एकत्र करें। सिस्टम का `SOFTWARE`
भी एकत्र करें: यह हर उपयोगकर्ता का SID देता है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **Key times list times हैं।** `Default`, `SshHostKeys` और `Map Network Drive MRU` lists रखते हैं,
  और उनकी key का last write वह समय है जब list अंतिम बार बदली। वह समय पहले item
  (`MruPosition` 0) का है। अन्य के लिए यह केवल ऊपरी सीमा है। `Servers\<host>`, session,
  `Network\<letter>` या share key का अपना समय होता है।
- **सहेजा गया उपयोग किया गया नहीं है।** PuTTY या WinSCP session बिना कभी connect किए सहेजा जा सकता है। PuTTY
  सहेजा गया session शुरू होने पर `Recent sessions` फिर से लिखता है, और किसी server तक पहली बार पहुँचने पर
  host key जोड़ता है।
- **Passwords:**
  - WinSCP session, tunnel और proxy passwords संग्रहीत कर सकता है।
  - PuTTY केवल proxy का password संग्रहीत करता है, plain text में।
  - `PasswordStored` और `Details` का `StoredPasswords` बताते हैं कि कौन-सी values set हैं। इन्हें कभी
    decode या लिखा नहीं जाता।
- **WinSCP defaults छोड़ देता है।** `FSProtocol` के बिना session SFTP है, और
  `PortNumber` के बिना session protocol का default port उपयोग करता है। तब `Protocol` और `Port` अनुमान लगाने के बजाय
  खाली रहते हैं।
- **Host key fingerprints** OpenSSH के `SHA256:…` हैं, जैसे `ssh-keygen -l`, `ssh` और PuTTY 0.75
  और बाद के versions उन्हें दिखाते हैं। Juntei इन्हें उस key से गणना करता है जिसे PuTTY और WinSCP रखते हैं। किसी नाम या IP
  address के पीछे के server की पहचान के लिए इनकी तुलना server की अपनी key, `known_hosts` फ़ाइलों और logs से करें।
  SSH-1 key (`rsa`) का कोई fingerprint नहीं होता।
- **Mapped drives:** `RemoteUsername` केवल तब set होता है जब drive किसी अन्य
  account से map किया गया हो; जब उपयोगकर्ता का अपना account उपयोग हुआ हो तो Windows `REG_DWORD` 0 लिखता है।
- **Shares:** `MountPoints2` में volumes और drive letters के लिए भी keys होती हैं। यहाँ केवल network shares
  (`##server#share`) report किए जाते हैं; `\\server@SSL@443\DavWWWRoot` एक WebDAV share है।
- **हटाना:** lists साफ़ करना, या portable client का उपयोग करना, यहाँ कोई निशान नहीं छोड़ता। इसकी तुलना
  Jump Lists, Prefetch, `.rdp` फ़ाइलों, RDP bitmap cache और target के logs से करें।

## Juntei से parse करें { #parse-with-juntei }

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

Summary client के अनुसार connections गिनता है। यह उन connections को सूचीबद्ध करता है जिनकी keys सबसे अंत में लिखी गईं,
सबसे नए पहले, और वे sessions जो password रखते हैं।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, profile फ़ोल्डर का नाम, सिस्टम के `ProfileList` से उपयोगकर्ता का SID (सिस्टम के `SOFTWARE` hive के साथ), और hive। |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive` या `network_share`। |
| `Kind` | Record कहाँ से आता है: `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point`। |
| `Target`, `Port` | Computer (host नाम या IP address) और port, या share (`\\server\share`)। Port केवल वहीं दिया जाता है जहाँ Windows या client ने उसे लिखा। |
| `RemoteUsername` | Target पर उपयोग किया गया account: `UsernameHint`, किसी session का `UserName` (या PuTTY के `HostName` का `user@`), किसी mapped drive का `UserName`। |
| `Protocol` | PuTTY का `Protocol` (`ssh`, `telnet`, `raw`, …), WinSCP का `FSProtocol` (`sftp`, `scp`, `ftp`, `webdav`, `s3`), किसी mapped drive का provider (`Microsoft Windows Network`)। |
| `MRU-Position` | अपनी list में स्थान (0 सबसे हाल का): `MRU<n>`, `MRUList`, PuTTY का `Recent sessions`। |
| `SessionName` | सहेजे गए session का नाम, `%20` और अन्य escapes decode किए हुए। |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | स्वीकार की गई SSH host key का type (`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …), उसका OpenSSH fingerprint, और संग्रहीत रूप में उसकी संख्याएँ। |
| `DriveLetter` | Mapped drive का letter। |
| `PasswordStored` | क्या session password रखता है; कभी decode नहीं किया जाता। |
| `Details` | अन्य settings, `Name=Value; …` (CSV) या object (JSON) के रूप में: RDP certificate का `CertHash`, `LocalDevices`, किसी session की key फ़ाइल, proxy, SSH tunnel, port forwardings, remote command और directories, `StoredPasswords`, किसी share का `Label`, किसी drive का `ConnectionType`। |
| `KeyPath`, `ValueName` | Key, और उस record के लिए value जो एक value है। |
| `KeyLastWrittenUTC` | Key का last write (lists के लिए सावधानियाँ देखें)। |
| `DataRaw` | Value के bytes (hex), जब वह string न हो या decode न हो। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का context। |
| `Problems` | गलत type की values (उनके bytes के साथ), ऐसी host keys जिन्हें पढ़ा नहीं जा सकता, `MRUList` के ऐसे letters जो मौजूद नहीं हैं, और ऐसी values जिन्हें यह सूचीबद्ध नहीं करता। |

## जाँच की रणनीति { #investigation-strategy }

1. हर उपयोगकर्ता के targets को client के अनुसार सूचीबद्ध करें, और उनकी जाँच उन systems से करें जिन तक उस उपयोगकर्ता को
   पहुँचना चाहिए। Administrative shares (`C$`, `ADMIN$`) और IP addresses खोजें।
2. उपयोग किए गए accounts (`RemoteUsername`) नोट करें: domain administrators और अन्य
   machines पर local accounts।
3. `HostKeyFingerprint` का मिलान servers की keys से, और `CertHash` का मिलान RDP servers के
   certificates से करें।
4. Targets के logon events (4624 type 10 और 3, 1149), Jump Lists और LNK फ़ाइलों
   (`mstsc.exe`, `putty.exe`, shares से खोली गई फ़ाइलें), Prefetch और RDP bitmap cache से correlate करें।
5. हटाई गई entries के लिए `parse-registry read --recover-deleted` का उपयोग करें, और साफ़ की गई lists के लिए
   profile के `NTUSER.DAT` की पुरानी copies (Volume Shadow Copies, backups) का।

## अधिक जानकारी { #further-details }

- [Microsoft: Remove entries from the Remote Desktop Connection Computer box](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [PuTTY FAQ: Where does PuTTY store its data?](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: Configuration storage](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
