# กิตติกรรมประกาศ { #acknowledgements }

Juntei เกิดขึ้นได้เพราะชุมชน DFIR และชุมชนโอเพนซอร์สได้เผยแพร่พาร์เซอร์ งานวิจัยด้านรูปแบบไฟล์
เอาต์พุตอ้างอิง ข้อกำหนด และหลักฐานสำหรับการทดสอบ โปรเจกต์ด้านล่างมีส่วนช่วยในการพัฒนา การตรวจสอบความถูกต้อง
การตั้งชื่อฟิลด์ หรือการกำหนดเอาต์พุตที่นักวิเคราะห์คาดหวัง การมีชื่ออยู่ในรายการนี้ไม่ได้หมายความว่าโค้ดของโปรเจกต์นั้น
ถูกรวมอยู่ใน Juntei โปรดตรวจสอบสัญญาอนุญาตของแต่ละโปรเจกต์ด้วยตนเอง

## พาร์เซอร์พื้นฐานที่ดูแลโดย Yamato Security { #parser-foundations-maintained-by-yamato-security }

| ส่วนของ Juntei | พาร์เซอร์ที่ดูแล | ต้นทาง/พื้นฐาน |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `mft`) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| USN Journal | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `usn`) | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `logfile`) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| ดัชนีไดเรกทอรี (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `mft`, โมดูล `i30`) | งานวิจัยด้าน index slack ที่ระบุไว้ด้านล่าง |
| NTFS `$Boot`, `$Secure:$SDS` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `boot` และ `secure`) | MS-DTYP ของ Microsoft และงานวิจัย NTFS ที่ระบุไว้ด้านล่าง |
| SRUM, UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) และ [ese](https://github.com/Sootmark/ese) |
| เว็บเบราว์เซอร์ / SQLite | [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) | [Sootmark browser](https://github.com/Sootmark/browser), [sqlite](https://github.com/Sootmark/sqlite) และ [plist](https://github.com/Sootmark/plist) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | งานด้าน compound file และ Jump List ของ Sootmark shell |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| อิมเมจดิสก์ VHDX (`mount`, `auto-parse --vhdx`) | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) และ [ntfs-core](https://github.com/SecurityRonin/ntfs-forensic) ให้บริการไฟล์ผ่าน [fuser](https://github.com/cberner/fuser) | MS-VHDX ของ Microsoft |
| E01 / Ex01 (`mount`, `auto-parse --e01`) | [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) | [ewf-image](https://github.com/ebrig/ewf-image) (Apache-2.0); [libewf](https://github.com/libyal/libewf) (EWF1/EWF2) |
| รีจิสทรี | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | งานวิจัยด้านรูปแบบรีจิสทรีและการกู้คืนข้อมูลที่ระบุไว้ด้านล่าง |

## เครื่องมืออ้างอิงและการตรวจสอบโดยอิสระ { #reference-tools-and-independent-validation }

- [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) โดย Hexastrike
  เป็นแหล่งที่มาของแคตตาล็อกการตรวจสอบการคงอยู่ (persistence) allowlist เริ่มต้น สแนปช็อต LOLBAS และพฤติกรรมอ้างอิง
  สำหรับคำสั่ง `persistence-hunter` ที่เขียนด้วย Rust แบบเนทีฟ สัญญาอนุญาต MIT และประกาศลิขสิทธิ์ของโปรเจกต์นี้
  ถูกเก็บรักษาไว้ใน `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt` ส่วน
  [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) ต้นฉบับเป็นแรงบันดาลใจให้กับโปรเจกต์ Python ดังกล่าว
  ดูประวัติความเป็นมาทั้งหมดได้ที่[กิตติกรรมประกาศของ Persistence Hunter](persistence-hunter/acknowledgements.md)

- **ข้อกำหนด regf** และ **yarp** ของ Maxim Suhanov มีส่วนช่วยในเรื่องโครงสร้างรีจิสทรี การเล่นซ้ำ (replay) บันทึกธุรกรรม
  และการกู้คืนเรคคอร์ดที่ถูกลบ ส่วน **dfir_ntfs** ของเขาเป็นข้อมูลอ้างอิงสำหรับเรคคอร์ด INDX และการกู้คืน (carving) ข้อมูลจาก index slack
- **INDXRipper** ของ Harel Segev เป็นอีกหนึ่งข้อมูลอ้างอิงสำหรับการกู้คืนชื่อไฟล์จาก index slack
- **dfir_ntfs**, **LogFileParser** ของ Joakim Schicht และ **ntfs-linker** ของ Stroz Friedberg มีส่วนช่วยในการตรวจสอบความถูกต้อง
  และการตีความ `$LogFile` ส่วน **go-ntfs** ของ Velocidex เป็นแหล่งหลักฐาน NTFS สาธารณะสำหรับการทดสอบ
- โปรเจกต์ของ Joachim Metz / libyal ได้แก่ **libregf**, **libfsntfs**, **libscca**, **libfwsi** และ
  **libesedb** ให้ข้อมูลวิจัยด้านรูปแบบไฟล์อย่างละเอียดและเป็นข้อมูลอ้างอิงการพาร์สโดยอิสระ
- **dissect.esedb**, **dissect.ntfs** และ **dissect.target** ของ Fox-IT ให้การตีความ SRUM, UAL,
  `$Secure:$SDS`, SAM และ Defender โดยอิสระ ซึ่งใช้ระหว่างการตรวจสอบความถูกต้อง
- `fsstat` ของ **The Sleuth Kit** เป็นข้อมูลอ้างอิงสำหรับบูตเซกเตอร์ NTFS ของวอลุ่มที่สร้างด้วย
  `mkntfs`
- เครื่องมือบางส่วนของ Eric Zimmerman เป็นที่มาของชื่อฟิลด์และความคาดหวังของผู้ตรวจพิสูจน์ที่ใช้ในการเปรียบเทียบ
- **regipy**, **python-registry**, **Notatin** และ **artemis** ให้การพัฒนาพาร์เซอร์รีจิสทรี
  ชุดทดสอบ หรือตัวอย่างการตีความ
- **LnkParse3**, **lnk_parser** และ **jumplist_parser** ของ AbdulRhmanAlfaifi และ JLECmd ให้พฤติกรรม
  LNK/Jump List โดยอิสระและกรณีทดสอบสาธารณะ
- **prefetch-forensic** ของ Security Ronin และโปรเจกต์ **forensicnomicon** มีส่วนช่วยในข้อสังเกตที่มีคุณค่าสูง
  ด้าน Prefetch/Amcache/ShimCache ของ Juntei
- **defender-dump** และ dissect.target เป็นการพัฒนาแยกต่างหากที่ช่วยให้เข้าใจโครงสร้างการกักกัน (quarantine) ของ Defender
  และพฤติกรรมการแยกไฟล์อย่างปลอดภัย
- **Plaso**, **Chainsaw** และ **Velociraptor** ให้การเปรียบเทียบพาร์เซอร์ หลักฐานสำหรับการทดสอบ หรือพฤติกรรมอ้างอิง
  ที่เน้นการเก็บรวบรวมข้อมูลสำหรับอาร์ติแฟกต์หลายประเภท
- **Hindsight** ของ Obsidian Forensics และนิยามของ **SQLiteHunter** ของ Velocidex เป็นข้อมูลอ้างอิงสำหรับ
  ประวัติ การดาวน์โหลด คุกกี้ และประวัติฟอร์มของเบราว์เซอร์
- **rifiuti2** ของ Abel Cheung เป็นข้อมูลอ้างอิงสำหรับไฟล์ `$I` และ INFO2 ของถังรีไซเคิล ร่วมกับเอกสาร
  **dtformats** ของ libyal ที่อธิบายทั้งสองรูปแบบ
- **bmc-tools** ของ ANSSI และประกาศของ CERT-FR เกี่ยวกับรูปแบบนี้, **RdpCacheStitcher** ของ BSI,
  อาร์ติแฟกต์ RDPCache ของ **Velociraptor** และปลั๊กอินแคช RDP ของ **dissect.target** เป็นข้อมูลอ้างอิง
  สำหรับแคชบิตแมปของ RDP ร่วมกับแคชบิตแมปแบบถาวรและตัวถอดรหัส interleaved RLE ของ **FreeRDP**

## ข้อกำหนดและงานวิจัย { #specifications-and-research }

เราขอขอบคุณ Microsoft ที่เผยแพร่ Open Specifications สำหรับ Shell Links, Compound Binary Files,
โปรโตคอล NTFS/SAM, โครงสร้างรีจิสทรี/บริการ/งานตามกำหนดเวลา, Remote Desktop Protocol และเอกสาร API นอกจากนี้เรายังขอขอบคุณ
ผู้เขียนงานวิจัยด้านนิติวิทยาศาสตร์ดิจิทัลเกี่ยวกับ AppCompatCache, Amcache, SRUM, Prefetch, รีจิสทรี และ NTFS ที่ทำให้
ผู้ปฏิบัติงานสามารถเข้าใจอาร์ติแฟกต์เหล่านี้ได้

## หลักฐานสำหรับการทดสอบ { #test-evidence }

หลักฐานสาธารณะจาก
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) ของ Andrew Rathbun, NIST CFReDS, Plaso,
rifiuti2, Regipy, Artemis, รีโพซิทอรีชุดทดสอบของพาร์เซอร์ต่าง ๆ และคอลเลกชันอื่น ๆ ที่มีสัญญาอนุญาตอย่างถูกต้อง ทำให้สามารถทำ
regression test กับข้อมูล Windows จริงได้ Juntei เก็บหมายเหตุด้านการระบุแหล่งที่มา/สัญญาอนุญาตไว้คู่กับ
ไฟล์ทดสอบที่รวมไว้

## ผู้มีส่วนร่วม { #contributors }

ขอขอบคุณผู้มีส่วนร่วมพัฒนาพาร์เซอร์ของ Juntei และ Yamato Security ผู้รีวิว ผู้รายงานปัญหา และ
ผู้ใช้ทุกคนที่ให้กรณีพิเศษ (edge case) มา รายชื่อผู้มีส่วนร่วมในปัจจุบันแสดงโดย GitHub ที่
[หน้าผู้มีส่วนร่วมของ Juntei](https://github.com/Yamato-Security/juntei/graphs/contributors)

## สัญญาอนุญาต { #license }

Juntei เผยแพร่ภายใต้
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) และเป็นลิขสิทธิ์ของ Shirofune Security Co., Ltd. ซอร์สโค้ดของ Juntei เคยเผยแพร่ภายใต้
AGPLv3 ก่อนเวอร์ชัน 1.0.0 สัญญาอนุญาตของ crate และโค้ดและข้อมูลของบุคคลที่สามอื่น ๆ ที่ใช้ในการสร้าง Juntei
อยู่ใน
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES)
