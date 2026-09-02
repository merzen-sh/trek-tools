# trek

[English](README.md) | [ไทย](README.th.md)

CLI ยูทิลิตี้ที่เบาและมีประสิทธิภาพสูง สำหรับการสร้างโครงสร้างและแพ็กเกจ FiveM resources

---

## คุณสมบัติ

- **โหมดโต้ตอบและไม่โต้ตอบ**: สร้างโครงสร้าง FiveM resource มาตรฐานพร้อม boilerplate หลายเฟรมเวิร์ก (ESX, QBCore, Qbox) ในไม่กี่วินาที
- **แพ็กเกจตามรูปแบบ**: บีบ resource เป็นไฟล์ `.zip` พร้อมใช้งานจริงโดยใช้รูปแบบ allowlist ใน `.pack`
- **ไบนารีขนาดเล็กมาก**: บิลด์ Rust ที่ปรับให้เหมาะสมสำหรับขนาดไบนารีต่ำสุด

---

## การติดตั้งและการบิลด์

### ข้อกำหนดเบื้องต้น
- [Rust toolchain](https://rustup.rs/) (edition 2024 / stable)

### บิลด์จากซอร์ส

Repository นี้เป็น Cargo workspace monorepo CLI crate อยู่ที่ `crates/trek`

```bash
# บิลด์ไบนารี release ที่ปรับให้เหมาะสม
cargo build --release

# ไบนารีที่คอมไพล์แล้วอยู่ที่ target/release/trek
# ติดตั้งแบบ global ลง cargo bin path (ถ้าต้องการ):
cargo install --path crates/trek
```

---

## การใช้งานและคำสั่ง

```bash
trek [COMMAND]
```

### 1. `generate`

สร้างโฟลเดอร์ FiveM resource ใหม่พร้อมไฟล์ config, client/server scripts, และ shared utilities

```bash
trek generate [OPTIONS]
```

#### ตัวเลือกและแฟล็ก

| Flag / Option | Short | Type | คำอธิบาย |
| :--- | :--- | :--- | :--- |
| `--name <NAME>` | `-n` | `String` | ชื่อ resource หากละเว้นจะเข้าสู่โหมดโต้ตอบ |
| `--description <DESC>` | `-d` | `String` | คำอธิบาย resource ใน `fxmanifest.lua` หากละเว้นจะใช้ค่า `"A FiveM resource for <NAME>"` |
| `--frameworks <LIST>...` | `-f` | `List` | เฟรมเวิร์กที่จะรวม (`ESX`, `QBCore`, `Qbox`, `None`) รองรับหลายค่า |
| `--help` | `-h` | | แสดงข้อมูลช่วยเหลือ |

#### ตัวอย่าง

**โหมดโต้ตอบ:**
```bash
trek generate
```
*จะ prompt ให้กรอกชื่อ resource, คำอธิบาย, และเลือกเฟรมเวิร์ก*

**โหมดไม่โต้ตอบ / CLI Flags:**
```bash
# สร้าง resource พร้อม ESX และ QBCore
trek generate -n my-resource -d "Custom vehicle shop" -f ESX QBCore

# สร้าง resource แบบ standalone
trek generate -n simple-teleport -f None
```

#### โครงสร้างโฟลเดอร์ที่สร้างขึ้น

```text
my-resource/
├── .pack           # รูปแบบ allowlist สำหรับการแพ็กเกจ release
├── fxmanifest.lua       # FiveM resource manifest
├── config/
│   ├── client.lua       # Client-side configuration
│   ├── server.lua       # Server-side configuration
│   └── share.lua        # Shared configuration
├── src/
│   ├── client/
│   │   └── client.lua   # Client-side logic
│   ├── server/
│   │   └── server.lua   # Server-side logic
│   └── shared/
│       └── utils.lua    # Shared utility functions
```

---

### 2. `pack`

แพ็กเกจ FiveM resource ปัจจุบันเป็นไฟล์ `.zip` บีบอัดพร้อมใช้งานโดยใช้รูปแบบ include ที่กำหนดใน `.pack`

```bash
trek pack [OPTIONS]
```

#### ตัวเลือกและแฟล็ก

| Flag / Option | Short | Type | Default | คำอธิบาย |
| :--- | :--- | :--- | :--- | :--- |
| `-o <OUT_DIR>` | `-o` | `Path` | `.` | โฟลเดอร์เอาต์พุตสำหรับเก็บไฟล์ zip |
| `--dry-run` | | `bool` | `false` | จำลองการแพ็กโดยไม่สร้าง/เขียนไฟล์ `.zip` |
| `--report` | | `bool` | `false` | แสดงรายงานสรุปแบบละเอียดในรูปแบบ Markdown |
| `--sha256` | | `bool` | `false` | แสดง SHA-256 checksum ของ archive และรวมในรายงาน |
| `--help` | `-h` | | | แสดงข้อมูลช่วยเหลือ |

#### ตัวอย่าง

```bash
# รันภายในโฟลเดอร์ resource ของคุณ:
cd my-resource

# แพ็ก resource เป็น ./my-resource.zip
trek pack

# Dry run พร้อมแสดงเวลา
trek pack --dry-run

# แพ็กและสร้างรายงาน Markdown
trek pack --report

# แพ็กไปที่โฟลเดอร์เฉพาะพร้อมรายงาน markdown
trek pack -o ./dist --report
```

#### รูปแบบไฟล์ `.pack`

ไฟล์ `.pack` ที่รูทของ resource ระบุไฟล์ที่จะรวมใน release archive โดยใช้ glob patterns บรรทัดเริ่มต้นด้วย `#` และบรรทัดว่างจะถูกข้าม:

```text
# trek include patterns
fxmanifest.lua
config/**/*.lua
src/**/*.lua
```

---

### 3. `version`

แสดงเวอร์ชันปัจจุบันของ resource หรือทำ SemVer version bump อัตโนมัติใน `fxmanifest.lua` พร้อมรักษารูปแบบ syntax เดิม (เช่น `version '1.0.0'`, `version("1.0.0")`, หรือ `version = "1.0.0"`)

```bash
trek version [OPTIONS]
```

#### ตัวเลือกและแฟล็ก

| Flag / Option | Short | Type | Default | คำอธิบาย |
| :--- | :--- | :--- | :--- | :--- |
| `--patch` | | `bool` | | Bump patch version (เช่น `1.2.3` -> `1.2.4`) |
| `--minor` | | `bool` | | Bump minor version และ reset patch (เช่น `1.2.3` -> `1.3.0`) |
| `--major` | | `bool` | | Bump major version และ reset minor/patch (เช่น `1.2.3` -> `2.0.0`) |
| `--ci` | | `bool` | | แสดงแค่ raw version โดยไม่มี styling (machine-readable) |
| `-m, --manifest <PATH>` | `-m` | `Path` | `fxmanifest.lua` | พาธไปยัง `fxmanifest.lua` เป้าหมาย |
| `--help` | `-h` | | | แสดงข้อมูลช่วยเหลือ |

> **หมายเหตุ:** หากไม่มี bump flag จะแสดงเวอร์ชันปัจจุบันโดยไม่แก้ไข manifest สูงสุดหนึ่ง increment flag (`--major`, `--minor`, หรือ `--patch`) ต่อการรันครั้งหนึ่ง

#### ตัวอย่าง

```bash
# แสดงเวอร์ชันปัจจุบัน (read-only)
trek version

# แสดงแค่ raw version (สำหรับ scripts / CI)
trek version --ci
# -> 1.2.3

# Bump patch version (1.0.0 -> 1.0.1)
trek version --patch

# Bump minor version (1.0.1 -> 1.1.0)
trek version --minor

# Bump major version (1.1.0 -> 2.0.0)
trek version --major

# รันกับ resource directory เฉพาะ
trek version --patch -m ./my-resource/fxmanifest.lua
```

---

### 4. `validate`

ตรวจสอบ `fxmanifest.lua` หาปัญหาทั่วไปโดยใช้ built-in manifest parser

ตรวจสอบ:
- มีการประกาศที่จำเป็น (`fx_version`, `game`)
- แนะนำ `lua54 'yes'`
- script/file entries มีอยู่จริงบนดิสก์ (relative to manifest)
- ไม่มี entries ซ้ำใน script list
- Framework imports (`@es_extended`, `@qb-core`, ...) ประกาศภายใต้ `dependency`/`dependencies`

```bash
trek validate [OPTIONS]
```

| Flag / Option | Short | Type | Default | คำอธิบาย |
| :--- | :--- | :--- | :--- | :--- |
| `-m, --manifest <PATH>` | `-m` | `Path` | `fxmanifest.lua` | พาธไปยัง `fxmanifest.lua` เป้าหมาย |

Exit code จะเป็น non-zero เมื่อพบ error-level finding (CI-friendly); warning เพียงอย่างเดียวจะไม่ทำให้ fail

#### ตัวอย่าง

```bash
# Validate resource ปัจจุบัน
trek validate

# Validate manifest เฉพาะ
trek validate -m ./my-resource/fxmanifest.lua
```

---

### 5. `codegen`

สร้าง typed React NUI hooks และ Lua NUI helpers จาก YAML contract สามารถสร้าง starter contract และ JSON Schema สำหรับ editor completion ได้ด้วย

```bash
# สร้าง nui-schema.yaml และ trek-nui.schema.json
trek codegen --init-schema

# สร้าง React และ Lua bindings เริ่มต้น
trek codegen

# ใช้พาธกำหนดเอง
trek codegen -s ./nui-schema.yaml -t ./react/src/generated/nui.ts -l ./src/shared/nui_events.lua
```

---

### 6. `release`

คำสั่งรวมที่รัน full release pipeline: **validate → bump → pack**

จะหยุดก่อน pack หาก validation รายงาน error Version bump เป็น optional — หากไม่มี bump flag จะเก็บเวอร์ชันปัจจุบัน รายงาน Markdown จะรวมอยู่เสมอ พร้อม SHA-256 checksum ของ archive (ถ้าเปิดใช้งาน)

```bash
trek release [OPTIONS]
```

| Flag / Option | Short | Type | Default | คำอธิบาย |
| :--- | :--- | :--- | :--- | :--- |
| `--patch` | | `bool` | | Bump patch version (เช่น `1.2.3` -> `1.2.4`) ก่อน pack |
| `--minor` | | `bool` | | Bump minor version และ reset patch (เช่น `1.2.3` -> `1.3.0`) |
| `--major` | | `bool` | | Bump major version และ reset minor/patch (เช่น `1.2.3` -> `2.0.0`) |
| `-o <OUT_DIR>` | `-o` | `Path` | `.` | โฟลเดอร์เอาต์พุตสำหรับเก็บไฟล์ zip |
| `-m, --manifest <PATH>` | `-m` | `Path` | `fxmanifest.lua` | พาธไปยัง `fxmanifest.lua` เป้าหมาย |
| `--sha256` | | `bool` | `false` | แสดง SHA-256 checksum ของ archive และรวมในรายงาน |

#### ตัวอย่าง

```bash
# Validate + pack พร้อมรายงาน (ไม่ bump version)
trek release

# Release พร้อม patch bump และ SHA-256 checksum
trek release --patch --sha256

# Minor release ลงโฟลเดอร์ dist
trek release --minor -o ../dist
```

---

## License

MIT / Apache-2.0
