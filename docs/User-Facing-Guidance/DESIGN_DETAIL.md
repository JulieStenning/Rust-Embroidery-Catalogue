# Design Detail

The **Design Detail** page is where you review and update one design at a time.

Open it by clicking any design card from **Browse Designs**.

---

## What this page is for

Use the detail page when you want to:

- inspect a design preview and technical metadata,
- update notes, rating, stitched status, and reference metadata,
- review, verify, and edit assigned tags,
- open the file in your default editor or reveal it in Explorer,
- add or remove the design from projects,
- print a clean summary sheet.

---

## Page layout at a glance

The page has three main areas:

1. Top navigation strip:
- Back to Browse link.
- Previous/Next navigation when you came from a browse list.
- The current desktop view also shows Prev/Next buttons and a position counter when browse history is available.

2. Main detail area:
- Left: preview image and 3D preview generation button.
- Right: filename, file actions, metadata badges, rating, stitched toggle, notes/metadata form, projects.
- The 3D button changes text based on the current preview state.

3. Collapsible Tags editor:
- Grouped tag checkboxes.
- Save button to apply selected tags.
- Saving tags marks the design as verified.

---

## Quick start workflows

### Update notes and metadata

1. Open the design detail page.
2. Edit **Notes**.
3. Choose Designer, Source, and Hoop if needed.
4. Click **Save**.

### Rate and mark as stitched

1. Click stars to set rating (1 to 5).
2. Use **clear** to remove a rating.
3. Click **Mark as Stitched** (or **Mark as Not Stitched**) to toggle status.

### Edit and verify tags

1. Expand the **Tags** section.
2. Tick/untick tag checkboxes.
3. Click **Save Tags**.

Result:
- Saving tags marks the design as verified.

### Generate or refresh a 3D preview

1. In the image panel, click **Generate 3D Preview** (or **Render 3D Preview**).
2. Wait for redirect back to the page.

Result:
- Preview is replaced with a 3D-rendered image.
- Dimensions are refreshed when file bounds are available.

### Add to or remove from project

1. Choose a project from the **Add to Project** selector and click **Add to Project**.
2. Use **Remove** next to any listed project membership to detach it.

### Review tags and verify

1. Expand the **Tags** section.
2. Review the grouped tag lists.
3. Tick or clear the checkboxes.
4. Click **Save tags**.

Result:
- Saving tags also marks the design as verified.

### Print a design summary

1. Click **Print View** from the detail page, or use **Print** from the printable page.
2. A print-friendly summary opens.
3. Print from your browser.

---

## Field and status guide

- **Dimensions**: width x height in mm.
- **Hoop badge**: assigned hoop name.
- **Stitches / Colours / Colour changes**: technical stitch metadata.
- **Verified / Verify**: review status for tags/metadata.
- **Rating stars**: personal priority/reference rating.
- **Stitched**: whether you have sewn the design.
- **Verified / Verify**: whether the tag set has been reviewed.

---

## Open in Editor / Show in Explorer

- **Open in Editor** opens the design file with your system default app for that file type.
- **Show in Explorer** opens Windows Explorer and selects the file when possible.
- If the exact file is missing, Explorer opens the nearest existing folder instead.

Notes:
- In some test/locked-down environments, external launches are intentionally suppressed.
- The page still returns to detail view after the action.

---

## Troubleshooting

| Problem | What to do |
|---|---|
| Preview is missing | Click **Generate 3D Preview**. If it still fails, confirm the source file still exists. |
| Open in Editor does nothing | Check file association on Windows for that embroidery format and try again. |
| Show in Explorer opens folder but not selected file | The file may no longer exist at the stored path; verify managed storage location and file presence. |
| Tags did not look updated | Re-open Tags and confirm checkboxes were saved; save action should mark as verified. |
| Rating not accepted | Ratings are limited to 1 to 5 (or clear). |
| Project add/remove did not apply | Confirm the target project exists and refresh the detail page. |

For broader app issues, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

---

## Related guides

- [IMPORT_WORKFLOW.md](IMPORT_WORKFLOW.md)
- [TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md)
- [COLOUR_COUNTS.md](COLOUR_COUNTS.md)
- [BACKUP_RESTORE.md](BACKUP_RESTORE.md)
- [GETTING_STARTED.md](GETTING_STARTED.md)