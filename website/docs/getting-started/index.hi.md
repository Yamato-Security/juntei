# Juntei इंस्टॉल करें { #install-juntei }

## Release binaries { #release-binaries }

अपने सिस्टम के लिए zip को
[Juntei releases page](https://github.com/Yamato-Security/juntei/releases) से download करें। यदि अभी तक कोई release binary
प्रकाशित नहीं हुई है, तो नीचे बताए अनुसार source से compile करें। Archive या source commit और
checksum को अपने case notes के साथ रखें, ताकि कोई अन्य विश्लेषक उसी run को दोहरा सके।

| सिस्टम | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` या `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` या `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

हर zip में केवल binary होती है, जिसका नाम version और platform के साथ होता है (उदाहरण के लिए
`juntei-1.0.0-win-x64.exe`): Juntei को किसी अन्य फ़ाइल की आवश्यकता नहीं होती, और Windows binary को
Visual C++ Redistributable की आवश्यकता नहीं होती। musl Linux binaries statically linked हैं और किसी भी
distribution पर चलती हैं; gnu वाली binaries को हाल का glibc चाहिए। macOS पर, download की गई binary को
System Settings > Privacy & Security में अनुमति दें, या उसका quarantine attribute
`xattr -d com.apple.quarantine juntei-<version>-mac-*` से हटाएँ।

```bash
juntei --version
juntei --help
```

उदाहरण binary के लिए `juntei` का उपयोग करते हैं; इसका नाम बदलें, या इसका पूरा नाम type करें।

## Source से compile करें { #compile-from-source }

Juntei को वर्तमान में `Cargo.toml` में घोषित Rust version की आवश्यकता है (यह page लिखे जाने के समय
Rust 1.99)।

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` `Cargo.lock` में दर्ज सटीक dependency revisions का उपयोग करता है।

## Command का मूल स्वरूप { #basic-command-shape }

हर parser या तो एक फ़ाइल (`-f`) या एक directory collection (`-d`) स्वीकार करता है:

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

किसी report से पुराना command copy करने के बजाय वर्तमान options दिखाएँ:

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

सुरक्षित workflow के लिए [पहली जाँच](first-investigation.md) पर आगे बढ़ें।
