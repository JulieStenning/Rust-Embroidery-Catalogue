## Plan: Shared Backfilling and Assignment Utility

**Purpose:**  
Unify the logic for assigning stitch types, image, and color/thread count to designs, so both the import process and tagging actions use a single, flexible utility. Ensure batch and commit settings are respected and sourced from user input, settings, or defaults.

---

### Steps

1. **Centralize Batch/Commit Retrieval**
   - Create a helper function to get batch size and commit frequency:
     - Priority: user input (tagging actions UI) → settings page → defaults from settings table.
     - If a value is blank, fall back to the next source.

2. **Design the Shared Utility Function**
   - Inputs: `design`, `desc_to_tag`, `db`, boolean flags for each assignment (`do_stitch_types`, `do_image`, `do_color_counts`), `dry_run`, `batch_size`, `commit_frequency`.
   - For each assignment:
     - If the flag is True (or always True for import), perform the extraction and update the design.
     - Use existing helpers: `suggest_stitching_from_pattern`, `_render_preview`, color/thread count extraction logic.
     - Handle errors and log appropriately.
   - Use `batch_size` to limit records fetched per batch.
   - Commit after every `commit_frequency` processed records.

3. **Refactor Import Process**
   - Replace direct calls to assignment logic with a call to the shared utility with all flags set to True.
   - Fetch batch/commit values from the settings table (unless overridden for testing).

4. **Refactor Tagging Actions**
   - Pass user-specified batch/commit values from the UI if provided, else fetch from settings table.
   - Pass flags based on user selection (from the UI or API payload) to the shared utility.

5. **Testing & Verification**
   - Add/expand logging to confirm which assignments are performed and which batch/commit values are used.
   - Test both flows for correct, selective updates.

---

**Decisions:**
- Batch/commit value retrieval is handled by a helper function for reusability.
- The shared utility applies those values to SQL queries and commit logic.
- The utility always does all assignments for import, and only selected ones for tagging actions.

---

**Further Considerations:**
- Ensure error handling and logging are robust for all assignment types.
- Consider future extensibility for additional assignment types.

---
