# ညွှန်ကြားချက်စာရင်း { #command-list }

Parser တစ်ခုချင်းစီကို ၎င်းတို့၏ source အမျိုးအစားအလိုက် အုပ်စုဖွဲ့ထားသည်။ ၎င်းတို့ကို လေ့လာရန် `juntei --help`၊
`juntei <SOURCE> --help` သို့မဟုတ် `juntei <SOURCE> <ARTIFACT> --help` ကို run ပါ-

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

ကွန်ပျူတာများစွာပါဝင်သော Windows အထောက်အထား စုဆောင်းမှုအတွက်
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md) ကို အသုံးပြုပါ။ ၎င်းသည် အုပ်စုဖွဲ့ထားသော တူညီသည့်
ညွှန်ကြားချက်များကို ခေါ်ယူပြီး ရလဒ်များနှင့် log များကို ကွန်ပျူတာနှင့် စုံစမ်းစစ်ဆေးရေး category အလိုက် စီစဉ်ပေးသည်။

ခွဲခြမ်းစိတ်ဖြာထားသော ရှေးဟောင်းပစ္စည်းများအတွက် အချိန်ဇယားအတွက် [`juntei super-timeline -d <PARSED> -o SuperTimeline.csv`](super-timeline.md) ကိုသုံးပါ။ ၎င်းသည် အချိန်တံဆိပ်တုံး အဓိပ္ပါယ်များကို ချဲ့ထွင်ပြီး တံဆိပ်ခတ်ထားသည်။ အထွက်အား ပုံသေအားဖြင့် စီထားခြင်းမရှိပါ။ အချိန်နှင့် တပြေးညီ အမှာစာအတွက် `--sort` ကို ပေါင်းထည့်၍ ပွားများ စုစည်းပါ။ `auto-parse -j` သည် မူရင်းမှတ်တမ်းများကို ဆက်လက်ထိန်းသိမ်းထားသည်။

## [Disk image များ](mount.md) { #disk-images }

| ညွှန်ကြားချက် | Input | အသေးစိတ် လမ်းညွှန် |
|---|---|---|
| `mount` | VHDX / E01 / Ex01 disk image | [Disk image များကို mount လုပ်ခြင်း](mount.md) |

## [ESE ညွှန်ကြားချက်များ](user-activity.md) { #ese-commands }

| ညွှန်ကြားချက် | Input အထောက်အထား | အသေးစိတ် လမ်းညွှန် |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း (UAL)](../artifacts/ual.md) |
| `parse-ese webcache` | ESE `WebCacheV01.dat` | [Internet Explorer နှင့် legacy Edge](../artifacts/webcache.md) |

## [ဖိုင် ညွှန်ကြားချက်များ](execution.md) { #file-commands }

| ညွှန်ကြားချက် | Input အထောက်အထား | အသေးစိတ် လမ်းညွှန် |
|---|---|---|
| `parse-file defender-quarantine` | Defender quarantine tree သို့မဟုတ် entry | [Defender သီးခြားခွဲထားသော ဒေတာ](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [LNK ဖြတ်လမ်းများ](../artifacts/lnk.md) |
| `parse-file outlook` | `*.pst`, `*.ost`, `*.msg`, `*.oft` | [Outlook](../artifacts/outlook.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [NTFS ညွှန်ကြားချက်များ](filesystem.md) { #ntfs-commands }

| ညွှန်ကြားချက် | Input အထောက်အထား | အသေးစိတ် လမ်းညွှန် |
|---|---|---|
| `parse-ntfs boot` | `$Boot` သို့မဟုတ် NTFS volume image | [Boot sector ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`) သို့မဟုတ် disk image | [ဖိုင်တွဲ အညွှန်းများ ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | NTFS `$LogFile` | [NTFS အပြောင်းအလဲမှတ်တမ်း](../artifacts/logfile.md) |
| `parse-ntfs mft` | NTFS `$MFT` | [ပင်မဖိုင်ဇယား](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [လုံခြုံရေးဖော်ပြချက်များ ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [USN ဂျာနယ်](../artifacts/usn-journal.md) |

## [Registry ညွှန်ကြားချက်များ](security-registry.md) { #registry-commands }

| ညွှန်ကြားချက် | Input အထောက်အထား | အသေးစိတ် လမ်းညွှန် |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [ကွန်ရက်များ](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [LSA လျှို့ဝှက်ဒေတာ](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (နှင့် `BCD-Template`) | [Boot ပြင်ဆင်မှု ဒေတာ (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [အပလီကေးရှင်း လိုက်ဖက်မှု](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [စစ်ဆေးမှတ်တမ်း မူဝါဒ](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage၊ RecentApps နှင့် Taskbar တွင် ပင်ထိုးထားသော အရာများ](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [အပလီကေးရှင်း မှတ်တမ်း](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [နောက်ဆုံးအသုံးပြုထားသော စာရင်းများ](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (အသုံးပြုသူအမည်များအတွက် `SAM`, `SOFTWARE` နှင့်အတူ) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (Windows XP တွင် `NTUSER.DAT`) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [ထည့်သွင်းထားသော ပရိုဂရမ်များ](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [ဖွင့်ရန်နှင့် အမည်သစ်ဖြင့် သိမ်းရန် dialog များ](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [Registry အလိုအလျောက် စတင်မှု](../artifacts/persistence.md) |
| `parse-registry read` | မည်သည့် primary registry hive မဆို | [Registry hive များ](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (အသုံးပြုသူ SID များအတွက် `SOFTWARE` နှင့်အတူ) | [အဝေးမှ ဝင်ရောက်မှု (အသုံးပြုသူဘက်)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [SAM စက်တွင်းအကောင့်များ](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [လုံခြုံရေးဆက်တင်များ](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Windows ဝန်ဆောင်မှုများ](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` သို့မဟုတ် export လုပ်ထားသော AppCompatCache တန်ဖိုး | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | စနစ်တစ်ခု၏ `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` | [စနစ်အချက်အလက်](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Task Scheduler cache](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`၊ `SOFTWARE` နှင့် `NTUSER.DAT` နှင့်အတူ | [USB စက်ပစ္စည်းများ](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## [SQLite ညွှန်ကြားချက်များ](sqlite.md) { #sqlite-commands }

| ညွှန်ကြားချက် | Input အထောက်အထား | အသေးစိတ် လမ်းညွှန် |
|---|---|---|
| `parse-sqlite browser` | `History`, `Cookies`, `Web Data`, `places.sqlite`, `cookies.sqlite`, `formhistory.sqlite` | [ဝဘ် ဘရောက်ဇာများ](../artifacts/browser.md) |

## Source အမျိုးအစားများနှင့် format ပြောင်းလဲမှုများ { #source-types-and-format-changes }

အုပ်စုများသည် Juntei လက်ရှိ ပံ့ပိုးထားသော format များကို ဖော်ပြသည်။ `parse-registry amcache` သည် `Amcache.hve` ကို
ဖတ်သည်။ ၎င်းသည် SQLite Amcache inventory ဒေတာဘေ့စ်များကို ပံ့ပိုးသည်ဟု မဆိုလိုပါ။ အခြား storage format အတွက် ပံ့ပိုးမှုကို
သီးခြားလွတ်လပ်စွာ ထပ်ထည့်နိုင်ရန် artifact parser crate များကို သီးခြားစီ ထားရှိသည်။

ယခင် flat parser အမည်များကို အထက်ပါ source-and-artifact path များဖြင့် အစားထိုးထားသည်။ ရှိပြီးသား parser option များကို
artifact အမည်နောက်တွင် ဆက်ထားပါ။ `auto-parse` နှင့် ရှိပြီးသား registry subcommand များသည် ၎င်းတို့၏ အမည်များကို
ဆက်လက် ထိန်းသိမ်းထားသည်။

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` သည် source format များစွာကို ဖြတ်ကျော်၍ ရှာဖွေသောကြောင့် ထိပ်တန်းအဆင့် ခွဲခြမ်းစိတ်ဖြာရေး
ညွှန်ကြားချက် ဖြစ်သည်။ ၎င်းသည် `parse-registry persistence` မှ ပြင်ဆင်မှု စာရင်းအပြည့်အစုံကို စစ်ဆေးမှု 131 ခု၊
allow/block အမျိုးအစားခွဲခြင်း၊ payload metadata၊ coverage နှင့် timestamp ချိတ်ဆက်စစ်ဆေးမှုတို့ဖြင့် ဖြည့်စွက်ပေးသည်။

## ထိပ်တန်းအဆင့် အသုံးပြုပုံ { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX or E01 disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  parse-sqlite        Parse SQLite databases
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Source စာမျက်နှာများသည် ညွှန်ကြားချက် အသုံးပြုပုံကို ပြန်လည်ဖော်ပြပြီး ဥပမာတိုများ ပေးသည်။ အထောက်အထား လမ်းညွှန်များသည်
အထောက်အထား၊ ထုတ်ယူမှု field များ၊ ကန့်သတ်ချက်များနှင့် စုံစမ်းစစ်ဆေးမှု ဗျူဟာကို ရှင်းပြသည်။
