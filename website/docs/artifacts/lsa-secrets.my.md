# LSA လျှို့ဝှက်ဒေတာ { #lsa-secrets }

## ၎င်းတို့ ဆိုသည်မှာ { #what-they-are }

Local Security Authority (LSA) သည် လျှို့ဝှက်ဒေတာ (secret) များကို `SECURITY` hive တွင် သိမ်းဆည်းထားသည်။
၎င်းတို့တွင် autologon password၊ service များ run သည့် အကောင့်များ၏ password များ၊ ကွန်ပျူတာအကောင့်၏ domain
password၊ DPAPI system key များနှင့် cache လုပ်ထားသော domain logon များကို encrypt လုပ်ရာတွင် သုံးသော key တို့
ပါဝင်သည်။ Domain အသုံးပြုသူများ domain controller မပါဘဲ sign in ဝင်နိုင်စေရန် cache လုပ်ထားသော domain logon
များကိုလည်း သိမ်းဆည်းထားသည်။

`parse-registry lsa-secrets` သည် စနစ်တစ်ခုတွင် မည်သည့် secret များ ရှိပြီး ၎င်းတို့ကို မည်သည့်အချိန်တွင် သတ်မှတ်ခဲ့သည်
နှင့် cache လုပ်ထားသော logon မည်မျှကို သိမ်းထားပြီး အသုံးပြုနေသည်ကို ဖော်ပြသည်။ **၎င်းသည် အမည်များနှင့် အချိန်များကိုသာ
ဖတ်သည်- မည်သည့်အရာကိုမျှ decrypt မလုပ်ဘဲ မည်သည့် secret ၏ byte များကိုမျှ ဖတ်ခြင်း သို့မဟုတ် ရေးသားခြင်း မပြုပါ။**
Juntei သည် အထောက်အထားကို ဖတ်ရှုသည်။ ၎င်းသည် credential များကို ထုတ်ယူခြင်း မပြုပါ။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> စနစ်တွင် ဤအမည်ရှိ secret တစ်ခုရှိခဲ့ပြီး ၎င်းကို
ဤအချိန်တွင် သတ်မှတ်ခဲ့သည်။ Service တစ်ခု၏ secret (`_SC_<service>`) ဆိုသည်မှာ LSA သည် ထို service run သည့်
အကောင့်၏ password ကို သိမ်းဆည်းထားသည်ဟု ဆိုလိုသည်။

## တည်နေရာ { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| အမည် | အမျိုးအစား | ၎င်းသည် အဘယ်နည်း |
|---|---|---|
| `DefaultPassword` | `autologon` | Winlogon က အလိုအလျောက် sign in ဝင်ရာတွင် သုံးသော password။ |
| `_SC_<service>` | `service_account` | `<service>` run သည့် အကောင့်၏ password။ |
| `$MACHINE.ACC` | `machine_account` | ကွန်ပျူတာအကောင့်၏ ၎င်း domain အတွင်းရှိ password။ |
| `DPAPI_SYSTEM` | `dpapi_system` | စနစ်၏ DPAPI key များ။ |
| `NL$KM` | `cached_logon_key` | Cache လုပ်ထားသော logon များကို encrypt လုပ်ရာတွင် သုံးသော key။ |
| `L$_SQSA_<SID>` | `security_questions` | စက်တွင်းအကောင့်တစ်ခု၏ လုံခြုံရေးမေးခွန်းများ (Windows 10 နှင့် နောက်ပိုင်း)။ |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | Dial-up နှင့် VPN credential များ။ |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | `LsaStorePrivateData` က ပေးသော prefix အလိုက် အခြား private data များ။ |

`SECURITY` ကို ၎င်း၏ transaction log များနှင့်အတူ စုဆောင်းပြီး host ကို အမည်ပေးသော စနစ်၏ `SYSTEM` hive ကိုလည်း
စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **Value များ မဟုတ်ဘဲ အမည်များနှင့် အချိန်များ။** Value တစ်ခုတွင် ဒေတာ ရှိ/မရှိကို ၎င်း၏ အရွယ်အစားမှ သိရှိသည်။
  ၎င်း၏ အကြောင်းအရာကို မဖတ်ပါ။
- **`DefaultPassword` ရှိခြင်းက autologon ဟု မဆိုလိုပါ။** ဤ secret သည် အလိုအလျောက် sign in တစ်ကြိမ်မျှ
  မဝင်ခဲ့သော စနစ်များအပါအဝင် စနစ်အများစုတွင် ရှိသည်။ `SOFTWARE` ၏
  `Microsoft\Windows NT\CurrentVersion\Winlogon` (`AutoAdminLogon`, `DefaultUserName`) က autologon ကို
  ပြင်ဆင်ထားခြင်း ရှိ/မရှိကို ဖော်ပြသည်။ မကြာသေးမီက `CurrentSetUTC` သည် ရှင်းပြသင့်သော ပြောင်းလဲမှု ဖြစ်သည်။
- **Service secret များ-** service တစ်ခုကို password ရှိသော အကောင့်ဖြင့် run ရန် သတ်မှတ်ထားသည့်အခါ
  `_SC_<service>` ပေါ်လာသည်။ ၎င်းကို `parse-registry services` (`ObjectName`) နှင့် နှိုင်းယှဉ်ပြီး မကြာသေးမီက
  ထည့်သွင်းခဲ့သော သို့မဟုတ် မသိသော service များကို ရှာဖွေပါ။ Service မရှိတော့ပြီးနောက်တွင်လည်း secret ကျန်ရှိနိုင်သည်။
- **အချိန်များ-** `CurrentSetUTC` နှင့် `OldSetUTC` သည် Windows က လက်ရှိနှင့် ယခင် value များကို သတ်မှတ်ခဲ့သည့်
  အချိန်များ ဖြစ်သည်။ `$MACHINE.ACC` password သည် ပုံသေအားဖြင့် ရက် 30 တိုင်း ပြောင်းလဲသောကြောင့် ၎င်း၏ အချိန်များ
  ရွေ့လျားသည်။ `KeyLastWrittenUTC` နှင့် `CurrVal`/`OldVal` key အချိန်များသည် secret နောက်ဆုံးပြောင်းလဲခဲ့သည့်
  အချိန်နှင့် သက်ဆိုင်သည်။
- **Cache လုပ်ထားသော logon များ-** slot တစ်ခု၏ record က အသုံးပြုသူတစ်ဦးကို အမည်ပေးထားသည့်အခါ ၎င်းကို အသုံးပြုနေသည်။
  Record ၏ header ကို encrypt မလုပ်ထားပါ။ ၎င်း၏ အသုံးပြုသူအမည် အရှည်ကိုသာ ဖတ်သည်။ `CachedLogonsKept` သည်
  `NL$Control` ၏ အရေအတွက်ဖြစ်ပြီး `CachedLogonsCount` (ပုံသေ 10) ကို လိုက်နာသည်။ အသုံးပြုနေသော slot များက domain
  အသုံးပြုသူများ interactive အနေဖြင့် sign in ဝင်ခဲ့ကြောင်း ဆိုလိုပြီး ၎င်းတို့၏ credential များကို hive မှ offline ဖြင့်
  crack လုပ်နိုင်သည်။ အသုံးပြုသူများ၏ အမည်များကို encrypt လုပ်ထားပြီး decode မလုပ်ပါ။ ပြည့်စုံသော header နှင့် ကိုက်ညီသော အသုံးပြုသူအမည် အရှည်ရှိသည့် စစ်ဆေးအတည်ပြုထားသော binary record များကိုသာ အသုံးပြုနေသည်ဟု ရေတွက်သည်။ `Problems` က ပျက်စီးနေသော slot များကို အစီရင်ခံပါက အသုံးပြုနေသော အရေအတွက်တွင် အတည်ပြုထားသော slot များသာ ပါဝင်သည်။ ပျက်စီးနေသော slot များကို ဗလာဟု အဓိပ္ပာယ်မဖော်ပါနှင့်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

Hive တစ်ခုစီအတွက် အနှစ်ချုပ်တွင် secret တစ်ခုစီကို ၎င်း၏ အမျိုးအစားနှင့် သတ်မှတ်ခဲ့သည့်အချိန်တို့နှင့်အတူ ဖော်ပြသည်။
ထို့နောက် LSA က အကောင့် password များကို သိမ်းဆည်းထားသော service များကို အမည်ဖော်ပြပြီး အသုံးပြုနေသော cache
logon slot များကို ရေတွက်သည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `SourcePath` | Host (စနစ်၏ `SYSTEM` hive မှ) နှင့် `SECURITY` hive။ |
| `Secret`, `Kind` | Secret ၏ အမည်နှင့် ၎င်းသည် အဘယ်နည်း (အထက်ပါ ဇယားကို ကြည့်ပါ)။ Cache လုပ်ထားသော logon များ၏ record အတွက် `Cache` နှင့် `cached_logons`။ |
| `ServiceName`, `UserSID` | `_SC_` secret ရည်ရွယ်သော service နှင့် အမည်က ပေးသော အသုံးပြုသူ၏ SID (`L$_SQSA_`, `RasDialParams`)။ |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` နှင့် `OupdTime`- လက်ရှိနှင့် ယခင် value များကို သတ်မှတ်ခဲ့သည့်အချိန်။ |
| `HasCurrentValue`, `HasOldValue` | `CurrVal` နှင့် `OldVal` တွင် ဒေတာ ရှိ/မရှိ။ |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | `CurrVal`၊ `OldVal` နှင့် secret key များ၏ နောက်ဆုံးရေးသားချိန်များ။ |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | Cache လုပ်ထားသော logon များ၏ record- slot များ၊ အသုံးပြုနေသော slot များနှင့် `NL$Control` ၏ အရေအတွက်။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | ပုံစံမမှန်သော သို့မဟုတ် အတိုင်းအတာပြင်ပ အချိန်များ (အချိန် metadata ကိုသာ ပြသည်)၊ ပျက်စီးနေသော cache header များ၊ မှားယွင်းသော cache value အမျိုးအစားများနှင့် ဖတ်၍မရသော key များ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. မသိသော သို့မဟုတ် မကြာသေးမီက ထည့်သွင်းခဲ့သော service များ၏ `_SC_` secret များကို ရှာဖွေပြီး ၎င်းတို့ကို
   `parse-registry services` နှင့် event 7045 (service တစ်ခု ထည့်သွင်းခဲ့သည်) တို့နှင့် တိုက်ဆိုင်ပါ။
2. `DefaultPassword` ၏ အချိန်များကို `Winlogon` ၏ autologon ဆက်တင်များနှင့် တိုက်ဆိုင်စစ်ဆေးပါ။
3. Domain member များတွင် အသုံးပြုနေသော cache logon slot များနှင့် `Cache` ကို နောက်ဆုံးရေးသားခဲ့သည့်အချိန်ကို မှတ်သားပါ။
4. Secret များ၏ အချိန်များကို timeline နှင့် နှိုင်းယှဉ်ပါ- ကျူးကျော်ဝင်ရောက်မှု ကာလအတွင်း သတ်မှတ်ခဲ့သော secret သည်
   service အကောင့်အသစ်၊ autologon သို့မဟုတ် machine account ပြောင်းလဲမှုကို ညွှန်ပြနိုင်သည်။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Private data object](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: Cached domain logon information](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: Cached credentials](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
