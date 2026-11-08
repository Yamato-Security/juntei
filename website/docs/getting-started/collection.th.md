# การเก็บรวบรวมหลักฐาน { #collecting-evidence }

Juntei พาร์สไฟล์ที่รวบรวมจาก Windows แล้ว ใช้เครื่องมือเก็บหลักฐานที่เชื่อถือได้และรักษาเมทาดาทาต้นฉบับเมื่อทำได้ อิมเมจดิสก์ VHDX สามารถพาร์สได้โดยไม่ต้องแยกไฟล์ ดู[การเมานต์อิมเมจดิสก์](../commands/mount.md)

## ตำแหน่งทั่วไป { #common-locations }

| อาร์ติแฟกต์ | ตำแหน่งทั่วไป |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | ไฟล์เมทาดาทาของวอลุ่ม NTFS `C:\$MFT` |
| `$LogFile` | ไฟล์เมทาดาทาของวอลุ่ม NTFS `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| USN Journal | สตรีมเมทาดาทาของ NTFS `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| การบันทึกการเข้าถึงของผู้ใช้ (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| เว็บเบราว์เซอร์ | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\` และโฟลเดอร์โปรไฟล์ของเบราว์เซอร์อื่น ๆ |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` และตำแหน่งอื่นของผู้ใช้/ระบบ |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` และ `CustomDestinations` |
| ShimCache | ไฮฟ์ `SYSTEM`: `...\Session Manager\AppCompatCache\AppCompatCache` |
| การกักกันของ Defender | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| รีจิสทรีระดับเครื่อง | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| รีจิสทรีของผู้ใช้ | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## ไฟล์ที่ต้องเก็บรวบรวมไปด้วยกัน { #files-that-must-travel-together }

- เก็บรวบรวมไฮฟ์รีจิสทรีพร้อมไฟล์ `.LOG`, `.LOG1` และ `.LOG2` ที่อยู่ข้างกัน Juntei สามารถเล่นซ้ำบันทึก
  สำหรับคำสั่งรีจิสทรีและ ShimCache ได้เมื่อไฮฟ์อยู่ในสถานะ dirty
- เก็บรวบรวม `SRUDB.dat` ของ SRUM ในสถานะที่สอดคล้องกัน ปัจจุบัน Juntei รายงานฐานข้อมูล ESE ที่อยู่ในสถานะ dirty แต่
  ไม่ได้เล่นซ้ำ `SRU*.log` เรคคอร์ดที่มีอยู่เฉพาะในบันทึกเหล่านั้นจึงไม่สามารถใช้ได้
- เก็บรวบรวม `$MFT` ไปพร้อมกับ `$Extend/$J` เมื่อทำได้ เรคคอร์ด USN จัดเก็บ parent file reference
  ไม่ใช่พาธที่สมบูรณ์ MFT จะช่วยให้ resolve พาธได้ดีขึ้น
- เก็บรวบรวม `$LogFile` พร้อมกับ `$MFT` ของวอลุ่มเดียวกันในเวลาที่ใกล้เคียงกันที่สุด `$LogFile` วนทับ
  อย่างรวดเร็ว การเก็บรวบรวมที่ล่าช้าอาจทำให้เรคคอร์ด redo/undo ก่อนหน้าสูญหายไปทั้งหมด
- เก็บรวบรวม SRUM พร้อมไฮฟ์ `SOFTWARE` ที่ตรงกันเพื่อ resolve ชื่อบัญชีและชื่อโปรไฟล์ Wi-Fi
- เก็บรวบรวมไดเรกทอรี `Sum` ของการบันทึกการเข้าถึงของผู้ใช้ทั้งหมด เพราะ `SystemIdentity.mdb` ระบุชื่อบทบาทและ
  ฐานข้อมูล `{GUID}.mdb` รายปี นอกจากนี้ Juntei ก็ไม่ได้เล่นซ้ำบันทึกธุรกรรม `Svc*.log` ของ UAL เช่นกัน
- เก็บรวบรวมฐานข้อมูลของเบราว์เซอร์พร้อมไฟล์ `-wal` ที่อยู่ข้างกัน (`places.sqlite-wal`,
  `cookies.sqlite-wal`) เพราะการเปลี่ยนแปลงล่าสุด และเพจเวอร์ชันเก่าที่ใช้กู้คืนรายการที่ถูกลบ
  จะอยู่ในไฟล์เหล่านั้นจนกว่าเบราว์เซอร์จะทำ checkpoint เท่านั้น
- เก็บรวบรวมโครงสร้าง `Quarantine` ของ Defender ทั้งหมด เพื่อให้จับคู่ Entries, ResourceData และ Resources ได้

## ข้อควรระวังสำหรับระบบที่กำลังทำงาน { #live-system-cautions }

ไฟล์ที่เปิดอยู่อาจเปลี่ยนแปลงระหว่างการคัดลอก ไฮฟ์รีจิสทรีและฐานข้อมูล ESE อาจอยู่ในสถานะ dirty ภายใน
ShimCache อาจยังไม่ flush รายการล่าสุดในหน่วยความจำจนกว่าจะปิดเครื่อง เครื่องมือเก็บรวบรวมทางนิติวิทยาศาสตร์
ที่ใช้ volume snapshot หรือการเข้าถึงข้อมูลดิบดีกว่าการคัดลอกไฟล์ตามปกติ

## ความปลอดภัย { #safety }

หลักฐานอาจมีมัลแวร์ ตัวอย่าง exploit ไฟล์ LNK ที่เป็นอันตราย และข้อความที่เป็นสูตรสเปรดชีต
ให้วิเคราะห์บนระบบที่แยกออกจากเครือข่าย อย่าเรียกใช้สตรีมที่กู้คืนมา และใช้ `--extract` เฉพาะเมื่อคุณ
ตั้งใจจะจัดการกับเนื้อหาที่เป็นอันตราย ให้เก็บรักษาต้นฉบับที่ไม่ได้แตะต้องแยกไว้ต่างหาก
