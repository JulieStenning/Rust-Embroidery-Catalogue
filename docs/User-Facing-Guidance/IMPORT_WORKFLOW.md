# Import Workflow

This guide explains the full import workflow from selecting folders to seeing your designs in the catalogue.

## Before you start
- Put your embroidery files into one or more folders you can access.
- The app will copy imported files into managed catalogue storage.
- Existing-file duplicates are skipped automatically.
- Import runs entirely locally using **File & Folder Rules** — no Google API key is needed.

## Step 1 - Select folder(s)
1. Open Import from the top menu.
2. Add one or more source folder paths (type them or use **Browse…**).
3. Click **Scan folder(s)**.

What happens:
- Subfolders are scanned automatically.
- Only supported embroidery formats are picked up (JEF, PES, HUS, DST, EXP, VP3).
- Files already in the catalogue, and non-embroidery files, are ignored.
- Each selected source folder name is preserved inside managed storage.

## Step 2 - Review scanned files
The review screen groups results by source folder and shows, per folder:
- the folder label and its full path,
- a selection counter ("All N selected" / "N of M selected" / "None selected"),
- "Designer for this folder" and "Source for this folder" controls,
- the matched files (large folders start collapsed — use **Show files (N)**).

You can:
- select or deselect files for import, per file or per folder,
- use the global "Select all" / "Deselect all" buttons,
- keep the inferred Designer/Source,
- choose an existing Designer/Source,
- apply a global Designer/Source override to every folder.

Then click **Continue with N designs**.

## Step 3 - Before You Import
Before importing, the app shows a short confirmation step:
- an explanatory note about how import and Visual AI tagging work,
- **Import Designs** and **Cancel**.

AI tagging:
- Imports run File & Folder Rules only (local, no API key) and always apply the free
  keyword and stitching tags.
- Visual AI is run separately afterwards from **Admin → Batch Operations**.

First import into an empty catalogue with no hoops:
- Pressing **Import Designs** shows a warning that hoops are not configured and asks for
  confirmation. Skipping hoop setup does not create hoops for you.
- Add your own machine hoops in **Admin → Hoops** first if hoop matching matters to you.

## Step 4 - Confirm and save
When you run the import:
- selected files are processed,
- previews and metadata are generated where possible,
- designs are written to the catalogue database,
- files are copied into managed catalogue storage.

The button shows live progress while it runs; **Cancel** becomes **Stop** if you need to
halt it. After success, you are redirected to Browse Designs.

## Leaving the import wizard
You can leave the Import page at any point using the top navigation — there is no
"unsaved changes" prompt. The wizard keeps your scan and selections, so returning to
**Import** restores where you left off. The import context itself is time-limited
(~15 minutes); if it expires, the app re-checks your selections and retries.

## Notes about supported formats
- The app supports many machine embroidery formats.
- Wilcom .art files are limited support and may use fallback preview/metadata paths.

For current list and details: [docs/SUPPORTED_FORMATS.md](../SUPPORTED_FORMATS.md)

## Related guides
- First import action details: [FIRST_IMPORT_ACTIONS.md](FIRST_IMPORT_ACTIONS.md)
- Per-folder Designer/Source assignment details: [IMPORT_FOLDER_ASSIGNMENT.md](IMPORT_FOLDER_ASSIGNMENT.md)
- AI tagging settings and behavior: [BATCH_OPERATIONS_BACKFILL.md](BATCH_OPERATIONS_BACKFILL.md)
- Getting started quick setup: [GETTING_STARTED.md](GETTING_STARTED.md)

## Troubleshooting quick tips
- If no files were selected, return to review and choose at least one file.
- If you leave import open a long time and context expires, the app re-checks your
  selections automatically; if that fails, restart from Import.
- If some files fail, the import can still continue for valid files.
