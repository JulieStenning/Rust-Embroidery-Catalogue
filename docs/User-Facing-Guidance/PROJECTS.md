# Projects

Projects let you group designs for a planned embroidery task and print a practical reference sheet.

Open Projects from the top navigation bar or visit `/projects/`.

---

## What this feature is for

Use Projects when you want to:

- collect designs for one sewing session,
- keep a short plan or notes in the project description,
- add designs one-by-one or in bulk,
- remove designs from a project without deleting the design records,
- print a single project sheet for planning at your machine.

---

## Pages at a glance

### Project List (`/projects/`)

- Shows all projects in a card grid.
- Each card shows project name, optional description, and created date.
- Use **+ New Project** to create a project.

### New Project (`/projects/new`)

- Enter a required **Name**.
- Optionally add a **Description**.
- Click **Create Project**.

### Project Detail (`/projects/{id}`)

- Edit project name/description inline and click **Save**.
- View all designs currently in the project.
- Remove any design from the project with **Remove**.
- Open **Print Sheet** for a print-friendly summary.
- Delete the project with **Delete Project** (designs are not deleted).

### Print Sheet (`/projects/{id}/print`)

- Printable summary of project designs.
- Includes preview image (or fallback) and key metadata where available.

---

## Quick workflows

### Create a project

1. Go to **Projects**.
2. Click **+ New Project**.
3. Enter a project name.
4. Optionally add a description.
5. Click **Create Project**.

### Add one design from Design Detail

1. Open a design.
2. In the **Projects** section, choose a project.
3. Click **Add to Project**.

### Add several designs from Browse

1. Go to **Browse**.
2. Select multiple designs.
3. In the bottom action bar, choose a project.
4. Click **Add to project**.

### Remove a design from a project

1. Open the project detail page.
2. Find the design card.
3. Click **Remove**.

### Edit project details

1. Open project detail.
2. Change name and/or description.
3. Click **Save**.

### Delete a project

1. Open project detail.
2. Click **Delete Project**.
3. Confirm in the browser prompt.

Result:
- The project is removed.
- Design records remain in your catalogue.

### Print a project sheet

1. Open project detail.
2. Click **Print Sheet**.
3. Print from your browser.

---

## What appears on the print sheet

Per design, the print sheet may include:

- preview image,
- filename,
- size (mm),
- hoop,
- stitch count,
- colour count,
- colour changes,
- designer,
- rating,
- stitched status,
- notes.

Only fields with available data are shown.

---

## Troubleshooting

| Problem | What to do |
|---|---|
| Add to project fails from Browse | Confirm at least one design is selected and a project is chosen. |
| Add to project fails from Design Detail | Confirm the target project still exists, then retry. |
| Project detail shows no designs | Add designs from Design Detail or Browse bulk actions. |
| Print sheet shows "No image" for some designs | Generate/refresh previews for those designs and print again. |
| Deleted a project by mistake | Recreate the project and add designs again; deleting a project does not delete designs. |

For broader issues, see [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md).

---

## Related guides

- [DESIGN_DETAIL.md](DESIGN_DETAIL.md)
- [BROWSE_BULK_ACTIONS.md](BROWSE_BULK_ACTIONS.md)
- [TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md)
- [BACKUP_RESTORE.md](BACKUP_RESTORE.md)
- [GETTING_STARTED.md](GETTING_STARTED.md)