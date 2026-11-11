# ဆောင်းပါးနှင့် ကိုးကားချက်များ { #articles-and-references }

ဤစာမျက်နှာသည် အထောက်အထား လမ်းညွှန်များထက် ကျော်လွန်၍ လေ့လာလိုသော စာဖတ်သူများအတွက် အရည်အသွေးမြင့် စတင်ရာနေရာများကို
စုစည်းထားသည်။ အထောက်အထားတစ်ခုစီ၏ **နောက်ထပ် အသေးစိတ်များ** အပိုင်းများတွင် အဆင့်နိမ့် အသေးစိတ်ဆိုင်ရာ ပစ်မှတ်ထားသည့်
link များ ပါဝင်သည်။

## စတင်လေ့လာသူများအတွက် DFIR မိတ်ဆက် { #beginner-dfir-orientation }

- [SANS Windows Forensic Analysis poster](https://www.sans.org/posters/windows-forensic-analysis/) — အသုံးများသော Windows အထောက်အထားများနှင့် တည်နေရာများ၏ ရုပ်ပုံမြေပုံ။
- [SANS Hunt Evil poster](https://www.sans.org/posters/hunt-evil/) — လက်တွေ့ hunting pivot များနှင့် Windows စုံစမ်းစစ်ဆေးမှု သတိပေးချက်များ။
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — လေ့လာရန်နှင့် parser မှန်ကန်မှု စစ်ဆေးရန်အတွက် အများသုံး အထောက်အထား နမူနာများ။
- [Velociraptor artifact ကိုးကားချက်](https://docs.velociraptor.app/artifact_references/) — Windows အထောက်အထား အများအပြားအတွက် စုဆောင်းမှုအခြေပြု ဖော်ပြချက်များနှင့် VQL implementation များ။
- [Plaso စာရွက်စာတမ်း](https://plaso.readthedocs.io/) — အထောက်အထားများ ချိတ်ဆက်ခွဲခြမ်းစိတ်ဖြာမှုအတွက် timeline သဘောတရားများနှင့် parser များ။

## ဖိုင်စနစ် အထောက်အထားများ { #file-system-artifacts }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Security descriptor များ](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) နှင့် [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — security descriptor များ၊ ACE များနှင့် conditional expression များ။
- [libfsntfs ပုံစံ စာရွက်စာတမ်း](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Omer Ben-Amram ၏ mft parser](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — INDX record နှင့် index-slack carving ပါဝင်သော NTFS parser။
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — ဖိုင်အမည်များအတွက် `$I30` slack ကို carve လုပ်သည်။
- [LogFileParser](https://github.com/jschicht/LogFileParser) — အသေးစိတ် NTFS transaction-log သုတေသနနှင့် ခွဲခြမ်းစိတ်ဖြာမှု။
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — `$LogFile`၊ `$MFT` နှင့် USN မှတ်တမ်းကို ချိတ်ဆက်ပေးသည်။

## လုပ်ဆောင်မှုနှင့် စာရင်းကောက် အထောက်အထားများ { #execution-and-inventory-artifacts }

- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — Prefetch ပုံစံ/library လုပ်ငန်း။
- [Mandiant: Forensic စုံစမ်းစစ်ဆေးမှုများတွင် Application Compatibility Cache ကို အသုံးချခြင်း](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)

## အသုံးပြုသူ လှုပ်ရှားမှု အထောက်အထားများ { #user-activity-artifacts }

- [Microsoft Open Specifications: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## Registry နှင့် အမြဲတမ်းဝင်ရောက်နိုင်မှု { #registry-and-persistence }

- [Microsoft: Registry value type များ](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Maxim Suhanov ၏ Windows registry ပုံစံ သတ်မှတ်ချက်](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) နှင့် [libregf](https://github.com/libyal/libregf) — အဆင့်နိမ့် hive ခွဲခြမ်းစိတ်ဖြာမှုနှင့် ပြန်လည်ရယူမှု ကိုးကားချက်များ။
- [Microsoft: Services registry tree](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: Task Scheduler schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender နှင့် malware ကိုင်တွယ်ခြင်း { #defender-and-malware-handling }

- [Microsoft: Defender တွင် quarantine လုပ်ထားသော ဖိုင်များကို ပြန်လည်ရယူခြင်း](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [dissect.target Defender plugin](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — သီးခြား သုတေသန/ထုတ်ယူမှု implementation တစ်ခု။
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/) — ကိုင်တွယ်မှုနှင့် ခွဲခြမ်းစိတ်ဖြာမှု အရင်းအမြစ်များ။

## Juntei parser project များ { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`, `$I30`, USN Journal, `$LogFile`, `$Boot`, `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "ပြင်ပ အကြောင်းအရာများ ပြောင်းလဲတတ်သည်"
    ဖော်ပြချက်များကို သက်ရောက်ခံရသော Windows ဗားရှင်းနှင့် သင်စုဆောင်းထားသော ဒေတာတို့နှင့် တိုက်ဆိုင်စစ်ဆေးပါ။ အသုံးဝင်သော
    ဆောင်းပါးတစ်ခုသည် စကြာဝဠာ စည်းမျဉ်းတစ်ခုထက် တိကျသော operating-system မျိုးဆက်၊ parser ဗားရှင်း သို့မဟုတ်
    စမ်းသပ်မှုတစ်ခုကို ဖော်ပြထားခြင်း ဖြစ်နိုင်သည်။
