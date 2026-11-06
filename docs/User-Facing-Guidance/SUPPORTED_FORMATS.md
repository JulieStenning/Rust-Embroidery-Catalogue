# Supported Embroidery Formats

Embroidery Catalogue imports and displays standard machine stitch formats and digitising master files.

---

## Standard machine stitch formats

Standard machine stitch formats are parsed directly by the built-in reader engine. Dimensions, stitch totals, colour stops, and rendered preview images are generated automatically during import.

| Format | Machine manufacturer / eco-system | Notes |
|---|---|---|
| `.pes` | Brother / Baby Lock / Bernina | Includes colour palette and multi-needle data |
| `.jef` | Janome / Elna | Standard Janome machine format |
| `.vp3` | Husqvarna Viking / Pfaff | Includes thread colours and hoop boundaries |
| `.hus` | Husqvarna Viking | Legacy Viking stitch format |
| `.dst` | Tajima | Standard commercial format (generic colour palette) |
| `.exp` | Melco / Bernina | Commercial and domestic stitch format |

---

## Digitising master & outline formats

You can enable detection for working design files created in popular digitising software under [Admin → Settings](#/admin/system/settings):

- **Paired Designs**: When a master file sits in the same folder as a matching machine stitch file (e.g. `flower.eof` and `flower.pes`), they appear together as a single design card with a **Master** badge.
- **Master-Only Designs**: If only the master file exists, it is catalogued with a placeholder and marked for export to a machine stitch format.

| Format | Software | Notes |
|---|---|---|
| `.eof` | Embird Studio | Embird Digitising Studio master outline format |
| `.ecf` | Embird Cross Stitch | Embird Cross Stitch project format |
| `.emb` | Wilcom / Hatch | Wilcom and Hatch all-in-one object design format |
| `.art` | Bernina | Bernina Embroidery Software working format |
| `.be` | Embrilliance | Embrilliance working design format |
| `.jan` | Janome Digitizer | Janome Digitizer working format |
| `.edo` | mySewnet / Premier+ | mySewnet Design Outline format |
| `.vp4` | mySewnet / Viking | mySewnet interactive design format |

You can add custom file extensions in [Admin → Settings](#/admin/system/settings) under **Digitising & Master File Formats**.

---

## Excluded file types

The import scanner automatically ignores non-stitch and sidecar files:

| Format | Origin / Category | Reason / Notes |
|---|---|---|
| `.bro` | Brother legacy | Excluded due to low-detail outline-only previews |
| `.ksm` | Pfaff legacy | Excluded due to low-detail outline-only previews |
| `.pcd` | Legacy format | Excluded due to unreliable/incomplete decode output |
| `.col`, `.edr`, `.inf` | Sidecars | Thread colour palette files; not stitch data |
| `.json`, `.csv`, `.txt` | Data files | Text exports and metadata; not stitch files |
| `.png`, `.jpg`, `.svg` | Images | Raster images and vector graphics; not embroidery |
| `.zip` | Archives | Font packages are catalogued via sample glyphs, not raw archives |

---

## Related guides

- [Import Workflow](IMPORT_WORKFLOW.md)
- [Settings](SETTINGS.md)
- [Getting Started](GETTING_STARTED.md)
