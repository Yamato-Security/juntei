# SAM စက်တွင်းအကောင့်များ { #sam-local-accounts }

## SAM တွင် ပါဝင်သည့် အရာများ { #what-the-sam-contains }

Security Account Manager (`SAM`) hive သည် စက်တွင်း account record များကို သိမ်းဆည်းပါသည်။ Juntei သည် account
အမည်များ၊ RID များ၊ profile/comment field များ၊ account-control flag များ၊ last-logon/password timestamp များ၊ counter များ၊
password hint များ၊ Microsoft-account ချိတ်ဆက်မှု၊ profile path များနှင့် Windows က account ကို sign-in UI မှ ဝှက်ထား/မထား
တို့ကို decode လုပ်ပါသည်။

၎င်းသည် အသစ်ဖန်တီးထားသော စက်တွင်း administrator များ၊ ပြန်ဖွင့်ထားသော အသုံးမပြုတော့သည့် account များ၊ password
ပြောင်းလဲမှုများ၊ ဝှက်ထားသော account များနှင့် ကွန်ရက်အတွင်း ဘေးတိုက်ရွေ့လျားမှုအတွက် ပြင်ဆင်ထားသော account များကို
ရှာဖွေရာတွင် အသုံးဝင်ပါသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> စက်တွင်း SAM သည် account ကို မှတ်တမ်းတင်ထားသော
အခြေအနေဖြင့် ကိုယ်စားပြုခဲ့ပါသည်။ ၎င်းသည် ပြည့်စုံသော authentication မှတ်တမ်း မဟုတ်ဘဲ domain account များကိုလည်း မဖော်ပြပါ။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

Hive နှင့် log များကို လုံလောက်သော privilege/raw ဝင်ရောက်ခွင့်ဖြင့် စုဆောင်းပါ။ Host အထောက်အထားကို collection
အတွင်းရှိ သက်ဆိုင်ရာ `SYSTEM` hive မှ ဖြည့်စွက်နိုင်ပါသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- ဤညွှန်ကြားချက်သည် Active Directory domain အသုံးပြုသူများ မဟုတ်ဘဲ **စက်တွင်း** account များကို parse လုပ်ပါသည်။
- Juntei သည် password hash များကို ရည်ရွယ်ချက်ရှိရှိ မထုတ်ယူပါ။
- RID 500 သည် အမည်ပြောင်းထားလျှင်ပင် built-in Administrator account ဖြစ်ပါသည်။ အမည်တစ်ခုတည်းဖြင့် မလုံလောက်ပါ။
- `LastLogon` နှင့် counter များသည် registry အခြေအနေဖြစ်ပြီး session တစ်ခုချင်းစီ၏ ပြည့်စုံသော audit trail မဟုတ်ပါ။
- `HiddenFromLogonUI` သည် account ကို sign-in မျက်နှာပြင်မှ ဝှက်ထားခြင်းသာ ဖြစ်ပြီး account ကို ပိတ်ထားခြင်း မဟုတ်ပါ။
- Built-in `DefaultAccount` နှင့် `WDAGUtilityAccount` ကို တရားဝင် ဝှက်ထားနိုင်ပါသည်။
- ပျက်စီးနေသော/မရှိသော `F` value မှ null account flag များကို enabled အဖြစ် အဓိပ္ပာယ်မဖော်ရပါ။
- `NamesKeyLastWrittenUTC` သည် account-name မှတ်ပုံတင်ခြင်း context ကို ခန့်မှန်းပေးသော်လည်း ဖန်တီးချိန်ဟု အာမခံချက် မရှိပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `Host`, `SourcePath` | Host context နှင့် တိကျသော SAM hive။ |
| `RID` | Relative Identifier။ account ကို ခွဲခြားရန် ခိုင်မာသော pivot ဖြစ်ပြီး 500 သည် built-in Administrator ဖြစ်ပါသည်။ |
| `User` | လက်ရှိ စက်တွင်း account အမည်။ အမည်ပြောင်းနိုင်သဖြင့် RID ဖြင့် ဆက်စပ်စစ်ဆေးပါ။ |
| `FullName`, `Comment`, `UserComment` | ဖော်ပြချက် စာသား။ တိုက်ခိုက်သူ ထိန်းချုပ်နိုင်ပြီး ဟန်ဆောင်ခြင်းကို ပြန်လည်စစ်ဆေးရာတွင် အသုံးဝင်ပါသည်။ |
| `AccountFlags` | disabled၊ locked out၊ password not required၊ normal account၊ password never expires ကဲ့သို့ decode လုပ်ထားသော အခြေအနေများ။ |
| `AccountFlagsValue` | မှန်ကန်မှု စစ်ဆေးရန်နှင့် မပံ့ပိုးသော bit များအတွက် raw ဂဏန်း flag များ။ |
| `CountryCode`, `CodePage` | Account ၏ locale/code-page metadata။ များသောအားဖြင့် context ဖြစ်ပြီး attribution မဟုတ်ပါ။ |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | တူညီသော last-logon အခြေအနေကို precision အပြည့်ဖြင့်နှင့် စက္ကန့်ပြည့် compatibility ဖြင့်။ ပြည့်စုံသော မှတ်တမ်း မဟုတ်ပါ။ |
| `PasswordLastSetUTC` | မှတ်တမ်းတင်ထားသော password ပြောင်းလဲချိန်။ မမျှော်လင့်ထားသော မကြာသေးမီ/ဟောင်းနွမ်းသော တန်ဖိုးများကို စုံစမ်းပါ။ |
| `AccountExpirationUTC` | အကန့်အသတ်ရှိလျှင် ပြင်ဆင်ထားသော သက်တမ်းကုန်ဆုံးချိန်။ Null သည် never/not set ဟု ဆိုလိုနိုင်ပါသည်။ |
| `LastFailedLogonUTC` | SAM အခြေအနေတွင် မှတ်တမ်းတင်ထားသော နောက်ဆုံး မအောင်မြင်သည့် စက်တွင်း logon။ Security event များဖြင့် အတည်ပြုပါ။ |
| `FailedLogonCount`, `LogonCount` | Windows-specific update အပြုအမူရှိသော စုစုပေါင်း/အခြေအနေ counter များ။ တိကျသော event အရေအတွက် မဟုတ်ဘဲ context အဖြစ် အသုံးပြုပါ။ |
| `PasswordHint` | အသုံးပြုသူ ပေးထားသော အရေးကြီး hint။ password ပုံစံများကို ဖော်ထုတ်နိုင်ပါသည်။ သင့်လျော်စွာ ကိုင်တွယ်ပါ။ |
| `InternetUserName` | ရှိလျှင် ချိတ်ဆက်ထားသော Microsoft account အထောက်အထား။ |
| `HiddenFromLogonUI` | UI-ဝှက်ထားမှု အခြေအနေ။ RID မြင့်သော/မမျှော်လင့်ထားသော custom account များကို ပြန်လည်စစ်ဆေးပါ။ disabled ဟု မဆိုလိုပါ။ |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | စက်တွင်း account ပတ်ဝန်းကျင်/logon ပြင်ဆင်မှု။ ပုံမှန်မဟုတ်သော script/UNC path များသည် persistence ကို ညွှန်ပြနိုင်ပါသည်။ |
| `KeyLastWrittenUTC` | RID account-key ပြောင်းလဲချိန်။ key အဆင့်ဖြစ်ပြီး field တစ်ခုချင်းစီ မဟုတ်ပါ။ |
| `NamesKeyLastWrittenUTC` | Username mapping-key ပြောင်းလဲချိန်။ account သက်တမ်းစက်ဝန်းကို ခန့်မှန်းရာတွင် အသုံးဝင်ပါသည်။ |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | မှန်ကန်မှု စစ်ဆေးရန်နှင့် အထူး sentinel ကိုင်တွယ်ရန် ထိန်းထားသော raw 64-bit FILETIME တန်ဖိုးများ။ |
| `TransactionLogStatus`, `LogsApplied` | Registry replay context။ |
| `Problems` | F/V/Names အကြား သဘောမတူညီမှု၊ ပျောက်နေသော entry များ၊ ပုံစံမမှန်သော offset များ သို့မဟုတ် decode ပျက်စီးမှု။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. RID၊ အမည်၊ flag များ၊ profile path နှင့် `NamesKeyLastWrittenUTC` ကို host ၏ baseline နှင့် နှိုင်းယှဉ်ပါ။
2. Enabled ဖြစ်သော RID 1000+ account များ၊ password-not-required၊ never-expiring၊ ဝှက်ထားသော သို့မဟုတ် မကြာသေးမီက ပြောင်းလဲထားသော account များကို ပြန်လည်စစ်ဆေးပါ။
3. Security 4720/4722/4724/4732 event များ၊ logon event များ၊ profile ဖန်တီးမှုနှင့် အဝေးမှ လှုပ်ရှားမှုတို့နှင့် ဆက်စပ်စစ်ဆေးပါ။
4. Collection မတိုင်မီ account တစ်ခုကို ဖယ်ရှားခဲ့လျှင် registry ပြန်လည်ရယူမှု မှတ်တမ်းကို ရှာဖွေပါ။
5. တရားဝင် provisioning/default account များကို တိုက်ခိုက်သူ ဖန်တီးထားသော interactive account များမှ ခွဲခြားပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [dissect.target SAM plugin](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
