# လုံခြုံရေးဖော်ပြချက်များ (`$Secure:$SDS`) { #security-descriptors-securesds }

## `$SDS` ဆိုသည်မှာ { #what-sds-is }

NTFS ပေါ်ရှိ ဖိုင်နှင့် directory တိုင်းတွင် **security descriptor** တစ်ခု ရှိသည်- ၎င်း၏ ပိုင်ရှင်၊ ၎င်း၏ primary group၊
**DACL** (၎င်းနှင့် မည်သူက မည်သည်ကို လုပ်ဆောင်နိုင်သည်) နှင့် **SACL** (မည်သည့် ဝင်ရောက်မှုကို audit လုပ်သည်နှင့် ၎င်း၏ integrity label)။
NTFS သည် ကွဲပြားသော descriptor တစ်ခုစီကို `$Secure` ဖိုင် (MFT entry 9) ၏ `$SDS` stream တွင် တစ်ကြိမ်သာ သိမ်းထားပြီး
၎င်းကို **security ID** တစ်ခု ပေးသည်။ ဖိုင်တစ်ခု၏ `$STANDARD_INFORMATION` သည် ထို security ID ကိုသာ မှတ်တမ်းတင်သည်-
[`parse-ntfs mft`](mft.md) ၏ `SecurityID` column ဖြစ်သည်။ `$SDS` က ၎င်းကို ပိုင်ရှင်နှင့် ACL များအဖြစ် ပြောင်းလဲပေးသည်။

`$SDS` သည် 256 KiB block များဖြင့် ဖွဲ့စည်းထားပြီး block တစ်ခုစီနောက်တွင် ၎င်း၏ မိတ္တူ (mirror) ပါရှိသည်။ Entry တစ်ခုစီတွင်
၎င်း၏ descriptor ၏ hash (NTFS က ရှိပြီးသား descriptor ကို ရှာရန် အသုံးပြုသည် (`$SDH`)) နှင့် security ID (`$SII` က
၎င်းကို index လုပ်သည်) ပါဝင်သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> `SecurityID` က descriptor တစ်ခုကို ညွှန်းသော ဖိုင်များတွင်
၎င်းတို့၏ `$MFT` record ကို နောက်ဆုံးရေးသားခဲ့ချိန်က ထိုပိုင်ရှင်နှင့် ထို ACL များ ရှိခဲ့သည်။ ဖိုင်တစ်ခု၏ ပိုင်ရှင်သည်
အများအားဖြင့် ၎င်းကို ဖန်တီးခဲ့သော အကောင့်ဖြစ်သော်လည်း administrator၊ installer သို့မဟုတ် ပိုင်ဆိုင်မှုကို ရယူသော ပရိုဂရမ်တစ်ခုက
၎င်းကို ပြောင်းလဲနိုင်သည်။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

၎င်းကို raw-aware မှုခင်းဆိုင်ရာ tool ဖြင့် စုဆောင်းပါ။ `:` သည် ဖိုင်အမည်တွင် မပါဝင်နိုင်သောကြောင့် စုဆောင်းမှုများတွင်
၎င်းကို အများအားဖြင့် `$Secure_$SDS` ဟု အမည်ပေးကြသည်။ တူညီသော volume ၏ `$MFT` ကိုလည်း ၎င်းနှင့်အတူ စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- ပိုင်ရှင်သည် SID တစ်ခု ဖြစ်သည်။ လူသိများသော SID များ (`NT AUTHORITY\SYSTEM`, `BUILTIN\Administrators`,
  `NT SERVICE\TrustedInstaller`, …) နှင့် စက် သို့မဟုတ် domain ၏ built-in အကောင့်များ (`-500`
  `Administrator`, …) ကိုသာ အမည်ဖော်ပြသည်။ ကျန်သည်များကို host ၏ `SAM`
  ([SAM အကောင့်များ](sam.md))၊ ၎င်း၏ profile စာရင်း ([စနစ်အချက်အလက်](system-info.md)) သို့မဟုတ် domain ဖြင့် ဖြေရှင်းပါ။
- Descriptor တစ်ခုကို တူညီသော ပိုင်ရှင်နှင့် ACL များရှိသည့် ဖိုင်တိုင်းက မျှဝေအသုံးပြုပြီး ထိုဖိုင်များကို ဖျက်ပြီးနောက်တွင်လည်း
  အများအားဖြင့် `$SDS` တွင် ဆက်ရှိနေသည်။ `$MFT` တွင် ဖိုင်မရှိသော descriptor တစ်ခုသည် ၎င်းတစ်ခုတည်းဖြင့်
  ဖျက်ထားသော ဖိုင်တစ်ခု မဟုတ်ပါ။
- ACE သည် ဝင်ရောက်မှု သမိုင်းကြောင်း မဟုတ်ပါ။ DACL က ဖိုင်ကို မည်သူ ဝင်ရောက် *ခွင့်ရှိ* သည်ကို ပြောပြီး event log များနှင့်
  အခြား အထောက်အထားများက မည်သူ ဝင်ရောက်ခဲ့သည်ကို ပြောသည်။ SACL ၏ audit entry များသည် ပြင်ဆင်သတ်မှတ်ချက်ကို ဖော်ပြပြီး
  log လုပ်ခဲ့သည့်အရာကို မဖော်ပြပါ။
- ACE များကို ACL အတွင်းရှိ ၎င်းတို့၏ အစီအစဉ်အတိုင်း စာရင်းပြထားသည်။ Juntei သည် ထိရောက်သော ဝင်ရောက်ခွင့်ကို
  မတွက်ချက်ပါ- deny entry များ၊ inheritance၊ group အဖွဲ့ဝင်ဖြစ်မှု၊ privilege များနှင့် ပိုင်ရှင်၏ သွယ်ဝိုက်သော အခွင့်အရေးများ
  အားလုံး ထည့်တွက်ရသည်။
- `DACL` နှင့် `SACL` သည် ACE တစ်ခုစီကို `icacls` ကဲ့သို့ ရေးသားသည်။ `SDDL` သည် ပံ့ပိုးထားသော ခွင့်ပြုချက်များကို mask များကို hexadecimal ဖြင့်လည်းကောင်း၊ အကောင့်များကို SID အဖြစ်လည်းကောင်း ဖော်ပြသည်။ ၎င်းသည် binary descriptor ၏ byte တစ်ခုချင်းတူညီသော မိတ္တူ မဟုတ်ပါ။ ဖော်ပြ၍မရသော conditional စာသားရှိပါက ၎င်းကို ဗလာထားပြီး diagnose လုပ်သည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

အနှစ်ချုပ်သည် descriptor များကို ရေတွက်ပြီး ၎င်းတို့၏ ပိုင်ရှင်များ (တစ်ဦးစီ ပိုင်ဆိုင်သော descriptor အရေအတွက်နှင့်အတူ)၊
NULL DACL (မည်သည့် ကန့်သတ်ချက်မျှ မရှိ) ပါသော descriptor များနှင့် ပျက်စီးမှုများကို စာရင်းပြသည်။

## CSV ထုတ်ယူမှု field များ { #csv-output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `SourcePath` | Input ဖိုင်။ |
| `SecurityID` | Security ID- တူညီသော volume အတွက် [`parse-ntfs mft`](mft.md) ၏ `SecurityID` နှင့် join လုပ်ပါ။ |
| `Hash` | Descriptor နှင့်အတူ မှတ်တမ်းတင်ထားသော hash။ |
| `EntryOffset` | Stream အတွင်း entry ရှိသည့်နေရာ။ |
| `FromMirror` | Mirror block မှ ဖတ်ထားသော မိတ္တူအတွက် `True`- ၎င်း၏ entry နှင့် ကွဲပြားသော မိတ္တူ (၎င်းနောက်တွင် စာရင်းပြထားသည်) သို့မဟုတ် ပျက်စီးနေသော entry ၏ မိတ္တူ။ |
| `Size` | ၎င်း၏ 20-byte header အပါအဝင် entry ၏ အရွယ်အစား။ |
| `Owner`, `OwnerName` | ပိုင်ရှင်၏ SID နှင့် လူသိများပါက ၎င်း၏ အမည်။ |
| `Group`, `GroupName` | Primary group ၏ SID နှင့် လူသိများပါက ၎င်း၏ အမည်။ Windows သည် ၎င်းကို ဝင်ရောက်ခွင့် စစ်ဆေးမှုများအတွက် မသုံးပါ။ |
| `Control` | Descriptor ၏ control flag များ- `SE_DACL_PRESENT`, `SE_DACL_PROTECTED` (inheritance ပိတ်ဆို့ထားသည်), `SE_DACL_AUTO_INHERITED`, … ကို `|` ဖြင့် ဆက်ထားသည်။ |
| `DACL-ACE-Count`, `DACL` | `; ` ဖြင့် ခွဲထားသော DACL ၏ entry များ။ DACL မရှိပါက ဗလာ။ Descriptor က လူတိုင်းကို အရာအားလုံး ခွင့်ပြုပါက `NULL DACL: no restriction`။ |
| `SACL-ACE-Count`, `SACL` | SACL ၏ entry များ- audit entry များ၊ mandatory integrity label၊ process trust label များနှင့် resource attribute များ။ |
| `SDDL` | SDDL ဖြင့် ဖော်ပြထားသော descriptor တစ်ခုလုံး။ ၎င်းအတွင်းရှိ တစ်စုံတစ်ရာကို decode မလုပ်နိုင်မှသာ ဗလာဖြစ်သည်။ |
| `Problems` | Descriptor ၏ hash နှင့် မကိုက်ညီသော hash၊ decode မလုပ်နိုင်သော အစိတ်အပိုင်းများ၊ အမျိုးအစား မသိသော ACE များ။ |

`DACL` နှင့် `SACL` ၏ entry တစ်ခုစီကို `icacls` ရေးသားသည့်အတိုင်း ရေးသားထားသည်- အကောင့်၊ allow entry မဟုတ်ပါက
အမျိုးအစား၊ inheritance flag များနှင့် အခွင့်အရေးများ။

| အစိတ်အပိုင်း | အဓိပ္ပာယ် |
|---|---|
| အကောင့် | လူသိများသော အမည် သို့မဟုတ် SID။ |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | Allow entry မဟုတ်သည့်အခါ ACE အမျိုးအစား။ Mandatory label တွင် အမျိုးအစား ပြသထားခြင်း မရှိပါ။ |
| `(I)` | Parent directory မှ အမွေဆက်ခံထားသည်။ |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | ဖိုင်များက အမွေဆက်ခံသည် (object inherit)၊ directory များက အမွေဆက်ခံသည် (container inherit)၊ ၎င်းတို့ကသာ အမွေဆက်ခံပြီး ဤ object မပါ (inherit only) နှင့် တစ်ဆင့်အောက်သာ (no propagate)။ |
| `(SA)`, `(FA)` | အောင်မြင်သော သို့မဟုတ် မအောင်မြင်သော ဝင်ရောက်မှုအတွက် audit entry။ |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | Full control၊ modify၊ read and execute၊ read၊ write။ |
| `(RD,WD,AD,…)` | အခြားအခြေအနေတွင် သီးခြား အခွင့်အရေးများ- read data၊ write data၊ append data၊ extended attribute များ ဖတ်ခြင်းနှင့် ရေးခြင်း (`REA`, `WEA`)၊ execute (`X`)၊ delete child (`DC`)၊ attribute များ ဖတ်ခြင်းနှင့် ရေးခြင်း (`RA`, `WA`)၊ delete (`DE`)၊ read control (`RC`)၊ write DAC (`WDAC`)၊ write owner (`WO`)၊ synchronize (`S`) နှင့် generic အခွင့်အရေးများ `GA`, `GR`, `GW`, `GE`။ `N` သည် ဝင်ရောက်ခွင့် မရှိခြင်း ဖြစ်သည်။ |
| `(NW)`, `(NR)`, `(NX)` | Mandatory label ၏ မူဝါဒ- no write up၊ no read up၊ no execute up။ |
| `if (…)` | `(WIN://SYSAPPID Contains "…")` ကဲ့သို့သော callback ACE ၏ အခြေအနေ- ၎င်းမှန်ကန်သည့် process များအတွက်သာ entry သက်ရောက်သည်။ Windows သည် app container package များကို ဝင်ရောက်ခွင့်ပေးရန် ၎င်းတို့ကို အသုံးပြုသည်။ |

Process trust label များနှင့် resource attribute များသည် ၎င်းတို့၏ mask ကို hexadecimal ဖြင့် ပြသပြီး resource attribute က
၎င်း၏ claim (`("IMAGELOAD",TU,0x0,1)`) ကိုလည်း ပြသသည်။

JSON/JSONL တွင် တူညီသော field များနှင့် အမည်များ ရှိပြီး နံပါတ်များကို နံပါတ်များအဖြစ်၊ ဗလာ value များကို `null` အဖြစ်နှင့်
`Problems` ကို စာရင်းအဖြစ် ဖော်ပြသည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. တူညီသော volume ၏ `$MFT` ကို ခွဲခြမ်းစိတ်ဖြာပြီး ၎င်း၏ `SecurityID` ကို ဤထုတ်ယူမှု၏ security ID နှင့် join လုပ်ပါ- ဖိုင်တစ်ခုစီသည်
   ၎င်း၏ ပိုင်ရှင်နှင့် ACL များကို ရရှိသည်။
2. အသုံးပြုသူအကောင့်များ (`S-1-5-21-…`) ဖြစ်သော ပိုင်ရှင်များကို ကြည့်ပါ- သံသယဖြစ်ဖွယ် အကောင့်တစ်ခုက မည်သည့်ဖိုင်များကို
   ပိုင်ဆိုင်သနည်း၊ system directory များရှိ ဖိုင်များကို မည်သည့်အကောင့်များက ပိုင်ဆိုင်သနည်း။
3. ပရိုဂရမ်များ run သည့် သို့မဟုတ် load လုပ်သည့် directory များပေါ်ရှိ ခွင့်ပြုချက်ကျယ်ပြန့်သော ACL များကို ရှာပါ- `(F)`၊ `(M)` သို့မဟုတ်
   `(W)` ပါသော `Everyone`၊ `Authenticated Users` သို့မဟုတ် `User` နှင့် NULL DACL များ။
4. ဖိုင်တစ်ခုတည်းကသာ အသုံးပြုသော သို့မဟုတ် ပုံမှန်မဟုတ်သော ဖိုင်အစုတစ်ခုက အသုံးပြုသော descriptor တစ်ခုသည် ပုံမှန်
   ထည့်သွင်းမှု၏ ပြင်ပတွင် ဖန်တီးခဲ့သော သို့မဟုတ် ပြောင်းလဲခဲ့သော ဖိုင်များကို ပြသနိုင်သည်။
5. ပိုင်ဆိုင်မှုနှင့် ခွင့်ပြုချက် ပြောင်းလဲမှုများကို Security event log entry များ (4670၊ SACL ပါသော 4663) နှင့်
   [USN ဂျာနယ်](usn-journal.md) ၏ `SECURITY_CHANGE` record များဖြင့် ထပ်မံအတည်ပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (security descriptor များ၊ ACE များ၊ conditional expression များ)
- [Microsoft: Security Descriptor String Format (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: NTFS ပုံစံ စာရွက်စာတမ်း](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (၎င်း၏ `secure` crate)

Structured ထုတ်ယူမှုကို stdout သို့ ပို့သည့်အခါ (`-o` မပါဘဲ `-t`) stream အဆင့် recovery သတိပေးချက်များကို stderr သို့ ပို့သောကြောင့် pipe များသည် ပျက်စီးနေသော သို့မဟုတ် ပြန်လည်ရယူထားသော နေရာများကို ဖုံးကွယ်ခြင်းမရှိဘဲ ဖတ်ရှုနိုင်သော CSV/JSON/JSONL ကို ထိန်းသိမ်းထားသည်။
