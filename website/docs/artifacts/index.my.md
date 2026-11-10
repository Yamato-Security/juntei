# အထောက်အထား လမ်းညွှန် { #artifact-guide }

Windows သည် ဖိုင်စနစ်၊ registry၊ compatibility ဒေတာဘေ့စ်များ၊ user-interface မှတ်တမ်း၊ resource accounting နှင့်
လုံခြုံရေးထုတ်ကုန်များတွင် တစ်ခုနှင့်တစ်ခု ထပ်နေသော ခြေရာများကို ထိန်းသိမ်းထားသည်။ Juntei သည် ဤရင်းမြစ်များကို
ခွဲခြမ်းစိတ်ဖြာပေးသော်လည်း ရင်းမြစ်တစ်ခုစီ၏ အဓိပ္ပာယ်ကို စုံစမ်းစစ်ဆေးသူက ဆက်လက် အဓိပ္ပာယ်ဖော်ရမည် ဖြစ်သည်။

## မေးခွန်းအလိုက် အထောက်အထားကို ရွေးချယ်ပါ { #choose-an-artifact-by-question }

| စုံစမ်းစစ်ဆေးရေး မေးခွန်း | ဤနေရာမှ စတင်ပါ | အတည်ပြုရန် |
|---|---|---|
| ဤ executable ကို run ခဲ့သလား။ | [Prefetch](prefetch.md) | [ShimCache](shimcache.md)၊ [SRUM](srum.md)၊ event log များ |
| ဤဖိုင် တစ်ချိန်ချိန်တွင် တည်ရှိခဲ့သလား။ | [Amcache](amcache.md)၊ [$MFT](mft.md) | [USN ဂျာနယ်](usn-journal.md)၊ [$I30](i30.md)၊ ShimCache |
| ဖိုင်တစ်ခုကို မည်သည့်အချိန်တွင် ဖန်တီး၊ အမည်ပြောင်း သို့မဟုတ် ဖျက်ခဲ့သနည်း။ | [USN ဂျာနယ်](usn-journal.md) | [$MFT](mft.md)၊ [$I30](i30.md)၊ Defender quarantine |
| Metadata ပြောင်းလဲမှုမတိုင်မီ မည်သည့်အမည် သို့မဟုတ် timestamp ရှိခဲ့သနည်း။ | [$LogFile](logfile.md) | [$MFT](mft.md)၊ [USN ဂျာနယ်](usn-journal.md)၊ [$I30](i30.md) |
| ဖိုင်တစ်ခုကို မည်သူပိုင်ဆိုင်ပြီး မည်သူ ပြောင်းလဲနိုင်သနည်း။ | [$MFT](mft.md) နှင့်အတူ [$Secure:$SDS](secure-sds.md) | [SAM](sam.md)၊ Security event log များ |
| ဖြတ်လမ်း သို့မဟုတ် Prefetch ဖိုင်၏ ပစ်မှတ်သည် မည်သည့် volume ပေါ်တွင် ရှိခဲ့သနည်း။ | [$Boot](boot.md) | [LNK](lnk.md)၊ [Prefetch](prefetch.md)၊ [USB စက်ပစ္စည်းများ](usb.md) |
| အသုံးပြုသူက မည်သည့်အရာကို ဖွင့်ခဲ့သနည်း။ | [Jump Lists](jump-lists.md)၊ [LNK](lnk.md)၊ [RecentDocs](recentdocs.md) | SRUM၊ Prefetch |
| ပရိုဂရမ်တစ်ခုက ဒေတာ ပမာဏများစွာ လွှဲပြောင်းခဲ့သလား။ | [SRUM](srum.md) | Firewall၊ proxy၊ DNS၊ packet သို့မဟုတ် EDR telemetry |
| အသုံးပြုသူက မည်သည့် website များကို ကြည့်ရှုခဲ့ပြီး မည်သည့်အရာများကို download လုပ်ခဲ့သနည်း။ | [ဝဘ် ဘရောက်ဇာများ](browser.md)၊ [Internet Explorer နှင့် legacy Edge](webcache.md) | Proxy နှင့် DNS log များ၊ `$MFT` နှင့် USN Journal၊ Defender၊ SRUM |
| အသုံးပြုသူက မည်သူနှင့် e-mail အပြန်အလှန် ပေးပို့ခဲ့ပြီး မည်သည့်အရာများကို ပူးတွဲထားခဲ့သနည်း။ | [Outlook](outlook.md) | Mail server နှင့် gateway log များ၊ `$MFT` နှင့် USN Journal၊ Defender၊ LNK နှင့် Jump Lists |
| ဤ server ကို မည်သည့် client များနှင့် account များက မည်သည့်နေ့များတွင် အသုံးပြုခဲ့သနည်း။ | [အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း (UAL)](ual.md) | Security event log များ၊ SMB နှင့် RDP log များ၊ firewall log များ |
| အမြဲတမ်းဝင်ရောက်နိုင်မှု (persistence) ကို ပြင်ဆင်ထားခဲ့သလား။ | [ဝန်ဆောင်မှုများ](services.md)၊ [TaskCache](taskcache.md)၊ [Registry](registry.md) | Event log များ၊ Prefetch၊ `$MFT` |
| Defender က မည်သည့်အရာကို ဖယ်ရှားခဲ့သနည်း။ | [Defender သီးခြားခွဲထားသော ဒေတာ](defender-quarantine.md) | Event log များ၊ `$MFT`၊ USN Journal |
| Local အကောင့်များကို ထပ်ထည့်ခဲ့ သို့မဟုတ် ပြောင်းလဲခဲ့သလား။ | [SAM](sam.md) | Security event log များနှင့် registry transaction မှတ်တမ်း |

## Field များမတိုင်မီ သတိပေးချက်များကို ဖတ်ပါ { #read-warnings-before-fields }

အထောက်အထား စာမျက်နှာတိုင်းသည် တူညီသော အစီအစဉ်ကို လိုက်နာသည်-

1. **၎င်း ဆိုသည်မှာ** နှင့် ၎င်းက ဖြေဆိုနိုင်သော မေးခွန်းများ။
2. **၎င်းကို ရှာဖွေရမည့်နေရာ** နှင့် စုဆောင်းခြင်းက ၎င်းကို မည်သို့ သက်ရောက်သည်။
3. **၎င်းက သက်သေပြသည့်အရာ—နှင့် သက်သေမပြသည့်အရာ။**
4. Juntei ဖြင့် **၎င်းကို ခွဲခြမ်းစိတ်ဖြာပုံ**။
5. **ထုတ်ယူမှု field တိုင်းကို ဖတ်ရှုပုံ**။
6. လွတ်လပ်သော အထောက်အထားများနှင့် **၎င်းကို ချိတ်ဆက်စစ်ဆေးပုံ**။
7. အနိမ့်ပိုင်းအဆင့် သုတေသနအတွက် **နောက်ထပ် အသေးစိတ်များ**။

!!! danger "Parser တွေ့ရှိချက်ကို အန္တရာယ်ရှိခြင်းနှင့် တူညီသည်ဟု မယူဆပါနှင့်"
    Juntei တွေ့ရှိချက်များသည် `System32` ပြင်ပရှိ system-binary အမည်ကဲ့သို့ ပြန်လည်သုံးသပ်ထိုက်သော ပုံစံများကို
    ဖော်ထုတ်သည်။ Installer များ၊ စီမံခန့်ခွဲသူများ၊ compatibility layer များနှင့် ဆော့ဖ်ဝဲ အပ်ဒိတ်များသည် ပုံမှန်မဟုတ်သော
    မှတ်တမ်းများကို ထုတ်ပေးနိုင်သည်။ အခြေအနေနှင့် လွတ်လပ်သော အထောက်အထားများဖြင့် အတည်ပြုပါ။
