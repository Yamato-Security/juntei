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

`--vhdx` ဖြင့် `auto-parse` သည် [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) ကို သုံးကာ **Linux၊ macOS နှင့် Windows** တွင် VHDX ကို တိုက်ရိုက်ဖတ်သည်။ OS mount၊ FUSE သို့မဟုတ် administrator အခွင့်အရေး မလိုအပ်ပါ။

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Parser များအတွက် ဖိုင်လမ်းကြောင်း လိုအပ်သဖြင့် NTFS reader မှ ရရှိသောဖိုင်များကို သီးသန့်ယာယီနေရာသို့ ကန့်သတ် buffer ဖြင့် ကူးယူသည်။ **ရလဒ်ထွက်သော artifact များသာမက ထိုဖိုင်အားလုံးအတွက် disk နေရာ လိုအပ်သည်။** ပံ့ပိုးလျှင် သုညဧရိယာများကို sparse အဖြစ် ရေးသည်။ အမည်၊ ဖွဲ့စည်းပုံ၊ hard link နှင့် modification time များကို ထိန်းသိမ်းသည်။ LNK နှင့် Jump List source time များသည် ယာယီဖိုင်များမှ မဟုတ်ဘဲ မူရင်း NTFS metadata snapshot မှ လာသည်။ Parser error ရှိလည်း ပုံမှန်အဆုံးသတ်ပါက မိတ္တူများကို ဖယ်ရှားပြီး forced termination တွင် ကျန်နိုင်သည်။ မူရင်းကို read-only ထားပြီး pending VHDX log များကို memory ထဲတွင်သာ အသုံးချသည်။

Host ရှာဖွေမှု၊ CSV/JSON/JSONL နှင့် `-j` သည် ပုံမှန် pipeline ကို သုံးသည်။ `manifest.json` ရှိ `Image` သည် မူရင်း image ဖြစ်ပြီး `EvidenceRoot` သည် အတွင်း relative path ဖြစ်သည်။ Read error နှင့် မလုံခြုံသောအမည်များကို `ImageErrors` နှင့် `manifest.csv` failed row များတွင် မှတ်တမ်းတင်သည်။ ဖတ်နိုင်သောရလဒ်များကို ထိန်းသိမ်းပြီး input မပြည့်စုံလျှင် exit code 1 ပြန်ပေးသည်။ မလုံခြုံသော သို့မဟုတ် မရှင်းလင်းသောအမည်များကို ပြောင်းမပေးပါ။ Named NTFS stream များကို နောက်ထပ်ဖိုင်အဖြစ် မထုတ်ပေးသော်လည်း သာမန်ဖိုင်အဖြစ် စုဆောင်းထားသော artifact များကို အသုံးပြုနိုင်သည်။

အခြား program များအတွက် filesystem ဖော်ပြသော သီးခြား [`mount` command](mount.md) သည် Linux FUSE လိုအပ်ဆဲ ဖြစ်သည်။ `auto-parse --vhdx` က ၎င်းကို မသုံးပါ။

Staging အတွက် default သည် RAM ပေါ်ရှိ နေရာနည်းသော system temp directory အစား output filesystem ဖြစ်သည်။ အခြား disk ပေါ်ရှိ ရှိပြီးသား directory ကို ရွေးရန် `--temp-dir` ကို သုံးပါ။ မူရင်း metadata ကို output filesystem တွင် သီးခြားသိမ်းထားသဖြင့် staging disk ပြည့်လျှင်လည်း ဖတ်နိုင်ပြီးသောဖိုင်များကို parse လုပ်နိုင်သည်။ မိတ္တူနှင့် ရလဒ်နှစ်မျိုးလုံးအတွက် နေရာချန်ပါ။ Output volume ကိုယ်တိုင်ပြည့်လျှင် ရလဒ်များမသိမ်းနိုင်ပါ။ Ctrl+C သည် parser process များကို ရပ်ပြီး private workspace ကိုရှင်းကာ code 130 ဖြင့်ထွက်သည်။ SIGKILL သို့မဟုတ် မီးပျက်ခြင်းကြောင့် မိတ္တူများကျန်နိုင်သည်။ Image ရှာရာတွင် ဖတ်မရသော subdirectory များကို ကျော်သော်လည်း input root ဖတ်မရခြင်းသည် error ဖြစ်ဆဲဖြစ်သည်။

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## OS မှ မြင်နိုင်သော mount လိုအပ်ချက်များ { #os-visible-mount-requirements }

`juntei mount` သည် Linux FUSE 3 (`fusermount3`) နှင့် `/dev/fuse` access လိုအပ်သည်။ Windows နှင့် macOS တွင် အထက်ပါ native analysis ကို သုံးပါ။
