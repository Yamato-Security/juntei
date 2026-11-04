# नेटवर्क { #networks }

## ये क्या हैं { #what-they-are }

Windows उन networks को याद रखता है जिनसे system जुड़ा था, और यह भी कि उसका network कैसे सेट किया गया था।

- **`SOFTWARE`:**
  - **Network profiles** (`NetworkList\Profiles`): system जिस भी network से जुड़ा, उसके लिए एक,
    जिसमें उसका नाम (Wi-Fi network का SSID), category (public, private, domain), connection type
    (wired, wireless, VPN, mobile broadband), और system पहली और अंतिम बार उससे कब जुड़ा, यह होता है।
  - **Signatures** (`NetworkList\Signatures`): प्रत्येक network की पहचान करने वाली जानकारी, जैसे
    gateway का MAC address और DNS suffix।
  - **Network cards** (`NetworkCards`): adapters।
- **`SYSTEM`** (प्रत्येक control set के लिए):
  - **TCP/IP interfaces:** addresses, DHCP server और lease, DNS servers, domain।
  - System द्वारा उपलब्ध कराए गए **Shares** (`LanmanServer\Shares`)।
  - **Port proxies** (`PortProxy`): `netsh interface portproxy` द्वारा सेट किए गए tunnels, जिनका
    उपयोग attackers pivot करने और किसी अन्य port के माध्यम से RDP तक पहुँचने के लिए करते हैं।
  - **Persistent routes।**
  - **Time zone**, जो profiles के local times को UTC में बदलता है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> system इस नाम और gateway वाले network से
जुड़ा था, पहली और अंतिम बार इन समयों पर। उसके पास इस DHCP server से यह address था,
और hive सहेजे जाने के समय वह ये shares और tunnels उपलब्ध करा रहा था। केवल network का नाम यह
पहचान नहीं करता कि वह कहाँ था: कई networks का नाम "Network" होता है या वे एक ही SSID साझा करते हैं।

## स्थान { #location }

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

दोनों hives को उनके transaction logs के साथ संग्रहित करें। Profile times को UTC में बदलने के लिए SYSTEM hive
आवश्यक है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **Profile times local होते हैं।** `DateCreated` और `DateLastConnected` system के
  local time में लिखे जाते हैं।
  - `FirstConnectedLocal` और `LastConnectedLocal` इन्हें जैसा मिला वैसा रखते हैं।
  - `FirstConnectedUTC` और `LastConnectedUTC` इन्हें system के SYSTEM hive के time zone (`TimeZone`)
    और उसके daylight saving नियम से बदलते हैं।
  - Time zone बदलने से पहले का समय, या किसी अन्य daylight saving नियम वाले वर्ष का समय
    (2007 से पहले की US तिथियाँ), उस अंतर जितना गलत हो सकता है।
  - पढ़ने योग्य SYSTEM hive और मान्य current control set के बिना, केवल local times दिए जाते हैं। DST gaps और अस्पष्ट दोहराए गए घंटों का कोई एकल सिद्ध UTC timestamp नहीं होता: UTC खाली छोड़ा जाता है और `Problems` सीमा को स्पष्ट करता है। अमान्य offsets या तिथियाँ भी बिना बदले रहती हैं।
- **केवल पहला और अंतिम।** प्रत्येक profile में एक पहला और एक अंतिम connection होता है, हर
  connection नहीं। Profile key का अंतिम write आमतौर पर अंतिम connection से मेल खाता है।
- **नाम दोहराए जाते हैं।** Windows unmanaged wired networks का नाम "Network", "Network 2" आदि रखता है।
  Wi-Fi SSIDs अद्वितीय नहीं होते। Networks में भेद करने के लिए `GatewayMac` और `DnsSuffix` का उपयोग करें, और
  इन्हें DHCP logs और asset inventory से correlate करें। Gateway MAC आवश्यक रूप से Wi-Fi access point का BSSID नहीं होता और अकेले भौतिक स्थान स्थापित नहीं करता।
- **Shares, proxies और routes** configuration हैं: कोई share या port proxy सेट किया जा सकता है और
  कभी उपयोग न किया जाए। प्रत्येक value का अपना कोई समय नहीं होता; key का अंतिम write वह समय है जब इनमें से कोई भी
  बदला।
- **Interfaces:** किसी GUID के अंतर्गत बिना addresses वाली interface key ऐसा adapter है जो सेट
  नहीं है। DHCP के साथ, addresses अंतिम lease के होते हैं। DHCP से पहले से बचा हुआ static address
  उपयोग में के रूप में नहीं दिखाया जाता।
- **Control sets:** default रूप से वह control set पढ़ा जाता है जिसे Windows ने अंतिम बार उपयोग किया था। `--all-control-sets`
  हर control set पढ़ता है, जिससे ऐसा share या proxy दिख सकता है जिसे current control set से हटा दिया गया था।

## Juntei से parse करें { #parse-with-juntei }

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

Summary अंतिम बार जुड़े networks (सबसे नए पहले), और हर share, port proxy तथा
persistent route को सूचीबद्ध करता है।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `SourcePath` | Host, और hive। |
| `ControlSet`, `IsCurrentControlSet` | SYSTEM records के लिए: control set, और क्या यह वही है जिसे Windows ने अंतिम बार उपयोग किया था। |
| `Source` | `network_profile`, `network_signature` (ऐसा signature जो किसी profile में नहीं है), `network_card`, `interface`, `share`, `port_proxy`, `route`। |
| `ProfileName` | Profile का नाम (Wi-Fi network का SSID); signature का description। |
| `Category`, `ConnectionType` | `public`, `private`, `domain`; `wired`, `wireless`, `vpn`, `mobile_broadband` (अन्यथा संख्या)। |
| `FirstConnectedUTC`, `LastConnectedUTC` | `TimeZone` से UTC में बदले गए `DateCreated` और `DateLastConnected`। |
| `FirstConnectedLocal`, `LastConnectedLocal` | वही, जैसा मिला (local time, millisecond तक)। |
| `TimeZone` | Conversion के लिए उपयोग किया गया time zone। |
| `GatewayMAC`, `DNS-Suffix` | Profile के signatures का gateway MAC address और DNS suffix (कई होने पर बीच में `|`)। |
| `InterfaceGUID` | Interface का GUID; network card का `ServiceName`। |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | Interface के उपयोग में addresses, DHCP server और lease। |
| `ShareName`, `SharePath` | Share और वह folder जो वह उपलब्ध कराता है। |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | Port proxy: वह किस पर listen करता है और कहाँ forward करता है। |
| `Route` | जैसा संग्रहीत है वैसा persistent route: `destination,netmask,gateway,metric`। |
| `Details` | अन्य values, `Name=Value; …` (CSV) या एक object (JSON) के रूप में: profile का description, `Managed`, पहला नाम और signature keys; interface का adapter (`NetworkCards` से), masks, gateways, DNS servers और domain; share का type, remark और अन्य fields; proxy का प्रकार; route के भाग। |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | Key, share, proxy या route के लिए value, और key का अंतिम write। |
| `DataRaw` | Decode न होने वाले bytes: ऐसा समय जो मान्य `SYSTEMTIME` नहीं है, ऐसा share जो strings की list नहीं है। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का संदर्भ। |
| `Problems` | गलत type या size की values, असंभव तिथियाँ, और ऐसे नाम जो अपने अपेक्षित रूप में नहीं हैं। |

## जाँच की रणनीति { #investigation-strategy }

1. Profiles को `LastConnectedUTC` के अनुसार sort करें और incident के आसपास के networks जाँचें: अज्ञात
   Wi-Fi networks, mobile hotspots, VPNs।
2. समान नाम वाले networks में भेद करने, और hosts के बीच किसी network का मिलान करने के लिए
   `GatewayMac` और `DnsSuffix` का उपयोग करें।
3. हर port proxy और share जाँचें। किसी internal host के 3389 या 445 तक port proxy, या किसी
   staging folder (`C:\ProgramData`, `C:\Users\Public`) का share एक सुराग है। Event
   logs (`netsh` process creation, 5142/5144 share events) और key times से correlate करें।
4. Network पर host की स्थिति तय करने के लिए interfaces के addresses और lease times का उपयोग करें, और उनका मिलान
   firewall, DHCP और proxy logs से करें।
5. हटाए गए shares, proxies और profiles के लिए `--all-control-sets` और `parse-registry read --recover-deleted`
   का उपयोग करें।

## अधिक जानकारी { #further-details }

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
