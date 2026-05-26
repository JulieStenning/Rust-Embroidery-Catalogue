# Stitch and Colour Counts

The Embroidery Catalogue can store technical embroidery metadata for each design:

- stitch count,
- colour count,
- colour-change count.

This guide covers the supported **UI workflow** for filling and reviewing these values.

---

## What gets stored

For each design, the app can store:

- **Stitches**: total stitch count
- **Colours**: number of unique thread colours
- **Colour changes**: number of colour-change commands

These values are stored per design in the catalogue database and shown in selected UI screens.

---

## When values are populated

### During import

When you import new designs, the scanner reads pattern data and attempts to populate stitch/colour counts as part of normal metadata extraction.

If a file cannot be parsed for counts, fields may remain empty until a later backfill run.

### During Tagging Actions backfill

Use **Admin -> Tagging Actions** and select **Threads and Colours** to backfill missing values for existing designs.

This is the recommended path for large libraries that were imported before colour counts were available or for designs with missing technical metadata.

---

## How to run Threads and Colours

1. Open **Admin -> Tagging Actions**.
2. Tick **Threads and Colours**.
3. Optionally combine with other actions in the same run.
4. Choose batch/commit/worker values (or keep defaults).
5. Click **Run selected actions**.
6. Keep the page open until completion.
7. Review progress and, if needed, download the error log.

For full combined-action guidance, see [TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md).

---

## Where you can see the values

Colour-count metadata is currently visible in:

- Design detail page:
  - Stitches badge
  - Colours badge
  - Colour changes badge
- Design print view:
  - Stitches
  - Colours
  - Colour changes
- Project print view:
  - Stitches
  - Colours
  - Colour changes

---

## Interpreting missing values

If one or more fields are blank for a design, common reasons are:

- file format not parseable by the current reader,
- source file missing or moved,
- parse/read error during import or backfill.

Run **Threads and Colours** again after fixing source-file issues.

---

## Troubleshooting

| Problem | What to do |
|---|---|
| Values stay empty after backfill | Check that the source file exists and is readable, then rerun **Threads and Colours**. |
| Some designs updated, some not | Download the error log from Tagging Actions and review file-level failures. |
| Run is too slow | Use smaller batches and/or lower workers; keep defaults if unsure. |
| Browser closed during run | Reopen Tagging Actions and run again; keep the page open to monitor completion. |

For broader application issues, see [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md).

---

## Notes for pre-release cleanup

This guide documents the supported user path (UI).
Any standalone colour-count scripts or maintenance-only routes are internal/decommission paths and are not part of the supported user workflow.
