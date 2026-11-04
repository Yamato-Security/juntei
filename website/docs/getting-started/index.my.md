# Juntei ထည့်သွင်းခြင်း { #install-juntei }

## Release binary များ { #release-binaries }

သင့်စနစ်အတွက် zip ကို [Juntei releases စာမျက်နှာ](https://github.com/Yamato-Security/juntei/releases) မှ
download လုပ်ပါ။ Release binary ကို မထုတ်ဝေရသေးပါက အောက်တွင် ဖော်ပြထားသည့်အတိုင်း source မှ compile လုပ်ပါ။
အခြား စိစစ်သူတစ်ဦး run ကို ပြန်လည်လုပ်ဆောင်နိုင်စေရန် archive သို့မဟုတ် source commit နှင့် checksum ကို
သင့် case မှတ်စုများနှင့်အတူ ထားရှိပါ။

| စနစ် | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` သို့မဟုတ် `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` သို့မဟုတ် `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

Zip တစ်ခုစီတွင် ဗားရှင်းနှင့် platform ဖြင့် အမည်ပေးထားသော binary တစ်ခုတည်းသာ ပါဝင်သည် (ဥပမာ
`juntei-1.0.0-win-x64.exe`)- Juntei သည် အခြားဖိုင်များ မလိုအပ်ဘဲ Windows binary သည်
Visual C++ Redistributable ကို မလိုအပ်ပါ။ musl Linux binary များသည် statically link လုပ်ထားသောကြောင့် မည်သည့်
distribution တွင်မဆို run နိုင်သည်။ gnu binary များမူ မကြာသေးမီက glibc ဗားရှင်း လိုအပ်သည်။ macOS တွင် download လုပ်ထားသော
binary ကို System Settings > Privacy & Security တွင် ခွင့်ပြုပါ၊ သို့မဟုတ် ၎င်း၏ quarantine attribute ကို
`xattr -d com.apple.quarantine juntei-<version>-mac-*` ဖြင့် ဖယ်ရှားပါ။

```bash
juntei --version
juntei --help
```

ဥပမာများတွင် binary အတွက် `juntei` ကို အသုံးပြုထားသည်။ ၎င်းကို အမည်ပြောင်းပါ၊ သို့မဟုတ် ၎င်း၏ အမည်အပြည့်အစုံကို ရိုက်ထည့်ပါ။

## Source မှ compile လုပ်ခြင်း { #compile-from-source }

Juntei သည် လက်ရှိတွင် `Cargo.toml` တွင် ကြေညာထားသော Rust ဗားရှင်း (ဤစာမျက်နှာကို ရေးသားချိန်တွင် Rust 1.99)
လိုအပ်သည်။

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` သည် `Cargo.lock` တွင် မှတ်တမ်းတင်ထားသော dependency revision အတိအကျကို အသုံးပြုသည်။

## အခြေခံ ညွှန်ကြားချက် ပုံစံ { #basic-command-shape }

Parser တိုင်းသည် ဖိုင်တစ်ခု (`-f`) သို့မဟုတ် directory စုဆောင်းမှု (`-d`) တစ်ခုခုကို လက်ခံသည်-

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

အစီရင်ခံစာတစ်ခုမှ ညွှန်ကြားချက်ဟောင်းကို ကူးယူမည့်အစား လက်ရှိ option များကို ပြသပါ-

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

ဘေးကင်းသော လုပ်ငန်းစဉ်တစ်ခုအတွက် [ပထမဆုံး စုံစမ်းစစ်ဆေးမှု](first-investigation.md) ဖြင့် ဆက်လက်လုပ်ဆောင်ပါ။
