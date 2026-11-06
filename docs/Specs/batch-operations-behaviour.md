# Batch Operations Specification

Reference for batch processing and backfill subsystem behaviour (`src/services/backfill.rs`, `frontend/src/lib/views/BatchOperationsView.svelte`).

## Tagging & Categorisation

### Goals
| Goal | Mode | Description |
|---|---|---|
| `file_folder` | File & Folder Rules | Purely local tokenisation of design filenames and folder hierarchies against tag catalogue and synonyms. |
| `ai_vision` | Enrich with Gemini Vision | Submits rendered PNG thumbnails to Google Gemini API to detect visual subject matter. Requires API key. |
| `full_rescan` | Full Re-Scan | Combines File & Folder Rules and Gemini Vision in sequence and merges results according to merge mode. |

### Scopes
| Scope | Identifier | Filter Criteria |
|---|---|---|
| Untagged designs only | `untagged` | `WHERE NOT EXISTS (SELECT 1 FROM design_tags WHERE design_id = d.id)` |
| Specific Folder(s) | `folder` | Targets designs located within selected directory subtrees. |
| Entire collection | `all` | Evaluates all catalogue records. |
| Missing Gemini Vision | `vision_not_analyzed` | `WHERE COALESCE(d.vision_ai_analyzed, 0) = 0` |
| Gemini Vision found no match | `vision_no_match` | `WHERE COALESCE(d.vision_ai_analyzed, 0) = 1 AND COALESCE(d.vision_ai_matched, 0) = 0` |
| Re-analyse analysed | `vision_analyzed` | `WHERE COALESCE(d.vision_ai_analyzed, 0) = 1` |

### Merge Modes
- **Add**: Inserts new candidate tags without removing existing tags.
- **Reset**: Purges unverified tags prior to inserting new suggestions. Preserves user-verified tags (`image_tags_verified = 1`, `stitching_tags_verified = 1`).

## Maintenance & File Processing

### Tasks
- **Detect / recalculate stitching tags**: Parses stitch geometry (densities, angles, repeated step patterns) via `StitchIdentifier` to detect stitching tags (`Applique`, `Cross Stitch`, `Cutwork`, `Filled`, `In The Hoop`, `Lace`, `Line Outline`, `Satin Stitch`).
- **Generate preview images**: Renders 2D or 3D thread simulation PNGs from raw stitch vectors and saves to thumbnail cache. Updates width, height, and suggested hoop.
- **Recalculate colour / stitch counts**: Parses raw file headers and colour change commands to update `stitch_count`, `color_count`, and `color_change_count`.
- **Recalculate hoop & dimensions**: Recomputes bounding box and hoop suitability without full image redraw.

