# Outlook { #outlook }

## Outlook ၏ ဖိုင်များ ဆိုသည်မှာ { #what-outlooks-files-are }

Windows အတွက် Outlook သည် mail၊ contact များ၊ ပြက္ခဒိန်များနှင့် task များကို ၎င်း၏ ကိုယ်ပိုင် ဖိုင်များတွင် သိမ်းဆည်းပါသည်။

- **OST** (`.ost`): mailbox တစ်ခု၏ offline မိတ္တူ ဖြစ်ပြီး Outlook သည် ၎င်းကို Exchange နှင့် Microsoft 365 account များအတွက်
  လည်းကောင်း၊ Outlook 2013 မှစ၍ IMAP account များအတွက်လည်းကောင်း ထားရှိပါသည်။ Outlook 2013 နှင့် နောက်ပိုင်း၏ OST
  များတွင် 4 KiB page များနှင့် compress လုပ်ထားသော block များ ရှိပါသည်။
- **PST** (`.pst`): personal store တစ်ခု ဖြစ်ပါသည်: POP3 account များ၊ archive များ (`archive.pst`)၊ export များနှင့်
  အသုံးပြုသူများ ဖန်တီးသော store များ။ ANSI PST များသည် Outlook 97 မှ 2002 အထိ၏ PST များ ဖြစ်ပြီး Unicode PST များသည်
  Outlook 2003 နှင့် နောက်ပိုင်း၏ PST များ ဖြစ်ပါသည်။
- **MSG** (`.msg`): Outlook မှ ဆွဲထုတ်ခြင်း (drag)၊ **Save As** ဖြင့် သိမ်းဆည်းခြင်း သို့မဟုတ် အခြား program က export
  လုပ်ခြင်းဖြင့် ဖိုင်အဖြစ် သိမ်းဆည်းထားသော item တစ်ခု ဖြစ်ပါသည်။ ပူးတွဲထားသော message များသည်လည်း MSG ဖိုင်များ ဖြစ်ပါသည်။
- **OFT** (`.oft`): Outlook template တစ်ခု ဖြစ်ပါသည်: item အသစ်များကို ဖန်တီးရာတွင် အသုံးပြုသော MSG ဖိုင်တစ်ခု။

Store တစ်ခုစီတွင် ဖိုင်တွဲများ ရှိပြီး ဖိုင်တွဲတစ်ခုစီတွင် ၎င်း၏ item များ ရှိပါသည်: e-mail၊ meeting request များနှင့် response
များ၊ contact များနှင့် distribution list များ၊ ပြက္ခဒိန် item များ၊ task များ၊ note များ၊ journal entry များနှင့် report များ။
ဖိုင်တွဲတစ်ခုစီသည် ဝှက်ထားသော item များ၊ ဆိုလိုသည်မှာ ၎င်း၏ folder associated information (FAI) ကိုလည်း ထားရှိပါသည်: view
များ၊ rule များ၊ form များနှင့် Outlook ၏ setting များ (Exchange mailbox ၏ autocomplete list ကဲ့သို့သော
`IPM.Configuration.*` item များ)။ Item တစ်ခု၏ ပူးတွဲဖိုင်များသည် ဖိုင်များ၊ ဖိုင်များသို့ ရည်ညွှန်းချက်များ၊ OLE object
များ သို့မဟုတ် အခြား item များ (ပူးတွဲဖိုင်အဖြစ် forward လုပ်ထားသော message၊ ထပ်ခါထပ်ခါ ဖြစ်သော appointment ၏
exception များ) ဖြစ်နိုင်ပြီး ၎င်းတို့တွင်လည်း ၎င်းတို့၏ ကိုယ်ပိုင် ပူးတွဲဖိုင်များ ရှိနိုင်ပါသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> စုဆောင်းချိန်တွင် store သို့မဟုတ် ဖိုင်၌ ဤ ပေးပို့သူများ၊
လက်ခံသူများ၊ အချိန်များနှင့် ပူးတွဲဖိုင်များပါသော item ရှိခဲ့ပါသည်။ အသုံးပြုသူက ၎င်းကို ဖတ်ခဲ့သည်၊ ရေးခဲ့သည် သို့မဟုတ်
ပေးပို့ခဲ့သည်ကို မဖော်ပြသလို ၎င်း၏ ပေးပို့သူသည် item က ဖော်ပြထားသူ အမှန်ပင် ဖြစ်သည်ကိုလည်း မဖော်ပြပါ။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

PST များသည် မည်သည့်နေရာတွင်မဆို ရှိနိုင်ပါသည်: အသုံးပြုသူများသည် archive များကို အခြား ဖိုင်တွဲများ၊ drive များနှင့် file
share များသို့ ရွှေ့တတ်ပါသည်။ Outlook သည် profile တစ်ခုစီ၏ store များ၏ path များကို အသုံးပြုသူ၏ registry တွင် သိမ်းဆည်းပါသည်။
အသုံးပြုသူက Outlook မှ ဖွင့်ခဲ့သော ပူးတွဲဖိုင်များကို `%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` ၏
ဖိုင်တွဲတစ်ခုထဲသို့ ကူးယူပြီး (Windows 8.1 မတိုင်မီ `Temporary Internet Files\Content.Outlook`) ၎င်းတို့သည် ထိုနေရာတွင်
ကျန်ရှိနေနိုင်ပါသည်။

Outlook သည် လည်ပတ်နေစဉ် ၎င်း၏ store များကို ဖွင့်ထားပြီး OST တစ်ခုသည် gigabyte ဆယ်ချီ ရှိနိုင်ပါသည်။ ၎င်းတို့ကို Outlook
ပိတ်ထားစဉ် သို့မဟုတ် disk image သို့မဟုတ် volume shadow copy မှ စုဆောင်းပါ။ Juntei သည် store တစ်ခုစီကို stream အဖြစ်
ဖတ်ပါသည်: store တစ်ခုလုံးကို memory ထဲတွင် ဘယ်တော့မှ မထားပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- Juntei သည် ခွင့်လွှတ်စွာ (leniently) ဖတ်ပါသည်: ပျက်စီးနေသော အပိုင်းကို ကျော်သွားပြီး ဖတ်နိုင်သမျှကို ဖတ်ကာ ကျန်အရာများကို
  problem များအဖြစ် အစီရင်ခံပါသည်။ ၎င်းတို့ကို print ထုတ်ပြ (ဖိုင်တစ်ခုစီ၏ ပထမ 10 ခု)၊ summary နှင့် Files table တွင်
  ရေတွက်ပြီး ၎င်းတို့ သက်ဆိုင်သော item များနှင့် ပူးတွဲဖိုင်များ၏ `Problems` field တွင် စာရင်းပြုစုပါသည်။ လုံးဝ မဖတ်နိုင်သော
  item တွင်လည်း ၎င်း၏ `ItemID` နှင့် အကြောင်းရင်းနှင့်အတူ Items table တွင် row တစ်ခု ရှိပါသည်။
- အချိန်များသည် UTC ဖြင့် 100 nanosecond အထိ ဖြစ်ပါသည်။ `SubmitTime` သည် ပေးပို့သူ၏ client က message ကို submit
  လုပ်ခဲ့သည့်အချိန် (ထို ကွန်ပျူတာ၏ နာရီအတိုင်း) ဖြစ်ပြီး `DeliveryTime` သည် ၎င်း ဤ mailbox သို့ ရောက်ရှိခဲ့သည့်အချိန်
  ဖြစ်ပါသည်။ `CreationTime` သည် item ၏ ဤမိတ္တူကို ပြုလုပ်ခဲ့သည့်အချိန် ဖြစ်ပါသည်: mailbox သို့ deliver လုပ်ခြင်း သို့မဟုတ်
  mailbox ထဲတွင် ဖန်တီးခြင်း၊ OST သို့ synchronize လုပ်ခြင်း၊ PST ထဲသို့ ကူးယူခြင်း သို့မဟုတ် import လုပ်ခြင်း၊ သို့မဟုတ် MSG
  ဖိုင်အဖြစ် သိမ်းဆည်းခြင်း။ ၎င်းသည် message ကို ရေးသားခဲ့သည့်အချိန် မဟုတ်ပါ။ `LastModifiedTime` သည် store အတွင်း item
  ကို နောက်ဆုံး ပြောင်းလဲခဲ့သည့်အချိန် ဖြစ်ပါသည်။
- Task တစ်ခု၏ `StartTime` နှင့် `DueTime` သည် အသုံးပြုသူ၏ time zone ရှိ ရက်စွဲများ ဖြစ်ပြီး Outlook က ၎င်းတို့ကို
  သန်းခေါင်ယံအဖြစ် သိမ်းဆည်းပါသည်။ Juntei သည် ၎င်းတို့ကို offset မပါဘဲ ရေးပြီး (`2017-02-19T00:00:00`) `-Z` က ၎င်းတို့ကို
  ပြောင်းလဲခြင်း မပြုပါ။
- Task ၏ due date သို့မဟုတ် reminder ၏ အချိန် ကဲ့သို့သော ရက်စွဲအချို့တွင် Outlook သည် **None** အတွက် 4501 ခုနှစ်
  ဇန်နဝါရီ 1 ရက် သန်းခေါင်ယံကို ရေးပါသည်။ Juntei သည် ၎င်းကို သိမ်းဆည်းထားသည့်အတိုင်း ရေးပါသည်: `4501-01-01T00:00:00Z`၊
  task ၏ `StartTime` နှင့် `DueTime` တွင်မူ `Z` မပါဘဲ။
- `Read` သည် store ကို စုဆောင်းခဲ့ချိန်က read flag ဖြစ်ပါသည်: တစ်စုံတစ်ယောက် သို့မဟုတ် rule တစ်ခုက item ကို ဖတ်ပြီးဟု
  အမှတ်အသားပြုခဲ့ပါသည်။ ၎င်းသည် မည်သူ သို့မဟုတ် မည်သည့်အချိန်ကို မဖော်ပြပါ။
- ပေးပို့သူနှင့် လက်ခံသူ၏ အမည်များနှင့် လိပ်စာများသည် ပေးပို့သော program နှင့် mail server များက မှတ်တမ်းတင်ခဲ့သည့်
  အရာများ ဖြစ်ပြီး အတုလုပ်နိုင်ပါသည်။ `EX` လိပ်စာများသည် Exchange distinguished name များ ဖြစ်ပါသည်
  (`/o=…/ou=…/cn=Recipients/cn=…`)။ Item က SMTP လိပ်စာကိုပါ သိမ်းဆည်းထားလျှင် ၎င်းသည် `SenderSMTP-Address` နှင့်
  `SMTP-Address` တွင် ရှိပါသည်။ Message မည်သည့်နေရာမှ လာခဲ့သည်ကို စစ်ဆေးရန် ၎င်း၏ Internet header များကို ဖတ်ပါ:
  Properties table ၏ `0x007D001F` သို့မဟုတ် `0x007D001E` (`PidTagTransportMessageHeaders`)။
- Outlook တွင် ဖျက်ထားသော item များသည် **Deleted Items** ကို ရှင်းလင်းသည်အထိ ထိုနေရာတွင် ရှိနေပြီး အခြား item များကဲ့သို့ပင်
  ဖတ်ပါသည်။ မည်သည့် ဖိုင်တွဲ၏ table တွင်မျှ မရှိတော့သော item များ (store သည် ၎င်းတို့၏ နေရာကို ပြန်လည်အသုံးမပြုမီအထိ
  ထားရှိနိုင်သည်) ကို ယခုထိ မဖတ်သေးပါ။
- `ItemID` သည် ၎င်း၏ PST သို့မဟုတ် OST အတွင်းရှိ message ၏ node id ဖြစ်ပါသည်: store ၏ အခြား မိတ္တူတစ်ခုတွင် item
  တစ်ခုတည်းပင် အခြား id ရှိနိုင်ပါသည်။ ထည့်သွင်းထားသော item ၏ id သည် ၎င်း၏ parent ၏ id နှင့် ပူးတွဲဖိုင်၏ index ဖြစ်ပါသည်
  (`2097348/0`၊ `0/0/1`)။ `.msg` သို့မဟုတ် `.oft` ဖိုင်၏ item ၏ id သည် `0` ဖြစ်ပါသည်။
- `User` သည် path မှသာ လာပြီး (`Users` ၏ နောက်ရှိ ဖိုင်တွဲ) ဖိုင်ကို အခြားနေရာသို့ ကူးယူသည့်အခါ ပျောက်သွားပါသည်။ PST
  တစ်ခုသည် မည်သည့် အသုံးပြုသူ၏ ဖိုင်မဆို ဖြစ်နိုင်ပါသည်။
- Unicode မဟုတ်သော စာသားကို item ၏ code page (၎င်း၏ ကိုယ်ပိုင်၊ ၎င်း၏ locale ၏ သို့မဟုတ် ၎င်း၏ Internet code page)
  ဖြင့် ဖတ်ပါသည်။ `--code-page` သည် code page မဖော်ပြသော item များ၏ code page ကို ပေးပါသည်: ၎င်းတို့ကို ရေးခဲ့သော စနစ်၏
  ANSI code page၊ ဂျပန် စနစ်အတွက် `932`။
- PST ၏ password သည် ၎င်းကို encrypt မလုပ်ပါ: Juntei သည် password ကာကွယ်ထားသော PST များကို အခြား PST များကဲ့သို့ပင်
  ဖတ်ပါသည်။ Store တစ်ခု၏ `Encryption` (`Permute`၊ Outlook ၏ "compressible encryption" သို့မဟုတ် `Cyclic`) သည် ၎င်း၏
  block များကို ဖုံးကွယ်ရုံသာ ဖြစ်ပါသည်။ S/MIME ဖြင့် encrypt လုပ်ထားသော message များသည် ၎င်းတို့၏ body နှင့် ပူးတွဲဖိုင်များကို
  encrypt လုပ်ထားသော ပူးတွဲဖိုင်တစ်ခု (`smime.p7m`) ထဲတွင် ထားရှိပြီး ၎င်းကို decrypt မလုပ်ပါ။
- ပူးတွဲဖိုင်များသည် မကြာခဏ malware ဖြစ်တတ်ပါသည်။ `--extract-attachments` သည် ၎င်းတို့၏ data ကို ၎င်းတို့၏ SHA-256 ဖြင့်
  အမည်ပေးထားသော ဖိုင်များထဲသို့ Unix ပေါ်ရှိ ဖိုင်အသစ်များအတွက် mode `0600` ဖြင့် ရေးပါသည်။ ၎င်းတို့ကို ဘယ်တော့မှ မ run ပါနှင့်၊ သီးခြားခွဲထားသော
  (isolated) analysis စနစ်တွင်သာ ထားပါ။

### မဖတ်ရသေးသော အရာများ { #not-read-yet }

Offline address book များ (OAB)၊ autocomplete stream (`Stream_Autocomplete_*.dat`) နှင့် `.nk2` ဖိုင်များ၊ send/receive
setting များ (`.srs`)၊ Outlook ၏ setting cache များ (RoamCache၊ Autodiscover)၊ Outlook ၏ SQLite database များနှင့် မည်သည့်
ဖိုင်တွဲ၏ table တွင်မျှ မရှိတော့သော message များ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pst, *.ost, *.msg and *.oft
  -f, --file <FILE>      Parse one PST, OST, MSG or OFT file, whatever its name
      --code-page <CP>   Code page of items that don't give theirs (default 1252)

Output:
  -o, --output <PATH>              Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>         csv, json, or jsonl
      --properties                 Add the Properties table: every property as stored
      --extract-attachments <DIR>  Write each attachment's data once, named by its SHA-256
  -C, --clobber                    Overwrite files when saving or extracting
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl \
  | jq 'select(.Table == "Items" and .Hidden == false) | {DeliveryTime, SenderAddress, Subject}'
juntei parse-file outlook -f suspicious.msg -o msg-csv --properties --extract-attachments isolated/
```

`-d` သည် `.pst`၊ `.ost`၊ `.msg` နှင့် `.oft` အမည်များကို စာလုံးအကြီးအသေး မခွဲဘဲ ရှာပြီး header များကို စစ်သည်။ PST သို့မဟုတ် OST အတွက် `!BDN`၊ MSG သို့မဟုတ် OFT အတွက် compound file signature ကို အသုံးပြုသည်။ Tcl message catalog ကဲ့သို့ တူညီသော extension သုံးသော မသက်ဆိုင်သည့် ဖိုင်များကို ကျော်သည်။ မဖတ်နိုင်သော ဖိုင်များနှင့် signature ကို သိနိုင်သေးသည့် ပျက်စီးသော container များကို error အဖြစ် ဆက်လက်ဖော်ပြသည်။ `-f` သည် ဖိုင်အမည် မည်သို့ပင်ဖြစ်စေ ပေးထားသောဖိုင်ကို စစ်ပြီး မမှန်သည့် input ကို error အဖြစ် ဖော်ပြသည်။ ဖိုင်တစ်ခုမျှ မဖတ်နိုင်လျှင် exit status သည် 1 ဖြစ်သည်။

`--extract-attachments` သည် ပူးတွဲဖိုင်တစ်ခုစီ၏ data ကို တစ်ကြိမ်သာ `<SHA-256>` အဖြစ် ရေးပြီး ပူးတွဲဖိုင်၏ extension သည်
ASCII စာလုံးနှင့် ဂဏန်း 1 လုံးမှ 10 လုံးအထိ ဖြစ်လျှင် ထို extension ကို ထည့်ပါသည် (`0c7d…1311.jpg`)။ Item မှ လာသော
မည်သည့် အမည်ကိုမျှ path တွင် အသုံးမပြုပါ။ `-C` မပေးလျှင် directory သည် ဗလာ ဖြစ်ရပါမည်။ `-C` ဖြင့်ဆိုလျှင် အမည်တူဖြစ်ပြီး
အခြား အကြောင်းအရာ ပါဝင်သော ဖိုင်ကို အစားထိုးပါသည်။

## Output table များ { #output-tables }

CSV သည် ဖိုင် ရှစ်ခု (`--properties` ဖြင့် ကိုးခု) ဖန်တီးပြီး row မရှိလျှင် ဖိုင်တစ်ခုစီတွင် ၎င်း၏ header သာ ပါဝင်ပါသည်။ JSON
နှင့် JSONL row များတွင် `SourceFile` ၏ နောက်တွင် `Table` (`Files`၊ `Folders`၊ `Items`၊ `Recipients`၊ `Attachments`၊
`Contacts`၊ `Calendar`၊ `Tasks` သို့မဟုတ် `Properties`) ပါဝင်ပါသည်။ Item တစ်ခု၏ row များသည် ၎င်း၏ နောက်တွင် လိုက်ပြီး
၎င်း၏ ထည့်သွင်းထားသော item များ၏ row များသည် ထို row များ၏ နောက်တွင် လိုက်ပါသည်။ ဖိုင်တစ်ခု၏ Files row သည် ၎င်း၏ ကျန်
row အားလုံး၏ နောက်တွင် လာပါသည်။

| CSV ဖိုင် | အကြောင်းအရာ |
|---|---|
| `Outlook-Files.csv` | ဖတ်ခဲ့သော ဖိုင်တစ်ခုလျှင် row တစ်ခု: store သို့မဟုတ် ဖိုင်နှင့် ၎င်းတွင် ပါဝင်သည့်အရာ။ |
| `Outlook-Folders.csv` | PST များနှင့် OST များ၏ ဖိုင်တွဲများ။ |
| `Outlook-Items.csv` | Item တိုင်း: ဖိုင်တွဲများ၏ message များ၊ ၎င်းတို့၏ ဝှက်ထားသော item များ၊ `.msg` နှင့် `.oft` ဖိုင်များ၏ item များနှင့် ၎င်းတို့၏ ပူးတွဲဖိုင်များတွင် ထည့်သွင်းထားသော item များ။ |
| `Outlook-Recipients.csv` | Item များ၏ လက်ခံသူများ။ |
| `Outlook-Attachments.csv` | Item များ၏ ပူးတွဲဖိုင်များ။ |
| `Outlook-Contacts.csv` | Contact များနှင့် distribution list များ၏ field များ။ |
| `Outlook-Calendar.csv` | ပြက္ခဒိန် item များနှင့် meeting message များ၏ field များ။ |
| `Outlook-Tasks.csv` | Task များနှင့် task request များ၏ field များ။ |
| `Outlook-Properties.csv` | `--properties` ဖြင့်: store များ၊ ဖိုင်တွဲများ၊ item များ၊ လက်ခံသူများနှင့် ပူးတွဲဖိုင်များ၏ property တိုင်း၊ သိမ်းဆည်းထားသည့်အတိုင်း။ |

## Field များ { #fields }

Table တိုင်းသည် ဤ field များဖြင့် စတင်ပါသည်။

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `SourceFile` | Row ကို ဖတ်ယူခဲ့သော ဖိုင်။ |
| `User` | Path ပါဝင်သည့် profile ၏ Windows အသုံးပြုသူ (`Users` ၏ နောက်ရှိ ဖိုင်တွဲ)။ |

### Files { #files }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `FileType` | `PST`၊ `OST`၊ `MSG`၊ `OFT` သို့မဟုတ် item အဖြစ် ဖတ်ခဲ့သော်လည်း class ID သည် message ၏ ID လည်း မဟုတ်၊ template ၏ ID လည်း မဟုတ်သည့် compound file အတွက် `CFB` (Outlook မှလွဲ၍ အခြား program အများအပြားက ၎င်းကို ဗလာ ထားခဲ့ပါသည်)။ |
| `Format` | Store ၏ layout: `ANSI` (Outlook 97 မှ 2002)၊ `Unicode` (Outlook 2003 နှင့် နောက်ပိုင်း) သို့မဟုတ် `Unicode 4K` (Outlook 2013 နှင့် နောက်ပိုင်း၏ OST များ)။ MSG သို့မဟုတ် OFT ဖိုင်၏ compound file version (`CFB v3`၊ `CFB v4`)။ |
| `Version`, `ClientVersion` | Store ၏ header ရှိ `wVer` နှင့် `wVerClient`။ |
| `Encryption` | Store ၏ block များကို encode လုပ်ထားပုံ: `None`၊ `Permute` သို့မဟုတ် `Cyclic`။ |
| `DisplayName` | Store ၏ အမည် (`Personal Folders`၊ e-mail လိပ်စာ တစ်ခု)။ |
| `RecordKey` | Store ၏ identity (hex)၊ ၎င်း၏ item များ၏ entry id များတွင် ပါဝင်ပါသည်။ |
| `Folders` | Store ၏ ဖိုင်တွဲများ။ |
| `Items` | ၎င်း၏ ဖိုင်တွဲများ၏ message များ၊ သို့မဟုတ် MSG သို့မဟုတ် OFT ဖိုင်အတွက် 1။ |
| `HiddenItems` | ၎င်း၏ ဖိုင်တွဲများ၏ ဝှက်ထားသော item များ (folder associated information)။ |
| `EmbeddedItems` | အဆင့်တိုင်းရှိ ပူးတွဲဖိုင်များတွင် ထည့်သွင်းထားသော item များ။ |
| `Attachments` | ထို item အားလုံး၏ ပူးတွဲဖိုင်များ။ |
| `Problems` | ဖိုင်တွင် တွေ့ရှိခဲ့သော problem များ။ |

### Folders { #folders }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `FolderID`, `ParentID` | ဖိုင်တွဲ၏ node id နှင့် ၎င်း၏ parent ဖိုင်တွဲ၏ node id (root ဖိုင်တွဲအတွက် ဗလာ)။ |
| `FolderPath` | Root မှ အောက်သို့ ဖိုင်တွဲများ၏ အမည်များ: `/Top of Personal Folders/Inbox`၊ `/Root - Mailbox/IPM_SUBTREE/Sent Items`။ |
| `Name` | ဖိုင်တွဲ၏ အမည်။ |
| `ContainerClass` | ဖိုင်တွဲတွင် ပါဝင်သည့်အရာ: `IPF.Note` (mail)၊ `IPF.Appointment`၊ `IPF.Contact`၊ `IPF.Task`၊ `IPF.Imap`၊ …။ |
| `ContentCount`, `UnreadCount` | ဖိုင်တွဲက ရှိသည်ဟု ဖော်ပြသော message များနှင့် မဖတ်ရသေးသော message များ။ |
| `Messages`, `HiddenItems`, `Subfolders` | ၎င်း၏ contents table၊ associated contents table နှင့် hierarchy table ၏ row များ: Juntei ဖတ်သည့်အရာ။ `ContentCount` နှင့် ကွာခြားချက် ရှိလျှင် စစ်ဆေးကြည့်သင့်ပါသည်။ |
| `CreationTime`, `LastModifiedTime` | ဖိုင်တွဲကို ဖန်တီးခဲ့သည့်အချိန်နှင့် နောက်ဆုံး ပြောင်းလဲခဲ့သည့်အချိန် (UTC)၊ store က ၎င်းတို့ကို ထားရှိလျှင်။ |
| `Problem` | ဖိုင်တွဲကို မဖွင့်နိုင်ခဲ့သည့် အကြောင်းရင်း။ ထိုအခါ ၎င်း၏ subfolder များကို မဖတ်ပါ။ |

### Items { #items }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `FolderPath` | Item ၏ ဖိုင်တွဲ။ MSG သို့မဟုတ် OFT ဖိုင်တွင် ဗလာ။ |
| `ItemID` | Item ၏ id: PST သို့မဟုတ် OST တွင် ၎င်း၏ node id၊ MSG သို့မဟုတ် OFT ဖိုင်တွင် `0`၊ ထည့်သွင်းထားသော item အတွက် ၎င်း၏ parent ၏ id နှင့် ပူးတွဲဖိုင်၏ index (`2097348/0`)။ |
| `Hidden` | Item သည် folder associated information (ဝှက်ထားသော item) ဟုတ်မဟုတ်။ |
| `EmbeddedIn` | ၎င်းကို ပူးတွဲထားသည့် item ၏ id။ အခြား item များအတွက် ဗလာ။ |
| `ItemType` | Message class အရ item ၏ အမျိုးအစား: `Mail`၊ `MeetingMessage`၊ `Appointment`၊ `Contact`၊ `DistributionList`၊ `Task`၊ `TaskRequest`၊ `StickyNote`၊ `Journal`၊ `Post`၊ `Report`၊ `Configuration` သို့မဟုတ် `Other`။ |
| `MessageClass` | သိမ်းဆည်းထားသည့်အတိုင်း message class (`IPM.Note`၊ `IPM.Note.SMIME`၊ `IPM.Schedule.Meeting.Request`၊ `REPORT.IPM.Note.NDR`၊ …)။ |
| `SubmitTime` | ပေးပို့သူ၏ client က message ကို submit လုပ်ခဲ့သည့်အချိန် (UTC၊ ပေးပို့သူ၏ နာရီ)။ |
| `DeliveryTime` | ၎င်းကို ဤ mailbox သို့ deliver လုပ်ခဲ့သည့်အချိန် (UTC)။ |
| `CreationTime`, `LastModifiedTime` | ဤမိတ္တူကို store သို့မဟုတ် ဖိုင်တွင် ပြုလုပ်ခဲ့သည့်အချိန်နှင့် နောက်ဆုံး ပြောင်းလဲခဲ့သည့်အချိန် (UTC)။ |
| `Subject` | ၎င်း၏ prefix (`RE: `) ပါသော subject။ |
| `SenderName`, `SenderAddressType`, `SenderAddress` | ပေးပို့သူ: အမည်၊ လိပ်စာ type (`SMTP`၊ `EX`) နှင့် ထို type ၏ လိပ်စာ။ |
| `SenderSMTP-Address` | အခြား type ၏ လိပ်စာနှင့်အတူ သိမ်းဆည်းထားလျှင် ပေးပို့သူ၏ SMTP လိပ်စာ။ |
| `SentRepresentingName`, `SentRepresentingAddress` | မည်သူ့ကိုယ်စား ပေးပို့ခဲ့သည် (delegate တစ်ဦးက manager ကိုယ်စား ပေးပို့ခြင်း၊ shared mailbox)၊ သိမ်းဆည်းထားလျှင် ၎င်းတို့၏ SMTP လိပ်စာနှင့်အတူ၊ မဟုတ်လျှင် ၎င်း၏ type ၏ လိပ်စာ။ |
| `To`, `Cc`, `Bcc` | Item က ပြသသည့်အတိုင်း လက်ခံသူများ၏ အမည်များ (၎င်းတို့၏ လိပ်စာများအတွက် Recipients ကို ကြည့်ပါ)။ `Bcc` သည် ပေးပို့သူ၏ မိတ္တူတွင်သာ ရှိပါသည်။ |
| `InternetMessageID`, `InReplyTo` | Message ၏ `Message-ID` နှင့် ၎င်းက ပြန်ကြားသည့် message ၏ ID။ |
| `ConversationTopic` | Prefix မပါသော subject၊ conversation တစ်ခု၏ message များက မျှဝေသုံးပါသည်။ |
| `Importance`, `Sensitivity` | `Low`၊ `Normal` သို့မဟုတ် `High`။ `Normal`၊ `Personal`၊ `Private` သို့မဟုတ် `Confidential`။ |
| `Read` | Item ကို ဖတ်ပြီးဟု အမှတ်အသားပြုထားခြင်း ရှိမရှိ။ |
| `Flags` | Message flag များ (`PidTagMessageFlags`၊ hex): `0x1` ဖတ်ပြီး၊ `0x2` မပြင်ဆင်ရသေး၊ `0x4` submit လုပ်ပြီး၊ `0x8` မပို့ရသေး (draft)၊ `0x10` ပူးတွဲဖိုင် ပါ၊ `0x20` ကျွန်ုပ်ထံမှ၊ `0x40` ဝှက်ထား၊ `0x80` ပြန်ပို့ (resent)၊ `0x400` တစ်ကြိမ်ကြိမ် ဖတ်ခဲ့ဖူး၊ `0x2000` Internet မှ။ |
| `Size` | Store အတွင်းရှိ item ၏ size၊ ၎င်း၏ ပူးတွဲဖိုင်များ အပါအဝင်။ |
| `Recipients`, `Attachments` | ၎င်းတွင် ရှိသော လက်ခံသူ အရေအတွက်နှင့် ပူးတွဲဖိုင် အရေအတွက်။ |
| `Categories` | ၎င်း၏ category များ၊ ၎င်းတို့ကြားတွင် `;` ပါသည်။ |
| `BodyPreview` | ၎င်း၏ plain-text body ၏ အစပိုင်း၊ white space များကို ချုံ့ထားပြီး အများဆုံး စာလုံး 256 လုံး။ HTML သို့မဟုတ် RTF body သာ ရှိလျှင် ဗလာ။ |
| `Problems` | Item တွင် မဖတ်နိုင်ခဲ့သည့်အရာ (အများဆုံး 10 ခု၊ ထို့နောက် ကျန်အရေအတွက်)။ |

### Recipients { #recipients }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `ItemID` | Item ၏ id။ |
| `Index` | Item ၏ recipient table ရှိ လက်ခံသူ၏ row၊ 0 မှ စတင်သည်။ |
| `RecipientType` | `To`၊ `Cc`၊ `Bcc`၊ `Originator` သို့မဟုတ် သိမ်းဆည်းထားသည့်အတိုင်း value (hex)။ |
| `Name`, `AddressType`, `Address` | လက်ခံသူ၏ အမည်၊ လိပ်စာ type နှင့် လိပ်စာ။ |
| `SMTP-Address` | အခြား type ၏ လိပ်စာနှင့်အတူ သိမ်းဆည်းထားလျှင် SMTP လိပ်စာ။ |

### Attachments { #attachments }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `ItemID`, `Index` | Item ၏ id နှင့် ၎င်း၏ attachment table ရှိ ပူးတွဲဖိုင်၏ row၊ 0 မှ စတင်သည်။ |
| `Method` | `ByValue` (ဖိုင်)၊ `ByReference`၊ `ByReferenceResolve`၊ `ByReferenceOnly` (ဖိုင်သို့ path)၊ `EmbeddedMessage` (item)၊ `OLE` (OLE object)၊ `ByWebReference` သို့မဟုတ် နံပါတ်။ |
| `Filename` | ပူးတွဲဖိုင်၏ အမည်: ၎င်း၏ long filename၊ short filename သို့မဟုတ် display name။ |
| `Extension` | သိမ်းဆည်းထားသည့်အတိုင်း ၎င်း၏ extension (`.pdf`)၊ မရှိလျှင် ၎င်း၏ အမည်မှ extension။ |
| `MimeType`, `ContentID` | ၎င်း၏ MIME type နှင့် HTML body က ၎င်းကို ရည်ညွှန်းရာတွင် သုံးသော content id။ |
| `Inline` | Body က ၎င်းကို စာရင်းတွင် ပြမည့်အစား body ထဲတွင် ပြသခြင်း (HTML body ၏ ပုံ) ရှိမရှိ။ |
| `Hidden` | Outlook က ၎င်းကို attachment list မှ ဝှက်ထားခြင်း ရှိမရှိ။ |
| `Size` | Store က ပေးသည့်အတိုင်း ၎င်း၏ size၊ ၎င်း၏ property များ အပါအဝင်။ |
| `DataSize`, `SHA256` | ၎င်း၏ data ၏ size နှင့် SHA-256: ဖိုင် သို့မဟုတ် PST သို့မဟုတ် OST ရှိ OLE object ၏ compound file။ ထည့်သွင်းထားသော item နှင့် reference ဖြင့် ပူးတွဲထားသော ဖိုင်အတွက် ဗလာ။ |
| `Pathname` | Reference ဖြင့် ပူးတွဲထားသော ဖိုင်၏ path။ |
| `CreationTime`, `LastModifiedTime` | ပူးတွဲဖိုင်၏ အချိန်များ (UTC): မကြာခဏ ၎င်းကို ပြုလုပ်ခဲ့ရာ ဖိုင်၏ အချိန်များ။ |
| `EmbeddedSubject` | ထည့်သွင်းထားသော item ၏ subject။ ထို item ၏ ကိုယ်ပိုင် row များတွင် `ItemID` သည် `<ItemID>/<Index>` ဖြစ်ပါသည်။ |
| `ExtractedFile` | `--extract-attachments` ဖြင့် ၎င်း၏ data ကို ရေးခဲ့သော ထို directory ထဲရှိ ဖိုင်။ |
| `Problems` | ပူးတွဲဖိုင်တွင် မဖတ်နိုင်ခဲ့သော သို့မဟုတ် မရေးနိုင်ခဲ့သော အရာ။ |

### Contacts { #contacts }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | Items table ကဲ့သို့: `Contact` သို့မဟုတ် `DistributionList`။ |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | Contact ၏ အမည်များ။ |
| `FileUnder` | Contact ကို file လုပ်ထားသော အမည်။ |
| `Company`, `Title`, `Department` | Contact အလုပ်လုပ်သော နေရာ။ |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | Contact ၏ e-mail လိပ်စာ သုံးခု၊ တစ်ခုစီ ၎င်း၏ display name နှင့်အတူ။ |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | Contact ၏ ဖုန်းနံပါတ်များ။ |
| `InstantMessaging` | Contact ၏ instant messaging လိပ်စာ။ |
| `CreationTime`, `LastModifiedTime` | Items table ကဲ့သို့။ |

### Calendar { #calendar }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Items table ကဲ့သို့: `Appointment` သို့မဟုတ် `MeetingMessage`။ |
| `StartTime`, `EndTime` | Appointment စတင်ချိန်နှင့် ပြီးဆုံးချိန် (UTC)။ ထပ်ခါထပ်ခါ ဖြစ်သော appointment အတွက် ပထမဆုံး အကြိမ်၏ အချိန်များ။ |
| `DurationMinutes` | ၎င်း၏ ကြာချိန် (မိနစ်)။ |
| `Location` | ၎င်း ကျင်းပမည့်နေရာ။ |
| `AllDay`, `Recurring` | တစ်နေ့လုံး event ဟုတ်မဟုတ်နှင့် ထပ်ခါထပ်ခါ ဖြစ်ခြင်း ရှိမရှိ (၎င်း၏ exception များသည် ထည့်သွင်းထားသော item များ ဖြစ်သည်)။ |
| `BusyStatus` | `Free`၊ `Tentative`၊ `Busy`၊ `OutOfOffice` သို့မဟုတ် `WorkingElsewhere`။ |
| `ResponseStatus` | အသုံးပြုသူ၏ response: `None`၊ `Organized`၊ `Tentative`၊ `Accepted`၊ `Declined` သို့မဟုတ် `NotResponded`။ |
| `Organizer` | Item ၏ ပေးပို့သူ၏ အမည်။ |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | Item က စာရင်းပြုထားသည့်အတိုင်း တက်ရောက်သူများ။ |
| `GlobalObjectID` | Mailbox များအကြား meeting ၏ identity (hex): organizer ၏ ပြက္ခဒိန်နှင့် တက်ရောက်သူများ၏ ပြက္ခဒိန်များရှိ meeting တစ်ခုတည်း။ |
| `ReminderSet`, `ReminderTime` | Reminder သတ်မှတ်ထားခြင်း ရှိမရှိနှင့် ၎င်းအတွက် သတ်မှတ်ထားသော အချိန် (UTC)။ |
| `CreationTime`, `LastModifiedTime` | Items table ကဲ့သို့။ |

### Tasks { #tasks }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Items table ကဲ့သို့: `Task` သို့မဟုတ် `TaskRequest`။ |
| `Status` | `NotStarted`၊ `InProgress`၊ `Complete`၊ `Waiting` သို့မဟုတ် `Deferred`။ |
| `PercentComplete` | ၎င်း၏ ပြီးစီးမှု ပမာဏ၊ 0 မှ 100 အထိ။ |
| `StartTime`, `DueTime` | ၎င်း စတင်မည့်ရက်နှင့် due ဖြစ်မည့်ရက်: အသုံးပြုသူ၏ time zone ရှိ ရက်စွဲများ၊ offset မပါ။ |
| `CompletedTime` | ၎င်းကို ပြီးစီးခဲ့သည့်အချိန် (UTC)။ |
| `Complete` | ၎င်း ပြီးစီးပြီ ဟုတ်မဟုတ်။ |
| `Owner`, `Assigner` | ၎င်းကို ပိုင်ဆိုင်သူနှင့် ၎င်းကို တာဝန်ပေးခဲ့သူ။ |
| `ReminderSet`, `ReminderTime` | Calendar table ကဲ့သို့။ |
| `CreationTime`, `LastModifiedTime` | Items table ကဲ့သို့။ |

### Properties { #properties }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `ItemID` | Item ၏ id။ store ၏ ကိုယ်ပိုင် property များအတွက် `store`၊ ဖိုင်တွဲ၏ property များအတွက် `folder:<FolderID>`။ |
| `Scope` | Property ၏ ပိုင်ရှင်: `Store`၊ `Folder`၊ `Item`၊ `Recipient` သို့မဟုတ် `Attachment`။ |
| `Index` | လက်ခံသူ သို့မဟုတ် ပူးတွဲဖိုင်၏ index။ အခြားများအတွက် ဗလာ။ |
| `PropertyTag` | Property ၏ id နှင့် type၊ `0x0037001F` ပုံစံဖြင့် (`PidTagSubject`၊ string တစ်ခု)။ |
| `PropertyName` | Named property (`0x8000` မှစသော id များ) အတွက် ၎င်း၏ property set နှင့် အမည်: `PSETID_Address:0x8083`၊ `PS_PUBLIC_STRINGS:Keywords` သို့မဟုတ် လူသိများသော set မဟုတ်လျှင် set ၏ GUID။ |
| `Type` | Property type: `PtypString`၊ `PtypInteger32`၊ `PtypTime`၊ `PtypBinary`၊ `PtypMultipleString`၊ …။ |
| `Value` | Value: string များအတွက် စာသား၊ အချိန်များအတွက် ISO 8601 UTC (`-Z` က ပြောင်းလဲခြင်း မပြုပါ)၊ binary value များအတွက် hex (ပထမ 4,096 byte၊ ထို့နောက် `…(N bytes)`)၊ multi-valued property ၏ value များကို စာရင်းအဖြစ် (CSV တွင် ၎င်းတို့ကြား `, `)၊ ထည့်သွင်းထားသော item သို့မဟုတ် ပူးတွဲဖိုင်၏ data အတွက် `(object)`၊ decode မလုပ်သော type ၏ value အတွက် မှတ်ချက်တစ်ခုနှင့်အတူ byte များကို hex ဖြင့် နှင့် ပျက်စီးနေသော value အတွက် ၎င်းမှ ပြန်လည်ရယူနိုင်ခဲ့သည့်အရာ၊ ထို့နောက် ၎င်း၏ byte များကို hex ဖြင့် ပါဝင်သော မှတ်ချက် (ဘာမျှ ပြန်လည်မရယူနိုင်ပါက byte များနှင့် မှတ်ချက်သာ)။ |

## အလိုအလျောက် ခွဲခြမ်းစိတ်ဖြာခြင်း { #automatic-parsing }

`auto-parse` သည် table များကို `Communication` (ဆက်သွယ်ရေး) category အသစ်ထဲသို့ `Outlook-Files.csv`၊
`Outlook-Folders.csv`၊ `Outlook-Items.csv`၊ `Outlook-Recipients.csv`၊ `Outlook-Attachments.csv`၊ `Outlook-Contacts.csv`၊
`Outlook-Calendar.csv` နှင့် `Outlook-Tasks.csv` အဖြစ် (`-t json` သို့မဟုတ် `-t jsonl` ဖြင့်ဆိုလျှင် `Outlook.json` သို့မဟုတ်
`Outlook.jsonl` ဖိုင်တစ်ခုတည်း) `Communication/Outlook-logs.txt` နှင့်အတူ ရေးသားပါသည်။ ၎င်းသည် parser ကို ၎င်း၏
`--code-page` ကို ပေးပြီး Properties table ကို မထည့်သလို ပူးတွဲဖိုင်များကိုလည်း မထုတ်ယူပါ: ၎င်းတို့အတွက်
`parse-file outlook` ကို run ပါ။ ပေါင်းစပ်ထားသော `-j` timeline တွင် row တိုင်းကို `Comp` နှင့် `Artifact` identifier
`outlook` တို့နှင့်အတူ ထည့်သွင်းပါသည်။ `-Z` သည် task ၏ `StartTime` နှင့် `DueTime` မှလွဲ၍ `Time` ဖြင့် အဆုံးသတ်သော
field တိုင်းကို ပြောင်းလဲပေးပါသည်။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Personal Folder File (PFF) format](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
