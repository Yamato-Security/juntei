# การตั้งค่าความปลอดภัย { #security-settings }

## สิ่งเหล่านี้คืออะไร { #what-they-are }

ผู้โจมตีเปลี่ยนการตั้งค่าความปลอดภัยของ Windows เพื่อซ่อนตัวและเพื่อเข้าถึงข้อมูลรับรอง พวกเขาปิด
Microsoft Defender หรือยกเว้นเครื่องมือของตนจากการสแกน ทำให้ Windows เก็บรหัสผ่านแบบ cleartext
เปิด Remote Desktop ปิดไฟร์วอลล์ และปิดการบันทึก log ของ PowerShell รีจิสทรีเก็บ
การตั้งค่าเหล่านี้ไว้:

- **`SOFTWARE`:** Defender, ข้อยกเว้นและนโยบายของ Defender; UAC; นโยบายของ Remote Desktop และ
  ไฟร์วอลล์; execution policy, นโยบายการบันทึก log และผู้ให้บริการ AMSI ของ PowerShell; System Restore;
  ช่องทาง (channel) ของ event log
- **`SYSTEM`** (ต่อ control set): `WDigest`, การป้องกัน LSA, Credential Guard, Restricted Admin,
  NTLM; Remote Desktop; โปรไฟล์ไฟร์วอลล์; SMBv1; เวลาเข้าถึงล่าสุด; ขนาดของ event log แต่ละรายการ
- **`NTUSER.DAT`:** execution policy และนโยบายการบันทึก log ของ PowerShell ของผู้ใช้ และ AMSI สำหรับ
  Windows Script Host

`parse-registry security-settings` รายงานการตั้งค่าแต่ละรายการที่พบเป็นหนึ่งระเบียน แต่ละระเบียนมี
ค่าของการตั้งค่า ความหมายของค่า และค่านั้น **ลดระดับจากค่าเริ่มต้น (weakens the default)** หรือไม่: กล่าวคือ ปิด
การป้องกันหรือแหล่งหลักฐาน หรือลดระดับให้ต่ำกว่าสิ่งที่ Windows ปัจจุบัน (10 และ 11) ทำเมื่อ
ไม่มีค่านั้นหรือไม่

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> การตั้งค่านี้มีค่านี้ ณ เวลาที่ไฮฟ์
ถูกบันทึก และคีย์ถูกเขียนครั้งล่าสุด ณ เวลานี้ การตั้งค่าที่ถูกลดระดับเป็นเบาะแส ไม่ใช่ข้อพิสูจน์ว่า
ถูกบุกรุก ผู้ดูแลระบบ Group Policy ผลิตภัณฑ์ความปลอดภัยอื่น และ Windows รุ่นเก่าก็ตั้งค่า
บางค่าเหล่านี้เช่นกัน และคำอธิบายความหมายจะระบุไว้ในกรณีที่พบบ่อย

## ตำแหน่ง { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

ให้เก็บรวบรวมไฮฟ์ทั้งสามพร้อม transaction log การตั้งค่าที่เปลี่ยนไม่นานก่อนการเก็บรวบรวม
อาจมีอยู่เฉพาะใน log

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **การกำหนดค่าที่จัดเก็บไว้ไม่ใช่สถานะการป้องกันที่มีผลจริง** ค่ารีจิสทรีที่เก็บรวบรวมมาและแถววินิจฉัยสำหรับรายการค่าที่เสียหาย
  จะถูกรายงาน ข้อมูลที่ขาดหายอาจสะท้อนค่าเริ่มต้น แหล่งข้อมูลที่ไม่ได้เก็บรวบรวม หรือกลไก
  การจัดการอื่น ไม่ใช่ข้อพิสูจน์ว่าการป้องกันเปิดหรือปิดอยู่ `WeakensDefault` ระบุ
  การตั้งค่าที่อาจลดระดับต่ำกว่า baseline ที่ระบุไว้เมื่อการตั้งค่านั้นมีผล
- **แฟล็กแบบเดิมของ Defender อาจถูกละเลย** `DisableAntiSpyware` และ `DisableAntivirus` ไม่ได้
  พิสูจน์ว่า Defender หยุดทำงาน การมีผลขึ้นอยู่กับเวอร์ชันของแพลตฟอร์ม การ onboarding และ tamper
  protection [Microsoft อธิบายการตั้งค่าแบบเดิมเหล่านี้](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware)
- **ข้อยกเว้นที่จัดการแบบใหม่อาจไม่มีอยู่ในข้อมูลรีจิสทรีที่เก็บมา** ตั้งแต่แพลตฟอร์ม
  4.18.25110.6 อุปกรณ์ที่ใช้การจัดการการกำหนดค่าของ Defender for Endpoint ต้องใช้ cmdlet ของ
  Defender PowerShell ที่รองรับเพื่อดึงการกำหนดค่าข้อยกเว้น ดังนั้นการไม่พบในรีจิสทรีแบบออฟไลน์
  จึงไม่ได้ยืนยันว่าไม่มีข้อยกเว้น
  [คำแนะนำการกำหนดค่าของ Microsoft](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings)
- **ถูกลดระดับไม่ได้หมายความว่าเป็นอันตราย** ตัวอย่าง:
  - `DisableAntiSpyware` = 1 ถูกตั้งโดย Windows เองด้วยในขณะที่มีโปรแกรมป้องกันไวรัสอื่นลงทะเบียนอยู่
  - Remote Desktop ถูกเปิดโดยเจตนาบนเซิร์ฟเวอร์
  - Windows 7 เก็บ event log ขนาดเล็กกว่าและไม่ได้อัปเดตเวลาเข้าถึงล่าสุด
- **ค่าเริ่มต้นขึ้นอยู่กับเวอร์ชัน** บางค่าเคยเป็นค่าเริ่มต้นมาก่อน:
  - `NtfsDisableLastAccessUpdate` = 1 เป็นค่าเริ่มต้นก่อน Windows 10 1803 จึงถูกรายงาน
    โดยไม่มีคำตัดสิน แต่ `0x80000001` ที่ตั้งด้วยมือบนเวอร์ชันที่ใหม่กว่าถือว่าถูกลดระดับ
  - `RunAsPPL`, `LsaCfgFlags` และ `UseLogonCredential` ถูกเปรียบเทียบกับ Windows ปัจจุบัน
- **ข้อยกเว้นของ Defender:** แต่ละพาธ นามสกุล โพรเซส หรือที่อยู่เป็นหนึ่งระเบียน ข้อยกเว้นทุกรายการ
  ลดระดับ Defender `TemporaryPaths` ซึ่ง Defender เพิ่มเอง ไม่มีคำตัดสิน ให้มองหาโฟลเดอร์ที่ผู้ใช้เขียนได้
  นามสกุล `.ps1`/`.exe` และ LOLBin (`rundll32.exe`, `powershell.exe`)
- **คีย์นโยบาย** (`Policy` = True) ถูกเขียนโดย Group Policy โดยเครื่องมือ และโดยผู้โจมตีที่
  รู้ว่านโยบายมีลำดับความสำคัญเหนือการตั้งค่าภายในเครื่อง ให้เชื่อมโยงกับ GPO ของโดเมน
- **ช่องทางของ event log:** รายงานเฉพาะ 25 ช่องทางที่เกี่ยวข้องกับความปลอดภัย พร้อมระบุว่า Windows
  เปิดแต่ละช่องทางตามค่าเริ่มต้นหรือไม่ ตัวอย่างเช่น `TaskScheduler/Operational` และ `DNS-Client/Operational`
  ปิดอยู่ตามค่าเริ่มต้น ใช้ `parse-registry read` สำหรับช่องทางอื่น
- **เวลาของคีย์:** `KeyLastWrittenUTC` เป็นของคีย์ จึงไม่จำเป็นต้องเป็นเวลาที่ค่านี้
  เปลี่ยนแปลง การเขียนคีย์ `Policies` ทั้งคีย์ในคราวเดียวเป็นเรื่องปกติ
- **Control set:** ตามค่าเริ่มต้นจะอ่าน control set ที่ Windows ใช้ครั้งล่าสุด `--all-control-sets`
  อ่านทุก control set

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

ส่วนสรุปนับจำนวนการตั้งค่าตามหมวด แสดงรายการการตั้งค่าที่ลดระดับจากค่าเริ่มต้น และ
ข้อยกเว้นทุกรายการของ Defender

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath`, `HiveType` | โฮสต์ ไฮฟ์ และชนิดของไฮฟ์ (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`) |
| `ControlSet`, `IsCurrentControlSet` | สำหรับ `SYSTEM`: control set และเป็นชุดที่ Windows ใช้ครั้งล่าสุดหรือไม่ ไม่ทราบเมื่อ `Select\Current` ไม่มีอยู่หรือไม่ถูกต้อง |
| `User`, `UserSID` | สำหรับ `NTUSER.DAT`: ชื่อโฟลเดอร์โปรไฟล์ และ SID ของผู้ใช้จาก `ProfileList` ของระบบ |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence` |
| `Setting` | การตั้งค่านั้นคืออะไร: `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | ค่า: ตัวเลขฐานสิบ สตริง หรือตัวข้อยกเว้นเอง ว่างเปล่าเมื่อค่าไม่ได้มีชนิดตรงกับการตั้งค่า |
| `Meaning` | ความหมายของค่า |
| `WeakensDefault` | `True` เมื่อค่าลดระดับจากสิ่งที่ Windows ปัจจุบันทำเมื่อไม่มีค่านั้น `False` เมื่อไม่ได้ลดระดับ ว่างเปล่าเมื่อไม่เกี่ยวข้อง (ขนาด พอร์ต ผู้ให้บริการ AMSI) หรือไม่สามารถบอกได้ |
| `Policy` | คีย์อยู่ใต้คีย์ `Policies` หรือไม่ |
| `KeyPath`, `ValueName` | คีย์และค่า ชื่อค่าของข้อยกเว้นคือรายการที่ถูกยกเว้น ผู้ให้บริการ AMSI ไม่มีชื่อค่า |
| `KeyLastWrittenUTC` | เวลาเขียนล่าสุดของคีย์ |
| `DataRaw` | ไบต์ของค่า (hex) เมื่อไม่ได้มีชนิดตรงกับการตั้งค่า |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ค่าที่มีชนิดไม่ถูกต้องและคีย์ที่อ่านไม่ได้ |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เรียกใช้ด้วย `--weakened-only` บนทุกโฮสต์ และเปรียบเทียบโฮสต์ต่าง ๆ เข้าด้วยกัน: การตั้งค่า
   ที่มีเพียงโฮสต์เดียวจะโดดเด่นขึ้นมา
2. ตรวจสอบข้อยกเว้นแต่ละรายการของ Defender เทียบกับเครื่องมือที่พบบนโฮสต์และใน log ของ Defender
   (Event 5007 บันทึกการเปลี่ยนแปลงการกำหนดค่า)
3. เชื่อมโยงเวลาของคีย์กับการเข้าสู่ระบบ การติดตั้งบริการ และการสร้างโพรเซส มองหาบรรทัดคำสั่ง
   `reg.exe`, `Set-MpPreference`, `netsh advfirewall` และ `powershell` (เหตุการณ์
   4688, Sysmon 1 และ 13, PowerShell 4104)
4. สำหรับการตั้งค่าเกี่ยวกับข้อมูลรับรอง (`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`) ให้มองหาการดัมพ์ข้อมูลรับรองและ pass-the-hash ที่เกิดขึ้นหลังจากนั้น
5. ใช้ `parse-registry read --recover-deleted` สำหรับค่าที่ถูกลบเพื่อคืนค่าเริ่มต้น และ
   `--all-control-sets` สำหรับ control set ที่เก่ากว่า

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: กำหนดค่าและตรวจสอบข้อยกเว้นสำหรับ Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: การตั้งค่าและการกำหนดค่า User Account Control](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: กำหนดค่าการป้องกัน LSA เพิ่มเติม](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
