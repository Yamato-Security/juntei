# အဝေးမှ ဝင်ရောက်မှု (အသုံးပြုသူဘက်) { #remote-access-user-side }

## ၎င်း ဆိုသည်မှာ { #what-it-is }

အသုံးပြုသူ၏ `NTUSER.DAT` သည် အသုံးပြုသူက ကွန်ပျူတာမှ ပြုလုပ်ခဲ့သော အဝေးမှ ချိတ်ဆက်မှုများ၏ ခြေရာများကို ထိန်းသိမ်းထားသည်။
၎င်းတို့သည် ကွန်ရက်အတွင်း ဘေးတိုက်ရွေ့လျားမှု (lateral movement) ၏ ရင်းမြစ်ဘက် ဖြစ်သည်-

- **Remote Desktop (`mstsc`):**
  - client တွင် ရိုက်ထည့်ခဲ့သော ကွန်ပျူတာများ၊ နောက်ဆုံးကို ပထမ၊
  - အသုံးပြုသူ sign in ဝင်ခဲ့သော ကွန်ပျူတာတစ်လုံးလျှင် key တစ်ခု၊ အသုံးပြုခဲ့သော အကောင့်အမည်နှင့် အသုံးပြုသူက
    လက်ခံခဲ့သော server certificate ၏ thumbprint နှင့်အတူ၊
  - အသုံးပြုသူက local device များကို အသုံးပြုခွင့်ပေးခဲ့သော ကွန်ပျူတာများ။
- **PuTTY:**
  - သိမ်းဆည်းထားသော session များ- host၊ port၊ အကောင့်၊ protocol၊ key ဖိုင်၊ proxy နှင့် port forwarding များ၊
  - နောက်ဆုံးစတင်ခဲ့သော session များ၊
  - အသုံးပြုသူက လက်ခံခဲ့သော SSH server key တိုင်း။
- **WinSCP:** သိမ်းဆည်းထားသော session များနှင့် လက်ခံခဲ့သော SSH server key များ၊ PuTTY ၏ ပုံစံအတိုင်း။
- **Map လုပ်ထားသော drive များ:** sign-in တွင် ပြန်လည်ချိတ်ဆက်ရန် map လုပ်ထားသော drive တစ်ခုစီနှင့်
  *Map Network Drive* တွင် ရိုက်ထည့်ခဲ့သော path များ။
- **Share များ:** Explorer က mount လုပ်ခဲ့သော share များ (`MountPoints2`)။

၎င်းတို့သည် အဝေးရှိ ကွန်ပျူတာ၏ ကိုယ်ပိုင် log များ ပျောက်သွားပြီးနောက်တွင်ပင် အကောင့်တစ်ခု နောက်တစ်ဆင့် မည်သည့်နေရာသို့
သွားခဲ့သည်နှင့် မည်သည့် credential များဖြင့် သွားခဲ့သည်ကို ပြသသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ဤအသုံးပြုသူ၏ profile သည် ဤပစ်မှတ်သို့ ချိတ်ဆက်ခဲ့သည်
သို့မဟုတ် ချိတ်ဆက်မှုတစ်ခုကို သိမ်းဆည်းခဲ့သည်။ သိမ်းဆည်းထားသော session သို့မဟုတ် ရိုက်ထည့်ထားသော အမည်တစ်ခုတည်းဖြင့်
ချိတ်ဆက်မှု အောင်မြင်ခဲ့ကြောင်း သက်သေမပြပါ။ လက်ခံထားသော host key သို့မဟုတ် `UsernameHint` ပါသော `Servers` key သည်
ပိုခိုင်မာသည်- client က server သို့ ရောက်ရှိခဲ့သည်။

## တည်နေရာ { #location }

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

Profile တိုင်းအတွက် `NTUSER.DAT` ကို ၎င်း၏ transaction log များနှင့်အတူ စုဆောင်းပါ။ စနစ်၏ `SOFTWARE` ကိုလည်း
စုဆောင်းပါ- ၎င်းက အသုံးပြုသူတစ်ဦးစီ၏ SID ကို ပေးသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **Key အချိန်များသည် စာရင်း အချိန်များ ဖြစ်သည်။** `Default`, `SshHostKeys` နှင့် `Map Network Drive MRU` တွင်
  စာရင်းများ ပါဝင်ပြီး ၎င်းတို့၏ key ၏ နောက်ဆုံးရေးသားချိန်သည် စာရင်း နောက်ဆုံးပြောင်းလဲခဲ့သည့်အချိန် ဖြစ်သည်။ ထိုအချိန်သည်
  ပထမ item (`MruPosition` 0) နှင့် သက်ဆိုင်သည်။ အခြား item များအတွက် ၎င်းသည် အပေါ်ဘက် ကန့်သတ်ချက်သာ ဖြစ်သည်။
  `Servers\<host>`၊ session၊ `Network\<letter>` သို့မဟုတ် share key တွင် ၎င်း၏ ကိုယ်ပိုင်အချိန် ရှိသည်။
- **သိမ်းဆည်းထားခြင်းသည် အသုံးပြုခဲ့ခြင်း မဟုတ်ပါ။** PuTTY သို့မဟုတ် WinSCP session ကို တစ်ကြိမ်မျှ မချိတ်ဆက်ဘဲ
  သိမ်းဆည်းနိုင်သည်။ PuTTY သည် သိမ်းဆည်းထားသော session တစ်ခု စတင်သည့်အခါ `Recent sessions` ကို ပြန်ရေးပြီး server
  တစ်ခုသို့ ပထမဆုံး ရောက်ရှိသည့်အခါ host key တစ်ခု ထပ်ထည့်သည်။
- **Password များ-**
  - WinSCP သည် session၊ tunnel နှင့် proxy password များကို သိမ်းဆည်းနိုင်သည်။
  - PuTTY သည် proxy ၏ password ကိုသာ plain text ဖြင့် သိမ်းဆည်းသည်။
  - `PasswordStored` နှင့် `Details` ၏ `StoredPasswords` က မည်သည့်တန်ဖိုးများ သတ်မှတ်ထားသည်ကို ဖော်ပြသည်။ ၎င်းတို့ကို
    ဘယ်သောအခါမျှ decode မလုပ်ပါ၊ မရေးထုတ်ပါ။
- **WinSCP သည် ပုံသေတန်ဖိုးများကို ချန်လှပ်ထားသည်။** `FSProtocol` မပါသော session သည် SFTP ဖြစ်ပြီး `PortNumber`
  မပါသော session သည် protocol ၏ ပုံသေ port ကို အသုံးပြုသည်။ ထိုအခါ `Protocol` နှင့် `Port` ကို ခန့်မှန်းခြင်းမပြုဘဲ
  ဗလာထားသည်။
- **Host key fingerprint များ** သည် `ssh-keygen -l`၊ `ssh` နှင့် PuTTY 0.75 နှင့် နောက်ပိုင်းက ပြသသည့်အတိုင်း OpenSSH ၏
  `SHA256:…` ဖြစ်သည်။ Juntei သည် ၎င်းတို့ကို PuTTY နှင့် WinSCP သိမ်းဆည်းထားသော key မှ တွက်ချက်သည်။ အမည် သို့မဟုတ် IP
  လိပ်စာ၏ နောက်ကွယ်ရှိ server ကို ခွဲခြားသတ်မှတ်ရန် ၎င်းတို့ကို server ၏ ကိုယ်ပိုင် key၊ `known_hosts` ဖိုင်များနှင့် log
  များနှင့် နှိုင်းယှဉ်ပါ။ SSH-1 key (`rsa`) တွင် fingerprint မရှိပါ။
- **Map လုပ်ထားသော drive များ-** `RemoteUsername` ကို drive ကို အခြားအကောင့်ဖြင့် map လုပ်ခဲ့သည့်အခါသာ သတ်မှတ်သည်။
  အသုံးပြုသူ၏ ကိုယ်ပိုင်အကောင့်ကို အသုံးပြုခဲ့သည့်အခါ Windows က `REG_DWORD` 0 ကို ရေးသည်။
- **Share များ-** `MountPoints2` တွင် volume များနှင့် drive letter များအတွက် key များလည်း ရှိသည်။ ဤနေရာတွင် network share
  များ (`##server#share`) ကိုသာ ဖော်ပြသည်။ `\\server@SSL@443\DavWWWRoot` သည် WebDAV share ဖြစ်သည်။
- **ဖယ်ရှားခြင်း-** စာရင်းများကို ရှင်းလင်းခြင်း သို့မဟုတ် portable client ကို အသုံးပြုခြင်းသည် ဤနေရာတွင် ခြေရာ မကျန်စေပါ။
  Jump Lists၊ Prefetch၊ `.rdp` ဖိုင်များ၊ RDP bitmap cache နှင့် ပစ်မှတ်၏ log များနှင့် နှိုင်းယှဉ်ပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

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

အနှစ်ချုပ်သည် ချိတ်ဆက်မှုများကို client အလိုက် ရေတွက်သည်။ ၎င်းသည် key များကို နောက်ဆုံးရေးသားခဲ့သော ချိတ်ဆက်မှုများကို
အသစ်ဆုံးမှ စတင်၍ ဖော်ပြပြီး password သိမ်းဆည်းထားသော session များကိုလည်း ဖော်ပြသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host၊ profile ဖိုင်တွဲ၏ အမည်၊ စနစ်၏ `ProfileList` မှ အသုံးပြုသူ၏ SID (စနစ်၏ `SOFTWARE` hive ဖြင့်) နှင့် hive။ |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive` သို့မဟုတ် `network_share`။ |
| `Kind` | မှတ်တမ်း၏ ရင်းမြစ်- `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point`။ |
| `Target`, `Port` | ကွန်ပျူတာ (host အမည် သို့မဟုတ် IP လိပ်စာ) နှင့် port သို့မဟုတ် share (`\\server\share`)။ Port ကို Windows သို့မဟုတ် client က ရေးထားသည့်နေရာတွင်သာ ပေးသည်။ |
| `RemoteUsername` | ပစ်မှတ်ပေါ်တွင် အသုံးပြုခဲ့သော အကောင့်- `UsernameHint`၊ session ၏ `UserName` (သို့မဟုတ် PuTTY ၏ `HostName` ရှိ `user@`)၊ map လုပ်ထားသော drive ၏ `UserName`။ |
| `Protocol` | PuTTY ၏ `Protocol` (`ssh`, `telnet`, `raw`, …)၊ WinSCP ၏ `FSProtocol` (`sftp`, `scp`, `ftp`, `webdav`, `s3`)၊ map လုပ်ထားသော drive ၏ provider (`Microsoft Windows Network`)။ |
| `MRU-Position` | ၎င်း၏ စာရင်းအတွင်း နေရာ (0 သည် နောက်ဆုံး)- `MRU<n>`, `MRUList`၊ PuTTY ၏ `Recent sessions`။ |
| `SessionName` | သိမ်းဆည်းထားသော session ၏ အမည်၊ `%20` နှင့် အခြား escape များကို decode လုပ်ထားသည်။ |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | လက်ခံထားသော SSH host key ၏ အမျိုးအစား (`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …)၊ ၎င်း၏ OpenSSH fingerprint နှင့် သိမ်းဆည်းထားသည့်အတိုင်း ၎င်း၏ ဂဏန်းများ။ |
| `DriveLetter` | Map လုပ်ထားသော drive ၏ letter။ |
| `PasswordStored` | Session က password သိမ်းဆည်းထားခြင်း ရှိ/မရှိ။ ဘယ်သောအခါမျှ decode မလုပ်ပါ။ |
| `Details` | အခြားဆက်တင်များ၊ `Name=Value; …` (CSV) သို့မဟုတ် object (JSON) အဖြစ်- RDP certificate ၏ `CertHash`, `LocalDevices`၊ session ၏ key ဖိုင်၊ proxy၊ SSH tunnel၊ port forwarding များ၊ အဝေးမှ ညွှန်ကြားချက်နှင့် directory များ၊ `StoredPasswords`၊ share ၏ `Label`၊ drive ၏ `ConnectionType`။ |
| `KeyPath`, `ValueName` | Key နှင့် တန်ဖိုးတစ်ခုဖြစ်သော မှတ်တမ်းအတွက် တန်ဖိုး။ |
| `KeyLastWrittenUTC` | Key ၏ နောက်ဆုံးရေးသားချိန် (စာရင်းများအတွက် သတိပြုရန်များကို ကြည့်ပါ)။ |
| `DataRaw` | String မဟုတ်သည့်အခါ သို့မဟုတ် decode မလုပ်နိုင်သည့်အခါ တန်ဖိုး၏ byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | အမျိုးအစားမှားသော တန်ဖိုးများ (၎င်းတို့၏ byte များနှင့်အတူ)၊ မဖတ်နိုင်သော host key များ၊ မရှိသော `MRUList` letter များနှင့် ၎င်း မဖော်ပြသော တန်ဖိုးများ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. အသုံးပြုသူတစ်ဦးစီ၏ ပစ်မှတ်များကို client အလိုက် စာရင်းပြုစုပြီး ထိုအသုံးပြုသူ ရောက်ရှိသင့်သော စနစ်များနှင့် တိုက်ဆိုင်စစ်ဆေးပါ။
   စီမံခန့်ခွဲရေး share များ (`C$`, `ADMIN$`) နှင့် IP လိပ်စာများကို ရှာဖွေပါ။
2. အသုံးပြုခဲ့သော အကောင့်များ (`RemoteUsername`) ကို မှတ်သားပါ- domain administrator များနှင့် အခြားစက်များပေါ်ရှိ
   local အကောင့်များ။
3. `HostKeyFingerprint` ကို server များ၏ key များနှင့်လည်းကောင်း၊ `CertHash` ကို RDP server များ၏ certificate များနှင့်
   လည်းကောင်း တိုက်ဆိုင်ပါ။
4. ပစ်မှတ်များ၏ logon event များ (4624 type 10 နှင့် 3, 1149)၊ Jump Lists နှင့် LNK ဖိုင်များ (`mstsc.exe`,
   `putty.exe`၊ share များမှ ဖွင့်ခဲ့သော ဖိုင်များ)၊ Prefetch နှင့် RDP bitmap cache တို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။
5. ဖျက်ပစ်ထားသော entry များအတွက် `parse-registry read --recover-deleted` ကို အသုံးပြုပြီး ရှင်းလင်းထားသော စာရင်းများအတွက်
   profile ၏ `NTUSER.DAT` မိတ္တူဟောင်းများ (Volume Shadow Copies၊ backup များ) ကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Remote Desktop Connection Computer box မှ entry များကို ဖယ်ရှားခြင်း](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [PuTTY FAQ: PuTTY သည် ၎င်း၏ ဒေတာကို မည်သည့်နေရာတွင် သိမ်းဆည်းသနည်း](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: Configuration storage](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
