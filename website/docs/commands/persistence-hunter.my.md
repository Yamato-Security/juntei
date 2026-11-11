# Persistence Hunter { #persistence-hunter }

အမြဲတမ်းဝင်ရောက်နိုင်မှု (persistence) သည် ပရိုဂရမ်တစ်ခုကို reboot၊ sign-in၊ အချိန်ဇယားသတ်မှတ်ထားသော trigger သို့မဟုတ်
အခြား event တစ်ခုပြီးနောက် ထပ်မံ run စေသည်။ တိုက်ခိုက်သူတစ်ဦးသည် ဤရည်ရွယ်ချက်အတွက် ပုံမှန် Windows ပြင်ဆင်မှုကို
အသုံးပြုနိုင်သည်။ တရားဝင်ပရိုဂရမ် အများအပြားသည် ယန္တရားအတိအကျကို အသုံးပြုကြသောကြောင့် persistence entry တစ်ခုကို
တွေ့ရှိခြင်းသည် ခွဲခြမ်းစိတ်ဖြာမှု၏ စတင်ရာ အမှတ်သာ ဖြစ်သည်။

`juntei persistence-hunter` သည်
[Hexastrike ၏ PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper) ၏ native Rust port ဖြစ်ပြီး
စစ်ဆေးမှု 131 ခု ပါဝင်သည်- စစ်ဆေးမှု 130 ခုပါသော PyrsistenceSniper catalogue နှင့် native Start Menu COM စစ်ဆေးမှု
တစ်ခု။

ယန္တရားတိုင်း၊ ၎င်း၏ အထောက်အထား တည်နေရာများနှင့် တွေ့ရှိချက်တစ်ခုကို စုံစမ်းစစ်ဆေးပုံ ရှင်းလင်းချက်အတွက်
[စစ်ဆေးမှု 131 ခုပါ persistence လမ်းညွှန်](../persistence-hunter/index.md) ကို ကြည့်ပါ။
[Persistence ကျေးဇူးတင်လွှာ](../persistence-hunter/acknowledgements.md) တွင် catalogue ၏ နောက်ကွယ်ရှိ Python နှင့်
PowerShell project များကို ဖော်ပြထားသည်။

## အသုံးပြုပုံ { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## ပထမဆုံး scan { #first-scan }

```bash
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

အထောက်အထားသည် mount လုပ်ထားသော Windows volume၊ Velociraptor စုဆောင်းမှု သို့မဟုတ် စုဆောင်းထားသော ကွန်ပျူတာများစွာ
ပါဝင်သည့် directory တစ်ခု ဖြစ်နိုင်သည်။ ခွဲခြမ်းစိတ်ဖြာစဉ် ကွန်ပျူတာများကို သီးခြားစီ ထားရှိပြီး ရလဒ်တိုင်းတွင် `Comp`
ပါဝင်သည်။ Windows path များကို ဖြေရှင်းသည့်အခါ စုဆောင်းထားသော ဖိုင်အမည် escape လုပ်ခြင်းကို decode လုပ်သည်။

Hive တစ်ခုအတွက်-

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

Hive တစ်ခုတည်းသည် အခြား hive သို့မဟုတ် ဖိုင်စနစ် အထောက်အထား လိုအပ်သော စစ်ဆေးမှုများကို မဖြည့်ဆည်းပေးနိုင်ပါ။ ဗလာရလဒ်ကို
အဓိပ္ပာယ်ရှိသည်ဟု မသဘောထားမီ coverage အစီရင်ခံစာကို ဖတ်ပါ။

Check ID များနှင့် ၎င်းတို့၏ ATT&CK mapping များကို စာရင်းပြုစုရန်-

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

`--list-checks` က ပုံနှိပ်ပြသော check ID များ သို့မဟုတ် `T1547.001` ကဲ့သို့သော ATT&CK ID ကို အသုံးပြုပါ။
`T1547` ကဲ့သို့သော parent ID သည် ၎င်း၏ sub-technique များကို ရွေးချယ်သည်။ `--technique` သည် `--check` ၏ alias ဖြစ်သည်။
စစ်ဆေးမှုတစ်ခုစီသည် ၎င်း၏ technique၊ ဖော်ပြချက်နှင့် ကိုးကား link များကို တွေ့ရှိချက်များတွင် ထည့်သွင်းထားသည်။

## အပြိုင် scan များ { #parallel-scans }

ညွှန်ကြားချက်သည် မျှဝေထားသော worker pool ကို အသုံးပြုပြီး ပုံသေအားဖြင့် logical CPU အရေအတွက် ဖြစ်သည်။ Worker အရေအတွက်ကို
ရွေးချယ်ရန် `-T, --threads <N>` ကို သတ်မှတ်ပါ၊ သို့မဟုတ် single-thread scan အတွက် `-T 1` ကို သတ်မှတ်ပါ-

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

အထောက်အထား ရှာဖွေခြင်း၊ သီးခြားလွတ်လပ်သော စစ်ဆေးမှုများ၊ payload metadata နှင့် MFT/EVTX timeline ရင်းမြစ်များသည်
အပြိုင် run သည်။ Host တစ်ခု၏ load လုပ်ထားသော registry hive များကို read-only အဖြစ် မျှဝေပြီး တူညီသော payload အတွက်
cache လုပ်ထားသော metadata ကို တစ်ကြိမ်သာ initialize လုပ်သည်။ စုဆောင်းမှုတစ်လျှောက် hive memory များပြားခြင်းကို ရှောင်ရှားရန်
host များကို တစ်ကြိမ်လျှင် တစ်ခုစီ load လုပ်သည်။ တွေ့ရှိချက်များ၊ coverage၊ timestamp ရွေးချယ်မှုနှင့် သတိပေးချက် အစီအစဉ်တို့သည်
thread အရေအတွက် ကွဲပြားသော်လည်း တူညီစွာ ရှိနေသည်။

`auto-parse` သည် parser process များကို အပြိုင် run ပြီးဖြစ်သောကြောင့် `persistence-hunter` child တစ်ခုစီကို worker
တစ်ခု ပေးသည်။ ၎င်း၏ ကိုယ်ပိုင် `-T` option သည် တစ်ပြိုင်နက် run သော parser process အရေအတွက်ကို ထိန်းချုပ်သည်။

## စုဆောင်းရမည့် အထောက်အထား { #evidence-to-collect }

| ရင်းမြစ် | စစ်ဆေးမှု ဥပမာများ |
|---|---|
| `SYSTEM` နှင့် ဘေးရှိ transaction log များ | ဝန်ဆောင်မှုများ၊ ဝန်ဆောင်မှု ပြန်လည်ထူထောင်ရေး ညွှန်ကြားချက်များ၊ LSA package များ၊ boot ဆက်တင်များ၊ authentication provider များ |
| `SOFTWARE` နှင့် ဘေးရှိ transaction log များ | Run key များ၊ Winlogon၊ IFEO၊ COM မှတ်ပုံတင်မှုများ၊ TaskCache၊ Office ဆက်တင်များ |
| အသုံးပြုသူတစ်ဦးစီ၏ `NTUSER.DAT` နှင့် `UsrClass.dat` | အသုံးပြုသူ autostart များ၊ environment variable များ၊ COM override များ၊ Office ဆက်တင်များ |
| `SAM` နှင့် `SOFTWARE` | RID ပုံမှန်မဟုတ်မှုများ၊ hidden အကောင့်များ၊ ပုံမှန်မဟုတ်သော local Administrators အဖွဲ့ဝင်ခြင်း |
| `Windows/System32/Tasks` | Scheduled-task ညွှန်ကြားချက်များနှင့် COM handler များ၊ TaskCache နှင့် နှိုင်းယှဉ်ခြင်း |
| အသုံးပြုသူအလိုက် Start Menu `.library-ms` ဖိုင်များနှင့် CLSID ဖြင့် အမည်ပေးထားသော ဖိုင်တွဲများ | ကိုက်ညီသော အသုံးပြုသူ/စက် class မှတ်ပုံတင်မှုများနှင့် ချိတ်ဆက်ထားသော COM activation-trigger ကိုယ်စားလှယ်လောင်းများ |
| Startup ဖိုင်တွဲများ၊ PowerShell profile များ၊ Office template များ၊ Terminal ဆက်တင်များ | အသုံးပြုသူ သို့မဟုတ် စနစ် trigger များက run သော ဖိုင်များနှင့် ညွှန်ကြားချက်များ |
| WMI `OBJECTS.DATA` | Event consumer instance များနှင့် filter-to-consumer binding များ |
| BITS `qmgr.db` သို့မဟုတ် `qmgr*.dat` | ပြန်လည်ရယူထားသော notification ညွှန်ကြားချက်များနှင့် owner SID များ |
| `$MFT` နှင့် သက်ဆိုင်သော EVTX ဖိုင်များ | သီးခြားလွတ်လပ်သော timestamp ချိတ်ဆက်စစ်ဆေးမှု |
| ကိုးကားထားသော executable များ၊ DLL များနှင့် catalog ဖိုင်များ | SHA256 hash များနှင့် ဖော်ပြချက်ဆိုင်ရာ PE/certificate metadata |

Registry transaction log များ ရရှိနိုင်သည့်အခါ replay လုပ်သည်။ `--no-logs` သည် hive များကို စုဆောင်းထားသည့်အတိုင်း ဖတ်သည်။
Replay မလုပ်ရသေးသော dirty hive တွင် မကြာသေးမီက ပြောင်းလဲမှုများ ပျောက်နေနိုင်ပြီး coverage တွင် ဖော်ပြသည်။ လက်ရှိ control
set ကို active-system စစ်ဆေးမှုများအတွက် အသုံးပြုသည်။ DSRM backdoor စစ်ဆေးမှုသည် ထို့အပြင် `ControlSet001`,
`ControlSet002` နှင့် `CurrentControlSet` ရှိပါက ၎င်းတို့ကို စစ်ဆေးသည်။ ထိုဆက်တင်ကို active ဟု မဖော်ပြမီ ၎င်း၏
ဖော်ပြထားသော path ကို `SYSTEM\\Select` နှင့် တိုက်ဆိုင်စစ်ဆေးပါ။ အခြား သမိုင်းဆိုင်ရာ control set များကို registry parser
ညွှန်ကြားချက် တစ်ခုချင်းစီဖြင့် စစ်ဆေးနိုင်သည်။

Hive ဖတ်ခြင်းများသည် ကွန်ပျူတာတစ်လုံးလျှင် 1 GiB input budget ကို မျှဝေသုံးစွဲသည်။ စက် hive များကို အသုံးပြုသူ hive များမတိုင်မီ
ကြိုးစားဖတ်သည်။ မဆံ့သော input များ သို့မဟုတ် transaction log များကို partial coverage တွင် အမည်ဖော်ပြသည်။ ဖတ်နိုင်သော
အခြား hive များနှင့် ဖိုင်အခြေပြု စစ်ဆေးမှုများ ဆက်လက်လုပ်ဆောင်သည်။ Canonical hive တည်နေရာများ သို့မဟုတ် တိကျစွာ ပေးထားသော
export များသာ hive ၏ ကိုယ်ပိုင်အမှတ်အသားကို သတ်မှတ်သည်။ အသုံးပြုသူ၏ ယာယီ directory အတွင်းရှိ မိတ္တူတစ်ခုသည် ထိုအသုံးပြုသူ၏
အစစ်အမှန် hive ကို အစားမထိုးနိုင်ပါ။ SOFTWARE တွင် မှတ်တမ်းတင်ထားသော မူရင်း system drive ကို ဖမ်းယူထားသော system volume
သို့ map လုပ်သောကြောင့် မူရင်း `C:` volume ကို အခြား evidence drive label အောက်တွင် mount လုပ်ခြင်းသည် payload metadata
ကို မပယ်ဖျက်ပါ။

`auto-parse` သည် hunter ကိုလည်း run ပြီး ကွန်ပျူတာ directory တစ်ခုစီအတွင်း `persistence/PersistenceHunter.csv` (သို့မဟုတ်
`.json`/`.jsonl`) ကို ၎င်း၏ ဘေးတွင် `PersistenceHunter-logs.txt` နှင့်အတူ သိမ်းဆည်းသည်။ ၎င်း၏ တွေ့ရှိချက်များသည်
ရွေးချယ်နိုင်သော `-j <FILE>` SIEM timeline ထဲသို့လည်း ဝင်ရောက်သည်။ အလိုအလျောက် ခွဲခြမ်းစိတ်ဖြာခြင်းသည် allow rule များနှင့်
ကိုက်ညီသော entry များ အပါအဝင် ပြင်ဆင်မှု စာရင်းအပြည့်အစုံကို ထိန်းသိမ်းထားရန် `--min-severity info` ကို အသုံးပြုသည်။

## COM မှတ်ပုံတင်မှုများနှင့် activation trigger များ { #com-registrations-and-activation-triggers }

`scheduled_task_files` စစ်ဆေးမှုသည် task XML ၏ `Actions Context` နှင့် ကိုက်ညီသော `Principal id` ကို အသုံးပြု၍
action တစ်ခုစီ၏ principal ကို ရွေးချယ်သည်။ Group principal ကို group အဖြစ် ထိန်းသိမ်းထားသည်။
၎င်းသည် မည်သည့် အသုံးပြုသူတစ်ဦးချင်း၏ class မှတ်ပုံတင်မှုများကို အသုံးပြုမည်ကို မသတ်မှတ်နိုင်ပါ။

COM-handler ကိုယ်စားလှယ်လောင်းများကို native နှင့် 32-bit class view များတွင် သီးခြားစီ ဖြေရှင်းသည်။ ကိုက်ညီသော
အသုံးပြုသူ၏ ဖမ်းယူထားသော `UsrClass.dat`/`NTUSER.DAT` နှင့် စက်၏ `SOFTWARE` ကိုသာ ထည့်သွင်းစဉ်းစားသည်။
SID principal များကို ဖမ်းယူထားသော `ProfileList` path များမှတစ်ဆင့် ချိတ်ဆက်သည်။ `HOST\user` နှင့် `DOMAIN\user`
အပါအဝင် အကောင့်အမည်များသည် ရိုးရိုး `user` profile ထက် ကိုက်ညီသော `user.DOMAIN` profile ကို ဦးစားပေးသည်။
Local host alias များနှင့် `.\user` သည် suffix ကိုက်ညီမှုအတွက် ဖမ်းယူထားသော host အမည်များကို အသုံးပြုသည်။
တိကျသော suffix သည် အခြား host alias များထက် ဦးစားပေးခံရသည်။ ရိုးရိုး profile ကို qualified profile တစ်ခုမျှ
မကိုက်ညီသည့်အခါမှသာ ထည့်သွင်းစဉ်းစားသည်။ ထိုကဲ့သို့သော ကိုက်ညီမှုတိုင်းကို `Description` တွင် **folder-name heuristic**
ဟု တိကျစွာ အမည်တပ်ထားသည်။ ၎င်းသည် အကောင့်ပိုင်ဆိုင်မှု သို့မဟုတ် domain အဖွဲ့ဝင်ခြင်းကို သက်သေမပြပါ။ ရှင်းလင်းမှုမရှိသော
ကိုက်ညီမှုများကို မဖြေရှင်းဘဲ ထားရှိပြီး တွေ့ရှိချက် ဖော်ပြချက်နှင့် coverage နှစ်ခုစလုံးတွင် ဖော်ပြသည်။ Qualified profile
များအကြား ရှင်းလင်းမှုမရှိခြင်းသည် ရိုးရိုး profile သို့ fallback ကို မဖြစ်စေပါ။ ၎င်းသည် ပုံမှန်အားဖြင့် memory တွင်သာ ရှိပြီး
registry hive များနှင့်အတူ မသိမ်းဆည်းသော `Volatile Environment` ပေါ်တွင် မမှီခိုပါ။ ထပ်နေသော အသုံးပြုသူ CLSID သည် စက်
CLSID ၏ child များကို ဖုံးအုပ်ထားသည်။ Group principal များသည် စက် COM ကိုယ်စားလှယ်လောင်းများကို အသုံးပြုပြီး ထိုကန့်သတ်ချက်ကို
တွေ့ရှိချက် ဖော်ပြချက်တွင် ထိန်းသိမ်းထားသည်။ ပုံမှန် group principal တစ်ခုသည် ၎င်းကိုယ်တိုင် coverage ကို partial ဖြစ်စေခြင်း
သို့မဟုတ် error-summary entry များကို သုံးစွဲခြင်း မပြုပါ။

Resolver သည် `InprocServer32` နှင့် `LocalServer32` ကို လိုက်နာပြီး managed `CodeBase` path များ (version subkey
များ အပါအဝင်) ကို အသုံးပြုကာ AppID `LocalService` ကို စုဆောင်းထားသော SYSTEM ဝန်ဆောင်မှု `ImagePath` နှင့်
ချိတ်ဆက်သည်။ AppID executable-name ဆက်စပ်မှုများကို ထိန်းသိမ်းထားသော်လည်း basename တစ်ခုသည် executable ၏ path
အပြည့်အစုံကို မသတ်မှတ်ပါ။ Local `file:` CodeBase URL များကို decode လုပ်သည်။ အဝေးရှိ URL များကို ဝင်ရောက်ခြင်းမပြုဘဲ
မှတ်တမ်းတင်သည်။ ပျောက်နေသော မှတ်ပုံတင်မှုများသည် CLSID ကို `ResolutionStatus: unresolved_com_class`၊ ဗလာ
path/hash field များနှင့် မသိရှိသော `Exists` တို့ဖြင့် ထိန်းသိမ်းထားသည်။

`start_menu_com` စစ်ဆေးမှုသည် အသုံးပြုသူအလိုက် Start Menu `.library-ms` XML URL များကို ဖတ်ပြီး ဗလာ directory များ
အပါအဝင် CLSID ဖြင့် အမည်ပေးထားသော ဖိုင်တွဲများကို စုဆောင်းသည်။ ၎င်းသည် trigger path၊ CLSID၊ ကိုယ်စားလှယ်လောင်း server
နှင့် registry ရင်းမြစ်မှတ်တမ်းကို ဖော်ပြသည်။ ဖိုင်သာ ဖမ်းယူသော acquisition သည် ဗလာ directory ကို ထိန်းသိမ်းမထားနိုင်ပါ၊
ပျောက်နေသော အသုံးပြုသူ hive များသည် အသုံးပြုသူအလိုက် မှတ်ပုံတင်မှု ရှာဖွေခြင်းကို ပြီးပြည့်စုံစွာ မပြုလုပ်နိုင်စေပါ။ Symbolic link
များကို မလိုက်နာပါ။ ဤအရာများသည် ပြင်ဆင်မှု ကိုယ်စားလှယ်လောင်းများ ဖြစ်သည်- architecture၊ elevation၊ UAC၊ policy နှင့်
activation context တို့သည် မှတ်ပုံတင်မှုကို အမှန်တကယ် အသုံးပြုခြင်း ရှိ/မရှိကို သက်ရောက်နိုင်သည်။ Execution ကို process၊
module-load သို့မဟုတ် အခြား သီးခြားလွတ်လပ်သော အထောက်အထားများဖြင့် အတည်ပြုပါ။

## အမျိုးအစားခွဲခြင်းနှင့် profile များ { #classification-and-profiles }

ပုံသေ အနိမ့်ဆုံး အမျိုးအစားမှာ `medium` ဖြစ်သည်။ ထည့်သွင်းပါရှိသော allowlist နှင့် ကိုက်ညီသော entry များ အပါအဝင်
စာရင်းအပြည့်အစုံကို စစ်ဆေးရန်-

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "မပြည့်စုံသော စုဆောင်းမှုများကို ပြန်လည်သုံးသပ်သည့်အခါ severity အားလုံးကို အသုံးပြုပါ"
    ကျယ်ပြန့်သော allow rule တစ်ခုသည် signer ပျောက်နေသည့်အခါ သို့မဟုတ် ကွဲပြားသည့်အခါတွင်ပင် တိုက်ခိုက်သူ၏ ညွှန်ကြားချက်နှင့်
    တစ်စိတ်တစ်ပိုင်း ကိုက်ညီနိုင်သည်။ Upstream သည် တစ်စိတ်တစ်ပိုင်း ကိုက်ညီမှုများကို `LOW` အဖြစ် အမျိုးအစားခွဲပြီး လက်ဖြင့်
    run သော ညွှန်ကြားချက်၏ ပုံသေ `medium` threshold က ၎င်းတို့ကို ဖုံးကွယ်ထားသည်။ ၎င်းသည် Windows directory များအောက်ရှိ
    အန္တရာယ်ရှိသော scheduled task များကို သက်ရောက်နိုင်သည်။ ပြီးပြည့်စုံသော စုံစမ်းစစ်ဆေးမှုအတွက် `--min-severity info` ကို
    အသုံးပြုပြီး `PolicyReason`, `SignerStatus`၊ ညွှန်ကြားချက်နှင့် source အထောက်အထားကို အတူတကွ ပြန်လည်သုံးသပ်ပါ။
    `auto-parse` သည် ဤ entry များကို ထိန်းသိမ်းထားပြီး ဖြစ်သည်။

| အမျိုးအစား | အဓိပ္ပာယ် |
|---|---|
| `HIGH` | ပြင်ဆင်ထားသော block rule တစ်ခု အပြည့်အဝ ကိုက်ညီခဲ့သည် |
| `MEDIUM` | Allow သို့မဟုတ် block rule တစ်ခုမျှ မကိုက်ညီခဲ့ပါ |
| `LOW` | Allow rule ၏ hard condition များ ကိုက်ညီခဲ့သော်လည်း signer metadata မကိုက်ညီခဲ့ သို့မဟုတ် မရရှိနိုင်ခဲ့ပါ |
| `INFO` | Allow rule တစ်ခု အပြည့်အဝ ကိုက်ညီခဲ့သည် |

ဤအရာများသည် policy အမျိုးအစားခွဲခြင်းများ ဖြစ်ပြီး တိုင်းတာထားသော ထိုးဖောက်ခံရနိုင်ခြေ ဖြစ်နိုင်ခြေများ မဟုတ်ပါ။ ပုံသေ `MEDIUM`
entry တစ်ခုသည် malware ၏ သက်သေ မဟုတ်ပါ။ `INFO` သည် ဖိုင်တစ်ခုတွင် မှန်ကန်သော signature ရှိကြောင်း သို့မဟုတ်
တိုက်ခိုက်သူက ၎င်းကို အလွဲသုံးစားမပြုခဲ့ကြောင်း မသတ်မှတ်ပါ။ တစ်စိတ်တစ်ပိုင်း allow ကိုက်ညီမှုသည် ပုံသေ threshold အောက်တွင်
ရှိသောကြောင့် ဒေတာနည်းသော စုဆောင်းမှုများကို ပြန်လည်သုံးသပ်သည့်အခါ သို့မဟုတ် allowlist ကို ချိန်ညှိသည့်အခါ
`--min-severity low` သို့မဟုတ် `info` ကို ထည့်သွင်းပါ။

`--profile profile.yaml` ဖြင့် YAML profile တစ်ခု ပေးပါ။ ၎င်းသည် ထည့်သွင်းပါရှိသော profile ကို **အစားထိုးသည်**။
ရွေးချယ်ထားသော ဖိုင်အတွင်းရှိ global နှင့် စစ်ဆေးမှုအလိုက် rule များကို ပေါင်းစပ်သည်။ အပြည့်အဝ ကိုက်ညီသော block rule
မည်သည့်တစ်ခုမဆို အနိုင်ရသည်။ ဥပမာ-

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

ပံ့ပိုးထားသော rule condition များမှာ `path_matches`, `value_matches`, `hash` (hexadecimal SHA256 ဂဏန်း 64 လုံး),
`not_lolbin` နှင့် `signer` (စာလုံးအကြီးအသေး မခွဲခြားသော substring) ဖြစ်သည်။ ဖော်ပြထားသော hard condition အားလုံး
ကိုက်ညီရမည်။ Signer metadata သည် အထက်တွင် ဖော်ပြထားသော soft condition ဖြစ်သည်။ Regex များသည် Rust ၏ regular-expression
syntax ကို အသုံးပြုသည်- lookaround များနှင့် backreference များကို မပံ့ပိုးပါ။ မသိသော check ID များ၊ မမှန်ကန်သော expression
များနှင့် မသိသော profile field များသည် policy ကို တိတ်တဆိတ် အားနည်းစေခြင်းမပြုဘဲ error ကို ထုတ်ပေးသည်။

## ထုတ်ယူမှု field များ { #output-fields }

CSV သည် ပုံသေ ဖိုင် format ဖြစ်သည်။ ဖတ်ရလွယ်သော JSON object များအတွက် `-t json` ကို သို့မဟုတ် တစ်ကြောင်းလျှင် object
တစ်ခုပါသော ကျစ်လျစ်သည့် output အတွက် `-t jsonl` ကို အသုံးပြုပါ။ `-o` မပါဘဲ တိကျစွာ ပေးထားသော `-t` သည် ဒေတာကိုသာ stdout
သို့ ရေးသည်။ Coverage သတိပေးချက်များသည် stderr သို့ သွားသည်။ Option နှစ်ခုစလုံး မပါလျှင် Juntei သည် အနှစ်ချုပ်နှင့်
ကိုယ်စားလှယ်လောင်း 30 ခုအထိကို ပုံနှိပ်ပြသည်။

| Field များ | ၎င်းတို့ကို အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Comp`, `CheckID`, `Technique`, `MitreID` | Row က ဖော်ပြသော ကွန်ပျူတာနှင့် persistence ယန္တရား |
| `Path`, `Value` | Registry တည်နေရာ၊ ဖိုင်၊ task သို့မဟုတ် ပြန်လည်ရယူထားသော မှတ်တမ်းနှင့် ၎င်း၏ ပြင်ဆင်ထားသော အကြောင်းအရာ |
| `Description`, `References` | ယန္တရား အရေးပါရသည့် အကြောင်းရင်းနှင့် ထပ်မံဖတ်ရှုရန် နေရာ |
| `AccessGained` | ယန္တရား၏ ပုံမှန် scope (`USER` သို့မဟုတ် `SYSTEM`၊ BITS သည် `LOCAL_SERVICE`/`NETWORK_SERVICE` ကိုလည်း ခွဲခြားနိုင်သည်)။ ညွှန်ကြားချက်တစ်ခုက ထို privilege ကို ရရှိခဲ့ကြောင်း သက်သေ မဟုတ်ပါ |
| `Severity`, `PolicyReason` | Profile အမျိုးအစားခွဲခြင်း၊ ကိုက်ညီသော policy ရှင်းလင်းချက်နှင့် တစ်စိတ်တစ်ပိုင်းကိုက်ညီမှု သို့မဟုတ် အတည်မပြုရသေးသော signer ဆိုင်ရာ မှတ်ချက် |
| `SourceFile`, `RecordStatus` | စုဆောင်းထားသော ရင်းမြစ်အတိအကျ၊ ပေးထားသည့်အတိုင်း input ဖြင့် စတင်သော ၎င်း၏ path (auto-parse မှ `./HOST/...`) နှင့် allocation/replay/ပြန်လည်ရယူမှု အခြေအနေ။ ၎င်းကို မူရင်းအထောက်အထားဆီသို့ ပြန်လည်ခြေရာခံပါ |
| `KeyLastWrite` | Registry **key** ၏ timestamp။ တန်ဖိုးတစ်ခုချင်းစီကို ရက်စွဲမသတ်မှတ်ပါ |
| `LastChange`, `ChangeSource`, `ChangeEvidence` | သီးခြားလွတ်လပ်စွာ ချိတ်ဆက်စစ်ဆေးထားသော ပြောင်းလဲချိန်၊ ရင်းမြစ် သက်သေနှင့် ချိတ်ဆက်စစ်ဆေးမှု အောင်မြင်ခဲ့ခြင်း ရှိ/မရှိ |
| `ResolvedPath`, `ResolutionStatus`, `Exists` | Payload path နှင့် offline ဖြေရှင်းမှု ရလဒ်။ မစုဆောင်းရသေးသော အထောက်အထားသည် မသိရှိခြင်း ဖြစ်ပြီး `false` မဟုတ်ပါ |
| `SHA256` | ဖတ်နိုင်သည့်အခါ စုဆောင်းထားသော payload byte များ၏ SHA256 |
| `Signer`, `SignerStatus` | ဖော်ပြချက်ဆိုင်ရာ embedded certificate/catalog metadata နှင့် ၎င်း၏ ရရှိနိုင်မှု။ Certificate ယုံကြည်မှုနှင့် signature မှန်ကန်မှုကို အတည်မပြုပါ |
| `IsLolbin`, `Launcher` | ပြင်ဆင်ထားသော launcher သို့မဟုတ် payload သည် ထည့်သွင်းပါရှိသော LOLBAS စာရင်းတွင် ပါဝင်ခြင်း ရှိ/မရှိနှင့် proxy လုပ်ထားသော payload အတွက် အသုံးပြုသော launcher |
| `IsBuiltin`, `IsInOS-Directory` | အသုံးများသော Windows ညွှန်ကြားချက်များနှင့် system directory များအတွက် အခြေအနေ။ မည်သည့် field ကမျှ တရားဝင်မှုကို မသတ်မှတ်ပါ |

`rundll32.exe` ညွှန်ကြားချက်တစ်ခုသည် DLL payload ကို အမည်ဖော်နိုင်သည်။ Juntei သည် payload ကို ဖြေရှင်းပြီး hash
လုပ်စဉ် launcher ကို ထိန်းသိမ်းထားသည်။ စုဆောင်းထားသော ကွန်ပျူတာ ပြင်ပရှိ reference၊ ပျောက်နေသော drive၊ ရှင်းလင်းမှုမရှိသော
စာလုံးအကြီးအသေး မခွဲခြားသည့် ကိုက်ညီမှု သို့မဟုတ် မဖြေရှင်းရသေးသော environment variable ကို စိစစ်သူ၏ စက်မှ ဖိုင်ဖြင့်
အစားထိုးခြင်း မပြုပါ။

## Timestamp များနှင့် ကန့်သတ်ချက်များ { #timestamps-and-limitations }

Hunter သည် စိစစ်သူ VM ၏ ကူးယူထားသော ဖိုင် ပြင်ဆင်ချိန်များကို တိုက်ခိုက်မှု timestamp များအဖြစ် မသုံးပါ။ ရရှိနိုင်သည့်နေရာတွင်
၎င်းသည် registry ရေးသားမှုများကို Sysmon event 13 နှင့်၊ task ထည့်သွင်းမှုကို Security 4698 သို့မဟုတ် Task Scheduler 106
နှင့်၊ ဝန်ဆောင်မှု ထည့်သွင်းမှုကို System 7045 နှင့်၊ ဖိုင်အခြေပြု ပြင်ဆင်မှုကို မူရင်း `$MFT` timestamp များနှင့်
ချိတ်ဆက်စစ်ဆေးသည်။ ရင်းမြစ်မှတ်တမ်းသည် `ChangeSource` တွင် ပေါ်လာသည်။ Event တစ်ခုသည် ပြင်ဆင်ထားသော payload run ခဲ့ကြောင်း
သက်သေမပြဘဲ ပြင်ဆင်မှု ပြောင်းလဲချိန်ကို ရက်စွဲသတ်မှတ်ပေးနိုင်သည်။ Local System event 6011 သည် ယခင် ကွန်ပျူတာအမည်ကို
သက်သေပြနိုင်သည်။ Hunter သည် စုဆောင်းထားသော ကွန်ပျူတာနှင့် ချိတ်ဆက်ထားသော အတည်ပြုပြီး အမည်ပြောင်းမှု ကွင်းဆက်များကို
လိုက်နာပြီး ၎င်း၏ လက်ရှိ `Comp` တန်ဖိုးကို ထိန်းသိမ်းကာ မူရင်း event ကွန်ပျူတာနှင့် အမည်ပြောင်းမှု မှတ်တမ်း သက်သေကို
`ChangeSource` တွင် မှတ်တမ်းတင်သည်။ သက်သေမပြရသေးသော နိုင်ငံခြား အမည်များ၊ archive လုပ်ထားသော log များ၊ forward
လုပ်ထားသော log များနှင့် အခြား Windows installation တစ်ခုကို ဤ rule မှတစ်ဆင့် လက်မခံပါ။

ဖမ်းယူထားသော ComputerName/ActiveComputerName၊ TCP/IP Hostname/NV Hostname နှင့် client-info hostname/FQDN
တို့သည် DNS/NetBIOS ကွာခြားချက်များနှင့် ဆိုင်းငံ့ထားသော အမည်ပြောင်းမှုများကို လွှမ်းခြုံပြီး host alias များကို ပေးသည်။
ဖမ်းယူထားသော ကိုယ်ပိုင်အမှတ်အသား မရှိပါက primary log များရှိ အမည်တစ်ခုတည်းသည် တိကျစွာ ဖော်ပြထားသော
unverified-identity ရင်းမြစ်မှတ်တမ်းနှင့် coverage သတိပေးချက်ဖြင့် ရက်စွဲသတ်မှတ်ခြင်းကို ထောက်ခံနိုင်သည်။ အတည်မပြုရသေးသော
အမည်များစွာကို host သို့ တာဝန်မသတ်မှတ်ပါ။ ငြင်းပယ်ခံရသော Computer အမည်များကို ချိတ်ဆက်စစ်ဆေးမှု coverage ကန့်သတ်ချက်အဖြစ်
ဖော်ပြသည်။ ၎င်းတို့၏ channel များကို စုဆောင်းထားသည်ဟု ဆက်လက် မှတ်သားထားသည်။

ကွန်ပျူတာတစ်လုံးတည်းအတွက် ပြင်ပမှ စုဆောင်းထားသော MFT ကို ရွေးချယ်ရန် `--mft <FILE>` ကို အသုံးပြုပါ။ Timestamp
ချိတ်ဆက်စစ်ဆေးမှုကို ကျော်ရန် `--no-timeline` ကို အသုံးပြုပါ။ `NOT_RUN`, `NOT_APPLICABLE`, `NO_ARTIFACT`, `NO_MATCH`
နှင့် `REJECTED` တို့သည် ဗလာ `LastChange` ကို ရှင်းပြသည်။ ဗလာ field တစ်ခုသည် entry အန္တရာယ်ကင်းကြောင်း အထောက်အထား
မဟုတ်ပါ။

WMI နှင့် BITS ပြန်လည်ရယူခြင်းသည် ပြီးပြည့်စုံသော ဒေတာဘေ့စ် transaction parser မဟုတ်ဘဲ ကန့်သတ်ထားသော carving ဖြစ်သည်။
ပြန်လည်ရယူထားသော string များ သို့မဟုတ် binding များတွင် ဒေတာဟောင်း/ဖျက်ပစ်ထားသော ဒေတာများ ပါဝင်နိုင်သည်။ ဤမှတ်တမ်းများကို
အတည်ပြုရမည်။ WMI class schema များတစ်ခုတည်းကို subscription များအဖြစ် မရေတွက်ပါ။ စုဆောင်းထားသော task XML ဖိုင် မပါသည့်
TaskCache entry သည် ပြန်လည်သုံးသပ်ရန် ကွဲလွဲချက် ဖြစ်သည်။ Triage စုဆောင်းမှုတစ်ခုသည် ထိုဖိုင်ကို ရိုးရိုးပင် ချန်လှပ်ထားခဲ့နိုင်သည်။
Catalog နှင့် PE metadata ကို အတည်မပြုရသေးဘဲ စုဆောင်းမှုတစ်ခုတွင် မပါဝင်နိုင်ပါ။ Accessibility-file အစားထိုးမှု စစ်ဆေးချက်များသည်
သင့်လျော်သော reference signature metadata ပေါ်တွင် မူတည်သည်။ ပျောက်နေသော catalog အထောက်အထားကြောင့် accessibility
executable တိုင်းကို flag မတပ်ရပါ။

Coverage ကို တွေ့ရှိချက် သုည ရှိသော စစ်ဆေးမှုများ အပါအဝင် စစ်ဆေးမှုတိုင်းအတွက် မှတ်တမ်းတင်သည်။ `complete` ဆိုသည်မှာ
သက်ဆိုင်ရာ စုဆောင်းထားသော ရင်းမြစ်ကို ဖတ်ခဲ့သည်ဟု ဆိုလိုသည်။ ၎င်းသည် စုဆောင်းမှု ပြီးပြည့်စုံကြောင်း သို့မဟုတ် ကွန်ပျူတာတွင်
အခြား persistence မရှိကြောင်း သက်သေမပြပါ။ `partial`, `not_collected`, `failed` နှင့် `disabled` တို့သည်
ကန့်သတ်ချက်များကို ရှင်းပြသည်။ `--coverage Coverage.csv` နှင့် parser log ကို သင်၏ တွေ့ရှိချက်များနှင့်အတူ ထိန်းသိမ်းထားပါ။

Port သည် Juntei ၏ CSV/JSON/JSONL format များကို အသုံးပြုသည်။ ၎င်းသည် upstream Python ပရိုဂရမ်ကို မခေါ်ပါ၊ online
enrichment ဝန်ဆောင်မှုများကို query မလုပ်ပါ၊ upstream HTML/XLSX UI ကို ပြန်လည်ထုတ်လုပ်ခြင်း မပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [PyrsistenceSniper အကောင်အထည်ဖော်မှုနှင့် profile များ](https://github.com/hexastrike/PyrsistenceSniper)
- [MITRE ATT&CK persistence technique များ](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft: Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [Registry အလိုအလျောက် စတင်မှု စာရင်း](../artifacts/persistence.md)
- [ဝန်ဆောင်မှုများ](../artifacts/services.md) နှင့် [TaskCache](../artifacts/taskcache.md)
