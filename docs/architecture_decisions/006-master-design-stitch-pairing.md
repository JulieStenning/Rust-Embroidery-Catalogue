# Architecture Decision Record 006: Master Design & Stitch File Pairing and Lifecycle

* **Status:** Accepted
* **Deciders:** Julie Stenning
* **Date:** 2026-10-07

---

## Context and Problem Statement

Embroidery digital libraries typically contain two distinct classes of files for the same embroidery design:
1. **Master Design Formats (`.ART`, `.EMB`, `.JAN`, `.BE`, etc.):** Native, object-based working design files created and edited inside professional digitizing software (e.g. Bernina, Wilcom, Embrilliance).
2. **Machine Stitch Formats (`.PES`, `.JEF`, `.VP3`, `.DST`, etc.):** Renderable, coordinate-based stitch sequences exported specifically for transfer to embroidery hardware.

Users need to view preview thumbnails, stitch metrics, and thread color stops (extracted from stitch files) while retaining the ability to quickly open and edit the original editable master design file in their preferred desktop software.

In earlier versions, importing required both files to be imported at the exact same time. If a user imported a batch of stitch files and later added the master files (or vice versa), the files remained unlinked or resulted in duplicate catalogue entries.

---

## Decision Drivers

* **Unified Representation:** A design in the catalogue should represent the conceptual embroidery work, rendering visual stitch metadata while seamlessly linking to its corresponding master working file.
* **Asynchronous Stem Pairing:** Importing files in any order (master first, or stitch file first) must automatically detect matching file stems and associate them without creating duplicate design rows.
* **Independent External Software Launch:** The UI must provide distinct, unambiguous actions to open the working master design in default digitizing software or inspect the raw stitch file.
* **Complete Drift Reconciliation:** Missing file scans and unmatched file imports must track both master designs and stitch files.

---

## Considered Options

1. **Strict Simultaneous Import:** Force users to import master files and stitch files in the same folder during a single import session.
2. **Treat Master Files as Standalone Designs:** Parse master container formats directly and catalogue them as separate independent designs (complex, error-prone for proprietary binary formats).
3. **Relational File Pairing via Canonical Stems (Chosen):** Catalogue stitch files as the primary rendered design entity, with relational linkage to master design files stored in `master_design_files`. Auto-link matching stems asynchronously and support dual launch actions.

---

## Decision Outcome

**Option 3 (Relational File Pairing via Canonical Stems)** was chosen.

### Architectural Invariants:
1. **Relational Linking:** The `designs` table stores metadata and stitch coordinates, linking to `master_design_files` via `master_design_file_id`.
2. **Asynchronous Pairing:** During bulk scan and unmatched file reconciliation, if a stitch design matches an existing master design stem (or a master design matches an existing stitch design stem in the same directory), the system links them automatically.
3. **Dual IPC Launch Actions:** Backend exposes dedicated commands `open_design_file` and `open_master_design_file` to independently launch the respective file via system handlers.
4. **Reconciliation Inclusion:** Scans for missing disk files and unmatched folder assets include both master design formats and machine stitch formats.
