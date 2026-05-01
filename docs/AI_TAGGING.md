# AI-Assisted Auto-Tagging

The Embroidery Catalogue includes an optional feature that uses **Google Gemini AI** to
suggest design-type tags for your embroidery files.  This is entirely optional — the
application works fully without it.

---

## Overview

Auto-tagging works in up to three tiers, tried in order for each design:

| Tier | Method | Requires API key? |
|------|--------|------------------|
| 1 | Keyword matching against the filename | No |
| 2 | Gemini text AI — sends filename stems to the API | Yes |
| 3 | Gemini vision AI — sends the design preview image to the API | Yes |

Tier 1 is always free and instant.  Tiers 2 and 3 call the Google Gemini API and
consume quota from your account.

After tagging, each design is marked *"Tags not verified"*.  You can
verify multiple tags on the Browse page by choosing **Verify Selected** 
or you can open the design detail page and tick **Verify** once you are 
happy with the suggestions.

---

## Step 1 — Get a Google API key

1. Go to <https://aistudio.google.com/> and sign in with a Google account.
2. Click **Get API key** (top-left) → **Create API key**.
3. Copy the key — it looks like `AIzaSy…`.

> **Free tier:**  The Gemini API offers a free tier that allows up to 15 requests per
> minute and 1,500 requests per day.  This is sufficient for tagging a few hundred
> designs.  Larger collections may need a paid plan.

---

## Step 2 — Add the key in Admin Settings

Open **Admin → Settings** in the app and paste the key into the **Google Gemini API key**
field, then click **Save settings**.

Alternatively, you can add the key manually to a file named **`.env`** in the project root
(the same folder as `start.bat`):

```
GOOGLE_API_KEY=AIzaSy_your_actual_key_here
```

> **Security note:**  The `.env` file contains a private secret.  Do **not** commit it
> to Git (it is already listed in `.gitignore`).

---

## Step 3 — Enable Tier 2 / Tier 3 in Settings

Open **Admin → Settings** and tick the options you want:

- **Run Tier 2 automatically during import** — sends the cleaned filename to Gemini
  to suggest tags each time you import new designs.
- **Run Tier 3 automatically during import** — sends the preview image to Gemini Vision
  to suggest tags for designs that are still untagged after Tiers 1 and 2.

If neither option is ticked, imports run Tier 1 keyword tagging only — no Gemini calls
are made even if a key is present.

### Batch size settings (optional)

For very large imports you can set an **AI tagging batch size** in Settings.  This limits
how many newly imported designs are sent to Gemini in a single import run, so you can
spread the API calls across multiple runs.  Leave the field blank to tag all new designs
in one go.

You can also set an **Import database commit batch size** in Settings.  This controls how
many designs are written or tag-updated before each database commit during import.
Leave it blank to use the default (1000).

---

## Import flow and warnings

When you start an import, the **Before You Import** screen shows:

- A blue notice if no API key is configured — import will use Tier 1 only.
- An amber notice if an API key is present — a cost/quota warning is shown, along with
  your current Tier 2 / Tier 3 settings.  Click the **Change in Settings** link if you
  want to adjust them before continuing.

---

## In-app tagging actions

Beyond the import wizard, the catalogue provides a **Tagging Actions** page (accessible from
**Admin → Tagging Actions** in the navigation bar) that lets you run AI tagging on your
existing designs without touching the command line.

### Available actions

| Action | What it does |
|--------|-------------|
| **Tag only untagged designs** | Processes designs with no tags at all.  Verified tags are never touched.  Safe to run at any time. |
| **Tag untagged and unverified designs** | Processes both untagged designs and designs whose tags have not yet been verified.  Manually verified designs are left untouched. |
| **Re-tag ALL designs** | Overwrites existing tags on every design, including ones you have manually verified.  Requires an explicit confirmation checkbox. |
| **Local stitching backfill** | Analyses the actual embroidery pattern geometry (via StitchIdentifier) to detect stitch types. Updates only the `stitching` tag group for existing unverified designs. No Gemini/API calls are made. |

### Options per action

- **Tiers** — choose which tiers to run.  Tier 1 is always included.  Tiers 2 and 3 are
  only available when an API key is configured.
- **Batch size** — limit to this many designs per run (defaults to the value in Settings).
  Useful for spreading large libraries across multiple sessions.
- **Delay** — seconds between Gemini API calls (defaults to the value in Settings).
  Increase this if you see *429 Too Many Requests* errors.

### Warnings and tips

- A cost/quota warning is displayed whenever an API key is configured, consistent with the
  import flow.
- A blue notice is shown when no API key is present, with links to Settings and the AI
  Tagging Guide.
- The **Re-tag ALL designs** action shows an extra confirmation checkbox because it will
  overwrite verified tags.- **Local stitching backfill** is fully offline and intended for the new stitching-only workflow.
  It preserves image tags and skips verified designs.- Actions run synchronously — the page reloads when complete and shows a summary.  For very
  large libraries, use the CLI scripts (below) for overnight runs.

---

## Standalone scripts (CLI)

The CLI tools remain available for batch runs outside the import wizard:

```bat
.venv\Scripts\activate
python auto_tag.py
```

### Common options

```bat
REM Tag only designs with no tags yet (default)
python auto_tag.py

REM Re-tag ALL designs, overwriting existing tags
python auto_tag.py --redo

REM Tag untagged and unverified designs, skip verified
python auto_tag.py --skip-verified

REM Keyword matching only — no API calls
python auto_tag.py --tier1-only

REM After Tier 1+2, also run vision AI on still-untagged designs
python auto_tag.py --tier3

REM Vision AI only — skip Tier 1 and 2
python auto_tag.py --tier3-only

REM Test vision AI on just 5 designs
python auto_tag.py --tier3-only --limit 5

REM Process at most 50 designs across all tiers (for testing)
python auto_tag.py --limit 50

REM Preview what would happen without writing anything
python auto_tag.py --dry-run

REM Slow down if you are hitting rate limits (default: 5 seconds between batches)
python auto_tag.py --delay 6.0
```

---

## Using AI tagging on the portable / USB copy

When the application is deployed to a USB stick or SD card, the `.env` file is copied
to the device automatically by `prepare_portable_target.bat`.

On the target machine, activate the portable virtual environment and run:

```bat
cd F:\EmbroideryApp\app
venv\Scripts\activate
python auto_tag.py
```

(Replace `F:\` with the actual drive letter of your device.)

Alternatively, add the API key via **Admin → Settings** on the target machine and use
the in-app import flow to trigger AI tagging automatically.

---

## Cost estimate

| Scenario | Approximate cost |
|---|---|
| Tier 1 only (keywords) | Free |
| Tier 2 text AI on 10,000 designs | Free tier (may take multiple sessions) |
| Tier 3 vision AI on 4,000 images | ~$0.33 on the paid tier (February 2026 estimate) |

> **Note:** Actual costs depend on the Gemini model pricing at the time of use, which
> may have changed since the above estimate.  Check the current rates at
> <https://ai.google.dev/pricing>.

---

## Reviewing and correcting tags

After running AI tagging:

1. Open the catalogue in your browser.
2. Select 1 or more tags.
3. Review the suggested tags on the tags edit page.
4. Correct any wrong tags using the tag selector.
5. Click **Apply tags** to mark the design(s) as verified.
OR
1. Open the catalogue in your browser.
2. Browse to a design.
3. Review the suggested tags on the detail page.
4. Correct any wrong tags using the tag selector.
5. Tick **Verify** to mark the design as verified.

Designs tagged by automation are shown with a *"Tags not verified"* badge until you
verify them.

If you need to tidy the tag library itself, open **Admin → Tags**. New tags must be created
as either **Image** or **Stitching** tags, and you can change an existing tag's group with the
dropdown and `✓` save button.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| `GOOGLE_API_KEY not set` | Add the key via Admin → Settings or add `GOOGLE_API_KEY=…` to your `.env` file |
| Tiers 2/3 not running during import | Check that the checkboxes are ticked in Admin → Settings and that an API key is saved |
| `429 Too Many Requests` | Increase the delay in Settings or per-action in Tagging Actions.  For CLI: `--delay 6.0`.  Set a batch size to spread calls across runs. |
| `403 Forbidden` | Your key may be restricted to certain APIs.  Check the key settings in Google Cloud Console. |
| Tier 2 tags are all wrong | Try enabling Tier 3 in Settings or run Tier 3 from Admin → Tagging Actions to let the vision AI look at the actual stitch pattern |
| `ModuleNotFoundError: No module named 'google'` | Run `pip install -r requirements.txt` with the virtual environment active |
| Want to retag only unverified designs without touching verified ones | Use **Tag untagged and unverified** in Admin → Tagging Actions, or run `python auto_tag.py --skip-verified` from the CLI |

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for general application problems.
