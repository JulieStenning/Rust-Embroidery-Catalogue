# Architecture Decision Record 001: Immutable Original Files & Non-Destructive Cataloguing

* **Status:** Accepted
* **Deciders:** Julie Stenning
* **Date:** 2026-10-06

---

## Context and Problem Statement

Users of the Embroidery Catalogue desktop application manage large and valuable collections of embroidery stitch files (e.g., `.pes`, `.jef`, `.hus`, `.vp3`, `.dst`). These files are frequently organised in bespoke directory structures on external storage drives or local libraries.

If an application moves, alters, overwrites, or deletes original design files during indexing, thumbnail rendering, or tagging, it creates a severe risk of data loss, design corruption, or broken workflows in external embroidery digitising software.

---

## Decision Drivers

* **Zero Data Loss:** Original design files must remain untouched under all conditions.
* **Non-Destructive Inspection:** Reading metadata, thread colours, stitch bounds, and stitch counts must be strictly read-only.
* **Local Storage Separation:** All thumbnails, cache data, database files, and search indices must reside in designated application storage folders, separate from user library folders unless explicitly configured.

---

## Considered Options

1. **In-Place Modification:** Embed metadata or thumbnail streams into proprietary embroidery container headers.
2. **Library Relocation:** Force all embroidery designs into a centralized internal app store upon import.
3. **Immutable Reference Model (Chosen):** Treat original embroidery files as strictly read-only source references, caching thumbnails and metadata in local SQLite/cache directories without modifying the originals.

---

## Decision Outcome

**Option 3 (Immutable Reference Model)** was chosen.

### Architectural Invariants:
1. Original embroidery design files must **NEVER** be renamed, edited, mutated, or deleted by the application.
2. Deleting a design within the application removes catalogue metadata and cached thumbnails; it does not delete original files outside the library root unless the user explicitly executes a confirmed file trash operation.
3. All binary parsing logic in Rust operates via read-only file streams (`File::open` + `BufReader`).
