# Tag Word Matching Specification

Technical reference for offline filename and folder path tokenisation and keyword matching (`src/services/auto_tagging.rs`, `frontend/src/lib/views/TagWordMatchesView.svelte`).

## Overview

The tagging engine extracts candidate tokens from design filenames and folder paths to assign catalogue tags without network calls. Tag matching executes in two stages:
1. **Pass 1 (Direct Tag Matching):** Exact token matching against canonical tag names.
2. **Pass 2 (Word Match / Synonym Matching):** Matching tokens against configured synonym keywords mapped to tags.

## Tokenisation Pipeline

### 1. Delimiter & Boundary Splitting
Paths and filenames are split into individual tokens based on three boundary conditions:
- **Punctuation & Whitespace:** Split on spaces, underscores (`_`), hyphens (`-`), periods (`.`), and path separators (`/`, `\`).
- **CamelCase Transitions:** Split between a lowercase letter and an uppercase letter (e.g. `MonarchButterfly` $\rightarrow$ `monarch`, `butterfly`).
- **Alphanumeric Transitions:** Split between alphabetic characters and numeric sequences (e.g. `Butterfly1` $\rightarrow$ `butterfly`, `12Floral` $\rightarrow$ `floral`).

### 2. Normalisation & Stemming
- **Case Normalisation:** All tokens and match rules are converted to lowercase.
- **Inflection Stemming:** The engine evaluates both singular and plural forms for each token using inflection heuristics:
  - `to_singular()`: Strips trailing `s`, `es`, `ies` $\rightarrow$ `y` (e.g. `butterflies` $\rightarrow$ `butterfly`, `roses` $\rightarrow$ `rose`).
  - `to_plural()`: Appends `s`, `es`, `y` $\rightarrow$ `ies`.

### 3. Masking & Stop-Word Filtering
Before candidate tokens are tested against tags, filtering rules prune non-semantic tokens:
- **Designer & Source Masking:** If any folder segment in the path matches an assigned or inferred `Designer` or `Source` name (e.g. `Urban Threads`, `Embroidery Library`), that folder's tokens are masked to prevent brand names from generating unintended subject tags.
- **Craft Stop Words:** Generic medium words are discarded: `embroidery`, `stitch`, `stitched`, `stitching`.
- **Format Extension Stop Words:** File extensions are discarded: `jef`, `pes`, `hus`, `vp3`, `dst`, `exp`, `eof`, `emb`, `art`, `be`, etc.
- **Root Storage Filtering:** Library root names (`MachineEmbroideryDesigns`, `Database`) are ignored.

## Matching Rules & Invariants

| Invariant | Rule | Rationale |
|---|---|---|
| Whole-Word Matching | Tokens match full words only (`\bword\b`). | Prevents substring false matches (e.g. `cat` will not match `catch` or `catalogue`). |
| No Singular/Plural Duplication | Users cannot configure both singular and plural forms of the same keyword (e.g. add `quote`, reject `quotes`). | The engine computes inflections automatically. |
| No Self-Synonyms | Users cannot add a synonym that matches words in the tag's own name (e.g. tag `Words` cannot have match `word`). | Pass 1 matches tag name tokens directly in singular and plural forms. |
| System Tag Separation | Text-based embroidery is partitioned between `Alphabets` (fonts, letter sets, glyphs, monograms) and `Words` (sayings, phrases, quotes). | Prevents typography compositions from being lumped with font sets. |

