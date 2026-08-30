# AI-Assisted Auto-Tagging

The Embroidery Catalogue includes an optional feature that uses **Google Gemini AI** to
suggest design-type tags for your embroidery files.  This is entirely optional — the
application works fully without it.

---

## Overview

Auto-tagging works in up to three tiers, tried in order for each design:

| Tier | Method | Requires API key? |
|------|--------|------------------|
| 1 | Keyword matching against the filename and file path | No |
| 2 | Gemini text AI — sends the cleaned filename to the API | Yes |
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

Alternatively, you can add the key manually to a file named **`.env`** in the repository
root (where the launch scripts live):

```
GOOGLE_API_KEY=AIzaSy_your_actual_key_here
```

> **Security note:**  The `.env` file contains a private secret.  Do **not** commit it
> to Git (it is already listed in `.gitignore`).

> **Portable mode note:**  If you use the app from a USB stick / SD card and want the
> API key to travel with the stick, place the `.env` file in the `Data\` folder next to
> the executable (the app reads it from the current working directory).  See
> [App Installer.md](App Installer.md) for the portable setup.

---

## Step 3 — Enable Tier 2 / Tier 3 in Settings

Open **Admin → Settings** and tick the options you want:

- **Run Tier 2 automatically during import** — sends the cleaned filename to Gemini
  to suggest tags each time you import new designs.
- **Run Tier 3 automatically during import** — sends the preview image to Gemini Vision
  to suggest tags for designs that are still untagged after Tiers 1 and 2.

If neither option is ticked, imports run Tier 1 keyword tagging only — no Gemini calls
are made even if a key is present.

### Batch size and delay settings (optional)

For very large imports you can set an **AI tagging batch size** in Settings.  This limits
how many newly imported designs are sent to Gemini in a single import run, so you can
spread the API calls across multiple runs.  Leave the field blank to use the default
(100).

You can also set an **AI delay** in Settings.  This is the pause between Gemini requests.  The
default is **0 (no delay) on a paid key** — paid keys are not rate-limited, so there is no need
to pace calls (concurrency is already bounded by Workers) — and **10 seconds on the free tier**
to stay under the ~15 requests/minute limit.  Leave it blank to use the default.  Increase it if
you see 429 rate-limit errors.  The delay paces each Tier 2/3 request made by Tagging Actions
when an API key is configured.

You can also set a **Commit every** and **Workers** value in Settings. **Commit every** is
how often a Tagging Actions run reports progress/commits (default 100); **Workers** is how
many designs are tagged in parallel (default 4). Lowering **Workers** helps avoid Gemini
rate-limit (429) errors.

If your Google API key is on the **free tier**, tick **"My Google API key is on the free
tier"** in Settings. Free-tier keys have strict per-minute and per-day limits (roughly 15
requests/minute and 1,500/day), so higher concurrency does not make a free-tier run faster —
it just makes it hit the rate limit sooner. When the free-tier option is enabled, **blank
Workers and AI-delay fields default to a conservative pair (2 workers / 10s)** so runs stay
under the limit; explicit values you enter are always used as-is. The Tagging Actions page
also shows guidance to keep **Workers** low and the **AI delay** high, and if a 429 rate-limit
error still occurs the run stops and tells you roughly how long to wait (it does not retry
automatically, which could lock the key out for the rest of the day).

You can also choose a **Gemini model** in Settings. This selects which Gemini model is used
for Tier 2/3 tagging. Leave it blank to let the app auto-select an available model at run
time (recommended — Gemini model names are renamed/retired over time). Use the **Refresh**
control to reload the model list, and **Test model** to verify a model actually works (the
Test button sends a real probe call — some models are *listed* but still restricted to new
users). If a model you chose is later retired, the app automatically falls back to another
model; if no model works it stops with a clear message pointing to the backfill log.

You can also set an **Import database commit batch size** in Settings.  This controls how
many designs are written or tag-updated before each database commit during import.
Leave it blank to use the default (100).

---

## Cost, models & the free tier

- **Use a flash model for the lowest cost.** Flash models are the fastest *and* cheapest tier
  and are more than sufficient for the small text/vision prompts tagging sends. The Settings
  model list is sorted flash-first, and auto-selection always prefers a flash model — the same
  flash model is chosen on both the free and paid tiers. Pro/thinking models cost more and run
  slower for no benefit on this small task.
- **The free-tier option is about rate limits, not cost.** Tick **"My Google API key is on the
  free tier"** only if your key is genuinely on the free tier (roughly 15 requests/minute and
  1,500/day). It is *not* a general "save money" setting for paid keys — it lowers concurrency,
  raises the delay, and changes how 429 errors are reported. A paid user wanting to keep spend
  low should simply use a flash model and skip Tier 3 (vision).
- **Zero-cost overnight tagging (free-tier keys).** If you have a Google account with free-tier
  API access, you can tag a large library at **no monetary cost** by selecting the free-tier
  option and running the backfill **overnight / across several days** — the app paces to the
  ~1,500/day limit and stops cleanly when it is reached. This is slow, but free.

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
| **Tag only untagged designs** | Processes designs with no image tags at all.  Verified tags are never touched.  Safe to run at any time. |
| **Tag untagged and unverified designs** | Processes both untagged designs and designs whose tags have not yet been verified.  Manually verified designs are left untouched. |
| **Re-tag ALL designs** | Overwrites existing image tags on every design, including ones you have manually verified. |
| **Local stitching backfill** | Analyses the actual embroidery pattern geometry (via StitchIdentifier) to detect stitch types. Updates only the `stitching` tag group for existing unverified designs. No Gemini/API calls are made. See [STITCH_TYPES.md](STITCH_TYPES.md) for the current detector behavior. |
| **Images** | Regenerates preview images for designs (optionally redoing existing previews).  Local only — no Gemini calls. |
| **Colour counts** | Recomputes stitch count / colour count / colour-change count metadata.  Local only — no Gemini calls. See [COLOUR_COUNTS.md](COLOUR_COUNTS.md). |
| **Fingerprinting** | Back-fills file hashes and sizes for designs that are missing them.  Local only — no Gemini calls. |

For a full walkthrough of combined backfill runs (tagging, stitch types, images, and thread/colour metadata), see
[TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md).

### Options per action

- **Tiers** — choose which tagging tiers to run.  Tier 1 is always included.  Tiers 2 and 3
  are only available when an API key is configured — if no key is present, any selection of
  tiers 2 or 3 is ignored and only Tier 1 runs.
- **Batch size** — the number of designs fetched and processed together per chunk
  (defaults to the value in Settings). A run pages through ALL matching designs, so
  batch size does not cap how many designs are touched.
- **Tier 2 delay** — pause between Tier 2 (text) Gemini requests, applied only when
  an API key is configured (to pace API calls and avoid 429 rate-limit errors).
- **Tier 3 delay** — pause between Tier 3 (vision) Gemini requests. Same as Tier 2 —
  only applies to real outbound Gemini calls.
- **Workers** — number of concurrent processing workers (default 4; range 1–32).
  Designs within a batch are tagged concurrently using this many workers.

### Warnings and tips

- A cost/quota warning is displayed whenever an API key is configured, consistent with the
  import flow.
- A blue notice is shown when no API key is present, with links to Settings and the AI
  Tagging Guide.
- **Local stitching backfill, images, colour counts, and fingerprinting** are fully offline
  and do not use the Gemini API.
- The current stitch-type detector is geometry-based and covers the 8 implemented types in [STITCH_TYPES.md](STITCH_TYPES.md).
- Actions run with inline progress display and a **Stop** button.  No page reload occurs
  when complete; results are shown in the UI.

---

## Using AI tagging on the portable / USB copy

When the application is deployed to a USB stick or SD card, add the API key via
**Admin → Settings** on the target machine, or place the `.env` file in the `Data\`
folder next to the executable.  The in-app **Admin → Tagging Actions** page then runs
AI tagging exactly as on any other machine.

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

1. Open the Embroidery Catalogue app.
2. Browse to the design(s).
3. Review the suggested tags on the detail page (or use the Browse page for multiple designs).
4. Correct any wrong tags using the tag selector.
5. Click **Apply tags** and/or tick **Verify** to mark the design(s) as verified.

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
| `404 Model not found` | The chosen Gemini model has been retired. Clear the **Gemini model** field in Settings (or pick a current model from the dropdown) to use auto-selection, then retry. |
| Tiers 2/3 not running during import | Check that the checkboxes are ticked in Admin → Settings and that an API key is saved |
| Tiers 2/3 not available in Tagging Actions | An API key must be configured.  Without a key, tiers 2 and 3 are automatically excluded and only Tier 1 runs |
| `429 Too Many Requests` | Increase the **AI delay** in Settings, or the **Tier 2/3 delay** in Tagging Actions.  Also set a batch size to spread calls across runs |
| `403 Forbidden` | Your key may be restricted to certain APIs.  Check the key settings in Google Cloud Console |
| Tier 2 tags are all wrong | Try enabling Tier 3 in Settings or run Tier 3 from Admin → Tagging Actions to let the vision AI look at the actual stitch pattern |
| Tier 3 does not seem to run | Tier 3 needs a generated preview image for the design.  Run the **Images** action in Tagging Actions first, then retry tagging |
| Want to retag only unverified designs without touching verified ones | Use **Tag untagged and unverified** in Admin → Tagging Actions |

See [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md) for general application problems.