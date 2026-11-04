# การเริ่มอัตโนมัติผ่านรีจิสทรี { #registry-autostarts }

## สิ่งเหล่านี้คืออะไร { #what-they-are }

Windows เริ่มโปรแกรมและโหลด DLL จากตำแหน่งในรีจิสทรีจำนวนมาก: ขณะบูต ขณะ logon เมื่อ
โปรแกรมเริ่มหรือจบการทำงาน เมื่อ `cmd.exe` เปิดขึ้น เมื่อ COM class ถูกสร้าง หรือเมื่อ spooler,
LSA, netsh หรือบริการเวลาโหลด plug-in ของตน ผู้โจมตีใช้ตำแหน่งเหล่านี้เพื่อการคงอยู่ (persistence)
การยกระดับสิทธิ์ และ DLL injection และซอฟต์แวร์ที่ถูกต้องตามกฎหมายก็ใช้เช่นกัน Juntei อ่านตำแหน่งเหล่านี้
จาก `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` และ `UsrClass.dat` มารวมเป็นตารางเดียว โดยเขียนหนึ่งเรคคอร์ดต่อ
โปรแกรม DLL หรือแพ็กเกจ และติดแท็กแต่ละรายการด้วยเทคนิค MITRE ATT&CK

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ไฮฟ์ได้กำหนดค่าให้โปรแกรมหรือ DLL นี้
เริ่มทำงานหรือถูกโหลด ซึ่งไม่ได้พิสูจน์ว่าโปรแกรมได้ทำงานแล้ว การทำงานต้องอาศัยเงื่อนไขที่
กลไกนั้นระบุ (การ logon การเริ่มโปรแกรมที่ถูก debug การสร้าง class) และหลักฐานอื่น
ของการทำงาน

## ตำแหน่ง { #locations }

| ไฮฟ์ | สิ่งที่อ่าน |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce` (+`Setup`), `RunOnceEx`, `RunServices(Once)` และ `Policies\Explorer\Run` พร้อมสำเนาใน `Wow6432Node` และสำเนาของ Terminal Server install-mode; `Explorer\StartupApproved\Run`, `Run32` และ `StartupFolder`; `Winlogon` (`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … และ `Notify`); `AppInit_DLLs` ของ `Windows` พร้อม `LoadAppInit_DLLs`; `Debugger` ของ `Image File Execution Options\*` (พร้อม `UseFilter`) และ `VerifierDlls`; `MonitorProcess` ของ `SilentProcessExit\*`; `StubPath` ของ `Active Setup\Installed Components\*`; `NetSh`; `AutoRun` ของ `Command Processor` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, ตัวแปร .NET profiler ใน `Environment`); `Authentication`, `Security` และ `Notification Packages` ของ `Control\Lsa` (+`OSConfig`); `SecurityProviders`; monitor, provider และ print processor ของ `Control\Print`; `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | คีย์ `Run` และ `StartupApproved` ของผู้ใช้; `Load` และ `Run` ของ `Windows NT\CurrentVersion\Windows`; `Shell` ของ `Winlogon`; `Environment` (`UserInitMprLogonScript`, .NET profiler); `SCRNSAVE.EXE` ของ `Control Panel\Desktop`; `PackagedAppXDebug`; `AutoRun` ของ `Command Processor`; `Software\Classes` (เช่นเดียวกับ `UsrClass.dat`) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` และ `LocalServer32` (รวมถึงใน `Wow6432Node`); `shell\open\command` ของชนิดโปรแกรม (`exefile`, `batfile`, `scrfile`, …) และของสิ่งที่โปรแกรมแบบ auto-elevate เริ่มทำงาน (`ms-settings`, `mscfile`, `Folder`, …); `ActivatableClasses\…\DebugInformation` |

ให้เก็บรวบรวมแต่ละไฮฟ์พร้อม transaction log ของไฮฟ์นั้น ไฮฟ์ของผู้ใช้อยู่ที่ `Users\<name>\NTUSER.DAT` และ
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat`

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **ค่าเริ่มต้นก็ถูกรายงานด้วย** เรคคอร์ดส่วนใหญ่บนระบบที่สะอาดเป็นค่าเริ่มต้นของ Windows:
  - `Userinit` คือ `userinit.exe`
  - `Shell` คือ `explorer.exe`
  - `KnownDLLs`, LSA package, print monitor, netsh helper และ Active Setup component ล้วนมีรายการมาตรฐานอยู่แล้ว

  ให้มองหาสิ่งที่แตกต่างจากระบบสะอาดของ Windows เวอร์ชันเดียวกัน
- **`StartupApproved` ครอบคลุมเฉพาะรายการ Startup ของ Task Manager** โดยบอกว่ารายการ `Run`
  หรือไฟล์ในโฟลเดอร์ Startup ถูกปิดใช้งานที่นั่นหรือไม่ และเมื่อใด รายการที่ไม่มีค่า `Run`
  ที่ตรงกันจะถูกรายงานพร้อมรายละเอียด "only in StartupApproved" ซึ่งอาจแสดงถึงการเริ่มอัตโนมัติที่
  ถูกลบออกไปแล้ว
- **`OverridesMachineRegistration` ต้องใช้ไฮฟ์ `SOFTWARE` ของระบบ** COM class หรือ
  คำสั่งของผู้ใช้จะถูกตรวจสอบเทียบกับไฮฟ์นี้ ซึ่งถูกค้นหาด้วยวิธีเดียวกับไฮฟ์ `SYSTEM` ของโฮสต์ การ
  override คือรูปแบบของ COM hijacking (T1546.015) OneDrive และการติดตั้งแบบต่อผู้ใช้อื่น ๆ ก็
  ลงทะเบียน class ของตนเองที่ไม่ได้ override สิ่งใดเช่นกัน
- **`ImagePath` และ `Args` จะถูกแยกออกจากบรรทัดคำสั่งเฉพาะเมื่อสามารถแยกได้**
  - โปรแกรมที่อยู่ในเครื่องหมายคำพูดจะถูกแยกที่เครื่องหมายคำพูดปิด
  - คำแรกที่ไม่มีโฟลเดอร์จะถือเป็นโปรแกรม (`cmd /c …`)
  - มิฉะนั้นจะแยกที่ส่วนต้นที่สั้นที่สุดซึ่งลงท้ายด้วยนามสกุลของโปรแกรม

  พาธที่ไม่มีเครื่องหมายคำพูดแต่มีช่องว่างอาจเรียกใช้โปรแกรมอื่น (`C:\Program.exe`) หากมีโปรแกรมนั้นอยู่
- **`GlobalFlag` เป็นตัวกำหนด `Enabled` ของ `VerifierDlls` และ `MonitorProcess`** `VerifierDlls` จะโหลด
  เฉพาะเมื่อมี `0x100` ส่วน `MonitorProcess` จะทำงานเฉพาะเมื่อมี `0x200` และ `0x1` ของ `ReportingMode`
  ใน `SilentProcessExit\<program>` โหมด `2` (dump) หรือ `4` (การแจ้งเตือน) จะไม่เรียกใช้สิ่งใด และ
  `Details` จะระบุชื่อแฟล็กของโหมด
- **`KeyLastWrittenUTC` คือเวลาเขียนล่าสุดของคีย์** ไม่ใช่ของค่า หลายค่าใช้เวลานี้ร่วมกัน

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

สรุปผลจะนับเรคคอร์ดตามกลไก และยังแสดงรายการ `StartupApproved\Run` และ `Run32`
ที่ไม่มีค่า `Run` ของตน (ไฟล์ในโฟลเดอร์ Startup ไม่มีค่าที่จะหายไป) รวมถึง COM class
และคำสั่งของผู้ใช้ที่ override ของเครื่อง

เช่นเดียวกับคำสั่ง `parse-registry` ทุกคำสั่ง คำสั่งนี้จะปฏิเสธ `-o` ที่เป็นหนึ่งในไฟล์ที่คำสั่งอ่าน แม้จะใช้
`-C` ก็ตาม ซึ่งครอบคลุมไฮฟ์ transaction log ของไฮฟ์ และไฮฟ์ SYSTEM และ SOFTWARE ของระบบ
ไม่ว่าจะระบุด้วยพาธ symbolic link หรือ hard link เมื่อไม่สามารถตัดสินได้ คำสั่งจะไม่เขียนไฟล์

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath`, `HiveType` | บริบทของโฮสต์ ไฮฟ์ต้นทาง และชนิดของไฮฟ์ |
| `User`, `UserSID` | สำหรับไฮฟ์ของผู้ใช้: โฟลเดอร์โปรไฟล์และ SID โดย SID มาจากชื่อคีย์ root ของ `UsrClass.dat` หากไม่มีจะมาจาก `ProfileList` ของระบบ |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | เทคนิค MITRE ATT&CK ที่เข้ากันได้ (`T1547.001`, `T1546.012`, …) |
| `KeyPath`, `ValueName` | คีย์ภายใต้ root ของไฮฟ์ (ชื่อตามที่เขียนไว้) และค่า (ว่าง: ค่า default) |
| `Data` | ข้อความของค่า หรือหนึ่ง item ของรายการ จะว่างเมื่ออ่านไม่ได้ หรือเมื่อมีเพียง `StartupApproved` ที่ระบุรายการนั้น |
| `ImagePath`, `Args` | โปรแกรมหรือ DLL และอาร์กิวเมนต์ เมื่อสามารถแยกออกจากกันได้ (ดูด้านบน) |
| `Enabled`, `DisabledAtUTC` | สิ่งที่ `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag` (พร้อม `ReportingMode`), `UseFilter`, `IsInstalled`, `Enabled` ของ time provider หรือ `ScreenSaveActive` ระบุ จะว่างเมื่อไม่มีสิ่งใดระบุ |
| `Details` | ข้อมูลเพิ่มเติม: ผลของการเปลี่ยนชื่อที่รอดำเนินการ ชื่อของ COM class, `ReportingMode`, `FilterFullPath`, `ShellComponent` ของ Active Setup, … |
| `OverridesMachineRegistration` | สำหรับ COM class หรือคำสั่งของผู้ใช้: เครื่องลงทะเบียนรายการเดียวกันไว้หรือไม่ จะว่างหากไม่มีไฮฟ์ `SOFTWARE` ของระบบ |
| `WOW64` | อยู่ภายใต้ `Wow6432Node`: สำหรับโปรแกรม 32 บิต |
| `KeyLastWrittenUTC` | เวลาเขียนล่าสุดของคีย์ |
| `DataRaw` | ไบต์ของค่า (hex) เมื่อไม่ใช่ข้อความหรือไม่ใช่ UTF-16 ที่ถูกต้อง |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ชนิดของค่าที่ไม่ถูกต้อง ข้อมูลที่อ่านไม่ได้ และรายการ `StartupApproved` ที่มีขนาดไม่ถูกต้อง |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เริ่มจากสรุปผล:
   - COM class และคำสั่งที่ override ของเครื่อง
   - รายการ `StartupApproved` ที่ไม่มีค่า `Run` ของตน
   - กลไกที่ไม่ค่อยมีรายการ (`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`)
2. เปรียบเทียบ LSA package, `BootExecute`, `KnownDLLs`, print monitor และ netsh helper กับระบบสะอาด
   ของ Windows เวอร์ชันเดียวกัน
3. ตรวจสอบ `ImagePath` กับระบบไฟล์ ($MFT แฮช และลายเซ็นดิจิทัล) และตรวจสอบพาธในโปรไฟล์ผู้ใช้
   `ProgramData` และ `Temp`
4. เชื่อมโยง `KeyLastWrittenUTC` กับเหตุการณ์รีจิสทรีของ Security และ Sysmon การสร้างโพรเซส
   Prefetch และ Amcache
5. ใช้ `parse-registry read --recover-deleted` เพื่อค้นหาการเริ่มอัตโนมัติที่ถูกลบออกจากคีย์เหล่านี้

## รายละเอียดเพิ่มเติม { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: คีย์รีจิสทรี Run และ RunOnce](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
