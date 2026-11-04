# ติดตั้ง Juntei { #install-juntei }

## ไบนารีจากรุ่นที่เผยแพร่ { #release-binaries }

ดาวน์โหลดไฟล์ zip สำหรับระบบของคุณจาก
[หน้ารุ่นที่เผยแพร่ของ Juntei](https://github.com/Yamato-Security/juntei/releases) หากยังไม่มีไบนารี
ที่เผยแพร่ ให้คอมไพล์จากซอร์สตามที่อธิบายไว้ด้านล่าง เก็บไฟล์บีบอัดหรือ commit ของซอร์สและ
checksum ไว้กับบันทึกของคดี เพื่อให้นักวิเคราะห์คนอื่นทำซ้ำการทำงานได้

| ระบบ | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` หรือ `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` หรือ `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

zip แต่ละไฟล์มีเพียงไบนารี ซึ่งตั้งชื่อตามเวอร์ชันและแพลตฟอร์ม (เช่น
`juntei-1.0.0-win-x64.exe`): Juntei ไม่ต้องใช้ไฟล์อื่นใด และไบนารีสำหรับ Windows ไม่ต้องใช้
Visual C++ Redistributable ไบนารี Linux แบบ musl ลิงก์แบบ static และทำงานได้บน
ทุกดิสทริบิวชัน ส่วนแบบ gnu ต้องใช้ glibc รุ่นใหม่ บน macOS ให้อนุญาตไบนารีที่ดาวน์โหลดมาใน
System Settings > Privacy & Security หรือลบแอตทริบิวต์ quarantine ด้วย
`xattr -d com.apple.quarantine juntei-<version>-mac-*`

```bash
juntei --version
juntei --help
```

ตัวอย่างในเอกสารนี้ใช้ `juntei` เป็นชื่อไบนารี ให้เปลี่ยนชื่อไฟล์ หรือพิมพ์ชื่อเต็มของไฟล์

## คอมไพล์จากซอร์ส { #compile-from-source }

ปัจจุบัน Juntei ต้องใช้ Rust เวอร์ชันที่ประกาศไว้ใน `Cargo.toml` (Rust 1.99 ณ เวลาที่เขียน
หน้านี้)

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` ใช้ dependency ตามรุ่นที่บันทึกไว้ใน `Cargo.lock` ทุกประการ

## รูปแบบคำสั่งพื้นฐาน { #basic-command-shape }

พาร์เซอร์ทุกตัวรับไฟล์เดียว (`-f`) หรือไดเรกทอรีของข้อมูลที่เก็บรวบรวมมา (`-d`):

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

ให้แสดงตัวเลือกปัจจุบัน แทนที่จะคัดลอกคำสั่งเก่าจากรายงาน:

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

ต่อด้วย[การตรวจสอบครั้งแรก](first-investigation.md) สำหรับขั้นตอนการทำงานที่ปลอดภัย
