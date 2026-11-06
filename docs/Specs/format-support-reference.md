# Embroidery Format Support & Exclusions Reference

Technical specification of supported binary stitch readers, digitising master outline formats, and format exclusion decisions (`src/readers/`, `src/services/scanning.rs`).

## Supported Machine Stitch Formats

Native binary parsing is implemented in pure Rust under `src/readers/`.

| Extension | Reader Module | Capabilities | Limitations |
|---|---|---|---|
| `.pes` | `pes_reader.rs` | Multi-needle thread palette, stitch coordinates, jumps, trims, dimensions. | Multiple internal PEC/PES sub-versions. |
| `.jef` | `jef_reader.rs` | Janome colour index, stitch coordinates, jumps, hoop code dimensions. | Palette mapped from built-in Janome thread tables. |
| `.vp3` | `vp3_reader.rs` | Extended RGB thread colours, hoop geometry, multi-block stitch paths. | Hierarchical block tree structure. |
| `.hus` | `hus_reader.rs` | Legacy Viking palette, stitch coordinates, jumps. | Compressed bitstream encoding. |
| `.dst` | `dst_reader.rs` | Industrial Tajima stitch coordinates, jumps, stop commands. | No embedded colour palette; stops represent colour-change commands. |
| `.exp` | `exp_reader.rs` | Melco/Bernina stitch coordinates, jumps, stops. | Monochrome stitch-command stream; no embedded palette. |

## Digitising Master & Outline Formats (Configurable)

Configured via `Admin → Settings` (`settings.enabled_master_formats`). Master formats preserve vector objects and digitising metadata:

| Extension | Associated Software | Mode | Pairing Behaviour |
|---|---|---|---|
| `.eof` | Embird Studio | Master outline | Paired with matching stitch file (`.pes`, `.jef`, etc.) into single card with `[Master]` badge. |
| `.ecf` | Embird Cross Stitch | Cross-stitch project | Paired with matching stitch file. |
| `.emb` | Wilcom / Hatch | All-in-one object design | Paired with matching stitch file. |
| `.art` | Bernina | Working design | Paired with matching stitch file (e.g. `.exp`, `.pes`). |
| `.be` | Embrilliance | Working design | Paired with matching stitch file. |
| `.jan` | Janome Digitizer | Working design | Paired with matching stitch file. |
| `.edo` | mySewnet / Premier+ | Design Outline | Paired with matching stitch file. |
| `.vp4` | mySewnet / Viking | Interactive design | Paired with matching stitch file. |

- **Paired Entry:** If a master file and a stitch file share the same base stem in the same folder (e.g. `rose.eof` and `rose.pes`), they form one catalogue entry. The machine stitch file provides the thumbnail preview and stitch data; the master file path is accessible via **Open in Editor**.
- **Master-Only Entry:** If only a master outline exists without an accompanying machine stitch file, it is catalogued with a placeholder icon and marked for export.

## Excluded Formats & Technical Rationale

The scanner intentionally skips the following formats.

| Format | Category / Origin | Reason / Technical Rationale |
|---|---|---|
| `.bro` | Brother legacy | Excluded due to low-detail decode output (produces sparse outline-only preview vectors insufficient for quality rendering). |
| `.ksm` | Pfaff legacy | Excluded due to low-detail decode output (outline-only vectors). |
| `.pcd` | Legacy format | Excluded due to unreliable/incomplete decodes observed in Embird-generated source files during benchmark testing. |
| `.col` | Sidecar file | Thread colour definition sidecar; contains no stitch geometry. |
| `.edr` | Sidecar file | Embird colour palette sidecar; contains no stitch geometry. |
| `.inf` | Sidecar file | Thread information sidecar; contains no stitch geometry. |
| `.json`, `.csv`, `.txt` | Data files | Text exports, log files, or metadata manifests; not embroidery files. |
| `.png`, `.jpg`, `.svg` | Images | Raster thumbnails or vector artwork; not machine stitch files. |
| `.zip` | Archive | Font packages are catalogued through representative preview glyphs; raw `.zip` archives are not parsed as standalone stitch files. |

