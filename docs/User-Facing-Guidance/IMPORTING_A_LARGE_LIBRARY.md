# Importing a Large Library

Importing a large collection—such as 20,000 to 100,000 designs—works the same way as importing a small folder, but requires consideration of disk space and time.

---

## 1. Prepare your storage

Because every imported design has a copy kept in managed storage and generates a thumbnail preview:

1. Check that your destination drive has sufficient free disk space (allow roughly 1 GB of free space per 10,000 designs for files and thumbnails).
2. If using an external drive or SD card, make sure it is formatted as **exFAT** or **NTFS** (not FAT32).
3. If using an SD card, use an **A2-rated** card for fast performance. See [Data Storage Guide](DATA_STORAGE_GUIDE.md).

---

## 2. Review large folder lists efficiently

When scanning a large folder tree:

- **Filter folders**: Type in **Filter folders by name or path…** to search for specific folders in the review list.
- **Collapsed folders**: Large folders start collapsed to keep the review screen fast. Click **Show files** on any folder to inspect individual files.
- **Top and bottom controls**: The **Continue with N designs**, **Select all**, and **Deselect all** buttons appear at both the top and bottom of the list so you do not need to scroll through all pages.

---

## 3. Monitor import progress

While the import runs:

- Keep the application open.
- The progress bar shows the current file count, percentage, and time remaining.
- If you need to stop, click **Stop running**. The import safely halts after the current file; designs already imported remain in your catalogue.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| Scanning or importing stops with disk full errors | Check your drive space. If the drive is formatted as FAT32, reformat it as exFAT or NTFS. |
| You accidentally interrupted an import | Return to [Import](#/import) and re-scan the folder. Designs already imported are skipped automatically, so the scan will pick up only the remaining files. |
| AI tagging did not run | Imports apply local keyword and stitching rules only. To add AI vision tags, run them in batches from [Admin → Batch Operations](#/admin/batch-operations). |

---

## Related guides

- [Import Workflow](IMPORT_WORKFLOW.md)
- [Data Storage Guide](DATA_STORAGE_GUIDE.md)
- [Folder & Designer Assignment](IMPORT_FOLDER_ASSIGNMENT.md)
- [Batch Operations](BATCH_OPERATIONS_BACKFILL.md)
