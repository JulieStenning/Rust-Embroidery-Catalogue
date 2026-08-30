# Importing a Large Design Library

This guide explains what happens when you import a very large collection of
embroidery files - for example 120,000 designs - and the feedback you will see
from the app while the import runs.

The import itself works the same whether you import 50 files or 120,000. The
difference at scale is **time**, **disk space**, and how much the app needs to
do for each individual design. Reading this before you start will help you set
expectations and avoid surprises.

## Before you start

- Put your embroidery files into one or more folders you can access. The app
  reads subfolders automatically, so a deep folder tree is fine.
- **Your original files are never altered, moved, or renamed.** Files that are
  already inside your managed catalogue storage are referenced in place. Files
  outside it are **copied** into managed storage, so a copy of every imported
  file is kept in the catalogue.
- **Existing files are skipped automatically.** If a design is already in the
  catalogue, it won't be imported twice - so importing the same folder again
  later is quick and safe.
- Supported formats: **JEF, PES, HUS, DST, EXP, VP3**. Other file types in
  your folders are ignored.

## Step 1 - Select folder(s)

1. Open **Import** from the top menu.
2. Add one or more source folder paths (use **Browse...** for the native
   folder picker, or type the path directly).
3. Click **Scan folder(s)**.

While scanning, the button shows **Running...**. The app walks every selected
folder and its subfolders, finds all supported embroidery files, and checks
which ones are already in the catalogue so they can be skipped.

If nothing is found, the app tells you exactly why:

- A path is missing or wrong:
  *"The selected folder(s) could not be found on disk. Check that the path is
  correct and the drive is available."*
- No supported embroidery files exist in the chosen folders:
  *"No supported embroidery files (JEF, PES, HUS, DST, EXP, VP3) were found in
  the selected folder(s)."*

## Step 2 - Review scanned files

The review screen groups results by source folder and shows the filename, file
size, and per-folder **Designer** and **Source** controls. You can:

- select or deselect individual files,
- use **Select all / Clear all**,
- keep the app's inferred Designer/Source per folder, or choose your own,
- apply one Designer/Source override to all folders in a multi-folder import.

The heading tells you the totals, e.g.:

- *"1,200 folder(s) scanned - 128,450 file(s) found. Selected files will be
  copied into the catalogue."*

Then click **Continue** to move to the pre-import step.

## Step 3 - Before You Import

This step shows a reminder to review your **hoops, tags, sources, or
designers** before importing. On the very first import into an empty catalogue,
hoops get special attention - if none are configured you'll be asked to confirm
before continuing without them.

An **AI tagging banner** tells you what will happen to tags during the import:

- If no Google API key is saved, the import uses **File & Folder Rules only**,
  which runs locally and never calls the internet. Every imported file gets
  tags matched from its filename and folder path.
- If a key is saved, the banner shows whether **Visual AI** auto-runs during the
  import, plus a cost and rate-limit note.

> **Very important for large libraries:** Gemini's free tier is roughly
> **15 requests per minute** and **1,500 requests per day**. For 120,000
> designs this means Visual AI tagging will not run on everything in one session.
> File & Folder Rules (local path/name matching) run on **every** file, but
> AI-assisted tagging is rate-limited and will not finish a 120,000-design
> library in one import. Plan to add an API key and run Visual AI in smaller
> passes afterwards if you want it, rather than relying on it during a huge
> import.


## Feedback you will see during the import

### The live progress line

While the import runs, the **Import Designs** button shows a running status,
for example:

- *"Running Import... Starting import for 120000 files..."*
- *"Running Import... Processing 45678/120000: Sunset_Floral.pes"*
- *"Running Import... 45680/120000 processed (45680 imported) - saving batch..."*
- *"Running Import... Completed 120000/120000 processed (120000 imported)"*

### Stop / cancel

- Before the import starts, the right-hand button reads **Cancel** and abandons
  the import entirely.
- Once the import is running, it changes to **Stop** (then **Stopping...**).
  The app stops at a safe point - every design already saved stays saved. The
  line then shows something like:
  *"Stopped after 72340/120000 processed (72340 imported)"*.
  When you import again, the files already in the catalogue are skipped, so you
  continue from where you left off.

### Error messages

The app keeps going if individual files fail. If a single file is unreadable or
corrupt, it is skipped and the import continues with the remaining files. If
something more serious happens (for example the selected folders could not be
scanned), you'll see a clear message such as:

- *"Import preview failed: ..."*
- *"Import precheck failed: ..."*
- *"Import action failed: ..."*

### If you leave the import wizard open too long

The app keeps your import selections in a short-lived session. If you navigate
away (for example to the About or Settings pages) and come back later, or leave
the wizard open for more than about fifteen minutes before clicking **Import
Designs**, the app may need to **re-check your selections** before starting.
You'll see a message such as *"Import context expired. Re-checking your
selections before retrying..."* - the app re-scans and re-selects the files you
chose, so you do not have to redo the whole wizard.


## When the import finishes

On success, the app:

1. clears the import wizard and returns you to **Browse Designs**,
2. shows the number of designs that were imported,
3. your new designs (and their preview images) are immediately browsable,
   searchable, and taggable.

## Notes for very large libraries

- **Import in one session if you can.** It is safe to stop and resume (finished
  files are skipped next time), but one continuous run avoids repeating work.
- **Tier 1 tags are free and local.** Every imported design gets keyword and
  stitching tags even with no API key.
- **AI tagging is optional and rate-limited.** Do not depend on it to tag a
  120,000-design library during import. Set it up in Settings and run smaller
  AI tagging passes afterwards.
- **Watch disk space.** The catalogue keeps a copy of every imported design.
- **Check the numbers.** Use the live progress line and the completion message
  to confirm the totals match what you expect.

## Related guides

- Import workflow (concise version): [IMPORT_WORKFLOW.md](IMPORT_WORKFLOW.md)
- First import action details: [FIRST_IMPORT_ACTIONS.md](FIRST_IMPORT_ACTIONS.md)
- Per-folder Designer/Source assignment: [IMPORT_FOLDER_ASSIGNMENT.md](IMPORT_FOLDER_ASSIGNMENT.md)
- AI tagging settings and behaviour: [AI_TAGGING.md](AI_TAGGING.md)
- Getting started quick setup: [GETTING_STARTED.md](GETTING_STARTED.md)
- Supported formats list: [SUPPORTED_FORMATS.md](SUPPORTED_FORMATS.md)
