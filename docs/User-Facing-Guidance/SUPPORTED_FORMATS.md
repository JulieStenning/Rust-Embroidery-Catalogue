# Supported Embroidery Formats

This document lists the embroidery file formats supported by the Embroidery Catalogue import scanner.

---

## Standard import formats

These formats are read via `pyembroidery`.  Dimensions, hoop suggestion, and stitch
preview are available for all of them where the file contains the necessary data.

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
