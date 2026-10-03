# Supported Embroidery Formats

This document lists the embroidery file formats supported by the Embroidery Catalogue import scanner.

---

## Standard import formats

These formats are read by the application's built-in Rust readers.  Dimensions, hoop
suggestion, and stitch preview are available for all of them where the file contains
the necessary data.

| Extension | Notes |
|-----------|-------|
| `.jef`    | Janome |
| `.pes`    | Brother / Babylock |
| `.hus`    | Viking/Husqvarna |
| `.vp3`    | Viking/Pfaff |
| `.dst`    | Tajima (industrial/interchange) |
| `.exp`    | Melco (industrial/interchange) |

---

## Helper format

| Extension | Notes |
|-----------|-------|
| `.pmv`    | Pfaff My Quilter helper format; included for catalogue completeness. |

---

## Master & Digitising Outline Formats (Configurable)

In addition to machine stitch formats, the catalogue supports indexing master digitising and outline design files. When enabled in **Admin → Settings**, these files are catalogued alongside machine formats:

- **Paired Designs:** When a master file and a stitch file share the same base name in the same folder (e.g. `flower.eof` and `flower.pes`), the catalogue pairs them into a single entry with a `[🎨 Master]` badge. The machine stitch file provides the thumbnail preview and stitch data, while both files can be opened in their respective applications.
- **Master-Only Designs:** If a master outline file exists without a corresponding stitch file, it is catalogued with a placeholder and marked for export (*"Export to a machine stitch format to generate preview and stitch data."*).

| Extension | Associated Software / Eco-system | Notes |
|-----------|----------------------------------|-------|
| `.eof`    | Embird                           | Embird Digitizing Studio master outline format |
| `.ecf`    | Embird                           | Embird Cross Stitch master format |
| `.emb`    | Wilcom / Hatch                   | All-in-one object & stitch design format |
| `.art`    | Bernina                          | Bernina ArtLink / Designer master format |
| `.be`     | Embrilliance                     | Embrilliance working design format |
| `.jan`    | Janome Digitizer                 | Janome Digitizer working format |
| `.edo`    | mySewnet / Premier+              | mySewnet Design Outline format |
| `.vp4`    | mySewnet / Viking                | mySewnet interactive design file |

Custom master extensions can also be configured in **Admin → Settings**.

---

## Excluded formats

The following formats are intentionally **not** included in the import allowlist:

| Extension | Reason |
|-----------|--------|
| `.json`   | Output / helper format, not a stitch file |
| `.col`    | Colour sidecar file, not a stitch file |
| `.edr`    | Colour sidecar file, not a stitch file |
| `.inf`    | Colour sidecar file, not a stitch file |
| `.bro`    | Excluded due to low-detail decode output (outline-only previews) |
| `.ksm`    | Excluded due to low-detail decode output (outline-only previews) |
| `.pcd`    | Excluded because source files generated in Embird were unreliable/incomplete |
| `.svg`    | Vector graphic, not an embroidery format |
| `.csv`    | Data export, not a stitch file |
| `.png`    | Raster image, not an embroidery format |
| `.txt`    | Plain text, not a stitch file |
