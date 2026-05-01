# First Import Tag Management Plan

## Goal

Before the **first bulk import**, the app should require the user to review the tag list and explain why this matters. For **subsequent imports**, the app should ask whether the user wants to review the tags before importing. If they choose to review them, show the existing tag management form, and when they finish, ask whether they are ready to continue with the import or cancel.

---

## Why this change is needed

The initial database includes a starter set of tags, but those defaults may not suit every user’s collection or way of organizing designs.

Because the bulk import process uses the tag list for automatic classification, users need a chance to check and adjust those tags **before their first import** so the catalogue starts off with more accurate tagging.

Suggested user-facing explanation:

> A starter set of tags is included with the catalogue, but it may not match the way you organise your own designs. The import process uses these tags to help classify designs automatically, so it is a good idea to review them before your first bulk import.

---

## Required behaviour

### 1. First-ever bulk import
When the catalogue has **no designs yet**:

- after the user selects files to import, **do not import immediately**
- show a message explaining why tag review is important before the first import
- present the user with the existing tag form/page so they can review and edit tags if needed
- when they are done, ask:
  - **Yes** → continue with the import
  - **Cancel** → stop and return without importing

### 2. Subsequent bulk imports
When the catalogue already contains one or more designs:

- after the user selects files to import, ask:
  - **Do you want to check the tags before importing?**
- options:
  - **Yes** → open the existing tag form/page
  - **No** → continue directly with the import
  - **Cancel** → stop and return without importing
- if they chose to review tags, then after editing, ask:
  - **Are you ready to do the import?**
  - **Yes** → continue with the import
  - **Cancel** → stop without importing

---

## Recommended implementation approach

Reuse the current `Manage Tags` page rather than building a second tag editor.

### Flow change
Current flow:
1. select folder
2. scan and review files
3. import confirmed files

Proposed flow:
1. select folder
2. scan and review files
3. **pre-import tag decision step**
4. optional or required tag review
5. final **Yes / Cancel** confirmation
6. import confirmed files

---

## Suggested technical design

### A. Detect whether this is the first import
Use the design count in the database to determine whether the catalogue is still empty.

Suggested check:
- `db.query(Design).count() == 0`

If true, this is the first import and tag review should be mandatory.

---

### B. Add a new pre-import decision route/page
In `src/routes/bulk_import.py`:

- add a lightweight intermediate route such as `/import/precheck`
- this route receives:
  - `folder_path`
  - `selected_files`
- it determines whether this is:
  - a **first import**
  - or a **subsequent import**

Render a new template such as:
- `templates/import/step3_precheck.html`

That page should:
- explain the purpose of tag review for the first import
- ask whether the user wants to review tags for later imports
- preserve the selected file list so the workflow can continue cleanly

---

### C. Reuse the existing tag management page in import mode
Instead of creating a new tag editor:

- reuse `GET /admin/tags/`
- reuse `templates/admin/tags.html`

Add an import-aware mode so that when the page is opened from the bulk import flow it can:
- show an explanatory banner
- preserve pending import data (`folder_path`, `selected_files`)
- show buttons at the bottom:
  - **Yes** / **Continue with Import**
  - **Cancel**

The standard admin tags page should still work normally outside the import workflow.

---

### D. Final confirmation after tag editing
When the tag page is opened from import mode and the user finishes reviewing tags:

show a clear final prompt such as:

> Are you ready to do the import?

Buttons:
- **Yes** → post back into `/import/confirm`
- **Cancel** → abort and return to `/import/`

This final confirmation is required after tag review, both for:
- the first import
- later imports when the user chooses to review tags

---

## Files likely to change

### Backend
- `src/routes/bulk_import.py`
  - add first-import detection
  - add the precheck step
  - hand off into tag review or straight to import

- `src/routes/tags.py`
  - support an import-review context
  - preserve workflow state after creating/editing/deleting tags

### Templates
- `templates/import/step2_review.html`
  - send selected files to the new precheck step instead of directly importing

- `templates/import/step3_precheck.html` *(new)*
  - show first-import explanation or later-import question

- `templates/admin/tags.html`
  - support import mode banner and final `Yes` / `Cancel` actions

### Tests
- `tests/test_routes.py`
  - first import forces tag review
  - later imports offer optional tag review
  - `Yes` continues to import
  - `Cancel` aborts cleanly
  - normal `/admin/tags/` behaviour remains intact

---

## Behaviour summary for the web agent

### First import
- mandatory tag review
- explain why
- then ask **Yes** or **Cancel** before importing

### Later imports
- ask whether they want to review tags first
- if **Yes**, show tag form and then ask **Yes** or **Cancel** to proceed
- if **No**, continue directly to import
- if **Cancel**, stop

---

## Acceptance criteria

- A brand-new installation with no designs cannot go straight into the first import without being shown the tag review explanation.
- The user can review and edit tags before the first import.
- After tag review, the user is asked whether they are ready to import, with **Yes** and **Cancel** options.
- On later imports, the user is asked whether they want to check tags before importing.
- Existing standalone tag management still works outside the import flow.
- The selected files/folder path are preserved throughout the workflow.
