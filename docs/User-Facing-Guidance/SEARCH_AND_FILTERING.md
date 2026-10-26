# Searching and Filtering Designs

The **Browse Designs** view provides quick keyword search and fine-grained filter controls to help you locate embroidery files across large libraries.

---

## 🔍 General Search

The main search box supports Google-style search operators to match keywords across filenames, folder paths, notes, and tags:

- **Keywords:** `rose flower` (matches designs containing both words).
- **Exact phrase:** `"cross stitch"` (matches the exact quoted phrase).
- **Exclusion:** `-applique` (excludes designs containing the term).
- **OR search:** `rose OR tulip` (matches designs containing either term).
- **Filename wildcard:** `*.pes` or `rose*.jef` (matches specific file patterns).
- **Search scopes:** Tick or untick **File name**, **Folder name**, and **Tags** to restrict where the query looks.
- **Verification status:** Use the **Verification** dropdown next to the search input to filter by **All**, **Unverified only** (designs with tags requiring review), **Verified only** (fully verified designs), or **Partially verified**.

### ⏳ Live Search, Busy Indicator & Cancellation
- **Live Search & Busy Indicator:** As you type into the general search box or adjust filters, an animated spinner appears inside the search input and the selection bar displays **"Searching designs..."**.
- **Cancelling a Search:** When searching large catalogues, an inline red **Cancel** button appears in the search box while a query is running. Clicking **Cancel** (or pressing the **`Escape`** key) immediately aborts the active search and restores your previous results.
- **Clearing Search Input:** When idle, clicking the **`✕`** button on the right of the search input or pressing the **`Escape`** key clears the text query.

### 💡 Practical Tip: Clustered Search for Fast Tag Verification
Automated tags from imports or AI runs do not have to be 100% accurate to save time. By searching for a keyword (such as `birds`) across filenames, folder paths, and tags while checking **Unverified only** (and optionally filtering by broad tags like *Animals* or *Food*), you isolate a concentrated cluster of similar designs. This allows you to inspect, retag, or verify entire rows or pages in one go using [Browse Bulk Actions](BROWSE_BULK_ACTIONS.md#rapid-tag-verification-using-search--bulk-selection-clustering).

---

## 🎛️ Additional Filters

Click **Additional Filters** to expand advanced criteria:

### 1. Categories & Attribution
- **Designer:** Multi-select checkboxes to filter by one or more designers.
- **Image Tags:** Multi-select semantic subject tags (e.g. *Floral*, *Animals*, *Holidays*).
- **Stitching Tags:** Multi-select technical stitch tags (e.g. *Applique*, *Filled*, *Line Work*).
- **Source:** Multi-select acquisition sources.

### 2. Hoop Size & Dimensions (mm)
- **Hoop size:** Filter by specific embroidery hoop frame (e.g. *Hoop A*, *Hoop B*) or find designs with uncalculated hoops (*Hoop unknown*).
- **Dimensions (mm):**
  - **Width (W):** Set a **Min** and/or **Max** width in millimetres.
  - **Height (H):** Set a **Min** and/or **Max** height in millimetres.
  - **Open-ended ranges:**
    - Entering only a **Max** (e.g. Max Height = `80`) matches designs from `0 mm` up to `80 mm`.
    - Entering only a **Min** (e.g. Min Width = `100`) matches designs from `100 mm` and wider.
    - Entering both sets a strict bounded range (e.g. `50 mm` to `150 mm`).

### 3. Ratings, Verification & Sewing Status
- **Image Tags Verification:** Filter by **Any**, **Verified only**, or **Unverified only**.
- **Stitching Tags Verification:** Filter by **Any**, **Verified only**, or **Unverified only**.
- **Minimum rating:** Filter by star rating (1★ to 5★).
- **Stitched status:** Filter by **Stitched** or **Not Stitched**.
- **Needs attention:** Restrict to designs missing preview images (e.g. unreadable or corrupted files) or master-only outline files awaiting machine stitch file export.

---

## 🔄 Resetting Filters

Click **Reset filters** at the bottom of the filter controls or clear the general search input to restore default browse view settings.

---

## Related Guides

- [BROWSE_BULK_ACTIONS.md](BROWSE_BULK_ACTIONS.md)
- [DESIGN_DETAIL.md](DESIGN_DETAIL.md)
- [GETTING_STARTED.md](GETTING_STARTED.md)
