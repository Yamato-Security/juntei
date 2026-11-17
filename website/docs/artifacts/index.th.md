# คู่มืออาร์ติแฟกต์ { #artifact-guide }

Windows เก็บรักษาร่องรอยที่ซ้อนทับกันไว้ในระบบไฟล์ รีจิสทรี ฐานข้อมูลความเข้ากันได้
ประวัติของส่วนติดต่อผู้ใช้ การบันทึกการใช้ทรัพยากร และผลิตภัณฑ์ความปลอดภัย Juntei วิเคราะห์แหล่งข้อมูลเหล่านี้
แต่ผู้ตรวจสอบยังคงต้องตีความว่าแหล่งข้อมูลแต่ละแหล่งมีความหมายอย่างไร

## เลือกอาร์ติแฟกต์ตามคำถาม { #choose-an-artifact-by-question }

| คำถามในการตรวจสอบ | เริ่มต้นที่ | ยืนยันด้วย |
|---|---|---|
| ไฟล์ปฏิบัติการนี้เคยทำงานหรือไม่ | [Prefetch](prefetch.md) | [ShimCache](shimcache.md), [SRUM](srum.md), event log |
| ไฟล์นี้เคยมีอยู่หรือไม่ | [Amcache](amcache.md), [$MFT](mft.md) | [USN Journal](usn-journal.md), [$I30](i30.md), ShimCache |
| ไฟล์ถูกสร้าง เปลี่ยนชื่อ หรือลบเมื่อใด | [USN Journal](usn-journal.md) | [$MFT](mft.md), [$I30](i30.md), ข้อมูลที่ Defender กักกัน |
| ผู้ใช้ลบอะไรลงถังรีไซเคิล และลบเมื่อใด | [ถังรีไซเคิล](recycle-bin.md) | [USN Journal](usn-journal.md), [$MFT](mft.md), [LNK](lnk.md), [Jump Lists](jump-lists.md) |
| ชื่อหรือ timestamp ใดมีอยู่ก่อนเมทาดาทาจะเปลี่ยนแปลง | [$LogFile](logfile.md) | [$MFT](mft.md), [USN Journal](usn-journal.md), [$I30](i30.md) |
| ใครเป็นเจ้าของไฟล์ และใครสามารถเปลี่ยนแปลงไฟล์ได้ | [$Secure:$SDS](secure-sds.md) ร่วมกับ [$MFT](mft.md) | [SAM](sam.md), Security event log |
| เป้าหมายของทางลัดหรือไฟล์ Prefetch อยู่บนวอลุ่มใด | [$Boot](boot.md) | [LNK](lnk.md), [Prefetch](prefetch.md), [อุปกรณ์ USB](usb.md) |
| ผู้ใช้เปิดอะไรบ้าง | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| โปรแกรมถ่ายโอนข้อมูลปริมาณมากหรือไม่ | [SRUM](srum.md) | telemetry จากไฟร์วอลล์ พร็อกซี DNS แพ็กเก็ต หรือ EDR |
| ผู้ใช้เข้าชมเว็บไซต์ใดบ้าง และดาวน์โหลดอะไร | [เว็บเบราว์เซอร์](browser.md), [Internet Explorer และ Edge รุ่นเก่า](webcache.md) | บันทึกของพร็อกซีและ DNS, `$MFT` และ USN Journal, Defender, SRUM |
| ผู้ใช้รับส่งอีเมลกับใคร และมีอะไรแนบมาบ้าง | [Outlook](outlook.md) | บันทึกของเมลเซิร์ฟเวอร์และเกตเวย์, `$MFT` และ USN Journal, Defender, LNK และ Jump Lists |
| ไคลเอนต์และบัญชีใดใช้งานเซิร์ฟเวอร์นี้ และในวันใดบ้าง | [การบันทึกการเข้าถึงของผู้ใช้ (UAL)](ual.md) | Security event log, บันทึกของ SMB และ RDP, บันทึกของไฟร์วอลล์ |
| ผู้ใช้เห็นอะไรบนคอมพิวเตอร์ที่เชื่อมต่อด้วย Remote Desktop | [แคชบิตแมปของ RDP](rdp-cache.md) | [การเข้าถึงระยะไกล (ฝั่งผู้ใช้)](remote-access.md), [Jump Lists](jump-lists.md), เหตุการณ์การเข้าสู่ระบบบนคอมพิวเตอร์ระยะไกล และ [UAL](ual.md) |
| มีการกำหนดค่าการคงอยู่ (persistence) หรือไม่ | [บริการ](services.md), [TaskCache](taskcache.md), [รีจิสทรี](registry.md) | Event log, Prefetch, `$MFT` |
| Defender ลบอะไรออกไปบ้าง | [ข้อมูลที่ Defender กักกัน](defender-quarantine.md) | Event log, `$MFT`, USN Journal |
| มีการเพิ่มหรือเปลี่ยนแปลงบัญชีภายในเครื่องหรือไม่ | [SAM](sam.md) | Security event log และประวัติ transaction ของรีจิสทรี |

## อ่านคำเตือนก่อนอ่านฟิลด์ { #read-warnings-before-fields }

หน้าอาร์ติแฟกต์ทุกหน้ามีลำดับเนื้อหาเหมือนกัน:

1. **อาร์ติแฟกต์คืออะไร** และตอบคำถามใดได้บ้าง
2. **พบได้ที่ใด** และการเก็บรวบรวมส่งผลต่ออาร์ติแฟกต์อย่างไร
3. **พิสูจน์อะไรได้ และพิสูจน์อะไรไม่ได้**
4. **วิธีวิเคราะห์** ด้วย Juntei
5. **วิธีอ่านฟิลด์เอาต์พุตทุกฟิลด์**
6. **วิธีเชื่อมโยง** กับหลักฐานอิสระ
7. **รายละเอียดเพิ่มเติม** สำหรับการศึกษาเชิงลึก

!!! danger "อย่าถือว่าสิ่งที่พาร์เซอร์ตรวจพบเท่ากับความเป็นอันตราย"
    ข้อค้นพบของ Juntei ระบุรูปแบบที่ควรตรวจทาน เช่น ชื่อไบนารีของระบบที่อยู่นอก
    `System32` ตัวติดตั้ง ผู้ดูแลระบบ compatibility layer และการอัปเดตซอฟต์แวร์สามารถสร้าง
    เรคคอร์ดที่ผิดปกติได้ ให้ตรวจสอบความถูกต้องด้วยบริบทและหลักฐานอิสระ
