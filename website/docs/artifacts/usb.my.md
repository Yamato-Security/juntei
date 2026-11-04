# USB စက်ပစ္စည်းများ { #usb-devices }

## USB မှတ်တမ်း ဆိုသည်မှာ { #what-the-usb-history-is }

Windows သည် ၎င်းထည့်သွင်းသော device တိုင်းကို မှတ်တမ်းတင်သည်။ USB storage၊ အခြား USB device များနှင့် portable device
များ (ဖုန်းများ၊ ကင်မရာများ၊ media player များ) အတွက် `SYSTEM` hive သည် device တစ်ခုလျှင် key တစ်ခုကို ၎င်း၏ အမည်၊
serial number နှင့် Windows က ၎င်းကို ထည့်သွင်းခဲ့ချိန်၊ ချိတ်ဆက်လာသည်ကို နောက်ဆုံးမြင်ခဲ့ချိန်နှင့် ဖြုတ်လိုက်သည်ကို
နောက်ဆုံးမြင်ခဲ့ချိန်တို့နှင့်အတူ ထိန်းသိမ်းထားသည်။ `MountedDevices` သည် ၎င်းတို့၏ volume များကို ပေးထားသော volume
GUID များနှင့် drive letter များကို မှတ်တမ်းတင်သည်။ `SOFTWARE` hive သည် Explorer က portable device များနှင့် volume
များကို ပေးခဲ့သော အမည်များနှင့် ReadyBoost ၏ volume serial number မှတ်တမ်းကို ထိန်းသိမ်းထားသည်။ အသုံးပြုသူတစ်ဦးစီ၏
`NTUSER.DAT` သည် ထိုအသုံးပြုသူ mount လုပ်ထားခဲ့သော volume များကို မှတ်တမ်းတင်သည်။

`parse-registry usb` သည် ၎င်းတို့ကို device နှင့် volume တစ်ခုလျှင် မှတ်တမ်းတစ်ခုအဖြစ် ပေါင်းစပ်သည်- မည်သည့် device
ဖြစ်ခဲ့သည်၊ မည်သည့်အချိန်တွင် ထည့်သွင်းခဲ့ပြီး ချိတ်ဆက်ခဲ့သည်၊ မည်သည့် drive letter နှင့် volume ရှိခဲ့သည်နှင့် မည်သည့်
အသုံးပြုသူများက ထို volume ကို mount လုပ်ထားခဲ့သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ဤ vendor၊ product နှင့် serial number ပါသော
device တစ်ခုကို ဤစနစ်သို့ ပထမဆုံးနှင့် နောက်ဆုံး ဤအချိန်များတွင် ချိတ်ဆက်ခဲ့သည်။ ၎င်း၏ volume တွင် ဤ drive letter
ရှိခဲ့သည်။ ဤအသုံးပြုသူများက ၎င်းကို mount လုပ်ထားခဲ့သည်။ Registry မှတ်တမ်းများသည် မည်သည့်ဖိုင်များကို ကူးယူခဲ့သည်ကို မပြသပါ။

## တည်နေရာ { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| Device property | Field | အဓိပ္ပာယ် (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`- ၎င်း၏ driver ကို နောက်ဆုံး ထည့်သွင်းခဲ့ချိန် |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`- device ကို ပထမဆုံး ထည့်သွင်းခဲ့ချိန် |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`- ၎င်းကို နောက်ဆုံး ချိတ်ဆက်ခဲ့ချိန် (Windows 8 နှင့် နောက်ပိုင်း) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`- ၎င်းကို နောက်ဆုံး ဖြုတ်ခဲ့ချိန် (Windows 8 နှင့် နောက်ပိုင်း) |

Windows 8 နှင့် နောက်ပိုင်းသည် အချိန်တစ်ခုစီကို `0064` … `0067` key ၏ ပုံသေတန်ဖိုးအဖြစ် ထိန်းသိမ်းထားသည်။ Windows 7
သည် ၎င်းကို `00000064\00000000\Data` တွင် ထိန်းသိမ်းထားသည်။ Juntei သည် နှစ်မျိုးစလုံးကို ဖတ်သည်။

DFIR ကိုးကားချက် အများအပြားသည် `0064` ကို ပထမဆုံး ထည့်သွင်းမှုဟုလည်းကောင်း၊ `0065` ကို ထည့်သွင်းမှုဟုလည်းကောင်း
အမည်တပ်ကြသည်။ Juntei သည် အထက်ပါ Microsoft ၏ `devpkey.h` အဓိပ္ပာယ်သတ်မှတ်ချက်များကို လိုက်နာပြီး column များကို
အချိန်တစ်ခုစီက မှတ်တမ်းတင်သည့်အရာအလိုက် အမည်ပေးထားသည်။

`-f` သို့မဟုတ် `-d` ကို `SYSTEM` hive ပေးပါ။ စနစ်၏ `SOFTWARE` hive နှင့် ၎င်း၏ အသုံးပြုသူများ၏ `NTUSER.DAT` hive
များကို စုဆောင်းမှုတစ်ခုက ထိန်းသိမ်းထားသည့်အတိုင်း (`Windows\System32\config\SOFTWARE`,
`Users\<user>\NTUSER.DAT`) ၎င်း၏ဘေးတွင်၊ သို့မဟုတ် hive များကို အတူတကွ export လုပ်ထားသည့်အခါ တူညီသော directory
နှင့် တစ်ဆင့်အောက်တွင် ရှာဖွေသည်။ ၎င်းတို့အားလုံးကို ၎င်းတို့၏ transaction log များနှင့်အတူ စုဆောင်းပါ။

## မှတ်တမ်းများကို ပေါင်းစပ်ပုံ { #how-the-records-are-joined }

- **Device များ-** device key တစ်ခုလျှင် မှတ်တမ်းတစ်ခု- `usbstor`, `usb` နှင့် `wpdbusenum` (`DeviceClass`)။ USB
  storage device တစ်ခုသည် ၎င်း၏ vendor နှင့် product ID များကို container ID တူသော (မဟုတ်ပါက serial number တူသော)
  USB device မှ ယူသည်။
- **Volume များ-** USB storage device ၏ volume များသည် device ကို အမည်ဖော်သော path တစ်ခုဖြစ်သည့် ဒေတာပါဝင်သော
  `MountedDevices` ၏ `\??\Volume{GUID}` တန်ဖိုးများ ဖြစ်သည်။ Volume နှစ်ခုရှိသော device တွင် မှတ်တမ်း နှစ်ခု ရှိသည်။
  ၎င်း၏ drive letter များသည် တူညီသော ဒေတာပါဝင်သော `\DosDevices\X:` တန်ဖိုးများ ဖြစ်သည်။
- **အမည်များနှင့် serial number များ-** volume အမည်များသည် `Windows Portable Devices` နှင့် တူညီသော device ကို
  အမည်ဖော်သော portable device များမှ လာသည်။ Volume label များနှင့် serial number များသည် ၎င်းကို အမည်ဖော်သော
  `EMDMgmt` key များမှ လာသည်။
- **အသုံးပြုသူများ-** ၎င်းတို့၏ `MountPoints2` တွင် volume ၏ GUID ပါဝင်သော အသုံးပြုသူများ၊ ထို key ၏ last-write
  အချိန်နှင့်အတူ။
- **Device key မရှိသော volume များ** (`volume`)- မည်သည့် device key ကမျှ အမည်မဖော်သော `MountedDevices` volume
  (internal disk ၏ သို့မဟုတ် `Enum` key မရှိတော့သော ဖယ်ရှားထားသည့် device ၏)၊ မည်သည့် volume GUID တန်ဖိုးနှင့်မျှ
  ဒေတာ မမျှဝေသော drive letter နှင့် အသုံးပြုသူများ၏ `MountPoints2` တွင်သာ ရှိသော volume။
- **`SOFTWARE` တစ်ခုတည်းက အမည်ဖော်သော device များ** (`portable_device`, `readyboost`)။

`SourceKeys` သည် မှတ်တမ်းတစ်ခုကို ပေါင်းစပ်ရာတွင် အသုံးပြုခဲ့သော key နှင့် တန်ဖိုးတိုင်းကို ၎င်း၏ hive နှင့်အတူ ဖော်ပြသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **Drive letter များကို ပြန်လည်အသုံးပြုသည်။** `MountedDevices` သည် letter တစ်ခုစီကို ပေးခဲ့သော နောက်ဆုံး device ကို
  ထိန်းသိမ်းထားသည်။ Letter တစ်ခုကို ၎င်း၏ ဒေတာက ထို letter ၏ ဒေတာ ဖြစ်နေဆဲ device ပေါ်တွင်သာ ပြသသည်။
- **Serial number များကို ဖန်တီးထားနိုင်သည်။** Instance ၏ ဒုတိယ စာလုံးသည် `&` ဖြစ်သည့်အခါ device က serial number
  မဖော်ပြသောကြောင့် Windows က ၎င်းကို ဖန်တီးထားခြင်း ဖြစ်သည်။ ထိုအခါ `Serial` သည် ဗလာ ဖြစ်သည်။ ထိုကဲ့သို့သော instance
  သည် အခြားကွန်ပျူတာပေါ်တွင် ကွဲပြားနိုင်သည်။
- **USB storage instance များသည် logical unit ဖြင့် အဆုံးသတ်သည်** (`&0`)။ `Serial` သည် ၎င်းကို ချန်လှပ်ထားပြီး
  `Instance` က ထိန်းသိမ်းထားသည်။
- **အချိန်များသည် Windows ဗားရှင်းအလိုက် ကွဲပြားသည်။** Windows 7 တွင် ထည့်သွင်းမှု အချိန်များသာ ရှိသည်။
  `LastArrivalUTC` နှင့် `LastRemovalUTC` သည် နောက်ဆုံး ချိတ်ဆက်မှုသာ ဖြစ်ပြီး ချိတ်ဆက်မှုတိုင်း မဟုတ်ပါ။ Event log များ
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`) နှင့်
  `setupapi.dev.log` က ပိုမိုပေးသည်။
- **Control set များ-** ပုံသေအားဖြင့် Windows နောက်ဆုံးအသုံးပြုခဲ့သော control set ကို ဖတ်သည်။ `--all-control-sets`
  သည် control set တိုင်းကို ဖတ်သည်။ ပိုဟောင်းသော control set တစ်ခုတွင် လက်ရှိ control set တွင် မရှိတော့သော device
  တစ်ခုကို ဖော်ပြထားဆဲ ဖြစ်နိုင်သည်။
- **Key ၏ last-write အချိန်သည် ချိတ်ဆက်ချိန် မဟုတ်ပါ။** Windows သည် device key များကို အကြောင်းရင်း အမျိုးမျိုးဖြင့်
  ပြန်ရေးသည်။
- **ဖယ်ရှားထားသော device များသည် ခြေရာများ ချန်ထားခဲ့သည်။** Vendor နှင့် product ပါသော်လည်း device key မရှိသော
  volume မှတ်တမ်းသည် ၎င်း၏ `Enum` key ကို (uninstall သို့မဟုတ် ရှင်းလင်းရေး tool များက) ဖယ်ရှားခဲ့သော်လည်း ၎င်း၏
  volume ကို `MountedDevices` က ထိန်းသိမ်းထားဆဲ device ဖြစ်သည်။
- **ReadyBoost serial number များ** သည် volume ၏ serial number (`VolumeSerials`၊ `dir` က ပြသသည့်အတိုင်း)
  ဖြစ်ပြီး LNK ဖိုင်များနှင့် Jump Lists ကလည်း ၎င်းကို မှတ်တမ်းတင်သည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

အနှစ်ချုပ်သည် နောက်ဆုံး ချိတ်ဆက်ခဲ့သော USB storage device များကို (အသစ်ဆုံးကို ပထမ) ၎င်းတို့၏ drive letter များနှင့်အတူ
ဖော်ပြပြီး device key မရှိသော USB storage device များနှင့် သက်ဆိုင်သည့် volume အရေအတွက်ကိုလည်း ဖော်ပြသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host` | `SYSTEM` မှ ကွန်ပျူတာ၏ အမည်။ |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum` သို့မဟုတ် device key မရှိလျှင် `volume`, `portable_device` သို့မဟုတ် `readyboost`။ |
| `ControlSet` | Device key ၏ control set။ |
| `Vendor`, `Product`, `Revision` | USB storage- device ၏ အမည် (`Disk&Ven_…&Prod_…&Rev_…`) မှ၊ volume သို့မဟုတ် `SOFTWARE` key က အမည်ဖော်သော device အတွက်လည်း။ |
| `Vid`, `PID` | USB vendor နှင့် product ID များ။ |
| `Serial`, `Instance` | Serial number နှင့် ရေးထားသည့်အတိုင်း instance key ၏ အမည်။ |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | ရေးထားသည့်အတိုင်း device key ၏ တန်ဖိုးများ (`DeviceDesc` သည် မကြာခဏ `@<inf>,%…%;<text>` ဖြစ်သည်)။ Portable device ၏ `FriendlyName` သည် ပုံမှန်အားဖြင့် ၎င်း၏ volume ၏ label သို့မဟုတ် drive ဖြစ်သည်။ |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | Device property အချိန်များ (အထက်တွင် ကြည့်ပါ)။ |
| `DriveLetters` | ၎င်းတို့၏ `MountedDevices` ဒေတာသည် volume ၏ ဒေတာဖြစ်သော drive letter များ။ |
| `VolumeGUID`, `VolumeData` | Volume ၏ GUID နှင့် ၎င်း၏ `MountedDevices` ဒေတာက ၎င်း ရှိရာဟု ဖော်ပြသည့်အရာ- device path၊ MBR disk signature နှင့် partition offset သို့မဟုတ် GPT partition။ |
| `VolumeNames`, `VolumeSerials` | Explorer က device သို့မဟုတ် ၎င်း၏ volume ကို ပေးခဲ့သော အမည်များ၊ ReadyBoost label များနှင့် volume serial number များ (`1234-ABCD`)။ |
| `User` | ၎င်းတို့၏ `MountPoints2` တွင် volume ပါဝင်သော အသုံးပြုသူများ၊ ထို key ၏ နောက်ဆုံးရေးသားချိန်နှင့်အတူ (JSON: `User`, `MountPoints2LastWritten`, `SourcePath`)။ |
| `FoundIn`, `SourceKeys` | ၎င်းကို တွေ့ရှိခဲ့သည့်နေရာ (`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`) နှင့် ၎င်းကို ပေါင်းစပ်ရာတွင် အသုံးပြုခဲ့သော key နှင့် တန်ဖိုးတိုင်း။ |
| `KeyPath`, `KeyLastWrittenUTC` | Device ၏ instance key နှင့် ၎င်း၏ နောက်ဆုံးရေးသားချိန်။ |
| `SourcePath`, `SoftwarePath` | `SYSTEM` hive နှင့် တစ်စုံတစ်ရာ ပေးခဲ့သည့်အခါ `SOFTWARE` hive။ |
| `TransactionLogStatus`, `LogsApplied` | မှတ်တမ်းကို ပေါင်းစပ်ရာတွင် အသုံးပြုခဲ့သော hive များ၏ Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | မဖတ်နိုင်သော key များနှင့် FILETIME မဟုတ်သော device property များ (၎င်းတို့၏ byte များနှင့်အတူ)။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. USB storage device များကို `LastArrivalUTC` (မဟုတ်ပါက `DriverLastInstalledUTC`) အလိုက် စီပြီး ကွန်ပျူတာ၏ ကိုယ်ပိုင်
   reader များနှင့် hub များကို ဘေးဖယ်ထားပါ။
2. Device တစ်ခုစီ၏ vendor၊ product၊ serial number၊ volume GUID၊ drive letter နှင့် volume serial number ကို မှတ်သားပြီး
   ၎င်းတို့ကို LNK ဖိုင်များ၊ Jump Lists၊ ShellBags နှင့် `$MFT` (ထို drive letter သို့မဟုတ် volume ပေါ်ရှိ path များ)
   တွင် ရှာဖွေပါ။
3. Volume ကို မည်သူ mount လုပ်ထားခဲ့သည်နှင့် မည်သည့်အချိန် (`MountPoints2` key ၏ နောက်ဆုံးရေးသားချိန်) ကို `User` တွင်
   စစ်ဆေးပါ။
4. USB storage device တစ်ခုကို အမည်ဖော်သော `volume` မှတ်တမ်းများကို ရှာဖွေပါ- ၎င်းတို့၏ key များကို ဖယ်ရှားခဲ့သော device
   များ ဖြစ်သည်။
5. နောက်ဆုံး ချိတ်ဆက်မှုသာမက ချိတ်ဆက်မှုတိုင်းအတွက် event log များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။
6. ဖျက်ပစ်ထားသော device key များအတွက် `parse-registry read --recover-deleted` ကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Device property များ (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: Mount manager တောင်းဆိုမှုများကို ပံ့ပိုးခြင်း](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (Wine ၏ header များတွင် ပါရှိသည့်အတိုင်း)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h)- device property နံပါတ်များ
- [libyal winreg-kb: USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md)၊
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) နှင့်
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

အသုံးပြုသူ ပေါင်းစပ်မှုများသည် hive တစ်ခုစီ၏ source ကိုယ်ပိုင်အမှတ်အသားကို အသုံးပြုသောကြောင့် profile အမည်တူသော export
လုပ်ထားသည့် hive များသည် သီးခြားစီ ရှိနေသည်။ `SourceKeys` သည် ပါဝင်ခဲ့သော အသုံးပြုသူ hive path များကို ထိန်းသိမ်းထားသည်။
တွဲဖက် အသုံးပြုသူ ရှာဖွေခြင်းသည် recursive hive ရှာဖွေခြင်းနှင့် ကိုက်ညီစွာ symbolic link များကို ကျော်သွားသည်။ VID/PID
ချိတ်ဆက်ခြင်းသည် ကိုက်ညီပြီး အသုံးပြုနိုင်သော container ID တစ်ခု လိုအပ်သည်။ မရှိပါက serial ကိုက်ညီမှုသည် ရှင်းလင်းသော ID
များကို ထုတ်ပေးရမည်။ ပဋိပက္ခဖြစ်နေသော ကိုယ်စားလှယ်လောင်းများသည် ပြဿနာတစ်ခုနှင့်အတူ မသိရှိသည့်အတိုင်း ရှိနေသည်။
