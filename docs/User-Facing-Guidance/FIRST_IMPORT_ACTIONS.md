# First Import Actions Guide

The import wizard's final step is **Before You Import**. For a brand-new catalogue it
pauses once to point out that no hoops are configured, and it explains how automatic
tagging works. There is no longer a set of "Review Hoops / Tags / Sources / Designers"
buttons on this step — reference data is managed in the Admin pages at any time.

## What the final step shows

- The **Before You Import** panel.
- A note explaining that the import itself uses fast, offline **File & Folder Rules**, and
  that automated **Visual AI** tagging can be run later from **Batch Operations**.
- Two actions: **Import Designs** and **Cancel**.

## Where It Appears

Import wizard flow:
1. Select folders.
2. Review scanned files.
3. **Before You Import** appears.
4. Import Designs, or cancel.

## First Import vs Later Imports

### First import (empty catalogue with no hoops)
- Pressing **Import Designs** does not start the import immediately.
- A warning appears: "Hoops are not configured for a first import. Confirm to continue anyway."
- Choosing **Confirm import without hoop setup** runs the import. Skipping hoop setup does
  not create any hoops for you.

### Later imports
- With at least one hoop configured, **Import Designs** starts the import immediately and
  no warning appears.

## What To Do on the Before You Import Step
You can choose either of these actions:
- **Import Designs** — run the import now (plus the one-off confirmation on a first import).
- **Cancel** — abandon the import and return to step 1.

If hoops matter to you (they drive hoop-size matching and hoop-based filters), add them in
**Admin → Hoops** before your first import.

## Managing Reference Data

Hoops, tags, sources and designers are managed on their own Admin pages
(**Manage Data** / **System** in the top navigation) at any time:
- changes are saved immediately and there is no separate "import mode",
- after editing reference data you can return to **Import** and continue — the wizard keeps
  your scan and selections while you are away.

## Continue and Cancel Behavior
- **Import Designs**: runs the import with your current selections.
- **Cancel** (or **Stop** while a run is in progress): abandons the run and clears the wizard.
- After a successful import the app routes you to **Browse Designs**.

## AI and Image Preference Notes

- The import itself never calls an AI service and needs no Google API key. It always
  applies the free, local keyword and stitching tags.
- **Visual AI** auto-tagging is run separately, after the import, from
  **Admin → Batch Operations**. See [BATCH_OPERATIONS_BACKFILL.md](BATCH_OPERATIONS_BACKFILL.md).
- There is no per-import 2D/3D preview choice. The preview style is a global setting under
  **Admin → Settings**.

## Troubleshooting

| Issue | What to do |
|---|---|
| Import Designs reports the import context expired | The import context is time-limited (~15 minutes). The app re-checks your selections automatically and retries; if it cannot, rescan and continue again. |
| You skipped hoops and now want auto hoop matching | Add hoops in **Admin → Hoops**, then run another import or regenerate previews as needed. |
| Visual AI did not run | Visual AI is not part of import. Run it from **Admin → Batch Operations** and check the API key and Visual AI settings in **Admin → Settings**. |
| You cannot find Review Hoops / Tags / Sources / Designers | Those buttons were removed from the import wizard. Use the Admin pages instead — your import selections are preserved while you are away. |

## Recommended First-Run Workflow

1. Add your real machine hoops in **Admin → Hoops**.
2. Check Tags, Sources and Designers in the Admin pages if your defaults are incomplete.
3. Import your designs.
4. Verify the imported designs in Browse and the design detail pages.
5. Optionally run **Batch Operations** to add Visual AI tags.

## Related Guides
- [IMPORT_WORKFLOW.md](IMPORT_WORKFLOW.md)
- [BATCH_OPERATIONS_BACKFILL.md](BATCH_OPERATIONS_BACKFILL.md)
- [GETTING_STARTED.md](GETTING_STARTED.md)
