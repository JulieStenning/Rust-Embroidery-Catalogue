# Stitch Types

Embroidery Catalogue analyses pattern geometry and stitch vectors to automatically detect stitching styles and assign tags in the **stitching** category.

---

## Supported stitch types

The built-in detector identifies 8 stitch types:

- **Applique**: designs with placement and tack-down stitch sequences.
- **Cross Stitch**: uniform X-stitch pattern sequences.
- **Cutwork**: open-space cutting boundary stitches.
- **Filled**: solid fill-stitch areas and tatami fills.
- **In The Hoop**: multi-step project designs constructed entirely within the hoop.
- **Lace**: free-standing lace (FSL) with interlocking underlay grids.
- **Line Outline**: single-run outline and redwork-style contour lines.
- **Satin Stitch**: column stitches with alternating left-right needle penetrations.

Stitch detection is completely local and offline. It does not require an API key or an internet connection.

---

## Filter designs by stitch type

1. Open **Browse** from the top menu.
2. In the filter sidebar on the left, locate the **Stitch Types** section.
3. Tick one or more stitch types (e.g. **Lace** or **In The Hoop**) to narrow your search.

---

## Recalculate stitching tags on existing designs

Stitch types are detected automatically during import. To update older designs:

1. Open [Admin → Batch Operations](#/admin/batch-operations).
2. Click the **Maintenance & File Processing** tab.
3. Under **Target Scope**, select **Entire collection** or **Specific folder(s)**.
4. Tick **Detect / recalculate stitching tags**.
5. Click **Start Maintenance**, then confirm.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| A design received an incorrect stitch tag | Open the design in [Browse](#/browse), click its card, and click the **✕** on the tag chip to remove it. User-edited tags are never overwritten by automated runs unless you explicitly choose to overwrite verified tags. |
| A specialised design (like FSL) was not detected | Include hints in the design's folder or filename (such as `FSL` or `Lace`), or configure keyword matches in [Admin → Manage Data → Word Matches](#/admin/data/tag-matches). |

---

## Related guides

- [Batch Operations](BATCH_OPERATIONS_BACKFILL.md)
- [Tag Word Matches](TAG_WORD_MATCHES.md)
- [Search and Filtering](SEARCH_AND_FILTERING.md)
