<div align="center">
  <br />
  <h1>trek</h1>
  <p><strong>CLI เครื่องมือช่วยสร้างโครงสร้าง ตรวจสอบความถูกต้อง และแพ็กเกจ FiveM resource สำหรับนักพัฒนา</strong></p>
  <p>
    <a href="README.md">English</a> | <a href="README.th.md">ไทย</a>
  </p>
</div>

---

## คุณสมบัติเด่น (Features)

- **สร้างโครงสร้างทั้งแบบโต้ตอบและระบุพารามิเตอร์ (Interactive & Non-interactive Scaffolding)**: สร้างโครงสร้างมาตรฐานสำหรับ FiveM resource พร้อมโค้ด boilerplate รองรับหลายเฟรมเวิร์ก (ESX, QBCore, Qbox) ได้ภายในไม่กี่วินาที
- **แพ็กเกจ Resource ตามรูปแบบที่กำหนด (Pattern-based Resource Packaging)**: บีบอัดและแพ็ก resource เป็นไฟล์ `.zip` สำหรับใช้งานจริง (production-ready) โดยใช้ allowlist patterns ที่กำหนดในไฟล์ `.pack`
- **สร้างโค้ด NUI อัตโนมัติตาม Contract (Contract-Driven NUI Codegen)**: สร้าง typed React NUI hooks และ Lua helpers จาก `nui-schema.yaml` พร้อมรองรับ Enums และ Two-Way RPC
- **ไบนารีขนาดเล็กและทำงานรวดเร็ว (Ultra-lean Binary)**: คอมไพล์ด้วย Rust ที่ปรับแต่งมาเพื่อประสิทธิภาพสูงและมีขนาดไฟล์เล็กที่สุด

---

## การติดตั้งและการบิลด์ (Installation & Build)

### สิ่งที่จำเป็นต้องมีก่อน (Prerequisites)
- [Rust toolchain](https://rustup.rs/) (edition 2024 / stable)

### บิลด์จากซอร์สโค้ด (Build from Source)

Repository นี้เป็น Cargo workspace monorepo โดย CLI crate จะอยู่ที่ `crates/trek`

```bash
# บิลด์ไบนารีแบบ release ที่ปรับแต่งประสิทธิภาพแล้ว
cargo build --release

# ไบนารีที่คอมไพล์เสร็จแล้วจะอยู่ที่ target/release/trek
# ติดตั้งลงใน cargo bin path ทั่วทั้งระบบ (Global):
cargo install --path crates/trek
```

---

## การใช้งานและคำสั่ง (Usage & Commands)

```bash
trek [COMMAND]
```

### 1. `generate`

สร้างโฟลเดอร์ FiveM resource ใหม่พร้อมไฟล์การตั้งค่า (config), สคริปต์ฝั่ง Client/Server และฟังก์ชันช่วยเหลือส่วนกลาง (Shared utilities)

```bash
trek generate [OPTIONS]
```

#### ตัวเลือกและแฟล็ก (Options & Flags)

| Flag / Option | Short | Type | คำอธิบาย |
| :--- | :--- | :--- | :--- |
| `--name <NAME>` | `-n` | `String` | ชื่อของ Resource หากไม่ระบุจะเข้าสู่โหมดโต้ตอบแบบ interactive |
| `--description <DESC>` | `-d` | `String` | คำอธิบาย Resource ใน `fxmanifest.lua` หากไม่ระบุจะมีค่าเริ่มต้นเป็น `"A FiveM resource for <NAME>"` |
| `--frameworks <LIST>...` | `-f` | `List` | เฟรมเวิร์กที่ต้องการติดตั้ง (`ESX`, `QBCore`, `Qbox`, `None`) รองรับการเลือกหลายค่าพร้อมกัน |
| `--help` | `-h` | | แสดงข้อมูลช่วยเหลือ |

#### ตัวอย่างการใช้งาน

**โหมด Interactive:**
```bash
trek generate
```
*ระบบจะสอบถามชื่อ resource, คำอธิบาย และตัวเลือกเฟรมเวิร์กที่ต้องการใช้งานทีละขั้นตอน*

**โหมดระบุ Flag ผ่าน Command Line:**
```bash
# สร้าง resource ที่รองรับทั้ง ESX และ QBCore
trek generate -n my-resource -d "Custom vehicle shop" -f ESX QBCore

# สร้าง resource แบบ Standalone (ไม่ใช้เฟรมเวิร์ก)
trek generate -n simple-teleport -f None
```

#### โครงสร้างไดเรกทอรีที่ถูกสร้างขึ้น

```text
my-resource/
├── .pack           # รูปแบบ allowlist สำหรับการแพ็กไฟล์ release
├── fxmanifest.lua  # ไฟล์ FiveM resource manifest
├── config/
│   ├── client.lua  # ไฟล์ตั้งค่าฝั่ง Client
│   ├── server.lua  # ไฟล์ตั้งค่าฝั่ง Server
│   └── share.lua   # ไฟล์ตั้งค่าส่วนกลาง (Shared)
└── src/
    ├── client/
    │   └── client.lua   # โค้ดการทำงานฝั่ง Client
    ├── server/
    │   └── server.lua   # โค้ดการทำงานฝั่ง Server
    └── shared/
        └── utils.lua    # ฟังก์ชันช่วยเหลือส่วนกลาง (Shared utilities)
```

---

### 2. `pack`

แพ็ก FiveM resource ปัจจุบันให้เป็นไฟล์ `.zip` สำหรับ release โดยบีบอัดตาม include patterns ที่ระบุไว้ในไฟล์ `.pack`

```bash
trek pack [OPTIONS]
```

#### ตัวเลือกและแฟล็ก (Options & Flags)

| Flag / Option | Short | Type | Default | คำอธิบาย |
| :--- | :--- | :--- | :--- | :--- |
| `-o <OUT_DIR>` | `-o` | `Path` | `.` | ไดเรกทอรีปลายทางสำหรับบันทึกไฟล์ zip archive |
| `--dry-run` | | `bool` | `false` | จำลองการแพ็กไฟล์โดยไม่สร้างหรือเขียนไฟล์ `.zip` จริง |
| `--report` | | `bool` | `false` | แสดงรายงานสรุปผลอย่างละเอียดในรูปแบบ Markdown |
| `--sha256` | | `bool` | `false` | แสดง SHA-256 checksum ของไฟล์ archive และบันทึกลงในรายงาน |
| `--help` | `-h` | | | แสดงข้อมูลช่วยเหลือ |

#### ตัวอย่างการใช้งาน

```bash
# รันคำสั่งภายในโฟลเดอร์ resource ของคุณ:
cd my-resource

# แพ็ก resource ออกมาเป็นไฟล์ ./my-resource.zip
trek pack

# จำลองการแพ็กไฟล์ (Dry run) พร้อมจับเวลาการทำงาน
trek pack --dry-run

# แพ็กไฟล์พร้อมแสดงรายงานสรุปแบบ Markdown
trek pack --report

# แพ็กไฟล์ไปยังโฟลเดอร์เฉพาะพร้อมสร้างรายงาน Markdown
trek pack -o ./dist --report
```

#### รูปแบบของไฟล์ `.pack`

ไฟล์ `.pack` ที่อยู่ในโฟลเดอร์ root ของ resource จะใช้ glob pattern เพื่อระบุไฟล์ที่ต้องการรวมเข้าในไฟล์ release บรรทัดที่ขึ้นต้นด้วย `#` และบรรทัดว่างจะถูกข้าม:

```text
# trek include patterns
fxmanifest.lua
config/**/*.lua
src/**/*.lua
```

---

### 3. `version`

แสดงเวอร์ชันปัจจุบันของ resource หรืออัปเดตเวอร์ชัน SemVer ใน `fxmanifest.lua` อัตโนมัติ โดยยังคงรักษารูปแบบ syntax เดิมไว้ทุกประการ (เช่น `version '1.0.0'`, `version("1.0.0")` หรือ `version = "1.0.0"`)

```bash
trek version [OPTIONS]
```

#### ตัวเลือกและแฟล็ก (Options & Flags)

| Flag / Option | Short | Type | Default | คำอธิบาย |
| :--- | :--- | :--- | :--- | :--- |
| `--patch` | | `bool` | | อัปเดต patch version (เช่น `1.2.3` -> `1.2.4`) |
| `--minor` | | `bool` | | อัปเดต minor version และรีเซ็ต patch เป็น 0 (เช่น `1.2.3` -> `1.3.0`) |
| `--major` | | `bool` | | อัปเดต major version และรีเซ็ต minor/patch เป็น 0 (เช่น `1.2.3` -> `2.0.0`) |
| `--ci` | | `bool` | | แสดงเฉพาะค่าเวอร์ชันแบบ raw สำหรับนำไปประมวลผลต่อใน Script / CI |
| `-m, --manifest <PATH>` | `-m` | `Path` | `fxmanifest.lua` | พาธไปยังไฟล์ `fxmanifest.lua` เป้าหมาย |
| `--help` | `-h` | | | แสดงข้อมูลช่วยเหลือ |

> **หมายเหตุ:** หากไม่ระบุ bump flag คำสั่งจะแสดงเวอร์ชันปัจจุบันโดยไม่มีการแก้ไขไฟล์ manifest และสามารถระบุ increment flag (`--major`, `--minor` หรือ `--patch`) ได้ไม่เกิน 1 แฟล็กต่อการรันหนึ่งครั้ง

#### ตัวอย่างการใช้งาน

```bash
# แสดงเวอร์ชันปัจจุบัน (อ่านอย่างเดียว)
trek version

# แสดงเฉพาะค่า raw version (สำหรับ scripts / CI)
trek version --ci
# -> 1.2.3

# อัปเดต patch version (1.0.0 -> 1.0.1)
trek version --patch

# อัปเดต minor version (1.0.1 -> 1.1.0)
trek version --minor

# อัปเดต major version (1.1.0 -> 2.0.0)
trek version --major

# ใช้งานกับโฟลเดอร์ resource ที่ต้องการ
trek version --patch -m ./my-resource/fxmanifest.lua
```

---

### 4. `validate`

ตรวจสอบ (Lint) ข้อผิดพลาดทั่วไปในไฟล์ `fxmanifest.lua` โดยใช้ built-in manifest parser

รายการที่ตรวจสอบ:
- มีการประกาศฟิลด์ที่จำเป็น (`fx_version`, `game`)
- แนะนำให้ใส่ `lua54 'yes'`
- ไฟล์สคริปต์/ไฟล์ทั่วไปที่ระบุมีอยู่จริงบนดิสก์ (อ้างอิง path สัมพัทธ์กับ manifest)
- ไม่มีรายการซ้ำซ้อนในรายการสคริปต์
- นำเข้า Framework (`@es_extended`, `@qb-core`, ...) ภายใต้การประกาศ `dependency`/`dependencies`

```bash
trek validate [OPTIONS]
```

| Flag / Option | Short | Type | Default | คำอธิบาย |
| :--- | :--- | :--- | :--- | :--- |
| `-m, --manifest <PATH>` | `-m` | `Path` | `fxmanifest.lua` | พาธไปยังไฟล์ `fxmanifest.lua` เป้าหมาย |

จะส่งค่า Exit code เป็น non-zero เมื่อพบข้อผิดพลาดระดับ error (เหมาะสำหรับการนำไปใช้ใน CI) ส่วนข้อความระดับ warning จะไม่ส่งผลให้การรันล้มเหลว

#### ตัวอย่างการใช้งาน

```bash
# ตรวจสอบ resource ปัจจุบัน
trek validate

# ตรวจสอบ manifest ที่ระบุ
trek validate -m ./my-resource/fxmanifest.lua
```

---

### 5. `codegen`

สร้าง typed React NUI hooks และ Lua NUI helpers จาก YAML contract (`nui-schema.yaml`) รองรับ Enums, One-way events และ Two-way RPC endpoints (`query` / `mutation`)

```bash
# สร้างไฟล์เริ่มต้น nui-schema.yaml และ trek-nui.schema.json
trek codegen --init-schema

# สร้าง React และ Lua bindings ค่าเริ่มต้น
trek codegen

# กำหนดพาธของไฟล์ด้วยตนเอง
trek codegen -s ./nui-schema.yaml -t ./react/src/generated/nui.ts -l ./src/shared/nui_events.lua
```

#### ตัวอย่างไฟล์ `nui-schema.yaml`

```yaml
version: "1.0"
resource: "my_resource"

enums:
  - name: PlayerStatus
    description: "สถานะปัจจุบันของผู้เล่น"
    values:
      - idle
      - in_combat
      - dead

events:
  - name: statusUpdated
    description: "ส่งอีเวนต์เมื่อสถานะของผู้เล่นเปลี่ยนแปลง"
    payload:
      status: PlayerStatus
      health: number

endpoints:
  - name: getPlayerStatus
    type: query
    response:
      status: PlayerStatus
```

> สำหรับข้อกำหนด schema ฉบับเต็ม, ระบบ Type และตัวอย่างการใช้งานแบบสมบูรณ์ สามารถดูได้ที่ [คู่มือ nui-schema.yaml](docs/nui-schema.md)

---

### 6. `release`

คำสั่งรวมสำหรับรันกระบวนการ release แบบครบวงจร: **validate → bump → pack**

จะยกเลิกกระบวนการก่อนขั้นตอนการ pack หาก validation ตรวจพบข้อผิดพลาด การอัปเดตเวอร์ชัน (version bumping) เป็นตัวเลือกเสริม — หากไม่ระบุ bump flag จะคงเวอร์ชันเดิมไว้ รายงานผลแบบ Markdown จะถูกสร้างขึ้นเสมอ พร้อมรองรับการคำนวณ SHA-256 checksum ของไฟล์ archive

```bash
trek release [OPTIONS]
```

| Flag / Option | Short | Type | Default | คำอธิบาย |
| :--- | :--- | :--- | :--- | :--- |
| `--patch` | | `bool` | | อัปเดต patch version (เช่น `1.2.3` -> `1.2.4`) ก่อนทำ packaging |
| `--minor` | | `bool` | | อัปเดต minor version และรีเซ็ต patch เป็น 0 (เช่น `1.2.3` -> `1.3.0`) ก่อนแพ็ก |
| `--major` | | `bool` | | อัปเดต major version และรีเซ็ต minor/patch เป็น 0 (เช่น `1.2.3` -> `2.0.0`) ก่อนแพ็ก |
| `-o <OUT_DIR>` | `-o` | `Path` | `.` | ไดเรกทอรีปลายทางสำหรับบันทึกไฟล์ zip archive |
| `-m, --manifest <PATH>` | `-m` | `Path` | `fxmanifest.lua` | พาธไปยังไฟล์ `fxmanifest.lua` เป้าหมาย |
| `--sha256` | | `bool` | `false` | แสดง SHA-256 checksum ของไฟล์ archive และบันทึกลงในรายงาน |

#### ตัวอย่างการใช้งาน

```bash
# Validate + pack พร้อมรายงานสรุป (ไม่เปลี่ยนเลขเวอร์ชัน)
trek release

# Release พร้อมอัปเดต patch version และแสดง SHA-256 checksum
trek release --patch --sha256

# ทำ minor release และบันทึกไฟล์ไปยังโฟลเดอร์ dist
trek release --minor -o ../dist
```

---

## สัญญาอนุญาต (License)

MIT / Apache-2.0
