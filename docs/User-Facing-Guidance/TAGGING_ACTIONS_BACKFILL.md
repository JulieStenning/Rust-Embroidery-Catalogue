# Tagging Actions and Backfill

The Embroidery Catalogue includes a **Tagging Actions** page that lets you update existing designs in bulk.
You can run AI tagging, stitching detection, image generation, and thread/colour count backfills from one place.

---

## What this page is for

Use **Admin -> Tagging Actions** when you want to:

- tag designs that were imported earlier,
- detect stitch types from embroidery geometry,
- generate or refresh preview images and dimensions,
- fill in missing stitch/colour/count metadata,
- run multiple maintenance actions in one run.

This is especially useful after large imports, folder cleanups, or app upgrades.

---

## Before you run

1. Keep the page open while actions run.
2. Make a backup first for large runs.
3. If using AI tiers (Tier 2 or Tier 3), confirm your API key is configured in **Admin -> Settings**.
4. Start with a small run if this is your first time using combined actions.

> Tip: If your library is very large, run in smaller batches over multiple sessions.

---

## Quick start

1. Open **Admin -> Tagging Actions**.
2. Tick one or more actions:
   - **Tagging (AI/keyword)**
   - **Stitch types**
   - **Images**
   - **Threads and Colours**
3. Configure options for each selected action.
4. Choose batch/commit/worker values.
5. Click **Run selected actions**.
6. Watch the progress message on the page.
7. If needed, click **Download error log** after completion.

---

## Action guide

### Tagging (AI/keyword)

Use this to apply or refresh image-group tags.

Tagging mode options:

- **Tag only untagged**
- **Re-tag ALL unverified**
- **Re-tag ALL (including verified)**

Tier options:

- **Tier 1** is always included.
- **Tier 2** and **Tier 3** require a configured Google API key.

When to use:

- Use **Tag only untagged** for routine top-up runs.
- Use **Re-tag ALL unverified** when you changed prompts/logic and want to reprocess pending items.
- Use **Re-tag ALL** only when you intentionally want to overwrite verified tag sets.

### Stitch types

Use this to detect stitching tags directly from pattern geometry.

Option:

- **Clear existing stitching tags for unverified designs first**

When to use:

- Run after changing stitch-related tags.
- Run after importing older designs that have no stitching tags.

### Images

Use this to generate previews, dimensions, and hoop suggestions.

Options:

- **Re-process all images**
- **Upgrade existing 2D images to 3D**
- **Use fast 2D preview (skip 3D rendering)**

Important:

- If **Re-process all images** is selected, the app asks for confirmation before clearing existing image fields.

### Threads and Colours

Use this to fill technical metadata fields:

- stitch count,
- colour count,
- colour-change count.

This is typically a safe maintenance action and can be combined with image/stitching runs.

For the dedicated stitch/colour-count walkthrough (import behavior, where values appear in the UI, and troubleshooting), see
[COLOUR_COUNTS.md](COLOUR_COUNTS.md).

For the current stitch-type detector behavior and its roadmap notes, see [STITCH_TYPES.md](STITCH_TYPES.md).

---

## Batch, commit, and workers

The page exposes advanced controls:

- **Batch size**
- **Commit every**
- **Workers**

Practical guidance:

- Keep defaults for most runs.
- Use lower values if your machine is resource-constrained.
- Increase **Workers** carefully; higher values can increase CPU and disk load.

---

## Progress, stopping, and logs

- The progress area shows a running/completed summary.
- Use **Stop running** to request a graceful stop.
- Use **Download error log** to review design-level failures.

Error log notes:

- The log helps identify file-specific problems (unsupported or corrupt files, parse failures, etc.).
- If any errors occurred, rerun targeted actions after fixing the root cause.

---

## Recommended workflows

### Safe routine maintenance

1. Run **Tag only untagged**.
2. Run **Stitch types** (without clearing existing) if needed.
3. Run **Threads and Colours**.
4. Check results and verify tags in Browse/Detail views.

### Full refresh pass

1. Back up first.
2. Run **Re-tag ALL unverified** with desired tiers.
3. Run **Upgrade existing 2D images to 3D**.
4. Run **Threads and Colours**.
5. Review and download error log.

### Image rebuild pass

1. Select **Images**.
2. Tick **Re-process all images**.
3. Confirm the clear-images prompt.
4. Run and monitor until completion.

---

## Troubleshooting

| Problem | What to do |
|---|---|
| Run does not start | Confirm at least one action is selected, then try again. |
| Tier 2/3 not available | Check API key in Settings and save. |
| Many failures in log | Verify source files still exist and are readable, then rerun affected actions. |
| Browser closed during run | Reopen Tagging Actions and rerun. Keep the page open until completion. |
| Performance is slow | Lower workers and run smaller batches. |

For broader issues, see [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md).

---

## Related guides

- [AI_TAGGING.md](AI_TAGGING.md) - AI tiers, costs, and API setup
- [COLOUR_COUNTS.md](COLOUR_COUNTS.md) - stitch/colour-count behavior and UI usage
- [BACKUP_RESTORE.md](BACKUP_RESTORE.md) - back up before large maintenance runs
- [GETTING_STARTED.md](GETTING_STARTED.md) - basic app setup and run flow
