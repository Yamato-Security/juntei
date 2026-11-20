# Windows Search အညွှန်း { #windows-search-index }

## အညွှန်းတွင် မှတ်တမ်းတင်ထားသည့်အရာ { #what-the-index-records }

Windows Search (`WSearch` ဝန်ဆောင်မှု) သည် အသုံးပြုသူ profile များ၊ Start menu၊ Outlook ၏ mailbox များ၊ Internet
Explorer ၏ history၊ offline ဖိုင်များနှင့် ၎င်း၏ scope အတွင်းရှိ အခြားအရာအားလုံးကို crawl လုပ်ပြီး ဖတ်ခဲ့သည့်အရာများကို
ESE database တစ်ခုဖြစ်သော `Windows.edb` တွင် သိမ်းဆည်းသည်။ ၎င်း၏ အစိတ်အပိုင်း နှစ်ခုသည် စုံစမ်းစစ်ဆေးမှုအတွက်
အရေးကြီးသည်-

- **property store** (`SystemIndex_PropertyStore`၊ Windows 7 တွင် `SystemIndex_0A`)- index လုပ်ထားသော item
  တစ်ခုလျှင် row တစ်ခုစီဖြင့်- ၎င်း၏ path၊ အမည်၊ အမျိုးအစားနှင့် အရွယ်အစား၊ ၎င်း၏ ပိုင်ရှင်၊ indexer က ဖိုင်မှ
  ကူးယူခဲ့သော ဖန်တီးချိန်၊ ပြင်ဆင်ချိန်နှင့် ဝင်ရောက်ချိန်များ၊ indexer က ၎င်းကို ဖတ်ခဲ့သည့်အချိန်၊ စာရွက်စာတမ်း၏
  title နှင့် author၊ ၎င်း၏ စာသား အစပိုင်း၊ ကွန်ပျူတာ အမည်နှင့် အခြား property ရာပေါင်းများစွာ၊
- **gatherer** table များ (`SystemIndex_Gthr`၊ `SystemIndex_GthrPth`)- item တစ်ခုစီအတွက် crawler ၏ ကိုယ်ပိုင်
  record ဖြင့်- ၎င်း၏ URL path၊ queue ထဲ ထည့်ခဲ့ချိန်က ၎င်း၏ ပြင်ဆင်ချိန်၊ crawl နံပါတ်နှင့် flag များနှင့် ဖျက်မှု
  counter တစ်ခု။

ဖိုင်၊ message သို့မဟုတ် စာမျက်နှာ ပျောက်သွားပြီးနောက်တွင်လည်း crawler က ဖျက်မှုကို သတိပြုမိသည်အထိ item သည်
အညွှန်းထဲတွင် ကျန်ရှိနေသည်။ ထို့ကြောင့် အညွှန်းသည် ယခု မရှိတော့သော ဖိုင်များကို ၎င်းတို့ index လုပ်ခံရချိန်က
ရှိခဲ့သည့်အတိုင်း ၎င်းတို့၏ metadata နှင့် စာသား ကောက်နုတ်ချက်တို့နှင့်အတူ အမည်ဖော်ပြသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> indexer သည် item ကို မှတ်တမ်းတင်ထားသော path တွင်၊
မှတ်တမ်းတင်ထားသော metadata ဖြင့်၊ `GatherTime` အချိန်၌၊ အမည်ဖော်ပြထားသော ကွန်ပျူတာပေါ်တွင် ဖတ်ခဲ့သည်။ အညွှန်းသည်
ဖိုင် တည်ရှိခဲ့ကြောင်းနှင့် ၎င်းတွင် မည်သည့်အရာ ပါဝင်ခဲ့ကြောင်းကို ပြသသည်။ အသုံးပြုသူတစ်ဦးက ၎င်းကို ဖွင့်ခဲ့ကြောင်း
မပြပါ။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Windows 7 မှ Windows 11 21H2 အထိနှင့် တူညီသော နှစ်များ၏ Windows Server ထုတ်ဝေမှုများသည် အညွှန်းကို `Windows.edb`
တွင် သိမ်းဆည်းသည်။ Windows 11 22H2 နှင့် နောက်ပိုင်းတို့သည် ၎င်းကို တူညီသော ဖိုင်တွဲအတွင်းရှိ SQLite database များ
(`Windows.db`၊ `Windows-gather.db`၊ `Windows-usn.db`) တွင် သိမ်းဆည်းသည်။ Juntei သည် ၎င်းတို့ကို ရှာဖွေတွေ့ရှိပြီး
အမည်ဖော်ပြကာ ပံ့ပိုးမထားသော ပိုသစ်သည့် format အဖြစ် ဖော်ပြသည်။

`Applications\Windows` ဖိုင်တွဲ တစ်ခုလုံးကို disk image၊ volume shadow copy မှ သို့မဟုတ် `WSearch` ဝန်ဆောင်မှုကို
ရပ်တန့်ပြီးနောက် စုဆောင်းပါ- ဝန်ဆောင်မှု လည်ပတ်နေစဉ် database ကို ဖွင့်ထားပြီး ၎င်း၏ နောက်ဆုံး ပြောင်းလဲမှုများသည်
Juntei က replay မလုပ်သော `MSS*.log` transaction log များတွင်သာ ရှိသည်။ လည်ပတ်နေသော စနစ်မှ ကူးယူထားသော
အညွှန်းသည် dirty အဖြစ် ကျန်နေသည်။ Output က ထိုသို့ ဖော်ပြပြီး disk ပေါ်တွင် ရှိသည့်အရာကို ဖတ်သည်။ Directory
input နှင့် `auto-parse` သည် မည်သည့်အဆင့်တွင်မဆို ရှိသော `Windows.edb` တိုင်းကို ဖတ်သည်။ ဖိုင်တစ်ခုတည်းအတွက် `-f`
ကို အသုံးပြုပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- မပြောင်းလဲသော သက်သေမိတ္တူကို ခွဲခြမ်းပါ။ ထိရောက်စွာ ဖတ်ရှုရန် Juntei သည် ဖိုင်ကို မှတ်ဉာဏ်ထဲသို့ ချိတ်ဆက်ထားသဖြင့် ဖတ်ရှုနေစဉ် ဒေတာဘေ့စ်ကို မပြင်ဆင်ပါနှင့်၊ မဖြတ်တောက်ပါနှင့်။
- `GatherTime` သည် indexer က item ကို ဖတ်ခဲ့သည့်အချိန် ဖြစ်ပြီး အသုံးပြုသူက ၎င်းကို ဖန်တီး၊ ပြောင်းလဲ သို့မဟုတ်
  ဖွင့်ခဲ့သည့်အချိန် မဟုတ်ပါ။ Index လုပ်ခြင်းသည် နောက်ခံတွင် လည်ပတ်ပြီး ပြောင်းလဲမှုတစ်ခု ပြီးနောက် မိနစ်ပိုင်းမှ
  နာရီပိုင်းအထိ ကြာမြင့်တတ်ကာ ကွန်ပျူတာ အလုပ်များနေစဉ် သို့မဟုတ် ဘက်ထရီ သုံးနေစဉ် ခေတ္တရပ်နားသည်။
- `DateCreated`၊ `DateModified` နှင့် `DateAccessed` တို့သည် indexer က ဖိုင်မှ ကူးယူထားသော မိတ္တူများ ဖြစ်သည်။
  Item ကို ထပ်မံ index လုပ်သောအခါ ၎င်းတို့ကို အသစ်ပြန်ရေးသောကြောင့် ၎င်းတို့သည် `GatherTime` အချိန်က ဖိုင်ကို
  ဖော်ပြသည်။
- `ItemPath` သည် ပြသရန် path ဖြစ်သည်။ `ItemURL` နှင့် `GatherPath` တို့သည် crawler ၏ URL များ ဖြစ်သည်-
  ဖိုင်များအတွက် `file:` (Windows 8 နှင့် နောက်ပိုင်းတွင် volume ကို `[GUID]` အဖြစ်)၊ Internet Explorer ၏ history
  အတွက် `iehistory://{SID}/`၊ Outlook item များအတွက် `mapi://`၊ offline ဖိုင်များအတွက် `csc://{SID}/` နှင့်
  Windows app များ၏ အကြောင်းအရာအတွက် `winrt://{SID}/`။
- `User` သည် item ၏ path တွင် အမည်ဖော်ပြထားသော profile (`C:\Users\<name>\…`) ဖြစ်သည်။ ၎င်းသည် item
  ရှိခဲ့သည့်နေရာ ဖြစ်ပြီး ၎င်းကို ဖန်တီးခဲ့သူ မဟုတ်ပါ။ `FileOwner` သည် indexer မြင်ခဲ့သည့်အတိုင်း ဖိုင်၏ ပိုင်ရှင်
  ဖြစ်သည်။
- `AutoSummary` သည် item ၏ စာသား အစပိုင်း ဖြစ်သည်။ ၎င်းသည် နောက်ပိုင်းတွင် ဖျက်ထား သို့မဟုတ် encrypt
  လုပ်ထားသော စာရွက်စာတမ်းတစ်ခုကို ကိုးကားနိုင်ပြီး ၎င်းသည် ဖြတ်တောက်ထားသော ကောက်နုတ်ချက် ဖြစ်ကာ စာရွက်စာတမ်း
  မဟုတ်ပါ။
- Dirty database တစ်ခုတွင် ၎င်း၏ branch page များက မရောက်ရှိသေးသော page များ ပါနိုင်သည်။ Juntei သည် leaf
  page များ၏ ကိုယ်ပိုင် chain ကို လိုက်ပြီး ထိုနည်းဖြင့် မည်မျှ ဖတ်ခဲ့သည်ကို summary တွင် ဖော်ပြသည်။ ၎င်း၏
  နောက်ဆုံး ပြောင်းလဲမှုများသည် transaction log များတွင် ဆက်ရှိနေသေးသည်။
- Property-store row မပါသော gatherer table များရှိ item (`InTables` သည် `Gthr`) ကို queue ထဲ ထည့်ခဲ့ သို့မဟုတ်
  crawl လုပ်ခဲ့ပြီး ၎င်း၏ property များကို မရေးရသေး သို့မဟုတ် ဖယ်ရှားခဲ့ပြီး ဖြစ်သည်။ ၎င်း၏ path၊ အမည်နှင့်
  gatherer အချိန်များ ဆက်ရှိနေသေးသည်။
- Windows 7 သည် string များကို ၎င်း၏ ကိုယ်ပိုင် compress လုပ်ထားပြီး obfuscate လုပ်ထားသော encoding ဖြင့်
  သိမ်းဆည်းသည်။ Juntei သည် ၎င်းကို decode လုပ်သည်။ Decode မလုပ်နိုင်သော value သည် hexadecimal အတိုင်း
  ကျန်ရှိပြီး `Problems` တွင် မှတ်ချက် ပါသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

မူလဖိုင်ပုံစံမှာ CSV ဖြစ်သည်။ `-t csv`၊ `-t json` သို့မဟုတ် `-t jsonl` ပါပြီး `-o` မပါလျှင် record များကို stdout သို့
ရေးသည်။ ရွေးချယ်စရာနှစ်ခုလုံး မပါလျှင် အနှစ်ချုပ်သာ ပြသည်။ ရှိပြီးသား output ကို ထပ်ရေးရန် `-C` ထည့်ပါ။ `-C`
ပါသော်လည်း output ကို input ဖိုင်တစ်ခုခုပေါ်သို့ ဘယ်တော့မှ ထပ်မရေးပါ။

Juntei သည် index လုပ်ထားသော item တစ်ခုလျှင် record တစ်ခု ရေးပြီး property store နှင့် gatherer table များကို item
၏ identifier ဖြင့် join လုပ်သည်။ Summary သည် database များကို schema အလိုက်၊ dirty ဖြစ်နေသော database များ၊
table နှစ်ခုလုံးတွင် ရှိသော item များ၊ gatherer တွင်သာ ရှိသော item များနှင့် property store တွင်သာ ရှိသော item
များ၊ ဖိုင်တွဲများ၊ ဖျက်ထားသည်ဟု အမှတ်အသားပြုထားသော item များ၊ store တစ်ခုစီရှိ item များ၊ သိမ်းဆည်းထားသော
အချိန် မပါသော item များနှင့် problem များကို ရေတွက်သည်။

ပျက်စီးမှုက run ကို မရပ်တန့်စေပါ။ အဆုံးမတိုင်မီ ပြတ်တောက်နေသော database သည် ၎င်း၏ ကျန်ရှိသော page များတွင်
ပါဝင်သော item များကို ထိန်းသိမ်းထားပြီး summary က မဖတ်နိုင်ခဲ့သည့်အရာကို ဖော်ပြသည်။ Windows Search အညွှန်း
မဟုတ်သော `Windows.edb` ဟု အမည်ပေးထားသည့် ဖိုင်ကို ဖော်ပြပြီး ကျော်သွားသည်။ အခြား database များကို ဆက်လက်
ဖတ်သည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `DocumentID` | အညွှန်းအတွင်း item ၏ identifier- property store ၏ `WorkID` (Windows 7 တွင် `DocID`) နှင့် gatherer row ၏ `DocumentID`။ |
| `InTables` | `Gthr+PropertyStore`၊ `Gthr` (property မပါသော queue ထဲ ထည့်ထား သို့မဟုတ် crawl လုပ်ထားသော item) သို့မဟုတ် `PropertyStore`။ |
| `ItemPath` | `System_ItemPathDisplay`- Windows က ပြသသည့်အတိုင်း path။ Property store တွင် row မရှိလျှင် ဗလာ။ |
| `ItemURL` | `System_ItemUrl`- item အတွက် crawler ၏ URL (`file:`၊ `iehistory://`၊ `mapi://`၊ `csc://`၊ `winrt://`)။ |
| `GatherPath` | Scope tree နှင့် row ၏ ဖိုင်အမည်မှ ပေါင်းစပ်ထားသော gatherer ၏ path။ |
| `Name` | `System_ItemName` သို့မဟုတ် gatherer row ၏ ဖိုင်အမည်။ |
| `ItemType`, `ItemTypeText` | `System_ItemType` (extension သို့မဟုတ် `Directory`) နှင့် ၎င်း၏ ပြသရန် အမည်။ |
| `Kind` | `System_Kind`- `document`၊ `folder`၊ `link`၊ `program`၊ `email`၊ `picture`၊ …။ အများအပြား ရှိပါက `\|` ဖြင့် ပေါင်းစပ်ထားသည်။ |
| `MimeType` | `System_MIMEType`။ |
| `Store` | `System_Search_Store`- `file`၊ `mapi` (Outlook)၊ `iehistory`၊ `csc` (offline ဖိုင်များ) သို့မဟုတ် `winrt` (Windows app များ)။ |
| `IsFolder`, `IsDeleted` | သိမ်းဆည်းထားသည့်အတိုင်း `System_IsFolder` နှင့် `System_IsDeleted`။ |
| `DeletedCount` | Gatherer row ၏ ဖျက်မှု counter။ |
| `Size` | `System_Size` (byte ဖြင့်)။ Placeholder ပါသော ဖိုင်တွဲများအတွက် ဗလာ။ |
| `FileAttributes` | `System_FileAttributes` ကို နံပါတ်အဖြစ် (16 directory၊ 32 archive၊ 2 hidden၊ …)။ |
| `FileOwner` | Indexer ဖတ်ခဲ့သည့်အတိုင်း `System_FileOwner` (`DOMAIN\user`)။ |
| `DateCreated`, `DateModified`, `DateAccessed` | Indexer ကူးယူခဲ့သည့်အတိုင်း ဖိုင်၏ အချိန်များ (UTC)။ |
| `ItemDate` | `System_ItemDate`- message တစ်ခု၏ ပေးပို့ချိန် သို့မဟုတ် ဓာတ်ပုံတစ်ပုံ၏ ရိုက်ကူးသည့်ရက်စွဲကဲ့သို့ item ၏ ကိုယ်ပိုင် ရက်စွဲ။ ဖိုင်တစ်ခုအတွက် များသောအားဖြင့် စက္ကန့်အထိ ၎င်း၏ ပြင်ဆင်ချိန်။ |
| `GatherTime` | `System_Search_GatherTime`- indexer က item ကို ဖတ်ခဲ့သည့်အချိန် (UTC)။ |
| `GatherLastModified` | Gatherer row ၏ `LastModified`- crawler က ၎င်းကို queue ထဲ ထည့်ခဲ့ချိန်က item ၏ ပြင်ဆင်ချိန် (UTC)။ |
| `Title`, `Author` | `System_Title` နှင့် `System_Author`။ |
| `AutoSummary` | `System_Search_AutoSummary`- item ၏ စာသား အစပိုင်း။ |
| `CompName` | `System_ComputerName`- အညွှန်း ပိုင်ဆိုင်သော ကွန်ပျူတာ။ |
| `TargetURL`, `DateVisited` | Internet Explorer history item တစ်ခု၏ URL နှင့် ကြည့်ရှုချိန် (`System_Link_TargetUrl`၊ `System_Link_DateVisited`)။ |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | Windows 10 activity history item တစ်ခု (`System_Activity_*`၊ `System_ActivityHistory_*`)။ |
| `User`, `UserSource` | Item ၏ path တွင် အမည်ဖော်ပြထားသော profile နှင့် ၎င်းကို မည်သည့် path မှ ဖတ်ခဲ့သည် (`ItemPath`၊ `ItemURL` သို့မဟုတ် `GatherPath`)။ Profile ပြင်ပတွင် ဗလာ။ |
| `ScopeID`, `SDID` | Gatherer row ၏ scope နှင့် security descriptor identifier များ။ |
| `CrawlNumber` | Gatherer row ၏ `CrawlNumberCrawled`။ |
| `GatherFlags` | Gatherer row ၏ `TransactionFlags`။ |
| `Priority` | Queue ထဲတွင် ရှိနေသေးသော item ၏ crawl priority။ Item ကို crawl လုပ်ပြီးသည်နှင့် ESE က ၎င်းကို NULL အဖြစ် အမှတ်အသားပြုသောကြောင့် item အများစုအတွက် ဗလာ ဖြစ်သည်။ |
| `PropertyCount` | Item အတွက် property store တွင် ပါဝင်သော property အရေအတွက်။ |
| `OtherProperties` | အခြား property တိုင်း၊ `; ` ဖြင့် ခြားထားသော `Name=Value` အဖြစ်။ |
| `SourceFile` | ဖတ်ခဲ့သော database၊ input directory မှ relative path သို့မဟုတ် `-f` ဖြင့်ဆိုလျှင် ၎င်း၏ အမည်။ |
| `Problems` | Decode မလုပ်နိုင်ခဲ့သော value များ၊ အချိန်မဟုတ်သော အချိန်များ၊ ထပ်နေသော row များနှင့် ပျက်စီးမှု။ ဖုံးကွယ်မထားပါနှင့်။ |

JSON နှင့် JSONL သည် type ပါသော value များဖြင့် တူညီသော field များကို ထိန်းသိမ်းသည်- နံပါတ်များသည် နံပါတ်အဖြစ် ကျန်ပြီး
ပျောက်နေသော value များသည် `null` ဖြစ်သည်။ `OtherProperties` အစား ၎င်းတို့တွင် `Properties` ပါသည်- item ၏
property တိုင်းကို ၎င်း၏ အမည်၊ နံပါတ် identifier၊ column type နှင့် value တို့ဖြင့် ဖော်ပြပြီး အချိန်တစ်ခုအတွက် ၎င်းမှ
ဖတ်ရသော အချိန်၏ ဘေးတွင် ၎င်း၏ raw FILETIME tick count ကို ထည့်သွင်းသည်။ `Gather` တွင် gatherer row ၏ column
များ ပါသည်။

## Timeline အပြုအမူ { #timeline-behavior }

`auto-parse` သည် record များကို `FileExistence/WindowsSearch.csv` (သို့မဟုတ် `.json`/`.jsonl`) အဖြစ်
သိမ်းဆည်းပြီး ပေါင်းစပ်ထားသော `-j` timeline ထဲသို့ ထည့်သည်။ `super-timeline` သည် `DateCreated`၊ `DateModified`
နှင့် `DateAccessed` (Windows Search က မှတ်တမ်းတင်ထားသည့်အတိုင်း ဖိုင်၏ အချိန်များ) တို့မှ၊ `GatherTime` (item ကို
index လုပ်ခဲ့သည့်အချိန်) မှ၊ Internet Explorer history item တစ်ခု၏ `DateVisited` မှနှင့် activity တစ်ခု၏
`ActivityStartTime` နှင့် `ActivityEndTime` တို့မှ event များကို ဖန်တီးသည်။ `GatherLastModified` သည် property
store တွင် ပြင်ဆင်ချိန် မပါသော item အတွက်သာ event ဖြစ်လာသည်။ ဤအချိန်များ တစ်ခုမျှ မပါသော item သည် parse
လုပ်ထားသော output ထဲတွင် ကျန်ရှိပြီး အချိန်မပါသော item အဖြစ် ရေတွက်သည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. `Store` ကို `file` ဖြင့် filter လုပ်ပြီး `ItemPath`၊ `Name` နှင့် `AutoSummary` တို့တွင် အမှုနှင့် ဆိုင်သော
   ဖိုင်များ၊ အမည်များနှင့် စကားလုံးများကို ရှာဖွေပါ။ အညွှန်းသည် နောက်ပိုင်းတွင် ဖျက်ထားပြီးသော ဖိုင်များကို
   အမည်ဖော်ပြသည်။
2. `GatherTime` ကို မှတ်တမ်းတင်ထားသော metadata ဖြင့် ဖိုင် တည်ရှိခဲ့သည့် နောက်ဆုံးအချိန်အဖြစ်လည်းကောင်း၊
   `DateModified` ကို ထိုအချိန်က ၎င်း၏ ပြင်ဆင်ချိန်အဖြစ်လည်းကောင်း ဖတ်ပါ။
3. ဖိုင်တစ်ခုကို account တစ်ခုနှင့် ချိတ်ဆက်ရန် `FileOwner` နှင့် `User` ကို အသုံးပြုပြီး `$MFT`၊ USN Journal နှင့်
   `$Secure:$SDS` တို့ဖြင့် အတည်ပြုပါ။
4. E-mail (`Store` `mapi`) နှင့် Internet Explorer history (`iehistory`) အတွက် record အပြည့်အစုံ ရရှိရန် Outlook ၏
   ဖိုင်များနှင့် WebCache သို့ pivot လုပ်ပါ။
5. Host များအကြား အညွှန်းကို နှိုင်းယှဉ်ပါ- `CompName` သည် အညွှန်း ပိုင်ဆိုင်သော ကွန်ပျူတာကို အမည်ဖော်ပြပြီး image
   တစ်ခုမှ စုဆောင်းထားသော အညွှန်းသည် ထို image ပေါ်ရှိ ဖိုင်များကို အမည်ဖော်ပြသည်။

## စမ်းသပ်ထားသော ဗားရှင်းများ { #tested-versions }

စမ်းသပ်မှုများသည် အခြား tool များက သီးခြားလွတ်လပ်စွာ ဖတ်ရှုထားသော ရလဒ်များပါသည့် အများသုံး အညွှန်းများကို အသုံးပြုသည်-

- plaso ၏ Windows 7 အညွှန်း (`SystemIndex_0A`)၊ Windows 7 string encoding များ အပါအဝင် libyal ၏ `esedbexport`
  နှင့် တိုက်စစ်ထားသည်။
- DFIRArtifactMuseum ၏ Windows 10 အညွှန်း (dirty အဖြစ် ထားရှိ)၊ Stroz Friedberg ၏ SIDR နှင့် တိုက်စစ်ထားပြီး ၎င်း၏
  Windows 11 အညွှန်း (ESE format revision 200) ကို `esedbexport` နှင့် တိုက်စစ်ထားသည်။
- DFIRArtifactMuseum ၏ Windows 11 22H2 SQLite အညွှန်း၊ ပံ့ပိုးမထားသော ပိုသစ်သည့် format အဖြစ် ဖော်ပြသည်။
- အဆုံးမတိုင်မီ ပြတ်တောက်နေသော၊ ဗလာဖြစ်နေသော သို့မဟုတ် အခြားအမျိုးအစား database များ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft — Windows Search overview](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE database file format](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb — Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg — SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft — Extensible Storage Engine source](https://github.com/microsoft/Extensible-Storage-Engine)
