# ဖိုင် ညွှန်ကြားချက်များ { #file-commands }

`parse-file` သည် အထောက်အထားအမျိုးအစား သီးသန့်ဖိုင်များကို ကိုင်တွယ်သည်- Prefetch၊ ဖြတ်လမ်းများ၊ Jump Lists၊ Defender
quarantine နှင့် Outlook ၏ PST၊ OST၊ MSG နှင့် OFT ဖိုင်များ။ Automatic Jump Lists သည် compound-file container ကို
အသုံးပြုပြီး custom Jump Lists သည် မတူညီသော binary layout ကို အသုံးပြုသည်။ နှစ်မျိုးစလုံးသည် တူညီသော အထောက်အထား
ညွှန်ကြားချက်တစ်ခုတည်းတွင် ပါဝင်သည်။

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file <COMMAND> [OPTIONS]

Commands:
  defender-quarantine  Parse Defender quarantine
  help                 Print this message or the help of the given subcommand(s)
  jump-lists           Parse Jump Lists
  lnk                  Parse shortcuts
  outlook              Parse Outlook PST, OST, MSG and OFT files
  prefetch             Parse Prefetch files

Options:
  -h, --help  Print help
```

## `parse-file defender-quarantine` { #parse-file-defender-quarantine }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Defender quarantine folders (a volume, a Quarantine folder, a collection of hosts)
  -f, --file <FILE>      File path to one quarantine entry (Quarantine\Entries\{GUID})

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, the records are written to stdout
      --extract <DIR>       Write quarantined files (likely malware, never run) with a .quarantined extension
  -C, --clobber             Overwrite files when saving or extracting

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract isolated-output/
```

ထုတ်ယူခြင်း (extraction) သည် binary security stream များကို ထိန်းသိမ်းထားပြီး ပိုင်ဆိုင်မှု၊ ခွင့်ပြုချက်များနှင့် SDDL
ပါဝင်သော ဖတ်ရှုနိုင်သည့် `.security_descriptor.txt` အစီရင်ခံစာများကိုလည်း ဖန်တီးပေးသည်။ [Defender သီးခြားခွဲထားသော ဒေတာ](../artifacts/defender-quarantine.md) ကို ကြည့်ပါ။

## `parse-file jump-lists` { #parse-file-jump-lists }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for jump lists (*.automaticDestinations-ms, *.customDestinations-ms)
  -f, --file <FILE>      File path to one jump list
      --code-page <CP>   Code page of text in links that isn't Unicode: the system's ANSI code page (932 for Japanese) [default: 1252]

Output:
  -o, --output <PATH>       Save every entry: a directory of CSV tables, or a file (json, jsonl)
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, json and jsonl are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-lists
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

[Jump Lists](../artifacts/jump-lists.md) ကို ကြည့်ပါ။

## `parse-file lnk` { #parse-file-lnk }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for shortcuts (*.lnk)
  -f, --file <FILE>      File path to one shortcut
      --code-page <CP>   Code page of text that isn't Unicode: the system's ANSI code page (932 for Japanese) [default: 1252]

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

[LNK ဖြတ်လမ်းများ](../artifacts/lnk.md) ကို ကြည့်ပါ။

## `parse-file outlook` { #parse-file-outlook }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Outlook files (*.pst, *.ost, *.msg, *.oft)
  -f, --file <FILE>      File path to one PST, OST, MSG or OFT file (told apart by its contents, whatever its name)
      --code-page <CP>   Code page of text that isn't Unicode, for items that don't give theirs: the system's ANSI code page (932 for Japanese) [default: 1252]

Output:
  -o, --output <PATH>              Save every row: a directory of CSV tables, or a file (json, jsonl)
  -t, --output-type <TYPE>         Output type: csv (default), json, jsonl. Without -o, json and jsonl rows are written to stdout
      --properties                 Also write every property of the stores, folders, items, recipients and attachments (the Properties table)
      --extract-attachments <DIR>  Write each attachment's data once to DIR, named by its SHA-256 (attachments can be malware: never run them)
  -C, --clobber                    Overwrite files when saving or extracting

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl | jq 'select(.Table == "Items")'
juntei parse-file outlook -f invoice.msg -o msg-csv --properties --extract-attachments isolated-output/
```

Table များမှာ ဖတ်ခဲ့သော ဖိုင်များ၊ ၎င်းတို့၏ ဖိုင်တွဲများ၊ item တိုင်း (ဖိုင်တွဲများ၏ ဝှက်ထားသော item များနှင့်
ပူးတွဲဖိုင်များတွင် ထည့်သွင်းထားသော item များ အပါအဝင်)၊ လက်ခံသူများ၊ ပူးတွဲဖိုင်များ၊ contact များ၊ ပြက္ခဒိန် item များနှင့်
task များ ဖြစ်ပြီး `--properties` ဖြင့် သိမ်းဆည်းထားသည့်အတိုင်း property တိုင်း ဖြစ်သည်။ [Outlook](../artifacts/outlook.md) ကို ကြည့်ပါ။

## `parse-file prefetch` { #parse-file-prefetch }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for prefetch files (*.pf)
  -f, --file <FILE>      File path to one prefetch file

Records:
  -T, --timeline  One record per execution (the last 8 run times of every file), sorted by time

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
juntei parse-file prefetch -d evidence/ -T -o prefetch-timeline.csv
```

[Prefetch](../artifacts/prefetch.md) ကို ကြည့်ပါ။
