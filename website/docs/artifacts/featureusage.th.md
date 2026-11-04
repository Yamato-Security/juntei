# FeatureUsage, RecentApps และรายการที่ปักหมุดบนแถบงาน { #featureusage-recentapps-and-taskbar-pins }

## สิ่งเหล่านี้คืออะไร { #what-they-are }

Explorer เก็บระเบียนหลายประเภทเกี่ยวกับวิธีที่ผู้ใช้ใช้งานแอปพลิเคชันไว้ใน `NTUSER.DAT` ของผู้ใช้:

- **FeatureUsage** (Windows 10 ขึ้นไป): ตัวนับต่อแอปพลิเคชัน แยกตามลักษณะการใช้งาน
  - `AppSwitched`: จำนวนครั้งที่ผู้ใช้สลับไปยังหน้าต่างของแอปพลิเคชัน (คลิกที่แถบงาน
    หรือ Alt+Tab)
  - `AppLaunch`: จำนวนครั้งที่เริ่มแอปพลิเคชันจากแถบงาน (รายการที่ปักหมุดไว้)
  - `ShowJumpView`: จำนวนครั้งที่เปิด jump list ของแอปพลิเคชัน (คลิกขวาที่แถบงาน)
  - `AppBadgeUpdated`: จำนวนครั้งที่ badge บนแถบงาน (จำนวนรายการที่ยังไม่อ่าน) เปลี่ยนแปลง
  - `TrayButtonClicked`: จำนวนครั้งที่คลิกปุ่มบนแถบงานแต่ละปุ่ม (`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …)

  แอปพลิเคชันในที่นี้คือ application user model ID (`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`) พาธที่ส่วนแรกอาจเป็น GUID ของ known
  folder (`{1AC14E77-…}\cmd.exe` คือ `%windir%\system32\cmd.exe`) หรือ process ID
  (`*PID00001d74`) สำหรับหน้าต่างที่ Explorer จับคู่กับแอปพลิเคชันไม่ได้
- **RecentApps** (Windows 10 จนถึง 1709): แอปพลิเคชันที่ผู้ใช้เรียกใช้ล่าสุด พร้อมจำนวนครั้งที่เปิด
  และเวลาครั้งล่าสุด และภายใต้แต่ละรายการคือไฟล์ที่แอปพลิเคชันนั้นเปิด (`RecentItems`)
- **รายการที่ปักหมุดบนแถบงาน** (`Taskband`): โปรแกรมที่ปักหมุดไว้บนแถบงาน ในรูปแบบ shell item
  (`Favorites`) และในรูปแบบลิงก์ของทางลัดแต่ละรายการที่ปักหมุด (`FavoritesResolve`) บน Windows 10 ขึ้นไป
  แต่ละรายการระบุว่า Windows เป็นผู้ปักหมุด (`SystemPinned`) หรือผู้ใช้เป็นผู้ปักหมุด (`UserPinned`)
- **ProgramsCache** (`StartPage`, `StartPage2`; Windows XP ถึง 7): โปรแกรมที่เมนู Start
  แคชไว้ (`ProgramsCache`) และบน Windows 7 รวมถึงโปรแกรมที่ปักหมุดไว้บนเมนู Start และแถบงาน
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`)

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ผู้ใช้รายนี้ใช้แอปพลิเคชันนี้ด้วยวิธีนี้
เป็นจำนวนครั้งเท่านี้ (FeatureUsage) เรียกใช้เมื่อเวลานี้ (RecentApps) หรือได้ปักหมุดไว้
ตัวนับไม่มีเวลาของตัวเอง มีเพียงเวลาเขียนล่าสุดของคีย์และเวลาของ RecentApps เท่านั้นที่ระบุวันเวลา
ได้

## ตำแหน่ง { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

`KeyCreationTime` ของ `FeatureUsage` คือเวลาที่เริ่มนับ ซึ่งแสดงอยู่ในส่วนสรุป ให้เก็บรวบรวม
`NTUSER.DAT` พร้อม transaction log ของทุกโปรไฟล์ และไฮฟ์ `SYSTEM` และ `SOFTWARE` ของระบบ
เพื่อระบุโฮสต์และ SID ของผู้ใช้

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **จำนวนครั้งโดยไม่มีเวลา** จำนวนครั้งของ FeatureUsage บอกว่าบ่อยแค่ไหน ไม่ได้บอกว่าเมื่อใด เวลาเขียนล่าสุด
  ของคีย์แต่ละประเภท (`KeyLastWrittenUTC`) คือการเปลี่ยนแปลงครั้งล่าสุดของตัวนับใดตัวหนึ่งในคีย์นั้น
- **ไม่ใช่ทุกการเรียกใช้** `AppSwitched` นับการสลับหน้าต่าง ไม่ใช่การเริ่มโปรแกรม แอปพลิเคชันที่ทำงาน
  โดยไม่มีหน้าต่าง หรือที่ผู้ใช้ไม่เคยสลับไปหา จะไม่ถูกนับ `AppLaunch` นับเฉพาะ
  การเริ่มจากแถบงาน
- **แอปแบบแพ็กเกจและพาธ** แอปพลิเคชันถูกตั้งชื่อตามที่ Windows จัดกลุ่มหน้าต่าง: ID ของแอป
  แบบแพ็กเกจ ID ที่โปรแกรมกำหนดเอง (`Chrome`, `VMware.Workstation.vmui`) หรือ
  พาธ `AppPath` ให้พาธที่แทนที่ GUID ของ known folder ด้วยพาธเริ่มต้น
  ของโฟลเดอร์นั้น
- **รายการที่ปักหมุดตามค่าเริ่มต้น** โปรไฟล์ใหม่จะได้รับรายการที่ปักหมุดตามค่าเริ่มต้น (File Explorer, Edge, Store) บน Windows 10
  ขึ้นไปรายการเหล่านี้ระบุว่า `SystemPinned` ส่วนรายการ `UserPinned` คือสิ่งที่ผู้ใช้เลือกเอง
- **รายการที่ปักหมุดอยู่ในสองรายการ** `Favorites` และ `FavoritesResolve` อธิบายรายการที่ปักหมุดชุดเดียวกันใน
  ลำดับเดียวกัน (`Index`) แต่ถูกแสดงเป็นระเบียนแยกกัน
- **พาธของ ProgramsCache** เป็นพาธสัมพัทธ์: โฟลเดอร์ (`Programs\Accessories`) จะตามด้วย
  ทางลัดที่อยู่ในโฟลเดอร์นั้น (`Notepad.lnk`)
- **ค่าที่เสียหาย** จำนวนครั้งที่ไม่ใช่ `REG_DWORD` เวลาที่ไม่ใช่ `REG_QWORD` รายการที่ปักหมุด
  ที่ไม่สามารถตีความได้ หรือลิงก์ที่ไม่ใช่ลิงก์ จะเก็บไบต์ไว้ (`DataRaw`) และถูกรายงานใน
  `Problems`

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

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
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

ส่วนสรุปแสดงระเบียนแยกตามแหล่งข้อมูล เวลาที่ FeatureUsage ของแต่ละไฮฟ์เริ่มนับ
แอปพลิเคชันที่ถูกสลับไปหามากที่สุด แอปและรายการล่าสุด (ใหม่สุดก่อน) และรายการที่ปักหมุดบนแถบงาน

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | โฮสต์ ชื่อโฟลเดอร์โปรไฟล์ SID ของผู้ใช้จาก `ProfileList` ของระบบ และไฮฟ์ |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked` (FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` หรือ `ProgramsCache` |
| `App` | ชื่อค่าของ FeatureUsage, `AppId` ของแอปล่าสุด หรือ application user model ID ของรายการที่ปักหมุด |
| `AppPath` | `App` ในรูปแบบพาธ โดยแทนที่ GUID ของ known folder ด้วยพาธเริ่มต้น และว่างเปล่าสำหรับ ID |
| `Count` | จำนวนครั้งของ FeatureUsage หรือ `LaunchCount` ของแอปล่าสุด |
| `LastAccessedUTC` | `LastAccessedTime` ของแอปหรือรายการล่าสุด |
| `ItemPath` | `Path` ของรายการล่าสุด (หรือ `AppPath` ของแอปล่าสุด) พาธ shell item ของรายการที่ปักหมุดหรือโปรแกรมที่แคชไว้ (`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`) หรือพาธของลิงก์ใน `FavoritesResolve` |
| `DisplayName` | `DisplayName` ของรายการล่าสุด |
| `PinType` | `SystemPinned` หรือ `UserPinned` สำหรับรายการที่ปักหมุดใน `Favorites` |
| `Index` | ตำแหน่งของรายการที่ปักหมุดหรือโปรแกรมที่แคชไว้ในรายการของตน |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | ค่าและคีย์ที่เป็นที่มา และเวลาเขียนล่าสุดของคีย์ |
| `DataRaw` | ไบต์ (hex) ที่ถอดรหัสไม่ได้ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ค่าที่มีชนิดไม่ถูกต้อง รายการที่ปักหมุดหรือรายการที่ตีความไม่ได้ และลิงก์ที่ไม่ใช่ลิงก์ |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เรียง `AppSwitched` ตาม `Count` เพื่อดูแอปพลิเคชันที่ผู้ใช้ทำงานด้วยมากที่สุด และมองหา
   เครื่องมือดูแลระบบ การเข้าถึงระยะไกล การบีบอัดไฟล์ และการถ่ายโอนข้อมูล รวมถึงพาธในตำแหน่งที่ผู้ใช้เขียนได้
   (`\Users\`, `Downloads`, `Temp`)
2. ตรวจสอบรายการที่ปักหมุด: เครื่องมือที่เป็น `UserPinned` หรือรายการที่ปักหมุดซึ่งทางลัดอยู่ในตำแหน่งที่ผิดปกติ
3. ใช้เวลาของ RecentApps ร่วมกับ UserAssist, Prefetch, BAM และ Jump Lists เพื่อระบุวันเวลาการใช้งาน
4. เปรียบเทียบ `KeyCreationTime` กับเวลาที่สร้างโปรไฟล์: ตัวนับเริ่มนับตั้งแต่เวลานั้น
5. ใช้ `parse-registry read --recover-deleted` สำหรับตัวนับและรายการที่ปักหมุดที่ถูกลบ

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: Application User Model IDs](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: Programs cache](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
