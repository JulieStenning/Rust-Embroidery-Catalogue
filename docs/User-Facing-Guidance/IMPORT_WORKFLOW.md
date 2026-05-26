# Import Workflow

This guide explains the full import workflow from selecting folders to seeing your designs in the catalogue.

## Before you start
- Put your embroidery files into one or more folders you can access.
- The app will copy imported files into managed catalogue storage.
- Existing-file duplicates are skipped automatically.

## Step 1 - Select folder(s)
1. Open Import from the top menu.
2. Add one or more source folder paths.
3. Use Browse if you prefer the native folder picker.
4. Click Scan Folder(s).

What happens:
- Subfolders are scanned automatically.
- Each selected source folder name is preserved inside managed storage.

## Step 2 - Review scanned files
The review screen groups results by source folder and shows:
- filename,
- size where available,
- hoop suggestion where available,
- per-file status,
- per-folder Designer and Source controls.

You can:
- select or deselect files for import,
- keep inferred Designer/Source,
- choose existing Designer/Source,
- create new Designer/Source during import,
- set Designer/Source to blank deliberately,
- apply global overrides in multi-folder imports.

Then click Continue.

## Step 3 - Choose pre-import actions
Before importing, the app shows a decision step.

First import into an empty catalogue:
- You are guided to check setup actions before import.
- Hoops are emphasized because hoop matching depends on your own machine frames.
- You can review hoops, tags, sources, and designers.

Later imports:
- Review actions are optional.
- You can review reference data or import immediately.

AI tagging banner:
- If no API key is configured, import runs Tier 1 keyword tagging only.
- If API key is configured, the page shows your Tier 2 and Tier 3 auto-run settings.

## Step 4 - Confirm and save
When you continue with import:
- selected files are processed,
- previews and metadata are generated where possible,
- designs are written to the catalogue database,
- files are copied into managed catalogue storage.

After success, you are redirected to Browse Designs.

## Import mode on admin pages
If you open tags, hoops, sources, or designers from Step 3:
- the app keeps import context active,
- save actions keep you in import mode,
- Continue with import returns you to the confirm path.

## Notes about supported formats
- The app supports many machine embroidery formats.
- Wilcom .art files are limited support and may use fallback preview/metadata paths.

For current list and details: [docs/SUPPORTED_FORMATS.md](../SUPPORTED_FORMATS.md)

## Related guides
- First import action details: [FIRST_IMPORT_ACTIONS.md](FIRST_IMPORT_ACTIONS.md)
- Per-folder Designer/Source assignment details: [IMPORT_FOLDER_ASSIGNMENT.md](IMPORT_FOLDER_ASSIGNMENT.md)
- AI tagging settings and behavior: [AI_TAGGING.md](AI_TAGGING.md)
- Getting started quick setup: [GETTING_STARTED.md](GETTING_STARTED.md)

## Troubleshooting quick tips
- If no files were selected, return to review and choose at least one file.
- If you leave import open a long time and context expires, restart from Import.
- If some files fail, the import can still continue for valid files.