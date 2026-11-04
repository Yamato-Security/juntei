# Boot sector (`$Boot`) { #boot-sector-boot }

## `$Boot` ဆိုသည်မှာ { #what-boot-is }

`$Boot` သည် NTFS volume တိုင်း၏ MFT entry 7 ဖြစ်သည်- volume ၏ ပထမ 8 KiB ဖြစ်သည်။ ၎င်း၏ ပထမ sector ဖြစ်သော
**volume boot record** တွင် ဖိုင်စနစ်ကို ဖော်ပြသည့် BIOS parameter block ပါဝင်သည်-

- sector နှင့် cluster အရွယ်အစားများ၊ volume အတွင်းရှိ sector အရေအတွက်နှင့် **hidden sector များ** (disk ပေါ်တွင်
  volume မတိုင်မီရှိသော sector များ၊ ဆိုလိုသည်မှာ ၎င်း၏ partition စတင်သည့်နေရာ)။
- `$MFT` နှင့် `$MFTMirr` စတင်သည့်နေရာနှင့် FILE နှင့် INDX record များ၏ အရွယ်အစား။
- volume ကို format လုပ်သည့်အခါ သတ်မှတ်သော 64-bit **volume serial number**။

ပထမ sector ၏ ကျန်အပိုင်းနှင့် ၎င်းနောက်ရှိ sector 15 ခုတွင် Windows ကို load လုပ်သော **boot code** ပါဝင်သည်။

Serial number ၏ အောက်ပိုင်း 32 bit သည် Windows က `XXXX-XXXX` (`vol`, `dir`) အဖြစ် ပြသသည့်အရာဖြစ်ပြီး
[LNK ဖြတ်လမ်းများ](lnk.md)၊ [Jump Lists](jump-lists.md) နှင့် [Prefetch](prefetch.md) တို့က ဖိုင်တစ်ခု
ရှိခဲ့သော volume အကြောင်း မှတ်တမ်းတင်သည့်အရာ ဖြစ်သည်။ `$Boot` သည် ထိုမှတ်တမ်းများကို စုဆောင်းထားသော host ၏ volume
တစ်ခုနှင့် ချိတ်ဆက်ပေးသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> `$Boot` ၏ `VolumeSerialNumber` နှင့် တူညီသော
LNK၊ Jump List သို့မဟုတ် Prefetch volume serial number သည် ထို volume ကို ညွှန်ပြသည်၊ သို့မဟုတ် တူညီသော serial number
ပေးထားခြင်း သို့မဟုတ် ၎င်းနှင့်အတူ ကူးယူထားခြင်းခံရသော အခြား volume တစ်ခုကို ညွှန်ပြသည်။ Serial number များကို
ပြောင်းလဲနိုင်ပြီး host များတစ်လျှောက် တစ်မူထူးခြားခြင်း မရှိပါ။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

၎င်းကို raw ဒေတာ ဖတ်နိုင်သော forensic tool ဖြင့် စုဆောင်းပါ၊ သို့မဟုတ် volume image ၏ ပထမ sector ကို ဖတ်ပါ- နှစ်ခုစလုံးသည်
တူညီသော byte များ ဖြစ်သည်။ Juntei သည် NTFS boot sector ဖြင့် စတင်သော မည်သည့်ဖိုင်ကိုမဆို ဖတ်သောကြောင့် volume image ကို
input အဖြစ် အသုံးပြုနိုင်သည်။ Disk image သည် ယင်းအစား partition table ဖြင့် စတင်သည်- Juntei ကို partition ၏ volume ကို
ပေးပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- Volume ကို format လုပ်သည့်အခါ serial number ပြောင်းလဲသည်။ လက်ရှိ volume တစ်ခုနှင့်မျှ serial number မကိုက်ညီသော
  LNK ဖိုင်တစ်ခုသည် removable media၊ network share ၏ server သို့မဟုတ် နောက်ပိုင်းတွင် format လုပ်ခဲ့သော volume ကို
  ညွှန်ပြနိုင်သည်။
- တူညီသော Windows က format လုပ်ထားသော volume များပေါ်တွင် boot code သည် တူညီသည်- DFIR Artifact Museum ၏
  Windows 10၊ 11 နှင့် Server 2012 R2 `$Boot` ဖိုင်များပေါ်တွင် ၎င်းသည် ထပ်တူဖြစ်သည်။ Volume boot record bootkit
  တစ်ခုသည် ၎င်းကို ပြောင်းလဲသည်၊ သို့မဟုတ် boot code က `$Boot` ၏ ကျန်အပိုင်းကို ရှာဖွေရန် အသုံးပြုသော hidden sector
  field ကို ပြောင်းလဲသည်။ မည်သည့်ကောက်ချက်မျှ မချမီ `BootCodeSHA256`, `JumpInstruction` နှင့် `HiddenSectors` ကို
  အခြား host များနှင့်လည်းကောင်း၊ partition table နှင့်လည်းကောင်း နှိုင်းယှဉ်ပါ- boot manager သို့မဟုတ် encryption
  ဆော့ဖ်ဝဲကလည်း boot code ကို ပြောင်းလဲနိုင်သည်။
- BitLocker ဖြင့် encrypt လုပ်ထားသော volume ၏ image သည် BitLocker ၏ ကိုယ်ပိုင် boot sector (OEM ID
  `-FVE-FS-`) ဖြင့် စတင်သည်။ `-f` ဖြင့် ပေးလျှင် Juntei က ၎င်းကို ဖော်ပြပြီး `Problems` က ၎င်းသည် NTFS မဟုတ်ကြောင်း
  ဖော်ပြသည်။ NTFS boot sector သည် encrypt လုပ်ထားသော ဒေတာအတွင်းတွင် ရှိသည်။ Unlock လုပ်ထားသော volume မှ စုဆောင်းထားသော
  `$Boot` သည် NTFS boot sector ဖြစ်သည်။
- `Problems` သည် ပြီးပြည့်စုံသော Windows mount လုပ်နိုင်မှု စစ်ဆေးချက်ကို ပေးခြင်းထက် အသိအမှတ်ပြုထားသော ဖွဲ့စည်းပုံဆိုင်ရာ
  ကွဲလွဲချက်များနှင့် ဖြတ်တောက်ခံရမှုများကို ဖော်ပြသည်။ ၎င်းကို carve လုပ်ထားသော sector များနှင့် မိတ္တူများကို
  အကဲဖြတ်ရန် အသုံးပြုပါ၊ ၎င်းတစ်ခုတည်းကို တိုက်ခိုက်မှု တွေ့ရှိချက်အဖြစ် မသုံးပါနှင့်။ Boot-code hash တွင် ကနဦး jump၊
  BIOS parameter block နှင့် end signature မပါဝင်ပါ- ထို field များကိုလည်း စစ်ဆေးပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

အနှစ်ချုပ်တွင် boot sector တစ်ခုစီ၏ volume serial number (Windows ပြသသည့်အတိုင်းနှင့် LNK နှင့် Prefetch field များ
ပြသသည့်အတိုင်း)၊ အရွယ်အစား၊ cluster အရွယ်အစားနှင့် `$MFT` cluster ကို ဖော်ပြသည်။ Host များစွာကို ခွဲခြမ်းစိတ်ဖြာသည့်အခါ
၎င်းတို့ကို boot code hash အလိုက် အုပ်စုဖွဲ့သောကြောင့် boot code ကွဲပြားသော volume သည် ထင်ရှားစွာ ပေါ်လာသည်။

## CSV ထုတ်ယူမှု field များ { #csv-output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `SourcePath` | Input ဖိုင်။ |
| `VolumeSerialNumber` | LNK၊ Jump List နှင့် Prefetch output ပြသသည့်အတိုင်း 32-bit serial number (`180B1E67`)။ Windows က ၎င်းကို `180B-1E67` အဖြစ် ပြသသည်။ |
| `VolumeSerialNumber64` | 64-bit serial number အပြည့်အစုံ။ |
| `OEM-ID` | NTFS volume ပေါ်တွင် `NTFS`။ |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | Volume ၏ geometry။ 128 ထက်ကျော်သော cluster တစ်ခုလျှင် sector အရေအတွက်ကို နှစ်၏ power အဖြစ် မှတ်တမ်းတင်ထားပြီး ဤနေရာတွင် decode လုပ်ထားသည်။ မမှန်ကန်ပါက ဗလာ။ |
| `TotalSectors`, `VolumeSize` | Volume အတွင်းရှိ sector များနှင့် ၎င်းတို့၏ byte အရွယ်အစား၊ ၎င်း၏ အဆုံးရှိ boot sector မိတ္တူ မပါဘဲ။ |
| `HiddenSectors` | Disk ပေါ်တွင် volume မတိုင်မီရှိ sector များ- ၎င်း၏ partition စတင်သည့်နေရာ။ |
| `MFT-Cluster`, `MFT-Offset` | `$MFT` စတင်သည့်နေရာ၊ cluster ဖြင့်နှင့် volume အစမှ byte ဖြင့်။ |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | `$MFTMirr` စတင်သည့်နေရာ။ |
| `FileRecordSize`, `IndexRecordSize` | FILE record တစ်ခုလျှင် byte (ပုံမှန် 1,024) နှင့် INDX record တစ်ခုလျှင် byte (ပုံမှန် 4,096)။ |
| `MediaDescriptor` | Hard disk အတွက် `F8`။ |
| `JumpInstruction` | ပထမ byte သုံးခု (`EB 52 90`)- BIOS parameter block ကို ကျော်၍ boot code သို့ jump လုပ်ခြင်း။ |
| `EndSignature` | မှန်ကန်သော boot sector ၏ အဆုံးရှိ `55 AA`။ |
| `BootCodeSHA256` | Boot code ၏ SHA-256- ပထမ sector ၏ byte 0x54 မှ 0x1FD အထိနှင့် 8 KiB ၏ ကျန်အပိုင်း။ |
| `Problems` | Windows က mount မလုပ်မည့်အရာ- `NTFS` မဟုတ်သော OEM ID၊ `55 AA` မရှိခြင်း၊ မမှန်ကန်သော အရွယ်အစားများ၊ NTFS က သုညအဖြစ် ထားသော FAT field များ သတ်မှတ်ထားခြင်း၊ volume ၏ အဆုံးကို ကျော်လွန်သော `$MFT`၊ နှင့် 8 KiB ထက်တိုသော ဖိုင်။ |

JSON/JSONL တွင် တူညီသော field နှင့် အမည်များ ပါဝင်ပြီး ဂဏန်းများကို ဂဏန်းအဖြစ်၊ မသိသော တန်ဖိုးများကို `null` အဖြစ်နှင့်
`Problems` ကို စာရင်းအဖြစ် ထားသည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. Host တစ်ခုစီ၏ `VolumeSerialNumber` ကို ၎င်း၏ LNK ဖိုင်များ၊ Jump Lists နှင့် Prefetch ဖိုင်များ၏ volume serial
   number များနှင့် ယှဉ်ထားပါ- စုဆောင်းထားသော volume တစ်ခုနှင့်မျှ မကိုက်ညီသည့်အရာများသည် အခြား volume များ (USB
   drive များ၊ share များ၊ နောက်ပိုင်းတွင် format လုပ်ခဲ့သော volume များ) ဖြစ်သည်။
2. အခြား serial number များကို mount လုပ်ခဲ့သော removable volume များ၏ volume serial number များကို ဖော်ပြသည့်
   [USB စက်ပစ္စည်းများ](usb.md) output တွင် ရှာဖွေပါ။
3. `BootCodeSHA256` နှင့် `HiddenSectors` ကို host များတစ်လျှောက် နှိုင်းယှဉ်ပါ။ ကွဲပြားသော boot code ကို
   disassembler နှင့် disk ၏ partition table ဖြင့် စုံစမ်းစစ်ဆေးပြီး boot နှင့်ဆက်စပ်သော event log entry များကို ရှာဖွေပါ။
4. Volume image တွင် `$MFT` ကို ရှာဖွေရန် `MFT-Offset` နှင့် `ClusterSize` ကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: NTFS ခြုံငုံသုံးသပ်ချက်](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: NTFS format စာရွက်စာတမ်း](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (volume header)
- [The Sleuth Kit: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (၎င်း၏ `boot` crate)
