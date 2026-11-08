# ဝဘ် ဘရောက်ဇာများ { #web-browsers }

## ဘရောက်ဇာ database များ ဆိုသည်မှာ { #what-the-browser-databases-are }

ဘရောက်ဇာများသည် ၎င်းတို့၏ history၊ download များ၊ cookie များနှင့် form entry များကို အသုံးပြုသူတစ်ဦးချင်းစီ၏
profile ရှိ SQLite database များတွင် သိမ်းဆည်းပါသည်။ Chromium-based ဘရောက်ဇာများ (Chrome၊ Edge၊ Brave၊ Opera၊
Vivaldi) နှင့် Firefox-based ဘရောက်ဇာများသည် ကိုယ်ပိုင် ဖိုင်များနှင့် table များကို အသီးသီး အသုံးပြုကြသော်လည်း
တူညီသော လှုပ်ရှားမှု အမျိုးအစားများကို မှတ်တမ်းတင်ပါသည်။

- **History**: ကြည့်ရှုခဲ့သော စာမျက်နှာများ၊ visit တစ်ခုစီကို ၎င်း၏ အချိန်၊ ဘရောက်ဇာက စာမျက်နှာသို့ ရောက်ရှိခဲ့ပုံ
  (ရိုက်ထည့်ခြင်း၊ link၊ redirect၊ bookmark) နှင့် Chromium တွင် စာမျက်နှာ ရှေ့တွင် ရှိနေခဲ့သည့် ကြာချိန်တို့နှင့်အတူ။
- **Download များ**: ဖိုင် လာခဲ့သော URL (နှင့် လမ်းတစ်လျှောက်ရှိ redirect များ)၊ ၎င်းကို သိမ်းဆည်းခဲ့သည့်နေရာ၊ ၎င်း၏
  size၊ download စတင်ခဲ့ပြီး ပြီးဆုံးခဲ့သည့်အချိန်နှင့် ပြီးမြောက်ခဲ့ခြင်း ရှိမရှိ။
- **Cookie များ**: ၎င်းတို့ကို သတ်မှတ်ခဲ့သော site များ၊ သတ်မှတ်ခဲ့သည့်အချိန်၊ နောက်ဆုံး ပေးပို့ခဲ့သည့်အချိန်နှင့်
  သက်တမ်းကုန်မည့်အချိန်။
- **Form history**: web form များနှင့် search box များတွင် ရိုက်ထည့်ခဲ့သော value များ၊ ရိုက်ထည့်ခဲ့သည့် အကြိမ်ရေနှင့်
  ပထမဆုံးနှင့် နောက်ဆုံး ရိုက်ထည့်ခဲ့သည့်အချိန်။

ဘရောက်ဇာ လှုပ်ရှားမှုသည် အသုံးပြုသူ ဘာလုပ်ခဲ့သည်ကို အတိုက်ရိုက်ဆုံး ပြသသော မှတ်တမ်း ဖြစ်လေ့ရှိပါသည်။ ၎င်းတို့
လိုက်သွားခဲ့သော phishing စာမျက်နှာ၊ download လုပ်ခဲ့သော tool၊ အသုံးပြုခဲ့သော webmail နှင့် ရိုက်ထည့်ခဲ့သော
search term များ ဖြစ်ပါသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ဘရောက်ဇာ profile သည် မှတ်တမ်းတင်ထားသော အချိန်တွင်
URL သို့ visit တစ်ခုကို မှတ်တမ်းတင်ခဲ့သည် သို့မဟုတ် download ကို မှတ်တမ်းတင်ထားသော path သို့ သိမ်းဆည်းခဲ့ပါသည်။
Keyboard ရှေ့တွင် မည်သူ ထိုင်နေခဲ့သည်ကို database က မဖော်ပြပါ။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` သည် `Default`၊ `Profile 1`၊ `Profile 2` စသည်ဖြင့် ဖြစ်ပါသည်။ Chromium သည် ဗားရှင်း 96 တွင် `Cookies` ကို
`Network` ထဲသို့ ရွှေ့ခဲ့ပြီး ယခင် ဗားရှင်းများတွင် ၎င်းကို `History` ဘေးတွင် ထားပါသည်။ Firefox သည် ဗားရှင်း 25 အထိ
download များကို `downloads.sqlite` တွင် ထားခဲ့ပြီး နောက်ပိုင်း ဗားရှင်းများတွင် `places.sqlite` တွင် ထားပါသည်။

Profile ၏ ဖိုင်တိုင်းကို စုဆောင်းပြီး database တစ်ခုစီကို ၎င်း၏ အမည်ကို မျှဝေသော ဘေးရှိ ဖိုင်များနှင့်အတူ စုဆောင်းပါ:
`places.sqlite-wal`၊ `cookies.sqlite-wal`၊ `formhistory.sqlite-wal`၊ `History-journal`။

!!! warning "Write-ahead log များ"
    Firefox သည် ၎င်း၏ database များကို write-ahead log mode ဖြင့် ထားပါသည်။ နောက်ဆုံး ပြောင်းလဲမှုများသည် ဘရောက်ဇာက
    ၎င်းတို့ကို database ထဲသို့ ကူးထည့်သည်အထိ (checkpoint) `-wal` ဖိုင်တွင်သာ ရှိပါသည်။ Juntei သည် ဘရောက်ဇာ
    ဖတ်သကဲ့သို့ database တစ်ခုစီကို ၎င်း၏ `-wal` ဖိုင်နှင့်အတူ ဖတ်ပါသည်။ ထိုဖိုင် မရှိလျှင် မကြာသေးမီက visit များ၊
    cookie များနှင့် form entry များ ပျောက်နေပြီး log ထဲတွင် ဖျက်ခဲ့သော entry များသည် live အဖြစ် ဆက်လက် ဖတ်ရပါသည်။

## ဖျက်ထားသော entry များ { #deleted-entries }

ဘရောက်ဇာက history ကို ဖျက်သည့်အခါ row များသည် table များမှ ထွက်သွားသော်လည်း ၎င်းတို့၏ record များသည် database ၏
free space တွင်၊ ၎င်း၏ freelist ပေါ်တွင် သို့မဟုတ် database ၏ page များ၏ ဗားရှင်းဟောင်းများတွင် ကျန်ရှိနေနိုင်ပါသည်။
ထိုဗားရှင်းဟောင်းများသည် `-wal` ဖိုင်တွင် သို့မဟုတ် ၎င်းတို့ကို အစားထိုးခဲ့သော `-wal` ၏ အောက်ရှိ database ဖိုင်တွင်
ရှိပါသည်။ Juntei သည် ထိုနေရာများမှ ဖျက်ထားသော visit များ၊ စာမျက်နှာများ၊ download များနှင့် form entry များကို
`Status` သည် `Deleted` ဖြစ်သော row များအဖြစ် ပြန်လည်ရယူပါသည်။

- `Confidence` သည် record နှင့် ၎င်း၏ table ကိုက်ညီမှု မည်မျှ သေချာသည်ကို ဖော်ပြပါသည်။ Value တိုင်းတွင် ၎င်း၏
  column ၏ type ရှိသော record အပြည့်အစုံအတွက် `High`၊ ပျက်စီးနေသော record များ သို့မဟုတ် table အများအပြားနှင့်
  ကိုက်ညီသော record များအတွက် `Medium` သို့မဟုတ် `Low` ဖြစ်ပါသည်။
- `RecoveredFrom` သည် record ရှိခဲ့သည့်နေရာကို ဖော်ပြပါသည်: page ၏ နေရာ (cell၊ freeblock၊ unallocated space သို့မဟုတ်
  free page)၊ မည်သည့် page နှင့် ၎င်း၏ မည်သည့် ဗားရှင်းနှင့် page အတွင်းရှိ record ၏ offset။ WAL frame များကို 0 မှ
  စ၍ ရေတွက်ပါသည်။
- `LostColumns` သည် ပျက်စီးနေသော record တစ်ခုတွင် မရှိတော့သော column များကို စာရင်းပြုစုပါသည်။ ၎င်းတို့၏ field များ
  ဗလာ ဖြစ်ပါသည်။

ဘရောက်ဇာများသည် SQLite ၏ `secure_delete` ဖြင့် ဖျက်ပါသည်။ Database အတွင်း လွတ်သွားသော space ကို သုညဖြင့်
ဖြည့်ပါသည်။ ထို့ကြောင့် ပြန်လည်ရယူနိုင်သည့်အရာများမှာ အများအားဖြင့် page ဗားရှင်းဟောင်းများတွင် ရှိပါသည်။ ၎င်းတို့မှာ
`-wal` ဖိုင်တွင် ဆက်ရှိနေသေးသော ဗားရှင်းများနှင့် `-wal` က အစားထိုးခဲ့သော page များ၏ database ဖိုင်ကိုယ်တိုင်ရှိ
မိတ္တူများ ဖြစ်ပြီး နောက် checkpoint အထိသာ ရှိပါသည်။ `-wal` ဖိုင်ကို ၎င်း၏ database နှင့်အတူ ကူးယူပါ၊ ထို့ပြင်
ဘရောက်ဇာ ထပ်မံ မလည်ပတ်မီ ကူးယူပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- အချိန်များသည် UTC ဖြင့် ဖြစ်ပါသည်။ Chromium နှင့် Firefox သည် ၎င်းတို့ကို microsecond အထိ မှတ်တမ်းတင်ပြီး
  Chromium ၏ form history ကိုမူ second အထိသာ မှတ်တမ်းတင်ပါသည်။
- Visit တစ်ခုသည် အသုံးပြုသူက ရိုက်ထည့်ခဲ့သည် သို့မဟုတ် click လုပ်ခဲ့သည်ဟု မဆိုလိုပါ။ Redirect များ၊ frame များနှင့်
  အခြား program များက ဖွင့်ခဲ့သော စာမျက်နှာများသည်လည်း visit များ ဖြစ်ပါသည်။ `Transition`၊ `Typed` နှင့် `Hidden`
  ကို ဖတ်ပါ။
- Chromium သည် cookie value များကို encrypt လုပ်ပါသည်။ ၎င်းတို့အတွက် `Value` သည် ဗလာ ဖြစ်ပါသည်။ Juntei သည် decrypt
  မလုပ်ပါ။
- `Removed` form-history row သည် Firefox ၏ `moz_deleted_formhistory` မှ လာပြီး ၎င်းကို Android အတွက် Firefox က Sync
  အတွက် ဖြည့်ပါသည်။ Desktop Firefox သည် ၎င်းကို ပုံမှန်အားဖြင့် ဗလာ ထားပါသည်။
- `Low` confidence ရှိသော ဖျက်ထားသည့် entry တစ်ခုသည် အခြား record တစ်ခု၏ အပိုင်းအစ ဖြစ်နိုင်ပါသည်။ ၎င်းကို live
  row များနှင့် အခြား အထောက်အထားများနှင့် တိုက်ဆိုင်စစ်ဆေးပါ။
- ဖျက်ထားသော row မရှိခြင်းသည် ဘာမျှ မဖျက်ခဲ့ဟု မဆိုလိုပါ။ သုညဖြင့် ဖြည့်ထားသော space နှင့် checkpoint တို့သည်
  ပြန်လည်ရယူစရာ ဘာမျှ မချန်ထားပါ။ ဖျက်ထားသော cookie များကို ယခုထိ ပြန်လည်မရယူသေးပါ။
- Database အကြောင်းအရာက browser မျိုးစုကို သတ်မှတ်သည်။ ကိုက်ညီသော profile path ရှိလျှင် `Browser` ကို Chrome သို့မဟုတ် Firefox ကဲ့သို့ ဖော်ပြသည်။ မကိုက်ညီသော path ကြောင့် မျိုးစု မပြောင်းပါ။ အခြားအခြေအနေတွင် `Chromium-based`၊ `Firefox-based` သို့မဟုတ် `Safari` ဖြစ်ပြီး မသိလျှင် `Unknown` ဖြစ်သည်။ `User` နှင့် `Profile` သည် path မှ ရရှိပြီး ဖိုင်ကူးယူသည့်အခါ ပျောက်ဆုံးနိုင်သည်။
- Rollback journal များ (`-journal`) ကို မဖတ်ပါ။ Internet Explorer နှင့် legacy Edge ၏ `WebCacheV01.dat` ကို
  `parse-ese webcache` က ဖတ်ပါသည်။ [Internet Explorer နှင့် legacy Edge](webcache.md) ကို ကြည့်ပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` သည် ဖိုင်များကို အမည်ဖြင့် ရှာဖွေပြီး ဖိုင်တစ်ခုစီကို ၎င်း၏ table များဖြင့် ခွဲခြားသိရှိပါသည်။ အခြား program
တစ်ခု၏ `History` ဖိုင်ကို ကျော်သွားပါသည်။ `-f` သည် database တစ်ခုကို ၎င်း၏ အမည် မည်သို့ပင်ဖြစ်စေ ဖတ်ပါသည်။

## Output table များ { #output-tables }

CSV သည် ဖိုင်လေးခု ဖန်တီးပြီး row မရှိလျှင် ဖိုင်တစ်ခုစီတွင် ၎င်း၏ header သာ ပါဝင်ပါသည်။ JSON နှင့် JSONL row များတွင်
`SourceFile` ၏ နောက်တွင် `Table` (`History`၊ `Downloads`၊ `Cookies` သို့မဟုတ် `FormHistory`) ပါဝင်ပါသည်။

| CSV ဖိုင် | အကြောင်းအရာ |
|---|---|
| `Browser-History.csv` | Visit များ၊ live နှင့် ဖျက်ထားသည်များ။ မည်သည့် live စာမျက်နှာတွင်မျှ မရှိသော URL ပါသည့် ဖျက်ထားသော စာမျက်နှာများ။ |
| `Browser-Downloads.csv` | Download များ၊ live နှင့် ဖျက်ထားသည်များ။ |
| `Browser-Cookies.csv` | Cookie များ။ |
| `Browser-FormHistory.csv` | Form entry များ၊ live နှင့် ဖျက်ထားသည်များ။ Firefox ၏ removal record များ။ |

## Field များ { #fields }

Table တိုင်းသည် ဤ field များဖြင့် စတင်ပါသည်။

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `SourceFile` | Row ကို ဖတ်ယူခဲ့သော database။ |
| `User` | Path ပါဝင်သည့် profile ၏ Windows အသုံးပြုသူ (`Users` ၏ နောက်ရှိ ဖိုင်တွဲ)။ |
| `Browser` | အကြောင်းအရာမှ သတ်မှတ်သော browser မျိုးစု၊ ကိုက်ညီသော profile path ဖြင့် ထပ်မံခွဲခြားသည်။ |
| `Profile` | ဘရောက်ဇာ profile: database ၏ ဖိုင်တွဲ။ |
| `Status` | `Live`၊ `Deleted` (ပြန်လည်ရယူထားသည်) သို့မဟုတ် `Removed` (Firefox removal record)။ |

### History { #history }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `RecordType` | `Visit` သို့မဟုတ် visit များပါ ပျောက်သွားနိုင်သော ဖျက်ထားသည့် စာမျက်နှာအတွက် `Page`။ |
| `VisitTime` | Visit စတင်ခဲ့သည့်အချိန် (UTC)။ |
| `URL`, `Title` | စာမျက်နှာနှင့် ၎င်း၏ title၊ နောက်ဆုံး တွေ့ခဲ့သည့်အတိုင်း။ |
| `Transition` | ဘရောက်ဇာက စာမျက်နှာသို့ ရောက်ရှိခဲ့ပုံ။ Chromium: core type နှင့် ၎င်းနောက်တွင် qualifier များ (`LINK`၊ `TYPED`၊ `AUTO_BOOKMARK`၊ `FORM_SUBMIT`၊ `RELOAD`၊ … နှင့်အတူ `CHAIN_START`၊ `SERVER_REDIRECT`၊ `FROM_ADDRESS_BAR`၊ …)။ Firefox: `LINK`၊ `TYPED`၊ `BOOKMARK`၊ `EMBED`၊ `REDIRECT_PERMANENT`၊ `REDIRECT_TEMPORARY`၊ `DOWNLOAD`၊ `FRAMED_LINK`၊ `RELOAD`။ |
| `VisitDuration` | စာမျက်နှာ ရှေ့တွင် ရှိနေခဲ့သော စက္ကန့် (Chromium၊ မတိုင်းတာထားလျှင် 0)။ |
| `VisitCount` | စာမျက်နှာသို့ visit စုစုပေါင်း။ |
| `Typed`, `TypedCount` | URL ကို တစ်ကြိမ်ကြိမ် ရိုက်ထည့်ခဲ့ခြင်း ရှိမရှိနှင့် အကြိမ်မည်မျှ (Chromium)။ |
| `Hidden` | History စာရင်းမှ ဝှက်ထားသည်: subframe များ၊ redirect source များ၊ error များ။ |
| `Frecency` | Address bar အကြံပြုချက်များအတွက် Firefox ၏ စာမျက်နှာ အဆင့်သတ်မှတ်ချက်။ |
| `VisitID`, `FromVisitID` | Visit ၏ row id နှင့် ၎င်း လာခဲ့သော visit ၏ row id (ညွှန်းဆိုသော စာမျက်နှာ၊ redirect ၏ source)။ ပြန်လည်ရယူထားသော record တွင် row id ပျောက်ဆုံးသွားပါက visit ၏ row id သည် ဗလာ ဖြစ်သည်။ |
| `LastVisitTime` | ဖျက်ထားသော စာမျက်နှာတစ်ခု၏ နောက်ဆုံး visit (UTC)။ |
| `PageSource` | ဖျက်ထားသော visit တစ်ခု၏ URL နှင့် title လာရာ: `Live` (live စာမျက်နှာ)၊ `Recovered` (ဖျက်ထားသော စာမျက်နှာ) သို့မဟုတ် `NotFound`။ |
| `Confidence`, `RecoveredFrom`, `LostColumns` | ဖျက်ထားသော entry တစ်ခုကို ပြန်လည်ရယူခဲ့ပုံ။ [ဖျက်ထားသော entry များ](#deleted-entries) ကို ကြည့်ပါ။ |

### Download များ { #downloads }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `StartTime`, `EndTime` | Download စတင်ခဲ့ပြီး ပြီးဆုံးခဲ့သည့်အချိန် (UTC)။ |
| `URL`, `RedirectChain` | ဖိုင် လာရာနေရာနှင့် လမ်းတစ်လျှောက်ရှိ URL တိုင်း (Chromium၊ CSV တွင် ၎င်းတို့ကြား ` -> `)။ |
| `TargetPath`, `CurrentPath` | ဖိုင်ကို သိမ်းဆည်းခဲ့သည့်နေရာ (Firefox တွင် `file://` URI) နှင့် download လုပ်နေစဉ် ရေးသားခဲ့သည့်နေရာ။ |
| `ReceivedBytes`, `TotalBytes` | လက်ခံရရှိခဲ့သော byte များနှင့် ဖိုင်၏ size။ |
| `State` | `Complete`၊ `InProgress`၊ `Cancelled`၊ `Interrupted`၊ `Paused`၊ `Blocked` သို့မဟုတ် `Dirty` (malware အဖြစ် ပိတ်ဆို့ထားသည်)။ |
| `DangerType` | ဖိုင်အပေါ် Chromium ၏ ဆုံးဖြတ်ချက်: 0 အန္တရာယ်မရှိ၊ 1 အန္တရာယ်ရှိသော file type၊ 2 အန္တရာယ်ရှိသော URL၊ 3 အန္တရာယ်ရှိသော content၊ 4 အန္တရာယ်ရှိနိုင်သည်၊ 5 အသုံးနည်းသည် (uncommon)၊ 6 အသုံးပြုသူက ထိန်းထားသည်၊ 7 အန္တရာယ်ရှိသော host၊ 8 မလိုလားအပ်နိုင်သည် (potentially unwanted)၊ … |
| `InterruptReason` | ကြားဖြတ်ရပ်တန့်သွားသော download ရပ်သွားရသည့် အကြောင်းရင်း (Chromium၊ မရှိလျှင် 0)။ |
| `Referrer`, `TabURL` | ညွှန်းဆိုသော စာမျက်နှာနှင့် download ကို စတင်ခဲ့သော tab ရှိ စာမျက်နှာ။ |
| `MimeType`, `Opened` | Server က ပေးပို့ခဲ့သည့်အတိုင်း ဖိုင်၏ type နှင့် ၎င်းကို ဘရောက်ဇာမှ ဖွင့်ခဲ့ခြင်း ရှိမရှိ (Chromium)။ |
| `FileDeletedByUser` | အသုံးပြုသူက download လုပ်ထားသော ဖိုင်ကို ဘရောက်ဇာ၏ download စာရင်းမှတစ်ဆင့် disk ပေါ်မှ ဖျက်ခဲ့ခြင်း ရှိမရှိ (Firefox)။ |
| `DownloadID` | Download ၏ row id။ ပြန်လည်ရယူထားသော record တွင် ပျောက်ဆုံးသွားပါက ဗလာ ဖြစ်သည်။ |

### Cookie များ { #cookies }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `Created`, `LastAccessed`, `Expires` | Cookie ကို သတ်မှတ်ခဲ့သည့်အချိန်၊ နောက်ဆုံး ပေးပို့ခဲ့သည့်အချိန်နှင့် သက်တမ်းကုန်မည့်အချိန် (UTC၊ session cookie အတွက် `Expires` ဗလာ)။ |
| `Host`, `Name`, `Path` | Cookie သက်ဆိုင်သော site၊ ၎င်း၏ အမည်နှင့် path။ |
| `Value` | သိမ်းဆည်းထားသည့်အတိုင်း value: Chromium က encrypt လုပ်ထားလျှင် ဗလာ။ |
| `Secure`, `HttpOnly`, `Persistent` | HTTPS ဖြင့်သာ ပေးပို့သည်၊ script များမှ ဝှက်ထားသည်၊ session များကြား ထိန်းထားသည် (Chromium)။ |
| `GoogleAnalytics` | Google Analytics cookie (`__utma`၊ `__utmb`၊ `__utmt`၊ `__utmz`) ၏ decode လုပ်ထားသော field များ: visit အချိန်များ၊ session များ၊ campaign source နှင့် search term များ။ |
| `RowID` | Cookie ၏ row id။ |

### Form history { #form-history }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `FirstUsed`, `LastUsed` | Value ကို ပထမဆုံးနှင့် နောက်ဆုံး ထည့်သွင်းခဲ့သည့်အချိန် (UTC)။ |
| `RemovedTime` | Firefox removal record အရ entry ကို ဖယ်ရှားခဲ့သည့်အချိန် (UTC)။ |
| `FieldName`, `Value` | Form field (`email`၊ `q`၊ Firefox ၏ search bar: `searchbar-history`) နှင့် ရိုက်ထည့်ခဲ့သည့်အရာ။ |
| `Count` | Value ကို ထည့်သွင်းခဲ့သည့် အကြိမ်အရေအတွက်။ |
| `GUID` | Removal record များက ညွှန်းဆိုသော entry ၏ Firefox identifier။ |
| `Sources` | Firefox က entry ကို သိမ်းဆည်းခဲ့သည့် ရင်းမြစ်၊ ၎င်းကို ထိန်းသိမ်းသော ဗားရှင်းများတွင် (CSV တွင် ဒေါင်လိုက်မျဉ်းဖြင့် ခွဲထားသည်)။ |
| `RowID` | Entry ၏ row ID။ ပြန်လည်ရယူရာတွင် ပျောက်ဆုံးလျှင် ဗလာ/null ဖြစ်သည်။ အမှန်တကယ် မှတ်တမ်းတင်ထားသော သုညကို `0` အဖြစ် ထားသည်။ |

## အလိုအလျောက် ခွဲခြမ်းစိတ်ဖြာခြင်း { #automatic-parsing }

`auto-parse` သည် table များကို `BrowserActivity` ထဲသို့ `Browser-History.csv`၊ `Browser-Downloads.csv`၊
`Browser-Cookies.csv` နှင့် `Browser-FormHistory.csv` အဖြစ် (`-t json` သို့မဟုတ် `-t jsonl` ဖြင့်ဆိုလျှင်
`Browser.json` သို့မဟုတ် `Browser.jsonl` ဖိုင်တစ်ခုတည်း) `BrowserActivity/Browser-logs.txt` နှင့်အတူ ရေးသားပါသည်။
ပေါင်းစပ်ထားသော `-j` timeline တွင် row တိုင်းကို `Comp` နှင့် `Artifact` identifier `browser` တို့နှင့်အတူ
ထည့်သွင်းပါသည်။ `-Z` သည် `VisitTime`၊ `LastVisitTime`၊ `StartTime`၊ `EndTime`၊ `Created`၊ `LastAccessed`၊ `Expires`၊
`FirstUsed`၊ `LastUsed` နှင့် `RemovedTime` တို့ကို ပြောင်းလဲပေးပါသည်။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Chromium — page transition types](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — download danger types](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places database](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — write-ahead logging](https://www.sqlite.org/wal.html)
- [Plaso — browser SQLite parsers](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
