# First Import Actions Guide

When you import designs for the first time, the app now pauses before import so you can review setup items that affect automatic classification and hoop matching.

## What First Import Actions Means
First import actions are the optional or required checks shown after file review and before import runs:
- Hoops
- Tags
- Sources
- Designers

This is one pre-import step. Tag review is part of it, but not the whole feature.

## Where It Appears
Import wizard flow:
1. Select folders.
2. Review scanned files.
3. First import actions page appears.
4. Review any setup lists you want.
5. Continue with import, or cancel.

## First Import vs Later Imports

### First import
- You are prompted to review hoops first.
- If no hoops are set up yet, the app warns you.
- If you still choose import now, the app asks for extra confirmation.

### Later imports
- You can still review hoops, tags, sources, or designers before import.
- You can also skip review and import now.

## What To Do on the First Import Actions Page
You can choose any of these actions:
- Review Hoops
- Review Tags
- Review Sources
- Review Designers
- Import now
- Cancel

Tip: for brand new catalogues, start with Review Hoops, then quickly check Tags, Sources, and Designers.

## Using Review Pages in Import Mode
When you open any review page from first import actions, you are in import mode.

In import mode you can:
- Add, edit, or delete entries as normal.
- Jump between review pages using quick links.
- Continue import from the top or bottom continue button.
- Cancel and return to the import start page.

## Continue and Cancel Behavior
- Yes, continue with import: runs import with your current selections.
- Cancel: exits to the import start page without importing.

If you navigated through review pages, continue uses your saved pending file selection from the current import session.

## AI and Image Preference Notes
The first import actions page also shows:
- AI tagging status (key configured or not configured).
- Tier 2 and Tier 3 auto-tagging status.
- Session image preference choice (2D or 3D previews).

These settings affect how import runs when you continue.

## Troubleshooting

| Issue | What to do |
|---|---|
| Continue button sends you back to import start | Your import session token is invalid or expired. Restart from scan and run first import actions again. |
| You do not see import mode banner on review page | Open that review page from the first import actions step, not from admin navigation. |
| You skipped hoops and now want auto hoop matching | Add hoops in Admin -> Hoops, then run another import (or reprocess affected designs as needed). |
| AI tiers did not run | Check API key and tier settings in Admin -> Settings. |

## Recommended First-Run Workflow
1. Review Hoops and add your real machine hoops.
2. Review Tags and adjust naming/groups to your catalogue style.
3. Review Sources and Designers if your defaults are incomplete.
4. Continue with import.
5. Verify imported designs in Designs list and detail pages.

## Related Guides
- [TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md)
- [AI_TAGGING.md](AI_TAGGING.md)
- [GETTING_STARTED.md](GETTING_STARTED.md)
