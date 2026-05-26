# Stitch Types

The Embroidery Catalogue can identify stitch-style tags by analysing the actual embroidery pattern geometry.
This is a local feature that works without Gemini or any other external API.

---

## What this feature does

Stitch type detection looks at the pattern itself and suggests tags in the **stitching** group.
It is used in two places:

1. during import, and
2. from **Admin -> Tagging Actions** using the stitching backfill option.

The current implementation uses geometry and stitch-path analysis, not example images.

---

## Current stitch types

The detector currently covers these 8 stitch types:

- `Applique`
- `Cross Stitch`
- `Cutwork`
- `Filled`
- `In The Hoop`
- `Lace`
- `Line Outline`
- `Satin Stitch`

If you see older notes about `Redwork` or `Blackwork`, those belong to the roadmap version of the feature and are not part of the current implementation.

---

## How it works

The feature analyses pattern geometry locally through `StitchIdentifier`.
It uses:

- stitch vectors,
- stitch angles,
- stitch density,
- repeated path patterns,
- name hints from the filename and folder name.

Some stitch types are especially likely to be detected from the name alone, such as:

- `In The Hoop`
- `Applique`
- `Cross Stitch`
- `Lace`

That means a folder or filename like `wedding_lace_design.pes` can help the detector make a stronger match.

For the technical background, see [docs/Specs/stitch-types-backend-spec.md](../Specs/stitch-types-backend-spec.md).

---

## During import

When you import new designs, stitch detection runs as part of the normal import flow.

What to expect:

- the feature is automatic,
- it does not require an API key,
- it only updates the stitching tag group,
- any result is still something you should review later if you want manual verification.

If you also use AI tagging, stitch detection is separate from the Gemini tiers.

For the AI tagging flow, see [AI_TAGGING.md](AI_TAGGING.md).

---

## In Admin Tagging Actions

Use **Admin -> Tagging Actions** when you want to run stitch detection on existing designs.

The stitching action is useful when:

- older files were imported before stitch detection was available,
- you want to refresh stitching tags after improving the detector,
- you want to run a local backfill without touching AI tags.

Important behavior:

- the stitching backfill is local,
- verified designs are left alone when the backfill is configured to work on unverified items only,
- it only updates tags in the stitching group,
- it keeps running in the browser while the page stays open.

For the full combined maintenance workflow, see [TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md).

---

## Good file naming habits

File and folder names can help the detector.

Examples that are helpful:

- `cross_stitch`
- `applique`
- `wedding_lace_design`
- `in_the_hoop`

Examples that are still fine if the folder name is descriptive:

- `001.pes`
- `sample_01.dst`
- `design.pes`

The current detector uses the pattern geometry first, so a simple filename can still be enough when the stitches are distinctive.

---

## What it does not do yet

This feature does not currently:

- use example-image matching,
- call Gemini or any external service,
- change image tags,
- detect the older 10-type plan as a completed feature set.

If you were expecting `Redwork` or `Blackwork`, treat those as future work rather than a current result.

---

## Reviewing the result

After an import or a stitching backfill:

1. Open the design in Browse or Detail view.
2. Check the stitching tags that were suggested.
3. Verify the tags if they look correct.
4. Use **Admin -> Tags** if you need to tidy the tag library itself.

If you are running the stitching backfill on a large catalogue, keep the page open until the run finishes.

---

## Related guidance

- [AI_TAGGING.md](AI_TAGGING.md) - AI tiers, import-time tag behavior, and where stitching fits alongside them
- [TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md) - combined maintenance runs and batch guidance
- [GETTING_STARTED.md](GETTING_STARTED.md) - basic application setup and import flow