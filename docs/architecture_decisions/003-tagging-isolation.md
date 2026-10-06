# Architecture Decision Record 003: Strict Tag Category & Verification Isolation

* **Status:** Accepted
* **Deciders:** Julie Stenning
* **Date:** 2026-10-06

---

## Context and Problem Statement

The application supports two fundamentally distinct categories of tags:
1. **Image / Subject Tags:** Semantic subject matter (e.g., *Floral*, *Animals*, *Christmas*) inferred via offline path/filename rules or Google Gemini Vision AI.
2. **Stitching / Technical Tags:** Density, fill, and structural characteristics (e.g., *Dense Fill*, *Applique*, *Light Underlay*) detected deterministically by parsing binary stitch command sequences.

Conflating image tags with stitching tags or combining their verification flags caused bugs where technical maintenance passes wiped user-verified semantic tags, or vice versa.

---

## Decision Drivers

* **Semantic vs Technical Isolation:** Automated passes in one category must never modify, clear, or inspect tags belonging to the other category.
* **Independent Verification States:** A design can be verified for Image Tags (`image_tags_verified = 1`) while remaining unverified for Stitching Tags (`stitching_tags_verified = 0`), or vice versa.
* **Invariant Reset on Modification:** Adding or removing any tag in a category must immediately reset the corresponding verification flag (`image_tags_verified` or `stitching_tags_verified`) to `0` (unverified).

---

## Decision Outcome

1. **Tag Group Segregation:** Tags in SQLite are partitioned by `tag_group` (`'stitching'` vs standard/image tags).
2. **Dedicated Verification Flags:** The `designs` table maintains two distinct boolean columns: `image_tags_verified` and `stitching_tags_verified`.
3. **UI Boundary:**
   - Tab 1 ("Tagging & Categorisation") handles Image/Subject Tagging (Steps 1–3) with a callout link to Tab 2.
   - Tab 2 ("Maintenance & File Processing") handles technical stitching tag detection, preview generation, and stitch metrics recalculation.
