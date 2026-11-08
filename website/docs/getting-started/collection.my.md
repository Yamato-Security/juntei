# အထောက်အထား စုဆောင်းခြင်း { #collecting-evidence }

Juntei သည် Windows မှ စုဆောင်းထားပြီးသော ဖိုင်များကို parse လုပ်သည်။ ယုံကြည်ရသော acquisition tool ကို အသုံးပြုပြီး မူလ metadata ကို ဖြစ်နိုင်သမျှ ထိန်းသိမ်းပါ။ VHDX disk image များကို ထုတ်ယူစရာမလိုဘဲ parse လုပ်နိုင်သည်။ [Disk image များကို mount လုပ်ခြင်း](../commands/mount.md) ကို ကြည့်ပါ။

## အသုံးများသော တည်နေရာများ { #common-locations }

| Artifact | ပုံမှန် တည်နေရာ |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | NTFS volume metadata ဖိုင် `C:\$MFT` |
| `$LogFile` | NTFS volume metadata ဖိုင် `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| USN Journal | NTFS metadata stream `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| User Access Logging (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| ဝဘ် ဘရောက်ဇာများ | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\` နှင့် အခြား ဘရောက်ဇာများ၏ profile ဖိုင်တွဲများ |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` နှင့် အခြား အသုံးပြုသူ/စနစ် တည်နေရာများ |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` နှင့် `CustomDestinations` |
| ShimCache | `SYSTEM` hive: `...\Session Manager\AppCompatCache\AppCompatCache` |
| Defender quarantine | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| Machine registry | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| User registry | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## အတူတကွ ပါရမည့် ဖိုင်များ { #files-that-must-travel-together }

- Registry hive များကို ဘေးချင်းကပ်ရှိသော `.LOG`၊ `.LOG1` နှင့် `.LOG2` ဖိုင်များနှင့်အတူ စုဆောင်းပါ။ Hive သည် dirty
  ဖြစ်သည့်အခါ Juntei သည် registry ညွှန်ကြားချက်များနှင့် ShimCache အတွက် log များကို replay လုပ်နိုင်ပါသည်။
- SRUM ၏ `SRUDB.dat` ကို ကိုက်ညီမှုရှိစွာ စုဆောင်းပါ။ Juntei သည် လက်ရှိတွင် dirty ESE database ကို အစီရင်ခံသော်လည်း
  `SRU*.log` ကို replay မလုပ်ပါ။ ထို log များတွင်သာ ရှိသော record များကို မရရှိနိုင်ပါ။
- ဖြစ်နိုင်လျှင် `$MFT` ကို `$Extend/$J` ဘေးတွင် စုဆောင်းပါ။ USN record သည် path အပြည့်အစုံ မဟုတ်ဘဲ parent file
  reference များကို သိမ်းဆည်းပါသည်။ MFT သည် path ဖြေရှင်းမှုကို ပိုမိုကောင်းမွန်စေပါသည်။
- `$LogFile` ကို တူညီသော volume ၏ `$MFT` နှင့်အတူ အချိန်အနီးဆုံးတွင် စုဆောင်းပါ။ `$LogFile` သည် အလျင်အမြန်
  လည်ပတ်ထပ်ရေးသောကြောင့် နောက်ကျမှ စုဆောင်းလျှင် အစောပိုင်း redo/undo record များကို လုံးဝ ဆုံးရှုံးနိုင်ပါသည်။
- Account နှင့် Wi-Fi profile အမည်များကို ဖြေရှင်းရန် SRUM ကို ကိုက်ညီသော `SOFTWARE` hive နှင့်အတူ စုဆောင်းပါ။
- User Access Logging ၏ `Sum` ဖိုင်တွဲတစ်ခုလုံးကို စုဆောင်းပါ။ `SystemIdentity.mdb` သည် role များနှင့် နှစ်စဉ်
  `{GUID}.mdb` database များကို အမည်ဖော်ပြပါသည်။ Juntei သည် ၎င်း၏ `Svc*.log` transaction log များကိုလည်း replay မလုပ်ပါ။
- ဘရောက်ဇာ database များကို ၎င်းတို့ဘေးရှိ `-wal` ဖိုင်များ (`places.sqlite-wal`၊ `cookies.sqlite-wal`) နှင့်အတူ
  စုဆောင်းပါ။ နောက်ဆုံး ပြောင်းလဲမှုများနှင့် ဖျက်ထားသော entry များကို ပြန်လည်ရယူရာ page ဗားရှင်းဟောင်းများသည်
  ဘရောက်ဇာ checkpoint မလုပ်မီအထိသာ ထိုနေရာတွင် ရှိပါသည်။
- Entries၊ ResourceData နှင့် Resources တို့ကို ကိုက်ညီစေနိုင်ရန် Defender `Quarantine` tree တစ်ခုလုံးကို စုဆောင်းပါ။

## Live စနစ်တွင် သတိပြုရန် { #live-system-cautions }

ဖွင့်ထားသော ဖိုင်များသည် ကူးယူနေစဉ် ပြောင်းလဲနိုင်ပါသည်။ Registry hive များနှင့် ESE database များသည် အတွင်းပိုင်း dirty
ဖြစ်နိုင်ပါသည်။ ShimCache သည် ၎င်း၏ memory အတွင်းရှိ အသစ်ဆုံး entry များကို shutdown မတိုင်မီ flush မလုပ်နိုင်ပါ။ Volume
snapshot သို့မဟုတ် raw ဝင်ရောက်ခွင့်ကို အသုံးပြုသော forensic collection tool သည် သာမန် ဖိုင်ကူးယူခြင်းထက် ပိုကောင်းပါသည်။

## ဘေးကင်းရေး { #safety }

အထောက်အထားတွင် malware၊ exploit နမူနာများ၊ အန္တရာယ်ရှိသော LNK ဖိုင်များနှင့် spreadsheet-formula စာသားများ ပါဝင်နိုင်ပါသည်။
သီးခြားခွဲထားသော စနစ်ပေါ်တွင် ခွဲခြမ်းစိတ်ဖြာပါ၊ ပြန်လည်ရယူထားသော stream များကို execute မလုပ်ပါနှင့်၊ အန္တရာယ်ရှိသော
အကြောင်းအရာကို ကိုင်တွယ်ရန် ရည်ရွယ်မှသာ `--extract` ကို အသုံးပြုပါ။ မထိခိုက်ရသေးသော ရင်းမြစ်ကို သီးခြား ထိန်းသိမ်းပါ။
