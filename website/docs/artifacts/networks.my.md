# ကွန်ရက်များ { #networks }

## ၎င်းတို့ ဆိုသည်မှာ { #what-they-are }

Windows သည် စနစ်ချိတ်ဆက်ခဲ့သော ကွန်ရက်များနှင့် ၎င်း၏ ကွန်ရက်ကို မည်သို့ ပြင်ဆင်ထားခဲ့သည်ကို မှတ်သားထားပါသည်။

- **`SOFTWARE`:**
  - **ကွန်ရက် profile များ** (`NetworkList\Profiles`): စနစ်ချိတ်ဆက်ခဲ့သော ကွန်ရက်တစ်ခုလျှင် တစ်ခု ရှိပြီး
    ၎င်း၏ အမည် (Wi-Fi ကွန်ရက်၏ SSID)၊ category (public, private, domain)၊ ချိတ်ဆက်မှု အမျိုးအစား
    (wired, wireless, VPN, mobile broadband) နှင့် စနစ်က ၎င်းကို ပထမဆုံးနှင့် နောက်ဆုံး ချိတ်ဆက်ခဲ့သည့် အချိန်တို့ ပါဝင်ပါသည်။
  - **Signature များ** (`NetworkList\Signatures`): gateway ၏ MAC address နှင့် DNS suffix ကဲ့သို့
    ကွန်ရက်တစ်ခုစီကို ခွဲခြားသတ်မှတ်ပေးသည့် အရာများ။
  - **ကွန်ရက်ကတ်များ** (`NetworkCards`): adapter များ။
- **`SYSTEM`** (control set တစ်ခုချင်းစီအလိုက်):
  - **TCP/IP interface များ:** address များ၊ DHCP server နှင့် lease၊ DNS server များ၊ domain။
  - စနစ်က ပေးထားသော **Share များ** (`LanmanServer\Shares`)။
  - **Port proxy များ** (`PortProxy`): `netsh interface portproxy` က တည်ဆောက်သော tunnel များဖြစ်ပြီး
    တိုက်ခိုက်သူများက pivot လုပ်ရန်နှင့် အခြား port မှတစ်ဆင့် RDP သို့ ရောက်ရှိရန် အသုံးပြုပါသည်။
  - **Persistent route များ။**
  - Profile များ၏ local အချိန်များကို UTC အဖြစ် ပြောင်းပေးသော **time zone**။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> စနစ်သည် ဤအမည်နှင့် gateway ရှိသော ကွန်ရက်တစ်ခုသို့
ဤအချိန်များတွင် ပထမဆုံးနှင့် နောက်ဆုံး ချိတ်ဆက်ခဲ့ပါသည်။ hive ကို သိမ်းဆည်းခဲ့သည့်အချိန်တွင် ၎င်းသည် ဤ DHCP server
ထံမှ ဤ address ကို ရရှိထားပြီး ဤ share များနှင့် tunnel များကို ပေးထားခဲ့ပါသည်။ ကွန်ရက်၏ အမည်တစ်ခုတည်းဖြင့် ၎င်းသည်
မည်သည့်နေရာတွင် ရှိခဲ့သည်ကို မခွဲခြားနိုင်ပါ။ ကွန်ရက်များစွာကို "Network" ဟု အမည်ပေးထားခြင်း သို့မဟုတ် SSID တူညီခြင်း
ရှိသောကြောင့် ဖြစ်ပါသည်။

## တည်နေရာ { #location }

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

Hive နှစ်ခုလုံးကို ၎င်းတို့၏ transaction log များနှင့်အတူ စုဆောင်းပါ။ Profile အချိန်များကို UTC သို့ ပြောင်းရန်
SYSTEM hive လိုအပ်ပါသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **Profile အချိန်များသည် local ဖြစ်ပါသည်။** `DateCreated` နှင့် `DateLastConnected` ကို စနစ်၏
  local အချိန်ဖြင့် ရေးသားထားပါသည်။
  - `FirstConnectedLocal` နှင့် `LastConnectedLocal` သည် ၎င်းတို့ကို တွေ့ရှိသည့်အတိုင်း ထိန်းထားပါသည်။
  - `FirstConnectedUTC` နှင့် `LastConnectedUTC` သည် ၎င်းတို့ကို စနစ်၏ SYSTEM hive ၏ time zone (`TimeZone`)
    နှင့် ၎င်း၏ daylight saving စည်းမျဉ်းဖြင့် ပြောင်းလဲပါသည်။
  - Time zone မပြောင်းမီက အချိန် သို့မဟုတ် daylight saving စည်းမျဉ်း ကွဲပြားသော နှစ်မှ အချိန် (2007 မတိုင်မီ
    US ရက်စွဲများ) သည် ထိုကွာခြားချက်အတိုင်း လွဲနိုင်ပါသည်။
  - ဖတ်ရှုနိုင်သော SYSTEM hive နှင့် မှန်ကန်သော current control set မရှိလျှင် local အချိန်များကိုသာ ပေးပါသည်။ DST
    ကွာဟချက်များနှင့် နှစ်ကြိမ်ထပ်သော မရေရာသည့် နာရီများတွင် သက်သေပြနိုင်သော UTC timestamp တစ်ခုတည်း မရှိပါ။ UTC ကို
    ဗလာထားပြီး ကန့်သတ်ချက်ကို `Problems` က ရှင်းပြပါသည်။ မမှန်ကန်သော offset များ သို့မဟုတ် ရက်စွဲများကိုလည်း မပြောင်းလဲဘဲ ထားပါသည်။
- **ပထမဆုံးနှင့် နောက်ဆုံးသာ။** Profile တစ်ခုစီတွင် ပထမဆုံးနှင့် နောက်ဆုံး ချိတ်ဆက်မှု တစ်ခုစီသာ ရှိပြီး
  ချိတ်ဆက်မှုတိုင်း မဟုတ်ပါ။ Profile key ၏ နောက်ဆုံးရေးသားချိန်သည် များသောအားဖြင့် နောက်ဆုံးချိတ်ဆက်မှုနှင့် ကိုက်ညီပါသည်။
- **အမည်များ ထပ်နေပါသည်။** Windows သည် manage မလုပ်ထားသော wired ကွန်ရက်များကို "Network"၊ "Network 2" စသည်ဖြင့်
  အမည်ပေးပါသည်။ Wi-Fi SSID များသည် တစ်မူထူးခြားခြင်း မရှိပါ။ ကွန်ရက်များကို ခွဲခြားရန် `GatewayMac` နှင့် `DnsSuffix` ကို
  အသုံးပြုပြီး ၎င်းတို့ကို DHCP log များနှင့် asset inventory တို့နှင့် ဆက်စပ်စစ်ဆေးပါ။ Gateway MAC သည် Wi-Fi access point
  ၏ BSSID ဖြစ်ချင်မှ ဖြစ်ပြီး ၎င်းတစ်ခုတည်းဖြင့် ရုပ်ပိုင်းဆိုင်ရာ တည်နေရာကို အတည်မပြုနိုင်ပါ။
- **Share များ၊ proxy များနှင့် route များ** သည် ပြင်ဆင်မှုများ ဖြစ်ပါသည်။ share သို့မဟုတ် port proxy တစ်ခုကို
  တည်ဆောက်ပြီး တစ်ခါမျှ အသုံးမပြုဘဲ ရှိနိုင်ပါသည်။ တန်ဖိုးတစ်ခုစီတွင် ကိုယ်ပိုင်အချိန် မရှိပါ။ key ၏ နောက်ဆုံးရေးသားချိန်သည်
  ၎င်းတို့ထဲမှ တစ်ခုခု ပြောင်းလဲခဲ့သည့် အချိန် ဖြစ်ပါသည်။
- **Interface များ:** address မရှိသော GUID အောက်ရှိ interface key သည် ပြင်ဆင်မထားသော adapter ဖြစ်ပါသည်။
  DHCP ဖြင့်ဆိုလျှင် address များသည် နောက်ဆုံး lease ၏ address များ ဖြစ်ပါသည်။ DHCP မတိုင်မီကမှ ကျန်ရှိနေသော
  static address ကို အသုံးပြုဆဲအဖြစ် မပြသပါ။
- **Control set များ:** မူလအားဖြင့် Windows နောက်ဆုံးအသုံးပြုခဲ့သော control set ကို ဖတ်ပါသည်။ `--all-control-sets`
  သည် အားလုံးကို ဖတ်ပြီး လက်ရှိ control set မှ ဖယ်ရှားထားသော share သို့မဟုတ် proxy ကို ပြသနိုင်ပါသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

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

အနှစ်ချုပ်တွင် နောက်ဆုံး ချိတ်ဆက်ခဲ့သော ကွန်ရက်များ (အသစ်ဆုံးမှ စ၍) နှင့် share၊ port proxy နှင့်
persistent route တိုင်းကို ဖော်ပြပါသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `Host`, `SourcePath` | Host နှင့် hive။ |
| `ControlSet`, `IsCurrentControlSet` | SYSTEM record များအတွက်: control set နှင့် ၎င်းသည် Windows နောက်ဆုံးအသုံးပြုခဲ့သော control set ဟုတ်/မဟုတ်။ |
| `Source` | `network_profile`, `network_signature` (မည်သည့် profile တွင်မျှ မပါသော signature), `network_card`, `interface`, `share`, `port_proxy`, `route`။ |
| `ProfileName` | Profile ၏ အမည် (Wi-Fi ကွန်ရက်၏ SSID)၊ signature ၏ ဖော်ပြချက်။ |
| `Category`, `ConnectionType` | `public`, `private`, `domain`; `wired`, `wireless`, `vpn`, `mobile_broadband` (မဟုတ်လျှင် နံပါတ်)။ |
| `FirstConnectedUTC`, `LastConnectedUTC` | `TimeZone` ဖြင့် UTC သို့ ပြောင်းထားသော `DateCreated` နှင့် `DateLastConnected`။ |
| `FirstConnectedLocal`, `LastConnectedLocal` | အထက်ပါအတိုင်း၊ တွေ့ရှိသည့်အတိုင်း (local အချိန်၊ millisecond အထိ)။ |
| `TimeZone` | ပြောင်းလဲရာတွင် အသုံးပြုသော time zone။ |
| `GatewayMAC`, `DNS-Suffix` | Profile ၏ signature များ၏ gateway MAC address နှင့် DNS suffix (အများအပြားဆိုလျှင် ကြားတွင် `|`)။ |
| `InterfaceGUID` | Interface ၏ GUID၊ ကွန်ရက်ကတ်၏ `ServiceName`။ |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | Interface ၏ အသုံးပြုဆဲ address များ၊ DHCP server နှင့် lease။ |
| `ShareName`, `SharePath` | Share တစ်ခုနှင့် ၎င်းပေးထားသော ဖိုင်တွဲ။ |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | Port proxy: ၎င်း listen လုပ်သည့်နေရာနှင့် forward လုပ်သည့်နေရာ။ |
| `Route` | သိမ်းဆည်းထားသည့်အတိုင်း persistent route: `destination,netmask,gateway,metric`။ |
| `Details` | အခြားတန်ဖိုးများ၊ `Name=Value; …` (CSV) သို့မဟုတ် object (JSON) အဖြစ်: profile ၏ ဖော်ပြချက်၊ `Managed`၊ ပထမအမည်နှင့် signature key များ၊ interface ၏ adapter (`NetworkCards` မှ)၊ mask များ၊ gateway များ၊ DNS server များနှင့် domain၊ share ၏ type၊ remark နှင့် အခြား field များ၊ proxy ၏ အမျိုးအစား၊ route ၏ အစိတ်အပိုင်းများ။ |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | Key၊ share၊ proxy သို့မဟုတ် route အတွက် တန်ဖိုးနှင့် key ၏ နောက်ဆုံးရေးသားချိန်။ |
| `DataRaw` | Decode မလုပ်နိုင်သော byte များ: မှန်ကန်သော `SYSTEMTIME` မဟုတ်သည့် အချိန်၊ string list မဟုတ်သည့် share။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု context။ |
| `Problems` | Type သို့မဟုတ် size မှားသော တန်ဖိုးများ၊ မဖြစ်နိုင်သော ရက်စွဲများနှင့် ၎င်းတို့၏ ပုံစံနှင့် မကိုက်ညီသော အမည်များ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. Profile များကို `LastConnectedUTC` ဖြင့် စီပြီး ဖြစ်စဉ်ပတ်ဝန်းကျင်ရှိ ကွန်ရက်များကို စစ်ဆေးပါ: မသိသော
   Wi-Fi ကွန်ရက်များ၊ mobile hotspot များ၊ VPN များ။
2. အမည်တူ ကွန်ရက်များကို ခွဲခြားရန်နှင့် host များအကြား ကွန်ရက်တစ်ခုကို ကိုက်ညီစေရန် `GatewayMac` နှင့် `DnsSuffix`
   ကို အသုံးပြုပါ။
3. Port proxy နှင့် share တိုင်းကို စစ်ဆေးပါ။ အတွင်းပိုင်း host ၏ 3389 သို့မဟုတ် 445 သို့ port proxy သို့မဟုတ်
   staging ဖိုင်တွဲ (`C:\ProgramData`, `C:\Users\Public`) ၏ share သည် ခြေရာခံစရာ ဖြစ်ပါသည်။ Event log များ
   (`netsh` process creation, 5142/5144 share event များ) နှင့် key အချိန်များနှင့် ဆက်စပ်စစ်ဆေးပါ။
4. Host ကို ကွန်ရက်ပေါ်တွင် နေရာချရန် interface များ၏ address များနှင့် lease အချိန်များကို အသုံးပြုပြီး
   firewall၊ DHCP နှင့် proxy log များနှင့် ကိုက်ညီစေပါ။
5. ဖယ်ရှားထားသော share များ၊ proxy များနှင့် profile များအတွက် `--all-control-sets` နှင့်
   `parse-registry read --recover-deleted` ကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
