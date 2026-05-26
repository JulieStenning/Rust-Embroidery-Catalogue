# Image Generation

The Embroidery Catalogue can generate preview images and related metadata from your embroidery files.
This feature reads the design file, interprets stitch/digitising data, and stores a preview image with dimensions and technical counts.

---

## What this feature is for

Use image generation when you want to:

- create missing design previews,
- refresh existing previews after an upgrade,
- switch previously-generated 2D previews to 3D,
- fill missing image dimensions and hoop suggestions,
- update stitch and colour count metadata as part of maintenance.

You can run image generation in two primary ways:

- during import confirmation,
- from **Admin -> Tagging Actions** for existing designs.

---

## Before you run

1. Keep the browser page open while actions are running.
2. Back up before large regeneration runs.
3. Start with a smaller batch if this is your first full image pass.
4. Use default settings first unless you have a performance reason to tune them.

> Tip: For very large libraries, run multiple smaller passes and review logs between runs.

---

## Quick start

### Import-time image generation

1. Start an import and reach the confirmation step.
2. Choose your image preference (2D or 3D) for generated previews.
3. Confirm import.
4. Let the import complete and review generated previews.

### Admin Tagging Actions image generation

1. Open **Admin -> Tagging Actions**.
2. Select **Images**.
3. Choose the image options you need:
   - **Re-process all images**
   - **Upgrade existing 2D images to 3D**
   - **Use fast 2D preview (skip 3D rendering)**
4. Choose batch/commit/workers settings.
5. Click **Run selected actions**.
6. Monitor progress and download error log if needed.

---

## 2D vs 3D preview modes

### 3D preview

Use 3D when visual quality is the priority.

- More realistic stitch simulation.
- Better for final browsing and verification.
- Slower than 2D on large runs.

### Fast 2D preview

Use 2D when throughput is the priority.

- Faster bulk processing.
- Good for first-pass population and low-resource machines.
- Less visual depth than 3D.

### Upgrade workflow

Use **Upgrade existing 2D images to 3D** when you already have 2D previews and want to improve quality without fully reprocessing all records.

---

## Option behavior

### Re-process all images

- Regenerates image data for all selected designs.
- Also refreshes dimensions and related image metadata.
- Intended for full rebuilds and migration scenarios.

### Upgrade existing 2D images to 3D

- Targets designs currently marked as 2D (or legacy image type).
- Leaves already-3D images untouched.
- Useful after using fast 2D mode in earlier runs.

### Use fast 2D preview

- Runs the image pipeline in 2D mode.
- Works well for high-volume maintenance passes.
- Can be upgraded later with the 2D-to-3D option.

---

## Performance controls

The image pipeline in Tagging Actions exposes:

- **Batch size**
- **Commit every**
- **Workers**

Practical guidance:

- Keep defaults for normal runs.
- Lower values on constrained hardware.
- Increase **Workers** carefully to avoid CPU and disk saturation.
- If performance degrades, reduce workers first.

---

## Progress, stopping, and logs

- Progress area shows processed/error totals.
- Use **Stop running** for a graceful stop request.
- Use **Download error log** after a run if failures occurred.

Error log usage:

- Identify file-specific failures (missing/corrupt/unsupported files, read/render issues).
- Fix root causes and rerun targeted actions.

---

## Troubleshooting

| Problem | What to do |
|---|---|
| No previews were generated | Confirm **Images** action is selected and rerun. |
| Many image errors in log | Verify source files still exist and are readable, then rerun affected designs. |
| Run seems slow | Lower workers and run smaller batches. |
| Existing previews did not change | Use **Re-process all images** or **Upgrade existing 2D images to 3D** depending on goal. |
| Dimensions/hoop are still missing on some files | Re-run images and check file-specific log errors; some formats may have limited metadata paths. |
| Browser closed during run | Reopen Tagging Actions and rerun required actions. |

For broader issues, see [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md).

---

## Recommended workflows

### Fast first pass for large libraries

1. Run **Images** with **Use fast 2D preview**.
2. Validate coverage and review error log.
3. Run **Upgrade existing 2D images to 3D** for final quality.

### Full rebuild

1. Back up first.
2. Run **Images** with **Re-process all images**.
3. Use 3D mode if visual quality is the priority.
4. Download and review error log after completion.

### Import-first workflow

1. Import with your preferred image mode.
2. Review results in browse/detail views.
3. Run Tagging Actions image pass only for records that need refresh or upgrade.

---

## Related guides

- [TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md) - unified maintenance actions overview
- [SUPPORTED_FORMATS.md](SUPPORTED_FORMATS.md) - format compatibility and notes
- [COLOUR_COUNTS.md](COLOUR_COUNTS.md) - stitch/colour metadata behavior
- [BACKUP_RESTORE.md](BACKUP_RESTORE.md) - backup guidance before large operations
