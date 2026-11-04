# စစ်ဆေးမှတ်တမ်း မူဝါဒ { #audit-policy }

## ၎င်း ဆိုသည်မှာ { #what-it-is }

Audit policy သည် Windows က Security event log တွင် မည်သည့် လုံခြုံရေး event များကို မှတ်တမ်းတင်မည်ကို သတ်မှတ်သည်။
Windows Vista နှင့် နောက်ပိုင်းတွင် subcategory အလိုက် (Windows 10 နှင့် 11 တွင် 59 ခု) သတ်မှတ်ပြီး Windows XP နှင့်
Server 2003 တွင် category အလိုက် သတ်မှတ်သည်။ တစ်ခုစီသည် အောင်မြင်မှုများ၊ မအောင်မြင်မှုများ၊ နှစ်မျိုးစလုံး သို့မဟုတ်
ဘာမျှ မှတ်တမ်းမတင်ခြင်း ဖြစ်သည်။

LSA သည် `auditpol`၊ local security policy သို့မဟုတ် Group Policy မှ လာသည်ဖြစ်စေ အသက်ဝင်နေသော မူဝါဒကို `SECURITY`
hive တွင် သိမ်းဆည်းထားသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> hive ကို နောက်ဆုံးရေးသားခဲ့ချိန်တွင် စနစ်ကို
ဤ event များ မှတ်တမ်းတင်ရန် (သို့မဟုတ် မတင်ရန်) သတ်မှတ်ထားခဲ့သည်။ Subcategory တစ်ခုကို audit မလုပ်ထားသည့်အခါ
event မရှိခြင်းသည် လှုပ်ရှားမှု ဖြစ်ပေါ်ခဲ့ခြင်း ရှိ/မရှိနှင့် ပတ်သက်၍ ဘာမျှ မပြောပါ။

## တည်နေရာ { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista နှင့် နောက်ပိုင်းတွင် 12-byte header တစ်ခု၊ ထို့နောက် category အလိုက် subcategory တစ်ခုစီအတွက်
ဆက်တင်တစ်ခု၊ ထို့နောက် category တစ်ခုစီတွင် subcategory မည်မျှ ပါဝင်သည်ကို သိမ်းဆည်းထားသည်။ ဗားရှင်းများသည် ထိုအရေအတွက်များ၌သာ
ကွဲပြားသည်- တစ်ခုစီက category တစ်ခု၏ အဆုံးတွင် subcategory များ ထပ်ထည့်သည် (ဖတ်ခဲ့သော Windows 8.x နှင့်
Server 2012 R2 hive များတွင် 56 ခု၊ Windows 10၊ 11 နှင့် Server 2022 တွင် 59 ခု)။ Windows XP နှင့် Server 2003
တွင် auditing ဖွင့်ထား/မထား (byte တစ်ခု၊ ထို့နောက် Windows က မရှင်းလင်းသော padding) နှင့် category ဆက်တင် ကိုးခုကို
သိမ်းဆည်းထားသည်။ ထို layout ကို ဘာမျှ audit မလုပ်ထားသော NIST ၏ CFReDS Hacking Case ၏ အများပြည်သူဆိုင်ရာ
Windows XP image ပေါ်တွင် စစ်ဆေးခဲ့သည်။ ၎င်း၏ category ဆက်တင်များ၏ တန်ဖိုးများသည် Microsoft ၏ စာရွက်စာတမ်းအတိုင်း
ဖြစ်သည်။

`SECURITY` ကို ၎င်း၏ transaction log များနှင့်အတူ စုဆောင်းပြီး host အမည်ကို ဖော်ပြသော စနစ်၏ `SYSTEM` hive ကိုလည်း
စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **မသိသော ဒေတာသည် မသိသည့်အတိုင်း ရှိနေသည်။** ပံ့ပိုးမထားသော revision များကို diagnostic များနှင့်အတူ ထိန်းသိမ်းထားသည်။
  မသိသော ဆက်တင် flag များသည် `Success` နှင့် `Failure` ကို ဗလာထားပြီး audit လုပ်ထားသော ဆက်တင် အရေအတွက်များမှ
  ချန်လှပ်ထားသည်။ ပျက်စီးနေသော အရေအတွက်ဇယားသည် reserved word ကို အခြား audit ဆက်တင်အဖြစ် မြှင့်တင်၍ မရပါ။
  ဖမ်းယူထားသော ဒေတာများတွင် အမည်/GUID catalogue ထက်ကျော်လွန်သော slot အပိုများ ပါဝင်နိုင်သည်။ ၎င်းတို့၏ ဆက်တင်များနှင့်
  ဂဏန်း `SubcategoryId` ကို ဗလာ အမည်/GUID များဖြင့် ထိန်းသိမ်းထားသည်။ mapping မရှိသော slot တစ်ခုသည် ၎င်းကိုယ်တိုင်
  ပျက်စီးမှု၏ အထောက်အထား မဟုတ်ပါ။


- **အစီအစဉ်သည် Windows ၏ ကိုယ်ပိုင်ဖြစ်သည်။** Category တစ်ခုအတွင်း ဆက်တင်များသည် subcategory များ၏ GUID အစီအစဉ်
  မဟုတ်ဘဲ ၎င်းတို့၏ message ID အစီအစဉ်အတိုင်း ရှိသည်။ Special Logon (`%%12548`၊ Logon/Logoff ၏ index 4) သည်
  IPsec Quick Mode ရှေ့တွင် ရှိသည်။ ၎င်းတို့ကို GUID အစီအစဉ်ဖြင့် ဖတ်သော tool များသည် Windows 10 ၏ ပုံသေတန်ဖိုးကို
  Special Logon "No Auditing" နှင့် IPsec Quick Mode "Success" ဟု ဖော်ပြကြသည်။ `SubcategoryId` သည် event 4719
  (System audit policy was changed) က ပေးသော ID နှင့် တူညီသောကြောင့် နှစ်ခုကို တိုက်ဆိုင်နိုင်သည်။
- **အသက်ဝင်နေသော မူဝါဒဖြစ်ပြီး ၎င်း၏ ရင်းမြစ်မဟုတ်ပါ။** Group Policy နှင့် local policy တို့သည် တူညီသော တန်ဖိုးကို
  ရေးသည်။ Group Policy ၏ `audit.csv` (SYSVOL နှင့် `C:\Windows\System32\GroupPolicy` တွင်) သည် အသုံးချရန်
  ရည်ရွယ်ခဲ့သည့်အရာကို ပြသည် (policy တစ်ခု၏ `Machine\Microsoft\Windows NT\Audit\audit.csv`)။
- **အချိန်ကြာလာသည်နှင့်အမျှ ပြောင်းလဲမှုများ-** `KeyLastWrittenUTC` သည် မူဝါဒ နောက်ဆုံးပြောင်းလဲခဲ့သည့် အချိန်ခန့်မှန်း
  ဖြစ်သည်။ Event 4719 သည် ပြောင်းလဲမှုတစ်ခုစီကို မှတ်တမ်းတင်သည် (Security log က ၎င်းကို ထိန်းသိမ်းထားသရွေ့)။
  တိုက်ခိုက်သူတစ်ဦးသည် auditing ကို ပိတ်ပြီးနောက် ပြန်ဖွင့်နိုင်သည်။
- **Subcategory များနှင့် category များ။** Windows Vista နှင့် နောက်ပိုင်းတွင် Windows အသုံးပြုသည်မှာ subcategory
  များဖြစ်သည် (`SCENoApplyLegacyAuditPolicy` security option ကို ပိတ်ထားပြီး legacy category policy က ၎င်းတို့ကို
  အစားထိုးထားခြင်း မရှိလျှင်)။
- **Windows XP နှင့် 2003-** auditing ပိတ်ထားလျှင် (`AuditingEnabled` False) ၎င်း၏ ဆက်တင်မည်သို့ပင်ရှိစေ မည်သည့်
  category ကိုမျှ audit မလုပ်ပါ။
- **Event များ-** `Events` သည် advanced audit policy ဆက်တင်များဆိုင်ရာ Microsoft ၏ စာရွက်စာတမ်းမှ subcategory
  တစ်ခုစီ၏ အဓိက event များကို ဖော်ပြသည်။ ၎င်းတို့သည် စုံစမ်းစစ်ဆေးမှုတစ်ခုက ပုံမှန်ရှာဖွေလေ့ရှိသည့်အရာများဖြစ်ပြီး
  subcategory က မှတ်တမ်းတင်သော event တိုင်း မဟုတ်ပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

Hive တစ်ခုစီအတွက် အနှစ်ချုပ်သည် ၎င်း၏ layout၊ audit လုပ်ထားသော ဆက်တင် အရေအတွက်နှင့် မူဝါဒကို နောက်ဆုံးရေးသားခဲ့သည့်
အချိန်ကို ပေးသည်။ ၎င်းသည် အောင်မြင်မှုနှင့် မအောင်မြင်မှု နှစ်မျိုးစလုံး၊ အောင်မြင်မှုသာ၊ မအောင်မြင်မှုသာ audit လုပ်ထားသည့်အရာများကို
ဖော်ပြပြီးနောက် audit မလုပ်ထားသော subcategory များကို ၎င်းတို့၏ အဓိက event များနှင့်အတူ ဖော်ပြသည်- ထို event များကို
ဖမ်းယူထားသော မူဝါဒတွင် ပိတ်ထားသည်။ ၎င်းသည် ယခင်လှုပ်ရှားမှုများအတွင်း မူဝါဒကို သက်သေမပြပါ။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `SourcePath` | Host (စနစ်၏ `SYSTEM` hive မှ) နှင့် `SECURITY` hive။ |
| `Layout` | `subcategories` (Windows Vista နှင့် နောက်ပိုင်း) သို့မဟုတ် `categories` (XP နှင့် 2003)။ တန်ဖိုးကို decode မလုပ်နိုင်သည့်အခါ ဗလာ။ |
| `Category`, `Subcategory` | `auditpol` က အမည်ပေးသည့်အတိုင်း။ |
| `SubcategoryGUID`, `SubcategoryID` | Subcategory ၏ GUID နှင့် event 4719 က ပေးသည့်အတိုင်း ၎င်း၏ message ID (`%%12548`)။ |
| `Setting` | `none`, `success`, `failure` သို့မဟုတ် `success_and_failure`။ |
| `Success`, `Failure` | အောင်မြင်မှုများနှင့် မအောင်မြင်မှုများကို မှတ်တမ်းတင်ခြင်း ရှိ/မရှိ။ |
| `AuditingEnabled` | Windows XP နှင့် 2003- auditing ကို လုံးဝ ဖွင့်ထားခြင်း ရှိ/မရှိ။ |
| `Events` | Subcategory ၏ အဓိက event ID များ။ |
| `KeyLastWrittenUTC` | `PolAdtEv` ၏ နောက်ဆုံးရေးသားချိန်- မူဝါဒ နောက်ဆုံးပြောင်းလဲခဲ့သည့် အချိန်ခန့်မှန်း။ |
| `DataRaw` | တန်ဖိုးကို အပြည့်အဝ decode မလုပ်နိုင်သည့်အခါ ၎င်း၏ byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | မသိသော layout သို့မဟုတ် အရွယ်အစား၊ ကိုက်ညီမှုမရှိသော အရေအတွက်များနှင့် 3 ထက်ကျော်သော တန်ဖိုးများ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. တစ်စုံတစ်ရာ မဖြစ်ပေါ်ခဲ့ဟု ကောက်ချက်မချမီ ၎င်း၏ subcategory ကို audit လုပ်ထားခြင်း ရှိ/မရှိ စစ်ဆေးပါ-
   Process Creation (4688)၊ Logon (4624, 4625)၊ Special Logon (4672)၊ Other Object Access Events
   (scheduled task များ၊ 4698)၊ Security System Extension (service ထည့်သွင်းမှုများ၊ 4697)၊ User Account နှင့်
   Security Group Management (4720, 4732)၊ File Share (5140)၊ Kerberos (4768, 4769)။
2. မူဝါဒကို Security log နှင့် နှိုင်းယှဉ်ပါ- ယခု audit မလုပ်ထားသော subcategory များ၏ event များ ရှိနေခြင်းသည်
   မူဝါဒ ပြောင်းလဲခဲ့ကြောင်း ဆိုလိုသည်။ Event 4719 နှင့် မကြာသေးမီက `KeyLastWrittenUTC` ကို ရှာဖွေပါ။
3. Domain တစ်ခုလုံးဆိုင်ရာ မူဝါဒကို local ပြောင်းလဲမှုမှ ခွဲခြားရန် Group Policy ၏ `audit.csv` နှင့် နှိုင်းယှဉ်ပါ။
4. `PolAdtEv` ၏ ယခင်ဗားရှင်းများအတွက် `parse-registry read --recover-deleted` နှင့် transaction log များကို
   အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Advanced security audit policy ဆက်တင်များ](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: Auditing constants (subcategory GUID များ)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
