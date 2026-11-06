# Batch Operations and Maintenance

Use [Admin → Batch Operations](#/admin/batch-operations) to update or enrich designs in bulk across your library.

You can run automated tagging, detect stitch types from embroidery geometry, refresh preview images, and backfill colour and stitch counts.

---

## Tag designs using File & Folder Rules (free and offline)

1. Open [Admin → Batch Operations](#/admin/batch-operations).
2. On the **Tagging & Categorisation** tab, choose **Apply file & folder rules** under **Goal**.
3. Under **Target Scope**, choose whether to tag **Untagged designs only**, a **Specific Folder(s)**, or your **Entire collection**.
4. Choose a **Merge Mode**:
   - **Add to existing tags**: keeps your current tags and adds any newly discovered ones.
   - **Reset and replace all tags**: replaces unverified tags with fresh results.
5. Click **Start Tagging**, then confirm.

---

## Enrich designs with Gemini Vision (AI tagging)

AI tagging sends design thumbnails to Google Gemini to suggest subject tags (such as flowers, animals, or holidays). It requires an API key.

1. If you haven't already entered your Google API key, add it in [Admin → Settings](#/admin/system/settings). See [Settings](SETTINGS.md).
2. In [Admin → Batch Operations](#/admin/batch-operations), choose **Enrich with Gemini Vision** under **Goal**.
3. Under **Target Scope**, select which designs to analyse (e.g. **Designs missing Gemini Vision analysis** or **Specific Folder(s)**).
4. Click **Start Tagging**, then confirm.
5. Keep the application open while the batch runs.

---

## Run file maintenance (stitching, images, and colour counts)

1. In [Admin → Batch Operations](#/admin/batch-operations), click the **Maintenance & File Processing** tab.
2. Under **1. Target Scope**, choose **Entire collection** or **Specific folder(s)**.
3. Under **2. Maintenance Tasks**, tick the tasks you want to run:
   - **Detect / recalculate stitching tags**: analyses stitch geometry to detect techniques (such as Satin Stitch, Applique, or In The Hoop).
   - **Generate preview images**: regenerates thumbnail previews and dimensions.
   - **Recalculate colour / stitch counts**: refreshes stitch totals, unique thread colours, and colour changes from the files.
   - **Recalculate hoop & dimensions**: updates hoop matching and bounding dimensions.
4. Click **Start Maintenance**, then confirm.

---

## Find missing or unmatched design files

If designs were moved or copied outside the app:

1. Go to [Admin → System Maintenance → Library Discrepancies](#/admin/system/orphans).
2. Choose a tool:
   - **Find Unmatched Design Files**: scans for physical files on disk that have no catalogue entry. Click **Scan for unmatched files**, then **Import** to add them.
   - **Missing Design Files (Orphans)**: scans the database for records whose files have been removed from disk. Click **Scan Disk**, then delete orphaned database records.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| AI tagging fails with "429 Too Many Requests" | Tick **My Google API key is on the free tier** in [Admin → Settings](#/admin/system/settings). |
| You need to interrupt a long run | Click **Stop running**. Designs processed so far remain saved; progress can be resumed later. |
| Some files failed to process | Click **Download error log** on the completion banner to view file-level details. |

---

## Related guides

- [Settings](SETTINGS.md)
- [Stitch Types](STITCH_TYPES.md)
- [Colour Counts](COLOUR_COUNTS.md)
- [Tag Word Matches](TAG_WORD_MATCHES.md)
- [Image Generation](IMAGE_GENERATION.md)
