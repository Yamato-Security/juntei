# รายการคำสั่ง { #command-list }

พาร์เซอร์แต่ละตัวถูกจัดกลุ่มตามชนิดของแหล่งข้อมูล เรียกใช้ `juntei --help`,
`juntei <SOURCE> --help` หรือ `juntei <SOURCE> <ARTIFACT> --help` เพื่อสำรวจคำสั่งเหล่านี้

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

สำหรับการเก็บรวบรวมหลักฐาน Windows จากคอมพิวเตอร์หลายเครื่อง ให้ใช้
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md) คำสั่งนี้เรียกใช้คำสั่งที่จัดกลุ่มเดียวกัน
และจัดระเบียบผลลัพธ์และ log ตามคอมพิวเตอร์และหมวดหมู่การตรวจสอบ

## [อิมเมจดิสก์](mount.md) { #disk-images }

| คำสั่ง | อินพุต | คู่มือโดยละเอียด |
|---|---|---|
| `mount` | อิมเมจดิสก์ VHDX | [การเมานต์อิมเมจดิสก์](mount.md) |

## [คำสั่ง ESE](user-activity.md) { #ese-commands }

| คำสั่ง | อาร์ติแฟกต์อินพุต | คู่มือโดยละเอียด |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [การบันทึกการเข้าถึงของผู้ใช้ (UAL)](../artifacts/ual.md) |
| `parse-ese webcache` | ESE `WebCacheV01.dat` | [Internet Explorer และ Edge รุ่นเก่า](../artifacts/webcache.md) |

## [คำสั่งสำหรับไฟล์](execution.md) { #file-commands }

| คำสั่ง | อาร์ติแฟกต์อินพุต | คู่มือโดยละเอียด |
|---|---|---|
| `parse-file defender-quarantine` | ทรีหรือรายการกักกันของ Defender | [ข้อมูลที่ Defender กักกัน](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [ทางลัด LNK](../artifacts/lnk.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [คำสั่ง NTFS](filesystem.md) { #ntfs-commands }

| คำสั่ง | อาร์ติแฟกต์อินพุต | คู่มือโดยละเอียด |
|---|---|---|
| `parse-ntfs boot` | `$Boot` หรืออิมเมจวอลุ่ม NTFS | [บูตเซกเตอร์ ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`) หรืออิมเมจดิสก์ | [ดัชนีไดเรกทอรี ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | NTFS `$LogFile` | [บันทึกธุรกรรม NTFS](../artifacts/logfile.md) |
| `parse-ntfs mft` | NTFS `$MFT` | [ตารางไฟล์หลัก](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [ตัวบอกความปลอดภัย ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [บันทึก USN](../artifacts/usn-journal.md) |

## [คำสั่งรีจิสทรี](security-registry.md) { #registry-commands }

| คำสั่ง | อาร์ติแฟกต์อินพุต | คู่มือโดยละเอียด |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [เครือข่าย](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [ความลับ LSA](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (และ `BCD-Template`) | [การกำหนดค่าบูต (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [ความเข้ากันได้ของแอปพลิเคชัน](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [นโยบายการตรวจสอบ](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage, RecentApps และรายการที่ปักหมุดบนแถบงาน](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [ประวัติแอปพลิเคชัน](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [รายการ MRU](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (พร้อม `SAM`, `SOFTWARE` สำหรับชื่อผู้ใช้) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (`NTUSER.DAT` บน Windows XP) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [โปรแกรมที่ติดตั้ง](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [กล่องโต้ตอบเปิดและบันทึกเป็น](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [การเริ่มอัตโนมัติผ่านรีจิสทรี](../artifacts/persistence.md) |
| `parse-registry read` | ไฮฟ์รีจิสทรีหลักใดก็ได้ | [ไฮฟ์รีจิสทรี](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (พร้อม `SOFTWARE` สำหรับ SID ของผู้ใช้) | [การเข้าถึงระยะไกล (ฝั่งผู้ใช้)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [บัญชีภายในเครื่อง SAM](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [การตั้งค่าความปลอดภัย](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [บริการ Windows](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` หรือค่า AppCompatCache ที่ส่งออกมา | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` ของระบบเดียว | [ข้อมูลระบบ](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [แคช Task Scheduler](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM` พร้อม `SOFTWARE` และ `NTUSER.DAT` | [อุปกรณ์ USB](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## [คำสั่ง SQLite](sqlite.md) { #sqlite-commands }

| คำสั่ง | อาร์ติแฟกต์อินพุต | คู่มือโดยละเอียด |
|---|---|---|
| `parse-sqlite browser` | `History`, `Cookies`, `Web Data`, `places.sqlite`, `cookies.sqlite`, `formhistory.sqlite` | [เว็บเบราว์เซอร์](../artifacts/browser.md) |

## ชนิดของแหล่งข้อมูลและการเปลี่ยนแปลงรูปแบบ { #source-types-and-format-changes }

กลุ่มต่าง ๆ อธิบายรูปแบบที่ Juntei รองรับในปัจจุบัน `parse-registry amcache` อ่าน
`Amcache.hve` ซึ่งไม่ได้หมายความว่ารองรับฐานข้อมูล inventory ของ Amcache แบบ SQLite crate ของพาร์เซอร์อาร์ติแฟกต์
ยังคงแยกจากกัน เพื่อให้สามารถเพิ่มการรองรับรูปแบบการจัดเก็บอื่นได้อย่างอิสระ

ชื่อพาร์เซอร์แบบแบนเดิมถูกแทนที่ด้วยพาธแบบแหล่งข้อมูลและอาร์ติแฟกต์ข้างต้นแล้ว ให้ใส่ตัวเลือกของพาร์เซอร์
ที่มีอยู่เดิมไว้หลังชื่ออาร์ติแฟกต์ `auto-parse` และคำสั่งย่อยรีจิสทรีที่มีอยู่เดิม
ยังคงใช้ชื่อเดิม

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` ล่าหาร่องรอยข้ามรูปแบบแหล่งข้อมูลหลายแบบ จึงเป็นคำสั่งวิเคราะห์ระดับบนสุด
คำสั่งนี้เสริม inventory การกำหนดค่าแบบครบถ้วนจาก `parse-registry persistence` ด้วย
การตรวจสอบ 131 รายการ การจำแนกแบบ allow/block เมทาดาทาของ payload ความครอบคลุม (coverage) และการเชื่อมโยง timestamp

## การใช้งานระดับบนสุด { #top-level-usage }

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
  parse-sqlite        Parse SQLite databases
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

หน้าแหล่งข้อมูลแสดงวิธีใช้คำสั่งและให้ตัวอย่างสั้น ๆ ส่วนคู่มืออาร์ติแฟกต์อธิบาย
หลักฐาน ฟิลด์เอาต์พุต ข้อจำกัด และกลยุทธ์การตรวจสอบ
