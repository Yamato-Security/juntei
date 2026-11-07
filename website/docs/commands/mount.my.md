# Disk image များကို mount လုပ်ခြင်း { #mounting-disk-images }

`juntei mount` သည် VHDX disk image အတွင်းရှိ ဖိုင်များကို ၎င်း run နေသရွေ့ ဖိုင်တွဲတစ်ခုတွင် read-only အဖြစ်
ဖတ်နိုင်အောင် ပြုလုပ်ပေးပါသည်။ ထို့နောက် KAPE ၏ `--vhdx` collection များနှင့် Hyper-V disk များကို Hayabusa
သို့မဟုတ် Juntei ၏ ကိုယ်ပိုင် ညွှန်ကြားချက်များကဲ့သို့ မည်သည့် tool ဖြင့်မဆို scan လုပ်နိုင်ပါသည်။ မည်သည့်အရာကိုမျှ
ထုတ်ယူရန် မလိုသလို root လည်း မလိုဘဲ operating system က VHDX သို့မဟုတ် NTFS ကို ပံ့ပိုးရန်လည်း မလိုပါ။ ၎င်းသည်
FUSE ကို အသုံးပြုပါသည်- Linux သို့မဟုတ် macFUSE ပါသော macOS။

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

- Mount point တွင် NTFS volume ၏ ဖိုင်များ- KAPE collection အတွက်ဆိုလျှင် စုဆောင်းထားသော ဖိုင်များ ပါဝင်သည့်
  `C` ဖိုင်တွဲ၊ KAPE ၏ `LongFileNames` ဖိုင်တွဲနှင့် `$MFT` ကဲ့သို့သော volume ၏ ကိုယ်ပိုင် metadata ဖိုင်များ။
  NTFS volume များစွာ ပါဝင်သော image သည် ၎င်းတို့ကို `Volume1`၊ `Volume2` စသည်ဖြင့် ပြသပါသည်။
- ဖိုင်တိုင်းသည် read-only ဖြစ်ပြီး mount point ၏ ပိုင်ရှင်က ပိုင်ဆိုင်ပါသည်။ ဖိုင်အချိန်များသည် NTFS ၏
  `$STANDARD_INFORMATION` အချိန်များ ဖြစ်ပါသည်။
- အမည်များကို NTFS က သိမ်းဆည်းထားသည့်အတိုင်း ဖော်ပြပြီး အတိအကျ ကိုက်ညီမှသာ ရှာဖွေတွေ့ရှိပါသည်။ DOS 8.3 အမည်များနှင့်
  alternate data stream များကို မပြသပါ။
- Compress လုပ်ထားသော ဖိုင်များကို decompress လုပ်ပေးပါသည် (ဖိုင်တစ်ခုလျှင် 1 GiB အထိ)။ EFS ဖြင့် encrypt
  လုပ်ထားသော ဖိုင်များကို မဖတ်နိုင်ပါ။

## အထောက်အထားကို မပြောင်းလဲပါ { #the-evidence-is-not-changed }

Image ကို read-only အဖြစ် ဖွင့်ပြီး mount သည် မည်သည့် ရေးသားမှုကိုမျှ လက်မခံပါ။ ပုံမှန်အတိုင်း ပိတ်ခြင်း မပြုခဲ့သော VHDX
ကို ၎င်း၏ ဆိုင်းငံ့ထားသော log ကို memory အတွင်း၌ အသုံးချ၍ ဖတ်ပြီး ဖိုင်ထဲသို့ ဘယ်တော့မှ ပြန်မရေးပါ။ KAPE သည်
၎င်း၏ VHDX ဖိုင်များကို DiscUtils ဖြင့် ရေးသားပြီး DiscUtils က NTFS boot sector တွင် jump instruction နှင့်
`55 AA` signature မပါဘဲ ချန်ထားခဲ့ပါသည်။ Juntei သည် ထို volume များကို ရှိသည့်အတိုင်း ဖတ်ပါသည်။

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
- KAPE သည် Volume Shadow Copies မှ ကူးယူသော ဖိုင်များကို `VSS32` ကဲ့သို့သော ဖိုင်တွဲများတွင် သိမ်းဆည်းပါသည်။
  ၎င်းတို့တစ်ခုစီကို အခြားစနစ်တစ်ခုအဖြစ် ရှာဖွေတွေ့ရှိပြီး parse လုပ်ပါသည် (`HOST1_2`, …)။
- Mount မလုပ်နိုင်သော image ကို report လုပ်ပါသည်။ ကျန် image များကို ဆက်လက် parse လုပ်ပြီး run သည်
  exit status 1 ဖြင့် ပြီးဆုံးပါသည်။

## လိုအပ်ချက်များ { #requirements }

- Linux- FUSE 3 (`fusermount3`၊ `fuse3` package) နှင့် `/dev/fuse` သို့ ဝင်ရောက်ခွင့်။ ပုံမှန်အသုံးပြုသူများတွင်
  ထိုခွင့် အများအားဖြင့် ရှိပါသည်။ Root မလိုအပ်ပါ။
- macOS- macFUSE။
- Windows တွင် FUSE မရှိပါ။ ယင်းအစား administrator အခွင့်အရေးဖြင့် run ထားသော (elevated) PowerShell တွင် image ကို
  read-only အဖြစ် attach လုပ်ပြီး ၎င်း၏ drive letter ကို tool များအား ပေးပါ-

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
