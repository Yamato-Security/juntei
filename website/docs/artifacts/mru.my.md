# နောက်ဆုံးအသုံးပြုထားသော စာရင်းများ (RunMRU, TypedPaths, WordWheelQuery, TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## ဤစာရင်းများ ဆိုသည်မှာ { #what-the-lists-are }

Windows နှင့် ၎င်း၏ ပရိုဂရမ်များသည် အသုံးပြုသူ နောက်ဆုံးရိုက်ထည့်ခဲ့သော သို့မဟုတ် ဖွင့်ခဲ့သော အရာများ၏ စာရင်းတိုများကို
အသုံးပြုသူ၏ `NTUSER.DAT` တွင် သိမ်းထားသည်-

- **RunMRU:** Run dialog (Win+R) တွင် ရိုက်ထည့်ခဲ့သော ညွှန်ကြားချက်များ- `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`။
- **TypedPaths:** Explorer ၏ address bar တွင် ရိုက်ထည့်ခဲ့သော path များ။
- **WordWheelQuery:** Explorer ၏ ရှာဖွေရေးအကွက်တွင် ရှာဖွေခဲ့သော စကားလုံးများ။
- **TypedURLs:** Internet Explorer ၏ address bar တွင် ရိုက်ထည့်ခဲ့သော လိပ်စာများနှင့် Windows 8 မှစ၍
  တစ်ခုချင်းစီကို ရိုက်ထည့်ခဲ့သည့်အချိန်ကို ပြသော `TypedURLsTime`။
- **Paint နှင့် WordPad:** တစ်ခုချင်းစီက နောက်ဆုံးဖွင့်ခဲ့သော ဖိုင်များ (`Recent File List`)။
- **ACMru (Windows XP):** Search Assistant ၏ ရှာဖွေမှုအမျိုးအစားအလိုက် ရှာဖွေစကားလုံးများ။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> အသုံးပြုသူသည် ဤညွှန်ကြားချက်၊ path၊
ရှာဖွေစကားလုံး သို့မဟုတ် လိပ်စာကို ရိုက်ထည့်ခဲ့သည်၊ သို့မဟုတ် ပရိုဂရမ်က ဤဖိုင်ကို ဖွင့်ခဲ့သည်။ စာရင်းတစ်ခုစီ၏
နောက်ဆုံး entry ကိုသာ ၎င်း၏ key ၏ last-write အချိန်ဖြင့် ရက်စွဲသတ်မှတ်နိုင်ပြီး ရိုက်ထည့်ခဲ့သော လိပ်စာများကိုမူ
`TypedURLsTime` က ရက်စွဲသတ်မှတ်ပေးသည်။

## တည်နေရာ { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

`ACMru` ၏ subkey များသည် ရှာဖွေမှုအမျိုးအစားများ ဖြစ်သည်- `5603` ဖိုင်နှင့် ဖိုင်တွဲအမည်များ၊ `5604` ဖိုင်များအတွင်းရှိ
စကားလုံး သို့မဟုတ် စကားစုများ၊ `5647` ကွန်ပျူတာများနှင့် လူများ၊ `5001` အင်တာနက်။

Profile တိုင်းအတွက် `NTUSER.DAT` ကို ၎င်း၏ transaction log များနှင့်အတူ စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **စာရင်းတစ်ခုလျှင် အချိန်တစ်ခုသာ။** Key ၏ last-write အချိန်သည် ၎င်း၏ စာရင်းပြောင်းလဲသည့်အခါ ပြောင်းလဲသောကြောင့်
  နောက်ဆုံး entry ကိုသာ ရက်စွဲသတ်မှတ်ပေးသည် (`IsMostRecent`, `LastUsedUTC`)။ ကျန် entry များသည် အစီအစဉ်အတိုင်း
  ရှိသော်လည်း ရက်စွဲမရှိပါ။ `TypedURLsTime` က ရိုက်ထည့်ခဲ့သော လိပ်စာတစ်ခုစီကို ရက်စွဲသတ်မှတ်ပေးသည် (`TypedTimeUTC`)။
- **ရိုက်ထည့်ခဲ့ခြင်းသာ၊ run ခဲ့ခြင်း မဟုတ်။** RunMRU သည် Run dialog တွင် ရိုက်ထည့်ခဲ့သည့်အရာကို အောင်မြင်စွာ run ခဲ့သည်
  ဖြစ်စေ မဖြစ်စေ မှတ်တမ်းတင်သည်။ Windows သည် ညွှန်ကြားချက်တစ်ခုစီကို နောက်ဆက်တွဲ `\1` ဖြင့် သိမ်းဆည်းပြီး Juntei က
  ၎င်းကို ဖယ်ရှားပေးသည်။ Prefetch၊ BAM၊ UserAssist နှင့် process ဖန်တီးမှု event များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။
- **စာရင်းများ တိုတောင်းသည်။** တစ်ခုစီသည် ၎င်း၏ နောက်ဆုံး entry များကိုသာ သိမ်းထားပြီး privacy ဆက်တင်များနှင့်
  cleanup tool များက ၎င်းတို့ကို ရှင်းလင်းပစ်သည်။ ဖျက်ထားသော value များအတွက် `parse-registry read --recover-deleted`
  ကို အသုံးပြုပါ။
- **စကားလုံးများသည် ရိုက်ထည့်ခဲ့သည့်အတိုင်း ဖြစ်သည်။** WordWheelQuery နှင့် ACMru သည် ရှာတွေ့ခဲ့သည့်အရာကို မဟုတ်ဘဲ
  ရိုက်ထည့်ခဲ့သော စာသားကို သိမ်းထားသည်။
- **အစီအစဉ်ကို စာရင်းမှ ရယူသည်။** RunMRU ၏ `MRUList` နှင့် WordWheelQuery ၏ `MRUListEx` က အစီအစဉ်ကို
  ပေးသည်။ ကျန်စာရင်းများအတွက်မူ value ၏ နံပါတ်က ပေးသည် (`url1`, `File1` နှင့် `000` တို့သည် နောက်ဆုံးဖြစ်သည်)။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

အကျဉ်းချုပ်တွင် စာရင်းအလိုက် entry များနှင့် နောက်ဆုံးရက်စွဲပါသော entry များ (ရိုက်ထည့်ခဲ့သော လိပ်စာများနှင့်
စာရင်းတစ်ခုစီ၏ နောက်ဆုံး entry) ကို အသစ်ဆုံးမှစ၍ ဖော်ပြသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host၊ profile ပိုင်ရှင် (path သို့မဟုတ် shell ဖိုင်တွဲများမှ)၊ စနစ်၏ `ProfileList` မှ SID နှင့် hive။ |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad` သို့မဟုတ် `ACMru`။ |
| `Subkey` | `ACMru` ၏ subkey- ရှာဖွေမှုအမျိုးအစား (`5603`, `5604`, `5647`, `5001`)။ |
| `MRU-Position`, `IsMostRecent` | စာရင်းထဲရှိ နေရာ (0 သည် နောက်ဆုံးဖြစ်သည်)။ စာရင်းက value ကို အမည်မဖော်ပါက ဗလာဖြစ်သည်။ |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | ညွှန်ကြားချက် (၎င်း၏ `\1` မပါ)၊ path၊ ရှာဖွေစကားလုံး၊ လိပ်စာ သို့မဟုတ် ဖိုင်။ |
| `TypedTimeUTC` | `TypedURLsTime` မှ လိပ်စာကို ရိုက်ထည့်ခဲ့သည့်အချိန်။ |
| `LastUsedUTC` | Key ၏ last-write အချိန်၊ နောက်ဆုံး entry အတွက်သာ။ |
| `KeyPath`, `KeyLastWrittenUTC` | Key နှင့် ၎င်း၏ နောက်ဆုံးရေးသားချိန်- row တိုင်း၏ အချိန်မဟုတ်ဘဲ ၎င်း၏ နောက်ဆုံး entry ၏ အချိန်။ |
| `DataRaw` | Decode မလုပ်နိုင်သည့်အခါ value ၏ byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | စာရင်းက အမည်ဖော်သော်လည်း key တွင်မရှိသော value များ၊ စာရင်းက အမည်မဖော်သော value များ၊ အမည်မှားနေသော value များ၊ အမျိုးအစားမှားများ၊ ဖြတ်တောက်ခံရသော စကားလုံးများ၊ FILETIME မဟုတ်သော အချိန်များ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. အသုံးပြုသူ ရိုက်ထည့်ခဲ့သော ညွှန်ကြားချက်များ၊ script များ၊ network share များနှင့် စီမံခန့်ခွဲရေး tool များအတွက် RunMRU ကို ဖတ်ပါ။
2. အသုံးပြုသူ ရှာဖွေခဲ့သော ဖိုင်တွဲများနှင့် ဖိုင်များအတွက် TypedPaths နှင့် WordWheelQuery ကို ဖတ်ပါ-
   credential များ၊ export များ၊ archive များ၊ removable နှင့် network တည်နေရာများ။
3. ရိုက်ထည့်ခဲ့သော လိပ်စာများကို `TypedTimeUTC` ဖြင့် timeline ပေါ်တွင် ထားပါ။
4. Prefetch၊ BAM၊ UserAssist၊ ShellBags၊ LNK ဖိုင်များ၊ jump list များနှင့် browser မှတ်တမ်းတို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [libyal winreg-kb: နောက်ဆုံးအသုံးပြုထားသော (MRU) key များ](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
