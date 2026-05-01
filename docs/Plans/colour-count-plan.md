# Plan: Add Stitch Count, Colour Count, and Colour Changes to Design Information

## 1. Findings

### 1.1 pyembroidery (forked version) already provides the data

The [`EmbPattern`](src/services/preview.py) class (from the forked version at `D:\My Software Development\PyEmbroidery\pyembroidery\pyembroidery`) has three methods directly relevant:

| Method | Returns | Meaning |
|---|---|---|
| [`pattern.count_stitches()`](src/services/preview.py) | `int` | Total stitch count |
| [`pattern.count_threads()`](src/services/preview.py) | `int` | Number of unique thread colours in the design |
| [`pattern.count_color_changes()`](src/services/preview.py) | `int` | Number of colour-change commands in the stitch sequence |

These are **zero-cost** calls — they iterate over the already-loaded pattern's internal data. They do **not** re-parse the file or re-run any stitching simulation.

### 1.2 Where the pattern is already loaded

The pattern is loaded via [`pyembroidery.read(filepath)`](src/services/preview.py:335) inside [`_process_file()`](src/services/preview.py:315). This function is the single entry point for all design scanning and preview generation. It is called from:

1. [`scan_folder()`](src/services/scanning.py:199) — during bulk import scanning (with `generate_preview=False`)
2. [`process_selected_files()`](src/services/scanning.py:300) — during import confirmation (with `generate_preview=True`)
3. Potentially other places that need design metadata

The pattern object is available at the point where [`ScannedDesign`](src/services/scanning.py:99) is populated, but currently only `width_mm`, `height_mm`, `hoop_id`, `hoop_name`, and `image_data` are extracted from it.

### 1.3 PngWriter does NOT provide metadata

[`PngWriter.write(pattern, buf, settings={"3d": True})`](src/services/preview.py:309) only renders the PNG image. It does not expose or return any metadata about stitches or colours. The stitch/colour data must be read from the pattern **before** or **after** the PNG render call, but since both use the same pattern object, there is no extra file I/O cost.

### 1.4 The `ScannedDesign` dataclass is the carrier

[`ScannedDesign`](src/services/scanning.py:99) is the dataclass that carries all extracted metadata from scanning through to database persistence. Currently it has no fields for stitch count, colour count, or colour change count. These would need to be added.

### 1.5 The `Design` model stores persisted data

The [`Design`](src/models.py:136) ORM model has no columns for stitch count, colour count, or colour change count. These would need to be added via a new Alembic migration.

### 1.6 Stitch count is not currently displayed

Looking at the detail template [`templates/designs/detail.html`](templates/designs/detail.html:68-76), the only metadata shown is dimensions (`width_mm` × `height_mm` mm) and hoop name. There is no stitch count, colour count, or colour change count anywhere in the current UI.

## 2. Recommended Approach

### Extract from pattern in `_process_file()` (RECOMMENDED)

Since `_process_file()` already has the pattern object, add the extraction there. This is the most efficient approach because:

- The pattern is already loaded into memory
- `count_stitches()`, `count_threads()`, and `count_color_changes()` are O(n) scans of in-memory data, not file re-reads
- No second call to `pyembroidery.read()` is needed
- The data flows naturally through `ScannedDesign` → `Design` ORM record

**Data flow:**

```
pyembroidery.read(filepath) → pattern object
  ├── pattern.count_stitches()       → stitch_count
  ├── pattern.count_threads()        → color_count
  ├── pattern.count_color_changes()  → color_change_count
  ├── pattern.bounds()               → width_mm, height_mm
  └── PngWriter.write(pattern, ...)  → image_data (preview)
```

## 3. Implementation Steps

### Step 1: Add fields to `ScannedDesign` dataclass

File: [`src/services/scanning.py`](src/services/scanning.py:99)

Add three new fields:

```python
stitch_count: int | None = None
color_count: int | None = None
color_change_count: int | None = None
```

### Step 2: Extract values in `_process_file()`

File: [`src/services/preview.py`](src/services/preview.py:315)

After the pattern is successfully loaded and bounds are extracted (around line 358), add:

```python
scanned.stitch_count = pattern.count_stitches()
scanned.color_count = pattern.count_threads()
scanned.color_change_count = pattern.count_color_changes()
```

### Step 3: Add columns to `Design` ORM model

File: [`src/models.py`](src/models.py:136)

Add three new nullable integer columns:

```python
stitch_count: Mapped[int | None] = mapped_column(Integer, nullable=True)
color_count: Mapped[int | None] = mapped_column(Integer, nullable=True)
color_change_count: Mapped[int | None] = mapped_column(Integer, nullable=True)
```

### Step 4: Create Alembic migration

Generate a new migration to add `stitch_count`, `color_count`, and `color_change_count` columns to the `designs` table.

### Step 5: Persist the values during import

File: [`src/services/bulk_import.py`](src/services/bulk_import.py:198)

In `_build_design_records()`, add the new fields to the `Design(...)` constructor:

```python
design = Design(
    ...
    stitch_count=sd.stitch_count,
    color_count=sd.color_count,
    color_change_count=sd.color_change_count,
    ...
)
```

### Step 6: (Later) Display in the UI

Once the data is in the database, the detail template [`templates/designs/detail.html`](templates/designs/detail.html) can show these values alongside the existing dimensions and hoop info (around line 69-76).

## 4. Key Considerations

- **Performance**: `count_stitches()`, `count_threads()`, and `count_color_changes()` iterate over the in-memory pattern. They are fast and do not re-read the file. The PNG render (`PngWriter.write`) also uses the same pattern object, so there is no duplication of work.
- **Existing designs**: Designs already in the database will have `NULL` for these columns until they are re-scanned or backfilled. A backfill script could be written to update existing records.
- **`.art` files**: For `.art` files that fail to load via pyembroidery, the values will remain `None`. This is consistent with how other metadata (dimensions) is handled for those files. Note that `_read_art_metadata()` already extracts `stitch_count` and `color_count` from the Wilcom metadata stream for `.art` files — this could be used as a fallback if desired.
