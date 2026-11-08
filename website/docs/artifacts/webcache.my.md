# Internet Explorer နှင့် legacy Edge { #internet-explorer-and-legacy-edge }

## WebCache database ဆိုသည်မှာ { #what-the-webcache-database-is }

`WebCacheV01.dat` သည် အသုံးပြုသူတစ်ဦးချင်းစီ၏ profile ရှိ Windows ၏ Internet function များ (WinINet) ၏ database
ဖြစ်ပြီး ESE database တစ်ခု ဖြစ်ပါသည်။ Internet Explorer 10 နှင့် 11 သည် ၎င်းတို့၏ history၊ download များ၊ cache
နှင့် cookie များကို ၎င်းတွင် သိမ်းဆည်းပြီး legacy Edge (Edge သည် Chromium-based မဖြစ်မီ Windows 10 ၏ Edge)
သည်လည်း ထိုနည်းတူ သိမ်းဆည်းပါသည်။ Record အမျိုးအစားတစ်ခုစီသည် container တစ်ခု၊ ကိုယ်ပိုင် table တစ်ခု ဖြစ်ပါသည်။

- **History** (`History` နှင့် တစ်ရက် သို့မဟုတ် တစ်ပတ်အတွက် `MSHist01…`): ကြည့်ရှုခဲ့သော URL များ၊ တစ်ခုစီကို
  စာမျက်နှာ၏ title၊ ၎င်းအတွက် မှတ်တမ်းတင်ခဲ့သော account၊ နောက်ဆုံး ကြည့်ရှုခဲ့သည့်အချိန်နှင့် ကြည့်ရှုခဲ့သည့် အကြိမ်ရေတို့နှင့်အတူ။
  File Explorer သည် ၎င်းဖွင့်သော local ဖိုင်များကိုလည်း ဤနေရာတွင် `file:///` URL များအဖြစ် မှတ်တမ်းတင်ပါသည်။
- **Download များ** (`iedownload`): ဖိုင် လာခဲ့သော URL၊ ၎င်းကို သိမ်းဆည်းခဲ့သည့်နေရာနှင့် download လုပ်နေစဉ်
  ရေးသားခဲ့သည့်နေရာ၊ ၎င်း၏ size၊ download စတင်ခဲ့သည့်အချိန်၊ ညွှန်းဆိုသော စာမျက်နှာနှင့် MIME type။
- **Cookie များ**: Windows 10 နှင့် နောက်ပိုင်းတွင် cookie များကိုယ်တိုင် (`CookieEntryEx` table များ)၊ ၎င်းတို့၏
  host၊ အမည်၊ value နှင့် အချိန်များနှင့်အတူ။ ယခင်က ၎င်းတို့ကို သိမ်းထားခဲ့သော cookie ဖိုင်များ။
- **Cache** (`Content`): စာမျက်နှာများကို ပြသရန် download လုပ်ခဲ့သော ဖိုင်များ၊ တစ်ခုစီကို ၎င်း၏ URL၊ disk ပေါ်ရှိ
  မိတ္တူ၏ တည်နေရာနှင့် HTTP response header များနှင့်အတူ။ WinINet ကို အသုံးပြုသော အခြား program များသည်လည်း
  ၎င်းတို့၏ download များကို ဤနေရာတွင် cache လုပ်ပါသည်။
- Web storage (`DOMStore`)၊ compatibility list များ (`iecompat`) နှင့် အခြားများ။

Database သည် ဘရောက်ဇာများထက် ပိုကြာရှည်စွာ ကျန်ရှိပါသည်။ Internet Explorer ကို အသုံးပြုမှု ရပ်စဲလိုက်ပြီ ဖြစ်ပြီး
Chromium-based Edge သည် ၎င်း၏ ကိုယ်ပိုင် SQLite database များကို သိမ်းဆည်းပါသည်
([ဝဘ် ဘရောက်ဇာများ](browser.md) ကို ကြည့်ပါ)။ သို့သော် database သည် profile တိုင်းတွင် ယခင်က မှတ်တမ်းတင်ခဲ့သည့်
အရာများ၊ File Explorer နှင့် အခြား program များက ယခုထိ မှတ်တမ်းတင်နေဆဲ အရာများနှင့်အတူ ရှိနေဆဲ ဖြစ်ပါသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> အသုံးပြုသူ၏ profile သည် မှတ်တမ်းတင်ထားသော အချိန်တွင်
URL၊ download သို့မဟုတ် cache လုပ်ထားသော ဖိုင်ကို မှတ်တမ်းတင်ခဲ့ပါသည်။ Keyboard ရှေ့တွင် မည်သူ ထိုင်နေခဲ့သည်ကို
သို့မဟုတ် cache လုပ်ထားသော ဖိုင်ကို ဖွင့်ခဲ့သည်ကို database က မဖော်ပြပါ။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

Container များက အမည်ဖော်ပြသော ဖိုင်များ (cache လုပ်ထားသော ဖိုင်များ၊ cookie ဖိုင်များ) သည် ၎င်းတို့၏ ကိုယ်ပိုင်
ဖိုင်တွဲများတွင် ရှိပါသည်: `%LOCALAPPDATA%\Microsoft\Windows\INetCache\`၊ `INetCookies\`၊ `History\` နှင့်
app များအတွက် `%LOCALAPPDATA%\Packages\` အောက်တွင်။

`WebCache` ဖိုင်တွဲတစ်ခုလုံးကို စုဆောင်းပါ။ အသုံးပြုသူ logon ဝင်ထားစဉ် Windows သည် database ကို ဖွင့်ထားသောကြောင့်
လည်ပတ်နေသော စနစ်မှ ကူးယူထားသော မိတ္တူသည် များသောအားဖြင့် dirty ဖြစ်နေပါသည်။ ၎င်း၏ နောက်ဆုံး ပြောင်းလဲမှုများသည်
`V01*.log` transaction log များတွင်သာ ရှိပါသည်။

!!! warning "Transaction log များ"
    Juntei သည် ESE transaction log များကို replay မလုပ်ပါ။ ပုံမှန်အတိုင်း ပိတ်ခြင်း မခံရသော database ကို summary
    တွင် dirty အဖြစ် ကျန်နေသည်ဟု ရေတွက်ပြီး problem အဖြစ် အစီရင်ခံပါသည်။ ၎င်း၏ log များတွင်သာ ရှိနေသေးသော
    ပြောင်းလဲမှုများ ပျောက်နေပါသည်။ ၎င်းတို့ကို ထည့်သွင်းလိုလျှင် ဖိုင်တွဲ၏ မိတ္တူတစ်ခုထဲသို့ log များကို ဦးစွာ replay
    လုပ်ပါ။ ထိုမိတ္တူထဲတွင် Windows ၏ `esentutl /r V01 /d` ကို run ပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- အချိန်များသည် UTC ဖြင့် 100 nanosecond အထိ ဖြစ်ပါသည်။ ခြွင်းချက်မှာ တစ်ရက် သို့မဟုတ် တစ်ပတ်၏ history container ၏
  `ModifiedTime` ဖြစ်ပြီး ၎င်းသည် offset မပါဘဲ ရေးထားသော စနစ်၏ local time ဖြစ်ကာ `-Z` က ၎င်းကို ပြောင်းလဲခြင်း
  မပြုပါ။ ၎င်းကို ထိုအချိန်က source စနစ်၏ time zone ဖြင့် ပြောင်းလဲပါ။ ၎င်းနှင့် `AccessedTime` ကြား ကွာခြားချက်သည်
  မကြာခဏ ထို time zone ၏ offset ဖြစ်သော်လည်း အမြဲတမ်း မဟုတ်ပါ: နှစ်ခုသည် မတူညီသော အချိန်အခိုက်အတန့်များ ဖြစ်နိုင်ပါသည်။
- History entry တစ်ခုသည် visit တစ်ခု မဟုတ်ဘဲ URL တစ်ခု ဖြစ်ပါသည်။ `AccessedTime` သည် ၎င်း၏ နောက်ဆုံး visit
  ဖြစ်ပြီး `AccessCount` သည် visit အရေအတွက် ဖြစ်ပါသည်။ `History` container သည် URL တစ်ခုလျှင် entry တစ်ခု
  ထားပါသည်။ တစ်ရက် သို့မဟုတ် တစ်ပတ်၏ container သည် ထိုကာလ၏ entry များကို ထားပြီး ၎င်း၏ အမည်သည် ကာလ၏
  ပထမဆုံးနေ့နှင့် နောက်ဆုံးနေ့၏ နောက်တစ်ရက်ကို ဖော်ပြပါသည် (`MSHist012026110120261102`: 2026 ခုနှစ်
  နိုဝင်ဘာ 1 ရက်)။
- History ရှိ `file:///` URL များသည် web စာမျက်နှာများ မဟုတ်ဘဲ File Explorer နှင့် အခြား program များမှ ဖွင့်ခဲ့သော
  local ဖိုင်များ ဖြစ်ပါသည်။
- `Account` သည် entry ကို မှတ်တမ်းတင်ခဲ့သည့် account ဖြစ်ပါသည် (`Visited: alice@…`)။ `User` သည် database ၏ path
  ပါဝင်သည့် profile ဖိုင်တွဲ ဖြစ်ပါသည်။ ၎င်းတို့ ကွဲပြားနိုင်ပါသည်: profile ဖိုင်တွဲကို ၎င်း၏ account အမည်အတိုင်း
  အမြဲ အမည်ပေးထားသည် မဟုတ်သလို အခြားနေရာသို့ ကူးယူထားသော database ၏ path တွင် profile ဖိုင်တွဲ မပါပါ။
- `Package` သည် entry ၏ ပိုင်ရှင်ကို ဖော်ပြပါသည်: legacy Edge (`microsoft.microsoftedge_8wekyb3d8bbwe`)၊
  Enhanced Protected Mode ရှိ Internet Explorer (`windows_ie_ac_001`) သို့မဟုတ် အခြား app တစ်ခု။ ဗလာ ဖြစ်လျှင်
  ၎င်းသည် အသုံးပြုသူ၏ ကိုယ်ပိုင် ဖြစ်ပါသည်: Internet Explorer၊ File Explorer နှင့် အခြား program များ။
- Microsoft သည် download record ကို စာရွက်စာတမ်း မပြုစုထားပါ။ Juntei သည် ၎င်း၏ စတင်ချိန်နှင့် size ကို ၎င်း၏
  header သည် record အတွင်း ဆံ့မှသာ ဖတ်ပြီး ၎င်း၏ path များနှင့် အခြား string များကို နောက်ဆုံး string နှစ်ခုသည်
  URL တစ်ခုနှင့် Windows path တစ်ခု ဖြစ်မှသာ ဖတ်ပါသည်။ မဟုတ်လျှင် row တွင် entry ၏ ကိုယ်ပိုင် URL ကိုလည်းကောင်း၊
  စတင်ချိန်ကို မဖတ်နိုင်သည့်အခါ ၎င်း၏ access အချိန်ကိုလည်းကောင်း problem တစ်ခုနှင့်အတူ ထည့်ပါသည်။ `Publisher`
  သည် ဘရောက်ဇာက မှတ်တမ်းတင်ခဲ့သော အမည် ဖြစ်ပြီး ဖိုင်၏ signature ကို စစ်ဆေးထားခြင်း မဟုတ်ပါ။
- Cache entry တစ်ခုသည် ဖိုင်ကို download လုပ်ပြီး cache လုပ်ခဲ့ကြောင်း ဖော်ပြပါသည်။ Disk ပေါ်ရှိ မိတ္တူသည်
  ပျောက်သွားနိုင်၊ အစားထိုးခံရနိုင် သို့မဟုတ် တစ်ခါမျှ ဖွင့်ခံရခြင်း မရှိခဲ့နိုင်ပါ။
- ဖျက်ထားသော entry များကို ယခုထိ ပြန်လည်မရယူသေးပါ။ ESE သည် ဖျက်ထားသော record ကို ၎င်း၏ page များကို
  ရှင်းလင်းသည်အထိ ထို page များတွင် ထားနိုင်သော်လည်း Juntei သည် live record များကိုသာ ဖတ်ပါသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` သည် `WebCacheV01.dat` ကို အမည်ဖြင့် ရှာဖွေပါသည်။ `-f` သည် database တစ်ခုကို ၎င်း၏ အမည် မည်သို့ပင်ဖြစ်စေ
ဖတ်ပါသည်။

## Output table များ { #output-tables }

CSV သည် ဖိုင်ငါးခု ဖန်တီးပြီး row မရှိလျှင် ဖိုင်တစ်ခုစီတွင် ၎င်း၏ header သာ ပါဝင်ပါသည်။ JSON နှင့် JSONL row များတွင်
`SourceFile` ၏ နောက်တွင် `Table` (`History`၊ `Downloads`၊ `Cookies`၊ `Entries` သို့မဟုတ် `Containers`) ပါဝင်ပါသည်။

| CSV ဖိုင် | အကြောင်းအရာ |
|---|---|
| `WebCache-History.csv` | History container များ၏ entry များ။ |
| `WebCache-Downloads.csv` | Download များ။ |
| `WebCache-Cookies.csv` | Cookie များ (Windows 10 နှင့် နောက်ပိုင်း)။ |
| `WebCache-Entries.csv` | အခြား container တိုင်း၏ entry များ: cache လုပ်ထားသော ဖိုင်များ၊ cookie ဖိုင်များ၊ web storage၊ compatibility list များနှင့် ကျန်အရာများ။ |
| `WebCache-Containers.csv` | Container များ။ |

## Field များ { #fields }

Table တိုင်းသည် ဤ field များဖြင့် စတင်ပါသည်။

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `SourceFile` | Row ကို ဖတ်ယူခဲ့သော database။ |
| `User` | Path ပါဝင်သည့် profile ၏ Windows အသုံးပြုသူ (`Users` ၏ နောက်ရှိ ဖိုင်တွဲ)။ |
| `Package` | Row ၏ data ပိုင်ရှင် app: ၎င်း၏ container ၏ directory (သို့မဟုတ် ၎င်း၏ cookie table ၏ directory) ရှိ `Packages` အောက်ရှိ ဖိုင်တွဲ။ အသုံးပြုသူ၏ ကိုယ်ပိုင်အတွက် ဗလာ။ |

### History { #history }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `Container` | Container: `History` သို့မဟုတ် `MSHist01` နှင့် ၎င်းနောက်တွင် ၎င်း၏ ကာလ၏ ပထမဆုံးနေ့နှင့် နောက်ဆုံးနေ့၏ နောက်တစ်ရက်။ |
| `AccessedTime` | URL ၏ နောက်ဆုံး visit (UTC)။ တစ်ရက် သို့မဟုတ် တစ်ပတ်၏ container တွင် ထိုကာလ၏ နောက်ဆုံး visit။ |
| `ModifiedTime` | Entry ကို နောက်ဆုံး ပြောင်းလဲခဲ့သည့်အချိန် (UTC)။ တစ်ရက် သို့မဟုတ် တစ်ပတ်၏ container တွင် offset မပါသော စနစ်၏ local time။ |
| `CreationTime` | Entry ကို ဖန်တီးခဲ့သည့်အချိန် (UTC)။ |
| `ExpiryTime`, `SyncTime` | Entry သည် history မှ ထွက်ရန် သတ်မှတ်ထားသည့်အချိန်နှင့် နောက်ဆုံး synchronize လုပ်ခဲ့သည့်အချိန် (UTC)။ |
| `Account` | Entry ကို မှတ်တမ်းတင်ခဲ့သည့် account။ |
| `URL` | `Visited: account@` prefix မပါသော စာမျက်နှာ သို့မဟုတ် ဖိုင်။ |
| `Title` | Entry က မှတ်တမ်းတင်ခဲ့သည့်အတိုင်း စာမျက်နှာ၏ title။ တစ်ရက် သို့မဟုတ် တစ်ပတ်၏ container တွင် ဗလာ။ |
| `AccessCount`, `SyncCount` | URL ကို ကြည့်ရှုခဲ့သည့် အကြိမ်ရေနှင့် entry ကို synchronize လုပ်ခဲ့သည့် အကြိမ်ရေ။ |
| `EntryType`, `Flags` | သိမ်းဆည်းထားသည့်အတိုင်း entry ၏ type နှင့် flag များ။ |
| `EntryID`, `ContainerID` | ၎င်း၏ container အတွင်းရှိ entry ၏ id နှင့် container ၏ id။ |

### Download များ { #downloads }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `StartTime` | Download စတင်ခဲ့သည့်အချိန် (UTC)။ |
| `URL` | ဖိုင် လာရာနေရာ။ |
| `TargetPath`, `CurrentPath` | ဖိုင်ကို သိမ်းဆည်းခဲ့သည့်နေရာနှင့် download လုပ်နေစဉ် ရေးသားခဲ့သည့်နေရာ: legacy Edge အတွက် `.partial` ဖိုင်၊ Internet Explorer အတွက် cache ၏ မိတ္တူ။ |
| `TotalBytes` | ဖိုင်၏ size။ |
| `Referrer` | ညွှန်းဆိုသော စာမျက်နှာ။ |
| `MimeType` | Server က ပေးပို့ခဲ့သည့်အတိုင်း ဖိုင်၏ type (Internet Explorer)။ |
| `Publisher` | ဖိုင်၏ publisher အတွက် legacy Edge က မှတ်တမ်းတင်ခဲ့သော အမည်။ |
| `AccessedTime` | Entry ၏ ကိုယ်ပိုင် access အချိန် (UTC)၊ အများအားဖြင့် စတင်ချိန်။ |
| `DownloadID`, `ContainerID` | ၎င်း၏ container အတွင်းရှိ entry ၏ id နှင့် container ၏ id။ |

### Cookie များ { #cookies }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `LastModified`, `Expires` | Cookie ကို သတ်မှတ်ခဲ့သည့်အချိန်နှင့် သက်တမ်းကုန်မည့်အချိန် (UTC)။ |
| `Host`, `Name`, `Path` | Cookie သက်ဆိုင်သော site၊ ၎င်း၏ အမည်နှင့် path။ |
| `Value` | သိမ်းဆည်းထားသည့်အတိုင်း value။ |
| `Flags` | သိမ်းဆည်းထားသည့်အတိုင်း cookie ၏ flag များ။ |
| `Directory` | Database က cookie table အား ပေးထားသော directory: cookie များ၏ ပိုင်ရှင် (`…\INetCookies\ESE\`၊ legacy Edge ၏ `…\MicrosoftEdge\Cookies\ESE\`၊ app တစ်ခု၏ directory)။ |
| `CookieTable`, `EntryID` | Table (`CookieEntryEx_<n>`) နှင့် ၎င်းအတွင်းရှိ cookie ၏ id။ |

### Entry များ { #entries }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `Container` | Container: `Content` (cache လုပ်ထားသော ဖိုင်များ)၊ `Cookies` (cookie ဖိုင်များ)၊ `DOMStore` (web storage)၊ `iecompat`၊ …။ |
| `AccessedTime` | Entry ကို နောက်ဆုံး အသုံးပြုခဲ့သည့်အချိန် (UTC)။ |
| `ModifiedTime` | Server က ပေးခဲ့သည့်အတိုင်း resource ၏ နောက်ဆုံး ပြင်ဆင်ချိန် (UTC)။ |
| `CreationTime` | Entry ကို ဖန်တီးခဲ့သည့်အချိန် (UTC)။ |
| `ExpiryTime`, `SyncTime` | သက်တမ်းကုန်မည့်အချိန်နှင့် server နှင့် နောက်ဆုံး စစ်ဆေးခဲ့သည့်အချိန် (UTC)။ |
| `URL` | သိမ်းဆည်းထားသည့်အတိုင်း URL: cache လုပ်ထားသော ဖိုင်အတွက် URL၊ cookie ဖိုင်အတွက် `Cookie:alice@example.com/`၊ `DOMStore:…`၊ `iecompat:…`။ |
| `Account` | Cookie ဖိုင်တစ်ခု၏ account။ |
| `Filename`, `FilePath` | Entry ၏ ဖိုင်နှင့် ၎င်း၏ တည်နေရာ: container ၏ directory နှင့် entry က အမည်ဖော်ပြသော subfolder။ |
| `FileSize` | ဖိုင်၏ size။ |
| `AccessCount`, `SyncCount` | Entry ကို အသုံးပြုခဲ့သည့် အကြိမ်ရေနှင့် server နှင့် စစ်ဆေးခဲ့သည့် အကြိမ်ရေ။ |
| `EntryType`, `Flags` | သိမ်းဆည်းထားသည့်အတိုင်း entry ၏ type နှင့် flag များ။ |
| `RedirectURL` | URL redirect လုပ်သွားခဲ့သည့်နေရာ။ |
| `ResponseHeaders`, `RequestHeaders` | သိမ်းဆည်းထားသည့်အတိုင်း HTTP response header များ (`HTTP/1.1 200 OK`၊ `Content-Type`၊ `Server`၊ …) နှင့် request header များ၊ ၎င်းတို့၏ line များ အားလုံးနှင့်အတူ။ |
| `EntryID`, `ContainerID` | ၎င်း၏ container အတွင်းရှိ entry ၏ id နှင့် container ၏ id။ |

### Container များ { #containers }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `ContainerID`, `Name` | Container ၏ id နှင့် အမည်။ |
| `Directory` | Disk ပေါ်ရှိ ၎င်း၏ ဖိုင်များ၏ တည်နေရာ။ |
| `SecureDirectories` | ၎င်း၏ ဖိုင်များ ရှိသော `Directory` ၏ subfolder များ (CSV တွင် ၎င်းတို့ကြား comma)။ |
| `PartitionID` | အသုံးပြုသူ၏ ကိုယ်ပိုင် container များအတွက် `M`၊ Internet Explorer ၏ Protected Mode ၏ container များ (`Low` ဖိုင်တွဲများ) အတွက် `L`၊ app တစ်ခု၏ container များအတွက် ၎င်း၏ app container SID။ |
| `LastAccessTime`, `LastScavengeTime` | Container ကို နောက်ဆုံး အသုံးပြုခဲ့သည့်အချိန်နှင့် နောက်ဆုံး ရှင်းလင်းခဲ့သည့်အချိန် (UTC)။ |

## အလိုအလျောက် ခွဲခြမ်းစိတ်ဖြာခြင်း { #automatic-parsing }

`auto-parse` သည် table များကို `BrowserActivity` ထဲသို့ `WebCache-History.csv`၊ `WebCache-Downloads.csv`၊
`WebCache-Cookies.csv`၊ `WebCache-Entries.csv` နှင့် `WebCache-Containers.csv` အဖြစ် (`-t json` သို့မဟုတ်
`-t jsonl` ဖြင့်ဆိုလျှင် `WebCache.json` သို့မဟုတ် `WebCache.jsonl` ဖိုင်တစ်ခုတည်း) `BrowserActivity/WebCache-logs.txt`
နှင့်အတူ ရေးသားပါသည်။ ပေါင်းစပ်ထားသော `-j` timeline တွင် row တိုင်းကို `Comp` နှင့် `Artifact` identifier `webcache`
တို့နှင့်အတူ ထည့်သွင်းပါသည်။ `-Z` သည် တစ်ရက် သို့မဟုတ် တစ်ပတ်၏ history container ၏
local `ModifiedTime` မှလွဲ၍ အချိန်တိုင်းကို ပြောင်းလဲပေးပါသည်: `Time` ဖြင့် အဆုံးသတ်သော field များ၊ `LastModified` နှင့်
`Expires`။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft — About WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE database file format](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — MSIE WebCache parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
