# कमांड सूची { #command-list }

अलग-अलग parsers को उनके स्रोत type के अनुसार समूहित किया गया है। उन्हें explore करने के लिए `juntei --help`,
`juntei <SOURCE> --help`, या `juntei <SOURCE> <ARTIFACT> --help` चलाएँ:

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

कई computers वाले Windows साक्ष्य संग्रह के लिए,
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md) का उपयोग करें। यह वही समूहित commands चलाता है
और परिणामों तथा logs को computer और जाँच की category के अनुसार व्यवस्थित करता है।

## [डिस्क इमेज](mount.md) { #disk-images }

| Command | Input | विस्तृत मार्गदर्शिका |
|---|---|---|
| `mount` | VHDX disk image (KAPE `--vhdx`, Hyper-V) | [डिस्क इमेज माउंट करना](mount.md) |

## [ESE कमांड](user-activity.md) { #ese-commands }

| Command | Input आर्टिफ़ैक्ट | विस्तृत मार्गदर्शिका |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [उपयोगकर्ता एक्सेस लॉगिंग (UAL)](../artifacts/ual.md) |

## [फ़ाइल कमांड](execution.md) { #file-commands }

| Command | Input आर्टिफ़ैक्ट | विस्तृत मार्गदर्शिका |
|---|---|---|
| `parse-file defender-quarantine` | Defender क्वारंटीन tree या entry | [Defender क्वारंटीन](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [LNK शॉर्टकट](../artifacts/lnk.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [NTFS कमांड](filesystem.md) { #ntfs-commands }

| Command | Input आर्टिफ़ैक्ट | विस्तृत मार्गदर्शिका |
|---|---|---|
| `parse-ntfs boot` | `$Boot`, या NTFS volume image | [बूट सेक्टर ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`), या disk image | [डायरेक्टरी इंडेक्स ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | NTFS `$LogFile` | [NTFS ट्रांज़ैक्शन लॉग](../artifacts/logfile.md) |
| `parse-ntfs mft` | NTFS `$MFT` | [मास्टर फ़ाइल टेबल](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [सुरक्षा विवरणक ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [USN जर्नल](../artifacts/usn-journal.md) |

## [रेजिस्ट्री कमांड](security-registry.md) { #registry-commands }

| Command | Input आर्टिफ़ैक्ट | विस्तृत मार्गदर्शिका |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [नेटवर्क](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [LSA सीक्रेट](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (और `BCD-Template`) | [बूट कॉन्फ़िगरेशन डेटा (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [ऐप्लिकेशन संगतता](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [ऑडिट नीति](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage, RecentApps और Taskbar पिन](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [ऐप्लिकेशन इतिहास](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [हाल में उपयोग की गई (MRU) सूचियाँ](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (उपयोगकर्ता नामों के लिए `SAM`, `SOFTWARE` के साथ) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (Windows XP पर `NTUSER.DAT`) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [इंस्टॉल किए गए प्रोग्राम](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [खोलें और इस रूप में सहेजें संवाद](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [रेजिस्ट्री ऑटोस्टार्ट](../artifacts/persistence.md) |
| `parse-registry read` | कोई भी primary registry hive | [रेजिस्ट्री हाइव](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (उपयोगकर्ता SIDs के लिए `SOFTWARE` के साथ) | [रिमोट एक्सेस (उपयोगकर्ता पक्ष)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [SAM स्थानीय खाते](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [सुरक्षा सेटिंग](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Windows सेवाएँ](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` या export की गई AppCompatCache value | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | एक सिस्टम के `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` | [सिस्टम जानकारी](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [टास्क शेड्यूलर कैश](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`, `SOFTWARE` और `NTUSER.DAT` के साथ | [USB डिवाइस](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## स्रोत types और format में बदलाव { #source-types-and-format-changes }

समूह उन formats का वर्णन करते हैं जिनका Juntei वर्तमान में समर्थन करता है। `parse-registry amcache`
`Amcache.hve` पढ़ता है; इसका अर्थ SQLite Amcache inventory databases का समर्थन नहीं है। आर्टिफ़ैक्ट parser
crates अलग रहते हैं ताकि किसी अन्य storage format का समर्थन स्वतंत्र रूप से जोड़ा जा सके।

पुराने flat parser नामों को ऊपर दिए गए source-and-artifact paths से बदल दिया गया है। मौजूदा
parser options को आर्टिफ़ैक्ट नाम के बाद रखें। `auto-parse` और मौजूदा registry subcommands
अपने नाम बनाए रखते हैं।

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` कई स्रोत formats में hunting करता है, इसलिए यह एक top-level विश्लेषण
command है। यह `parse-registry persistence` की पूर्ण configuration inventory को
131 checks, allow/block वर्गीकरण, payload metadata, coverage और timestamp correlation के साथ पूरक बनाता है।

## Top-level उपयोग { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

स्रोत pages command का उपयोग दोहराते हैं और छोटे उदाहरण देते हैं। आर्टिफ़ैक्ट मार्गदर्शिकाएँ
साक्ष्य, आउटपुट fields, सीमाएँ और जाँच की रणनीति समझाती हैं।
