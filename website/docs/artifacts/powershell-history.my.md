# PowerShell မှတ်တမ်း { #powershell-history }

## မှတ်တမ်းတွင် သိမ်းဆည်းသည့်အရာ { #what-the-history-records }

PSReadLine သည် အပြန်အလှန်အသုံးပြုသော PowerShell host တွင် ရိုက်ထည့်သည့် command စာကြောင်းများကို သိမ်းဆည်းသည်။ ဖိုင်တွင် သိမ်းဆည်းသည့်အစီအစဉ်အတိုင်း စာသားပါဝင်ပြီး command တစ်ခုချင်းစီ၏ အချိန်တံဆိပ် မပါဝင်ပါ။

## တည်နေရာများနှင့် စုဆောင်းခြင်း { #locations-and-collection }

Windows PowerShell 5.1 နှင့် PowerShell 7+ အတွက် Windows ရှိ မူလတည်နေရာမှာ အောက်ပါအတိုင်းဖြစ်သည်။

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

အခြား host များသည် ထို directory တွင် `Visual Studio Code Host_history.txt` ကဲ့သို့ အမည်များကို အသုံးပြုသည်။ Linux နှင့် macOS ရှိ PowerShell သည် ပုံမှန်အားဖြင့် `~/.local/share/powershell/PSReadLine/<Host>_history.txt` ကို အသုံးပြုသည်။ PSReadLine ၏ `HistorySavePath` ဖြင့် ဖိုင်တည်နေရာကို ပြောင်းနိုင်သည်။ ဤမူလလမ်းကြောင်းများသည် ရှာဖွေရာတွင် သတ်မှတ်ထားသည့် ယူဆချက်များသာဖြစ်ပြီး တည်နေရာအားလုံးကို မလွှမ်းခြုံပါ။

Directory input နှင့် `auto-parse` သည် `PSReadLine` folder အတွင်း တိုက်ရိုက်ရှိသော `<Host>_history.txt` ဖိုင်များကို directory အဆင့်ဆင့် ရှာဖွေရွေးချယ်ပြီး ASCII စာလုံးအကြီးအသေးကို မခွဲခြားပါ။ အခြားနေရာရှိ မည်သည့် `*_history.txt` ဖိုင်ကိုမဆို မရွေးချယ်ပါ။ အမည်ပြောင်းထားသည့်ဖိုင် သို့မဟုတ် `PSReadLine` directory ပြင်ပရှိ မိတ္တူအတွက် `-f` input ကို တိတိကျကျ အသုံးပြုပါ။ အသုံးပြုသူကို သတ်မှတ်နိုင်ရန် ဖြစ်နိုင်သမျှ မူရင်း profile ဖွဲ့စည်းပုံကို ထိန်းသိမ်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- မည်သည့်စာကြောင်းတွင်မျှ အချိန် မပါပါ။ `FileLastWriteTime` သည် ဖိုင်၏ ပြင်ဆင်ချိန် ဖြစ်သည်။ PSReadLine သည် မူလအားဖြင့် command တစ်ခုပြီးတိုင်း မှတ်တမ်းကို သိမ်းသောကြောင့် (`HistorySaveStyle` `SaveIncrementally`) ၎င်းသည် နောက်ဆုံးစာကြောင်း ထည့်ခဲ့သည့်အချိန်နှင့် များသောအားဖြင့် နီးစပ်သော်လည်း ယခင်စာကြောင်းများအကြောင်း ဘာမျှ မဖော်ပြပါ။
- သိမ်းထားသော စာကြောင်းတစ်ကြောင်းသည် command လုပ်ဆောင်ခဲ့ကြောင်း သို့မဟုတ် အောင်မြင်စွာ ပြီးဆုံးခဲ့ကြောင်း မသက်သေပြပါ။ လုပ်ဆောင်မှုနှင့် အချိန်ကို PowerShell event log၊ process telemetry နှင့် အခြားလုပ်ဆောင်မှုအထောက်အထားများဖြင့် တိုက်ဆိုင်စစ်ဆေးပါ။
- မှတ်တမ်းကို ပြင်ဆင်၊ ဖျက် သို့မဟုတ် ပိတ်ထားနိုင်သည် (`HistorySaveStyle` `SaveNothing`)။ PSReadLine သည် `password`၊ `token` သို့မဟုတ် `apikey` ပါသော စာကြောင်းများကဲ့သို့ လျှို့ဝှက်ချက်ပါဟန်တူသော စာကြောင်းများကိုလည်း ချန်ထားပြီး Windows PowerShell ISE သည် PSReadLine ကို မသုံးပါ။ Script များနှင့် `-Command` သို့မဟုတ် `-EncodedCommand` ဖြင့် လုပ်ဆောင်သော command များကို ရိုက်ထည့်ခြင်း မဟုတ်သောကြောင့် မသိမ်းပါ။
- စာကြောင်းများစွာပါသော command ကို နောက်ဆုံးစာကြောင်းမှလွဲ၍ စာကြောင်းတိုင်း၏ အဆုံးတွင် backtick ဖြင့် သိမ်းသည်။ Juntei သည် ထိုစာကြောင်းများကို `EntryStartLine` ဖြင့် ချိတ်ဆက်ထားသော သီးခြား row များအဖြစ် ထားသည်။ ရိုက်ထည့်ထားသော backtick ဖြင့် ဆုံးသည့် စာကြောင်းလည်း ထိုနည်းတူ မြင်ရသောကြောင့် entry တစ်ခု၏ စာကြောင်းများကို အတူတကွ ဖတ်ပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

မူလဖိုင်ပုံစံမှာ CSV ဖြစ်သည်။ `-t csv`၊ `-t json` သို့မဟုတ် `-t jsonl` ပါပြီး `-o` မပါလျှင် record များကို stdout သို့ ရေးသည်။ ရွေးချယ်စရာနှစ်ခုလုံး မပါလျှင် အနှစ်ချုပ်သာ ပြသည်။ JSON တွင် ဖတ်ရလွယ်အောင် ပုံစံချထားသည့် သီးခြား object များပါပြီး JSONL တွင် တစ်ကြောင်းလျှင် object တစ်ခု ပါသည်။ ရှိပြီးသား output ကို ထပ်ရေးရန် `-C` ထည့်ပါ။ မူရင်းဖိုင်ကို ညွှန်းသည့် အခြားအမည်လမ်းကြောင်းများကို အစားထိုးမရေးနိုင်အောင် ကာကွယ်ထားသည်။

## စာသားကုဒ်နှင့် စာကြောင်းကိုင်တွယ်ပုံ { #encoding-and-line-handling }

- ဖိုင်အစရှိ UTF-8၊ UTF-16LE သို့မဟုတ် UTF-16BE byte order mark က စာသားကုဒ်ကို သတ်မှတ်ပြီး ထိုအမှတ်အသားကို ဖယ်ရှားသည်။ BOM မပါလျှင် input ကို အမြဲ UTF-8 ဟု ယူဆသည်။ BOM မပါသော UTF-16 ကို မခန့်မှန်းပါ။ NUL စာလုံးများကို ထိန်းသိမ်းပြီး စာသားကုဒ် ပျက်စီးမှု ဖြစ်နိုင်ကြောင်း ဖော်ပြသည်။
- မမှန်သော UTF-8 အစီအစဉ်များနှင့် အတွဲမရှိသော UTF-16 surrogate များကို U+FFFD ဖြင့် အစားထိုးပြီး ပျက်စီးမှုကို `Problems` တွင် ဖော်ပြသည်။ အဆုံး၌ ပိုနေသည့် UTF-16 byte တစ်ခုနှင့် ဖတ်ရှုမှုအမှားများကိုလည်း ဖော်ပြပြီး ဖတ်နိုင်သောစာကြောင်းများကို ထိန်းသိမ်းသည်။ ပြည့်စုံမှုကို အားမကိုးမီ `Problems` နှင့် parser log နှစ်ခုလုံးကို စစ်ဆေးပါ။
- ဖိုင်များကို တစ်ဆင့်ချင်း စီးဆင်းဖတ်ရှုသည်။ ရုပ်ပိုင်းစာကြောင်းတစ်ကြောင်းလျှင် input byte အများဆုံး 1,048,576 ကိုသာ ထိန်းသိမ်းသည်။ BOM နှင့် LF ခွဲခြားသင်္ကေတကို မတွက်ပါ (LF မတိုင်မီ CR ကို ကန့်သတ်ချက်တွင် ထည့်တွက်သည်)။ ပိုရှည်သောစာကြောင်းကို စာလုံးအပြည့် နယ်နိမိတ်၌ ဖြတ်ပြီး ကျန် byte များကို ဆက်ဖတ်သည်။ `Problems` က ချန်လှပ်သည့် byte အရေအတွက်ကို ဖော်ပြသည်။ ကန့်သတ်ချက်သည် ကုဒ်သွင်းထားသော input byte များအပေါ် သက်ရောက်ပြီး စာလုံးအရေအတွက်အပေါ် မဟုတ်ပါ။
- LF နှင့် CRLF စာကြောင်းအဆုံးများကို ဖယ်ရှားသည်။ စာကြောင်းအဆုံးသင်္ကေတမပါသော နောက်ဆုံးစာကြောင်း၏ အဆုံး CR ကိုလည်း ဖယ်ရှားသည်။ အခြား space နှင့် tab များကို ထိန်းသိမ်းသည်။ ဗလာစာကြောင်းများက record မထုတ်သော်လည်း စာကြောင်းနံပါတ်တွင် ထည့်တွက်ပြီး ဆက်ကြောင်းကွင်းဆက်ကို အဆုံးသတ်သည်။ နေရာလွတ်စာလုံးသာပါသော စာကြောင်းများက record အဖြစ် ကျန်သည်။ ဖတ်ရှုမှုအမှားကြောင့် ဖတ်ခြင်းရပ်သည့်နေရာကို ပြသရန် ဗလာစစ်ဆေးရေးစာကြောင်း ထွက်နိုင်သည်။
- စာကြောင်းများကို သီးခြားထားပြီး backtick နှင့် အတွင်းချန်အကွာအဝေးကို ထိန်းသိမ်းသည်။ `EntryStartLine` သည် ရှေ့စာကြောင်းများ backtick ဖြင့်ဆုံးသော ဆက်တိုက်ကွင်းဆက်၏ ပထမစာကြောင်းကို ညွှန်းသည်။ Shell syntax ကို အဓိပ္ပာယ်မဖော်ဘဲ command များကို ပြန်မတည်ဆောက်ပါ။ ဖြတ်တောက်မှုက ဆက်ကြောင်းသတ်မှတ်မှုကို ထိခိုက်နိုင်သဖြင့် ထိုစာကြောင်းများကို အုပ်စုမဖွဲ့မီ `Problems` ကို စစ်ဆေးပါ။

## CSV ထုတ်ယူမှု field များ { #csv-output-fields }

| အကွက် | အဓိပ္ပာယ် |
|---|---|
| `SourceFile` | Directory input နှင့် ဆက်စပ်သောလမ်းကြောင်းဖြစ်ပြီး `/` ကို အသုံးပြုသည်။ `-f` အတွက် ဖိုင်အမည်သာ ဖြစ်သည်။ မူရင်း absolute path ကို မထုတ်ပါ။ |
| `User` | လမ်းကြောင်းမှ ရရှိသော profile အမည်။ `AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine` (သို့မဟုတ် `.local/share/powershell/PSReadLine`) ၏ အပေါ်ရှိ folder က အသုံးပြုသူကို ဖော်ပြပြီး input သည် ထို profile သို့မဟုတ် `PSReadLine` folder ကိုယ်တိုင် ဖြစ်သည့်အခါတွင်လည်း ဤနည်းအတိုင်း သတ်မှတ်သည်။ Directory input သည် input root အောက်ရှိ `Users`/`Documents and Settings` ဖွဲ့စည်းပုံများကိုလည်း သိရှိသည်။ မသိသောတန်ဖိုးများသည် CSV တွင် ဗလာ၊ JSON တွင် `null` ဖြစ်သည်။ |
| `ShellHost` | `_history.txt` မတိုင်မီ ဖိုင်အမည်ရှေ့ပိုင်း၊ ဥပမာ `ConsoleHost`။ ထိုနောက်ဆက်မပါအောင် အမည်ပြောင်းထားသောဖိုင်များအတွက် မသိနိုင်ပါ။ |
| `LineNumber` | 1 မှစသော ရုပ်ပိုင်းစာကြောင်းနံပါတ်၊ ကျော်ထားသော ဗလာစာကြောင်းများပါဝင်သည်။ |
| `EntryStartLine` | ဆက်ကြောင်းကွင်းဆက်၏ ပထမရုပ်ပိုင်းစာကြောင်း သို့မဟုတ် ယခုစာကြောင်း၏ နံပါတ်။ |
| `Command` | စာကြောင်းအဆုံး ဖယ်ရှားထားသော ကုဒ်ဖော်ပြီး ရုပ်ပိုင်းစာကြောင်း။ ဖော်ပြထားသော ကန့်သတ်ချက် သက်ရောက်သည်။ |
| `FileLastWriteTime` | မူရင်းဖိုင်ပြင်ဆင်ချိန်ကို record တိုင်းတွင် ထပ်ဖော်ပြသည်။ **ဖိုင် metadata သာဖြစ်ပြီး command လုပ်ဆောင်ချိန် လုံးဝမဟုတ်ပါ။** Image ခွဲခြမ်းစိတ်ဖြာရာတွင် staging အချိန်မဟုတ်ဘဲ မူရင်း metadata ကို အသုံးပြုသည်။ |
| `FileSize` | မူရင်းဖိုင်အရွယ်အစား၊ byte ဖြင့်။ |
| `Encoding` | `UTF-8`၊ `UTF-8-BOM`၊ `UTF-16LE` သို့မဟုတ် `UTF-16BE`။ |
| `Problems` | စာကြောင်းပျက်စီးမှုနှင့် ဖြတ်တောက်မှု စစ်ဆေးချက်များ။ CSV တွင် semicolon ဖြင့်ခွဲပြီး JSON/JSONL တွင် array ဖြစ်သည်။ ဖိုင်အဆင့်စစ်ဆေးချက်များကို log တွင်လည်း ဖော်ပြသည်။ |

JSON နှင့် JSONL တွင် တူညီသော field များ ပါရှိပြီး ဂဏန်းများကို ၎င်းတို့၏ type အတိုင်း ဖော်ပြကာ မသိသော value များအတွက် `null` ကို သုံးပြီး `Problems` ကို list အဖြစ် ထားသည်။ ပုံစံတိုင်းတွင် command စာသားကို သိမ်းထားသည့်အတိုင်း ထိန်းသိမ်းသည်။

`FileLastWriteTime` သည် မူလအားဖြင့် ISO 8601 ပုံစံ UTC ကို အသုံးပြုသည်၊ ဥပမာ `2026-10-20T08:46:38.5Z`။ ပြသသည့်အချိန်ကွာဟချက်ကို ပြောင်းရန် `-Z, --timezone` ကို `auto-parse` သာ လက်ခံသည်။ သီးခြား `parse-file powershell-history` command တွင် အချိန်ဇုန်ရွေးချယ်စရာ မရှိပါ။

## အလိုအလျောက် ခွဲခြမ်းစိတ်ဖြာခြင်း { #automatic-parsing }

`auto-parse` သည် `ShellHistory/PowerShellHistory.csv` (`-t json` သို့မဟုတ် `-t jsonl` ဖြင့်ဆိုလျှင် `PowerShellHistory.json` သို့မဟုတ် `PowerShellHistory.jsonl`) ကို `ShellHistory/PowerShellHistory-logs.txt` နှင့်အတူ ရေးသားသည်။ VHDX နှင့် E01 image များမှဆိုလျှင် မှတ်တမ်းဖိုင်များကို ကူးယူပြီး `FileLastWriteTime` သည် image တွင် မှတ်တမ်းတင်ထားသော အချိန် ဖြစ်သည်။ ပေါင်းစပ်ထားသော `-j` timeline တွင် row တိုင်းကို `Artifact` identifier `powershell-history` ဖြင့် ထည့်သွင်းသည်။ စာကြောင်းများတွင် အချိန် မပါသောကြောင့် [`super-timeline`](../commands/super-timeline.md) သည် ၎င်းတို့မှ event မပြုလုပ်ဘဲ အချိန်မပါသော record များအဖြစ် ရေတွက်သည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. `Windows\System32\config\systemprofile` နှင့် `Windows\ServiceProfiles` အောက်ရှိ profile များအပါအဝင် profile တိုင်း၏ မှတ်တမ်းကို ဖတ်ပါ။ ၎င်းတို့တွင် ထို account များအဖြစ် လုပ်ဆောင်နေသော shell များ၌ ရိုက်ထည့်ခဲ့သည့် command များ ပါရှိသည်။
2. စာကြောင်းများစွာပါသော entry များကို အတူတကွ ဖတ်ပြီး download များ၊ encode လုပ်ထားသော command များ၊ security setting ပြောင်းလဲမှုများ၊ account နှင့် credential ဆိုင်ရာ command များနှင့် remote ချိတ်ဆက်မှုများကို ရှာပါ။
3. PowerShell event log များ (script block logging၊ event 4104)၊ process ဖန်တီးမှု event များနှင့် `powershell.exe` နှင့် `pwsh.exe` ၏ Prefetch ဖိုင်များဖြင့် တိုက်ဆိုင်စစ်ဆေးပါ။
4. နောက်ဆုံးစာကြောင်းများ ထည့်ခဲ့သည့်အချိန်ကို ကန့်သတ်ရန် `FileLastWriteTime` ကို မှတ်တမ်းဖိုင်၏ `$MFT` နှင့် USN ဂျာနယ် အချိန်များ၊ logon များနှင့် နှိုင်းယှဉ်ပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft — about_PSReadLine: command history](https://learn.microsoft.com/en-us/powershell/module/psreadline/about/about_psreadline)
- [Microsoft — Set-PSReadLineOption: HistorySavePath and HistorySaveStyle](https://learn.microsoft.com/en-us/powershell/module/psreadline/set-psreadlineoption)
- [Microsoft — about_Logging_Windows: script block logging](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [PSReadLine — how the history file is saved and read](https://github.com/PowerShell/PSReadLine/blob/master/PSReadLine/History.cs)
