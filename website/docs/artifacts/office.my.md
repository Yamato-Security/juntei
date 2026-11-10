# Microsoft Office (မကြာသေးမီ ဖိုင်များ၊ ယုံကြည်ရသော စာရွက်စာတမ်းများ၊ Macro ဆက်တင်များ) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## Office က မှတ်တမ်းတင်သည့် အရာများ { #what-office-records }

Microsoft Office သည် အသုံးပြုသူတစ်ဦးချင်းစီအလိုက် `NTUSER.DAT` တွင် ဗားရှင်းနှင့် application တစ်ခုချင်းစီအလိုက်
အောက်ပါတို့ကို ထိန်းသိမ်းပါသည်။

- **မကြာသေးမီ ဖိုင်များနှင့် နေရာများ:** application တစ်ခု ဖွင့်ခဲ့သော စာရွက်စာတမ်းများဖြစ်သည့် `File MRU` နှင့်
  ၎င်းတို့ကို ဖွင့်ခဲ့သည့် ဖိုင်တွဲများဖြစ်သည့် `Place MRU` ဖြစ်ပြီး item တစ်ခုစီတွင် နောက်ဆုံးဖွင့်ခဲ့သည့် အချိန် ပါဝင်ပါသည်။
  Microsoft 365 သည် sign in ဝင်ထားသော account တစ်ခုချင်းစီအလိုက် စာရင်းတစ်ခုကိုလည်း ထိန်းသိမ်းပါသည် (`User MRU\<account>`)။
- **Trusted Documents:** အသုံးပြုသူက စာရွက်စာတမ်းတစ်ခုပေါ်တွင် *Enable Editing* သို့မဟုတ် *Enable Content* ကို
  နှိပ်သည့်အခါ Office သည် ၎င်းကို `TrustRecords` တွင် မှတ်သားပါသည်။ စာရွက်စာတမ်း၏ path၊ ၎င်းကို ယုံကြည်ခဲ့သည့်အချိန်နှင့်
  ၎င်း၏ macro များကို ဖွင့်ခဲ့/မဖွင့်ခဲ့ ဖြစ်ပါသည်။
- **Macro ဆက်တင်များ:** `VBAWarnings` (မည်သည့် macro များ လည်ပတ်မည်)၊ `BlockContentExecutionFromInternet`
  (Internet မှလာသော ဖိုင်များရှိ macro များကို ပိတ်ဆို့/မပိတ်ဆို့)၊ `AccessVBOM` (ပရိုဂရမ်များက VBA project များကို
  ပြောင်းလဲခွင့်ရှိ/မရှိ) နှင့် စာရွက်စာတမ်းများ၏ macro များကို မေးမြန်းခြင်းမရှိဘဲ လည်ပတ်စေသော ဖိုင်တွဲများဖြစ်သည့်
  **Trusted Locations** ဖြစ်ပါသည်။ Group Policy သည် `Software\Policies` အောက်တွင် အလားတူ သတ်မှတ်နိုင်ပါသည်။

အန္တရာယ်ရှိသော စာရွက်စာတမ်းတစ်ခုသည် များသောအားဖြင့် အသုံးပြုသူက ၎င်း၏ macro များကို ဖွင့်ပေးရန် လိုအပ်ပါသည်။
macro များကို ဖွင့်ခဲ့သော trust record တစ်ခုသည် စာရွက်စာတမ်း၊ အသုံးပြုသူနှင့် အချိန်တို့ကို ချိတ်ဆက်ပေးပါသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> အသုံးပြုသူသည် ဤစာရွက်စာတမ်းကို ဤ application တွင်
ဖွင့်ခဲ့ပါသည် (နောက်ဆုံး ဤအချိန်တွင်)။ အသုံးပြုသူသည် ဤစာရွက်စာတမ်း၏ editing သို့မဟုတ် macro များကို ဤအချိန်တွင်
ဖွင့်ခဲ့ပါသည်။ Office ကို macro များ ဤနည်းဖြင့် လည်ပတ်စေရန် ပြင်ဆင်ထားခဲ့ပါသည်။

## တည်နေရာ { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

ဗားရှင်းများ: `16.0` သည် Office 2016 နှင့် နောက်ပိုင်း (2019, 2021, 2024, Microsoft 365)၊ `15.0` သည် Office 2013၊
`14.0` သည် Office 2010၊ `12.0` သည် Office 2007 ဖြစ်ပါသည်။

Item တစ်ခုသည် `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx` ကဲ့သို့ ဖြစ်ပါသည်။ `T`
သည် နောက်ဆုံးဖွင့်ခဲ့သည့်အချိန် (hexadecimal ဖြင့် FILETIME) ဖြစ်ပြီး path သည် `*` နောက်တွင် လိုက်ပါသည်။ Trust record
တစ်ခုသည် 24 byte ဖြစ်ပါသည်။ စာရွက်စာတမ်းကို ယုံကြည်ခဲ့သည့် FILETIME နောက်တွင် နောက်ဆုံး 4 byte ၌ ၎င်း၏ flag များ
ပါဝင်ပြီး macro များကို ဖွင့်ခဲ့လျှင် `0x7FFFFFFF`၊ editing ကိုသာ ဖွင့်ခဲ့လျှင် `0x00000001` ဖြစ်ပါသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **အချိန်များ။** `LastOpenedUTC` သည် item တစ်ခုစီ၏ ကိုယ်ပိုင်အချိန် ဖြစ်ပါသည်။ `TrustedAtUTC` သည် အသုံးပြုသူက
  စာရွက်စာတမ်းကို ယုံကြည်ခဲ့သည့်အချိန်၊ *Enable Editing* သို့မဟုတ် *Enable Content* ကို နှိပ်ခဲ့သည့် အခိုက်အတန့် ဖြစ်ပြီး
  နောက်ဆုံးဖွင့်ခဲ့သည့်အချိန်ထက် စောနိုင်ပါသည်။
- **Macro များကို ဖွင့်ခဲ့ခြင်းဖြစ်ပြီး လည်ပတ်ခဲ့ခြင်း မဟုတ်ပါ။** `MacrosEnabled` သည် အသုံးပြုသူက စာရွက်စာတမ်း၏
  macro များကို ဖွင့်ခဲ့ကြောင်း ဖော်ပြပါသည်။ macro တစ်ခု လည်ပတ်ခဲ့ကြောင်း သို့မဟုတ် ၎င်းက ဘာလုပ်ခဲ့သည်ကို မသက်သေပြပါ။
  Process creation၊ Prefetch၊ PowerShell log များနှင့် စာရွက်စာတမ်းကိုယ်တိုင်နှင့် ဆက်စပ်စစ်ဆေးပါ။
- **Office သိမ်းဆည်းသည့်အတိုင်း path များ။** Trust record များသည် စာရွက်စာတမ်းကို `%USERPROFILE%`၊ forward slash၊
  `file:///` URL နှင့် URL encoding (`%20`) ကို အသုံးပြုနိုင်သော path ဖြင့် ဖော်ပြပါသည်။ Cloud စာရွက်စာတမ်းများ၏
  MRU item များသည် URL များ ဖြစ်ပါသည်။
- **မူလတန်ဖိုးများနှင့် ထိရောက်သော policy။** Office သည် မူလ trusted location များ (`Templates`,
  `XLSTART`, `STARTUP`, …) ကို ဖန်တီးပါသည်။ Location ၏ `Date` သည် အသုံးဝင်သော သဲလွန်စဖြစ်သော်လည်း မည်သူဖန်တီးခဲ့သည်ကို
  သက်သေမပြပါ။ Administrator များ၊ script များ၊ policy များနှင့် ကူးယူထားသော registry ဆက်တင်များကလည်း ၎င်းကို
  ဖြည့်နိုင်ပါသည်။ `VBAWarnings` မရှိခြင်း တစ်ခုတည်းဖြင့် ထိရောက်သော macro policy ကို အတည်မပြုနိုင်ပါ။ Office ဗားရှင်း၊
  Internet ဖိုင် အမှတ်အသားပြုခြင်းနှင့် အခြား စီမံခန့်ခွဲမှု ဆက်တင်များလည်း အရေးပါပါသည်။
  [Internet macro ပိတ်ဆို့ခြင်းအကြောင်း Microsoft ၏ ရှင်းလင်းချက်](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked)။
- **ဖျက်ထားသော စာရွက်စာတမ်းများ။** စာရင်းများသည် စာရွက်စာတမ်းများထက် ကြာရှည်စွာ ကျန်ရှိပါသည်။ မရှိတော့သော ဖိုင်ကို
  ဤနေရာတွင် ပြသနေနိုင်ပါသည်။ စာရင်းများသည် တိုပြီး Office ၏ *Clear unpinned items* နှင့် cleanup tool များက ရှင်းလင်းပါသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

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
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

အနှစ်ချုပ်တွင် macro များကို ဖွင့်ခဲ့သော စာရွက်စာတမ်းများ၊ macro များကို ပိုမိုလွတ်လပ်စွာ လည်ပတ်စေသော ဆက်တင်များ
(`VBAWarnings` 1, `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1,
`AllowNetworkLocations` 1)၊ အသုံးပြုသူများ ထည့်ခဲ့သော သို့မဟုတ် policy က သတ်မှတ်ခဲ့သော trusted location များနှင့်
နောက်ဆုံးဖွင့်ခဲ့သော စာရွက်စာတမ်းများကို ဖော်ပြပါသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host၊ profile ပိုင်ရှင် (path သို့မဟုတ် shell folder များမှ)၊ စနစ်၏ `ProfileList` မှ SID နှင့် hive။ |
| `OfficeVersion`, `App` | ဗားရှင်း၏ key (`16.0`) နှင့် application (`Word`, `Excel`, …)။ |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security` သို့မဟုတ် `Trusted Locations`။ |
| `Account` | `User MRU` စာရင်း၏ Microsoft 365 account။ |
| `Policy` | ဆက်တင် သို့မဟုတ် location ကို policy က သတ်မှတ်ထားခြင်း ဟုတ်/မဟုတ်။ |
| `MRU-Position`, `IsMostRecent` | Item ၏ ၎င်းစာရင်းအတွင်း နေရာ (`Item 1` သည် 0၊ အသစ်ဆုံး)။ |
| `Path` | စာရွက်စာတမ်း သို့မဟုတ် ဖိုင်တွဲ၊ သိမ်းဆည်းထားသည့်အတိုင်း trusted document ၏ path၊ trusted location ၏ `Path`။ |
| `LastOpenedUTC` | Item ကို နောက်ဆုံးဖွင့်ခဲ့သည့်အချိန်။ |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | စာရွက်စာတမ်းကို ယုံကြည်ခဲ့သည့်အချိန်၊ ၎င်း၏ macro များကို ဖွင့်ခဲ့/မဖွင့်ခဲ့နှင့် record ၏ flag များ။ မသိသော flag များဖြစ်လျှင် `MacrosEnabled` ကို ဗလာထားပြီး diagnostic တစ်ခု ထုတ်ပေးပါသည်။ |
| `Setting`, `Value` | Macro ဆက်တင်တစ်ခုနှင့် ၎င်း၏ တန်ဖိုးနှင့် အဓိပ္ပာယ် (`1 (all macros enabled)`)၊ trusted location ၏ key (`Location3`) နှင့် ၎င်း၏ အခြားတန်ဖိုးများ (`AllowSubFolders=1; Date=…; Description=…`)။ |
| `KeyPath`, `KeyLastWrittenUTC` | Key နှင့် ၎င်း၏ နောက်ဆုံးရေးသားချိန်။ |
| `DataRaw` | Decode မလုပ်နိုင်သည့်အခါ တန်ဖိုး၏ byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု context။ |
| `Problems` | `*` မပါသော သို့မဟုတ် အချိန်မဟုတ်သည့် အချိန်ပါသော item များ၊ 24 byte မဟုတ်သော trust record များ၊ `Path` မပါသော location များ၊ မှားယွင်းသော type များ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. `MacrosEnabled` true ဖြစ်သည်များဖြင့် စတင်ပါ: စာရွက်စာတမ်းများ၊ အချိန်နှင့် မည်သူက ဖြစ်သည်။ ဖိုင်များ (`$MFT`၊
   USN Journal၊ `Zone.Identifier`)၊ ၎င်းတို့၏ ရင်းမြစ် (browser နှင့် mail မှတ်တမ်း) နှင့် နောက်တွင် ဘာလည်ပတ်ခဲ့သည်ကို ရှာပါ။
2. Macro ဆက်တင်များနှင့် trusted location များကို စစ်ဆေးပါ: macro အားလုံးကို ဖွင့်ထားခြင်း၊ Internet macro များကို
   မပိတ်ဆို့ထားခြင်း၊ VBA project ဝင်ရောက်ခွင့် သို့မဟုတ် အသုံးပြုသူထည့်ထားသော location (`Downloads`, `Temp`, ကွန်ရက် share
   တွင်) တို့သည် မူလတန်ဖိုးများကို အားနည်းစေပါသည်။
3. ဖယ်ရှားနိုင်သော နှင့် ကွန်ရက် location များ အပါအဝင် စာရွက်စာတမ်းများ၏ path များအတွက် `LastOpenedUTC` ကို LNK ဖိုင်များ၊
   jump list များနှင့် RecentDocs တို့နှင့်အတူ timeline ပေါ်တွင် ထားပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Microsoft 365 ဖိုင်များတွင် macro များ ဖွင့်ခြင်း သို့မဟုတ် ပိတ်ခြင်း](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: Trusted documents](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: Trusted location တစ်ခု ထည့်ခြင်း၊ ဖယ်ရှားခြင်း သို့မဟုတ် ပြောင်းလဲခြင်း](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
