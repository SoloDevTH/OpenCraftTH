# OpenCraft Studio

Ten open-source creative apps for Windows in one installer: CAD, page layout, motion graphics,
video editing, spreadsheets, photo library, PDF, image editing, vector illustration and word
processing. Every app is native, written in Rust, works offline, and opens and saves the
everyday file formats of its kind.

> [!IMPORTANT]
> **OpenCraft Studio is an unofficial fork.** Every app here is based on the original by the
> [ArtCraft team](https://getartcraft.com/), published at [github.com/storytold](https://github.com/storytold)
> under MIT OR Apache-2.0. The ArtCraft team does not make, endorse or support this fork, and it is
> not an ArtCraft product. As their brand terms ask of forks, the ArtCraft name and logos are not
> used here. What this fork changed is in [CHANGES.md](CHANGES.md); who made what is in
> [CREDITS.md](CREDITS.md).

[ภาษาไทย](#ภาษาไทย) · [Download](#download) · [The apps](#the-apps) · [Build from source](#build-from-source) · [Privacy](#privacy) · [Credits and license](#credits-and-license)

## Download

Get the installer from [Releases](https://github.com/SoloDevTH/OpenCraftTH/releases):

| File | What it is |
|---|---|
| `OpenCraft-Studio-Setup-<version>-x64.exe` | **One file for everything.** Tick the programs you want; run it again later to add or remove programs. |
| `<app>-<version>-windows-x64.msi` | One app on its own. |
| `<app>-<version>-windows-x64-portable.zip` | One app without installing: unzip and run (settings stay in the folder). |

Windows 10 or 11, 64-bit (x64). The installers are not code-signed yet, so Windows SmartScreen may
say it "protected your PC": choose **More info ▸ Run anyway**.

Each installed program gets a Start menu entry and is offered under **Open with** for its file
types, and can be chosen as the default app in **Settings ▸ Apps ▸ Default apps**. A program only
becomes the default for its own document format (`.pcraft`, `.vectorcraft`, `.fcproj` and so on),
never for shared formats like `.jpg`, `.pdf` or `.docx` unless you choose it.

Silent install (all apps but the ones set to 0):

```bat
OpenCraft-Studio-Setup-<version>-x64.exe /quiet Install_filmcraft=0 Install_effectcraft=0
```

## The apps

| App | What it does | Opens | Based on |
|---|---|---|---|
| [CADCraft](cadcraft/) | CAD drafting | DXF, DWG | [storytold/cadcraft](https://github.com/storytold/cadcraft) |
| [DesignCraft](designcraft/) | Page layout for books, magazines and posters | IDML, its own `.designcraft` | [storytold/designcraft](https://github.com/storytold/designcraft) |
| [EffectCraft](effectcraft/) | Animated titles, motion graphics, compositing | its own `.ecproj` | [storytold/effectcraft](https://github.com/storytold/effectcraft) |
| [FilmCraft](filmcraft/) | Video editing: multi-track timeline, colour, audio | MP4, MOV, MKV, WebM… | [storytold/filmcraft](https://github.com/storytold/filmcraft) |
| [GridCraft](gridcraft/) | Spreadsheets with formulas | XLSX, XLSM, CSV, TSV | [storytold/gridcraft](https://github.com/storytold/gridcraft) |
| [LightCraft](lightcraft/) | Photo library and RAW darkroom, non-destructive | JPEG, PNG, TIFF, WebP, DNG and camera RAW | [storytold/lightcraft](https://github.com/storytold/lightcraft) |
| [PdfCraft](pdfcraft/) | Read, annotate, fill, sign and OCR PDF files | PDF | [storytold/pdfcraft](https://github.com/storytold/pdfcraft) |
| [PhotoCraft](photocraft/) | Layered image editing | PSD, PSB, PNG, JPEG… | [storytold/photocraft](https://github.com/storytold/photocraft) |
| [VectorCraft](vectorcraft/) | Vector illustration | SVG, AI, EPS, PDF, DXF, EMF… | [storytold/vectorcraft](https://github.com/storytold/vectorcraft) |
| [WordCraft](wordcraft/) | Word processing | DOCX, DOC, ODT, RTF, Markdown, TXT | [storytold/wordcraft](https://github.com/storytold/wordcraft) |

Each folder keeps its app's own README, documentation, tests and licence files.

## Build from source

Needs Windows with Rust 1.95 or later (`rustup`), the MSVC build tools and the .NET 6+ runtime
(for WiX, which the build fetches by itself).

```bat
installer\build.cmd
```

That builds every app's MSI (`<app>\packaging\windows\package.ps1`) and then the one-file installer
in `installer\dist\`. Building only some apps, another architecture, or on another drive:

```powershell
powershell -ExecutionPolicy Bypass -File installer\build.ps1 -Build -Apps pdfcraft,photocraft
powershell -ExecutionPolicy Bypass -File installer\build.ps1 -Build -ScratchDir D:\build
```

Each app also builds and runs on its own: see the Quick start in its README (`cargo run --release -p <app>`).

## Privacy

No app sends anything by itself: no telemetry, no accounts, no update pings. The internet is used
only when you ask for it:

- **Opening a web page** you clicked (Help menu, About, credits): it opens in your browser.
- **Downloading an optional model** you chose: FilmCraft's speech-to-text and text-to-speech
  models, LightCraft's face and denoise models. LightCraft's place names (GeoNames) and EffectCraft's
  Roto Brush and face-tracking models are **already built in**.
- **Local only**: the automation port on `127.0.0.1` (off unless an app is started with
  `--control`) and WordCraft's connection to a Zotero running on the same computer.

## Credits and license

The apps are by the **ArtCraft team and their contributors**; this fork's changes are listed in
[CHANGES.md](CHANGES.md). Every app is dual-licensed under [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE) at your option, as the originals are; each app folder keeps its own
`LICENSE-MIT`, `LICENSE-APACHE` and `NOTICE`. Fonts, icons, models, data and other bundled assets
keep their own licences: see [CREDITS.md](CREDITS.md) and each app's attribution file.

Adobe, Photoshop, Illustrator, InDesign, Premiere Pro, After Effects, Lightroom and Acrobat are
trademarks of Adobe Inc.; Microsoft Excel and Word are trademarks of Microsoft; AutoCAD is a
trademark of Autodesk. These apps are independent and not affiliated with any of them; the names
only say which workflows they follow.

---

## ภาษาไทย

**OpenCraft Studio** คือชุดโปรแกรมสร้างสรรค์โอเพนซอร์ส 10 โปรแกรมสำหรับ Windows รวมในตัวติดตั้งเดียว
ทุกโปรแกรมทำงานแบบออฟไลน์ เขียนด้วย Rust และเปิด/บันทึกไฟล์มาตรฐานที่ใช้กันทั่วไปได้

> [!IMPORTANT]
> **OpenCraft Studio เป็น fork ที่ไม่เป็นทางการ** ทุกโปรแกรมพัฒนาต่อจากต้นฉบับของ
> [ทีม ArtCraft](https://getartcraft.com/) ที่เผยแพร่ใน [github.com/storytold](https://github.com/storytold)
> ภายใต้สัญญาอนุญาต MIT หรือ Apache-2.0 ทีม ArtCraft ไม่ได้สร้าง รับรอง หรือดูแล fork นี้
> และตามเงื่อนไขแบรนด์ของต้นฉบับ fork นี้จึงไม่ใช้ชื่อและโลโก้ ArtCraft
> รายการที่แก้ไขอยู่ใน [CHANGES.md](CHANGES.md) และเครดิตอยู่ใน [CREDITS.md](CREDITS.md)

### ดาวน์โหลดและติดตั้ง

ดาวน์โหลด `OpenCraft-Studio-Setup-<version>-x64.exe` จากหน้า
[Releases](https://github.com/SoloDevTH/OpenCraftTH/releases) แล้วเลือกโปรแกรมที่ต้องการติดตั้ง
(เปิดตัวติดตั้งอีกครั้งเพื่อเพิ่มหรือลบโปรแกรมได้) ต้องใช้ Windows 10 หรือ 11 แบบ 64 บิต

ตัวติดตั้งยังไม่ได้เซ็นรับรอง (code signing) ถ้า Windows SmartScreen เตือน ให้กด **More info ▸ Run anyway**

ทุกโปรแกรมจะอยู่ในเมนู Start และในเมนู **Open with** (คลิกขวา) ของไฟล์ที่รองรับ
และเลือกเป็นโปรแกรมเริ่มต้นได้ที่ **Settings ▸ Apps ▸ Default apps**

| โปรแกรม | ใช้ทำอะไร |
|---|---|
| CADCraft | เขียนแบบ CAD เปิดไฟล์แบบ DXF และ DWG |
| DesignCraft | จัดหน้าหนังสือ นิตยสาร และโปสเตอร์ |
| EffectCraft | ไตเติลเคลื่อนไหว โมชันกราฟิก และงานคอมโพสิต |
| FilmCraft | ตัดต่อวิดีโอ ไทม์ไลน์หลายแทร็ก ปรับสี และเสียง |
| GridCraft | ตารางคำนวณพร้อมสูตร เปิดไฟล์ Excel และ CSV |
| LightCraft | คลังรูปและห้องมืดแต่งไฟล์ RAW ไม่ทำลายต้นฉบับ |
| PdfCraft | อ่าน ใส่หมายเหตุ กรอกฟอร์ม เซ็น และ OCR ไฟล์ PDF |
| PhotoCraft | แต่งภาพแบบเลเยอร์ เปิดไฟล์ PSD ของ Photoshop |
| VectorCraft | วาดภาพเวกเตอร์ รองรับ SVG, AI, EPS และ PDF |
| WordCraft | พิมพ์เอกสาร เปิดไฟล์ Word, ODT และ RTF |

### ความเป็นส่วนตัว

ไม่มีโปรแกรมใดส่งข้อมูลออกไปเอง (ไม่มี telemetry ไม่มีบัญชีผู้ใช้ ไม่มีการเช็กอัปเดตอัตโนมัติ)
จะต่ออินเทอร์เน็ตเฉพาะเมื่อคุณสั่ง เช่น คลิกลิงก์ หรือดาวน์โหลดโมเดลเสริม
ข้อมูลชื่อสถานที่ของ LightCraft และโมเดล Roto Brush / Face Tracking ของ EffectCraft มีมาให้ในตัวแล้ว

### สร้างจากซอร์สโค้ด

ต้องมี Rust 1.95 ขึ้นไป, MSVC build tools และ .NET 6 runtime ขึ้นไป แล้วรัน `installer\build.cmd`
ตัวติดตั้งจะอยู่ในโฟลเดอร์ `installer\dist\`

### เครดิตและสัญญาอนุญาต

ทุกโปรแกรมสร้างโดย**ทีม ArtCraft และผู้ร่วมพัฒนา** ใช้สัญญาอนุญาต MIT หรือ Apache-2.0 (เลือกได้) เหมือนต้นฉบับ
ฟอนต์ ไอคอน โมเดล และข้อมูลที่รวมมาด้วยใช้สัญญาอนุญาตของแต่ละรายการ ดู [CREDITS.md](CREDITS.md)
