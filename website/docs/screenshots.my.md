# ဖန်သားပြင်ဓာတ်ပုံများ { #screenshots }

Juntei ၏ command line၊ အလိုအလျောက် ခွဲခြမ်းစိတ်ဖြာမှုနှင့် စနစ်တကျ စီစဉ်ထားသော စုံစမ်းစစ်ဆေးမှု ရလဒ်များကို လေ့လာကြည့်ပါ။
ဖန်သားပြင်ဓာတ်ပုံကို အရွယ်အစားအပြည့်ဖြင့် ကြည့်ရန် နှိပ်ပါ။

## 1. စတင်ဖွင့်ခြင်းနှင့် ညွှန်ကြားချက်များ { #1-startup-and-commands }

<figure markdown>

[![CODE BLUE release နှင့် အသုံးပြုနိုင်သော ညွှန်ကြားချက်များကို ပြသထားသည့် Juntei စတင်ဖွင့်မျက်နှာပြင်](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>Juntei ၏ စတင်ဖွင့်မျက်နှာပြင်တွင် အလိုအလျောက် ခွဲခြမ်းစိတ်ဖြာမှု၊ source အလိုက် parser များနှင့် Persistence Hunter တို့ကို စာရင်းပြထားသည်။</figcaption>

</figure>

## 2. စက္ကန့် 20 အတွင်း အလိုအလျောက် ခွဲခြမ်းစိတ်ဖြာခြင်း { #2-automatic-parsing-in-under-20-seconds }

**MacBook Pro M4** တစ်လုံးပေါ်တွင် အောက်ဖော်ပြပါ run သည် **စနစ် ခြောက်ခု** မှ **စုဆောင်းထားသော Windows ဒေတာ 8.7 GiB** ကို
ခွဲခြမ်းစိတ်ဖြာပြီး **ထုတ်ယူမှု 4.4 GiB ကို စက္ကန့် 20 အောက်တွင်** သိမ်းဆည်းခဲ့သည်။
[auto-parse](commands/auto-parse.md) ညွှန်ကြားချက်သည် parallel process 16 ခုကို အသုံးပြု၍ parser task 240 ခုကို run ခဲ့ပြီး
ရလဒ်ဖိုင် 482 ခုကို ထုတ်ပေးခဲ့သည်။ **အောင်မြင်သော task 210 ခု၊ အထောက်အထားများကို စုဆောင်းမထားသောကြောင့်
ကျော်သွားသော task 30 ခုနှင့် မအောင်မြင်မှု သုည** ရှိခဲ့သည်။

<figure markdown>

[![စနစ် ခြောက်ခု၊ input 8.7 GiB နှင့် output 4.4 GiB ကို စက္ကန့် 20 အောက်တွင် ပြီးဆုံးသွားသော Juntei auto-parse](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>Host များကို အလိုအလျောက် ရှာဖွေခြင်း၊ parallel ခွဲခြမ်းစိတ်ဖြာခြင်းနှင့် input အရွယ်အစား၊ output အရွယ်အစား၊ task ရလဒ်များနှင့် ကြာချိန်ကို ပြသသော ပြီးဆုံးမှု အနှစ်ချုပ်။</figcaption>

</figure>

## 3. စုံစမ်းစစ်ဆေးရန် စနစ်တကျ စီစဉ်ထားသော ရလဒ်များ { #3-results-organized-for-investigation }

<figure markdown>

[![DFIR အမျိုးအစားများအလိုက် အုပ်စုဖွဲ့ထားသော စနစ်တစ်ခု၏ output ဖိုင်တွဲများ၊ အကောင့် CSV များနှင့် ၎င်းတို့၏ parser log များ](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>ကွန်ပျူတာတစ်လုံးချင်းစီ၏ ရလဒ်များကို DFIR အမျိုးအစားအလိုက် အုပ်စုဖွဲ့ထားသည်။ အထောက်အထား CSV များနှင့် ၎င်းတို့နှင့် ကိုက်ညီသော <code>-logs.txt</code> ဖိုင်များကို အတူတကွ ထားရှိသောကြောင့် record များနှင့် ခွဲခြမ်းစိတ်ဖြာမှု အသေးစိတ်များကို အလွယ်တကူ ရှာတွေ့နိုင်သည်။</figcaption>

</figure>
