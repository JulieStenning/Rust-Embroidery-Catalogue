# Import Designer and Source Per Folder

Use this guide during bulk import when different folders need different Designer or Source values.

---

## Quick Steps

1. Open Import (`/import/`) and add one or more source folders.
2. Click Scan.
3. In the review step, check each folder group.
4. For each folder, choose Designer and Source:
   - Keep inferred
   - Choose existing
   - Create new
   - Leave blank
5. If needed, set one global Designer/Source at the top to apply broadly.
6. Keep or uncheck files using the file checkboxes.
7. Click Continue, then complete the precheck step.
8. Confirm import.

---

## How Assignment Works

When a design is imported, assignment is applied in this order:

1. Per-folder choice
2. Global choice
3. Inferred from folder/file path
4. Blank

This means explicit folder choices always win.

### Automatic Path Inference Hierarchy

When choosing **Keep inferred**, the application infers Designer and Source by examining folder path segments:
- **Leaf-First Traversal:** The folder directly containing your design files is evaluated first. Parent directories are only checked if the immediate folder does not match any registered Designer or Source.
- **Library Root Filtering:** Storage directories such as `MachineEmbroideryDesigns` and `EmbroideryCatalogue` are ignored so standard library paths never trigger false matches.
- **Specific Matching:** Exact folder names and registered names with web affixes (such as `.com`) match their specific folder without falsely claiming different designers in sub-folders.

---

## Create New During Import

If you choose Create new for Designer or Source:

1. Enter the new name.
2. The app checks existing values case-insensitively.
3. If a matching value already exists, it is reused.
4. If not, a new one is created and assigned.

---

## Multi-Folder Tips

1. Use global values when most folders should match.
2. Override only the folders that differ.
3. Keep inferred when folder names already match your saved Designer/Source names.
4. Use Leave blank when you intentionally do not want a Designer or Source.

---

For broader import flow information, see [GETTING_STARTED.md](GETTING_STARTED.md).
For troubleshooting, see [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md).