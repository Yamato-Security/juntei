# ကျေးဇူးတင်လွှာ { #acknowledgements }

DFIR နှင့် open-source အသိုင်းအဝိုင်းများက parser များ၊ ဖိုင်ပုံစံ သုတေသနများ၊ ကိုးကားရန် output များ၊
spec များနှင့် စမ်းသပ်ရန် အထောက်အထားများကို ထုတ်ဝေပေးခဲ့သောကြောင့် Juntei ဖြစ်ပေါ်လာခြင်း ဖြစ်ပါသည်။
အောက်ပါ project များသည် implementation၊ မှန်ကန်မှု စစ်ဆေးခြင်း၊ field အမည်ပေးခြင်း သို့မဟုတ် analyst များ
မျှော်လင့်သည့် output တို့အတွက် အထောက်အကူ ဖြစ်ခဲ့ပါသည်။ ဤစာရင်းတွင် ပါဝင်ခြင်းသည် ၎င်းတို့၏ code ကို
Juntei ထဲတွင် ထည့်သွင်းထားသည်ဟု မဆိုလိုပါ။ license အတွက် project တစ်ခုချင်းစီကို ကိုးကားပါ။

## Yamato Security က ထိန်းသိမ်းသော parser အခြေခံများ { #parser-foundations-maintained-by-yamato-security }

| Juntei နယ်ပယ် | ထိန်းသိမ်းထားသော parser | Upstream/အခြေခံ |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`mft` crate) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| USN Journal | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`usn` crate) | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`logfile` crate) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| Directory index များ (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`mft` crate၊ `i30` module) | အောက်တွင် ဖော်ပြထားသော index-slack သုတေသန |
| NTFS `$Boot`, `$Secure:$SDS` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`boot` နှင့် `secure` crate များ) | Microsoft ၏ MS-DTYP နှင့် အောက်တွင် ဖော်ပြထားသော NTFS သုတေသန |
| SRUM, UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) နှင့် [ese](https://github.com/Sootmark/ese) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Sootmark shell ၏ compound-file နှင့် Jump List လုပ်ငန်း |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| VHDX disk image များ (`mount`) | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) နှင့် [ntfs-core](https://github.com/SecurityRonin/ntfs-forensic)၊ [fuser](https://github.com/cberner/fuser) ဖြင့် serve လုပ်သည် | Microsoft ၏ MS-VHDX |
| Registry | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | အောက်တွင် ဖော်ပြထားသော registry ဖိုင်ပုံစံနှင့် ပြန်လည်ရယူခြင်း သုတေသန |

## ကိုးကားသည့် tool များနှင့် လွတ်လပ်သော မှန်ကန်မှု စစ်ဆေးခြင်း { #reference-tools-and-independent-validation }

- Hexastrike ၏ [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) သည် native Rust
  `persistence-hunter` ညွှန်ကြားချက်အတွက် persistence စစ်ဆေးချက် catalogue၊ မူလ allowlist၊ LOLBAS snapshot
  နှင့် ကိုးကားရန် အပြုအမူကို ပံ့ပိုးပေးခဲ့ပါသည်။ ၎င်း၏ MIT license နှင့် မူပိုင်ခွင့် ကြေညာချက်ကို
  `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt` တွင် ထိန်းသိမ်းထားပါသည်။ မူလ
  [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) သည် Python project ကို စိတ်ကူးရစေခဲ့ပါသည်။
  မျိုးရိုးစဉ်ဆက် အပြည့်အစုံအတွက် [persistence ကျေးဇူးတင်လွှာ](persistence-hunter/acknowledgements.md) ကို ကြည့်ပါ။

- Maxim Suhanov ၏ **regf specification** နှင့် **yarp** တို့သည် registry ဖွဲ့စည်းပုံများ၊ transaction-log
  replay နှင့် ဖျက်ထားသော record များ ပြန်လည်ရယူခြင်းအတွက် အထောက်အကူ ဖြစ်ခဲ့ပြီး ၎င်း၏ **dfir_ntfs** သည်
  INDX record များနှင့် index-slack carving အတွက် ကိုးကားချက် ဖြစ်ခဲ့ပါသည်။
- Harel Segev ၏ **INDXRipper** သည် index slack မှ ဖိုင်အမည်များကို carving လုပ်ရန် နောက်ထပ် ကိုးကားချက် ဖြစ်ပါသည်။
- **dfir_ntfs**၊ Joakim Schicht ၏ **LogFileParser** နှင့် Stroz Friedberg ၏ **ntfs-linker** တို့သည်
  `$LogFile` မှန်ကန်မှု စစ်ဆေးခြင်းနှင့် အဓိပ္ပာယ်ဖော်ခြင်းအတွက် အထောက်အကူ ဖြစ်ခဲ့ပြီး Velocidex ၏ **go-ntfs** သည်
  အများသုံး NTFS စမ်းသပ်အထောက်အထားများကို ပံ့ပိုးပေးခဲ့ပါသည်။
- Joachim Metz / libyal project များဖြစ်သော **libregf**၊ **libfsntfs**၊ **libscca**၊ **libfwsi** နှင့်
  **libesedb** တို့သည် အသေးစိတ် ဖိုင်ပုံစံ သုတေသနနှင့် လွတ်လပ်သော parsing ကိုးကားချက်များကို ပေးပါသည်။
- Fox-IT ၏ **dissect.esedb**၊ **dissect.ntfs** နှင့် **dissect.target** တို့သည် မှန်ကန်မှု စစ်ဆေးစဉ်
  အသုံးပြုခဲ့သော SRUM၊ UAL၊ `$Secure:$SDS`၊ SAM နှင့် Defender တို့၏ လွတ်လပ်သော အဓိပ္ပာယ်ဖော်ချက်များကို ပေးခဲ့ပါသည်။
- `mkntfs` ဖြင့် ပြုလုပ်ထားသော volume များ၏ NTFS boot sector များအတွက် **The Sleuth Kit** ၏ `fsstat` သည်
  ကိုးကားချက် ဖြစ်ခဲ့ပါသည်။
- Eric Zimmerman ၏ tool အချို့သည် နှိုင်းယှဉ်ရာတွင် အသုံးပြုခဲ့သော field အမည်များနှင့် စစ်ဆေးသူများ၏ မျှော်လင့်ချက်များကို
  ချမှတ်ပေးခဲ့ပါသည်။
- **regipy**၊ **python-registry**၊ **Notatin** နှင့် **artemis** တို့သည် registry implementation များ၊
  စမ်းသပ်ချက်များ သို့မဟုတ် နမူနာ အဓိပ္ပာယ်ဖော်ချက်များကို ပေးခဲ့ပါသည်။
- **LnkParse3**၊ AbdulRhmanAlfaifi ၏ **lnk_parser** နှင့် **jumplist_parser** နှင့် JLECmd တို့သည်
  LNK/Jump List ၏ လွတ်လပ်သော အပြုအမူနှင့် အများသုံး စမ်းသပ်ကိစ္စများကို ပေးခဲ့ပါသည်။
- Security Ronin ၏ **prefetch-forensic** နှင့် **forensicnomicon** project တို့သည် Juntei ၏
  တန်ဖိုးမြင့် Prefetch/Amcache/ShimCache ရှုမြင်ချက်များအတွက် အထောက်အကူ ဖြစ်ခဲ့ပါသည်။
- **defender-dump** နှင့် dissect.target တို့သည် Defender quarantine ဖွဲ့စည်းပုံများနှင့် ဘေးကင်းသော
  ထုတ်ယူမှု အပြုအမူကို နားလည်ရန် သီးခြား implementation များကို ပေးပါသည်။
- **Plaso**၊ **Chainsaw** နှင့် **Velociraptor** တို့သည် artifact အများအပြားတွင် parser နှိုင်းယှဉ်ချက်များ၊
  စမ်းသပ်အထောက်အထားများ သို့မဟုတ် စုဆောင်းမှုအခြေပြု ကိုးကားရန် အပြုအမူကို ပေးခဲ့ပါသည်။

## Spec များနှင့် သုတေသန { #specifications-and-research }

Shell Links၊ Compound Binary Files၊ NTFS/SAM protocol များ၊ registry/service/task ဖွဲ့စည်းပုံများနှင့်
API စာတမ်းများအတွက် Open Specifications ကို ထုတ်ဝေပေးသည့် Microsoft ကို ကျေးဇူးတင်ပါသည်။ ထို့အပြင်
ဤ artifact များကို လက်တွေ့လုပ်ကိုင်သူများ နားလည်နိုင်စေခဲ့သော AppCompatCache၊ Amcache၊ SRUM၊ Prefetch၊
registry နှင့် NTFS forensic သုတေသန စာရေးသူများကိုလည်း ကျေးဇူးတင်ပါသည်။

## စမ်းသပ်အထောက်အထား { #test-evidence }

Andrew Rathbun ၏
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)၊ NIST CFReDS၊ Plaso၊
Regipy၊ Artemis၊ parser စမ်းသပ် repository များနှင့် license မှန်ကန်စွာ ပါရှိသော အခြား collection များမှ
အများသုံး အထောက်အထားများကြောင့် တကယ့် Windows ဒေတာဖြင့် regression စမ်းသပ်ခြင်း ပြုလုပ်နိုင်ပါသည်။
Juntei သည် ထည့်သွင်းထားသော စမ်းသပ်ဖိုင်များ၏ ဘေးတွင် attribution/license မှတ်စုများကို ထားရှိပါသည်။

## ပါဝင်ကူညီသူများ { #contributors }

Juntei နှင့် Yamato Security parser များတွင် ပါဝင်ကူညီသူ၊ ပြန်လည်သုံးသပ်သူ၊ issue တင်ပြသူနှင့် edge case များကို
ပေးပို့ခဲ့သော အသုံးပြုသူ အားလုံးကို ကျေးဇူးတင်ပါသည်။ လက်ရှိ ပါဝင်ကူညီသူများကို GitHub ၏
[Juntei ပါဝင်ကူညီသူများ စာမျက်နှာ](https://github.com/Yamato-Security/juntei/graphs/contributors) တွင် ဖော်ပြထားပါသည်။

## License { #license }

Juntei ကို
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) ဖြင့် ထုတ်ဝေထားပြီး မူပိုင်ခွင့်မှာ Shirofune Security Co., Ltd. ဖြစ်ပါသည်။ ၎င်း၏ source code ကို
ဗားရှင်း 1.0.0 မတိုင်မီက AGPLv3 ဖြင့် ထုတ်ဝေခဲ့ပါသည်။ ၎င်းကို တည်ဆောက်ရာတွင် အသုံးပြုထားသော crate များနှင့်
အခြား third-party code နှင့် ဒေတာတို့၏ license များကို
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES) တွင် ကြည့်နိုင်ပါသည်။
