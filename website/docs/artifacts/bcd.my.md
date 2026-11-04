# Boot ပြင်ဆင်မှု ဒေတာ (BCD) { #boot-configuration-data-bcd }

## BCD store ဆိုသည်မှာ { #what-the-bcd-store-is }

Windows Vista နှင့် နောက်ပိုင်းဗားရှင်းများသည် ၎င်းတို့၏ boot ပြင်ဆင်မှုကို registry hive တစ်ခုဖြစ်သော BCD store
တွင် သိမ်းဆည်းပါသည်။ BIOS စနစ်များတွင် `\Boot\BCD`၊ UEFI စနစ်များ၏ EFI system partition တွင်
`\EFI\Microsoft\Boot\BCD` ဖြစ်ပါသည်။ `bcdedit` နှင့် Windows installer က ၎င်းကို ပြင်ဆင်ပြီး boot manager က
boot တက်တိုင်း ၎င်းကို ဖတ်ပါသည်။

Store တွင် object များ ပါဝင်ပြီး တစ်ခုချင်းစီကို GUID တစ်ခုဖြင့် အမည်ပေးထားပါသည်။

- **Windows boot manager** (`{bootmgr}`) — မူလ entry၊ ပြသမည့် အစီအစဉ်နှင့် menu timeout ပါဝင်ပြီး UEFI စနစ်များတွင်
  firmware ၏ boot အစီအစဉ်ပါဝင်သော **firmware boot manager** (`{fwbootmgr}`) လည်း ရှိပါသည်။
- Windows ထည့်သွင်းမှု တစ်ခုစီ (recovery environment အပါအဝင်) အတွက် **Windows boot loader** တစ်ခု၊
  **resume application** (hibernation) နှင့် **memory tester**၊
- အခြား object များက ဆက်ခံယူသော ဆက်တင်များပါဝင်သည့် **inherited settings** object များ (`{globalsettings}`,
  `{bootloadersettings}`, …)၊
- **device** object များ (recovery environment ၏ ramdisk ရွေးချယ်စရာများ)။

Object တစ်ခုစီတွင် element များ ရှိပါသည်။ ဖော်ပြချက်၊ device နှင့် path နှင့်
`recoveryenabled`၊ `bootstatuspolicy`၊ `testsigning`၊ `nointegritychecks` နှင့် `safeboot` ကဲ့သို့သော ဆက်တင်များ ဖြစ်ပါသည်။

တိုက်ခိုက်မှု နှစ်မျိုးက ၎င်းတို့ကို ပြောင်းလဲပါသည်။

- **Ransomware** သည် စနစ်ကို ပြုပြင်၍ မရစေရန် recovery ကို ပိတ်ပါသည်။
  `bcdedit /set {default} recoveryenabled No` နှင့်
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures` ဖြစ်ပါသည်။ အချို့ မျိုးကွဲများသည် လုံခြုံရေးထုတ်ကုန်များ
  မလည်ပတ်သော safe mode ဖြင့် Windows ပြန်စတင်စေရန် `safeboot` ကို သတ်မှတ်ပါသည်။
- **Rootkit များနှင့် လက်မှတ်မထိုးထားသော driver များ** သည် boot ချိန် စစ်ဆေးမှုများကို အားနည်းစေရန်
  `testsigning on` သို့မဟုတ် `nointegritychecks on` ကို အသုံးပြုနိုင်ပါသည်။ ဤဆက်တင်များ တစ်ခုတည်းဖြင့် rootkit
  လည်ပတ်ခဲ့ကြောင်း မသက်သေပြနိုင်ပါ။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> စုဆောင်းသည့်အချိန်တွင် ဤ boot entry ၌
ဤဆက်တင် ရှိခဲ့ပါသည်။ Element ၏ key ၏ နောက်ဆုံးရေးသားချိန်သည် ဆက်တင်ကို နောက်ဆုံးရေးသားခဲ့သည့် အချိန် ဖြစ်ပါသည်။

## တည်နေရာ { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

Element ၏ type သည် hexadecimal ဂဏန်း 8 လုံး ဖြစ်ပါသည်။ ၎င်း၏ class (`1` application အားလုံး၊ `2` application
အမျိုးအစား တစ်ခု၊ `3` device များ)၊ ၎င်း၏ format (`1` device၊ `2` string၊ `3` object၊ `4` object list၊ `5`
integer၊ `6` boolean၊ `7` integer list) နှင့် ၎င်း၏ နံပါတ် ဖြစ်ပါသည်။ ညွှန်ကြားချက်သည် element များကို Microsoft
စာတမ်းတွင် ဖော်ပြထားသည့်အတိုင်း အမည်ပေးပါသည် (`BcdLibraryBoolean_AutoRecoveryEnabled`)။

စုဆောင်းရေး tool များသည် store ကို မကြာခဏ မကူးယူပါ။ EFI system partition ကို mount မလုပ်ထားဘဲ `\Boot`
သည်လည်း ဝှက်ထားသောကြောင့် ဖြစ်ပါသည်။ ၎င်းကို တိကျစွာ စုဆောင်းပါ သို့မဟုတ် partition ကို mount လုပ်ပါ။ `-d` သည်
`BCD-Template` ကိုလည်း ဖတ်ပါသည် (မည်သည့်တစ်ခုဖြစ်ကြောင်း `HiveType` က ဖော်ပြပါသည်)။ Template သည် Windows စတင်
ပြုလုပ်သည့် အခြေခံဖြစ်ပြီး ၎င်း boot တက်သည့် store မဟုတ်ပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **ဆက်တင်များကို ဆက်ခံယူပါသည်။** Boot loader တစ်ခုသည် ၎င်း၏
  `BcdLibraryObjectList_InheritedObjects` ရှိ object များမှ ဆက်ခံယူပါသည် (များသောအားဖြင့် `{bootloadersettings}`
  ဖြစ်ပြီး ၎င်းက `{globalsettings}` ကို ဆက်ခံယူပါသည်)။ `RecoveryEnabled`၊ `TestSigning`၊ `NoIntegrityChecks`၊ `SafeBoot` နှင့်
  `BootStatusPolicy` တို့သည် ဖြေရှင်းပြီးသော သိမ်းဆည်းထားသည့် တန်ဖိုးများ (ကိုယ်ပိုင်၊ မရှိလျှင် ဆက်ခံယူထားသည့်) ဖြစ်ပြီး
  မည်သည့်တန်ဖိုးများ အခြား object မှ လာသည်ကို `Inherited` က ဖော်ပြပါသည်။ ဆက်ခံယူမှုကို အဆင့် ရှစ်ဆင့်အထိသာ လိုက်ပြီး
  cycle များကို ကန့်သတ်ထားပါသည်။ ကိုးကားထားသော object သည် မပြည့်စုံသော store တွင် မရှိနိုင်ပါ။ Field များ ဗလာဖြစ်ခြင်းက
  Windows ၏ ထိရောက်သော မူလတန်ဖိုးကို အတည်မပြုပါ။ Firmware နှင့် Secure Boot တို့ကလည်း သိမ်းဆည်းထားသော ရွေးချယ်မှု
  သက်ရောက်ခြင်းကို တားဆီးနိုင်ပါသည်။
- **သတ်မှတ်မထားခြင်းသည် ပိတ်ထားခြင်း မဟုတ်ပါ။** `RecoveryEnabled` ဗလာဖြစ်ခြင်းသည် element မရှိကြောင်း
  ဆိုလိုပြီး Windows ၏ မူလတန်ဖိုး သက်ရောက်ပါသည်။
- **အချိန်များသည် element တစ်ခုချင်းစီအလိုက် ဖြစ်ပါသည်။** Element တစ်ခုစီ၏ key အချိန် (`Elements` ရှိ
  `key_last_written`) သည် ၎င်းကို နောက်ဆုံးရေးသားခဲ့သည့် အချိန် ဖြစ်ပါသည်။ Windows သည် update နှင့် upgrade များတွင်
  element အချို့ကို ပြန်ရေးပါသည်။ အခြား element များ၏ အချိန်များနှင့် နှိုင်းယှဉ်ပါ။
- **Device များကို တစ်စိတ်တစ်ပိုင်းသာ decode လုပ်ပါသည်။** GPT partition ကို ၎င်း၏ partition နှင့် disk ၏ GUID များဖြင့်
  ပေးပြီး ramdisk ကို ၎င်း၏ options object အဖြစ် ပေးပါသည်။ အခြား device များသည် ၎င်းတို့၏ byte များကိုသာ ထိန်းထားပါသည် (`data_raw`)။
- **စာတမ်းတွင် မဖော်ပြထားသော element များ** (`0x15000065`၊ `0x4…` class) တွင် အမည်မရှိဘဲ ၎င်းတို့၏ byte များကို ထိန်းထားပါသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

အနှစ်ချုပ်တွင် store တစ်ခုစီ၏ မူလ entry နှင့် test signing၊ integrity check၊ recovery၊ boot status policy
(မည်သည့် `Ignore…` မဆို) နှင့် safe mode ဆက်တင် တစ်ခုစီကို ၎င်းသတ်မှတ်ခဲ့သည့် အချိန် သို့မဟုတ် ၎င်းဆက်ခံယူခဲ့သည့်
object နှင့်အတူ ဖော်ပြပါသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host၊ hive နှင့် `BCD` သို့မဟုတ် `BCD-Template`။ |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | Object ၏ GUID၊ ၎င်း၏ type (`0x10200003`) နှင့် ၎င်းသည် မည်သည့်အရာ ဖြစ်သည် (`Windows boot loader`, `inherited settings for Windows boot loader`, …)။ |
| `Description`, `Path`, `Device` | ဖော်ပြချက်၊ application ၏ path နှင့် ၎င်း၏ device (GPT partition နှင့် disk သို့မဟုတ် ramdisk ၏ options object)။ |
| `IsDefault`, `DisplayOrder` | Windows boot manager က ၎င်းကို မူလအားဖြင့် boot လုပ်/မလုပ်နှင့် boot menu တွင် ၎င်း၏ နေရာ။ |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | Store မှ ဖြေရှင်းထားသည့် `recoveryenabled`၊ `testsigning`၊ `nointegritychecks`။ |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`, `Network`, `DsRepair`) နှင့် `bootstatuspolicy` (`IgnoreAllFailures`, …)၊ နံပါတ်နှင့်အတူ။ |
| `Inherited` | ဤတန်ဖိုးများထဲမှ မည်သည်တို့က အခြား object မှ လာသည်နှင့် မည်သည့် object မှ လာသည်။ |
| `Elements` | Element တိုင်း — ၎င်း၏ type၊ အမည်၊ တန်ဖိုး၊ byte များ (device များ၊ စာတမ်းမဖော်ပြထားသော type များနှင့် decode မလုပ်နိုင်သော ဒေတာအတွက်) နှင့် key အချိန်။ |
| `KeyLastWrittenUTC` | Object ၏ key ၏ နောက်ဆုံးရေးသားချိန်။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု context။ |
| `Problems` | Type မရှိသော object များ၊ ဒေတာက ၎င်းတို့၏ format နှင့် မကိုက်ညီသော element များ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. မူလ entry နှင့် boot loader တိုင်းတွင် `TestSigning`၊ `NoIntegrityChecks`၊
   `RecoveryEnabled = false`၊ `Ignore…` `BootStatusPolicy` နှင့် `SafeBoot` ရှိ/မရှိ စစ်ဆေးပါ။
2. ပြောင်းလဲမှု၏ ရက်စွဲကို သတ်မှတ်ရန် element အချိန်များကို အသုံးပြုပြီး Prefetch၊ Amcache၊ process creation event များ
   (4688, Sysmon 1) နှင့် System event log ရှိ `bcdedit.exe` နှင့် ဆက်စပ်စစ်ဆေးပါ။
3. ရှိမနေသင့်သော boot entry များ သို့မဟုတ် device များ (အခြား loader path၊ ramdisk) နှင့်
   ပြောင်းလဲထားသော မူလ entry သို့မဟုတ် ပြသမည့် အစီအစဉ်ကို ရှာဖွေပါ။
4. Store ကို `BCD-Template` နှင့် build တူသော အခြားစနစ်တစ်ခုနှင့် နှိုင်းယှဉ်ပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: BCD boot ရွေးချယ်စရာများ ကိုးကားချက်](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (object type များ)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
