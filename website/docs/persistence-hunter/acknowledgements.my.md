# Persistence Hunter ကျေးဇူးတင်လွှာ { #persistence-hunter-acknowledgements }

Juntei ၏ `persistence-hunter` ညွှန်ကြားချက်သည် အသိုင်းအဝိုင်း၏ persistence ရှာဖွေရေး မျိုးဆက်နှစ်ခုအပေါ် အခြေခံထားပါသည်။
၎င်းတို့၏ နည်းစနစ်များ၊ implementation များနှင့် သုတေသနများကို စုံစမ်းစစ်ဆေးသူများ ရရှိနိုင်အောင် ပြုလုပ်ပေးခဲ့သော စာရေးသူများနှင့်
ပါဝင်ကူညီသူများကို ကျေးဇူးတင်ပါသည်။

## PyrsistenceSniper: Python project { #pyrsistencesniper-the-python-project }

[Hexastrike](https://github.com/Hexastrike) ၏ [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper)
သည် ဤ feature အတွက် တိုက်ရိုက် ကိုးကားချက် ဖြစ်ပါသည်။ Juntei သည် ၎င်း၏ စစ်ဆေးချက် 130 ခုပါ catalogue ကို မူလ allowlist၊
LOLBAS snapshot နှင့် အမျိုးအစားခွဲခြားမှု အပြုအမူတို့နှင့်အတူ native Rust သို့ port လုပ်ထားပါသည်။ ကိုးကားသည့် ဗားရှင်းကို
[commit `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f) တွင် pin လုပ်ထားပါသည်။

Python project သည် MIT license ဖြစ်ပါသည်။ Juntei သည် ၎င်း၏ မူပိုင်ခွင့်နှင့် license ကြေညာချက်ကို
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt) တွင် ထိန်းထားပါသည်။
Juntei ၏ implementation သည် Python interpreter မလိုဘဲ offline လည်ပတ်ပါသည်။ ၎င်း၏ collection ကိုင်တွယ်ပုံ၊
coverage အစီရင်ခံပုံနှင့် မပြည့်စုံသော အထောက်အထားကို သတိထား၍ အဓိပ္ပာယ်ဖော်ပုံတို့ကို
[ညွှန်ကြားချက် လမ်းညွှန်](../commands/persistence-hunter.md) တွင် ဖော်ပြထားပါသည်။

## PersistenceSniper: မူလ PowerShell tool { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) သည် PyrsistenceSniper ကို စိတ်ကူးရစေခဲ့သော မူလ
PowerShell persistence ရှာဖွေရေး project ဖြစ်ပါသည်။ PyrsistenceSniper သည် Federico Lagrasta ကို ဂုဏ်ပြုဖော်ပြထားပြီး
မူလ repository သည် Riccardo Ancarani နှင့် အခြား ပါဝင်ကူညီသူများကိုလည်း ဂုဏ်ပြုဖော်ပြထားပါသည်။ Windows persistence
နည်းလမ်းများကို စုံစမ်းစစ်ဆေးရန် ပိုမိုလွယ်ကူစေခဲ့သော မူလ project နှင့် ၎င်း၏ ပါဝင်ကူညီသူများကို ကျေးဇူးတင်ပါသည်။

Juntei ၏ တိုက်ရိုက် port ကိုးကားချက်မှာ Python project ဖြစ်ပါသည်။ မူလ PowerShell tool ကို ထိုမျိုးရိုးစဉ်ဆက်အတွက်
ဂုဏ်ပြုဖော်ပြထားပါသည်။ Juntei သည် မူလ project ၏
[scheduled-task COM resolution PR #31](https://github.com/last-byte/PersistenceSniper/pull/31)
နှင့် [Start Menu library/CLSID-folder PR #32](https://github.com/last-byte/PersistenceSniper/pull/32) တွင်
[hongson11698](https://github.com/hongson11698) က အဆိုပြုခဲ့သော နည်းလမ်းများကိုလည်း သီးခြားလွတ်လပ်စွာ implement လုပ်ထားပါသည်။
၎င်းတို့သည် Python ကိုးကား catalogue ကို ဖြည့်စွက်ပါသည်။ ၎င်းသည် upstream project နှစ်ခုစလုံးရှိ live-system feature တိုင်းကို
Juntei ၏ offline ညွှန်ကြားချက်က implement လုပ်ထားသည်ဟု မဆိုလိုပါ။

## ထောက်ပံ့သော သုတေသနနှင့် tool များ { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/) သည် catalogue တွင် ပါဝင်သော technique identifier များကို ပေးပါသည်။
  Technique mapping တစ်ခုသည် နည်းလမ်းတစ်ခု၏ ဖြစ်နိုင်သော အသုံးပြုမှုကို ဖော်ပြပါသည်။ ကိုက်ညီသော registry value
  သို့မဟုတ် ဖိုင်တစ်ခုသည် ၎င်းတစ်ခုတည်းဖြင့် ATT&CK က အတည်ပြုထားသော တိုက်ခိုက်မှု မဟုတ်ပါ။
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  ကိုလည်း Windows autostart များကို စစ်ဆေးရန် စိတ်ကူးရစေခဲ့သည့်အရာအဖြစ် PyrsistenceSniper က အသိအမှတ်ပြုထားပါသည်။
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) သည် အလွဲသုံးစားလုပ်နိုင်သော တရားဝင် Windows binary များကို
  စာတမ်းပြုထားပါသည်။ ဤ binary များထဲမှ တစ်ခုကို အသိအမှတ်ပြုခြင်းသည် command တစ်ခုကို အဓိပ္ပာယ်ဖော်ရာတွင် ကူညီသော်လည်း
  ထို command ကို အလိုအလျောက် အန္တရာယ်ရှိသည်ဟု မဖြစ်စေပါ။

Juntei တစ်ခုလုံးတွင် အသုံးပြုထားသော registry၊ NTFS၊ Shell Link၊ event-log နှင့် အခြား parser အခြေခံများအတွက်
[project တစ်ခုလုံး၏ ကျေးဇူးတင်လွှာ](../acknowledgements.md) ကို ကြည့်ပါ။

[Persistence နည်းလမ်းများ လမ်းညွှန်](index.md) သို့ ပြန်သွားပါ။
