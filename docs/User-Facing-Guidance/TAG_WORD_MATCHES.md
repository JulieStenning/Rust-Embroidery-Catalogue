# Tag Word Matches

This guide explains how **Tag Word Matches** work in the Embroidery Catalogue and how you can manage them to automate tagging when importing or backfilling designs.

---

## Overview

**Tag Word Matches** (synonyms and keywords) allow you to map words that appear in filenames and folder paths directly to specific catalogue tags.

For example:
- A file named `Cute_Puppy_4x4.pes` or in a folder called `Dogs` can automatically receive the **Animals** tag.
- A file containing `ITH_Zipper_Pouch.jef` can automatically receive the **In The Hoop** tag.
- A file containing `FSL_Snowflake.pes` can automatically receive the **Lace** tag.

---

## How Matching Works

Tag Word Matching is **100% local and offline** — no internet connection or Google API key is required.

### 1. File & Folder Rules (Import & Batch Operations)
When designs are imported (see [Import workflow and local rules](IMPORT_WORKFLOW.md)) or when you run **Step 1 (File & Folder Rules)** in **Admin → Batch Operations** (see [Running local batch tagging](BATCH_OPERATIONS_BACKFILL.md)):
- The catalogue scans every folder segment and the design filename.
- It tests against **exact tag names** as well as all configured **Tag Word Matches**.

### 2. Whole Word Boundaries
Matching is performed on whole word boundaries. For example:
- A match word `cat` will match `Black_Cat.pes` or a folder named `Cats`.
- It will **not** falsely match `Catch_The_Ball.pes` or `Catalogue`.

### 3. Plurals and Singulars
The matching engine automatically handles regular English plurals and singular forms:
- A match rule for `frog` will match both `frog` and `frogs`.
- A match rule for `puppies` will match both `puppy` and `puppies`.

### 4. Case-Insensitive Matching
Matching is case-insensitive. Entering `Rose` or `rose` matches `ROSE.PES`, `Rose_Border.jef`, or `red_roses.dst`. All match words are automatically normalised to lowercase when saved.

---

## Managing Word Matches

You can view, search, add, and remove word matches in two convenient places.

### 1. Dedicated Admin Page (`Manage Data → Word Matches`)
Navigate to **Admin → Manage Data → [Word Matches](#/admin/data/tag-matches)**.

Here you will find:
- **Search & Group Filter:** Filter tags by name or switch between **All Tags**, **Image Tags**, and **Stitching Tags**.
- **Grouped Tag Cards:** Every tag with matches is displayed as a clean card showing all assigned match words as removable chips.
- **Inline Add:** Add additional words directly on any tag's card.
- **Top Quick Add Bar:**
  - Search or select a target tag from the dropdown.
  - As soon as you select a tag, its **existing matches are displayed immediately below** the selector. This helps inspire new keyword ideas and prevents adding words that already exist.
  - If you type a word that is already assigned to the tag, an instant **"Already added"** duplicate warning appears.
  - Enter single words or multiple comma-separated words at once (e.g. `kitten, puppy, bunny, hedgehog`) and click **Add Matches** (or press <kbd>Enter</kbd>).

### 2. Contextual Tag Administration (`Manage Data → Tags`)
When maintaining your general tag list in **Admin → Manage Data → [Tags](#/admin/data/tags)**:
- Every tag row has a dedicated **Matches** button. Clicking it opens a window where you can view, add, or remove word matches for that specific tag.
- When creating a brand-new tag, you can optionally click to assign word matches immediately upon saving the tag.

---

## Starter Data & Customization

The catalogue comes pre-loaded with standard starter word matches for common embroidery themes (including animal species, floral types, holidays, and stitching techniques like In The Hoop and Free Standing Lace).

- **Fully Editable:** There are no system-locked word matches. You can delete any starter match that does not fit your workflow or add custom keywords that match your personal folder conventions.
- **Safe & Reversible:** Modifying word matches does not alter any of your physical embroidery files.

---

## Related Guides

- [Import workflow and local rules](IMPORT_WORKFLOW.md)
- [First import actions & review](FIRST_IMPORT_ACTIONS.md)
- [Assigning Designer & Source per folder](IMPORT_FOLDER_ASSIGNMENT.md)
- [Running local batch tagging](BATCH_OPERATIONS_BACKFILL.md)
- [Data storage and library paths](DATA_STORAGE_GUIDE.md)
- [Getting started and initial setup](GETTING_STARTED.md)
