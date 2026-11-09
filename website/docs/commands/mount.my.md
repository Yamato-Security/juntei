# Disk image များကို mount လုပ်ခြင်း { #mounting-disk-images }

`juntei mount` သည် လုပ်ဆောင်နေစဉ် VHDX disk image အတွင်းမှ ဖိုင်များကို directory တစ်ခုမှ read-only ဖတ်ရှုနိုင်စေသည်။ Hayabusa နှင့် Juntei ကဲ့သို့ tool များဖြင့် ဖိုင်ထုတ်ယူခြင်း သို့မဟုတ် root အခွင့်အရေးမလိုဘဲ စစ်ဆေးနိုင်သည်။ FUSE ကို အသုံးပြုပြီး Linux ပေါ်တွင် အလုပ်လုပ်သည်။ Reader နှင့် mounting library ကို [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) တွင် ထိန်းသိမ်းထားသည်။

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

အခြား terminal တစ်ခုတွင်-

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

Unmount လုပ်ရန် ပထမ terminal တွင် Ctrl+C ကို နှိပ်ပါ၊ သို့မဟုတ် `fusermount3 -u /mnt/host1` ကို run ပါ။

## Mount က ပြသသည့်အရာများ { #what-the-mount-shows }

- Mount point တွင် NTFS volume ၏ ဖိုင်များကို တွေ့ရသည်။ စုဆောင်းထားသော image တွင် `C`၊ `LongFileNames` ကဲ့သို့ directory များနှင့် `$MFT` ကဲ့သို့ volume metadata ပါဝင်နိုင်သည်။ NTFS volume များစွာရှိလျှင် `Volume1`၊ `Volume2` အဖြစ် ဖော်ပြသည်။
- ဖိုင်တိုင်းသည် read-only ဖြစ်ပြီး mount point ၏ ပိုင်ရှင်က ပိုင်ဆိုင်ပါသည်။ ဖိုင်အချိန်များသည် NTFS ၏
  `$STANDARD_INFORMATION` အချိန်များ ဖြစ်ပါသည်။
- အမည်များကို NTFS က သိမ်းဆည်းထားသည့်အတိုင်း ဖော်ပြပြီး အတိအကျ ကိုက်ညီမှသာ ရှာဖွေတွေ့ရှိပါသည်။ DOS 8.3 အမည်များနှင့်
  alternate data stream များကို မပြသပါ။
- Compress လုပ်ထားသော ဖိုင်များကို decompress လုပ်ပေးပါသည် (ဖိုင်တစ်ခုလျှင် 1 GiB အထိ)။ EFS ဖြင့် encrypt
  လုပ်ထားသော ဖိုင်များကို မဖတ်နိုင်ပါ။

## အထောက်အထားကို မပြောင်းလဲပါ { #the-evidence-is-not-changed }

Image ကို read-only ဖွင့်ပြီး mount မှ ရေးသားခြင်းကို ငြင်းပယ်သည်။ သေချာမပိတ်ခဲ့သော VHDX ၏ pending log ကို memory တွင်သာ အသုံးချပြီး image သို့ ပြန်မရေးပါ။ အချို့ DiscUtils writer များသည် NTFS boot sector ၏ jump instruction နှင့် `55 AA` signature ကို သုညထားသည်။ Juntei သည် ထို volume များကို မူလအတိုင်း ဖတ်နိုင်သည်။

## အသုံးပြုပုံ { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## အင်မေ့ဂျ်များကို မူလစနစ်ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` သည် **Linux၊ macOS နှင့် Windows** တွင် VHDX ကို တိုက်ရိုက်ဖတ်သည်။ OS mount၊ FUSE သို့မဟုတ် administrator အခွင့်အရေး မလိုအပ်ပါ။

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Parser များအတွက် ဖိုင်လမ်းကြောင်းများ လိုအပ်သဖြင့် NTFS reader မှ ရရှိသော ဖိုင်များကို သီးသန့်ယာယီ directory သို့ ကူးယူသည်။ **ထိုဖိုင်များအားလုံးအတွက် ယာယီ disk နေရာ လုံလောက်စွာ ရှိရမည်။** မူရင်း image ကို read-only အဖြစ် ထားပြီး ပုံမှန်ပြီးဆုံးပါက ယာယီမိတ္တူများကို ဖယ်ရှားသည်။

ဖတ်ရှုမှုအမှားများနှင့် မလုံခြုံသောအမည်များကို `ImageErrors` နှင့် `manifest.csv` ၏ failed row များတွင် မှတ်တမ်းတင်သည်။ ဖတ်နိုင်သောရလဒ်များကို ဆက်လက်သိမ်းဆည်းပြီး input မပြည့်စုံပါက exit code 1 ပြန်ပေးသည်။

အခြား program များမှ ဖိုင်များကို ဖတ်နိုင်ရန် အသုံးပြုသော သီးခြား `mount` command သည် Linux၊ FUSE 3 နှင့် `/dev/fuse` access လိုအပ်ဆဲ ဖြစ်သည်။

[English: implementation details and limitations](https://yamato-security.github.io/juntei/commands/auto-parse/#vhdx-disk-images).
