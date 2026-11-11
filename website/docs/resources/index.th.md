# บทความและแหล่งอ้างอิง { #articles-and-references }

หน้านี้รวบรวมจุดเริ่มต้นคุณภาพสูงสำหรับผู้อ่านที่ต้องการศึกษาเพิ่มเติมนอกเหนือจากคู่มือ
อาร์ติแฟกต์ ส่วน **รายละเอียดเพิ่มเติม** ของแต่ละอาร์ติแฟกต์มีลิงก์ระดับลึกที่ตรงประเด็นที่สุด

## แนะนำ DFIR สำหรับผู้เริ่มต้น { #beginner-dfir-orientation }

- [โปสเตอร์ SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/) — แผนภาพแสดงอาร์ติแฟกต์และตำแหน่งที่พบบ่อยของ Windows
- [โปสเตอร์ SANS Hunt Evil](https://www.sans.org/posters/hunt-evil/) — จุดเชื่อมโยงสำหรับการล่าภัยคุกคามในทางปฏิบัติและข้อเตือนใจในการตรวจสอบ Windows
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — ตัวอย่างอาร์ติแฟกต์สาธารณะสำหรับการเรียนรู้และการตรวจสอบความถูกต้องของพาร์เซอร์
- [ข้อมูลอ้างอิงอาร์ติแฟกต์ของ Velociraptor](https://docs.velociraptor.app/artifact_references/) — คำอธิบายที่เน้นการเก็บรวบรวมและการใช้งาน VQL สำหรับอาร์ติแฟกต์ Windows จำนวนมาก
- [เอกสาร Plaso](https://plaso.readthedocs.io/) — แนวคิดเรื่องไทม์ไลน์และพาร์เซอร์สำหรับการวิเคราะห์ข้ามอาร์ติแฟกต์

## อาร์ติแฟกต์ระบบไฟล์ { #file-system-artifacts }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) และ [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — security descriptor, ACE และ conditional expression
- [เอกสารรูปแบบไฟล์ของ libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [พาร์เซอร์ mft โดย Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — พาร์เซอร์ NTFS ที่รองรับเรคคอร์ด INDX และการกู้คืน (carving) จาก index slack
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — กู้คืนชื่อไฟล์จาก slack ของ `$I30`
- [LogFileParser](https://github.com/jschicht/LogFileParser) — งานวิจัยและการวิเคราะห์บันทึกธุรกรรม NTFS อย่างละเอียด
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — เชื่อมโยงประวัติจาก `$LogFile`, `$MFT` และ USN

## อาร์ติแฟกต์การเรียกใช้งานและรายการซอฟต์แวร์ { #execution-and-inventory-artifacts }

- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — งานด้านรูปแบบไฟล์/ไลบรารีของ Prefetch
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)

## อาร์ติแฟกต์กิจกรรมของผู้ใช้ { #user-activity-artifacts }

- [Microsoft Open Specifications: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## รีจิสทรีและการคงอยู่ { #registry-and-persistence }

- [Microsoft: ชนิดของค่ารีจิสทรี](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [ข้อกำหนดรูปแบบรีจิสทรีของ Windows โดย Maxim Suhanov](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) และ [libregf](https://github.com/libyal/libregf) — แหล่งอ้างอิงระดับล่างสำหรับการวิเคราะห์และกู้คืนไฮฟ์
- [Microsoft: โครงสร้างรีจิสทรีของบริการ](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: สคีมาของ Task Scheduler](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender และการจัดการมัลแวร์ { #defender-and-malware-handling }

- [Microsoft: กู้คืนไฟล์ที่ถูกกักกันใน Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [ปลั๊กอิน Defender ของ dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — การใช้งานสำหรับการวิจัย/การแยกไฟล์ที่พัฒนาแยกต่างหาก
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/) — แหล่งข้อมูลด้านการจัดการและการวิเคราะห์

## โปรเจกต์พาร์เซอร์ของ Juntei { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`, `$I30`, USN Journal, `$LogFile`, `$Boot`, `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "เนื้อหาภายนอกมีการเปลี่ยนแปลง"
    ให้ตรวจสอบข้อกล่าวอ้างเทียบกับเวอร์ชัน Windows ที่เกี่ยวข้องและข้อมูลที่คุณเก็บรวบรวมมา บทความที่มีประโยชน์อาจ
    อธิบายรุ่นของระบบปฏิบัติการ เวอร์ชันพาร์เซอร์ หรือการทดลองเฉพาะ มากกว่าจะเป็นกฎที่ใช้ได้ทั่วไป
