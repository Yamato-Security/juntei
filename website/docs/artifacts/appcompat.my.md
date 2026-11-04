# အပလီကေးရှင်း လိုက်ဖက်မှု { #application-compatibility }

## Windows က သိမ်းဆည်းထားသည့်အရာများ { #what-windows-keeps }

ShimCache (`parse-registry shimcache`) နှင့် Amcache (`parse-registry amcache`) အပြင် Windows ၏
application compatibility လုပ်ဆောင်ချက်များသည် အောက်ပါတို့ကို
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags` အောက်တွင် သိမ်းဆည်းထားသည်-

| Source | Hive နှင့် key | ပြသသည့်အရာ |
|---|---|---|
| `layers` | `SOFTWARE` (နှင့် `Wow6432Node`) နှင့် `NTUSER.DAT` `Layers` | Compatibility layer များဖြင့် run ရန် သတ်မှတ်ထားသော ပရိုဂရမ်များ- `RUNASADMIN` (အမြဲ administrator အဖြစ်)၊ `WIN7RTM`၊ `HIGHDPIAWARE`၊ … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 နှင့် နောက်ပိုင်း) | အသုံးပြုသူ run ခဲ့သည်ကို Program Compatibility Assistant က တွေ့ခဲ့သော ပရိုဂရမ်များ |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | အတူတူပင်၊ Windows 7 တွင် |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | Tracking လုပ်ထားသော module တစ်ခုကို load လုပ်ခဲ့သော ပရိုဂရမ်များနှင့် ၎င်းတို့ နောက်ဆုံး load လုပ်ခဲ့သည့်အချိန် |
| `cit_system` | `SOFTWARE` `CIT\System` (Windows 7 မှ အစောပိုင်း Windows 10 အထိ) | ချုံ့ထားသော CIT database များရှိ ပရိုဂရမ် path များနှင့် command line များ |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | `sdbinst` ဖြင့် ထည့်သွင်းထားသော စိတ်ကြိုက် shim database များ- ဖိုင်၊ ဖော်ပြချက်၊ ထည့်သွင်းချိန် |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | ပရိုဂရမ်တစ်ခုချင်းစီတွင် အသုံးချထားသော စိတ်ကြိုက် shim database များ၊ ၎င်းတို့၏ ထည့်သွင်းချိန်များနှင့်အတူ |

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> layer များနှင့် shim database များသည် ပရိုဂရမ်
run ပုံကို ပြောင်းလဲစေသော configuration ဖြစ်သည်။ Program Compatibility Assistant နှင့် CIT entry များက ပရိုဂရမ်တစ်ခုကို
ဤအသုံးပြုသူအဖြစ် သို့မဟုတ် ဤစနစ်ပေါ်တွင် run ခဲ့ကြောင်း (သို့မဟုတ် module တစ်ခုကို load လုပ်ခဲ့ကြောင်း) ပြသသည်။

စိတ်ကြိုက် shim database များသည် အမြဲတမ်းဝင်ရောက်နိုင်မှု (persistence) နှင့် injection နည်းလမ်းတစ်ခု ဖြစ်သည်
(MITRE ATT&CK T1546.011)- shim တစ်ခုသည် ပရိုဂရမ် run တိုင်း ထိုပရိုဂရမ်ထဲသို့ DLL တစ်ခုကို load လုပ်နိုင်သည် သို့မဟုတ်
အခြားပရိုဂရမ်တစ်ခုကို စတင်ဖွင့်နိုင်သည်။

## တည်နေရာ { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

`SOFTWARE`၊ `NTUSER.DAT` အားလုံးနှင့် ၎င်းတို့၏ transaction log များကို စုဆောင်းပြီး shim database ၏ path က
အမည်ပေးထားသော `.sdb` ဖိုင်များ (များသောအားဖြင့် `C:\Windows\AppPatch\Custom\`) ကိုလည်း စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **CIT ချဲ့ထွင်မှုကို ကန့်သတ်ထားသည်။** ချုံ့မထားသော အရွယ်အစား 64 MiB ထက်ပို၍ ကြေညာထားသော value များကို
  diagnostic တစ်ခုနှင့်အတူ raw byte များအဖြစ် ထိန်းသိမ်းထားသည်။ ပုံစံမမှန်သော compression နှင့် အရွယ်အစား မကိုက်ညီမှုများကို အစီရင်ခံသည်။


- **Program Compatibility Assistant entry များအတွက် run ချိန် မရှိ။** `Store` ဒေတာ (`SACP`၊ `DataRaw` တွင်
  ထိန်းသိမ်းထားသည်) တွင် အချိန်တစ်ခုရှိသော်လည်း Windows 8.1၊ 10 နှင့် 11 တွင် hive တစ်ခု၏ ပရိုဂရမ်အားလုံးအတွက်
  တူညီပြီး အခြားစနစ်များအတွက်လည်း ထိုနည်းတူပင်- ၎င်းသည် ပရိုဂရမ် run ခဲ့သည့်အချိန် မဟုတ်ပါ။ `Store` key ၏
  နောက်ဆုံးရေးသားချိန်သာ နောက်ဆုံးပြောင်းလဲမှုကို ရက်စွဲသတ်မှတ်ပေးသည်။
- **Removable media ပေါ်ရှိ ပရိုဂရမ်များ** ကို `SIGN.MEDIA=<id> <path>` ဟု အမည်ပေးထားသည်- `Path` သည် media
  ပေါ်ရှိ path ဖြစ်ပြီး `Details` တွင် media id ပါဝင်သည်။
- **CIT database များ** ကို LZNT1 ဖြင့် ချုံ့ထားပြီး ၎င်းတို့၏ ဖွဲ့စည်းပုံကို အများသိ မှတ်တမ်းမပြုထားပါ- Juntei သည်
  database တစ်ခုစီတွင် တွေ့ရှိသော ပရိုဂရမ် path များနှင့် command line များကို database တစ်ခုလျှင် တစ်ကြိမ်စီ၊
  အရေအတွက် သို့မဟုတ် အချိန်မပါဘဲ ဖော်ပြသည်။ Command line တစ်ခုသည် ၎င်း၏ ကိုယ်ပိုင် record ဖြစ်ပြီး ၎င်း၏
  ပရိုဂရမ်နှင့် ချိတ်ဆက်မထားပါ။ Windows 10 နှင့် နောက်ပိုင်းတွင် ဤနေရာ၌ အနည်းငယ်သာ သိမ်းဆည်းသည်။
- **Shim database များ။** `custom_sdb` နှင့် `installed_sdb` record တိုင်းကို စစ်ဆေးပါ- တရားဝင် software များက
  ၎င်းကို ထည့်သွင်းခဲသည်။ `.sdb` ဖိုင်ကိုယ်တိုင်က shim လုပ်ဆောင်သည့်အရာကို ဖော်ပြသည်။
- အသုံးပြုသူက သတ်မှတ်ထားသော **Layer များ** သည် ထိုအသုံးပြုသူအတွက်သာ သက်ရောက်သည်။ `SOFTWARE` ၏ layer များမူ
  အားလုံးအတွက် သက်ရောက်သည်။ `RUNASADMIN` က ပရိုဂရမ်ကို အကြိမ်တိုင်း elevation တောင်းစေသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

အနှစ်ချုပ်တွင် စိတ်ကြိုက် shim database များ၊ layer များရှိသော ပရိုဂရမ်များနှင့် CIT module များ၏ နောက်ဆုံး
အသုံးပြုမှုများကို ဖော်ပြသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host၊ hive ဖိုင်နှင့် `SOFTWARE` သို့မဟုတ် `NTUSER.DAT`။ |
| `User`, `UserSID` | အသုံးပြုသူ hive ၏ profile ဖိုင်တွဲအမည်နှင့် စနစ်၏ `ProfileList` မှ ၎င်း၏ SID။ |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb`။ `machine` သို့မဟုတ် `user`။ |
| `Path` | ပရိုဂရမ် (value ၏ အမည်)၊ CIT database ရှိ path သို့မဟုတ် command line၊ သို့မဟုတ် shim database အသုံးချသော ပရိုဂရမ်။ |
| `Layers` | ရေးသားထားသည့်အတိုင်း compatibility layer များ (`~ RUNASADMIN`)။ |
| `LastSeenUTC` | ပရိုဂရမ်တစ်ခုက CIT module ကို နောက်ဆုံး load လုပ်ခဲ့သည့်အချိန်။ |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | Shim database ၏ GUID၊ ဖိုင်နှင့် ထည့်သွင်းချိန်။ |
| `Details` | CIT module ၏ အမည်၊ CIT database ၏ value အမည်၊ ပရိုဂရမ်၏ removable media id (`SignMedia`)၊ shim database ၏ `DatabaseDescription` နှင့် `DatabaseType`။ |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | Record လာရာနေရာ၊ ထို key နောက်ဆုံးပြောင်းလဲခဲ့သည့်အချိန်နှင့် `Wow6432Node` အောက်တွင် ရှိ/မရှိ။ |
| `DataRaw` | `Store` value ၏ `SACP` ဒေတာ သို့မဟုတ် ဖြစ်သင့်သည့်အတိုင်း မဟုတ်သော ဒေတာ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | Decode မလုပ်နိုင်သော သို့မဟုတ် မဖတ်နိုင်သော ဒေတာ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. စိတ်ကြိုက် shim database တိုင်းကို ပြန်လည်သုံးသပ်ပါ- ၎င်း၏ `.sdb` ဖိုင်၊ ထည့်သွင်းချိန်နှင့် ၎င်းအသုံးချသော
   ပရိုဂရမ်များ။ Prefetch၊ Amcache နှင့် process ဖန်တီးမှု event များရှိ `sdbinst.exe` နှင့် ချိတ်ဆက်စစ်ဆေးပါ။
2. Program Compatibility Assistant entry များထဲတွင် အသုံးပြုသူ ရေးသားနိုင်သော ဖိုင်တွဲများရှိ ပရိုဂရမ်များနှင့်
   removable media မှ ပရိုဂရမ်များကို ရှာဖွေပါ။
3. ပုံမှန်မဟုတ်သော ပရိုဂရမ်များပေါ်ရှိ `RUNASADMIN` နှင့် အခြား layer များကို စစ်ဆေးပါ။
4. လုပ်ဆောင်ချိန်များအတွက် ShimCache၊ Amcache၊ BAM၊ Prefetch နှင့် UserAssist တို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Application compatibility database](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
