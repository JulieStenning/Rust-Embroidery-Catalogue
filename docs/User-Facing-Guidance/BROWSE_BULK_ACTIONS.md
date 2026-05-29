# Browse Bulk Actions

The Browse page includes a bottom action banner that appears when you select one or more designs.
Use this banner to verify designs, replace tags in bulk, add selected designs to a project, or clear the current selection.

---

## What this feature is for

Use Browse bulk actions when you want to:

- verify multiple designs at once,
- apply the same tag set to many designs,
- add several designs to a project in one step,
- clear selection quickly and continue browsing.

---

## Where to find it

1. Open Browse (`/designs/`).
2. Tick one or more design checkboxes.
3. The bottom banner appears with:
   - selected count,
   - **Choose tags...**,
   - **Verify selected**,
   - project selector,
   - **Add to project**,
   - **Clear selection**.

---

## Action guide

### Choose tags...

- Opens an in-page modal.
- Lets you tick tags to apply to all currently selected designs.
- Applying tags replaces existing tags on those selected designs.
- Applying tags marks selected designs as verified.

Use this when you want to standardize tag sets across many designs.

### Verify selected

- Marks selected designs as verified.
- Does not change their current tags.

Use this after checking that existing tags are already correct.

### Add to project

- Pick a project from the dropdown.
- Click **Add to project** to add all selected designs.

If no projects exist yet, create one first in Projects.

### Clear selection

- Unchecks current card selections on the page.
- Hides the bottom banner.

Use this before starting a different bulk action set.

---

## Selection behavior to know

- Selection is page-based.
- **Select all** affects visible results on the current page only.
- Moving to another page clears checkbox selection.
- After a bulk action submits, Browse reloads and the previous selection is not retained.

---

## Recommended workflows

### Quick verify pass

1. Filter Browse to a narrow set.
2. Select relevant designs.
3. Click **Verify selected**.
4. Continue with the next filtered set.

### Bulk retagging pass

1. Filter Browse to target designs.
2. Select designs.
3. Click **Choose tags...**.
4. Tick the final replacement tag set.
5. Click **Apply tags**.

### Project staging pass

1. Select designs for a sewing session.
2. Choose the target project from the dropdown.
3. Click **Add to project**.
4. Optionally clear selection and continue.

---

## Troubleshooting

| Problem | What to do |
|---|---|
| Bottom banner does not appear | Confirm at least one card checkbox is selected on the current page. |
| Add to project does nothing | Choose a project in the dropdown first. |
| No projects available | Create a project on `/projects/`, then return to Browse. |
| Selection disappeared after moving pages | This is expected; selection does not persist across pagination. |
| Tags changed unexpectedly after Choose tags | The bulk tag action replaces tags for selected designs; re-open the modal and apply the intended set. |

For broader issues, see [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md).

---

## Related guides

- [DESIGN_DETAIL.md](DESIGN_DETAIL.md)
- [HELP.md](HELP.md)
- [AI_TAGGING.md](AI_TAGGING.md)
- [TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md)