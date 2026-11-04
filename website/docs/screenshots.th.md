# ภาพหน้าจอ { #screenshots }

สำรวจบรรทัดคำสั่ง การวิเคราะห์อัตโนมัติ และผลการตรวจสอบที่จัดระเบียบไว้ของ Juntei
คลิกภาพหน้าจอเพื่อดูในขนาดเต็ม

## 1. หน้าจอเริ่มต้นและคำสั่ง { #1-startup-and-commands }

<figure markdown>

[![หน้าจอเริ่มต้นของ Juntei แสดงรุ่น CODE BLUE และคำสั่งที่ใช้ได้](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>หน้าจอเริ่มต้นของ Juntei แสดงรายการการวิเคราะห์อัตโนมัติ พาร์เซอร์เฉพาะแหล่งข้อมูล และ Persistence Hunter</figcaption>

</figure>

## 2. การวิเคราะห์อัตโนมัติในเวลาไม่ถึง 20 วินาที { #2-automatic-parsing-in-under-20-seconds }

บน **MacBook Pro M4** การทำงานที่แสดงด้านล่างได้วิเคราะห์ **ข้อมูล Windows ที่เก็บรวบรวมมา 8.7 GiB**
จาก **หกระบบ** และบันทึก **เอาต์พุต 4.4 GiB ในเวลาไม่ถึง 20 วินาที**
คำสั่ง [auto-parse](commands/auto-parse.md) เรียกใช้งานพาร์เซอร์ 240 งานด้วยโพรเซสแบบขนาน 16
โพรเซส และสร้างไฟล์ผลลัพธ์ 482 ไฟล์ โดยมี **งานที่สำเร็จ 210 งาน ถูกข้าม 30 งาน
เนื่องจากไม่ได้เก็บรวบรวมอาร์ติแฟกต์ และไม่มีงานที่ล้มเหลวเลย**

<figure markdown>

[![Juntei auto-parse ทำงานเสร็จสำหรับหกระบบ อินพุต 8.7 GiB และเอาต์พุต 4.4 GiB ในเวลาไม่ถึง 20 วินาที](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>การค้นหาโฮสต์อัตโนมัติ การวิเคราะห์แบบขนาน และสรุปเมื่อเสร็จสิ้นที่แสดงขนาดอินพุต ขนาดเอาต์พุต ผลลัพธ์ของงาน และเวลาที่ใช้</figcaption>

</figure>

## 3. ผลลัพธ์ที่จัดระเบียบไว้สำหรับการตรวจสอบ { #3-results-organized-for-investigation }

<figure markdown>

[![โฟลเดอร์เอาต์พุตของระบบหนึ่งที่จัดกลุ่มตามหมวดหมู่ DFIR พร้อม CSV ของบัญชีและ log ของพาร์เซอร์](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>ผลลัพธ์ของคอมพิวเตอร์แต่ละเครื่องถูกจัดกลุ่มตามหมวดหมู่ DFIR ไฟล์ CSV ของอาร์ติแฟกต์และไฟล์ <code>-logs.txt</code> ที่คู่กันจะอยู่ด้วยกัน ทำให้ค้นหาได้ง่ายทั้งระเบียนและรายละเอียดการวิเคราะห์</figcaption>

</figure>
