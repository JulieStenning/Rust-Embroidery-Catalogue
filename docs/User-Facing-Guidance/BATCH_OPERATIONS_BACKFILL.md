# Batch Operations and Backfill (with AI Tagging)

The Embroidery Catalogue includes a **Batch Operations** page that lets you update existing designs in bulk.
You can run AI tagging, stitching detection, image generation, and thread/colour count backfills from one place.

---

## What this page is for

Use **Admin -> Batch Operations** when you want to:

- tag designs based on their images using Visual AI,
- detect stitch types from embroidery geometry,
- generate or refresh preview images and dimensions,
- fill in missing stitch/colour/count metadata,
- run multiple maintenance actions in one run.

---

## Before you run

1. Keep the page open while actions run.
2. Make a backup first for large runs.
3. If using Visual AI, confirm your API key is configured in **Admin -> Settings**.
4. Start with a small run if this is your first time using combined actions.

> Tip: A single run processes **all** designs that match the selected action(s);
> batch size is just the internal chunk size. If your library is very large, keep
> the page open while it runs, or use **Stop running** to interrupt and re-run later
> (progress resumes from where it stopped).

---

## Quick start

1. Open **Admin -> Batch Operations**.
2. Choose a **Goal**:
   - **Apply file & folder rules** — fast, local, offline, free.
   - **Enrich with visual AI** — sends preview thumbnails to Google Gemini Vision (needs an API key).
   - **Full re-scan (both methods)** — runs File & Folder Rules and Visual AI and merges results.
3. Choose a **Scope** (see below), then a merge mode.
4. Configure any options for the other actions you ticked (Stitch types, Images, Threads and Colours).
5. Choose batch/commit/worker values.
6. Click **Run selected actions**.
7. Watch the progress message on the page, and download the error log afterwards if needed.

---

## AI tagging (Visual AI)

AI tagging uses Google Gemini to suggest image-group tags for a design. It is entirely optional — the
application works fully without it.

Two tagging modes are available when tagging a design:

- **File & Folder Rules** — instantaneous, local matching of the filename and folder path against your
  tag catalogue. It runs only when the chosen goal includes it, is free, and never calls the network.
  Does **not** require an API key.
- **Visual AI** — Gemini Vision, which sends the design's rendered preview image to the API for
  analysis. It requires an API key and consumes quota from your account. It needs a rendered preview
  image, so run the **Images** action first if a design has none.

Scopes for a Visual AI / Full Re-Scan run let you target designs by their analysis state:

- **Designs missing Visual AI analysis** — designs not yet scanned by Visual AI.
- **Visual AI found no match** — designs Visual AI analyzed but produced no tags.
- **Re-analyze (already analyzed)** — designs already analyzed, run again.
- **Untagged designs only**, **Specific Folder(s)**, and **Entire collection** are also available.


### Get a Google API key

1. Go to <https://aistudio.google.com/> and sign in with a Google account.
2. Click **Get API key** (top-left) → **Create API key**.
3. Copy the key — it looks like `AIzaSy…`.

> **Free tier:** the Gemini API offers a free tier that allows up to 15 requests per
> minute and 1,500 requests per day. This is sufficient for tagging a few hundred
> designs. Larger collections may need a paid plan.

### Add the key in Admin Settings

Open **Admin → Settings** and paste the key into the **Google Gemini API key** field, then click
**Save settings**. The key is only used by Visual AI in **Batch Operations** — imports never call Gemini.

> **Security note:** treat the key as a private secret. It is stored locally with your catalogue data
> (in the app's settings) and is used only by Visual AI in Batch Operations.

### Batch, delay, model, and free tier

These options on **Admin → Settings** control Visual AI runs in Batch Operations:

- **AI tagging batch size** — the number of designs fetched and processed together per chunk. A run
  pages through the whole candidate set, so this does not cap how many designs are touched.
- **Commit every** — how often the run reports progress/commits (default 100).
- **Workers** — how many designs are tagged in parallel (default 4). Lower this to reduce Gemini
  rate-limit (429) errors.
- **Delay between Gemini calls** — pause between API requests (default 0 on a paid key, 10 s on the
  free tier). Increase it if you see 429 errors.
- **My Google API key is on the free tier** — tick this for a free-tier key. It uses conservative
  defaults (2 workers / 10 s) so runs stay under the ~15 requests/minute and ~1,500/day limits.
- **Gemini model** — which model is used. Leave blank to auto-select an available model (recommended —
  model names are renamed/retired over time). Use **Refresh** to reload the list and **Test model** to
  verify a model works.

If a configured model has been retired, the app falls back to auto-selection. A 429 rate-limit error
stops the run with a "how long to wait" message (it does not retry, which could lock the key out for
the rest of the day).

---

## Cost, models & the free tier

- **Use a flash model for the lowest cost.** Flash models are the fastest *and* cheapest and are more
  than sufficient for the small prompts tagging sends. The Settings model list is sorted flash-first.
- **The free-tier option is about rate limits, not cost.** Tick it only if your key is genuinely on the
  free tier. It raises the pacing delay and changes how 429 errors are reported.
- **Zero-cost overnight tagging (free-tier keys).** You can tag a large library at **no monetary cost**
  by ticking the free-tier option and running the backfill **overnight / across several days** — the
  app paces to the ~1,500/day limit and stops cleanly when it is reached.

| Scenario | Approximate cost |
|---|---|
| File & Folder Rules only | Free |
| Visual AI on 4,000 images | ~$0.33 on the paid tier (February 2026 estimate) |

> **Note:** actual costs depend on the Gemini model pricing at the time of use, which may have changed
> since the above estimate. Check the current rates at <https://ai.google.dev/pricing>.

---

## Action guide

### Tagging (File & Folder Rules + Visual AI)

Use this to apply or refresh image-group tags.

- **Tag only untagged** — safe routine top-up; never touches verified tags.
- **Re-tag unverified / Visual AI scopes** — process designs missing or no-match analysis.
- **Re-tag ALL (including verified)** — overwrites everything; requires confirmation.

File & Folder Rules runs only when the chosen goal includes it (it is not run automatically).
Visual AI requires a configured Google API key and is skipped when no key is present.

### Stitch types

Use this to detect stitching tags directly from pattern geometry.

Option:

- **Clear existing stitching tags for unverified designs first**

Run after changing stitch-related tags, or after importing older designs with no stitching tags.
This is fully offline — no Gemini calls. See [STITCH_TYPES.md](STITCH_TYPES.md).

### Images

Use this to generate design preview images (fully offline).

Options:

- **Also generate preview images** — tick this to generate a preview image for designs that lack one.
- **Regenerate images for all designs** (shown once the option above is ticked) — also regenerate
  previews for designs that already have one.

Visual AI needs a preview image, so run this before Visual AI tagging on designs without one.

### Threads and Colours

Use this to fill technical metadata fields: stitch count, colour count, and colour-change count.
Typically safe to combine with image/stitching runs. See
[COLOUR_COUNTS.md](COLOUR_COUNTS.md).

---

## Batch, commit, and workers

Configured on **Admin → Settings**:

- **AI tagging batch size** — internal chunk size; does not limit total designs touched.
- **Commit every** — progress/commit cadence.
- **Workers** — parallel concurrency; lower to reduce 429 errors.

Practical guidance:

- Keep the defaults for most runs.
- Use lower values if your machine is resource-constrained.
- Increase **Workers** carefully; higher values can increase CPU and disk load.
- To process only a subset, tick exactly the actions you need and use **Stop running** to halt once
  enough designs have been handled.

---

## Progress, stopping, and logs

- The progress area shows a running/completed summary.
- A live **Progress** message updates after each commit.
- Use **Stop running** to request a graceful stop.
- Use **Download error log** to review design-level failures.

The log helps identify file-specific problems (unsupported or corrupt files, parse failures). If any
errors occurred, rerun targeted actions after fixing the root cause.

---

## Recommended workflows

### Safe routine maintenance

1. Run **Tag only untagged** (File & Folder Rules, optionally Visual AI).
2. Run **Stitch types** (without clearing existing) if needed.
3. Run **Threads and Colours**.
4. Check results and verify tags in Browse/Detail views.

### Full Visual AI refresh

1. Back up first.
2. Run **Re-tag ALL unverified** (or a Visual AI scope such as *missing analysis*).
3. If any designs lack a preview image, run the Images action to generate one.
4. Run **Threads and Colours**.
5. Review and download the error log.

### Image rebuild pass

1. In Batch Operations, tick **Also generate preview images**.
2. Tick **Regenerate images for all designs** to rebuild existing previews too.
3. Run and monitor until completion.

---

## Reviewing and correcting tags

After running AI tagging:

1. Open the app and browse to the design(s).
2. Review the suggested tags on the detail page (or the Browse page for multiple designs).
3. Correct any wrong tags with the tag selector.
4. Click **Apply tags** and/or tick **Verify** to mark the design(s) as verified.

Designs tagged by automation are shown with a *"Tags not verified"* badge until you verify them.

If you need to tidy the tag library itself, open **Admin → Tags**. New tags must be created as either
**Image** or **Stitching** tags.

---

## Troubleshooting

- **Run does not start** — confirm at least one action is selected, then try again.
- **Visual AI not available** — check the API key in Settings and save. Without a key only File & Folder Rules run.
- **API key not set** — add the key in Admin → Settings. Without a key only File & Folder Rules run.
- **`404 Model not found`** — the chosen Gemini model was retired. Clear the **Gemini model** field in Settings (or pick a current model) and retry.
- **Visual AI does not seem to run** — Visual AI needs a preview image. Run the **Images** action first, then retry tagging.
- **`429 Too Many Requests`** — increase the **AI delay** (or tick the free-tier option) in Settings, and/or lower Workers.
- **`403 Forbidden`** — your key may be restricted to certain APIs. Check the key settings in Google Cloud Console.
- **Many failures in log** — verify source files still exist and are readable, then rerun affected actions.
- **Browser closed during run** — reopen Batch Operations and rerun. Keep the page open until completion.
- **Performance is slow** — lower workers and a smaller batch size, or press **Stop running** and re-run later.
