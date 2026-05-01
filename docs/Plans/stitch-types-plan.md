# Plan: Embroidery Stitch Type Identification Module

## Overview

Create a portable, object-oriented `Stitch Identifier` module that analyzes `'pyembroidery.EmbPattern` objects to detect 10 embroidery stitch types. The module will use a two-phase approach:
1. **Name-based detection** for ITH and appliqué (check filename/folder keywords)
2. **Pattern analysis** on stitch sequences (for all types)

Returns a simple list of detected stitch types with a configurable confidence threshold (default: 70%).

---

## Module Specifications

### Location
- **File:** `pyembroidery/StitchIdentifier.py`
- **Import:** Add to `pyembroidery/__init__.py` for accessibility

### Input/Output

**Input:**
- `pattern` (EmbPattern object) — required
- `filename` (str) — required, for name-based detection (ITH, appliqué, redwork, blackwork, cross stitch)
- `folder_name` (str) — required, for name-based detection (ITH, appliqué, redwork, blackwork, cross stitch)
- `confidence_threshold` (float) — default 0.70, configurable

**Output:**
- `identify_stitches()` → `List[str]` — sorted list of detected stitch types (e.g., `['cross_stitch', 'filled']`)
- `get_detailed_analysis()` → `Dict` — returns confidence scores for each type (for debugging)

### Design Pattern
Object-oriented with a single `StitchIdentifier` class.

---

## Class Structure

### Constructor
```python
__init__(pattern: EmbPattern, 
         filename: str,
         confidence_threshold: float = 0.70)
```

2. **`get_detailed_analysis() -> Dict`**
   - Returns breakdown with confidence scores per type
   - Example: `{'cross_stitch': 0.85, 'filled': 0.92, 'satin': 0.55}`

### Private Helper Methods

- `_detect_applique() -> float`
- `_detect_filled() -> float`
- `_detect_cutwork() -> float`
- `_detect_outline() -> float`
**Utility Helpers:**
- `_extract_color_blocks()` — Uses `pattern.get_as_colorblocks()`
- `_calculate_distance(dx, dy) -> float` — Euclidean distance
- `_analyze_stitch_sequence()` — Generic helper for scanning consecutive stitches

---

## Detection Algorithms

### 1. Cross Stitch
**Two-phase detection:**

- Check if `filename` OR `folder_name` contains (case-insensitive): `"cross stitch"`, `"cross-stitch"`, `"cross_stitch"`
- If found: Return confidence **0.95** (name match is very reliable)

**Phase 2 (Pattern-based, if name not found):**
- Stitches are typically non-consecutive (interspersed with jumps or other stitches)

- Weak/unclear pattern → <0.60 (filtered out)

- If no name-based match: Iterate through color blocks
- For each block, analyze stitch angles
---


**Phase 1 (Name-based — always checked):**

**Phase 2 (Pattern-based, if name not found):**
  2. Tack-down stitches (running stitch fastening fabric)
  3. Decorative elements (may be satin, fill, etc.)
- Signature pattern: outline → stitch → satin → outline → stitch
- Return confidence **0.75-0.85** if pattern detected

**Implementation:**


### 3. Appliqué
**Two-phase detection:**
- If found: Return confidence **0.95** (name match is very reliable)
**Phase 2 (Pattern-based, if name not found):**
- Look for characteristic appliqué 3-line pattern:
  1. **Line 1:** Running stitch outline (shows where to place fabric)

- Always check keywords first in filename/folder, return 0.95 if match
- If no name-based match: Analyze stitch patterns
- Look for pairs of running stitch outlines with similar paths
---
### 4. Filled
**What to look for:**
- Dense stitching that covers an area with a pattern
- Visual characteristic: parallel lines of stitching close to each other
- Typically identified by presence of STITCH_BLOCK (groups of consecutive stitches)
- Pattern: Multiple consecutive STITCH commands with decreasing/increasing Y (or X) in structured pattern

**Confidence scoring:**
- Clear stitch block with parallel pattern → 0.80-0.90
- Multiple blocks found → higher confidence
- Unclear pattern → <0.60

**Implementation:**
- Use `pattern.get_as_stitchblock()` to detect blocks
- Check block size and density
- Analyze stitch direction consistency (parallel lines)
- Return confidence based on regularity and block count

---

### 5. Cutwork
**What to look for:**
- Running stitch outline of area to be cut away
- Optional: "Bridge" or "bride" stitches across opening (decorative webbed effect)
- Satin stitch or buttonhole stitch around raw edges (seals the cut)
- Pattern: Running stitch → (optional bridges) → Satin stitch densely covering outline

**Confidence scoring:**
- Clear running outline + satin finish → 0.75-0.85
- With bridges → 0.80-0.90
- Weak pattern → <0.60

**Implementation:**
- Identify running stitch outlines
- Detect if followed by satin stitch on same boundary
- Look for bridge patterns (short stitches crossing openings)
- Advanced: Detect 4-angle blade pattern (0°, 45°, 90°, 135°) if possible
- Return confidence based on completeness of pattern

---

### 6. Outline Stitching
**What to look for:**
- Running stitch without any fills or dense areas
- May include bean stitch (forward-backward-forward pattern for each step)
- Only STITCH and JUMP commands; no STITCH_BLOCK markers, no dense fills
- No satin stitch present

**Confidence scoring:**
- Clear outline-only pattern → 0.80-0.90
- Some ambiguity (could be redwork) → 0.60-0.70
- Multiple color blocks of outlines → higher confidence

**Implementation:**
- Check for absence of STITCH_BLOCK
- Check for absence of satin stitch
- Verify stitches form connected paths (low jump-to-stitch ratio)
- Analyze stitch density (should be low/sparse)
- Return confidence based on clarity

---

### 7. Satin Stitch
**What to look for:**
- Series of parallel stitches going left-to-right or right-to-left in a column
- Column may be straight or curved
- Creates smooth, shiny, filled appearance
- Pattern: Consecutive STITCH commands with minimal X-variance (column-like) and increasing/decreasing Y

**Confidence scoring:**
- Clear parallel column pattern → 0.80-0.90
- Multiple satin blocks → higher confidence
- Curved/angled → still counts if parallel pattern maintained
- Unclear pattern → <0.60

**Implementation:**
- Analyze stitch coordinates for each color block
- Detect column-like patterns (low x-variance, sequential y changes)
- Check for parallelism (consistent spacing between stitches)
- Return confidence based on regularity and extent

---

### 8. Redwork
**Two-phase detection:**

**Phase 1 (Name-based — always checked):**
- Check if `filename` OR `folder_name` contains (case-insensitive): `"red work"`, `"redwork"`, `"red_work"`
- If found: Return confidence **0.95** (name match is very reliable)

**Phase 2 (Pattern-based, if name not found):**
**What to look for:**
- Single contrasting color on light background (traditionally red on white/cream)
- Composed almost entirely of running stitch, bean stitch, or very narrow satin stitch
- No solid filled areas
- Digitized as continuous path with minimal thread jumps/trims
- Very low thread usage

**Confidence scoring:**
- Single color + outline-only pattern + low jumps → 0.75-0.85
- Continuous path (minimal trims) → higher confidence
- Multiple colors → lower confidence or filtered out
- Clear pattern → 0.80+

**Implementation:**
- Always check keywords first in filename/folder, return 0.95 if match
- If no name-based match: Check thread count (should be 1, max 2)
- Verify absence of fills and solid areas
- Count JUMP/TRIM commands (should be low)
- Check stitch path continuity
- Return confidence based on pattern clarity and color count

---

### 9. Blackwork
**Two-phase detection:**

**Phase 1 (Name-based — always checked):**
- Check if `filename` OR `folder_name` contains (case-insensitive): `"black work"`, `"blackwork"`, `"black_work"`
- If found: Return confidence **0.95** (name match is very reliable)

**Phase 2 (Pattern-based, if name not found):**
**What to look for:**
- Intricate geometric patterns (diamonds, stars, honeycombs) filling shapes
- Uses motif fills in digital files
- Shading created by density/thickness of geometric pattern
- Dense grid = dark, sparse grid = light
- Sophisticated, architectural appearance
- Results in lace-like or etched appearance

**Confidence scoring:**
- Regular geometric patterns with density variation → 0.70-0.80
- Clear region boundaries between dense and sparse → higher confidence
- Ambiguous pattern → <0.60
- Single motif block → <0.60

**Implementation:**
- Always check keywords first in filename/folder, return 0.95 if match
- If no name-based match: Analyze stitch density in regions
- Look for geometric pattern regularity (diamonds, stars, honeycombs)
- Detect density variation within color block (dark vs. light areas)
- Check for organized geometric spacing
- Distinguish from filled stitch (which is less regular/geometric)
- Return confidence based on pattern complexity and regularity

**Challenges:** Hardest to distinguish from filled stitch; recommend 0.75+ confidence threshold to avoid false positives.

---

## Algorithm Workflow

```
identify_stitches():
  1. Initialize results list
   2. For each of 9 stitch types:
     - Call _detect_{type}() method
     - Receives confidence score (0.0-1.0)
  3. Filter: Keep only types with confidence >= confidence_threshold
  4. Sort results alphabetically
  5. Return sorted list of type names
```

### Example Execution
```
Pattern with cross stitch and filled areas:
- _detect_cross_stitch() returns 0.85 ✓ (kept)
- _detect_ith() returns 0.30 ✗ (filtered)
- _detect_applique() returns 0.25 ✗ (filtered)
- _detect_filled() returns 0.88 ✓ (kept)
- ...all others return <0.70...
Result: ['cross_stitch', 'filled']
```

---

## Dependencies & Imports

```python
from .EmbPattern import EmbPattern
from .EmbConstant import (
    STITCH, JUMP, TRIM, STOP, 
    COLOR_CHANGE, COMMAND_MASK
)
```

**No external dependencies** beyond pyembroidery core modules.

**Reusable existing code:**
- `pattern.get_as_colorblocks()` — iterate color blocks
- `pattern.get_as_stitchblock()` — detect stitch blocks
- `pattern.count_stitch_commands(cmd)` — count specific command types
- `pattern.bounds()` — get bounding box

**Implement locally for portability:**
- Geometric helpers: `_calculate_distance()`, `_calculate_angle()`
- (Reference [CsvWriter.py](CsvWriter.py) and [EmbFunctions.py](EmbFunctions.py) for patterns)

---

## Implementation Steps

### Phase 1: Foundation (Sequential)
1. Create `pyembroidery/StitchIdentifier.py` with class skeleton, imports, constructor
2. Implement utility helpers:
   - `_extract_color_blocks()`
   - `_extract_stitch_blocks()`
   - `_has_name_pattern(keywords)`
   - `_calculate_distance(dx, dy)`
   - `_calculate_angle(dx, dy)`

### Phase 2: Detection Methods (Can be done in parallel)
**Note:** Five types use name-based detection first (cross_stitch, ith, applique, redwork, blackwork) with 0.95 confidence if keywords found in filename/folder.

3. Implement `_detect_cross_stitch()` — includes name-based detection
4. Implement `_detect_ith()` — includes name-based detection
5. Implement `_detect_applique()` — includes name-based detection
6. Implement `_detect_filled()`
7. Implement `_detect_cutwork()`
8. Implement `_detect_outline()`
9. Implement `_detect_satin()`
10. Implement `_detect_redwork()` — includes name-based detection
11. Implement `_detect_blackwork()` — includes name-based detection

### Phase 3: Integration (Sequential)
13. Implement `identify_stitches()` main method
14. Implement `get_detailed_analysis()` for debugging
15. Add `StitchIdentifier` to `pyembroidery/__init__.py`

### Phase 4: Testing (Sequential)
16. Create `test/test_stitch_identifier.py`
17. Write unit tests for each detection method
18. Create synthetic test patterns for each stitch type using EmbPattern API
19. Write integration tests against existing embroidery files
20. Verify portability when copied to separate application

---

## Testing Strategy

### Unit Tests
**Per detection method:**
- Create synthetic patterns for each stitch type
- Test high-confidence case (returns 0.70+)
- Test low-confidence case (returns <0.70)
- Test edge cases (empty pattern, single stitch)

**Name-based detection (mandatory filename/folder):**
- Test ITH with various filename formats: `"ith"`, `"In The Hoop"`, `"hoop"` (0.95 confidence)
- Test appliqué with various filename formats: `"applique"`, `"appliqué"`, `"appique"` (0.95 confidence)
- Test cross stitch with various filename formats: `"cross stitch"`, `"cross-stitch"`, `"cross_stitch"` (0.95 confidence)
- Test redwork with various filename formats: `"red work"`, `"redwork"`, `"red_work"` (0.95 confidence)
- Test blackwork with various filename formats: `"black work"`, `"blackwork"`, `"black_work"` (0.95 confidence)
- Test folder name detection for all types
- Test case insensitivity for all keyword matching

**Confidence threshold:**
- Verify filtering works at default 0.70
- Verify custom thresholds work (0.60, 0.80, etc.)

### Integration Tests
- Run `identify_stitches()` against existing test files in `test/`
- Compare results with manual analysis
- Verify expected stitch types detected for known patterns

### Edge Cases
- Empty pattern (no stitches)
- Single stitch
- Pattern with multiple overlapping types
- Very large patterns (10,000+ stitches)
- Files with unusual naming (uppercase, mixed case, special characters)
- Partial matches in filenames (e.g., "my_cross_stitch_design.pes") — should match

### Portability Verification
- Copy `StitchIdentifier.py` to separate application
- Verify it works without requiring full pyembroidery context
- Ensure no hard-coded paths or file I/O

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Confidence scoring (0.0-1.0)** | Allows fine-grained filtering; each detector independently determines certainty |
| **Name-based detection first (mandatory)** | File/folder names are most reliable indicator for 5 types (ITH, appliqué, cross stitch, redwork, blackwork); always checked first with 0.95 confidence |
| **Multiple types allowed** | Real designs often combine techniques (e.g., outline + redwork) |
| **Configurable threshold** | Different apps may have different accuracy requirements |
| **Minimal dependencies** | Uses only EmbPattern/EmbConstant; easy to copy and reuse in other projects |
| **No file I/O** | Pure analysis module; no reading/writing files |

---

## Further Considerations

### Challenge: Blackwork vs. Filled
- Both use dense stitching
- Distinction: Blackwork has **geometric pattern regularity**, filled does not
- Recommend confidence threshold of **0.75+** to reduce confusion
- May need to implement sophisticated density analysis

### Performance Optimization
- For very large patterns (10,000+ stitches), iterating color blocks multiple times is expensive
- Consider caching `get_as_colorblocks()` and `get_as_stitchblock()` results in `__init__`
- Cache only if pattern size exceeds threshold (e.g., 5,000 stitches)

### Future Enhancements
- Add fuzzy matching for name detection (handle typos like "applique" vs "appliqué")
- Add machine learning model for difficult-to-distinguish types (blackwork)
- Add detailed stitch type breakdown (what % of design is each type?)
- Add visualization of detected patterns for debugging

---

## References

**Relevant pyembroidery files:**
- [EmbPattern.py](EmbPattern.py) — Pattern storage and analysis
- [EmbConstant.py](EmbConstant.py) — Stitch type constants
- [CsvWriter.py](CsvWriter.py) — Example geometric analysis
- [EmbFunctions.py](EmbFunctions.py) — Command encoding/decoding
- [ReadHelper.py](ReadHelper.py) — Utility functions for reading

**Test resources:**
- `test/pattern_for_tests.py` — Test pattern utilities
- `test/` directory — Existing test embroidery files

---

## Sign-Off

This plan is ready for review and refinement. Key points:
- ✅ 9 stitch types defined with detection algorithms
- ✅ Two-phase approach (name-based + pattern analysis)
- ✅ Configurable confidence threshold (default 70%)
- ✅ Portable design (easy to copy to other apps)
- ✅ Comprehensive testing strategy outlined
- ✅ Edge cases and performance considerations noted

**Feedback requested on:**
- Detection algorithm accuracy (especially blackwork)
- Confidence score ranges (are they appropriate?)
- Testing strategy completeness
- Any additional stitch types or modifications needed
