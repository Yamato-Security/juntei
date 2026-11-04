# กิตติกรรมประกาศของ Persistence Hunter { #persistence-hunter-acknowledgements }

คำสั่ง `persistence-hunter` ของ Juntei ต่อยอดมาจากงานล่าหากลไกการคงอยู่ (persistence hunting) ของชุมชนสองรุ่น
ขอขอบคุณผู้เขียนและผู้มีส่วนร่วมที่เผยแพร่เทคนิค การพัฒนา และงานวิจัยของตน
ให้ผู้ตรวจสอบได้ใช้งาน

## PyrsistenceSniper: โปรเจกต์ Python { #pyrsistencesniper-the-python-project }

[**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) โดย
[Hexastrike](https://github.com/Hexastrike) เป็นข้อมูลอ้างอิงโดยตรงของฟีเจอร์นี้ Juntei พอร์ต
แคตตาล็อกการตรวจสอบ 130 รายการมาเป็น Rust แบบเนทีฟ พร้อมกับ allowlist เริ่มต้น สแนปช็อต LOLBAS และ
พฤติกรรมการจัดประเภท เวอร์ชันอ้างอิงถูกตรึงไว้ที่
[commit `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f)

โปรเจกต์ Python นี้ใช้สัญญาอนุญาต MIT Juntei เก็บรักษาประกาศลิขสิทธิ์และสัญญาอนุญาตไว้ใน
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt)
การพัฒนาของ Juntei ทำงานแบบออฟไลน์โดยไม่ต้องใช้ Python interpreter การจัดการชุดข้อมูลที่เก็บรวบรวม
การรายงานความครอบคลุม และการตีความหลักฐานที่ไม่สมบูรณ์อย่างระมัดระวังมีอธิบายไว้ใน
[คู่มือคำสั่ง](../commands/persistence-hunter.md)

## PersistenceSniper: เครื่องมือ PowerShell ต้นฉบับ { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) คือโปรเจกต์ล่าหากลไกการคงอยู่ด้วย PowerShell
ต้นฉบับที่เป็นแรงบันดาลใจให้ PyrsistenceSniper โดย PyrsistenceSniper ให้เครดิตแก่
Federico Lagrasta ส่วนรีโพซิทอรีต้นฉบับยังให้เครดิตแก่ Riccardo Ancarani และผู้มีส่วนร่วม
คนอื่น ๆ ด้วย เราขอขอบคุณโปรเจกต์ต้นฉบับและผู้มีส่วนร่วมที่ทำให้การตรวจสอบกลไกการคงอยู่ของ Windows
ทำได้ง่ายขึ้น

ข้อมูลอ้างอิงในการพอร์ตโดยตรงของ Juntei คือโปรเจกต์ Python ส่วนเครื่องมือ PowerShell ต้นฉบับได้รับเครดิต
ในฐานะต้นสายของงานนี้ Juntei ยังพัฒนากลไกที่เสนอโดย
[hongson11698](https://github.com/hongson11698) ในโปรเจกต์ต้นฉบับขึ้นเองอย่างอิสระ ได้แก่
[PR #31 การ resolve COM ของงานตามกำหนดเวลา](https://github.com/last-byte/PersistenceSniper/pull/31)
และ [PR #32 ไลบรารี/โฟลเดอร์ CLSID ของ Start Menu](https://github.com/last-byte/PersistenceSniper/pull/32)
สิ่งเหล่านี้เป็นส่วนเพิ่มเติมจากแคตตาล็อกอ้างอิงของ Python ซึ่งไม่ได้หมายความว่าทุกฟีเจอร์สำหรับระบบที่กำลังทำงานในโปรเจกต์ต้นทางทั้งสอง
จะถูกพัฒนาไว้ในคำสั่งแบบออฟไลน์ของ Juntei

## งานวิจัยและเครื่องมือที่สนับสนุน { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/) เป็นแหล่งของตัวระบุเทคนิคที่ใช้ใน
  แคตตาล็อก การจับคู่เทคนิคอธิบายการใช้งานที่เป็นไปได้ของกลไก ค่ารีจิสทรีหรือไฟล์ที่ตรงกัน
  เพียงลำพังไม่ใช่การโจมตีที่ ATT&CK ยืนยันแล้ว
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  ได้รับการกล่าวถึงจาก PyrsistenceSniper ว่าเป็นแรงบันดาลใจในการตรวจสอบการเริ่มอัตโนมัติของ Windows ด้วย
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) รวบรวมเอกสารของไบนารี Windows ที่ถูกต้องตามกฎหมายซึ่ง
  อาจถูกนำไปใช้ในทางที่ผิดได้ การรู้จักไบนารีเหล่านี้ช่วยในการตีความคำสั่ง แต่ไม่ได้ทำให้
  คำสั่งนั้นเป็นอันตรายโดยอัตโนมัติ

สำหรับพาร์เซอร์พื้นฐานด้านรีจิสทรี NTFS, Shell Link, event log และอื่น ๆ ที่ใช้ทั่วทั้ง Juntei
ดู[กิตติกรรมประกาศของทั้งโปรเจกต์](../acknowledgements.md)

กลับไปยัง[คู่มือกลไกการคงอยู่](index.md)
