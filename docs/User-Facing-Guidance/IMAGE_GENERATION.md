# Preview Images & Thumbnail Generation

Embroidery Catalogue automatically renders preview images and calculates hoop dimensions from your stitch files.

---

## Generate missing previews

Previews are created automatically when designs are imported. If some designs are missing previews (e.g. after a file restore or disk sync):

1. Open [Admin → Batch Operations](#/admin/batch-operations).
2. Click the **Maintenance & File Processing** tab.
3. Under **Target Scope**, choose **Entire collection** or **Specific folder(s)**.
4. Tick **Generate preview images**.
5. Click **Start Maintenance**, then confirm.

---

## Refresh existing previews

To refresh previews after changing display settings or if thumbnails appear out of date:

1. Open [Admin → Batch Operations](#/admin/batch-operations) → **Maintenance & File Processing**.
2. Tick **Generate preview images**.
3. Click **Start Maintenance**, then confirm.
4. Keep the app open while the images are refreshed.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| A design shows a placeholder icon instead of an image | The design file may be in an outline-only master format (such as `.be` or `.eof`). Export it to a machine stitch format (such as `.pes` or `.jef`) to render a preview. |
| Thumbnail generation is slow | On large collections, run maintenance on specific folders rather than your entire library. |
| You want to stop a running batch | Click **Stop running**. Completed thumbnails are preserved. |

---

## Related guides

- [Batch Operations](BATCH_OPERATIONS_BACKFILL.md)
- [Supported Formats](SUPPORTED_FORMATS.md)
- [Design Detail](DESIGN_DETAIL.md)
