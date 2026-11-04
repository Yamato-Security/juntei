# การเข้าถึงระยะไกล (ฝั่งผู้ใช้) { #remote-access-user-side }

## สิ่งนี้คืออะไร { #what-it-is }

`NTUSER.DAT` ของผู้ใช้เก็บร่องรอยของการเชื่อมต่อระยะไกลที่ผู้ใช้ทำจากคอมพิวเตอร์เครื่องนั้น
ซึ่งเป็นฝั่งต้นทางของการเคลื่อนย้ายภายในเครือข่าย:

- **Remote Desktop (`mstsc`):**
  - คอมพิวเตอร์ที่พิมพ์ลงในไคลเอนต์ โดยรายการล่าสุดอยู่ก่อน
  - คีย์หนึ่งคีย์ต่อคอมพิวเตอร์แต่ละเครื่องที่ผู้ใช้ลงชื่อเข้าใช้ พร้อมชื่อบัญชีที่ใช้และ thumbprint
    ของใบรับรองเซิร์ฟเวอร์ที่ผู้ใช้ยอมรับ
  - คอมพิวเตอร์ที่ผู้ใช้อนุญาตให้ใช้อุปกรณ์ภายในเครื่อง
- **PuTTY:**
  - เซสชันที่บันทึกไว้: โฮสต์ พอร์ต บัญชี โปรโตคอล ไฟล์คีย์ พร็อกซี และการส่งต่อพอร์ต (port forwarding)
  - เซสชันที่เริ่มล่าสุด
  - คีย์เซิร์ฟเวอร์ SSH ทุกคีย์ที่ผู้ใช้ยอมรับ
- **WinSCP:** เซสชันที่บันทึกไว้และคีย์เซิร์ฟเวอร์ SSH ที่ยอมรับ ในรูปแบบเดียวกับของ PuTTY
- **ไดรฟ์ที่แมป:** ไดรฟ์แต่ละตัวที่แมปไว้ให้คืนค่าเมื่อลงชื่อเข้าใช้ และพาธที่พิมพ์ลงใน
  *Map Network Drive*
- **แชร์:** แชร์ที่ Explorer เมานต์ไว้ (`MountPoints2`)

ร่องรอยเหล่านี้แสดงว่าบัญชีไปที่ใดต่อและด้วยข้อมูลประจำตัว (credential) ใด แม้ว่า log ของคอมพิวเตอร์
ระยะไกลเองจะหายไปแล้วก็ตาม

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> โปรไฟล์ของผู้ใช้นี้เชื่อมต่อ หรือบันทึก
การเชื่อมต่อ ไปยังเป้าหมายนี้ เซสชันที่บันทึกไว้หรือชื่อที่พิมพ์เพียงอย่างเดียวไม่ได้พิสูจน์ว่า
การเชื่อมต่อสำเร็จ host key ที่ยอมรับแล้วหรือคีย์ `Servers` ที่มี `UsernameHint` เป็นหลักฐานที่หนักแน่นกว่า:
ไคลเอนต์ได้ติดต่อถึงเซิร์ฟเวอร์แล้ว

## ตำแหน่ง { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Terminal Server Client\Default            MRU0 … MRU9
Software\Microsoft\Terminal Server Client\Servers\<host>     UsernameHint, CertHash
Software\Microsoft\Terminal Server Client\LocalDevices       <host> = devices
Software\SimonTatham\PuTTY\Sessions\<session>
Software\SimonTatham\PuTTY\Jumplist                          Recent sessions
Software\SimonTatham\PuTTY\SshHostKeys                       <type>@<port>:<host> = key
Software\Martin Prikryl\WinSCP 2\Sessions\<session>
Software\Martin Prikryl\WinSCP 2\SshHostKeys
Network\<letter>                                             RemotePath, UserName, ProviderName
Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU
Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##server#share
```

ให้เก็บรวบรวม `NTUSER.DAT` พร้อม transaction log ของทุกโปรไฟล์ และเก็บรวบรวม `SOFTWARE`
ของระบบด้วย เพราะไฟล์นี้ให้ SID ของผู้ใช้แต่ละคน

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **เวลาของคีย์คือเวลาของรายการ** `Default`, `SshHostKeys` และ `Map Network Drive MRU` เก็บรายการ
  และเวลาเขียนล่าสุดของคีย์คือเวลาที่รายการเปลี่ยนแปลงครั้งล่าสุด เวลานั้นเป็นของรายการแรก
  (`MruPosition` 0) สำหรับรายการอื่นเป็นเพียงขอบเขตบน คีย์ `Servers\<host>`, เซสชัน,
  `Network\<letter>` หรือแชร์ มีเวลาของตัวเอง
- **บันทึกไว้ไม่ได้แปลว่าใช้งาน** เซสชันของ PuTTY หรือ WinSCP สามารถบันทึกไว้โดยไม่เคยเชื่อมต่อเลย PuTTY
  เขียน `Recent sessions` ใหม่เมื่อเริ่มเซสชันที่บันทึกไว้ และเพิ่ม host key เมื่อติดต่อเซิร์ฟเวอร์
  เป็นครั้งแรก
- **รหัสผ่าน:**
  - WinSCP สามารถเก็บรหัสผ่านของเซสชัน tunnel และพร็อกซีได้
  - PuTTY เก็บเฉพาะรหัสผ่านของพร็อกซีเป็นข้อความธรรมดา
  - `PasswordStored` และ `StoredPasswords` ของ `Details` บอกว่ามีการตั้งค่าใดไว้บ้าง ค่าเหล่านี้จะไม่ถูก
    ถอดรหัสหรือเขียนออกมาเลย
- **WinSCP ละเว้นค่าเริ่มต้น** เซสชันที่ไม่มี `FSProtocol` คือ SFTP และเซสชันที่ไม่มี
  `PortNumber` ใช้พอร์ตเริ่มต้นของโปรโตคอล ในกรณีนั้น `Protocol` และ `Port` จะว่างไว้แทนที่
  จะเดาค่า
- **fingerprint ของ host key** อยู่ในรูปแบบ `SHA256:…` ของ OpenSSH ตามที่ `ssh-keygen -l`, `ssh` และ PuTTY 0.75
  ขึ้นไปแสดง Juntei คำนวณค่านี้จากคีย์ที่ PuTTY และ WinSCP เก็บไว้ ให้เปรียบเทียบกับ
  คีย์ของเซิร์ฟเวอร์เอง ไฟล์ `known_hosts` และ log เพื่อระบุเซิร์ฟเวอร์ที่อยู่เบื้องหลังชื่อหรือ
  IP address คีย์ SSH-1 (`rsa`) ไม่มีค่านี้
- **ไดรฟ์ที่แมป:** `RemoteUsername` จะถูกตั้งค่าเฉพาะเมื่อแมปไดรฟ์ด้วยบัญชีอื่น
  Windows จะเขียน `REG_DWORD` 0 เมื่อใช้บัญชีของผู้ใช้เอง
- **แชร์:** `MountPoints2` ยังมีคีย์สำหรับวอลุ่มและอักษรไดรฟ์ด้วย ในที่นี้รายงานเฉพาะแชร์บนเครือข่าย
  (`##server#share`) เท่านั้น `\\server@SSL@443\DavWWWRoot` คือแชร์ WebDAV
- **การลบ:** การล้างรายการ หรือการใช้ไคลเอนต์แบบพกพา (portable) จะไม่ทิ้งร่องรอยไว้ที่นี่ ให้เปรียบเทียบกับ
  Jump Lists, Prefetch, ไฟล์ `.rdp`, RDP bitmap cache และ log ของเป้าหมาย

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry remote-access -d evidence/ -o remote.csv
juntei parse-registry remote-access -f NTUSER.DAT -t jsonl \
  | jq 'select(.Client == "rdp" or .HostKeyFingerprint != null)'
```

สรุปผลจะนับการเชื่อมต่อตามไคลเอนต์ แสดงรายการการเชื่อมต่อที่คีย์ถูกเขียนล่าสุด
โดยเรียงจากใหม่ที่สุด และแสดงเซสชันที่เก็บรหัสผ่านไว้

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | โฮสต์ ชื่อโฟลเดอร์โปรไฟล์ SID ของผู้ใช้จาก `ProfileList` ของระบบ (เมื่อมีไฮฟ์ `SOFTWARE` ของระบบ) และไฮฟ์ |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive` หรือ `network_share` |
| `Kind` | แหล่งที่มาของเรคคอร์ด: `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point` |
| `Target`, `Port` | คอมพิวเตอร์ (ชื่อโฮสต์หรือ IP address) และพอร์ต หรือแชร์ (`\\server\share`) จะมีพอร์ตเฉพาะเมื่อ Windows หรือไคลเอนต์เขียนไว้ |
| `RemoteUsername` | บัญชีที่ใช้บนเป้าหมาย: `UsernameHint`, `UserName` ของเซสชัน (หรือส่วน `user@` ของ `HostName` ใน PuTTY), `UserName` ของไดรฟ์ที่แมป |
| `Protocol` | `Protocol` ของ PuTTY (`ssh`, `telnet`, `raw`, …), `FSProtocol` ของ WinSCP (`sftp`, `scp`, `ftp`, `webdav`, `s3`), ผู้ให้บริการของไดรฟ์ที่แมป (`Microsoft Windows Network`) |
| `MRU-Position` | ตำแหน่งในรายการ (0 คือล่าสุด): `MRU<n>`, `MRUList`, `Recent sessions` ของ PuTTY |
| `SessionName` | ชื่อของเซสชันที่บันทึกไว้ โดยถอดรหัส `%20` และ escape อื่น ๆ แล้ว |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | ชนิดของ SSH host key ที่ยอมรับ (`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …), fingerprint แบบ OpenSSH และค่าตัวเลขตามที่จัดเก็บ |
| `DriveLetter` | อักษรของไดรฟ์ที่แมป |
| `PasswordStored` | เซสชันเก็บรหัสผ่านไว้หรือไม่ โดยไม่มีการถอดรหัส |
| `Details` | การตั้งค่าอื่น ๆ ในรูป `Name=Value; …` (CSV) หรืออ็อบเจกต์ (JSON): `CertHash` ของใบรับรอง RDP, `LocalDevices`, ไฟล์คีย์ของเซสชัน พร็อกซี SSH tunnel การส่งต่อพอร์ต คำสั่งและไดเรกทอรีระยะไกล, `StoredPasswords`, `Label` ของแชร์, `ConnectionType` ของไดรฟ์ |
| `KeyPath`, `ValueName` | คีย์ และค่า (value) สำหรับเรคคอร์ดที่เป็นค่าเดียว |
| `KeyLastWrittenUTC` | เวลาเขียนล่าสุดของคีย์ (ดูข้อควรระวังเกี่ยวกับรายการ) |
| `DataRaw` | ไบต์ของค่า (hex) เมื่อค่านั้นไม่ใช่สตริงหรือถอดรหัสไม่ได้ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ค่าที่มีชนิดไม่ถูกต้อง (พร้อมไบต์), host key ที่อ่านไม่ได้, อักษรใน `MRUList` ที่ไม่มีอยู่ และค่าที่ไม่ได้อยู่ในรายการ |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. แสดงรายการเป้าหมายของผู้ใช้แต่ละคนตามไคลเอนต์ และตรวจสอบกับระบบที่ผู้ใช้นั้นควร
   เข้าถึง มองหาแชร์สำหรับผู้ดูแลระบบ (`C$`, `ADMIN$`) และ IP address
2. บันทึกบัญชีที่ใช้ (`RemoteUsername`): ผู้ดูแลระบบโดเมนและบัญชีภายในเครื่องของ
   เครื่องอื่น
3. จับคู่ `HostKeyFingerprint` กับคีย์ของเซิร์ฟเวอร์ และ `CertHash` กับใบรับรองของ
   เซิร์ฟเวอร์ RDP
4. เชื่อมโยงกับเหตุการณ์ logon ของเป้าหมาย (4624 type 10 และ 3, 1149), Jump Lists และไฟล์ LNK
   (`mstsc.exe`, `putty.exe`, ไฟล์ที่เปิดจากแชร์), Prefetch และ RDP bitmap cache
5. ใช้ `parse-registry read --recover-deleted` สำหรับรายการที่ถูกลบ และใช้สำเนาเก่าของ
   `NTUSER.DAT` ของโปรไฟล์ (Volume Shadow Copies, ข้อมูลสำรอง) สำหรับรายการที่ถูกล้าง

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: การลบรายการออกจากช่อง Computer ของ Remote Desktop Connection](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [PuTTY FAQ: PuTTY เก็บข้อมูลไว้ที่ใด](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: ที่จัดเก็บการกำหนดค่า](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
