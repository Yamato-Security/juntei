# Persistence Hunter { #persistence-hunter }

အမြဲတမ်းဝင်ရောက်နိုင်မှု (persistence) ဆိုသည်မှာ code ကို ထပ်မံ run စေသည့် နည်းလမ်းဖြစ်သည်- Windows စတင်သည့်အခါ၊
အသုံးပြုသူ sign in ဝင်သည့်အခါ၊ အပလီကေးရှင်းတစ်ခု ဖွင့်သည့်အခါ၊ အချိန်ဇယားသတ်မှတ်ထားသော အချိန်ရောက်သည့်အခါ သို့မဟုတ်
အခြား event တစ်ခု ဖြစ်ပေါ်သည့်အခါ။ တိုက်ခိုက်သူများသည် တရားဝင်ဆော့ဖ်ဝဲများက အပ်ဒိတ်များ၊ စက်ပစ္စည်း ပံ့ပိုးမှု၊ လုံခြုံရေးနှင့်
စီမံခန့်ခွဲမှုအတွက် အသုံးပြုသော ဆက်တင်များကိုပင် အလွဲသုံးစား ပြုနိုင်သည်။

`juntei persistence-hunter` သည် ဖမ်းယူထားသော Windows အထောက်အထားများကို offline ဖြင့် ဤယန္တရားများနှင့် ဆက်စပ်သော
အကောင့်/ကာကွယ်မှု ပုံမှန်မဟုတ်မှုများကို လွှမ်းခြုံသည့် **စစ်ဆေးမှု 131 ခု** ဖြင့် စစ်ဆေးသည်။ ၎င်းသည် registry hive များ၊
task ဖိုင်များ၊ script များ၊ အပလီကေးရှင်း ဆက်တင်များနှင့် ရွေးချယ်ထားသော ဒေတာဘေ့စ် မှတ်တမ်းများကို ဖတ်သည်။

ဤစာမျက်နှာသည် အကောင်အထည်ဖော်ထားသော စစ်ဆေးမှုတိုင်းကို ရှင်းပြသည်။ Option အပြည့်အစုံ၊ ထုတ်ယူမှု field များ၊ profile များနှင့်
timestamp ကိုင်တွယ်ပုံအတွက် [ညွှန်ကြားချက် လမ်းညွှန်](../commands/persistence-hunter.md) ကို ဖွင့်ပါ။
[ကျေးဇူးတင်လွှာ](acknowledgements.md) တွင် Python PyrsistenceSniper project နှင့် မူရင်း PowerShell
PersistenceSniper tool တို့၏ link များ ပါဝင်သည်။

## စုံစမ်းစစ်ဆေးမှု စတင်ခြင်း { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

ကိုယ်စားလှယ်လောင်း (candidate) စာရင်း အပြည့်အစုံအတွက် `--min-severity info` ကို အသုံးပြုပါ။ လက်ဖြင့် run သော ညွှန်ကြားချက်၏
ပုံသေတန်ဖိုးမှာ `medium` ဖြစ်သည်။ ပျောက်နေသော payload သို့မဟုတ် signer အထောက်အထားကြောင့် အပြည့်အဝ နှိုင်းယှဉ်၍ မရသည့်တိုင်
၎င်း၏ profile က တစ်စိတ်တစ်ပိုင်း allow ကိုက်ညီမှုများကို `LOW` အဖြစ် အမျိုးအစားခွဲနိုင်သည်။ allow ကိုက်ညီမှု သို့မဟုတ် မရင်းနှီးသော
entry တစ်ခုခုကမျှ တစ်စုံတစ်ရာ အန္တရာယ်ရှိ/မရှိကို ဆုံးဖြတ်ပေးခြင်း မရှိပါ။ `auto-parse` သည် severity အားလုံးကို
ထိန်းသိမ်းထားပြီးဖြစ်ကာ ကွန်ပျူတာတစ်လုံးစီ၏ `persistence/` အမျိုးအစားအောက်တွင် တွေ့ရှိချက်များကို ရေးသားသည်။

ယန္တရားတစ်ခုကို အာရုံစိုက်ရန် အောက်ပါမှ ၎င်း၏ **Check ID** ကို ကူးယူပါ-

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

ညွှန်ကြားချက်သည် ပုံသေအားဖြင့် မျှဝေထားသော worker pool ကို အသုံးပြုသည်။ worker လေးခု ရွေးချယ်ရန် `-T 4` ကို ထည့်ပါ၊ သို့မဟုတ်
single-thread scan အတွက် `-T 1` ကို ထည့်ပါ။ Input တွင် ကွန်ပျူတာများစွာ ပါဝင်နိုင်ပြီး ၎င်းတို့၏ အထောက်အထားများကို
သီးခြားစီ ထားရှိသည်။

## တွေ့ရှိချက်များကို အထောက်အထားအဖြစ် ဖတ်ရှုခြင်း { #read-findings-as-evidence }

| မေးခွန်း | စစ်ဆေးရမည့်အရာ |
|---|---|
| ဘာကို ဘယ်နေရာမှာ ပြင်ဆင်ထားသလဲ။ | `CheckId`၊ `Path` နှင့် argument များအပါအဝင် `Value` အပြည့်အစုံ။ ဖမ်းယူထားသော hive၊ ဖိုင် သို့မဟုတ် ဒေတာဘေ့စ်သို့ `SourceFile` ကို လိုက်ကြည့်ပါ။ |
| ပစ်မှတ်သည် စုဆောင်းမှုထဲတွင် ရှိသလား။ | `ResolvedPath`၊ `ResolutionStatus` နှင့် `Exists`။ မစုဆောင်းရသေးသော၊ မရေရာသော သို့မဟုတ် မပံ့ပိုးသော reference များသည် မသိရှိရပါ။ Juntei သည် စိစစ်သူ၏ စက်မှ ဖိုင်ဖြင့် အစားထိုးခြင်း မပြုပါ။ |
| ပြင်ဆင်မှုသည် အမှန်တကယ် သက်ရောက်မှုရှိသလား။ | အမှန်တကယ် trigger၊ enable flag များ၊ ရွေးချယ်ထားသော control set၊ Windows/အပလီကေးရှင်း ဗားရှင်း၊ အသုံးပြုသူနှင့် architecture။ စစ်ဆေးမှုအချို့သည် အသုံးမပြုတော့သော ပြင်ဆင်မှုကို တမင်ထိန်းသိမ်းထားသည်။ |
| ၎င်းသည် တိုက်ခိုက်မှုလား။ | `Severity` နှင့် `PolicyReason` တို့သည် profile ကိုက်ညီမှုကို ရှင်းပြခြင်းဖြစ်ပြီး စီရင်ချက် မဟုတ်ပါ။ မျှော်လင့်ထားသော ဆော့ဖ်ဝဲ၊ ညွှန်ကြားချက် အပြည့်အစုံ၊ ဖိုင် hash များ၊ သီးခြားလွတ်လပ်သော လုပ်ဆောင်မှု အထောက်အထားနှင့် ပတ်ဝန်းကျင် timeline တို့နှင့် နှိုင်းယှဉ်ပါ။ |
| မည်သူ run နိုင်သလဲ။ | `AccessGained` သည် နယ်ပယ်/အခြေအနေ အရိပ်အမြွက်ဖြစ်သည်။ စက်တစ်ခုလုံးဆိုင်ရာ ပြင်ဆင်မှုသည် `SYSTEM` ဟု မကြာခဏ ဖော်ပြသော်လည်း စတင်ခဲ့သော process သည် SYSTEM အဖြစ် run ခဲ့ကြောင်း သက်သေမပြပါ။ ရရှိနိုင်သည့်နေရာတွင် task principal များနှင့် BITS owner SID များက ပိုမိုတိကျသော အခြေအနေကို ပေးသည်။ |
| ဘယ်အချိန်က ပြောင်းလဲခဲ့သလဲ။ | `KeyLastWrite` သည် registry key တစ်ခုလုံးကို ရက်စွဲသတ်မှတ်သည်။ `LastChange`၊ `ChangeSource` နှင့် `ChangeEvidence` တို့သည် ရရှိနိုင်သည့်အခါ သီးခြား MFT/EVTX ချိတ်ဆက်စစ်ဆေးမှုကို ဖော်ပြသည်။ မည်သည့် field ကမျှ payload လုပ်ဆောင်ချိန်ကို အလိုအလျောက် ရက်စွဲသတ်မှတ်ခြင်း မပြုပါ။ |
| ပြန်လည်ရယူထားသော အခြေအနေသည် မည်မျှ ယုံကြည်စိတ်ချရသလဲ။ | Replay လုပ်ထားသော၊ carve လုပ်ထားသော၊ ပျက်စီးနေသော၊ ပျောက်နေသော သို့မဟုတ် ကန့်သတ်ထားသော အထောက်အထားများအတွက် `RecordStatus`၊ `Coverage.csv` နှင့် သတိပေးချက်များကို ဖတ်ပါ။ ပြန်လည်ရယူထားသော သမိုင်းဝင်မှတ်တမ်းသည် ယခုအခါ active မဖြစ်တော့နိုင်ပါ။ |
| အမည်ဖော်ထားသော publisher ကို ယုံကြည်နိုင်သလား။ | `Signer` နှင့် `SignerStatus` တို့သည် ဖော်ပြချက်သဘော PE/certificate/catalog metadata ဖြစ်သည်။ Juntei သည် Authenticode signature များ သို့မဟုတ် certificate ယုံကြည်မှုကို စစ်ဆေးအတည်ပြုခြင်း မပြုပါ။ |

တွေ့ရှိချက် မရှိခြင်းသည် persistence မရှိကြောင်း သက်သေမပြပါ။ `complete` coverage row ဆိုသည်မှာ သက်ဆိုင်ရာ စုဆောင်းထားသော
source ကို ဖတ်ခဲ့ကြောင်း ဆိုလိုခြင်းဖြစ်ပြီး ရယူမှု (acquisition) တွင် သက်ဆိုင်ရာ ဖိုင်တိုင်း၊ အသုံးပြုသူတိုင်း သို့မဟုတ်
ဖြစ်နိုင်သော Windows persistence ယန္တရားတိုင်း ပါဝင်ကြောင်း မဆိုလိုပါ။ အခြား coverage အခြေအနေများက ပျောက်နေသော၊
တစ်စိတ်တစ်ပိုင်းသာဖြစ်သော၊ ပိတ်ထားသော သို့မဟုတ် မအောင်မြင်သော စစ်ဆေးမှုများကို ရှင်းပြသည်။ Hunter သည် live-memory၊
ကွန်ရက် သို့မဟုတ် cloud-identity scan မဟုတ်ပါ။

## ကတ်တလောက်နှင့် တည်နေရာ ဖော်ပြပုံ { #catalogue-and-location-notation }

ကတ်တလောက်သည် PyrsistenceSniper နှင့် native `start_menu_com` စစ်ဆေးမှုကို အခြေခံထားသည်။
အောက်ပါ ဖော်ပြချက်များသည် Juntei ၏ native အပြုအမူကို ၎င်း၏ အထောက်အထားဆိုင်ရာ ကန့်သတ်ချက်များနှင့်တကွ
မှတ်တမ်းတင်ထားသည်။ **Registry အခြေပြု စစ်ဆေးမှု 116 ခုနှင့် ဖိုင်/repository စစ်ဆေးမှု 15 ခု** ရှိသည်။
ဖိုင်စစ်ဆေးမှုအချို့သည် registry ဒေတာကိုလည်း ပေါင်းစပ်သောကြောင့် ဤအရေအတွက်များသည် တစ်ခုနှင့်တစ်ခု သီးသန့်ဖြစ်သော
ရယူမှုလိုအပ်ချက်များ မဟုတ်ဘဲ implementation အရေအတွက်များ ဖြစ်သည်။ ယန္တရားများကို အောက်တွင် စုံစမ်းစစ်ဆေးရေး ရည်ရွယ်ချက်အလိုက်
အုပ်စုဖွဲ့ထားပြီး အုပ်စုတစ်ခုစီအတွင်း Check ID အလိုက် အက္ခရာစဉ် စီထားသည်။

- `HKLM\SYSTEM`၊ `HKLM\SOFTWARE` နှင့် `HKLM\SAM` တို့သည် ဖမ်းယူထားသော စက် hive များကို ရည်ညွှန်းပြီး ပုံမှန်အားဖြင့်
  `Windows\System32\config` မှ ရရှိနိုင်ပါက transaction log များနှင့်အတူ စုဆောင်းထားသည်။
- `HKU\<user>` သည် သက်ဆိုင်ရာ ဖမ်းယူထားသော အသုံးပြုသူ hive ကို ဆိုလိုသည်။ အသုံးပြုသူ ဆက်တင်အများစုသည် NTUSER.DAT မှ လာသည်။
  ဖော်ပြထားသည့်အတိုင်း class registration များသည် `AppData\Local\Microsoft\Windows\UsrClass.dat` မှ လာနိုင်သည်။
- `<ControlSet>` သည် SYSTEM ရှိ ရွေးချယ်ထားသော active control set ကို ဆိုလိုသည်။ DSRM စစ်ဆေးမှု၏ အပို သမိုင်းဝင်
  တည်နေရာများကို အောက်တွင် ရှင်းလင်းစွာ ဖော်ပြထားသည်။ ၎င်းတို့ကို ပြန်လည်သုံးသပ်သည့်အခါ `SYSTEM\Select` ကို စစ်ဆေးပါ။
- `<name>` သည် ပြောင်းလဲနိုင်သော path အစိတ်အပိုင်း ဖြစ်သည်။ Registry ပစ်မှတ်ဇယားများတွင် `<child>` သည် တိုက်ရိုက် child
  registration များကို ဆိုလိုပြီး အကန့်အသတ်မဲ့ recursive လမ်းလျှောက်ခြင်းကို မဆိုလိုပါ။ ရှင်းလင်းစွာ recursive ဖြစ်သော
  စစ်ဆေးမှုများက ၎င်းတို့၏ traversal ကို သီးခြားဖော်ပြထားသည်။ Office path များရှိ ကွင်းများသည် ရွေးချယ်နိုင်သော ဗားရှင်းကို ညွှန်ပြသည်။
- ဇယားများသည် စစ်ဆေးမှုက ရွေးချယ်ထားသော အမှန်တကယ် စက်နှင့် အသုံးပြုသူ view များကို ဖော်ပြသည်။ အကောင်အထည်ဖော်ထားသည့်
  နေရာတွင် 32-bit view ကို ဖော်ပြထားသည်။ စစ်ဆေးမှုတိုင်းက ဖြစ်နိုင်သော architecture/view တိုင်းကို စစ်ဆေးသည်ဟု မယူဆပါနှင့်။
- ယေဘုယျ value စစ်ဆေးမှုများသည် သက်ဆိုင်သည့်နေရာတွင် multi-string element များနှင့် ဂဏန်း သုည အပါအဝင်
  ဗလာမဟုတ်သော ပြင်ဆင်ထားသည့် entry များကို ထုတ်ပေးသည်။ ၎င်းတို့ကိုယ်တိုင်က အန္တရာယ်ရှိသော payload၊ enable လုပ်ထားသော
  trigger သို့မဟုတ် အောင်မြင်သော လုပ်ဆောင်မှုကို မလိုအပ်ပါ။ ပိုမိုရွေးချယ်တတ်သော predicate များကို စစ်ဆေးမှုအလိုက် ဖော်ပြထားသည်။

ATT&CK ID များသည် ကတ်တလောက်မှ အမွေဆက်ခံထားသော mapping များနှင့် နောက်ထပ်လေ့လာရန် link များ ဖြစ်သည်။ ၎င်းတို့သည်
ဖြစ်နိုင်ချေရှိသော အလွဲသုံးစားမှုကို ဖော်ပြခြင်းဖြစ်ပြီး တိုက်ခိုက်သူက ထို technique ကို ဆောင်ရွက်ခဲ့သည်ဟူသော တွေ့ရှိချက်
မဟုတ်ပါ။ ကာကွယ်မှုဆက်တင်နှင့် ရောဂါရှာဖွေရေး စစ်ဆေးမှုများသည် သီးခြားရပ်တည်သော persistence ယန္တရားထက် ဖြစ်ပေါ်စေနိုင်သော
အခြေအနေများကို ဖော်ပြနိုင်သည်။

| ယန္တရား အုပ်စု | စစ်ဆေးမှုများ |
|---|---:|
| [Boot၊ sign-in နှင့် startup](#boot-sign-in-and-startup) | 32 |
| [ဝန်ဆောင်မှုများနှင့် စနစ် extension များ](#services-and-system-extensions) | 13 |
| [အချိန်ဇယားသတ်မှတ်ထားသော task များ](#scheduled-tasks) | 3 |
| [WMI၊ BITS နှင့် အခြား event trigger များ](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorer၊ COM နှင့် ဖိုင် သို့မဟုတ် URL handler များ](#explorer-com-and-file-or-url-handlers) | 15 |
| [Office အပလီကေးရှင်းများနှင့် add-in များ](#office-applications-and-add-ins) | 7 |
| [Debugger များ၊ crash handler များနှင့် compatibility shim များ](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [.NET runtime နှင့် profiler ဆက်တင်များ](#net-runtime-and-profiler-settings) | 7 |
| [DLL နှင့် executable ရှာဖွေမှု override များ](#dll-and-executable-lookup-overrides) | 18 |
| [အကောင့်များနှင့် sign-in မြင်သာမှု](#accounts-and-sign-in-visibility) | 4 |
| [Authentication အစိတ်အပိုင်းများနှင့် credential ကာကွယ်မှု](#authentication-components-and-credential-protection) | 8 |
| **စုစုပေါင်း** | **131** |

## Boot၊ sign-in နှင့် startup { #boot-sign-in-and-startup }

ဤဆက်တင်များသည် boot၊ sign-in သို့မဟုတ် desktop စတင်ပြင်ဆင်ချိန်အတွင်း ညွှန်ကြားချက် သို့မဟုတ် library တစ်ခုကို run စေရန် စီစဉ်ပေးသည်။ Windows ဗားရှင်းနှင့် သက်ဆိုင်ရာ အသုံးပြုသူကို ပြန်လည်သုံးသပ်ပါ- ဆက်တင်တစ်ခုသည် ခေတ်မမီတော့ခြင်း၊ ပိတ်ထားခြင်း သို့မဟုတ် တိုက်ခိုက်သူအတွက်ထက် kiosk အတွက် ရည်ရွယ်ထားခြင်း ဖြစ်နိုင်သည်။

### Active Setup {#active_setup}

**Check ID:** `active_setup` · **ကတ်တလောက် ATT&CK:** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**Juntei စစ်ဆေးသည့်အရာ:** native နှင့် 32-bit view များရှိ စက်တစ်ခုလုံးဆိုင်ရာ Active Setup component registration များမှ `StubPath` ကို ဖတ်သည်။ Windows သည် အသုံးပြုသူ၏ profile ကို ပြင်ဆင်သည့်အခါ ဤညွှန်ကြားချက်များကို အသုံးပြုနိုင်သည်။ စစ်ဆေးမှုသည် စာသားအတိုင်း placeholder value များဖြစ်သော `U` နှင့် `/UserInstall` ကို ကျော်သွားသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` နှင့် သက်ဆိုင်ရာ `Wow6432Node` tree။

**စုံစမ်းစစ်ဆေးနည်း:** Component တစ်ခုစီကို ၎င်း၏ ထည့်သွင်းထားသော ထုတ်ကုန်နှင့် နှိုင်းယှဉ်ပြီး ညွှန်ကြားချက် argument များကို စစ်ဆေးပါ။ စက်တစ်ခုလုံးဆိုင်ရာ registration သည် sign in ဝင်နေသော အသုံးပြုသူ၏ session တွင် လုပ်ဆောင်မှုကို ဖြစ်ပေါ်စေနိုင်သည်။ ထုတ်ယူမှု၏ `SYSTEM` label သည် ပြင်ဆင်မှုနယ်ပယ်ကို ဖော်ပြခြင်းဖြစ်ပြီး ရလဒ် process token ၏ သက်သေ မဟုတ်ပါ။

### Boot Execute {#boot_execute}

**Check ID:** `boot_execute` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Session Manager ၏ `BootExecute` multi-string ကို ဖတ်ပြီး ဗလာမဟုတ်သော entry တစ်ခုချင်းစီကို ထိန်းသိမ်းသည်။ ၎င်းသည် boot အစောပိုင်းအဆင့်အတွက် ပြင်ဆင်ထားသော ညွှန်ကြားချက်များကို မှတ်တမ်းတင်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**စုံစမ်းစစ်ဆေးနည်း:** `autocheck autochk *` သည် ရင်းနှီးသော ပုံသေတန်ဖိုးဖြစ်ပြီး disk tool များက တရားဝင် entry များကို ထည့်နိုင်သည်။ ပုံသေမဟုတ်သော entry တိုင်း run ခဲ့သည် သို့မဟုတ် အန္တရာယ်ရှိသည်ဟု မယူဆဘဲ ထပ်ထည့်ထားသော ညွှန်ကြားချက်များနှင့် ၎င်းတို့၏ ဖိုင်များကို စုံစမ်းစစ်ဆေးပါ။

### Boot Verification Program {#boot_verification_program}

**Check ID:** `boot_verification_program` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** boot verification နှင့် ဆက်စပ်သော executable တည်နေရာဖြစ်သည့် `BootVerificationProgram` ၏ `ImagePath` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**စုံစမ်းစစ်ဆေးနည်း:** ဤယန္တရားအဟောင်းသည် ဖမ်းယူထားသော Windows ဗားရှင်းနှင့် boot ပြင်ဆင်မှုတွင် သက်ဆိုင်ခြင်း ရှိ/မရှိ စစ်ဆေးပါ။ ပစ်မှတ်ဖိုင်ကို ဖော်ထုတ်ပြီး ၎င်း၏ ရည်ရွယ်ချက်ကို မျှော်လင့်ထားသော boot-management ဆော့ဖ်ဝဲနှင့် နှိုင်းယှဉ်ပါ။

### Explorer Load Value {#explorer_load}

**Check ID:** `explorer_load` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Windows sign-in စတင်ပြင်ဆင်မှု ပြင်ဆင်ချက်နှင့် ဆက်စပ်သော `Load` value ကို စက်နှင့် အသုံးပြုသူ hive များမှ ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**စုံစမ်းစစ်ဆေးနည်း:** မရင်းနှီးသော ညွှန်ကြားချက်များ၊ အသုံးပြုသူ ရေးသားနိုင်သော ဦးတည်ရာများနှင့် အကြောင်းအရာကို ရယူ သို့မဟုတ် decode လုပ်သော argument များကို စုံစမ်းစစ်ဆေးပါ။ သက်ဆိုင်မှုနှင့် လုပ်ဆောင်မှုကို သီးခြားစီ အတည်ပြုပါ။ ဗလာရလဒ်သည် အခြား sign-in ယန္တရားများကို မလွှမ်းခြုံပါ။

### Font Drivers {#font_drivers}

**Check ID:** `font_drivers` · **ကတ်တလောက် ATT&CK:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei စစ်ဆေးသည့်အရာ:** legacy `Font Drivers` registry key ရှိ ဗလာမဟုတ်သော value များကို ဖြစ်နိုင်ချေရှိသော library registration များအဖြစ် စာရင်းကောက်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** ၎င်းကို ပုံမှန်မဟုတ်သော legacy ပြင်ဆင်မှု မျက်နှာပြင်အဖြစ် သဘောထားပါ။ Load လုပ်နိုင်သည်ဟု ကောက်ချက်မချမီ Windows ဗားရှင်း ပံ့ပိုးမှုနှင့် ရည်ညွှန်းထားသော library ကို စစ်ဆေးပြီး အမှန်တကယ် အသုံးပြုမှုကို process သို့မဟုတ် image-load အထောက်အထားဖြင့် အတည်ပြုပါ။

### Group Policy Extension DLLs {#gp_extension_dlls}

**Check ID:** `gp_extension_dlls` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Winlogon အောက်ရှိ တိုက်ရိုက် Group Policy extension registration တစ်ခုစီမှ `DllName` ကို ဖတ်သည်။ ဤ DLL များသည် policy လုပ်ဆောင်မှုကို ချဲ့ထွင်ပေးသည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**စုံစမ်းစစ်ဆေးနည်း:** စီမံခန့်ခွဲရေးနှင့် လုံခြုံရေး ထုတ်ကုန်များက extension များကို ထည့်သွင်းနိုင်သည်။ Extension GUID နှင့် DLL ကို မျှော်လင့်ထားသော ထုတ်ကုန်နှင့် ကိုက်ညီအောင် စစ်ဆေးပြီး လုပ်ဆောင်မှုကို အခိုင်အမာ မဆိုမီ policy-processing နှင့် image-load အထောက်အထားကို စစ်ဆေးပါ။

### Group Policy Scripts {#gp_scripts}

**Check ID:** `gp_scripts` · **ကတ်တလောက် ATT&CK:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei စစ်ဆေးသည့်အရာ:** ဖမ်းယူထားသော local Group Policy `scripts.ini` နှင့် `psscripts.ini` registration ဖိုင်များ (အသုံးပြုသူအလိုက် local policy store များ အပါအဝင်) မှ command/parameter အတွဲများကို ခွဲခြမ်းစိတ်ဖြာသည်။ Startup/shutdown နှင့် logon/logoff အပိုင်းများသည် ၎င်းတို့၏ ပြင်ဆင်ထားသော ညွှန်ကြားချက်များနှင့် argument များကို ထိန်းသိမ်းထားပြီး machine အပိုင်းများကို user အပိုင်းများမှ ခွဲခြားထားသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` နှင့် သက်ဆိုင်ရာ `GroupPolicy\User\Scripts` ဖိုင်များ။
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`။

**စုံစမ်းစစ်ဆေးနည်း:** အကောင်အထည်ဖော်ထားသော local policy registration တည်နေရာများကိုသာ စစ်ဆေးသည်။ ၎င်းသည် ပြည့်စုံသော SYSVOL/GPO စာရင်းစစ်ဆေးမှု မဟုတ်ပါ။ Policy directory အောက်ရှိ အခြားနေရာတွင် ရှိနေသော script ဖိုင်တစ်ခုသည် လုံလောက်မှု မရှိပါ။ ၎င်း၏ registration ကို policy လုပ်ဆောင်မှုနှင့် လုပ်ဆောင်မှု အထောက်အထားတို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။

### Logon Scripts (UserInitMprLogonScript) {#logon_scripts}

**Check ID:** `logon_scripts` · **ကတ်တလောက် ATT&CK:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei စစ်ဆေးသည့်အရာ:** အသုံးပြုသူ၏ sign-in အတွက် script တစ်ခုကို ရွေးချယ်နိုင်သော အသုံးပြုသူ၏ `UserInitMprLogonScript` environment value ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**စုံစမ်းစစ်ဆေးနည်း:** ခွင့်ပြုထားသော logon script များနှင့် နှိုင်းယှဉ်ပြီး အမှန်တကယ် script ကို စစ်ဆေးပါ။ Value သည် နောက်ဆက်တွဲ sign-in သို့မဟုတ် လုပ်ဆောင်မှုကို သက်သေမပြဘဲ domain မှ ပေးသော script များအတွက် အပို policy အထောက်အထား လိုအပ်သည်။

### Platform Execute {#platform_execute}

**Check ID:** `platform_execute` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Session Manager ရှိ ဗလာမဟုတ်သော `PlatformExecute` entry တစ်ခုစီကို ဖတ်သည်။ ၎င်းသည် platform ဆိုင်ရာ boot စတင်ပြင်ဆင်မှုအတွက် ရွေးချယ်ထားသော ပရိုဂရမ်များကို ဖော်ထုတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**စုံစမ်းစစ်ဆေးနည်း:** Hardware သို့မဟုတ် provisioning လုပ်ငန်းစဉ်များက entry ကို ရှင်းပြနိုင်သည်။ ဤ Windows build ၏ boot လမ်းကြောင်းက ၎င်းကို အသုံးပြုခြင်း ရှိ/မရှိ ဆုံးဖြတ်ပြီး value အမည်မှ တိုက်ခိုက်မှုဟု ကောက်ချက်ချမည့်အစား ရည်ညွှန်းထားသော image ကို စုံစမ်းစစ်ဆေးပါ။

### RDP Client Extension DLL {#rdp_clx_dll}

**Check ID:** `rdp_clx_dll` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Terminal Server ၏ ပုံသေ အသုံးပြုသူ ပြင်ဆင်မှုရှိ `ClxDllPath` ကို ဖြစ်နိုင်ချေရှိသော RDP ဆိုင်ရာ extension တည်နေရာအဖြစ် ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**စုံစမ်းစစ်ဆေးနည်း:** Windows/RDP ဗားရှင်းနှင့် ထည့်သွင်းထားသော extension များကို စစ်ဆေးပါ။ ပြင်ဆင်ထားသော DLL သည် image-load ခွဲခြမ်းစိတ်ဖြာမှုအတွက် သဲလွန်စတစ်ခု ဖြစ်သည်။ Registry value တစ်ခုတည်းက RDP ချိတ်ဆက်မှု သို့မဟုတ် DLL load ဖြစ်ပေါ်ခဲ့ကြောင်း မပြပါ။

### RDP Virtual Channel DLL {#rdp_virtual_channel}

**Check ID:** `rdp_virtual_channel` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** စက်တစ်ခုလုံးဆိုင်ရာ Terminal Server Client `Default\AddIns` registration များအောက်ရှိ value များကို စာရင်းကောက်သည်။ Binary value များကို ဖြစ်နိုင်သည့်အခါ UTF-16LE အဖြစ် decode လုပ်ပြီး အသုံးပြုနိုင်သော extension path များကို ထိန်းသိမ်းသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: value အားလုံး။

**စုံစမ်းစစ်ဆေးနည်း:** RDP client များနှင့် device-redirection ထုတ်ကုန်များသည် add-in များကို တရားဝင် register လုပ်ကြသည်။ အမှန်တကယ် DLL သို့မဟုတ် ညွှန်ကြားချက်နှင့် သက်ဆိုင်ရာ client session ကို စစ်ဆေးပါ။ Add-in အောက်ရှိ value တိုင်းသည် executable path မဟုတ်ပါ။

### RDP WDS Startup Programs {#rdp_wds_startup}

**Check ID:** `rdp_wds_startup` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** RDP Windows Station driver ပြင်ဆင်မှုမှ `StartupPrograms` ကို ဖတ်သည်။ ၎င်းသည် remote-session startup အတွက် ပြင်ဆင်ထားသော ပရိုဂရမ်များကို ဖော်ထုတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**စုံစမ်းစစ်ဆေးနည်း:** `rdpclip` သည် ရင်းနှီးသော entry ဖြစ်သည်။ ထပ်ထည့်ထားသည်များနှင့် host ၏ RDP ပြင်ဆင်မှုကို စုံစမ်းစစ်ဆေးပြီး ပြင်ဆင်ထားသော ပရိုဂရမ် အမှန်တကယ် စတင်ခဲ့ခြင်း ရှိ/မရှိ ဆုံးဖြတ်ရန် session/process မှတ်တမ်းများကို အသုံးပြုပါ။

### Registry Run Keys {#run_keys}

**Check ID:** `run_keys` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** စက်/အသုံးပြုသူ hive များမှ Run၊ RunOnce၊ RunEx၊ RunOnceEx နှင့် policy Explorer Run value များအပြင် ဖော်ပြထားသော စက် 32-bit key များကို စာရင်းကောက်သည်။ RunEx/RunOnceEx အပိုင်းများကို ကန့်သတ်ချက်များနှင့် cycle စစ်ဆေးမှုများဖြင့် traverse လုပ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | ဗလာမဟုတ်သော value အားလုံး |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | ဗလာမဟုတ်သော value အားလုံး |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | ဗလာမဟုတ်သော value အားလုံး |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | ဗလာမဟုတ်သော value အားလုံး |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | ဗလာမဟုတ်သော value အားလုံး |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | ဗလာမဟုတ်သော value အားလုံး |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | ဗလာမဟုတ်သော value အားလုံး |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | ဗလာမဟုတ်သော value အားလုံး |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | ဗလာမဟုတ်သော value အားလုံး |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | ဗလာမဟုတ်သော value အားလုံး |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | ဗလာမဟုတ်သော value အားလုံး |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | ဗလာမဟုတ်သော value အားလုံး |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** ဆော့ဖ်ဝဲ updater များနှင့် tray အပလီကေးရှင်းများ ဤနေရာတွင် အများအားဖြင့် ပေါ်လာသည်။ Executable၊ ညွှန်ကြားချက် argument များနှင့် အသုံးပြုသူ အခြေအနေကို နှိုင်းယှဉ်ပါ။ RunOnce entry သည် အသုံးပြုပြီးနောက် ပျောက်သွားနိုင်သည်။ ဖမ်းယူထားသော အခြေအနေသည် ယခင် sign-in များ၏ ပြည့်စုံသော မှတ်တမ်း မဟုတ်ပါ။

### RunServices (Legacy) {#run_services}

**Check ID:** `run_services` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** စက်တစ်ခုလုံးဆိုင်ရာ legacy `RunServices` key ရှိ value များကို စာရင်းကောက်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** ၎င်းသည် သမိုင်းဝင် ပြင်ဆင်မှု မျက်နှာပြင်ဖြစ်ပြီး ခေတ်သစ် Windows build က ၎င်းကို လုပ်ဆောင်မည်ဟူသော အာမခံချက် မဟုတ်ပါ။ ပစ်မှတ်နှင့် operating-system သက်ဆိုင်မှုကို စုံစမ်းစစ်ဆေးပြီးနောက် သီးခြားလွတ်လပ်သော လုပ်ဆောင်မှု အထောက်အထားကို ရှာပါ။

### RunServicesOnce (Legacy) {#run_services_once}

**Check ID:** `run_services_once` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** စက်တစ်ခုလုံးဆိုင်ရာ legacy `RunServicesOnce` key ရှိ value များကို စာရင်းကောက်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** ဗလာမဟုတ်သော value များကို ပြင်ဆင်မှု သဲလွန်စများအဖြစ် သဘောထားပါ။ Source Windows ဗားရှင်းက ယန္တရားကို ပံ့ပိုးခြင်း ရှိ/မရှိနှင့် deployment သို့မဟုတ် compatibility ဆော့ဖ်ဝဲက entry ကို ရှင်းပြနိုင်ခြင်း ရှိ/မရှိ စစ်ဆေးပါ။

### S0 Initial Command {#s0_initial_command}

**Check ID:** `s0_initial_command` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Session Manager မှ `S0InitialCommand` ကို ဖြစ်နိုင်ချေရှိသော Session 0 စတင်ပြင်ဆင်မှု ညွှန်ကြားချက်အဖြစ် ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**စုံစမ်းစစ်ဆေးနည်း:** ဖမ်းယူထားသော Windows build နှင့် boot အခြေအနေအပေါ် သက်ဆိုင်မှုကို အတည်ပြုပါ။ ပြင်ဆင်ထားသော ညွှန်ကြားချက်သည် boot လုပ်ဆောင်မှု၏ timestamp ပါ မှတ်တမ်း မဟုတ်ပါ။ ၎င်း၏ ဖိုင်ကို စစ်ဆေးပြီး boot/process telemetry နှင့် ချိတ်ဆက်စစ်ဆေးပါ။

### Session Manager Execute {#session_manager_execute}

**Check ID:** `session_manager_execute` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Session Manager မှ ဗလာမဟုတ်သော `Execute` entry များကို ဖတ်ပြီး session စတင်ပြင်ဆင်မှုအတွက် ရွေးချယ်ထားသော ပရိုဂရမ်များကို ဖော်ပြသည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**စုံစမ်းစစ်ဆေးနည်း:** မျှော်လင့်ထားသော image baseline နှင့် နှိုင်းယှဉ်ပြီး ပုံမှန်မဟုတ်သော native executable များ သို့မဟုတ် path များကို စစ်ဆေးပါ။ လုပ်ဆောင်မှုကို အခိုင်အမာ မဆိုမီ ပြောင်းလဲမှုနောက်ပိုင်း သက်ဆိုင်ရာ boot/session ဖြစ်ပေါ်ခဲ့ခြင်း ရှိ/မရှိ အတည်ပြုပါ။

### Session Manager SubSystems {#session_manager_subsystems}

**Check ID:** `session_manager_subsystems` · **ကတ်တလောက် ATT&CK:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei စစ်ဆေးသည့်အရာ:** Windows subsystem ညွှန်ကြားချက်ကို ပြင်ဆင်ပေးသော Session Manager ၏ `SubSystems` key ရှိ `Windows` value ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**စုံစမ်းစစ်ဆေးနည်း:** ပုံမှန်ညွှန်ကြားချက်သည် ရှည်လျားပြီး ဗားရှင်းပေါ် မူတည်သည်။ Value နှင့် argument အပြည့်အစုံကို တူညီသော build ၏ ယုံကြည်ရသော baseline နှင့် နှိုင်းယှဉ်ပါ။ အရေးကြီးသော subsystem ဆက်တင်ကို ပြောင်းလဲခြင်းသည် startup ကိုလည်း ပျက်စီးစေနိုင်သည်။

### Setup Execute {#setup_execute}

**Check ID:** `setup_execute` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Session Manager ရှိ `SetupExecute` entry များကို ဖတ်ပြီး startup အတွက် ပြင်ဆင်ထားသော setup ဆိုင်ရာ ညွှန်ကြားချက်များကို လွှမ်းခြုံသည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**စုံစမ်းစစ်ဆေးနည်း:** ထည့်သွင်းခြင်း၊ upgrade လုပ်ခြင်းနှင့် ပြန်လည်ရယူခြင်း လုပ်ငန်းများက တရားဝင် entry များကို ဖန်တီးနိုင်သည်။ ပြင်ဆင်မှုကို persistence ဟု သတ်မှတ်မီ setup log များနှင့် ရည်ညွှန်းထားသော image တို့နှင့် နှိုင်းယှဉ်ပါ။

### Shell Folders Startup Redirect {#shell_folders_startup}

**Check ID:** `shell_folders_startup` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Shell Folders နှင့် User Shell Folders နှစ်ခုစလုံးရှိ `Startup` နှင့် `Common Startup` ကို ပုံသေ Startup တည်နေရာများနှင့် နှိုင်းယှဉ်သည်။ ပုံသေမဟုတ်သော redirection များကို အစီရင်ခံပြီး ၎င်းတို့၏ စုဆောင်းထားသော ဖိုင်တွဲ အကြောင်းအရာများကို စစ်ဆေးသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` နှင့် `User Shell Folders`: `Common Startup`။
- NTUSER.DAT ရှိ သက်ဆိုင်ရာ `Software\Microsoft\Windows\CurrentVersion\Explorer` ဆက်တင်များ- `Startup` နှင့် redirect လုပ်ထားသော တည်နေရာများရှိ ဖမ်းယူထားသော ဖိုင်များ။

**စုံစမ်းစစ်ဆေးနည်း:** Redirect လုပ်ထားသော Startup ဖိုင်တွဲသည် autostart များကို ဖုံးကွယ်နိုင်သော်လည်း redirection သည် တရားဝင်လည်း ဖြစ်နိုင်သည်။ Path သည် ဖမ်းယူထားသော အထောက်အထားအတွင်း ဖော်ထုတ်နိုင်ရမည်။ မစုဆောင်းရသေးသော ဦးတည်ရာသည် မသိရှိရခြင်းဖြစ်ပြီး ဖိုင်တွဲ ဗလာဖြစ်ခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။

### Shell Launcher Override {#shell_launcher}

**Check ID:** `shell_launcher` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** SOFTWARE မှ policy `Shell` value နှင့် `system.ini` boot Shell mapping ကို ဖတ်သည်။ ၎င်းတို့သည် shell ရွေးချယ်မှုအတွက် အခြားရွေးချယ်စရာ မျက်နှာပြင်များ ဖြစ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**စုံစမ်းစစ်ဆေးနည်း:** Kiosk များနှင့် application appliance များသည် Explorer ကို တမင်အစားထိုးနိုင်သည်။ ရွေးချယ်ထားသော shell ကို စက်၏ အခန်းကဏ္ဍနှင့် နှိုင်းယှဉ်ပြီး သက်ဆိုင်ရာ session အတွက် ဆက်တင် အမှန်တကယ် သက်ရောက်ခဲ့ကြောင်း အတည်ပြုပါ။

### Startup Folder {#startup_folder}

**Check ID:** `startup_folder` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** ပုံသေ အသုံးပြုသူအလိုက်နှင့် common Startup ဖိုင်တွဲများရှိ တိုက်ရိုက်ဖိုင်များကို ပထမဆုံး ရရှိနိုင်သော Startup-folder redirection ကို လိုက်၍ စစ်ဆေးသည်။ စုဆောင်းထားသော `.lnk` ကို ခွဲခြမ်းစိတ်ဖြာနိုင်သည့်အခါ ဖြတ်လမ်း ပစ်မှတ်များကို ပြန်လည်ရယူသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` နှင့် `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`။
- စက်/အသုံးပြုသူ Shell Folders နှင့် User Shell Folders ဆက်တင်များရှိ သက်ဆိုင်ရာ `Common Startup` / `Startup` redirection များ။

**စုံစမ်းစစ်ဆေးနည်း:** Startup link များနှင့် ဖိုင်များသည် sign-in တွင် တရားဝင် အပလီကေးရှင်းများကို စတင်ဖွင့်နိုင်သည်။ ဖိုင်အမည်များအပြင် ပစ်မှတ် argument များကိုလည်း စစ်ဆေးပါ။ Common ဖိုင်တွဲသည် မျှဝေထားသော ပြင်ဆင်မှု တည်နေရာဖြစ်ပြီး စတင်ခဲ့သော ပရိုဂရမ်က SYSTEM token ကို အသုံးပြုခဲ့ကြောင်း သက်သေမပြပါ။

### Terminal Services Initial Program {#ts_initial_program}

**Check ID:** `ts_initial_program` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Terminal Services policy ၏ `InitialProgram` နှင့် စက်၏ Terminal Server installation Run/RunOnce/RunOnceEx ပြင်ဆင်မှု (ကန့်သတ်ထားသော nested RunOnceEx အပိုင်းများ အပါအဝင်) ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | ဗလာမဟုတ်သော value အားလုံး |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | ဗလာမဟုတ်သော value အားလုံး |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** Remote application deployment များသည် ဤဆက်တင်များကို တမင်အသုံးပြုနိုင်သည်။ သက်ရောက်ခံရသော session များကို အတည်ပြုပြီး ရွေးချယ်ထားသော ညွှန်ကြားချက်များကို process အထောက်အထားထဲသို့ လိုက်ကြည့်ပါ။ Registration တစ်ခုတည်းက remote logon ကို မပြပါ။

### Winlogon AppSetup {#winlogon_appsetup}

**Check ID:** `winlogon_appsetup` · **ကတ်တလောက် ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei စစ်ဆေးသည့်အရာ:** Winlogon မှ `AppSetup` ကို ဖတ်ပြီး ကော်မာဖြင့် ခွဲထားသော entry များကို သီးခြား ကိုယ်စားလှယ်လောင်းများအဖြစ် ခွဲထုတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**စုံစမ်းစစ်ဆေးနည်း:** Component တစ်ခုစီကို မျှော်လင့်ထားသော Windows setup/sign-in ဆော့ဖ်ဝဲနှင့် နှိုင်းယှဉ်ပါ။ စာရင်းထဲရှိ entry တစ်ခုသည် သီးခြားလေ့လာတွေ့ရှိထားသော လုပ်ဆောင်မှု မဟုတ်ဘဲ ၎င်း၏ သက်ဆိုင်မှုသည် Windows ဗားရှင်းနှင့် deployment အခြေအနေပေါ် မူတည်နိုင်သည်။

### Winlogon GinaDLL {#winlogon_gina_dll}

**Check ID:** `winlogon_gina_dll` · **ကတ်တလောက် ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei စစ်ဆေးသည့်အရာ:** Legacy Winlogon `GinaDLL` အစားထိုးဆက်တင်ကို ဖတ်သည်။ GINA သည် Windows sign-in extension မော်ဒယ်အဟောင်း ဖြစ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**စုံစမ်းစစ်ဆေးနည်း:** ၎င်းသည် အဓိကအားဖြင့် Vista မတိုင်မီ စနစ်များနှင့် သက်ဆိုင်သည်။ ပိုသစ်သော စနစ်ပေါ်တွင် ကျန်ရှိနေသော key သည် သက်ရောက်မှုရှိသော logon extension ၏ သက်သေ မဟုတ်ပါ။ Source ဗားရှင်းနှင့် ရည်ညွှန်းထားသော DLL တစ်ခုခုကို စစ်ဆေးပါ။

**နောက်ထပ် အသေးစိတ်များ:** [Microsoft စာရွက်စာတမ်း](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina)။

### Winlogon MPNotify {#winlogon_mpnotify}

**Check ID:** `winlogon_mpnotify` · **ကတ်တလောက် ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei စစ်ဆေးသည့်အရာ:** sign-in ဆိုင်ရာ extension မျက်နှာပြင်ဖြစ်သော Winlogon ၏ `mpnotify` command/library ဆက်တင်ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**စုံစမ်းစစ်ဆေးနည်း:** ပစ်မှတ်နှင့် argument များကို ကိုက်ညီသော Windows baseline နှင့် နှိုင်းယှဉ်ပါ။ Code run ခဲ့သည်ဟု အခိုင်အမာ မဆိုမီ ဗားရှင်းအလိုက် အပြုအမူကို စစ်ဆေးပြီး နောက်ဆက်တွဲ sign-in နှင့် ချိတ်ဆက်စစ်ဆေးပါ။

### Winlogon Notify Packages {#winlogon_notify_packages}

**Check ID:** `winlogon_notify_packages` · **ကတ်တလောက် ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei စစ်ဆေးသည့်အရာ:** တိုက်ရိုက် Winlogon `Notify` package registration တစ်ခုစီမှ `DllName` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**စုံစမ်းစစ်ဆေးနည်း:** Notification package များသည် legacy extension ယန္တရား ဖြစ်သည်။ ဖမ်းယူထားသော Windows ဗားရှင်းက ၎င်းတို့ကို load လုပ်ခြင်း ရှိ/မရှိ ဆုံးဖြတ်ပြီး ခွင့်ပြုထားသော authentication ဆော့ဖ်ဝဲကို ရှင်းပြ၍မရသော library များမှ ခွဲခြားပါ။

### Winlogon Shell {#winlogon_shell}

**Check ID:** `winlogon_shell` · **ကတ်တလောက် ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei စစ်ဆေးသည့်အရာ:** Desktop shell ရွေးချယ်မှုကို ဖော်ပြရန် စက်တစ်ခုလုံးဆိုင်ရာနှင့် အသုံးပြုသူအလိုက် Winlogon `Shell` value များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**စုံစမ်းစစ်ဆေးနည်း:** Explorer သည် အသုံးများသော်လည်း kiosk များသည် အခြား shell ကို အသုံးပြုနိုင်သည်။ ထပ်ထည့်ထားသော ညွှန်ကြားချက်များ သို့မဟုတ် ပုံမှန်မဟုတ်သော path များကို စုံစမ်းစစ်ဆေးပြီး sign-in တွင် မည်သည့်အသုံးပြုသူ၏ သက်ရောက်မှုရှိသော ပြင်ဆင်မှု သက်ဆိုင်ခဲ့သည်ကို ဆုံးဖြတ်ပါ။

### Winlogon System {#winlogon_system}

**Check ID:** `winlogon_system` · **ကတ်တလောက် ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei စစ်ဆေးသည့်အရာ:** Winlogon ၏ `System` value ကို ဖတ်ပြီး ကော်မာဖြင့် ခွဲထားသော entry များကို ကိုယ်စားလှယ်လောင်း တစ်ခုချင်းစီအဖြစ် ခွဲထုတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**စုံစမ်းစစ်ဆေးနည်း:** ၎င်းကို ဗားရှင်းအလိုက် ကွဲပြားသော sign-in ပြင်ဆင်မှု သဲလွန်စအဖြစ် သဘောထားပါ။ ရည်ညွှန်းထားသော component တိုင်းကို စစ်ဆေးပြီး စာရင်းကို run မှတ်တမ်းအဖြစ် မသဘောထားဘဲ လုပ်ဆောင်မှု အထောက်အထားဖြင့် အတည်ပြုပါ။

### Winlogon Taskman {#winlogon_taskman}

**Check ID:** `winlogon_taskman` · **ကတ်တလောက် ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei စစ်ဆေးသည့်အရာ:** အစားထိုး task-management ညွှန်ကြားချက်အတွက် စက်/အသုံးပြုသူ Winlogon `Taskman` value များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**စုံစမ်းစစ်ဆေးနည်း:** မည်သည့် Windows အပြုအမူက ဆက်တင်ကို အသုံးပြုမည်ဖြစ်ကြောင်းနှင့် ပစ်မှတ်သည် မျှော်လင့်ထားသည့်အရာ ဟုတ်/မဟုတ် ဆုံးဖြတ်ပါ။ ပြင်ဆင်ထားသော အစားထိုးမှုသည် အသုံးပြုသူက ထို feature ကို ခေါ်ယူခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။

### Winlogon Userinit {#winlogon_userinit}

**Check ID:** `winlogon_userinit` · **ကတ်တလောက် ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei စစ်ဆေးသည့်အရာ:** စက်/အသုံးပြုသူ Winlogon `Userinit` value များကို ဖတ်ပြီး ကော်မာဖြင့် ခွဲထားသော ပရိုဂရမ်များကို ခွဲထုတ်ကာ နောက်ဆုံးရှိ ဗလာ item များကို လျစ်လျူရှုသည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**စုံစမ်းစစ်ဆေးနည်း:** ပုံမှန် entry သည် `userinit.exe` ကို ညွှန်ပြလေ့ရှိသည်။ အပို entry များနှင့် ပြောင်းလဲထားသော path များကို စစ်ဆေးပါ။ သက်ဆိုင်ရာ အသုံးပြုသူနှင့် နောက်ပိုင်း sign-in ကို အတည်ပြုပါ။ `KeyLastWrite` သည် ဤ value အတွက် သီးသန့်မဟုတ်ဘဲ key တစ်ခုလုံးနှင့် သက်ဆိုင်သည်။

### Winlogon VMApplet {#winlogon_vmapplet}

**Check ID:** `winlogon_vmapplet` · **ကတ်တလောက် ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei စစ်ဆေးသည့်အရာ:** Winlogon ၏ `VMApplet` ညွှန်ကြားချက် ဆက်တင်ကို ဖတ်ပြီး စနစ် applet အဟောင်း စတင်ဖွင့်မှု မျက်နှာပြင်ကို လွှမ်းခြုံသည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**စုံစမ်းစစ်ဆေးနည်း:** Operating-system သက်ဆိုင်မှုနှင့် မျှော်လင့်ထားသော applet ကို စစ်ဆေးပါ။ မမျှော်လင့်ထားသော ညွှန်ကြားချက်သည် စုံစမ်းစစ်ဆေးထိုက်သော်လည်း trigger လုပ်သော interface ကို ဖွင့်ခဲ့ကြောင်း မသတ်မှတ်နိုင်ပါ။

## ဝန်ဆောင်မှုများနှင့် စနစ် extension များ { #services-and-system-extensions }

Windows ဝန်ဆောင်မှုများနှင့် ချဲ့ထွင်နိုင်သော စနစ် component များသည် executable များ သို့မဟုတ် DLL များကို load လုပ်ရန် တရားဝင်နည်းလမ်း များစွာကို ပေးသည်။ ပြင်ဆင်ထားသော ပစ်မှတ်ကို မျှော်လင့်ထားသော ထုတ်ကုန်၊ ဝန်ဆောင်မှုအကောင့်၊ start ဆက်တင်များနှင့် source အခြေပြု ထည့်သွင်းမှုမှတ်တမ်းတို့နှင့် နှိုင်းယှဉ်ပါ။

### DiagTrack DLL {#diagtrack_dll}

**Check ID:** `diagtrack_dll` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** DiagTrack ဝန်ဆောင်မှု၏ `ImagePath` ကို ဖတ်သည်။ ကတ်တလောက်၏ DLL ဦးတည်သော အမည်ရှိသော်လည်း ဤစစ်ဆေးမှုသည် ပြင်ဆင်ထားသော service image ညွှန်ကြားချက်ကို စစ်ဆေးသည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**စုံစမ်းစစ်ဆေးနည်း:** ဝန်ဆောင်မှု baseline နှင့် ယေဘုယျ ဝန်ဆောင်မှု စစ်ဆေးမှုများနှင့် နှိုင်းယှဉ်ပါ။ တူညီသော ပြင်ဆင်မှုသည် စစ်ဆေးမှုရလဒ် တစ်ခုထက်ပို၍ ထုတ်ပေးနိုင်သည်။ ကိုယ်စားလှယ်လောင်း အရေအတွက်များသည် ကွဲပြားသော တိုက်ခိုက်မှု အရေအတွက်များ မဟုတ်ပါ။

### DiagTrack Listener DLL {#diagtrack_listener_dll}

**Check ID:** `diagtrack_listener_dll` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** DiagTrack-Listener autologger ပြင်ဆင်မှုမှ `FileName` ကို ဖတ်သည်။ ၎င်းသည် ETW autologger ဆက်တင်ဖြစ်ပြီး DLL load လုပ်ခံရကြောင်း အထောက်အထား မဟုတ်ပါ။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**စုံစမ်းစစ်ဆေးနည်း:** Raw value နှင့် Windows logging ပြင်ဆင်မှုကို စစ်ဆေးပါ။ ကတ်တလောက်က ၎င်းကို DLL technique အဖြစ် အမှတ်အသားပြုထားသော်လည်း ရှာဖွေမှုသည် load လုပ်နိုင်သော DLL ကို စစ်ဆေးအတည်ပြုခြင်း သို့မဟုတ် ဤ value မှတစ်ဆင့် လုပ်ဆောင်မှုကို သက်သေပြခြင်း မပြုပါ။

### Netsh Helper DLL {#netsh_helper}

**Check ID:** `netsh_helper` · **ကတ်တလောက် ATT&CK:** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**Juntei စစ်ဆေးသည့်အရာ:** စက်၏ `NetSh` helper registration key အောက်ရှိ ဗလာမဟုတ်သော value များကို စာရင်းကောက်သည်။ Helper များသည် `netsh.exe` ကို DLL အခြေပြု ညွှန်ကြားချက်များဖြင့် ချဲ့ထွင်နိုင်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** ကွန်ရက် ထုတ်ကုန်များက helper များကို ထည့်သွင်းနိုင်သည်။ Value တစ်ခုစီကို မျှော်လင့်ထားသော ထုတ်ကုန်နှင့် ကိုက်ညီအောင် စစ်ဆေးပြီး ဖော်ထုတ်ထားသော library ကို စစ်ဆေးပါ၊ ထို့နောက် သက်ဆိုင်ရာ netsh လုပ်ဆောင်မှု သို့မဟုတ် image load များကို ရှာပါ။

### Print Monitors {#print_monitors}

**Check ID:** `print_monitors` · **ကတ်တလောက် ATT&CK:** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**Juntei စစ်ဆေးသည့်အရာ:** Active control set ရှိ တိုက်ရိုက် Print Monitor registration တစ်ခုစီအောက်မှ `Driver` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**စုံစမ်းစစ်ဆေးနည်း:** Printing ဆော့ဖ်ဝဲနှင့် port monitor များသည် spooler ကို တရားဝင် ချဲ့ထွင်ကြသည်။ DLL နှင့် ထုတ်ကုန် metadata ကို printer စာရင်းနှင့် နှိုင်းယှဉ်ပြီး spooler/image-load အထောက်အထားဖြင့် load လုပ်ခဲ့ခြင်းကို အတည်ပြုပါ။

### Print Processors {#print_processors}

**Check ID:** `print_processors` · **ကတ်တလောက် ATT&CK:** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**Juntei စစ်ဆေးသည့်အရာ:** x64 နှင့် x86 print-processor registration tree များအောက်မှ `Driver` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**စုံစမ်းစစ်ဆေးနည်း:** ပြင်ပ printer suite များသည် ရင်းနှီးသော Windows ပုံသေများအပြင် processor များကို ထပ်ထည့်နိုင်သည်။ အန္တရာယ်ရှိသော ရည်ရွယ်ချက်ဟု သတ်မှတ်မီ architecture နှစ်ခုစလုံး၊ အမှန်တကယ် DLL နှင့် ထည့်သွင်းထားသော printer ဆော့ဖ်ဝဲကို ပြန်လည်သုံးသပ်ပါ။

### Service Control Manager Extension {#scm_extension}

**Check ID:** `scm_extension` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** service-manager extension မျက်နှာပြင်ဖြစ်သော `ServiceControlManagerExtension` ရှိ ဗလာမဟုတ်သော value များကို စာရင်းကောက်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** အမည်ဖော်ထားသော library များနှင့် ဤ Windows build က ဆက်တင်ကို အသုံးပြုခြင်း ရှိ/မရှိကို စစ်ဆေးပါ။ Boot/service လှုပ်ရှားမှုနှင့် ချိတ်ဆက်စစ်ဆေးပါ။ စစ်ဆေးမှုသည် load လုပ်ထားသော extension ကို သက်သေပြခြင်းထက် registration များကို စာရင်းကောက်ခြင်း ဖြစ်သည်။

### DNS Server Level Plugin DLL {#server_level_plugin_dll}

**Check ID:** `server_level_plugin_dll` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Active SYSTEM control set ရှိ DNS ဝန်ဆောင်မှု၏ `ServerLevelPluginDll` ဆက်တင်ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**စုံစမ်းစစ်ဆေးနည်း:** ၎င်းသည် domain controller အချို့ အပါအဝင် DNS Server role ကို run နေသော စနစ်များနှင့် သက်ဆိုင်သည်။ DLL၊ ဝန်ဆောင်မှုအကောင့်နှင့် ဝန်ဆောင်မှု restart/load အထောက်အထားကို စုံစမ်းစစ်ဆေးပါ။ Role active မဖြစ်ခြင်း သို့မဟုတ် မရှိခြင်းက ကောက်ချက်ကို ပြောင်းလဲစေသည်။

### Service Failure Command {#service_failure_command}

**Check ID:** `service_failure_command` · **ကတ်တလောက် ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei စစ်ဆေးသည့်အရာ:** Active control set ရှိ ဝန်ဆောင်မှုတစ်ခုစီ၏ `FailureCommand` ကို ဖတ်ပြီး ပြင်ဆင်ထားသော recovery ညွှန်ကြားချက်ကို ဖော်ပြသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`။

**စုံစမ်းစစ်ဆေးနည်း:** ဝန်ဆောင်မှု၏ recovery action များက ၎င်းကို ရွေးချယ်ပြီး သက်ဆိုင်ရာ failure ဖြစ်ပေါ်သည့်အခါမှသာ ညွှန်ကြားချက် run သည်။ Recovery ပြင်ဆင်မှုနှင့် failure/process မှတ်တမ်းများကို စစ်ဆေးပါ။ Value တစ်ခုတည်းက active recovery action ကို မသတ်မှတ်နိုင်ပါ။

### SNMP Extension Agent DLL {#snmp_extension_agent}

**Check ID:** `snmp_extension_agent` · **ကတ်တလောက် ATT&CK:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei စစ်ဆေးသည့်အရာ:** SNMP ဝန်ဆောင်မှု၏ `ExtensionAgents` registration key အောက်ရှိ value များကို စာရင်းကောက်သည်။ Value များသည် တိုက်ရိုက် DLL path အစား အခြား registry တည်နေရာကို ဖော်ပြနိုင်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** သွယ်ဝိုက် registration ကို ၎င်း၏ library အထိ လက်ဖြင့် လိုက်ကြည့်ပြီး SNMP ထည့်သွင်းထားကာ အသုံးပြုနေကြောင်း စစ်ဆေးအတည်ပြုပါ။ ဤထုတ်ယူမှုရှိ ဖော်ထုတ်မရသော string သည် DLL ပျောက်နေကြောင်း သို့မဟုတ် အန္တရာယ်ရှိကြောင်း သက်သေ မဟုတ်ပါ။

### Time Providers {#time_providers}

**Check ID:** `time_providers` · **ကတ်တလောက် ATT&CK:** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**Juntei စစ်ဆေးသည့်အရာ:** တိုက်ရိုက် Windows Time `TimeProviders` registration များမှ `DllName` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**စုံစမ်းစစ်ဆေးနည်း:** Provider enable ဆက်တင်များနှင့် Windows Time ဝန်ဆောင်မှု အခြေအနေကို သီးခြားစီ ပြန်လည်သုံးသပ်ပါ။ DLL path စစ်ဆေးမှုသည် enable လုပ်ထားသော provider ကို မလိုအပ်ပါ။ Time synchronization ထုတ်ကုန်များက တရားဝင် အစားထိုးမှုများကို ပေးနိုင်သည်။

### Windows Service (ServiceDll) {#windows_service_dll}

**Check ID:** `windows_service_dll` · **ကတ်တလောက် ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei စစ်ဆေးသည့်အရာ:** Active SYSTEM control set ရှိ ဝန်ဆောင်မှုများအောက်မှ `Parameters\ServiceDll` ကို ဖတ်ပြီး ရရှိနိုင်သည့်အခါ ဝန်ဆောင်မှုအလိုက် timestamp အထောက်အထားကို ပူးတွဲပေးသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`။

**စုံစမ်းစစ်ဆေးနည်း:** `svchost.exe` ကဲ့သို့သော မျှဝေထားသော host သည် payload DLL မဟုတ်ပါ။ Library၊ ဝန်ဆောင်မှုအကောင့်၊ start အခြေအနေနှင့် ကိုက်ညီသော ဝန်ဆောင်မှု ထည့်သွင်းမှု/image-load အထောက်အထားကို စစ်ဆေးပါ။ Registration သည် ဝန်ဆောင်မှု run ခဲ့ကြောင်း သက်သေမပြပါ။

**နောက်ထပ် အသေးစိတ်များ:** [Microsoft စာရွက်စာတမ်း](https://docs.microsoft.com/en-us/windows/win32/services/service-programs)။

### Windows Service (ImagePath) {#windows_service_image_path}

**Check ID:** `windows_service_image_path` · **ကတ်တလောက် ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei စစ်ဆေးသည့်အရာ:** Active control set ရှိ ဝန်ဆောင်မှု `ImagePath` ညွှန်ကြားချက်များကို ဖတ်သည်။ ImagePath မပါသော driver type 1၊ 2 သို့မဟုတ် 8 တို့သည် သွယ်ဝိုက်ဆိုလိုထားသော `System32\drivers\<service>.sys` ပစ်မှတ်ကို ထိန်းသိမ်းထားသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath`၊ `Type` နှင့် ထောက်ပံ့ပေးသော ဝန်ဆောင်မှု metadata။

**စုံစမ်းစစ်ဆေးနည်း:** ဝန်ဆောင်မှုအကောင့်နှင့် start ဆက်တင်များကို ထုတ်ယူမှု၏ machine scope label နှင့် သီးခြားစီ ပြန်လည်သုံးသပ်ပါ။ Disabled သို့မဟုတ် demand-start registration များသည်လည်း စာရင်းတွင် ပါဝင်သည်။ Timestamp များအတွက် မရေရာသော မျှဝေထားသည့် executable path များထက် ဝန်ဆောင်မှုအမည် အထောက်အထားကို ဦးစားပေးသည်။

**နောက်ထပ် အသေးစိတ်များ:** [Microsoft စာရွက်စာတမ်း](https://docs.microsoft.com/en-us/windows/win32/services/service-programs)။

### Windows Update Service Startup DLL {#wu_service_startup_dll}

**Check ID:** `wu_service_startup_dll` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Active control set မှ Windows Update ဝန်ဆောင်မှု၏ `Parameters\ServiceDll` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**စုံစမ်းစစ်ဆေးနည်း:** Library ကို တူညီသော Windows build နှင့် ထည့်သွင်းထားသော servicing အခြေအနေတို့နှင့် နှိုင်းယှဉ်ပါ။ ၎င်းသည် ယေဘုယျ ServiceDll စာရင်းနှင့် ထပ်နေပြီး ဒုတိယ သီးခြား compromise တစ်ခုကို ကိုယ်စားမပြုပါ။

## အချိန်ဇယားသတ်မှတ်ထားသော task များ { #scheduled-tasks }

Task Scheduler သည် ပြင်ဆင်မှုကို XML ဖိုင်များနှင့် SOFTWARE hive နှစ်ခုစလုံးတွင် သိမ်းဆည်းသည်။ ထို source များကို အတူတကွ ဖတ်ပါ။ Task တစ်ခုသည် run ခြင်းမရှိဘဲ register လုပ်ထားနိုင်ပြီး မပြည့်စုံသော စုဆောင်းမှုသည် hidden သို့မဟုတ် ghost task နှင့် ဆင်တူနိုင်သည်။

### Ghost Scheduled Task {#ghost_task}

**Check ID:** `ghost_task` · **ကတ်တလောက် ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei စစ်ဆေးသည့်အရာ:** SOFTWARE TaskCache `Tree` နှင့် `Tasks` registration များကို နှိုင်းယှဉ်ပြီး စုဆောင်းမှုထဲတွင် task ဖိုင်များ ရှိသည့်အခါ ၎င်းတို့နှင့် သက်ဆိုင်သော task XML ဖိုင်များကို ရှာဖွေသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` နှင့် `TaskCache\Tasks`။
- ဖိုင်စနစ် နှိုင်းယှဉ်မှုအတွက် ဖမ်းယူထားသော `Windows\System32\Tasks\<task path>` ဖိုင်များ။

**စုံစမ်းစစ်ဆေးနည်း:** ပျောက်နေသော အတွဲဖက်များသည် ဖျက်ထားသော/ခေတ်မမီတော့သော task များ၊ built-in registration များ သို့မဟုတ် မပြည့်စုံသော စုဆောင်းမှုကို ထင်ဟပ်နိုင်သည်။ စုဆောင်းထားသော XML မရှိပါက ဖိုင်စနစ် နှိုင်းယှဉ်မှု မရရှိနိုင်ပါ။ Task ကို hidden သို့မဟုတ် အန္တရာယ်ရှိသည်ဟု မခေါ်မီ coverage နှင့် ကွဲလွဲချက်အတိအကျကို စစ်ဆေးပါ။

### Hidden Scheduled Task {#hidden_scheduled_task}

**Check ID:** `hidden_scheduled_task` · **ကတ်တလောက် ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei စစ်ဆေးသည့်အရာ:** TaskCache Tree registration များတွင် `SD` value ပျောက်နေခြင်း သို့မဟုတ် ဗလာ binary security descriptor ရှိခြင်းကို စစ်ဆေးသည်။ ၎င်းတို့သည် ပုံမှန်စာရင်းကောက်မှုမှ task ကို ဖုံးကွယ်ခြင်းနှင့် ဆက်စပ်သော အခြေအနေများ ဖြစ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: `SD` ရှိ/မရှိ/ဒေတာ၊ TaskCache အတွဲဖက် metadata နှင့်အတူ။

**စုံစမ်းစစ်ဆေးနည်း:** ပျောက်နေသော/ဗလာ SD သည် တိကျသော ပြင်ဆင်မှု ပုံမှန်မဟုတ်မှုဖြစ်ပြီး task လုပ်ဆောင်ခဲ့ကြောင်း သို့မဟုတ် အန္တရာယ်ရှိကြောင်း သက်သေ မဟုတ်ပါ။ XML ၏ `Hidden` ဆက်တင်နှင့် မည်သည့် ACL ပြဿနာမဆိုသည် ကွဲပြားသော အခြေအနေများဖြစ်ပြီး ဤနေရာတွင် အသုံးပြုသော predicate မဟုတ်ပါ။

**နောက်ထပ် အသေးစိတ်များ:** [Microsoft စာရွက်စာတမ်း](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/)။

### Scheduled Task (XML Files) {#scheduled_task_files}

**Check ID:** `scheduled_task_files` · **ကတ်တလောက် ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei စစ်ဆေးသည့်အရာ:** ဖမ်းယူထားသော task XML ကို Exec ညွှန်ကြားချက်/argument များနှင့် COM-handler class ID/ဒေတာများအတွက် ခွဲခြမ်းစိတ်ဖြာသည်။ ၎င်းသည် principal ကို `Actions Context`/`Principal id` ဖြင့် ရွေးချယ်ပြီး အသုံးပြုသူနှင့် group ကိုယ်ပိုင်အမှတ်အသားကို သီးခြားစီ ထိန်းသိမ်းကာ SYSTEM၊ USER သို့မဟုတ် UNKNOWN ကို အစီရင်ခံသည်။ ဖော်ထုတ်မရသော COM action များကို ဆက်လက်မြင်နိုင်အောင် ထားသည်။

**စစ်ဆေးသော အထောက်အထား:**

- Exec၊ ComHandler နှင့် ရရှိနိုင်သော principal field များ အပါအဝင် `Windows\System32\Tasks\<task path>` XML ဖိုင်များ။
- Native နှင့် 32-bit view များတွင် ကိုက်ညီသော အသုံးပြုသူ၏ ဖမ်းယူထားသော `UsrClass.dat` နှင့် NTUSER `Software\Classes` အပြင် စက် SOFTWARE `Classes\CLSID`။
- SID principal များအတွက် ဖမ်းယူထားသော SOFTWARE `ProfileList`။ အမည်ပါသော principal များသည် ထူးခြားစွာ ကိုက်ညီသော `user.DOMAIN` ဖိုင်တွဲကို ဦးစားပေးပြီး ထို့နောက် ဖမ်းယူထားသော local-host suffix များ၊ နောက်ဆုံးတွင် ရိုးရိုး `user` ဖိုင်တွဲကို ဦးစားပေးသည်။ ကိုက်ညီမှုများကို heuristic များအဖြစ် ရှင်းလင်းစွာ အမှတ်အသားပြုထားသည်။ အမှန်တကယ် မရေရာမှုကို ဖော်ထုတ်မထားဘဲ တွေ့ရှိချက်နှင့် coverage တွင် မှတ်ချက်ပြုထားသည်။ Volatile registry key များ မလိုအပ်ပါ။
- Managed `CodeBase` (ဗားရှင်း subkey များ အပါအဝင်)၊ AppID ဆက်နွယ်မှုများနှင့် AppID `LocalService` အတွက် SYSTEM ဝန်ဆောင်မှု `ImagePath`။

COM server ကိုယ်စားလှယ်လောင်းများတွင် ၎င်းတို့၏ registry source ကို `Description` ၌ ထည့်သွင်းထားသည်။ Executable အမည်သာပါသော
AppID ဆက်နွယ်မှုများကို ခန့်မှန်းထားသော directory တစ်ခု သတ်မှတ်ပေးမည့်အစား ဖော်ထုတ်မထားဘဲ ထားသည်။ Local
`file:` URL များကို decode လုပ်သည်။ Remote URL များကို ကွန်ရက်ဝင်ရောက်မှုမရှိဘဲ ထိန်းသိမ်းထားသည်။ အသုံးပြုသူ class override များသည်
ထို profile အတွက်သာ သက်ဆိုင်ပြီး မပြည့်စုံသော အသုံးပြုသူ CLSID သည် ၎င်းက ဖုံးအုပ်ထားသော စက် registration သို့ ဆက်မသွားပါ။
Group သာပါသော task များသည် စက် ကိုယ်စားလှယ်လောင်းများကို တစ်ဦးချင်း အသုံးပြုသူ ကန့်သတ်ချက်နှင့်အတူ ၎င်းတို့၏ ဖော်ပြချက်တွင်
ထိန်းသိမ်းထားပြီး coverage error အကျဉ်းချုပ်တွင် ပုံမှန်သတိပေးချက်များ မထည့်ပါ။

**စုံစမ်းစစ်ဆေးနည်း:** Source XML နှင့် event log များရှိ trigger များ၊ enable အခြေအနေ၊ principal နှင့် run မှတ်တမ်းကို စစ်ဆေးပါ။ ဖန်တီးထားသော task သည် ဘယ်တော့မှ run မည်မဟုတ်နိုင်ပါ။ ဖော်ထုတ်မရသော COM handler သည် ကိုယ်စားလှယ်လောင်း ဖြစ်နေဆဲဖြစ်ပြီး ပျောက်နေသော/group သာပါသော principal ဒေတာကို SYSTEM ဟု မယူဆရပါ။

## WMI၊ BITS နှင့် အခြား event trigger များ { #wmi-bits-and-other-event-triggers }

Windows Management Instrumentation (WMI) သည် စနစ်စီမံခန့်ခွဲမှုကို ပံ့ပိုးသည်။ Background Intelligent Transfer Service (BITS) သည် နောက်ခံ လွှဲပြောင်းမှုများကို ပံ့ပိုးသည်။ နှစ်ခုစလုံးသည် event-triggered ညွှန်ကြားချက်များကို ထိန်းသိမ်းထားနိုင်သည်။ ယန္တရားအချို့သည် စီမံခန့်ခွဲမှု event၊ လွှဲပြောင်းမှု အသိပေးချက်၊ အပလီကေးရှင်း စတင်ဖွင့်မှု သို့မဟုတ် အသုံးပြုသူ လုပ်ဆောင်ချက်ကို စောင့်ဆိုင်းသည်။ သိမ်းဆည်းထားသော trigger သည် အသုံးဝင်သော ပြင်ဆင်မှု အထောက်အထားဖြစ်သော်လည်း ၎င်း၏ တည်ရှိမှုက trigger ဖြစ်ပေါ်ခဲ့ကြောင်း မသတ်မှတ်နိုင်ပါ။

### Accessibility Features Backdoor {#accessibility_tools}

**Check ID:** `accessibility_tools` · **ကတ်တလောက် ATT&CK:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei စစ်ဆေးသည့်အရာ:** စုဆောင်းထားသော reference binary မှ အသုံးပြုနိုင်သော Microsoft catalog metadata ကို လိုအပ်ချက်အဖြစ် သတ်မှတ်ပြီးနောက် စုဆောင်းထားသော System32 accessibility binary ခုနစ်ခုကို PE OriginalFilename နှင့် certificate/catalog metadata ဖြင့် နှိုင်းယှဉ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `Windows\System32\sethc.exe`, `osk.exe`, `narrator.exe`, `magnify.exe`, `utilman.exe`, `atbroker.exe` နှင့် `DisplaySwitch.exe`။
- စုဆောင်းထားသော `Windows\System32\winlogon.exe`၊ `cmd.exe` သို့မဟုတ် `notepad.exe` reference metadata နှင့် သက်ဆိုင်ရာ စုဆောင်းထားသော catalog များ။

**စုံစမ်းစစ်ဆေးနည်း:** Metadata က အမည်ပြောင်းထားသော အစားထိုးမှုကို အရိပ်အမြွက်ပြနိုင်သော်လည်း Juntei သည် signature များ သို့မဟုတ် certificate ယုံကြည်မှုကို စစ်ဆေးအတည်ပြုခြင်း မပြုပါ။ လိုအပ်သော catalog များမပါသော ကျဲပါးသည့် စုဆောင်းမှုသည် signature မပါဟန်တူသော ဖိုင်တိုင်းကို backdoor အဖြစ် သဘောထားမည့်အစား မပြည့်စုံသော နှိုင်းယှဉ်မှု coverage ကို ထုတ်ပေးသည်။

### Accessibility Features {#assistive_technology}

**Check ID:** `assistive_technology` · **ကတ်တလောက် ATT&CK:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei စစ်ဆေးသည့်အရာ:** Native/32-bit SOFTWARE view များရှိ register လုပ်ထားသော accessibility `StartExe` နှင့် `StartParams` ညွှန်ကြားချက်များ၊ အသုံးပြုသူနှင့် DEFAULT hive များမှ enable လုပ်ထားသော accessibility `Configuration` entry များကို ဖတ်သည်။ ဂဏန်း/string မဟုတ်သော StartExe placeholder များကို ကျော်သွားသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` နှင့် `StartParams`; စက်၏ `Wow6432Node` view လည်း ပါဝင်သည်။
- NTUSER.DAT ရှိ `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` နှင့် DEFAULT hive ရှိ သက်ဆိုင်ရာ ဆက်တင်။

**စုံစမ်းစစ်ဆေးနည်း:** Accessibility ဆော့ဖ်ဝဲသည် တရားဝင်သည်။ Enable လုပ်ထားသော feature အမည်ကို ၎င်း၏ အမှန်တကယ် executable registration မှ ခွဲခြားပြီးနောက် ပစ်မှတ်နှင့် sign-in/desktop လုပ်ဆောင်မှု အထောက်အထားကို စစ်ဆေးပါ။ ၎င်းသည် accessibility executable ကိုယ်တိုင်ကို အစားထိုးခြင်းနှင့် သီးခြားဖြစ်သည်။

**နောက်ထပ် အသေးစိတ်များ:** [Technique ကိုးကားချက် (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/)။

### BITS Job Notification Command {#bits_notify_command}

**Check ID:** `bits_notify_command` · **ကတ်တလောက် ATT&CK:** [T1197](https://attack.mitre.org/techniques/T1197/)

**Juntei စစ်ဆေးသည့်အရာ:** ဖမ်းယူထားသော `qmgr.db`၊ `qmgr0.dat` သို့မဟုတ် `qmgr1.dat` မှ counted-string BITS job မှတ်တမ်းများကို ပြန်လည်ရယူပြီး ဗလာမဟုတ်သော notification ပရိုဂရမ်များ၊ parameter များ၊ job ID များနှင့် owner SID များကို ထိန်းသိမ်းသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`၊ `qmgr0.dat` နှင့် `qmgr1.dat`။

**စုံစမ်းစစ်ဆေးနည်း:** ၎င်းတို့သည် carve လုပ်ထားသော မှတ်တမ်းများ ဖြစ်သည်- allocated/လက်ရှိ job အခြေအနေကို စစ်ဆေးအတည်ပြုထားခြင်း မရှိဘဲ ခေတ်မမီတော့သော/ဖျက်ထားသော job များ ပေါ်လာနိုင်သည်။ ဒေတာဘေ့စ်၏ MFT ရေးသားချိန်သည် အားနည်းသော container အထောက်အထားဖြစ်ပြီး notification ၏ လုပ်ဆောင်ချိန် မဟုတ်ပါ။ Owner SID များသည် SYSTEM၊ local service၊ network service နှင့် အသုံးပြုသူများကို ခွဲခြားပေးသည်။

### Command Processor AutoRun {#cmd_autorun}

**Check ID:** `cmd_autorun` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** command-prompt startup ကို သက်ရောက်စေနိုင်သော ဆက်တင်ဖြစ်သည့် စက်/အသုံးပြုသူ Command Processor `AutoRun` ညွှန်ကြားချက်များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**စုံစမ်းစစ်ဆေးနည်း:** Development tool များနှင့် shell စိတ်ကြိုက်ပြင်ဆင်မှုများက value ကို ရှင်းပြနိုင်သည်။ `cmd.exe /d` သည် AutoRun ကို ကျော်ဖြတ်သောကြောင့် ပြင်ဆင်မှုတစ်ခုတည်းက cmd.exe ပေါ်လာတိုင်း ၎င်း run ခဲ့ကြောင်း သက်သေမပြပါ။ ညွှန်ကြားချက် အကြောင်းအရာနှင့် trigger လုပ်သော process argument များကို စစ်ဆေးပါ။

**နောက်ထပ် အသေးစိတ်များ:** [Microsoft စာရွက်စာတမ်း](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd)။

### ErrorHandler.cmd Persistence {#error_handler_cmd}

**Check ID:** `error_handler_cmd` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** စုဆောင်းထားသော `Windows\System32\ErrorHandler.cmd` script ၏ တည်ရှိမှုကို error-handler persistence ကိုယ်စားလှယ်လောင်းအဖြစ် အစီရင်ခံသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `Windows\System32\ErrorHandler.cmd`။

**စုံစမ်းစစ်ဆေးနည်း:** တည်ရှိမှုတစ်ခုတည်းက သက်ဆိုင်ရာ Windows recovery/error လမ်းကြောင်းက ၎င်းကို ခေါ်ယူခဲ့ကြောင်း သက်သေမပြပါ။ Script ကို ဖတ်ပြီး ၎င်း၏ မူလအရင်းအမြစ်ကို စစ်ဆေးကာ setup/error/process အထောက်အထားနှင့် ချိတ်ဆက်စစ်ဆေးပါ။

### Power Automate Desktop Flows {#power_automate}

**Check ID:** `power_automate` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** အသုံးပြုသူတစ်ဦးစီ၏ Power Automate Desktop `flows` နှင့် `scripts` store များအောက်ရှိ ဖမ်းယူထားသော flow/script directory များကို ရှာဖွေပြီး တွေ့ရှိသော flow တစ်ခုစီကို တစ်ကြိမ်စီ အစီရင်ခံသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` နှင့် `scripts\<flow>\...`။

**စုံစမ်းစစ်ဆေးနည်း:** ၎င်းသည် သိမ်းဆည်းထားသော automation ကို ဖော်ထုတ်ခြင်းဖြစ်ပြီး အချိန်ဇယားသတ်မှတ်ထားသော trigger တိုင်း သို့မဟုတ် လုပ်ဆောင်ခဲ့သော flow ကို မဟုတ်ပါ။ ၎င်းကို အလိုအလျောက် သို့မဟုတ် အန္တရာယ်ရှိသော persistence အဖြစ် မသဘောထားမီ flow အကြောင်းအရာများနှင့် ပြင်ပ စတင်ဖွင့်မှု ပြင်ဆင်ချက်ကို စစ်ဆေးပါ။

### PowerShell Profile {#powershell_profiles}

**Check ID:** `powershell_profiles` · **ကတ်တလောက် ATT&CK:** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**Juntei စစ်ဆေးသည့်အရာ:** အသုံးပြုသူ Documents၊ တွေ့ရှိထားသော OneDrive Documents နှင့် စုဆောင်းထားသော Documents redirection များ အပါအဝင် အသိအမှတ်ပြုထားသော Windows PowerShell နှင့် PowerShell installation/profile directory များရှိ profile.ps1 ပုံစံ ဖိုင်များကို ရှာဖွေသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `Windows\System32\WindowsPowerShell\v1.0` နှင့် `Windows\SysWOW64\WindowsPowerShell\v1.0` တွင် တိုက်ရိုက်ရှိသော `profile.ps1` ဖြင့် အဆုံးသတ်သည့် ဖိုင်များ။
- ဖမ်းယူထားသော Documents redirection များနှင့် တွေ့ရှိထားသော OneDrive Documents ဖိုင်တွဲများ အပါအဝင် အသုံးပြုသူအလိုက် `Documents\WindowsPowerShell` နှင့် `Documents\PowerShell`။
- `Program Files\PowerShell` သို့မဟုတ် `Program Files (x86)\PowerShell` အောက်ရှိ ဗားရှင်း directory များတွင် တိုက်ရိုက်ရှိသော `profile.ps1` ဖြင့် အဆုံးသတ်သည့် ဖိုင်များ။

**စုံစမ်းစစ်ဆေးနည်း:** Profile များတွင် အန္တရာယ်ကင်းသော စိတ်ကြိုက်ပြင်ဆင်မှုများ ပါဝင်နိုင်သည်။ Script ကို ဖတ်ပြီး ခေါ်ယူထားသော payload များကို လိုက်ကြည့်ပါ။ PowerShell ကို `-NoProfile` ဖြင့် စတင်ဖွင့်ပါက ၎င်းတို့ကို ကျော်သွားသည်။ ဖိုင်တည်ရှိမှုတစ်ခုတည်းက profile လုပ်ဆောင်မှု သို့မဟုတ် အန္တရာယ်ရှိသော အကြောင်းအရာကို သက်သေမပြပါ။

### Screensaver Hijack {#screensaver}

**Check ID:** `screensaver` · **ကတ်တလောက် ATT&CK:** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**Juntei စစ်ဆေးသည့်အရာ:** အသုံးပြုသူတစ်ဦးစီ၏ Desktop `SCRNSAVE.EXE` value ကို ဖတ်ပြီး ပြင်ဆင်ထားသော screensaver image ကို ဖော်ထုတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- NTUSER.DAT ရှိ `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE`။

**စုံစမ်းစစ်ဆေးနည်း:** ခွင့်ပြုထားသော screensaver များနှင့် နှိုင်းယှဉ်ပြီး ပစ်မှတ်ကို စစ်ဆေးပါ။ အခြား desktop ဆက်တင်များ၊ session အခြေအနေနှင့် timeout တို့က ၎င်း trigger ဖြစ်နိုင်/မဖြစ်နိုင်ကို ဆုံးဖြတ်သည်။ သိမ်းဆည်းထားသော path သည် screensaver စတင်ခဲ့ကြောင်း သက်သေမပြပါ။

### Telemetry Controller Command {#telemetry_controller}

**Check ID:** `telemetry_controller` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** TelemetryController နှင့် ၎င်း၏ တိုက်ရိုက် child registration များမှ `Binary`၊ `Command` နှင့် `MaintenanceCommand` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` နှင့် တိုက်ရိုက် child များ- `Binary`, `Command`, `MaintenanceCommand`။

**စုံစမ်းစစ်ဆေးနည်း:** ရွေးချယ်ထားသော executable နှင့် argument များကို တူညီသော Windows build နှင့် maintenance ဆော့ဖ်ဝဲတို့နှင့် နှိုင်းယှဉ်ပါ။ Registry ညွှန်ကြားချက်သည် maintenance trigger ဖြစ်ပေါ်ခဲ့ကြောင်း မပြသပါ။

### Windows Terminal Custom Profiles {#windows_terminal}

**Check ID:** `windows_terminal` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** Packaged၊ preview နှင့် unpackaged Windows Terminal JSON/JSONC ဆက်တင်များကို ခွဲခြမ်းစိတ်ဖြာပြီး ဗလာမဟုတ်သော profile/default `commandline` ဆက်တင်များကို အစီရင်ခံသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` နှင့် WindowsTerminalPreview တွင် သက်ဆိုင်ရာ ဖိုင်။
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: legacy profiles array များ၊ `profiles.list` နှင့် `profiles.defaults` commandline field များ။

**စုံစမ်းစစ်ဆေးနည်း:** ပုံမှန် shell များ ဤနေရာတွင် သဘာဝအတိုင်း ပေါ်လာသည်။ ရွေးချယ်ထားသော/ပုံသေ profile ကို ဆုံးဖြတ်ပြီး ပုံမှန်မဟုတ်သော ညွှန်ကြားချက်များ သို့မဟုတ် စတင်ဖွင့်မှု argument များကို စစ်ဆေးပါ။ Profile ကို သိမ်းဆည်းရုံဖြင့် ၎င်းကို စတင်ဖွင့်ခြင်း မရှိပါ။

### WMI Event Subscription {#wmi_event_subscription}

**Check ID:** `wmi_event_subscription` · **ကတ်တလောက် ATT&CK:** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**Juntei စစ်ဆေးသည့်အရာ:** ဖမ်းယူထားသော WMI OBJECTS.DATA repository မှ အသိအမှတ်ပြုထားသော event-consumer instance များနှင့် filter-to-consumer binding reference များကို carve လုပ်သည်။ အသိအမှတ်ပြုထားသော class များတွင် command-line၊ active-script၊ log-file၊ event-log နှင့် SMTP consumer များ ပါဝင်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` နှင့် `Repository\FS\OBJECTS.DATA`။

**စုံစမ်းစစ်ဆေးနည်း:** Raw repository hit သည် ခေတ်မမီတော့ခြင်း၊ bind မလုပ်ထားခြင်း သို့မဟုတ် သမိုင်းဝင်ဖြစ်ခြင်း ဖြစ်နိုင်သည်။ ထုတ်ယူမှုက ထိုသတ်မှတ်ချက်ကို ထိန်းသိမ်းထားသည်။ Carving သည် ပြည့်စုံသော active subscription graph ကို ပြန်လည်တည်ဆောက်ခြင်း သို့မဟုတ် trigger ဖြစ်ပေါ်ခဲ့ကြောင်း သက်သေပြခြင်း မပြုပါ။ Repository ၏ ဖိုင်ရေးသားချိန်သည် subscription ဖန်တီးချိန်/လုပ်ဆောင်ချိန် မဟုတ်ပါ။

## Explorer၊ COM နှင့် ဖိုင် သို့မဟုတ် URL handler များ { #explorer-com-and-file-or-url-handlers }

Component Object Model (COM) သည် အပလီကေးရှင်းများကို register လုပ်ထားသော ဆော့ဖ်ဝဲ component များကို activate လုပ်ခွင့်ပေးသည်။ Windows shell သည် ဖိုင်များ၊ URL များ၊ ဖိုင်တွဲများနှင့် အထူး object များကို ဖွင့်ရန် register လုပ်ထားသော handler များကို အသုံးပြုသည်။ COM identifier (CLSID) များသည် component များကို ဖော်ထုတ်သည်။ ProgID များသည် component အချို့ကို ဖတ်ရလွယ်သော အမည်များ ပေးသည်။ Registration တစ်ခုစီကို ၎င်းရွေးချယ်သော ညွှန်ကြားချက် သို့မဟုတ် DLL အထိ လိုက်ကြည့်ပါ။

### App Paths Hijack {#app_paths}

**Check ID:** `app_paths` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** Shell အခြေပြု စတင်ဖွင့်မှုများအတွက် အပလီကေးရှင်း ရှာဖွေမှုကို redirect လုပ်နိုင်သော စက်တစ်ခုလုံးဆိုင်ရာ `App Paths` registration များအောက်ရှိ default value များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>` default value။

**စုံစမ်းစစ်ဆေးနည်း:** App Paths registration သည် ထည့်သွင်းထားသော အပလီကေးရှင်းများအတွက် အသုံးများပြီး process စတင်ဖွင့်မှုတိုင်းကို ယေဘုယျ ပြန်ရေးခြင်း မဟုတ်ပါ။ Register လုပ်ထားသော အမည်နှင့် image ကို မျှော်လင့်ထားသော ဆော့ဖ်ဝဲနှင့် နှိုင်းယှဉ်ပြီး သက်ဆိုင်ရာ shell/process အထောက်အထားကို စစ်ဆေးပါ။

### COM Object Hijack {#com_treat_as}

**Check ID:** `com_treat_as` · **ကတ်တလောက် ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei စစ်ဆေးသည့်အရာ:** ပံ့ပိုးထားသော native/32-bit class store များတစ်လျှောက် COM `TreatAs` redirection များနှင့် စက် CLSID ကို ဖုံးအုပ်ထားသော အသုံးပြုသူအလိုက် InprocServer32/LocalServer32 registration များကို ရှာဖွေသည်။ TreatAs ပစ်မှတ်များကို တူညီသော အသုံးပြုသူ၏ class များနှင့် စက် class များကို အသုံးပြု၍ ဖော်ထုတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` နှင့် စက်၏ `Classes\Wow6432Node` view။
- NTUSER.DAT ရှိ အသုံးပြုသူအလိုက် `Software\Classes\CLSID` နှင့် UsrClass.dat ရှိ `CLSID` (အကောင်အထည်ဖော်ထားသော `Wow6432Node` view များ အပါအဝင်)- `TreatAs`၊ `InprocServer32` နှင့် `LocalServer32` default value များ။

**စုံစမ်းစစ်ဆေးနည်း:** အသုံးပြုသူအလိုက် override များသည် administrator အခွင့်အာဏာ မလိုအပ်နိုင်သော်လည်း တရားဝင်ဆော့ဖ်ဝဲများကလည်း ၎င်းတို့ကို register လုပ်နိုင်သည်။ ရွေးချယ်ထားသော class နှင့် DLL/executable ကို လိုက်ကြည့်ပြီး architecture နှင့် activation အခြေအနေကို အကဲဖြတ်ကာ အမှန်တကယ် COM activation ကို သီးခြားလွတ်လပ်စွာ အတည်ပြုပါ။

### Disk Cleanup Handler Hijack {#disk_cleanup_handler}

**Check ID:** `disk_cleanup_handler` · **ကတ်တလောက် ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei စစ်ဆေးသည့်အရာ:** Disk Cleanup VolumeCaches handler CLSID များကို ဖတ်ပြီး ဖော်ထုတ်နိုင်သော စက် COM InprocServer32 registration များကို ၎င်းတို့၏ library များအထိ လိုက်ကြည့်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` default CLSID နှင့် `Classes\CLSID\<CLSID>\InprocServer32`။

**စုံစမ်းစစ်ဆေးနည်း:** Handler run ရန် သက်ဆိုင်ရာ cleanup လုပ်ငန်းစဉ်က ၎င်းကို ခေါ်ယူရမည်။ ဖော်ထုတ်မရသော handler များကို ဤစစ်ဆေးမှုက ထုတ်ပေးခြင်း မရှိသောကြောင့် ဖမ်းယူမှု မပြည့်စုံပါက ၎င်းတို့၏ raw registration များကို စစ်ဆေးရန် registry parser ကို အသုံးပြုပါ။

### Explorer AppKey Override {#explorer_app_key}

**Check ID:** `explorer_app_key` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** တိုက်ရိုက် Explorer `AppKey` child များအောက်ရှိ `ShellExecute` နှင့် `Association` value များကို ဖတ်ပြီး application key များက ရွေးချယ်သော ညွှန်ကြားချက်/action များကို လွှမ်းခြုံသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` နှင့် `Association`။

**စုံစမ်းစစ်ဆေးနည်း:** Keyboard utility များနှင့် OEM tool များက ဤ value များကို သတ်မှတ်နိုင်သည်။ Association အမည်သည် ဖိုင် path မဟုတ်ဘဲ အခြား registry pivot တစ်ခု လိုအပ်နိုင်သည်။ ၎င်းကို trigger လုပ်မည့် အသုံးပြုသူ လုပ်ဆောင်ချက်ကို စစ်ဆေးပါ။

### Browser Helper Objects {#explorer_bho}

**Check ID:** `explorer_bho` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Browser Helper Object CLSID registration များကို ဖတ်ပြီး ရရှိနိုင်သည့်အခါ စက် InprocServer32 library များကို ဖော်ထုတ်သည်။ ဖော်ထုတ်မရသော class ID များကို ဆက်လက်မြင်နိုင်အောင် ထားသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` နှင့် စက်၏ `Classes\CLSID\<CLSID>\InprocServer32` default value။

**စုံစမ်းစစ်ဆေးနည်း:** ၎င်းသည် လိုက်ဖက်သော browser/Explorer အပြုအမူနှင့် ချိတ်ဆက်နေပြီး legacy ပတ်ဝန်းကျင်များနှင့် အထူးသက်ဆိုင်သည်။ ခွင့်ပြုထားသော browser/လုံခြုံရေး extension များ ပေါ်လာနိုင်သည်။ Register လုပ်ထားသော BHO သည် လက်ရှိ browser က ၎င်းကို load လုပ်ခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။

### Explorer Desktop CLSID Hijack {#explorer_clsid_hijack}

**Check ID:** `explorer_clsid_hijack` · **ကတ်တလောက် ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei စစ်ဆေးသည့်အရာ:** ဖော်ပြထားသော အထူး Explorer namespace CLSID သုံးခုပေါ်ရှိ ပြင်ဆင်ထားသော shell-open ညွှန်ကြားချက်များကို စက်/အသုံးပြုသူ class store များမှ ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**စုံစမ်းစစ်ဆေးနည်း:** Desktop၊ My Computer နှင့် My Documents အတွက် command handler များကို ကိုက်ညီသော baseline နှင့် နှိုင်းယှဉ်ပါ။ ၎င်းသည် ရွေးချယ်ထားသော CLSID များနှင့် path များကို စစ်ဆေးခြင်းဖြစ်ပြီး ဖြစ်နိုင်သော Explorer namespace hijack တိုင်းကို မဟုတ်ပါ။

### Explorer Context Menu Handlers {#explorer_context_menu}

**Check ID:** `explorer_context_menu` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Directory၊ Directory Background၊ Folder၊ Drive၊ AllFilesystemObjects နှင့် `*` အတွက် context-menu handler များကို စာရင်းကောက်ပြီး ဖြစ်နိုင်သည့်အခါ class ID များကို စုဆောင်းထားသော DLL registration များအထိ လိုက်ကြည့်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- စက် `HKLM\SOFTWARE\Classes` နှင့် အသုံးပြုသူ UsrClass.dat class store များ- `<subject>\shellex\ContextMenuHandlers\<handler>` default value နှင့် သက်ဆိုင်ရာ COM library registration များ။

**စုံစမ်းစစ်ဆေးနည်း:** Archive tool များ၊ လုံခြုံရေး ထုတ်ကုန်များနှင့် cloud client များမှ right-click extension များသည် အသုံးများသည်။ အမှန်တကယ် handler နှင့် trigger လုပ်သော ထိတွေ့ဆောင်ရွက်မှုကို စစ်ဆေးပါ။ ဖော်ထုတ်မရသော GUID များသည် DLL မရှိခဲ့ကြောင်း သက်သေထက် အသုံးဝင်သော သဲလွန်စများ ဖြစ်နေဆဲ ဖြစ်သည်။

### File Association Hijacking {#file_association_hijack}

**Check ID:** `file_association_hijack` · **ကတ်တလောက် ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei စစ်ဆေးသည့်အရာ:** `.txt`, `.pdf`, `.doc`, `.docx`, `.html`, `.htm`, `.js`, `.vbs`, `.hta`, `.exe`, `.bat`, `.cmd` နှင့် `.ps1` အတွက် တိုက်ရိုက် extension နှင့် ဖော်ထုတ်ထားသော ProgID `shell\open\command` registration များကို စစ်ဆေးသည်။

**စစ်ဆေးသော အထောက်အထား:**

- စက် `HKLM\SOFTWARE\Classes` နှင့် အသုံးပြုသူ UsrClass.dat class store များ- ရွေးချယ်ထားသော `<extension>` default များနှင့် `<extension-or-ProgID>\shell\open\command` default များ။

**စုံစမ်းစစ်ဆေးနည်း:** မျှော်လင့်ထားသော အပလီကေးရှင်းနှင့် နှိုင်းယှဉ်ပြီး ညွှန်ကြားချက် argument များကို စစ်ဆေးပါ။ ၎င်းသည် သတ်မှတ်ထားသော extension အစုဖြစ်ပြီး ဖိုင်အမျိုးအစားအားလုံး သို့မဟုတ် အပြည့်အဝ သက်ရောက်မှုရှိသော UserChoice ဖော်ထုတ်မှု မဟုတ်ပါ။ သိမ်းဆည်းထားသော ညွှန်ကြားချက်သည် စာရွက်စာတမ်းတစ်ခု ဖွင့်ခဲ့ကြောင်း သက်သေမပြပါ။

### Protocol Handler Hijacking {#protocol_handler_hijack}

**Check ID:** `protocol_handler_hijack` · **ကတ်တလောက် ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei စစ်ဆေးသည့်အရာ:** http၊ https၊ mailto၊ ms-msdt နှင့် ms-officecmd အတွက် URL open ညွှန်ကြားချက်များအပြင် တွေ့ရှိထားသော `URL Protocol` registration များနှင့် သက်ဆိုင်ရာ အသုံးပြုသူ override များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- စက် `HKLM\SOFTWARE\Classes` နှင့် အသုံးပြုသူ UsrClass.dat class store များ- `URL Protocol` marker များနှင့် `<scheme>\shell\open\command` default value များ။

**စုံစမ်းစစ်ဆေးနည်း:** Browser များနှင့် အပလီကေးရှင်း installer များသည် scheme များကို တရားဝင် register လုပ်ကြသည်။ ညွှန်ကြားချက်နှင့် argument များကို လိုက်ကြည့်ပြီး scheme ၏ မျှော်လင့်ထားသော ရည်ရွယ်ချက်ကို အကဲဖြတ်ကာ URL ဖွင့်ခြင်း/process အထောက်အထားနှင့် ချိတ်ဆက်စစ်ဆေးပါ။

### Recycle Bin COM Extension Handler {#recycle_bin_com_extension}

**Check ID:** `recycle_bin_com_extension` · **ကတ်တလောက် ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei စစ်ဆေးသည့်အရာ:** ဖော်ပြထားသော Recycle Bin CLSID ၏ open/empty/explore ညွှန်ကြားချက် override များနှင့် တိုက်ရိုက် context-menu/drag-drop handler registration များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**စုံစမ်းစစ်ဆေးနည်း:** ခွင့်ပြုထားသော shell extension များနှင့် နှိုင်းယှဉ်ပါ။ အစီရင်ခံထားသော value အချို့သည် တိုက်ရိုက်ဖော်ထုတ်ထားသော library မဟုတ်ဘဲ COM identifier များ ဖြစ်သည်။ Code လုပ်ဆောင်မှုကို အခိုင်အမာ မဆိုမီ ရည်ညွှန်းထားသော class နှင့် အသုံးပြုသူ လုပ်ဆောင်ချက်ကို စုံစမ်းစစ်ဆေးပါ။

### Search Protocol Handler Hijack {#search_protocol_handler}

**Check ID:** `search_protocol_handler` · **ကတ်တလောက် ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei စစ်ဆေးသည့်အရာ:** စက်/အသုံးပြုသူ `search-ms\shell\open\command` default handler ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**စုံစမ်းစစ်ဆေးနည်း:** Handler ပြောင်းလဲမှုသည် search-ms URL ဖွင့်သည့်အခါ ဖြစ်ပေါ်မည့်အရာကို ပြောင်းလဲနိုင်သော်လည်း ၎င်း၏ တည်ရှိမှုသည် exploit ပေးပို့ခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။ Windows ပုံသေများနှင့် နှိုင်းယှဉ်ပြီး ၎င်းကို အသုံးပြုမည့် URL/process လှုပ်ရှားမှုကို စုံစမ်းစစ်ဆေးပါ။

### SharedTaskScheduler {#shared_task_scheduler}

**Check ID:** `shared_task_scheduler` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** ဗလာမဟုတ်သော Explorer `SharedTaskScheduler` value များကို register လုပ်ထားသော shell component များအဖြစ် စာရင်းကောက်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** Value များသည် executable path အစား COM class များကို ရည်ညွှန်းနိုင်သည်။ ၎င်းတို့၏ server registration များကို လိုက်ကြည့်ပြီး operating-system အပြုအမူ၊ extension ၏ မူလအရင်းအမြစ်နှင့် သက်ဆိုင်ရာ shell loading အထောက်အထားကို စစ်ဆေးပါ။

### ShellExecuteHooks {#shell_execute_hooks}

**Check ID:** `shell_execute_hooks` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** ဗလာမဟုတ်သော Explorer `ShellExecuteHooks` value များကို စာရင်းကောက်ပြီး legacy shell-execution hook registration များကို လွှမ်းခြုံသည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** ရည်ညွှန်းထားသော CLSID တစ်ခုခုကို ဖော်ထုတ်ပြီး Windows build နှင့် policy က hook ကို ခွင့်ပြုခြင်း ရှိ/မရှိ စစ်ဆေးအတည်ပြုပါ။ Legacy registration သည် အသုံးမပြုဘဲ ရှိနေနိုင်သည်။ Value ကိုယ်တိုင်သည် လေ့လာတွေ့ရှိထားသော shell လုပ်ဆောင်မှု မဟုတ်ပါ။

### Start Menu Library and CLSID Folder COM Persistence {#start_menu_com}

**Check ID:** `start_menu_com` · **ကတ်တလောက် ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei စစ်ဆေးသည့်အရာ:** `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu` အောက်ရှိ
`simpleLocation` URL က COM CLSID ကို ရည်ညွှန်းသော `.library-ms` library XML ဖိုင်များနှင့် `<name>.{CLSID}`
အမည်ရှိ directory များကို ရှာဖွေသည်။ ဤ object များနှင့် Explorer ၏ ထိတွေ့ဆောင်ရွက်မှုသည် COM code ကို activate လုပ်နိုင်သည်။
စစ်ဆေးမှုတွင် ဖမ်းယူထားသော ဗလာ directory များ ပါဝင်သည်။ ၎င်းသည် symbolic link များကို လိုက်မကြည့်ပါ။ Start Menu
ပြင်ပရှိ သာမန် library ဖိုင်များနှင့် XML comment များရှိ GUID များကို ဤစစ်ဆေးမှုက အစီရင်ခံခြင်း မရှိပါ။

**စစ်ဆေးသော အထောက်အထား:**

- အသုံးပြုသူအလိုက် Start Menu `.library-ms` ဖိုင်များနှင့် CLSID အမည်ရှိ directory entry များ။
- ကိုက်ညီသော အသုံးပြုသူ၏ ဖမ်းယူထားသော class registration များနှင့် စက် SOFTWARE ကို architecture အလိုက် သီးခြားစီ။
- Scheduled task များနှင့် တူညီသော resolver မှတစ်ဆင့် managed CodeBase/AppID/service ပြင်ဆင်မှု။

**စုံစမ်းစစ်ဆေးနည်း:** `Path` နှင့် `SourceFile` တို့သည် activation-trigger အထောက်အထားကို ဖော်ထုတ်သည်။ `Value`
တွင် ကိုယ်စားလှယ်လောင်း server ညွှန်ကြားချက် သို့မဟုတ် ဖော်ထုတ်မှု မရရှိနိုင်ပါက CLSID ပါဝင်သည်။ `Description`
သည် trigger ကို CLSID နှင့် ဖမ်းယူထားသော registry source တို့နှင့် ချိတ်ဆက်ပေးသည်။ `ResolvedPath`/`Sha256` တို့သည်
ဖော်ထုတ်ထားပြီး စုဆောင်းထားသော payload ကိုသာ ဖော်ပြသည်။ Executable အမည်သာပါသော AppID ဆက်နွယ်မှုကို ဖော်ထုတ်မထားဘဲ ထားသည်။
Trigger သည် တရားဝင် ဖြစ်နိုင်သည်။ တည်ရှိမှုသည် sign-in၊ activation၊ လုပ်ဆောင်မှု သို့မဟုတ် တိုက်ခိုက်မှုကို သက်သေမပြပါ။
Library URL၊ class registration၊ payload၊ အသုံးပြုသူ အခြေအနေနှင့် သီးခြားလွတ်လပ်သော လုပ်ဆောင်မှု log များကို စစ်ဆေးပါ။

**စုဆောင်းမှုဆိုင်ရာ သတိပြုရန်:** ဖိုင်သာ စုဆောင်းမှုများသည် ဗလာ directory များကို ချန်လှပ်နိုင်သည်။ စံ အသုံးပြုသူ Start Menu
path များကို စစ်ဆေးသည်။ Redirect လုပ်ထားသော Start Menu များနှင့် အခြား Explorer namespace တည်နေရာများကို ဤစစ်ဆေးမှုက
မလွှမ်းခြုံပါ။ ပျောက်နေသော hive များ သို့မဟုတ် ဖော်ထုတ်မရသော registration များသည် ကိုယ်စားလှယ်လောင်းကို ထိန်းသိမ်းထားစဉ်
တစ်စိတ်တစ်ပိုင်း coverage ကို ထုတ်ပေးသည်။ Task architecture၊ elevation နှင့် policy တို့က သက်ရောက်မှုရှိသော COM ရှာဖွေမှု အပြုအမူကို ပြောင်းလဲနိုင်သည်။

**နောက်ထပ် အသေးစိတ်များ:** [မူရင်း အဆိုပြုချက်နှင့် သုတေသီ ချီးမွမ်းချက်](https://github.com/last-byte/PersistenceSniper/pull/32)၊
[COM အသုံးပြုသူအလိုက်/စက် ပေါင်းစပ် view များ](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root)၊
[MITRE COM hijacking](https://attack.mitre.org/techniques/T1546/015/)။

### TypeLib COM Hijacking {#typelib_hijack}

**Check ID:** `typelib_hijack` · **ကတ်တလောက် ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei စစ်ဆေးသည့်အရာ:** အသုံးပြုသူအလိုက် UsrClass.dat TypeLib GUID/version/locale win32/win64 registration များကို ဖတ်ပြီး path များနှင့် script-moniker value များကို ထိန်းသိမ်းသည်။ နောက်ဆက်တွဲ ဂဏန်း resource index များကို ပစ်မှတ်ဖိုင် ဖော်ထုတ်မှုအတွက်သာ ဖယ်ရှားသည်။

**စစ်ဆေးသော အထောက်အထား:**

- ဖမ်းယူထားသော UsrClass.dat တစ်ခုစီရှိ `TypeLib\<GUID>\<version>\<locale>\win32` နှင့် `win64` default value များ။

**စုံစမ်းစစ်ဆေးနည်း:** Type library သည် တရားဝင် DLL၊ အခြားဖိုင် သို့မဟုတ် script moniker ကို ညွှန်ပြနိုင်သည်။ စစ်ဆေးမှုက ဤ registration များကို စာရင်းကောက်သည်။ ပုံသေမဟုတ်သော path တိုင်းကို တိုက်ခိုက်မှုဟု မယူဆဘဲ သက်ရောက်ခံရသော အပလီကေးရှင်းကို အကဲဖြတ်ပြီး type-library loading ကို အတည်ပြုပါ။

## Office အပလီကေးရှင်းများနှင့် add-in များ { #office-applications-and-add-ins }

Office သည် တရားဝင် add-in များ၊ template များနှင့် အပလီကေးရှင်းအလိုက် extension များကို ပံ့ပိုးသည်။ သံသယဖြစ်ဖွယ် registration တစ်ခုသည် သီးခြား Office ဗားရှင်း၊ အပလီကေးရှင်း၊ loading ဆက်တင် သို့မဟုတ် အသုံးပြုသူ ထိတွေ့ဆောင်ရွက်မှုပေါ် မူတည်နိုင်သည်။ ပြင်ဆင်မှုတစ်ခုတည်းက macro လုပ်ဆောင်မှုကို မသတ်မှတ်နိုင်ပါ။

### Office Add-in Registration {#office_addins}

**Check ID:** `office_addins` · **ကတ်တလောက် ATT&CK:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei စစ်ဆေးသည့်အရာ:** အကောင်အထည်ဖော်ထားသော စက် Click-to-Run store နှင့် အသုံးပြုသူ root များ အပါအဝင် ဗားရှင်းမပါသော နှင့် 14.0/15.0/16.0 Office root များရှိ Word၊ Excel၊ PowerPoint၊ Outlook နှင့် Access add-in များကို စာရင်းကောက်သည်။ ၎င်းသည် ရှင်းလင်းစွာ ပါရှိသော Manifest/FileName/Path value များကို ထိန်းသိမ်းသည် သို့မဟုတ် သတ်မှတ်ချက်ပြည့်မီသော COM အခြေပြု entry များကို LoadBehavior အခြေအနေနှင့်အတူ ဖော်ထုတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` နှင့် အကောင်အထည်ဖော်ထားသော စက် Click-to-Run တွင် သက်ဆိုင်ရာ key။
- NTUSER.DAT ရှိ `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>`; SOFTWARE၊ NTUSER.DAT နှင့် UsrClass.dat မှ ကိုက်ညီသော စက်/အသုံးပြုသူ COM class များ။

**စုံစမ်းစစ်ဆေးနည်း:** Add-in များသည် အသုံးများသော လုပ်ငန်းသုံး ဆော့ဖ်ဝဲများ ဖြစ်သည်။ LoadBehavior၊ အပလီကေးရှင်း/ဗားရှင်း၊ manifest များ၊ အမှန်တကယ် payload များနှင့် မျှော်လင့်ထားသော vendor ကို ပြန်လည်သုံးသပ်ပါ။ စစ်ဆေးမှုသည် ထည့်သွင်းထားသော add-in ပုံစံ သို့မဟုတ် registration view တိုင်းကို စုဆောင်းထားခြင်း သို့မဟုတ် load လုပ်ထားခြင်းကို အာမမခံပါ။

### Office AI Add-in Hijack {#office_ai_hijack}

**Check ID:** `office_ai_hijack` · **ကတ်တလောက် ATT&CK:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei စစ်ဆေးသည့်အရာ:** Office 16.0 Click-to-Run Common AI registration တည်နေရာရှိ value များကို စာရင်းကောက်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: value အားလုံး။

**စုံစမ်းစစ်ဆေးနည်း:** ပုံမှန်မဟုတ်သော value များနှင့် path များကို ထည့်သွင်းထားသော Office build နှင့် နှိုင်းယှဉ်ပါ။ Value တည်ရှိခြင်းက မည်သည့် Office feature က ၎င်းကို အသုံးပြုခဲ့သည်ကို မပြသဘဲ executable code ကို ရွေးချယ်ခဲ့ကြောင်းလည်း သက်သေမပြပါ။

### Office DLL Override {#office_dll_override}

**Check ID:** `office_dll_override` · **ကတ်တလောက် ATT&CK:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei စစ်ဆေးသည့်အရာ:** အကောင်အထည်ဖော်ထားသော Click-to-Run နှင့် အသုံးပြုသူ root များ အပါအဝင် ဗားရှင်းမပါသော နှင့် 14.0/15.0/16.0 ဆက်တင်များအတွက် Word/PowerPoint root များအောက်ရှိ `WwlibtDll` နှင့် `PPCoreTDLL` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- စက်/အသုံးပြုသူ `Microsoft\Office\[<version>\]Word` နှင့် `PowerPoint` ဆက်တင်များအပြင် အကောင်အထည်ဖော်ထားသော စက် Click-to-Run Office tree- `WwlibtDll` နှင့် `PPCoreTDLL`။

**စုံစမ်းစစ်ဆေးနည်း:** မည်သည့် အပလီကေးရှင်း/build က override ကို အသိအမှတ်ပြုသည်ကို ဆုံးဖြတ်ပြီး library နှင့် မူလအရင်းအမြစ်ကို စစ်ဆေးကာ Office စတင်ဖွင့်မှု သို့မဟုတ် image load နှင့် ချိတ်ဆက်စစ်ဆေးပါ။ ပြင်ဆင်ထားသော override တစ်ခုတည်းသည် လုပ်ဆောင်မှု၏ သက်သေ မဟုတ်ပါ။

### Office Default Templates {#office_templates}

**Check ID:** `office_templates` · **ကတ်တလောက် ATT&CK:** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**Juntei စစ်ဆေးသည့်အရာ:** စုဆောင်းထားသော Normal.dotm/NormalEmail.dotm ဖိုင်များနှင့် Office installation startup ဖိုင်တွဲများ အပါအဝင် အသိအမှတ်ပြုထားသော Word Startup နှင့် Excel XLSTART ဖိုင်တွဲများတွင် တိုက်ရိုက်ရှိသော ဖိုင်များကို စာရင်းကောက်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` နှင့် `NormalEmail.dotm`။
- Roaming AppData အောက်ရှိ အသုံးပြုသူ `Microsoft\Word\STARTUP` နှင့် `Microsoft\Excel\XLSTART` တွင် တိုက်ရိုက်ရှိသော ဖိုင်များ။
- Program Files နှင့် Program Files (x86) အောက်ရှိ အသိအမှတ်ပြုထားသော Office `<version>\STARTUP` / `XLSTART` နှင့် `root\<version>\STARTUP` / `XLSTART` ဖိုင်တွဲများတွင် တိုက်ရိုက်ရှိသော ဖိုင်များ။

**စုံစမ်းစစ်ဆေးနည်း:** Template တစ်ခုသည် တရားဝင်ဖြစ်နိုင်သလို executable အကြောင်းအရာ မပါဝင်ခြင်းလည်း ဖြစ်နိုင်သည်။ Juntei သည် ဤစစ်ဆေးမှုတွင် macro များကို ထုတ်ယူခြင်း သို့မဟုတ် လုပ်ဆောင်ခြင်း မပြုပါ။ စာရွက်စာတမ်း၏ အတွင်းပိုင်း၊ macro policy နှင့် အပလီကေးရှင်း လှုပ်ရှားမှုကို စစ်ဆေးပါ။ စက် installation path သည် SYSTEM process token ကို မဆိုလိုပါ။

### Office Test DLL {#office_test_dll}

**Check ID:** `office_test_dll` · **ကတ်တလောက် ATT&CK:** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**Juntei စစ်ဆေးသည့်အရာ:** ဖော်ပြထားသော စက်/အသုံးပြုသူ နှင့် 32-bit စက် view များမှ Office Test Special Perf key ရှိ value များကို စာရင်းကောက်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | ဗလာမဟုတ်သော value အားလုံး |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | ဗလာမဟုတ်သော value အားလုံး |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** မမျှော်လင့်ထားသော override များကို Office ဗားရှင်းနှင့် payload စစ်ဆေးအတည်ပြုမှု လိုအပ်သော သဲလွန်စအဖြစ် သဘောထားပါ။ စစ်ဆေးမှုသည် သီးခြား Office installation က value ကို load လုပ်ခြင်း ရှိ/မရှိ စမ်းသပ်ခြင်းထက် ပြင်ဆင်မှုကို အစီရင်ခံသည်။

### Outlook Home Page Attack {#outlook_home_page}

**Check ID:** `outlook_home_page` · **ကတ်တလောက် ATT&CK:** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**Juntei စစ်ဆေးသည့်အရာ:** Office ဗားရှင်း 14.0၊ 15.0 နှင့် 16.0 အတွက် အသုံးပြုသူ Outlook WebView child များအောက်ရှိ `URL` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- NTUSER.DAT ရှိ `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL`။

**စုံစမ်းစစ်ဆေးနည်း:** ဖိုင်တွဲ home page သည် တရားဝင်ဖြစ်နိုင်ပြီး အပြုအမူသည် Outlook ဗားရှင်းနှင့် လုံခြုံရေး အပ်ဒိတ်များအလိုက် ကွဲပြားသည်။ URL/အကြောင်းအရာနှင့် သက်ရောက်မှုရှိသော policy ကို စစ်ဆေးပြီးနောက် Outlook/web/process အထောက်အထားနှင့် ချိတ်ဆက်စစ်ဆေးပါ။

### VBA Monitor DLL Hijack {#vba_monitors}

**Check ID:** `vba_monitors` · **ကတ်တလောက် ATT&CK:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei စစ်ဆေးသည့်အရာ:** စက်နှင့် အသုံးပြုသူအလိုက် class store များရှိ အကောင်အထည်ဖော်ထားသော VBA-monitor ဆိုင်ရာ CLSID နှစ်ခုအတွက် InprocServer32 value များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- စက် SOFTWARE ရှိ `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` နှင့် `{0002E157-0000-0000-C000-000000000046}` အပြင် သက်ဆိုင်ရာ အသုံးပြုသူ UsrClass.dat registration များ။

**စုံစမ်းစစ်ဆေးနည်း:** ရွေးချယ်ထားသော library များကို ထည့်သွင်းထားသော Office/VBA ပတ်ဝန်းကျင်နှင့် နှိုင်းယှဉ်ပါ။ ၎င်းသည် ပစ်မှတ်ထားသော CLSID စစ်ဆေးမှုဖြစ်ပြီး VBA extension တိုင်း၏ စာရင်း သို့မဟုတ် macro run ခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။

## Debugger များ၊ crash handler များနှင့် compatibility shim များ { #debuggers-crash-handlers-and-compatibility-shims }

Debugger သို့မဟုတ် error handler တစ်ခုသည် အပလီကေးရှင်း စတင်သည့်အခါ၊ crash ဖြစ်သည့်အခါ၊ hang ဖြစ်သည့်အခါ သို့မဟုတ် ထွက်သည့်အခါ run နိုင်သည်။ Application compatibility ဒေတာဘေ့စ်များကလည်း startup ကို သက်ရောက်စေနိုင်သည်။ အစီရင်ခံထားသော ညွှန်ကြားချက်နှင့်အတူ ကြိုတင်လိုအပ်သော ဆက်တင်များနှင့် trigger လုပ်သော ပရိုဂရမ်ကို စုံစမ်းစစ်ဆေးပါ။

### AeDebug Debugger Hijack {#ae_debug}

**Check ID:** `ae_debug` · **ကတ်တလောက် ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei စစ်ဆေးသည့်အရာ:** Just-in-time crash debugging ပြင်ဆင်မှုက အသုံးပြုသော native နှင့် 32-bit AeDebug `Debugger` ညွှန်ကြားချက်များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**စုံစမ်းစစ်ဆေးနည်း:** Development/debugging ဆော့ဖ်ဝဲက ၎င်းကို တရားဝင် သတ်မှတ်နိုင်သည်။ Debugger ညွှန်ကြားချက် အပြည့်အစုံနှင့် crash-trigger ကြိုတင်လိုအပ်ချက်များကို စစ်ဆေးပြီးနောက် application-error နှင့် process event များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။

### AeDebug Protected Process Debugger {#ae_debug_protected}

**Check ID:** `ae_debug_protected` · **ကတ်တလောက် ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei စစ်ဆေးသည့်အရာ:** Protected-process debugging မျက်နှာပြင်အတွက် native နှင့် 32-bit AeDebugProtected `Debugger` ညွှန်ကြားချက်များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**စုံစမ်းစစ်ဆေးနည်း:** Operating-system build နှင့် ကာကွယ်မှု policy က ပြင်ဆင်ထားသော ညွှန်ကြားချက်ကို အသုံးပြုနိုင်ခြင်း ရှိ/မရှိ ဆုံးဖြတ်ပါ။ သိမ်းဆည်းထားသော debugger သည် protected process crash ဖြစ်ခဲ့ကြောင်း သို့မဟုတ် ညွှန်ကြားချက် run ခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။

### Image File Execution Options Debugger {#ifeo_debugger}

**Check ID:** `ifeo_debugger` · **ကတ်တလောက် ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei စစ်ဆေးသည့်အရာ:** Nested filter key များ အပါအဝင် native/32-bit Image File Execution Options registration များတွင် `Debugger` value များကို ရှာဖွေသည်။

**စစ်ဆေးသော အထောက်အထား:**

- Nested filter key များနှင့် စက်၏ `Wow6432Node` view အပါအဝင် `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`။

**စုံစမ်းစစ်ဆေးနည်း:** အမည်ဖော်ထားသော image ကို စတင်ဖွင့်သည့်အခါ debugger ကို ရွေးချယ်နိုင်သည်။ Image/filter နယ်ပယ်နှင့် debugger argument များကို စစ်ဆေးပါ။ Developer tool များကလည်း IFEO ကို အသုံးပြုနိုင်သည်။ လုပ်ဆောင်မှုကို အတည်ပြုရန် ပစ်မှတ် စတင်ဖွင့်မှုကို ခြေရာခံပါ။

### IFEO Delegated NTDLL {#ifeo_delegated_ntdll}

**Check ID:** `ifeo_delegated_ntdll` · **ကတ်တလောက် ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei စစ်ဆေးသည့်အရာ:** Nested key များ အပါအဝင် native/32-bit Image File Execution Options registration များတွင် `VerifierDlls` override များကို ရှာဖွေသည်။

**စစ်ဆေးသော အထောက်အထား:**

- Nested key များနှင့် စက်၏ `Wow6432Node` view အပါအဝင် `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`။

**စုံစမ်းစစ်ဆေးနည်း:** စစ်ဆေးမှု၏ predicate သည် ပြင်ဆင်ထားသော VerifierDlls value ဖြစ်သည်။ ၎င်းသည် verifier/delegation ကြိုတင်လိုအပ်ချက်အားလုံး enable လုပ်ထားကြောင်း သက်သေမပြပါ။ အလုပ်လုပ်သော injection လမ်းကြောင်းဟု ကောက်ချက်မချမီ သက်ဆိုင်ရာ flag များ၊ Windows ဗားရှင်းနှင့် payload ကို ပြန်လည်သုံးသပ်ပါ။

### Silent Process Exit Monitor {#ifeo_silent_process_exit}

**Check ID:** `ifeo_silent_process_exit` · **ကတ်တလောက် ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei စစ်ဆေးသည့်အရာ:** Nested key များ အပါအဝင် native/32-bit SilentProcessExit registration များတွင် `MonitorProcess` ကို ရှာဖွေသည်။

**စစ်ဆေးသော အထောက်အထား:**

- Nested key များနှင့် စက်၏ `Wow6432Node` view အပါအဝင် `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`။

**စုံစမ်းစစ်ဆေးနည်း:** Silent-exit monitoring သည် သက်ဆိုင်ရာ reporting/IFEO ပြင်ဆင်မှုကိုလည်း လိုအပ်သည်။ ထိုဆက်တင်များနှင့် စောင့်ကြည့်ခံရသော image ၏ exit event များကို စစ်ဆေးပါ။ တရားဝင် diagnostic tool များက monitor ကို ရှင်းပြနိုင်သည်။

### Application Shimming {#installed_sdb}

**Check ID:** `installed_sdb` · **ကတ်တလောက် ATT&CK:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei စစ်ဆေးသည့်အရာ:** ထည့်သွင်းထားသော application compatibility ဒေတာဘေ့စ်များအတွက် တိုက်ရိုက် InstalledSDB registration များမှ `DatabasePath` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**စုံစမ်းစစ်ဆေးနည်း:** Compatibility fix များသည် တရားဝင် ဖြစ်နိုင်သည်။ SDB အကြောင်းအရာ၊ ပစ်မှတ်ထားသော executable များနှင့် ထည့်သွင်းမှု မှတ်တမ်းကို စစ်ဆေးပါ။ ဒေတာဘေ့စ် registration သည် ၎င်း၏ shim များကို ဖော်ထုတ်ခြင်း သို့မဟုတ် ကိုက်ညီသော အပလီကေးရှင်း run ခဲ့ကြောင်း သက်သေပြခြင်း မပြုပါ။

### LSM Debugger Hijack {#lsm_debugger}

**Check ID:** `lsm_debugger` · **ကတ်တလောက် ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei စစ်ဆေးသည့်အရာ:** Native နှင့် 32-bit စက် view များရှိ `lsm.exe` အတွက် SilentProcessExit အောက်မှ `MonitorProcess` ကို သီးသန့် ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**စုံစမ်းစစ်ဆေးနည်း:** ၎င်းသည် ပိုကျယ်ပြန့်သော SilentProcessExit စာရင်းနှင့် ထပ်နေသည်။ ပြင်ဆင်မှုကို လုပ်ဆောင်ခဲ့သော persistence အဖြစ် မသဘောထားမီ ကြိုတင်လိုအပ်သော flag များနှင့် သက်ဆိုင်ရာ LSM exit ကို အတည်ပြုပါ။ ကွဲပြားသော တွေ့ရှိချက်များကို အကဲဖြတ်သည့်အခါ တူညီသော entry ကို တစ်ကြိမ်သာ ရေတွက်ပါ။

### MiniDump Auxiliary DLLs {#minidump_auxiliary_dlls}

**Check ID:** `minidump_auxiliary_dlls` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** MiniDumpAuxiliaryDlls ရှိ အမည်ပါ၍ ဗလာမဟုတ်သော value များကို စာရင်းကောက်ပြီး value အမည်ကို trigger-module အခြေအနေအဖြစ်၊ value ဒေတာကို ရွေးချယ်ထားသော ပစ်မှတ်အဖြစ် ထိန်းသိမ်းသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: trigger-module value အမည်များနှင့် ပစ်မှတ်ဒေတာ။

**စုံစမ်းစစ်ဆေးနည်း:** Trigger module နှင့် auxiliary DLL နှစ်ခုစလုံးကို စစ်ဆေးပါ။ Crash/minidump tool များက entry ကို ရှင်းပြနိုင်ပြီး registration သည် dump လုပ်ဆောင်ချက်က library ကို load လုပ်ခဲ့ကြောင်း မပြပါ။

### Application Shimming {#shim_custom}

**Check ID:** `shim_custom` · **ကတ်တလောက် ATT&CK:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei စစ်ဆေးသည့်အရာ:** တိုက်ရိုက် AppCompatFlags Custom application key များအောက်ရှိ `.sdb` အမည်ပါ value များကို ဖတ်ပြီး အပလီကေးရှင်းနှင့် ဒေတာဘေ့စ် ဆက်နွယ်မှုကို ထိန်းသိမ်းသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: `.sdb` ဖြင့် အဆုံးသတ်သော value အမည်များ။

**စုံစမ်းစစ်ဆေးနည်း:** ဤဆက်နွယ်မှုများကို InstalledSDB နှင့် အမှန်တကယ် compatibility ဒေတာဘေ့စ်တို့နှင့် တွဲဖက်စစ်ဆေးပါ။ ဖိုင်အမည် ဆက်နွယ်မှုတစ်ခုတည်းက ဒေတာဘေ့စ်၏ အပြုအမူကို မဖော်ပြဘဲ ကိုက်ညီသော အပလီကေးရှင်း စတင်ဖွင့်မှုကိုလည်း မပြသပါ။

### WER Debugger Hijack {#wer_debugger}

**Check ID:** `wer_debugger` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** Native/32-bit Windows Error Reporting `Debugger` value များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**စုံစမ်းစစ်ဆေးနည်း:** တရားဝင် diagnostic ဆော့ဖ်ဝဲက debugger ကို ပြင်ဆင်နိုင်သည်။ ညွှန်ကြားချက် အပြည့်အစုံနှင့် သက်ဆိုင်ရာ crash event များကို စုံစမ်းစစ်ဆေးပါ။ Registration သည် Windows Error Reporting က ၎င်းကို စတင်ခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။

### WER Hangs Debugger Hijack {#wer_hangs}

**Check ID:** `wer_hangs` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** Native/32-bit Windows Error Reporting Hangs `Debugger` value များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**စုံစမ်းစစ်ဆေးနည်း:** Hang-handling လုပ်ငန်းစဉ်က ဤဆက်တင်ကို ခေါ်ယူမည် ဟုတ်/မဟုတ် စစ်ဆေးပြီး ပစ်မှတ် ပရိုဂရမ်ကို စစ်ဆေးပါ။ Registry ရေးသားမှုကို debugger လုပ်ဆောင်မှုအဖြစ် မသဘောထားဘဲ application hang နှင့် process မှတ်တမ်းများနှင့် ချိတ်ဆက်စစ်ဆေးပါ။

### WER ReflectDebugger Hijack {#wer_reflect_debugger}

**Check ID:** `wer_reflect_debugger` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** Native/32-bit Windows Error Reporting `ReflectDebugger` value များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**စုံစမ်းစစ်ဆေးနည်း:** ခွင့်ပြုထားသော diagnostic ပြင်ဆင်မှုနှင့် နှိုင်းယှဉ်ပြီး Windows ဗားရှင်း/trigger သက်ဆိုင်မှုကို အတည်ပြုပါ။ လုပ်ဆောင်မှုကို သတ်မှတ်မီ ညွှန်ကြားချက် argument များကို စစ်ဆေးပြီး crash/process အတည်ပြုချက်ကို ရှာပါ။

### WER Runtime Exception Modules {#wer_runtime_exception}

**Check ID:** `wer_runtime_exception` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** Native/32-bit RuntimeExceptionHelperModules အောက်ရှိ value အမည်များကို စာရင်းကောက်သည်။ ဤနေရာတွင် library path ကို value ၏ ဒေတာတွင် မဟုတ်ဘဲ value အမည်တွင် ထည့်သွင်းထားသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` နှင့် စက်၏ `Wow6432Node` view- value အမည်များ။

**စုံစမ်းစစ်ဆေးနည်း:** အမည်ဖော်ထားသော module နှင့် အပလီကေးရှင်း/runtime အခြေအနေကို စစ်ဆေးပါ။ ဒေတာရှိ ဂဏန်းတန်ဖိုးငယ်သည် payload path မဟုတ်ဘဲ registration တစ်ခုတည်းက helper ကို load လုပ်ခဲ့ကြောင်း သက်သေမပြပါ။

## .NET runtime နှင့် profiler ဆက်တင်များ { #net-runtime-and-profiler-settings }

Managed runtime များသည် အပလီကေးရှင်း code မတိုင်မီ profiler များ၊ startup assembly များ သို့မဟုတ် debugging library များကို load လုပ်နိုင်သည်။ ဤဆက်တင်များကို developer များနှင့် monitoring ထုတ်ကုန်များကလည်း အသုံးပြုသည်။ Runtime၊ process architecture၊ enable flag များနှင့် အသစ်စတင်သော process က ဆက်တင်ကို အမွေဆက်ခံနိုင်ခြင်း ရှိ/မရှိကို စစ်ဆေးပါ။

### AppDomainManager Injection {#appdomain_manager}

**Check ID:** `appdomain_manager` · **ကတ်တလောက် ATT&CK:** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**Juntei စစ်ဆေးသည့်အရာ:** စိတ်ကြိုက် .NET AppDomain manager အတွက် assembly နှင့် type ရွေးချယ်မှုကို လွှမ်းခြုံသော စက်/အသုံးပြုသူ `APPDOMAIN_MANAGER_ASM` နှင့် `APPDOMAIN_MANAGER_TYPE` environment variable များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**စုံစမ်းစစ်ဆေးနည်း:** အပလီကေးရှင်းများသည် စိတ်ကြိုက် manager များကို တမင်အသုံးပြုနိုင်သည်။ Assembly နှင့် type ကို အတူတကွ ဖော်ထုတ်ပြီး runtime လိုက်ဖက်မှုနှင့် process အမွေဆက်ခံမှုကို စစ်ဆေးပါ။ Variable တစ်ခုတည်း သို့မဟုတ် ၎င်း၏ key timestamp ကမျှ managed အပလီကေးရှင်းက ၎င်းကို load လုပ်ခဲ့ကြောင်း သက်သေမပြပါ။

### .NET CLR Profiler Hijack {#cor_profiler}

**Check ID:** `cor_profiler` · **ကတ်တလောက် ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei စစ်ဆေးသည့်အရာ:** စက်/အသုံးပြုသူ COR profiling environment ဆက်တင်များနှင့် ဝန်ဆောင်မှု တစ်ခုချင်းစီ၏ Environment multi-string များရှိ ကိုက်ညီသော အမည်ပါ entry များကို ဖတ်သည်။ ၎င်းသည် enable flag များ၊ profiler identifier များနှင့် architecture အလိုက် path များကို ထိန်းသိမ်းသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` နှင့် ဖမ်းယူထားသော အသုံးပြုသူ NTUSER.DAT တစ်ခုစီ၏ `Environment`။
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: multi-string ရှိ အမည်ပါ COR ဆက်တင်များ။

**စုံစမ်းစစ်ဆေးနည်း:** Profiler များသည် တရားဝင် monitoring ကို ပံ့ပိုးသည်။ Enable flag၊ CLSID နှင့် DLL path တို့သည် ပြင်ဆင်မှု၏ ကွဲပြားသော အစိတ်အပိုင်းများ ဖြစ်သည်။ Row တစ်ခုစီကို လုပ်ဆောင်ခဲ့သော profiler အဖြစ် မသဘောထားဘဲ သက်ရောက်မှုရှိသော enable လုပ်ထားမှုနှင့် သက်ဆိုင်ရာ .NET process ကို အတည်ပြုပါ။

### .NET Core CLR Profiler Hijack {#coreclr_profiler}

**Check ID:** `coreclr_profiler` · **ကတ်တလောက် ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei စစ်ဆေးသည့်အရာ:** Architecture အလိုက် path များ အပါအဝင် သက်ဆိုင်ရာ စက်/အသုံးပြုသူ CORECLR profiling environment ဆက်တင်များနှင့် ကိုက်ညီသော ဝန်ဆောင်မှု Environment entry များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` နှင့် ဖမ်းယူထားသော အသုံးပြုသူ NTUSER.DAT တစ်ခုစီ၏ `Environment`။
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: multi-string ရှိ အမည်ပါ CORECLR ဆက်တင်များ။

**စုံစမ်းစစ်ဆေးနည်း:** .NET runtime၊ architecture၊ enable flag နှင့် library ကို စစ်ဆေးအတည်ပြုပါ။ ခွင့်ပြုထားသော observability ထုတ်ကုန်များက ဤဆက်တင်များကို ပြင်ဆင်နိုင်ပြီး ရှိပြီးသား process သည် အသစ်ပြောင်းလဲထားသော environment ကို အမွေဆက်ခံမည် မဟုတ်နိုင်ပါ။

### .NET DbgManagedDebugger Hijack {#dotnet_dbg_managed_debugger}

**Check ID:** `dotnet_dbg_managed_debugger` · **ကတ်တလောက် ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei စစ်ဆေးသည့်အရာ:** ရှင်းလင်းစွာ ဖော်ပြထားသော စက် 32-bit path အပါအဝင် ဖော်ပြထားသော စက်/အသုံးပြုသူ `.NETFramework` `DbgManagedDebugger` ဆက်တင်များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**စုံစမ်းစစ်ဆေးနည်း:** သက်ဆိုင်ရာ .NET runtime/debugging policy က ဤညွှန်ကြားချက်ကို အသုံးပြုခြင်း ရှိ/မရှိ ဆုံးဖြတ်ပါ။ Developer tool များက ၎င်းကို ရှင်းပြနိုင်သည်။ လုပ်ဆောင်မှုကို အတည်ပြုရန် managed crash များနှင့် process ဖန်တီးမှုတို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။

### .NET Framework COR_PROFILER Registry Key {#dotnet_framework_profiler}

**Check ID:** `dotnet_framework_profiler` · **ကတ်တလောက် ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei စစ်ဆေးသည့်အရာ:** ဖော်ပြထားသော `.NETFramework` registry view များမှ `COR_PROFILER`၊ `COR_PROFILER_PATH` နှင့် `COR_ENABLE_PROFILING` ကို ဖတ်သည်။ ၎င်းသည် environment အခြေပြု profiler စစ်ဆေးမှုများကို ဖြည့်စွက်ပေးသည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**စုံစမ်းစစ်ဆေးနည်း:** Value များကို အတူတကွ အကဲဖြတ်ပြီး runtime က ထို registry တည်နေရာကို လိုက်နာခြင်း ရှိ/မရှိ ဆုံးဖြတ်ပါ။ ပြင်ဆင်ထားသော profiling tool သည် တရားဝင် ဖြစ်နိုင်သည်။ သီးခြားရှိနေသော enable flag ကို ဖော်ထုတ်ထားသော DLL load အဖြစ် အဓိပ္ပာယ်မဖော်ပါနှင့်။

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**Check ID:** `dotnet_startup_hooks` · **ကတ်တလောက် ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei စစ်ဆေးသည့်အရာ:** ပံ့ပိုးထားသော .NET runtime များအတွက် startup-hook assembly များကို ရွေးချယ်နိုင်သော စက်/အသုံးပြုသူ `DOTNET_STARTUP_HOOKS` environment value များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` နှင့် အသုံးပြုသူ NTUSER.DAT တစ်ခုစီ၏ `Environment\DOTNET_STARTUP_HOOKS`။

**စုံစမ်းစစ်ဆေးနည်း:** Assembly အများအပြားအတွက် value တစ်ခုလုံးကို စစ်ဆေးပြီး runtime ပံ့ပိုးမှုကို စစ်ဆေးအတည်ပြုပါ။ ဤစစ်ဆေးမှုသည် ဝန်ဆောင်မှုအလိုက် startup-hook environment များကို စာရင်းကောက်ခြင်း သို့မဟုတ် မည်သည့် process မဆို value ကို အမွေဆက်ခံ၍ အသုံးပြုခဲ့ကြောင်း သက်သေပြခြင်း မပြုပါ။

### Known Managed Debugging DLLs {#known_managed_debugging_dlls}

**Check ID:** `known_managed_debugging_dlls` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** `.NETFramework` `KnownManagedDebuggingDlls` registration များကို ဖြစ်နိုင်ချေရှိသော managed-debugger library override များအဖြစ် ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**စုံစမ်းစစ်ဆေးနည်း:** ထည့်သွင်းထားသော development/runtime tool များနှင့် နှိုင်းယှဉ်ပြီး trigger လုပ်သော debugger ကို ဆုံးဖြတ်ပါ။ Registry entry သည် debugger session သို့မဟုတ် library load ကို သက်သေမပြပါ။

## DLL နှင့် executable ရှာဖွေမှု override များ { #dll-and-executable-lookup-overrides }

ဤစစ်ဆေးမှုများသည် ရှင်းလင်းစွာ သတ်မှတ်ထားသော library တည်နေရာများ သို့မဟုတ် component တစ်ခုက code ကို ရှာဖွေပုံကို လွှမ်းမိုးသော ဆက်တင်များကို စစ်ဆေးသည်။ Path override သို့မဟုတ် အားနည်းစေထားသော ရှာဖွေမှု policy သည် စုံစမ်းစစ်ဆေးရေး သဲလွန်စဖြစ်သည်။ လုပ်ဆောင်မှုအတွက် payload နှင့် လိုက်ဖက်သော trigger လုပ်သည့် component တို့ လိုအပ်နေဆဲ ဖြစ်သည်။

### AppCert DLLs {#appcert_dlls}

**Check ID:** `appcert_dlls` · **ကတ်တလောက် ATT&CK:** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**Juntei စစ်ဆေးသည့်အရာ:** Process-creation API များနှင့် ဆက်စပ်သော library extension မျက်နှာပြင်ဖြစ်သည့် ဗလာမဟုတ်သော Session Manager AppCertDlls registration များကို စာရင်းကောက်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | ဗလာမဟုတ်သော value အားလုံး |

**စုံစမ်းစစ်ဆေးနည်း:** Library တစ်ခုစီနှင့် ၎င်း၏ ထည့်သွင်းမှု မူလအရင်းအမြစ်ကို စစ်ဆေးပါ။ သက်ဆိုင်မှုနှင့် runtime loading ကို သီးခြားစီ ဆုံးဖြတ်ပါ။ Timeline ရှိ ပုံမှန် process ဖန်တီးမှုသည် ၎င်းတစ်ခုတည်းဖြင့် ပြင်ဆင်ထားသော DLL load လုပ်ခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။

### AppInit DLLs {#appinit_dlls}

**Check ID:** `appinit_dlls` · **ကတ်တလောက် ATT&CK:** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**Juntei စစ်ဆေးသည့်အရာ:** Native/32-bit `AppInit_DLLs` ကို ဖတ်ပြီး ၎င်း၏ library စာရင်းကို ခွဲထုတ်ကာ ရရှိနိုင်သော `LoadAppInit_DLLs` နှင့် `RequireSignedAppInit_DLLs` အခြေအနေကို ကိုယ်စားလှယ်လောင်းတစ်ခုစီသို့ ထည့်ပေးသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs`၊ `LoadAppInit_DLLs` နှင့် `RequireSignedAppInit_DLLs`; စက်၏ `Wow6432Node` view လည်း ပါဝင်သည်။

**စုံစမ်းစစ်ဆေးနည်း:** သုည LoadAppInit_DLLs flag ကို inactive အဖြစ် ရှင်းလင်းစွာ အမှတ်အသားပြုထားသည်။ Windows ဗားရှင်း၊ Secure Boot နှင့် signing လိုအပ်ချက်များကလည်း loading ကို သက်ရောက်စေသည်။ AppInit DLL လုပ်ဆောင်ခဲ့သည်ဟု အခိုင်အမာ မဆိုမီ သက်ရောက်မှုရှိသော ဆက်တင်များကို ပြန်လည်သုံးသပ်ပါ။

**နောက်ထပ် အသေးစိတ်များ:** [Microsoft စာရွက်စာတမ်း](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls)။

### AutodialDLL Override {#autodial_dll}

**Check ID:** `autodial_dll` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** WinSock2 Parameters `AutodialDLL` override ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**စုံစမ်းစစ်ဆေးနည်း:** ဤဆက်တင်ကို အသုံးပြုသော networking feature နှင့် source Windows ဗားရှင်းကို စစ်ဆေးပါ။ အမှန်တကယ် DLL နှင့် ကွန်ရက်/process လှုပ်ရှားမှုကို ပြန်လည်သုံးသပ်ပါ။ Override တစ်ခုတည်းက active သို့မဟုတ် အန္တရာယ်ရှိသော load ကို မသတ်မှတ်နိုင်ပါ။

### CHM Helper DLL {#chm_helper_dll}

**Check ID:** `chm_helper_dll` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** help ဆိုင်ရာ library/location override မျက်နှာပြင်ဖြစ်သော HtmlHelp Author အောက်ရှိ `Location` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**စုံစမ်းစစ်ဆေးနည်း:** ဤစနစ်ပေါ်ရှိ သက်ဆိုင်ရာ help component က value ကို အသုံးပြုခြင်း ရှိ/မရှိ ဆုံးဖြတ်ပြီး ရွေးချယ်ထားသော ဖိုင်ကို စစ်ဆေးပါ။ Registry override သည် CHM စာရွက်စာတမ်း ဖွင့်ခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။

### NL6 ContentIndex DLL Override {#content_index_dll}

**Check ID:** `content_index_dll` · **ကတ်တလောက် ATT&CK:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei စစ်ဆေးသည့်အရာ:** Active control set ၏ တိုက်ရိုက် ContentIndex Language registration များမှ `DLLOverridePath` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`။

**စုံစမ်းစစ်ဆေးနည်း:** Language/search component များသည် Windows build အလိုက် ကွဲပြားနိုင်သည်။ Library နှင့် ကိုက်ညီသော ထည့်သွင်းထားသည့် component ကို စစ်ဆေးပါ။ Loading ကို အတည်ပြုရန် အမှန်တကယ် indexing process ကို လိုက်ကြည့်ပါ။

### Crypto ExpoOffload DLL {#crypto_expo_offload}

**Check ID:** `crypto_expo_offload` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Cryptography Offload `ExpoOffload` value ကို cryptographic library override ကိုယ်စားလှယ်လောင်းအဖြစ် ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**စုံစမ်းစစ်ဆေးနည်း:** Feature/build သက်ဆိုင်မှုကို အတည်ပြုပြီး ရွေးချယ်ထားသော library ၏ မူလအရင်းအမြစ်ကို စုံစမ်းစစ်ဆေးပါ။ ရှားပါးသော value တည်ရှိခြင်းကို credential ခိုးယူမှု သို့မဟုတ် လုပ်ဆောင်ခဲ့သော code ၏ သက်သေအဖြစ် မသဘောထားပါနှင့်။

### Direct3D Software Rasterizer DLL {#direct3d_dll}

**Check ID:** `direct3d_dll` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Direct3D Drivers `SoftwareRasterizer` library override ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**စုံစမ်းစစ်ဆေးနည်း:** Graphics နှင့် compatibility ဆော့ဖ်ဝဲများက rasterizer ပြင်ဆင်မှုကို လွှမ်းမိုးနိုင်သည်။ DLL နှင့် လိုက်ဖက်သော Direct3D အပလီကေးရှင်း၏ လှုပ်ရှားမှုကို စစ်ဆေးပါ။ Path ကို သိမ်းဆည်းထားခြင်းသည် ၎င်း load လုပ်ခဲ့ကြောင်း သက်သေမပြပါ။

### DLL Search Order Weakening {#dll_search_mode}

**Check ID:** `dll_search_mode` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Active Session Manager value ကို သုညအဖြစ် parse လုပ်ရသည့်အခါမှသာ `SafeDllSearchMode` သို့မဟုတ် `CWDIllegalInDllSearch` ကို အစီရင်ခံသည်။ ၎င်းတို့သည် ရွေးချယ်ထားသော payload များ မဟုတ်ဘဲ ရှာဖွေမှု policy အခြေအနေများ ဖြစ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` နှင့် `CWDIllegalInDllSearch`။

**စုံစမ်းစစ်ဆေးနည်း:** အားနည်းစေထားသော ရှာဖွေမှု policy သည် သီးခြား search-order တိုက်ခိုက်မှုကို ပိုမိုလွယ်ကူစေနိုင်သည်။ ၎င်းသည် ထည့်သွင်းထားသော DLL ကို ဖော်ထုတ်ခြင်း သို့မဟုတ် လုပ်ဆောင်မှုကို သက်သေပြခြင်း မပြုပါ။ သုညမဟုတ်သော သို့မဟုတ် မရှိသော value များကို ဤစစ်ဆေးမှုက ထုတ်ပေးခြင်း မရှိပါ။ သက်ရောက်မှုရှိသော ပုံသေများနှင့် policy ကို သီးခြားစီ အကဲဖြတ်ပါ။

**နောက်ထပ် အသေးစိတ်များ:** [Microsoft စာရွက်စာတမ်း](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security)။

### KnownDLLs Exclusion {#exclude_from_known_dlls}

**Check ID:** `exclude_from_known_dlls` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Session Manager မှ ဗလာမဟုတ်သော `ExcludeFromKnownDlls` entry များကို ကာကွယ်ထားသော/known DLL ရှာဖွေမှု အပြုအမူ ပြောင်းလဲမှုများအဖြစ် ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`။

**စုံစမ်းစစ်ဆေးနည်း:** Exclusion တစ်ခုသည် အစားထိုး library ကိုယ်တိုင်ကို မဟုတ်ဘဲ ရှာဖွေမှု အခြေအနေကို ဖော်ထုတ်သည်။ သက်ရောက်ခံရသော အပလီကေးရှင်းနှင့် ဖြစ်နိုင်ချေရှိသော DLL ရှာဖွေမှု တည်နေရာများကို ရှာပြီးနောက် စံမဟုတ်သော image load ကို အတည်ပြုပါ။

**နောက်ထပ် အသေးစိတ်များ:** [Microsoft စာရွက်စာတမ်း](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order)။

### hhctrl.ocx DLL Override {#hhctrl_ocx_dll}

**Check ID:** `hhctrl_ocx_dll` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** အကောင်အထည်ဖော်ထားသော HTML Help control CLSID အတွက် native/32-bit စက်နှင့် ပံ့ပိုးထားသော အသုံးပြုသူ class-store InprocServer32 override များကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**စုံစမ်းစစ်ဆေးနည်း:** မည်သည့် အပလီကေးရှင်းနှင့် architecture က control ကို activate လုပ်သည်ကို ဆုံးဖြတ်ပါ။ Register လုပ်ထားသော DLL နှင့် help ဆိုင်ရာ လှုပ်ရှားမှုကို စစ်ဆေးပါ။ COM registration သည် HTML Help က အကြောင်းအရာကို render လုပ်ခဲ့ကြောင်း သက်သေမပြပါ။

### Known DLLs {#known_dlls}

**Check ID:** `known_dlls` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Active Session Manager KnownDLLs key ရှိ အမည်ပါ value များကို စာရင်းကောက်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: အမည်ပါ value များ။

**စုံစမ်းစစ်ဆေးနည်း:** Entry အများအပြားသည် မျှော်လင့်ထားသော Windows library များဖြစ်ပြီး value အချို့သည် payload path မဟုတ်ဘဲ ထောက်ပံ့ပေးသော ပြင်ဆင်မှု ဖြစ်သည်။ Key တစ်ခုလုံးကို ယုံကြည်ရသော ကိုက်ညီသည့် build နှင့် နှိုင်းယှဉ်ပါ။ ဤစစ်ဆေးမှုသည် architecture အလိုက် အခြား KnownDLLs tree တိုင်းကို scan မလုပ်ပါ။

### LSA Extensions DLL {#lsa_extensions}

**Check ID:** `lsa_extensions` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** LsaExtensionConfig LsaSrv ပြင်ဆင်မှုမှ `Extensions` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**စုံစမ်းစစ်ဆေးနည်း:** လုံခြုံရေးဆော့ဖ်ဝဲများက authentication component များကို ချဲ့ထွင်နိုင်သည်။ အမှန်တကယ် library များနှင့် source Windows ဗားရှင်းနှင့် လိုက်ဖက်မှုကို စစ်ဆေးပြီးနောက် LSASS image-load သို့မဟုတ် အခြား သီးခြားလွတ်လပ်သော အသုံးပြုမှု အထောက်အထားကို ရှာပါ။

### MAPI32 DLL Path Override {#mapi32_dll_path}

**Check ID:** `mapi32_dll_path` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** Native နှင့် 32-bit စက် view များရှိ တိုက်ရိုက် Mail client registration များအောက်မှ `DLLPathEx` နှင့် `DLLPath` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**စုံစမ်းစစ်ဆေးနည်း:** Mail client များသည် တရားဝင် MAPI provider များကို register လုပ်ကြသည်။ မည်သည့် client/provider ကို ရွေးချယ်ခဲ့သည်ကို ဆုံးဖြတ်ပြီး အမှန်တကယ် library ကို စစ်ဆေးပါ။ ထည့်သွင်းထားသော provider သည် incident အတွင်း အပလီကေးရှင်းက MAPI ကို အသုံးပြုခဲ့ကြောင်း သက်သေမပြပါ။

### MSDTC XA DLL {#msdtc_xa_dll}

**Check ID:** `msdtc_xa_dll` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** MSDTC MTxOCI ပြင်ဆင်မှုမှ `OracleXaLib` နှင့် `OracleOciLib` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**စုံစမ်းစစ်ဆေးနည်း:** ဒေတာဘေ့စ် ပေါင်းစပ်မှုသည် ဤ library များကို တရားဝင် လိုအပ်နိုင်သည်။ Transaction-coordinator ပြင်ဆင်မှုနှင့် ထည့်သွင်းထားသော ဒေတာဘေ့စ် ဆော့ဖ်ဝဲကို စစ်ဆေးအတည်ပြုပြီး ပြင်ဆင်ထားသော path run ခဲ့သည်ဟု မယူဆဘဲ service/module အသုံးပြုမှုနှင့် ချိတ်ဆက်စစ်ဆေးပါ။

### NLDP DLL Override {#nldp_dll}

**Check ID:** `nldp_dll` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** NlsData `DllOverridePath` ပြင်ဆင်မှု value ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**စုံစမ်းစစ်ဆေးနည်း:** ပစ်မှတ် library၊ Windows build နှင့် override ကို အသုံးပြုနိုင်သော natural-language component ကို ပြန်လည်သုံးသပ်ပါ။ Value သည် indexing သို့မဟုတ် language processing ဖြစ်ပေါ်ခဲ့ကြောင်း မပြပါ။

### RDP TestDVCPlugin DLL {#rdp_test_dvc_plugin}

**Check ID:** `rdp_test_dvc_plugin` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** စက်၏ Terminal Server Client `TestDVCPlugin` library ဆက်တင်ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**စုံစမ်းစစ်ဆေးနည်း:** မရင်းနှီးသော test plugin နှင့် ဗားရှင်းအလိုက် အပြုအမူကို စုံစမ်းစစ်ဆေးပါ။ လုပ်ဆောင်မှုကို အတည်ပြုရန် လိုက်ဖက်သော client session နှင့် library load လိုအပ်သည်။ Registry value တစ်ခုတည်းက မည်သည့်တစ်ခုကိုမျှ မပြပါ။

### Search Indexer DLL Override {#search_indexer_dll}

**Check ID:** `search_indexer_dll` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** SOFTWARE မှ Windows Search `DllPath` override ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**စုံစမ်းစစ်ဆေးနည်း:** ကိုက်ညီသော Search component/ဗားရှင်းကို ဆုံးဖြတ်ပြီး ပစ်မှတ်ဖိုင်ကို စစ်ဆေးပါ။ Inactive Windows Search ဝန်ဆောင်မှု သို့မဟုတ် အသုံးမပြုသေးသော ဆက်တင်သည် လေ့လာတွေ့ရှိထားသော အန္တရာယ်ရှိသည့် library load နှင့် ကွဲပြားသည်။

### Winsock AutoProxy DLL {#winsock_auto_proxy}

**Check ID:** `winsock_auto_proxy` · **ကတ်တလောက် ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei စစ်ဆေးသည့်အရာ:** တိုက်ရိုက် NameSpace_Catalog5 Catalog_Entries နှင့် Catalog_Entries64 registration များအောက်မှ `LibraryPath` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**စုံစမ်းစစ်ဆေးနည်း:** Networking provider များသည် တရားဝင် ဖြစ်နိုင်သည်။ Architecture နှစ်ခုစလုံး၊ provider ၏ မူလအရင်းအမြစ်နှင့် သက်ဆိုင်ရာ ကွန်ရက် အပလီကေးရှင်း/image-load အထောက်အထားကို ပြန်လည်သုံးသပ်ပါ။ ဤပစ်မှတ်ထားသော namespace-provider ရှာဖွေမှုသည် ဖြစ်နိုင်သော Winsock extension တိုင်း မဟုတ်ပါ။

## အကောင့်များနှင့် sign-in မြင်သာမှု { #accounts-and-sign-in-visibility }

စက်တွင်း အကောင့်မှတ်တမ်းများနှင့် sign-in မြင်သာမှု ဆက်တင်များသည် ကိုယ်ပိုင်အမှတ်အသား အခြေပြု persistence ကို ဖော်ထုတ်နိုင်သည်။ ဤစစ်ဆေးမှုများသည် ဖမ်းယူထားသော SAM နှင့် SOFTWARE hive များကို စစ်ဆေးသည်။ ၎င်းတို့သည် Active Directory အကောင့်များ၊ domain group အဖွဲ့ဝင်မှု သို့မဟုတ် ယခင် logon များ၏ စာရင်း မဟုတ်ပါ။

### Administrators Group Membership {#admin_group_membership}

**Check ID:** `admin_group_membership` · **ကတ်တလောက် ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei စစ်ဆေးသည့်အရာ:** SAM မှ local Administrators alias အဖွဲ့ဝင်စာရင်းကို decode လုပ်သည်။ ၎င်းသည် ဖုံးကွယ်ထားသော အကောင့်ကို အမည်ဖော်သော သို့မဟုတ် ဖမ်းယူထားသော SAM တွင် အကောင့်/group မရှိသော local-machine SID များကို အစီရင်ခံသည်။

**စစ်ဆေးသော အထောက်အထား:**

- SAM hive ရှိ `SAM\Domains\Builtin\Aliases\00000220\C` နှင့် `SAM\Domains\Account` အောက်ရှိ machine SID/user/group မှတ်တမ်းများ။
- ဖုံးကွယ်ထားသော အမည် ချိတ်ဆက်စစ်ဆေးမှုအတွက် `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`။

**စုံစမ်းစစ်ဆေးနည်း:** ၎င်းသည် ပစ်မှတ်ထားသော ပုံမှန်မဟုတ်မှု စစ်ဆေးမှုဖြစ်ပြီး administrator အားလုံး၏ စာရင်း မဟုတ်ပါ။ ပြင်ပ/domain SID များကို ပျောက်နေသော local အသုံးပြုသူများအဖြစ် အကဲမဖြတ်ပါ။ ဖျက်ထားသော အကောင့်များ၊ ဖမ်းယူချိန် သို့မဟုတ် မပြည့်စုံသော hive များသည် အန္တရာယ်ရှိသော အဖွဲ့ဝင်မှုဟု ကောက်ချက်မချမီ စုံစမ်းစစ်ဆေးရန် လိုအပ်သည်။

### Hidden Local Account {#hidden_account}

**Check ID:** `hidden_account` · **ကတ်တလောက် ATT&CK:** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**Juntei စစ်ဆေးသည့်အရာ:** Winlogon SpecialAccounts UserList ရှိ ဂဏန်း သုည value များကို အစီရင်ခံပြီး ပုံမှန် sign-in မျက်နှာပြင်မှ ဖုံးကွယ်ထားသော အမည်များကို ဖော်ထုတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: အကောင့်အမည် value များ။

**စုံစမ်းစစ်ဆေးနည်း:** ဝန်ဆောင်မှု သို့မဟုတ် ပြုပြင်ထိန်းသိမ်းရေး အကောင့်များကို တမင်ဖုံးကွယ်ထားနိုင်ပြီး value သည် အမည်ဖော်ထားသော အကောင့် တည်ရှိကြောင်း သို့မဟုတ် administrator ဖြစ်ကြောင်း မသတ်မှတ်နိုင်ပါ။ SAM အကောင့်မှတ်တမ်းများနှင့် logon လှုပ်ရှားမှုတို့နှင့် ပေါင်းစပ်စစ်ဆေးပါ။

### RID Hijacking {#rid_hijacking}

**Check ID:** `rid_hijacking` · **ကတ်တလောက် ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei စစ်ဆေးသည့်အရာ:** SAM အကောင့် subkey ၏ hexadecimal RID ကို ၎င်း၏ F value ရှိ offset 0x30 တွင် သိမ်းဆည်းထားသော RID နှင့် နှိုင်းယှဉ်ပြီး မကိုက်ညီမှုများကို အစီရင်ခံသည်။

**စစ်ဆေးသော အထောက်အထား:**

- SAM hive ရှိ `SAM\Domains\Account\Users\<hexadecimal RID>\F`။

**စုံစမ်းစစ်ဆေးနည်း:** မကိုက်ညီမှုသည် ပြင်းထန်သော အကောင့်မှတ်တမ်း ပုံမှန်မဟုတ်မှု ဖြစ်သော်လည်း ပျက်စီးမှု/ဖမ်းယူမှု ခွဲခြမ်းစိတ်ဖြာမှုလည်း လိုအပ်နိုင်သည်။ Source byte များ၊ transaction-log အခြေအနေနှင့် အကောင့်/logon မှတ်တမ်းများကို စစ်ဆေးပါ။ ၎င်းသည် မည်သည့် actor က မှတ်တမ်းကို ပြင်ဆင်ခဲ့သည်ကို သက်သေမပြပါ။

### RID Suborner (Hidden Admin Account) {#rid_suborner}

**Check ID:** `rid_suborner` · **ကတ်တလောက် ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei စစ်ဆေးသည့်အရာ:** Subkey RID သည် 500 မဟုတ်သော်လည်း F-value RID သည် 500 ဖြစ်သော SAM အကောင့်များကို အစီရင်ခံသည်။ ၎င်းသည် ပိုမိုတိကျသော administrator-identity အစားထိုးမှု ပုံစံ ဖြစ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- SAM hive ရှိ `SAM\Domains\Account\Users\<hexadecimal RID>\F`။

**စုံစမ်းစစ်ဆေးနည်း:** ၎င်းသည် ယေဘုယျ RID မကိုက်ညီမှု စစ်ဆေးမှုနှင့် ထပ်နေသည်။ ကတ်တလောက် ခေါင်းစဉ်ရှိသော်လည်း ၎င်းသည် ဖုံးကွယ်ထားသော sign-in ဆက်တင်ကို သီးခြား မလိုအပ်ပါ။ အကောင့် မြင်သာမှု၊ အခွင့်အာဏာများနှင့် အမှန်တကယ် အကောင့်အသုံးပြုမှုကို သီးခြားစီ အတည်ပြုပါ။

## Authentication အစိတ်အပိုင်းများနှင့် credential ကာကွယ်မှု { #authentication-components-and-credential-protection }

Local Security Authority (LSA) သည် LSASS နှင့် ဆက်စပ် component များမှတစ်ဆင့် လုံခြုံရေးလုပ်ဆောင်ချက်များကို ကိုင်တွယ်သည်။ Authentication package များ၊ password filter များနှင့် network provider များသည် တရားဝင် လုံခြုံရေးဆော့ဖ်ဝဲ သို့မဟုတ် ကြာရှည် credential ဝင်ရောက်မှုသို့ လမ်းကြောင်း ဖြစ်နိုင်သည်။ စစ်ဆေးမှု နှစ်ခုသည် executable autostart များထက် ကာကွယ်မှုဆက်တင်များကို စာရင်းကောက်သည်။ ၎င်းတို့ကို host အခန်းကဏ္ဍ၊ Windows ဗားရှင်းနှင့် သက်ရောက်မှုရှိသော policy တို့ကို အသုံးပြု၍ အဓိပ္ပာယ်ဖော်ပါ။

### AMSI Provider DLL {#amsi_providers}

**Check ID:** `amsi_providers` · **ကတ်တလောက် ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei စစ်ဆေးသည့်အရာ:** စက် AMSI provider CLSID များကို စာရင်းကောက်ပြီး ရရှိနိုင်သည့်အခါ ၎င်းတို့၏ native/32-bit InprocServer32 library များကို ဖော်ထုတ်သည်။ ဖော်ထုတ်မရသော registration များသည် class ID ကို ထိန်းသိမ်းထားသည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` နှင့် ဖော်ထုတ်နိုင်သော `Classes\CLSID` / `Classes\Wow6432Node\CLSID` InprocServer32 registration များ။

**စုံစမ်းစစ်ဆေးနည်း:** Antivirus နှင့် လုံခြုံရေး ထုတ်ကုန်များသည် AMSI provider များကို တရားဝင် register လုပ်ကြသည်။ Provider/library ၏ မူလအရင်းအမြစ်နှင့် အမှန်တကယ် loading ကို ပြန်လည်သုံးသပ်ပါ။ Registration သည် AMSI ကို ကျော်ဖြတ်ခဲ့ကြောင်း သို့မဟုတ် provider အန္တရာယ်ရှိကြောင်း သက်သေမပြပါ။

### Authentication Packages {#authentication_packages}

**Check ID:** `authentication_packages` · **ကတ်တလောက် ATT&CK:** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**Juntei စစ်ဆေးသည့်အရာ:** Active control set မှ ဗလာမဟုတ်သော LSA `Authentication Packages` entry တစ်ခုချင်းစီကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**စုံစမ်းစစ်ဆေးနည်း:** Source Windows build နှင့် ထည့်သွင်းထားသော authentication ဆော့ဖ်ဝဲတို့နှင့် နှိုင်းယှဉ်ပါ။ ပုံသေမဟုတ်သော package သည် တရားဝင် ဖြစ်နိုင်သည်။ ကြားဖြတ်ဖမ်းယူမှု (interception) ဟု ကောက်ချက်မချမီ DLL ကို ဖော်ထုတ်ပြီး loading/credential လှုပ်ရှားမှုကို အတည်ပြုပါ။

### DSRM Admin Logon Behavior {#dsrm_backdoor}

**Check ID:** `dsrm_backdoor` · **ကတ်တလောက် ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei စစ်ဆေးသည့်အရာ:** အကောင်အထည်ဖော်ထားသော ControlSet001၊ ControlSet002 သို့မဟုတ် CurrentControlSet တည်နေရာများရှိ ဂဏန်း `DsrmAdminLogonBehavior=2` ကို အစီရင်ခံသည်။ ဤဆက်တင်သည် DSRM အကောင့်ကို ၎င်း၏ ပုံမှန် recovery အခြေအနေ ပြင်ပတွင် အသုံးပြုခွင့် ပေးနိုင်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior` အပြင် ရှိပါက ControlSet002 နှင့် CurrentControlSet။

**စုံစမ်းစစ်ဆေးနည်း:** အဓိကအားဖြင့် domain controller များနှင့် သက်ဆိုင်သည်။ SYSTEM Select ကို အသုံးပြု၍ မည်သည့် control set active ဖြစ်သည်ကို ဆုံးဖြတ်ပါ။ သမိုင်းဝင် control-set ကိုက်ညီမှုသည် သက်ရောက်မှုရှိသည်ဟု မဆိုလိုပါ။ Value သည် တိုက်ခိုက်သူက DSRM password ကို သိခဲ့ကြောင်း သို့မဟုတ် logon ဝင်ခဲ့ကြောင်း သက်သေမပြပါ။

### Credential Guard Configuration {#lsa_cfg_flags}

**Check ID:** `lsa_cfg_flags` · **ကတ်တလောက် ATT&CK:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei စစ်ဆေးသည့်အရာ:** သုညအပြင် ပုံမှန် enable လုပ်ထားသော value များ အပါအဝင် ပြင်ဆင်ထားသော LSA `LsaCfgFlags` value ကို ဖတ်သည်။ ၎င်းသည် executable autostart ထက် Credential Guard ပြင်ဆင်မှုကို စာရင်းကောက်ခြင်း ဖြစ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**စုံစမ်းစစ်ဆေးနည်း:** Value ကို Windows ဗားရှင်း၊ firmware/လုံခြုံရေး ကြိုတင်လိုအပ်ချက်များနှင့် သက်ရောက်မှုရှိသော policy တို့နှင့်အတူ အဓိပ္ပာယ်ဖော်ပါ။ သိမ်းဆည်းထားသော သုည သို့မဟုတ် မရှိသော value တစ်ခုတည်းက တိုက်ခိုက်သူ၏ ပြောင်းလဲမှု၊ သက်ရောက်မှုရှိသော ကာကွယ်မှုအခြေအနေ သို့မဟုတ် credential ခိုးယူမှုကို မသတ်မှတ်နိုင်ပါ။

### LSA Password Filter {#lsa_password_filter}

**Check ID:** `lsa_password_filter` · **ကတ်တလောက် ATT&CK:** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**Juntei စစ်ဆေးသည့်အရာ:** LSA `Notification Packages` ကို ဖတ်သည်။ ရိုးရိုး library အမည်များကို ဖမ်းယူထားသော Windows System32 directory နှင့် ယှဉ်၍ ဖော်ထုတ်ပြီး သင့်လျော်သည့်နေရာတွင် `.dll` ကို ထည့်ပေးသည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**စုံစမ်းစစ်ဆေးနည်း:** Password policy ထုတ်ကုန်များသည် filter များကို အများအားဖြင့် ထည့်သွင်းကြသည်။ Library နှင့် loading အထောက်အထားကို စစ်ဆေးပြီး ပြင်ဆင်ထားသော filter များကို password ပြောင်းလဲမှု သို့မဟုတ် credential ကြားဖြတ်ဖမ်းယူမှု ဖြစ်ပေါ်ခဲ့ကြောင်း သက်သေမှ ခွဲခြားပါ။

### LSASS PPL Protection Status {#lsa_run_as_ppl}

**Check ID:** `lsa_run_as_ppl` · **ကတ်တလောက် ATT&CK:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei စစ်ဆေးသည့်အရာ:** သုညမဟုတ်သော value များ အပါအဝင် ပြင်ဆင်ထားသော LSA `RunAsPPL` value ကို ဖတ်သည်။ ၎င်းသည် ကာကွယ်မှုဆက်တင် စာရင်းကောက်မှုဖြစ်ပြီး LSASS ကာကွယ်မှုမရှိဘဲ run ခဲ့ကြောင်း စမ်းသပ်ချက် မဟုတ်ပါ။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**စုံစမ်းစစ်ဆေးနည်း:** သက်ရောက်မှုရှိသော LSA ကာကွယ်မှုသည် Windows ဗားရှင်း၊ policy၊ firmware locking နှင့် restart အခြေအနေပေါ် မူတည်သည်။ မရှိသော value အတွက် row မရှိခြင်းသည် တိုင်းတာထားသော ကာကွယ်မှုရလဒ် မဟုတ်ပါ။ သက်ရောက်မှုရှိသော ကာကွယ်မှုကို အကဲဖြတ်ရန် သီးခြားလွတ်လပ်သော စနစ်/runtime အထောက်အထားကို အသုံးပြုပါ။

### Network Provider DLL {#network_provider_dll}

**Check ID:** `network_provider_dll` · **ကတ်တလောက် ATT&CK:** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**Juntei စစ်ဆေးသည့်အရာ:** Active SYSTEM control set ရှိ ဝန်ဆောင်မှုများအောက်မှ `NetworkProvider\ProviderPath` ကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`။

**စုံစမ်းစစ်ဆေးနည်း:** Network logon provider များသည် တရားဝင် ဖြစ်နိုင်သည်။ Provider အစီအစဉ်/သက်ဆိုင်မှု၊ ထည့်သွင်းထားသော ကွန်ရက်/လုံခြုံရေး ဆော့ဖ်ဝဲနှင့် အမှန်တကယ် library loading ကို စစ်ဆေးပါ။ ProviderPath value တစ်ခုတည်းက credential ကြားဖြတ်ဖမ်းယူမှုကို သက်သေမပြပါ။

### Security Packages {#security_packages}

**Check ID:** `security_packages` · **ကတ်တလောက် ATT&CK:** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**Juntei စစ်ဆေးသည့်အရာ:** ပြင်ဆင်ထားသော security support provider များကို လွှမ်းခြုံသည့် ဗလာမဟုတ်သော LSA `Security Packages` entry တစ်ခုချင်းစီကို ဖတ်သည်။

**စစ်ဆေးသော အထောက်အထား:**

| Registry key | စစ်ဆေးသော value |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**စုံစမ်းစစ်ဆေးနည်း:** မျှော်လင့်ထားသော Windows/လုံခြုံရေးထုတ်ကုန် baseline နှင့် နှိုင်းယှဉ်ပြီး library တစ်ခုစီကို စစ်ဆေးပါ။ ပြင်ဆင်မှုသည် provider load လုပ်ခဲ့ကြောင်း၊ credential များကို ဖမ်းယူခဲ့ကြောင်း သို့မဟုတ် incident အတွင်း ထပ်ထည့်ခဲ့ကြောင်း သက်သေမပြပါ။

## ကိုယ်စားလှယ်လောင်းမှ ကောက်ချက်သို့ ဆောင်ရွက်ခြင်း { #work-from-a-candidate-to-a-conclusion }

1. `SourceFile` က ဖော်ထုတ်ထားသော မူရင်း source ကို ဖွင့်ပါ။ Value အပြည့်အစုံ၊ XML action၊
   script သို့မဟုတ် ပြန်လည်ရယူထားသော ဒေတာဘေ့စ် မှတ်တမ်းနှင့် ၎င်းပိုင်ဆိုင်သော အသုံးပြုသူ/host အတိအကျကို အတည်ပြုပါ။
2. ယန္တရား လုပ်ဆောင်နိုင်ခြင်း ရှိ/မရှိ ဆုံးဖြတ်ပါ- enable flag များ၊ runtime သို့မဟုတ် Windows ဗားရှင်း၊
   service/task အခြေအနေ၊ လိုအပ်သော trigger၊ အသုံးပြုသူ၊ architecture နှင့် active control set ကို စစ်ဆေးပါ။
3. ရွေးချယ်ထားသော ပရိုဂရမ်၊ DLL၊ script သို့မဟုတ် assembly ကို စစ်ဆေးပါ။ စုဆောင်းထားသော byte များကို hash လုပ်ပြီး argument များနှင့်
   အကြောင်းအရာကို ပြန်လည်သုံးသပ်ပါ။ မှတ်မိနိုင်သော Windows launcher သို့မဟုတ် publisher string သည် တရားဝင်မှုကို မဆုံးဖြတ်ပေးပါ။
4. သီးခြားလွတ်လပ်သော အထောက်အထားကို အသုံးပြု၍ အမှန်တကယ် ဖြစ်ပျက်ခဲ့သည်ကို အတည်ပြုပါ။ ဥပမာများတွင် process ဖန်တီးမှု၊
   Sysmon image load များ၊ Task Scheduler runtime မှတ်တမ်းများ၊ ဝန်ဆောင်မှု လှုပ်ရှားမှု၊ Prefetch နှင့် ဆက်စပ်ဖိုင်များ ပါဝင်သည်။
   ပြင်ဆင်မှုပြောင်းလဲခြင်း event သည် ဆက်တင်ပြောင်းလဲခဲ့ကြောင်း သက်သေပြနိုင်သော်လည်း ၎င်း၏ payload run ခဲ့ကြောင်း သက်သေမပြပါ။
5. မူရင်း source timestamp များနှင့် မှတ်တမ်းတင်ထားသော ချိတ်ဆက်စစ်ဆေးမှု မူလအရင်းအမြစ်ကို အသုံးပြု၍ timeline တည်ဆောက်ပါ။
   Key ရေးသားမှုများ၊ ထည့်သွင်းမှု ပြောင်းလဲမှုများ၊ ဖိုင်/container ရေးသားမှုများနှင့် လုပ်ဆောင်မှုများကို သီးခြား အချက်အလက်များအဖြစ် ထားပါ။
6. တွေ့ရှိချက်များအပြင် ကန့်သတ်ချက်များကိုလည်း မှတ်တမ်းတင်ပါ- ပျောက်နေသော hive များ၊ ပျောက်နေသော ပစ်မှတ်ဖိုင်များ၊ မသိရှိရသော task principal များ၊
   စစ်ဆေးအတည်မပြုရသေးသော signer metadata၊ carve လုပ်ထားသော သမိုင်းဝင်မှတ်တမ်းများ သို့မဟုတ် အတည်မပြုနိုင်သော trigger။

ညွှန်ကြားချက် syntax၊ profile များ၊ ထုတ်ယူမှု field တိုင်းနှင့် coverage အခြေအနေများအတွက်
[Persistence Hunter ညွှန်ကြားချက် လမ်းညွှန်](../commands/persistence-hunter.md) သို့ ပြန်သွားပါ။
[အထောက်အထား လမ်းညွှန်များ](../artifacts/index.md) က အခြေခံ source များကို ရှင်းပြပြီး
[ကျေးဇူးတင်လွှာ](acknowledgements.md) က upstream persistence tool များကို ဖော်ပြသည်။
