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

## auto-parse ဖြင့် image တိုင်းကို parse လုပ်ခြင်း { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` သည် အထောက်အထား ဖိုင်တွဲရှိ VHDX disk image တိုင်း (သို့မဟုတ် `-d` ဖြင့် ပေးထားသော image
တစ်ခုတည်း) အတွင်းရှိ Windows စနစ်များကို သီးခြား mount လုပ်စရာမလိုဘဲ parse လုပ်ပါသည်-

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- Image တစ်ခုစီကို ယာယီ ဖိုင်တွဲအသစ်တစ်ခုအတွင်း image ၏ အမည်ဖြင့် အမည်ပေးထားသော ဖိုင်တွဲ (`host1-triage`)
  တွင် read-only အဖြစ် mount လုပ်ပြီး ထိုနေရာတွင် တွေ့ရှိသော စနစ်များကို disk ပေါ်ရှိ collection တစ်ခုကဲ့သို့ parse
  လုပ်ပါသည်။ Image များကို အဆုံးတွင်လည်းကောင်း၊ Ctrl+C နှိပ်သည့်အခါတွင်လည်းကောင်း unmount လုပ်ပါသည်။
- Terminal သည် image များနှင့် စနစ်တစ်ခုစီ၏ image ကို ဖော်ပြပါသည်-
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`။
- Manifest တွင် `EvidenceDirectory` သည် ပေးထားသော ဖိုင်တွဲ ဖြစ်ပြီး စနစ်တစ်ခုစီ၏ `Image` သည် ၎င်း၏ image ကို
  ဖော်ပြပါသည်။ `EvidenceRoot` နှင့် parser ညွှန်ကြားချက်များသည် mount ဖိုင်တွဲ (`./host1-triage/C`) ကို
  အသုံးပြုပါသည်။
- Image တွင် Volume Shadow Copies မှ ဖိုင်များကို `VSS32` ကဲ့သို့ folder များ၌ သိမ်းထားပါက folder တစ်ခုစီကို နောက်ထပ် system (`HOST1_2`၊ …) အဖြစ် ရှာဖွေ၍ parse လုပ်သည်။
- Mount မလုပ်နိုင်သော image ကို report လုပ်ပါသည်။ ကျန် image များကို ဆက်လက် parse လုပ်ပြီး run သည်
  exit status 1 ဖြင့် ပြီးဆုံးပါသည်။

NTFS volume တစ်ခုကို မဖွင့်နိုင်ပါက Juntei က ပြဿနာကို ဖော်ပြပြီး ဖတ်နိုင်သော volume များကို ဆက်လက်သုံးသည်။ Image ကို တစ်စိတ်တစ်ပိုင်းသာ ဖတ်နိုင်ပါက `auto-parse --vhdx` သည် exit status 1 ကို ပြန်ပေးပြီး `mount` သည်လည်း unmount ပြီးနောက် 1 ကို ပြန်ပေးသည်။

## လိုအပ်ချက်များ { #requirements }

- Linux- FUSE 3 (`fusermount3`၊ `fuse3` package) နှင့် `/dev/fuse` သို့ ဝင်ရောက်ခွင့်။ ပုံမှန်အသုံးပြုသူများတွင်
  ထိုခွင့် အများအားဖြင့် ရှိပါသည်။ Root မလိုအပ်ပါ။
- Windows နှင့် macOS ကို မပံ့ပိုးပါ။ Windows တွင် ယင်းအစား administrator အခွင့်အရေးဖြင့် run ထားသော (elevated) PowerShell တွင် image ကို
  read-only အဖြစ် attach လုပ်ပြီး ၎င်း၏ drive letter ကို tool များအား ပေးပါ-

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
