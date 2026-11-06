# Tag Word Matches

**Tag Word Matches** map words in filenames and folder paths to catalogue tags. For example, a file called `Puppy_4x4.pes` in a folder called `Dogs` can automatically receive the **Animals** tag.

Tag word matching is completely offline and runs during import and local batch tagging.

---

## Add keyword matches to a tag

1. Go to [Admin → Manage Data → Word Matches](#/admin/data/tag-matches).
2. At the top of the page, choose a tag from the **Select tag to add match** dropdown.
3. In the input box, type one or more words separated by commas (e.g. `pup, puppy, hound, dog`).
4. Click **Add Matches** (or press <kbd>Enter</kbd>).

Alternatively, find any tag card on the page and type new words into the **Add match…** box at the bottom of its card.

---

## Remove a word match

1. Open [Admin → Manage Data → Word Matches](#/admin/data/tag-matches).
2. Locate the tag card.
3. Click the **✕** button on any keyword chip to remove it.

---

## How word matching works

- **Whole word matching**: A match for `cat` matches `Black_Cat.pes` and a folder called `Cats`. It does not match `Catch.pes` or `Catalogue`.
- **Automatic plurals**: Matching handles standard English singular and plural forms automatically. Adding `puppy` matches both `puppy` and `puppies`.
- **Case-insensitive**: `Rose` matches `ROSE.pes`, `rose.jef`, and `Rose_Border.dst`.
- **Designer & Source exclusion**: Words matching your saved Designer or Source names (e.g. `Embroidery Library` or `Urban Threads`) are automatically excluded from tag matching so brand names do not create accidental tags.

---

## If something goes wrong

| Problem | What to do |
|---|---|
| A word shows an "Already added" warning | The word or a variation is already assigned to this tag. |
| A word cannot be added because it matches the tag name | The tagging engine automatically matches words from the tag name itself, so you do not need to add them as synonyms. |
| You want to re-apply updated rules to existing designs | Go to [Admin → Batch Operations](#/admin/batch-operations) and run **Apply file & folder rules**. |

---

## Related guides

- [Batch Operations](BATCH_OPERATIONS_BACKFILL.md)
- [Import Workflow](IMPORT_WORKFLOW.md)
- [Stitch Types](STITCH_TYPES.md)
- [Search and Filtering](SEARCH_AND_FILTERING.md)
