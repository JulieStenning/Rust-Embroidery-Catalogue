# Colour and Stitch Counts

Embroidery Catalogue reads technical stitch metadata from digital embroidery files:
- **Stitches**: total stitch count
- **Colours**: number of distinct thread colours
- **Colour changes**: total number of colour-stop commands

---

## View colour and stitch counts

1. Open **Browse** from the top menu and click any design card.
2. In the design detail view, review the **Technical Details** section:
   - Stitches badge (total stitches)
   - Colours badge (distinct thread count)
   - Colour changes badge (machine stop count)
3. You can also view these counts in the **Print View** ([Design Detail → Print](#/browse)) when creating work sheets.

---

## Backfill missing counts for existing designs

Counts are extracted automatically during import. If you imported designs before count extraction was available:

1. Open [Admin → Batch Operations](#/admin/batch-operations).
2. Click the **Maintenance & File Processing** tab.
3. Under **Target Scope**, select **Entire collection** or **Specific folder(s)**.
4. Tick **Recalculate colour / stitch counts**.
5. Click **Start Maintenance**, then confirm.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| A design shows no stitch or colour counts | The file format may not expose stitch totals, or the file was moved. Verify the file exists and run **Recalculate colour / stitch counts** again. |
| Colour counts differ from your machine display | Some formats (such as Tajima `.dst`) do not store specific thread colours, only colour-change commands. The app counts colour-stop commands as unique colours. |

---

## Related guides

- [Batch Operations](BATCH_OPERATIONS_BACKFILL.md)
- [Design Detail](DESIGN_DETAIL.md)
- [Supported Formats](SUPPORTED_FORMATS.md)
