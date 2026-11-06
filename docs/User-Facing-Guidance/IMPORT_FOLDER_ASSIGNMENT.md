# Assigning Designers and Sources During Import

When you import embroidery designs, you can assign who designed them (**Designer**) and where you acquired them (**Source**), either across all folders or folder by folder.

---

## Set a designer or source for all folders

When most of the folders you are importing come from the same designer or shop:

1. In **Step 2: Review Files**, use the global **Apply to all folders** dropdowns at the top of the page.
2. Select an existing **Designer** or **Source**, or click **+ Add new** to create one.
3. Every folder in the review list updates to that value.

---

## Override specific folders

When individual folders belong to different designers or sources:

1. Locate the specific folder row in the review table.
2. Under **Designer for this folder** or **Source for this folder**, choose:
   - **Keep inferred**: automatically detects the designer or source from the folder name if it matches one of your saved names.
   - **Choose existing**: pick from your saved list.
   - **Create new**: enter a new name directly.
   - **Leave blank**: import without a designer or source.

Individual folder settings always take precedence over the global dropdowns.

---

## How automatic folder inference works

When you leave a folder set to **Keep inferred**:
- The app checks the immediate folder name first.
- If no match is found, parent folder names are checked.
- Standard storage folders (like `MachineEmbroideryDesigns`) are ignored to avoid false matches.
- Matching is case-insensitive.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| A folder was assigned the wrong designer | Choose the correct designer from the folder's dropdown before clicking **Continue**. |
| You misspelled a new designer name | You can rename or merge designers at any time in [Admin → Designers](#/admin/manage-data/designers). |
| You want to clear an assigned designer | Select **Leave blank** from the folder's dropdown. |

---

## Related guides

- [Import Workflow](IMPORT_WORKFLOW.md)
- [Importing a Large Library](IMPORTING_A_LARGE_LIBRARY.md)
- [Settings](SETTINGS.md)
