# Plan: Reduce AI Tagging — Remove Stitching Tags from AI Tagging Pipeline

## Objective

Stitching-type tags (e.g. "Satin Stitch", "Filled", "Applique", "Cross Stitch", etc.) should **only** be assigned by the `StitchIdentifier` pattern-analysis engine, **not** by the AI tagging pipeline (Tier 1 keyword matching, Tier 2 Gemini text, or Tier 3 Gemini vision). The AI tagging pipeline should only assign tags in the **"image"** tag group.

## Background

- `StitchIdentifier` (`src/services/stitch_identifier.py`) analyses actual embroidery stitch geometry to detect stitch types.
- `suggest_stitching_from_pattern()` (`src/services/auto_tagging.py`) wraps `StitchIdentifier` and is called by the stitching backfill action.
- The AI tagging pipeline (`run_tagging_action()`) currently passes **all** valid tag descriptions (including stitching tags) to Tier 1 keyword matching, Tier 2 Gemini text, and Tier 3 Gemini vision.
- The `KEYWORD_MAP` in `auto_tagging.py` contains stitching-related keywords (e.g. "applique", "satin", "cross_stitch", "ith", "lace", etc.) that cause Tier 1 to assign stitching tags based on filenames.
- The design selection for `tag_untagged` already correctly filters to only designs without **image** tags (comment: "they may have stitching tags — those are unrelated to AI keyword tagging").

## Changes Required

### 1. `src/services/auto_tagging.py` — Remove stitching keywords from `KEYWORD_MAP`

Remove these entries from `KEYWORD_MAP` (lines 198-218):

| Lines | Keywords | Tag Description |
|-------|----------|-----------------|
| 199-200 | `"applique"`, `"appliqué"` | Applique |
| 201 | `"blackwork"` | Blackwork |
| 202 | `"redwork"` | Redwork |
| 203 | `"cutwork"` | Cutwork |
| 204 | `"trapunto"` | Trapunto |
| 205 | `"lace"` | Lace |
| 206-207 | `"quilt"`, `"patchwork"` | Quilting |
| 208-210 | `"cross_stitch"`, `"crossstitch"`, `"xstitch"` | Cross Stitch |
| 211 | `"satin"` | Satin Stitch |
| 215-219 | `"ith"`, `"in_the_hoop"`, `"ith_acc"`, `"mug_rug"`, `"zipper"` | In The Hoop / ITH Accessories |
| 344 | `"outline"` | Line Outline |

### 2. `src/services/auto_tagging.py` — Filter stitching tags from `valid_descriptions` in `suggest_tier2_batch()`

The `suggest_tier2_batch()` function receives `valid_descriptions` and passes them to Gemini. We need to filter out stitching-group tags before sending to Gemini.

**Approach**: Since `suggest_tier2_batch()` doesn't have access to the tag group, we should filter at the caller level (`run_tagging_action()`). The `valid_descriptions` passed to Tier 2 and Tier 3 should only include "image" group tags.

### 3. `src/services/auto_tagging.py` — Filter stitching tags from `valid_descriptions` in `suggest_tier3_vision()`

Same as Tier 2 — filter at the caller level.

### 4. `src/services/auto_tagging.py` — Update `run_tagging_action()` to filter valid descriptions

In `run_tagging_action()`, after loading `all_tags`, build `valid_descriptions` to only include tags where `tag_group == "image"`. This ensures Tier 2 and Tier 3 only suggest image tags.

### 5. `src/services/unified_backfill.py` — Update tagging action to filter valid descriptions

The `run_tagging_action_runner()` delegates to `run_tagging_action()`, so the fix in step 4 covers this path too. However, the `unified_backfill()` function's tagging design selection already correctly filters by image tags only.

### 6. Tests — Update tests that rely on stitching keywords in `KEYWORD_MAP`

- `tests/test_root_scripts.py`: Tests that mock `suggest_tier1` may need updating if they relied on stitching keywords.
- `tests/test_legacy_tagging_actions.py`: Tests that verify stitching tags are NOT assigned by AI tagging should be added.
- `tests/test_folder_picker_and_tagging.py`: Tests that mock `suggest_tier1` may need updating.

## Implementation Order

1. Remove stitching keywords from `KEYWORD_MAP` in `auto_tagging.py`
2. Filter `valid_descriptions` in `run_tagging_action()` to only include image-group tags
3. Update tests
4. Write the plan file

## Files Modified

- `src/services/auto_tagging.py` — Main changes
- `tests/test_root_scripts.py` — Test updates
- `tests/test_legacy_tagging_actions.py` — Test updates
- `tests/test_folder_picker_and_tagging.py` — Test updates
- `plans/reduce-ai-tagging.md` — This plan file
