"""Auto-tagging service: Tier 1 (keyword) + Tier 2 (Gemini AI) tag suggestion."""

from __future__ import annotations

import logging
import re
import time
from dataclasses import dataclass
from dataclasses import field as dc_field
from pathlib import Path
from typing import TYPE_CHECKING, Any

import pyembroidery

if TYPE_CHECKING:
    from src.models import Tag

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Tier 1 — keyword → tag description
# ---------------------------------------------------------------------------
# Keys are lowercase substrings to look for in the filename stem (after
# stripping path and extension). Values are tag descriptions that
# MUST match exactly what is stored in the ``tags`` table
# (accessible via the ``Tag`` model alias in new code).

KEYWORD_MAP: dict[str, str] = {
    # Alphabets & Letters
    "alphabet": "Alphabets",
    "monogram": "Monogram",
    # Animals (general before specific)
    "wildlife": "Animals",
    "animal": "Animals",
    "rabbit": "Animals",
    "bunny": "Easter",  # bunny → Easter more often than generic Animals
    "squirrel": "Animals",
    "fox": "Animals",
    "deer": "Animals",
    "frog": "Animals",
    "owl": "Birds",
    "penguin": "Birds",
    "parrot": "Birds",
    "swan": "Birds",
    "robin": "Birds",
    "bird": "Birds",
    "feather": "Birds",
    "cat": "Cats",
    "kitten": "Cats",
    "kitty": "Cats",
    "feline": "Cats",
    "dog": "Dogs",
    "puppy": "Dogs",
    "poodle": "Dogs",
    "labrador": "Dogs",
    "spaniel": "Dogs",
    "dachshund": "Dogs",
    "horse": "Horses",
    "pony": "Horses",
    "foal": "Horses",
    "equine": "Horses",
    "butterfly": "Butterflies and Insects",
    "dragonfly": "Butterflies and Insects",
    "ladybug": "Butterflies and Insects",
    "ladybird": "Butterflies and Insects",
    "insect": "Butterflies and Insects",
    "bee": "Butterflies and Insects",
    "fish": "Fish and Seashells",
    "seahorse": "Fish and Seashells",
    "shell": "Fish and Seashells",
    "starfish": "Fish and Seashells",
    "crab": "Fish and Seashells",
    "dolphin": "Fish and Seashells",
    "whale": "Fish and Seashells",
    # Seasonal / Holidays
    "christmas": "Christmas",
    "xmas": "Christmas",
    "santa": "Christmas",
    "reindeer": "Christmas",
    "rudolph": "Christmas",
    "snowman": "Christmas",
    "snowflake": "Christmas",
    "mistletoe": "Christmas",
    "poinsettia": "Christmas",
    "bauble": "Christmas",
    "nativity": "Christmas",
    "easter": "Easter",
    "easter_egg": "Easter",
    "chick": "Easter",
    "halloween": "Halloween",
    "pumpkin": "Halloween",
    "witch": "Halloween",
    "skeleton": "Halloween",
    "vampire": "Halloween",
    "ghost": "Ghosts",
    "valentine": "Valentine's Day",
    "thanksgiving": "Thanksgiving",
    "turkey": "Thanksgiving",
    "pilgrim": "Thanksgiving",
    "harvest": "Thanksgiving",
    "diwali": "Diwali",
    "hanukkah": "Hanukkah",
    "menorah": "Hanukkah",
    "eid": "Eid",
    "ramadan": "Eid",
    # Nature
    "flower": "Flowers",
    "floral": "Flowers",
    "rose": "Flowers",
    "tulip": "Flowers",
    "daisy": "Flowers",
    "lily": "Flowers",
    "sunflower": "Flowers",
    "orchid": "Flowers",
    "pansy": "Flowers",
    "hibiscus": "Flowers",
    "poppy": "Flowers",
    "lavender": "Flowers",
    "garden": "Garden",
    "tree": "Trees",
    "oak": "Trees",
    "pine": "Trees",
    "palm": "Trees",
    "wreath": "Wreaths",
    "garland": "Wreaths",
    "leaf": "Garden",
    "paisley": "Paisley",
    "landscape": "Landscapes and Travel",
    "travel": "Landscapes and Travel",
    "map": "Landscapes and Travel",
    # Celestial
    "zodiac": "Zodiac",
    "horoscope": "Zodiac",
    "aries": "Zodiac",
    "taurus": "Zodiac",
    "gemini": "Zodiac",
    "cancer": "Zodiac",
    "leo": "Zodiac",
    "virgo": "Zodiac",
    "libra": "Zodiac",
    "scorpio": "Zodiac",
    "sagittarius": "Zodiac",
    "capricorn": "Zodiac",
    "aquarius": "Zodiac",
    "pisces": "Zodiac",
    # People & Characters
    "angel": "Angels",
    "cherub": "Angels",
    "fairy": "Fairies, Elves etc.",
    "faerie": "Fairies, Elves etc.",
    "elf": "Fairies, Elves etc.",
    "gnome": "Fairies, Elves etc.",
    "pixie": "Fairies, Elves etc.",
    "sprite": "Fairies, Elves etc.",
    "baby": "Babies",
    "infant": "Babies",
    "nursery": "Babies",
    "toddler": "Babies",
    "child": "Children",
    "children": "Children",
    "cartoon": "Cartoon",
    "face": "Faces",
    "portrait": "Faces",
    "people": "People and Work",
    "person": "People and Work",
    "lady": "People and Work",
    "woman": "People and Work",
    "man": "People and Work",
    # Fantasy
    "dragon": "Fantasy",
    "unicorn": "Fantasy",
    "fantasy": "Fantasy",
    "mermaid": "Fantasy",
    "wizard": "Fantasy",
    "fairies": "Fairies, Elves etc.",
    # Celebrations / Events
    "wedding": "Wedding",
    "bride": "Wedding",
    "bridal": "Wedding",
    "groom": "Wedding",
    "celebration": "Celebrations",
    "birthday": "Celebrations",
    "party": "Celebrations",
    "balloon": "Celebrations",
    "mother": "Mother's Day",
    "mum": "Mother's Day",
    "mom": "Mother's Day",
    "father": "Father's Day",
    "dad": "Father's Day",
    # Religious
    "cross": "Religious",
    "religious": "Religious",
    "faith": "Religious",
    "church": "Religious",
    "bible": "Religious",
    "prayer": "Religious",
    "jesus": "Religious",
    "vintage": "Sketchy and Vintage",
    "sketchy": "Sketchy and Vintage",
    "sketch": "Sketchy and Vintage",
    # Transport
    "transport": "Transport",
    "car": "Transport",
    "truck": "Transport",
    "train": "Transport",
    "plane": "Transport",
    "aeroplane": "Transport",
    "airplane": "Transport",
    "helicopter": "Transport",
    "tractor": "Transport",
    "bike": "Transport",
    "bicycle": "Transport",
    "motorcycle": "Transport",
    # Nautical
    "nautical": "Nautical",
    "anchor": "Nautical",
    "ship": "Nautical",
    "sailboat": "Nautical",
    "lighthouse": "Nautical",
    "sea": "Nautical",
    "ocean": "Nautical",
    "wave": "Nautical",
    "rope": "Nautical",
    # Music
    "music": "Music",
    "note": "Music",
    "guitar": "Music",
    "piano": "Music",
    "violin": "Music",
    "trumpet": "Music",
    "harp": "Music",
    # Sport
    "sport": "Sport",
    "football": "Sport",
    "soccer": "Sport",
    "baseball": "Sport",
    "basketball": "Sport",
    "tennis": "Sport",
    "golf": "Sport",
    "hockey": "Sport",
    "cricket": "Sport",
    "rugby": "Sport",
    "swimming": "Sport",
    "cycling": "Sport",
    # Food
    "food": "Food",
    "cake": "Food",
    "cupcake": "Food",
    "fruit": "Food",
    "vegetable": "Food",
    "coffee": "Food",
    "tea": "Food",
    "wine": "Food",
    "strawberry": "Food",
    "cherry": "Food",
    "apple": "Food",
    # Borders / Frames / Layout
    "banner": "Banners",
    "border": "Borders",
    "corner": "Corners",
    "frame": "Frames",
    "scroll": "Scrolls",
    # Decorative styles
    "celtic": "Celtic and Tribal",
    "tribal": "Celtic and Tribal",
    "ornament": "Ornaments",
    "motif": "Patterns",
    "pattern": "Patterns",
    # Fashion / Clothing
    "fashion": "Fashion",
    "shoe": "Footwear",
    "boot": "Footwear",
    "heel": "Footwear",
    "flip_flop": "Footwear",
    "stiletto": "Footwear",
    "bag": "Handbags",
    "purse": "Handbags",
    "handbag": "Handbags",
    "collar": "Collars",
    "jewel": "Jewellery",
    "necklace": "Jewellery",
    "crown": "Jewellery",
    "ring": "Jewellery",
    # Buildings
    "house": "Buildings and Structures",
    "building": "Buildings and Structures",
    "castle": "Buildings and Structures",
    "cottage": "Buildings and Structures",
    "barn": "Buildings and Structures",
    "windmill": "Buildings and Structures",
    # Household
    "household": "Household",
    "kitchen": "Household",
    "teapot": "Household",
    "teacup": "Household",
    "chair": "Household",
    # Toys
    "toy": "Toys",
    "teddy": "Toys",
    "doll": "Toys",
    "rocking_horse": "Toys",
    # Winter
    "winter": "Winter",
    "snow": "Winter",
    "icicle": "Winter",
    # Hobbies
    "hobby": "Hobbies",
    "knitting": "Hobbies",
    "sewing": "Hobbies",
    "gardening": "Hobbies",
    "fishing": "Hobbies",
    "reading": "Hobbies",
    "cooking": "Hobbies",
    # Words
    "word": "Words and Letters",
    "saying": "Words and Letters",
    "quote": "Words and Letters",
    "text": "Words and Letters",
    # Ghosts / Monsters
    "monster": "Monsters",
    "zombie": "Monsters",
    "creature": "Monsters",
    # Silhouette / Line
    "silhouette": "Silhouette",
    # Badges / Crests / Flags
    "badge": "Badges and Crests",
    "crest": "Crests",
    "flag": "Flags",
    "coat_of_arms": "Badges and Crests",
    # Hobbies (job / work themed)
    "job": "Job",
    "profession": "Job",
    "nurse": "Job",
    "doctor": "Job",
    "teacher": "Job",
    # Sun / Moon / Stars (not Zodiac specific)
    "star": "Sun Moon and Stars",
    "moon": "Sun Moon and Stars",
    "heart": "Hearts and Lips",
    "lip": "Hearts and Lips",
    "kiss": "Hearts and Lips",
}

# ---------------------------------------------------------------------------
# StitchIdentifier integration — pattern-based stitch-type analysis
# ---------------------------------------------------------------------------
# Maps StitchIdentifier internal names to the app's tag descriptions.
_STITCH_TYPE_TO_TAG: dict[str, str] = {
    "applique": "Applique",
    "cross_stitch": "Cross Stitch",
    "cutwork": "Cutwork",
    "filled": "Filled",
    "ith": "In The Hoop",
    "lace": "Lace",
    "outline": "Line Outline",
    "satin": "Satin Stitch",
}


def suggest_stitching_from_pattern(
    pattern_path: str,
    filename: str,
    filepath: str,
    desc_to_tag: dict[str, Tag],
    confidence_threshold: float = 0.70,
    pattern: Any = None,
) -> list[str]:
    """Analyse an embroidery pattern file to detect stitch types.

    Uses :class:`StitchIdentifier` to examine the actual stitch geometry
    (vectors, angles, densities) and returns matching tag descriptions for
    the ``stitching`` tag group.

    Args:
        pattern_path: Full filesystem path to the embroidery file.
        filename: The design's filename (used for name-based detection).
        filepath: The design's stored filepath (folder name extracted for
                  name-based detection).
        desc_to_tag: Mapping of tag description → Tag, used to verify that
                     each detected type corresponds to a known tag.
        confidence_threshold: Minimum confidence for StitchIdentifier
                              (default 0.70).
        pattern: Optional pre-read pyembroidery pattern object. If provided,
                 the file is not re-read from disk.

    Returns:
        Sorted list of tag descriptions (e.g. ``["Filled", "Satin Stitch"]``).
        Empty list if the file cannot be read or no stitch types are detected.
    """
    from src.services.stitch_identifier import StitchIdentifier

    if pattern is None:
        try:
            pattern = pyembroidery.read(pattern_path)
        except Exception:
            logger.warning("Could not read pattern file %r for stitch analysis", pattern_path)
            return []

        if pattern is None:
            logger.warning(
                "Pattern file %r returned None (unsupported or corrupt format)", pattern_path
            )
            return []

    folder_name = str(Path(filepath).parent) if filepath else ""
    identifier = StitchIdentifier(pattern, filename, folder_name, confidence_threshold)
    detected = identifier.identify_stitches()

    if not detected:
        return []

    matched: list[str] = []
    for stitch_type in detected:
        tag_desc = _STITCH_TYPE_TO_TAG.get(stitch_type)
        if tag_desc and tag_desc in desc_to_tag:
            matched.append(tag_desc)

    return sorted(matched)


def _stem(filename: str) -> str:
    """Return lowercase filename stem (no path, no extension)."""
    return Path(filename).stem.lower()


def suggest_tier1(filename: str, valid_descriptions: set[str], filepath: str = "") -> list[str]:
    """Return a list of matching tag descriptions using keyword matching.

    Matches against the filename stem as well as any folder names in *filepath*,
    so that a file like ``\\Alphabets\\font pooh\\Poeh-Y.pes`` picks up the
    'Alphabets' tag from the folder name even when the filename itself has no
    recognisable keyword.
    """
    stem = _stem(filename)
    # tokenise on non-alpha boundaries so we get individual words
    tokens = set(re.split(r"[^a-z]+", stem)) - {""}

    # Also tokenise every folder component of the filepath
    if filepath:
        path_lower = filepath.lower()
        path_tokens = set(re.split(r"[^a-z]+", path_lower)) - {""}
        tokens |= path_tokens
        stem = stem + " " + path_lower  # allow substring matches against folder names too

    matched: set[str] = set()

    for keyword, description in KEYWORD_MAP.items():
        if description not in valid_descriptions:
            continue
        # check both substring match and token match
        if keyword in stem or keyword.replace("_", "") in stem or keyword in tokens:
            matched.add(description)

    return sorted(matched)


def suggest_tier2_batch(
    filenames: list[str],
    valid_descriptions: list[str],
    api_key: str,
    batch_size: int = 20,
    delay_seconds: float = 5.0,
    max_retries: int = 3,
) -> dict[str, list[str]]:
    """Send batches of filenames to Gemini and return tag suggestions.

    Adds a delay between calls to stay within the free-tier 15 RPM limit.
    Returns a dict mapping filename stem → list[description].
    """
    from src.services.gemini_client import call_gemini_text

    categories_str = "\n".join(f"- {d}" for d in sorted(valid_descriptions))
    valid_set = set(valid_descriptions)

    results: dict[str, list[str]] = {}
    total_batches = (len(filenames) + batch_size - 1) // batch_size

    for batch_num, i in enumerate(range(0, len(filenames), batch_size), start=1):
        batch = filenames[i : i + batch_size]
        stems = [_stem(f) for f in batch]
        stems_str = "\n".join(f"- {s}" for s in stems)

        prompt = f"""You are categorising embroidery design files by filename.

For each filename below, suggest which categories from the provided list apply.
Return ONLY a valid JSON object where each key is a filename stem and each value
is a list of matching category names (use an empty list [] if none apply).
Only use category names from the exact list provided.

CATEGORIES:
{categories_str}

FILENAMES:
{stems_str}

Respond with ONLY the JSON object, no explanation."""

        try:
            batch_result: dict[str, list[str]] = call_gemini_text(
                prompt,
                api_key=api_key,
                max_retries=max_retries,
                delay_seconds=delay_seconds,
            )
            for stem, tags in batch_result.items():
                results[stem] = [t for t in tags if t in valid_set]
            logger.info("Batch %d/%d OK (%d files)", batch_num, total_batches, len(batch))
        except Exception as exc:
            logger.error("Batch %d/%d skipped: %s", batch_num, total_batches, exc)
            for stem in stems:
                results.setdefault(stem, [])

        # Pause between calls to respect rate limits
        if i + batch_size < len(filenames):
            time.sleep(delay_seconds)

    return results


# ---------------------------------------------------------------------------
# Tier 3 — Gemini vision AI on stored PNG preview images
# ---------------------------------------------------------------------------


def suggest_tier3_vision(
    designs: list,
    valid_descriptions: list[str],
    api_key: str,
    delay_seconds: float = 2.0,
    max_retries: int = 3,
) -> dict[int, list[str]]:
    """Send stored PNG preview images to Gemini vision model for tagging.

    Args:
        designs: list of Design ORM objects; each must have .id, .filename,
                 and .image_data (PNG bytes). Designs where image_data is None
                 are skipped.
        valid_descriptions: list of all known tag descriptions.
        api_key: Google AI API key.
        delay_seconds: pause between API calls (default 2s — paid tier).
        max_retries: number of retry attempts on failure.

    Returns:
        dict mapping design.id → list[description].
    """
    from src.services.gemini_client import call_gemini_vision

    categories_str = "\n".join(f"- {d}" for d in sorted(valid_descriptions))
    prompt = (
        "You are categorising machine embroidery designs.\n"
        "Below is a preview image of an embroidery design (shown as stitches on a plain background).\n"
        "Choose ALL tags from this list that accurately describe the design.\n\n"
        f"{categories_str}\n\n"
        'Respond with ONLY a JSON array of matching tag names, e.g. ["Flowers", "Birds"].\n'
        "If nothing matches, respond with [].\n"
        "Use exact tag names from the list above."
    )

    valid_set = set(valid_descriptions)
    results: dict[int, list[str]] = {}
    candidates = [d for d in designs if d.image_data is not None]
    skipped = len(designs) - len(candidates)
    if skipped:
        logger.info("Tier 3: skipping %d designs with no image data.", skipped)

    total = len(candidates)
    for idx, design in enumerate(candidates, start=1):
        if idx % 10 == 0 or idx == 1 or idx == total:
            logger.debug("Vision [%d/%d] %s", idx, total, design.filename)

        try:
            tags: list[str] = call_gemini_vision(
                design.image_data,
                prompt,
                api_key=api_key,
                max_retries=max_retries,
                delay_seconds=delay_seconds,
            )
            results[design.id] = [t for t in tags if t in valid_set]
        except Exception as exc:
            logger.error("Vision [%d/%d] %s: gave up — %s", idx, total, design.filename, exc)
            results[design.id] = []

        # Pause between calls
        if idx < total:
            time.sleep(delay_seconds)

    return results


# ---------------------------------------------------------------------------
# High-level tagging action — used by the in-app Tagging Actions UI
# ---------------------------------------------------------------------------


@dataclass
class TaggingActionResult:
    """Summary returned after running a batch tagging action."""

    action: str
    tiers_run: list[int]
    designs_considered: int = 0
    tier1_tagged: int = 0
    tier2_tagged: int = 0
    tier3_tagged: int = 0
    total_tagged: int = 0
    still_untagged: int = 0
    already_matched: int = 0
    no_match: int = 0
    cleared_only: int = 0
    tag_breakdown: dict[str, int] = dc_field(default_factory=dict)
    errors: list[str] = dc_field(default_factory=list)


def _resolve_design_filepath(design_filepath: str, designs_base_path: str) -> str | None:
    """Return the full filesystem path to an embroidery pattern file.

    Design filepaths are stored relative to the managed storage root
    (``DESIGNS_BASE_PATH``).  This helper rebuilds the absolute path so the
    file can be opened for pattern analysis.
    """
    import os

    rel = design_filepath.lstrip("/\\")
    full = os.path.normpath(os.path.join(designs_base_path, rel))
    if os.path.isfile(full):
        return full
    logger.debug("Pattern file not found at %r (stored filepath: %r)", full, design_filepath)
    return None


def run_stitching_backfill_action(
    db,
    batch_size: int | None = None,
    dry_run: bool = False,
    allowed_descriptions: set[str] | list[str] | tuple[str, ...] | None = None,
    clear_existing_stitching: bool = False,
) -> TaggingActionResult:
    """Backfill stitching tags for existing unverified designs using pattern analysis.

    Uses :class:`StitchIdentifier` to examine the actual stitch geometry of each
    design's embroidery file.  Only tags in the ``stitching`` tag group are
    considered; existing non-stitching tags are preserved.

    The action is intentionally scoped to designs whose tags have **not** been
    marked as verified by the user.
    """
    from src.models import Design, Tag
    from src.services.tagging import _unique_tags_from_descriptions
    from src.services.unified_backfill import is_stop_requested

    result = TaggingActionResult(action="backfill_stitching", tiers_run=[1])
    all_tags: list[Tag] = db.query(Tag).order_by(Tag.description).all()
    desc_to_tag: dict[str, Tag] = {tag.description: tag for tag in all_tags}

    allowed_set = (
        {desc.strip() for desc in allowed_descriptions if str(desc).strip()}
        if allowed_descriptions
        else set()
    )

    if not any(getattr(tag, "tag_group", None) == "stitching" for tag in all_tags):
        result.errors.append("No stitching tags are configured.")
        return result

    if allowed_set and not any(
        getattr(tag, "tag_group", None) == "stitching" and tag.description in allowed_set
        for tag in all_tags
    ):
        result.errors.append(
            "None of the requested backfill tags are configured: " + ", ".join(sorted(allowed_set))
        )
        return result

    from src.config import DESIGNS_BASE_PATH

    try:
        total_designs_considered = 0
        from sqlalchemy import and_, exists, select

        from src.models import design_tags

        overwrite = clear_existing_stitching
        iteration = 0
        # Track processed design IDs to prevent re-processing in subsequent iterations.
        # This is essential when clear_existing_stitching=True (no subquery to exclude
        # already-processed designs) and also acts as a safety net for the normal path.
        processed_design_ids: set[int] = set()
        while True:
            iteration += 1
            logger.info("[Stitching Backfill] Iteration %d", iteration)
            # Build the base filter: only unverified designs
            base_filter = Design.tags_checked.is_(False) | Design.tags_checked.is_(None)
            # Exclude already-processed designs to prevent infinite loops
            if processed_design_ids:
                base_filter = base_filter & Design.id.notin_(processed_design_ids)
            # Only select designs that do NOT already have a stitching tag, unless overwriting
            if not overwrite:
                # Subquery: design has a stitching tag
                stitching_tag_ids = [
                    tag.id for tag in all_tags if getattr(tag, "tag_group", None) == "stitching"
                ]
                if stitching_tag_ids:
                    subq = (
                        select(1)
                        .select_from(design_tags.join(Tag, design_tags.c.tag_id == Tag.id))
                        .where(
                            and_(
                                design_tags.c.design_id == Design.id, Tag.id.in_(stitching_tag_ids)
                            )
                        )
                    )
                    query = db.query(Design).filter(base_filter & ~exists(subq)).order_by(Design.id)
                else:
                    logger.warning(
                        "[Stitching Backfill] No stitching tags found; skipping subquery."
                    )
                    query = db.query(Design).filter(base_filter).order_by(Design.id)
            else:
                query = db.query(Design).filter(base_filter).order_by(Design.id)
            if batch_size and batch_size > 0:
                query = query.limit(batch_size)
            try:
                sql_str = str(query.statement.compile(compile_kwargs={"literal_binds": True}))
                logger.info("[Stitching Backfill] SQL: %s", sql_str)
            except Exception as e:
                logger.warning("[Stitching Backfill] Could not log SQL statement: %s", e)
            designs = query.all()
            logger.info("[Stitching Backfill] Fetched %d designs", len(designs))
            total_designs_considered += len(designs)
            if not designs:
                logger.info(
                    "[Stitching Backfill] No more designs to process. Stopping at iteration %d.",
                    iteration,
                )
                break

            # Process each design in this batch
            for i, design in enumerate(designs, 1):
                matched_descriptions: list[str] = []
                pattern_path = _resolve_design_filepath(design.filepath, DESIGNS_BASE_PATH)
                if pattern_path:
                    try:
                        pattern_descriptions = suggest_stitching_from_pattern(
                            pattern_path, design.filename, design.filepath, desc_to_tag
                        )
                        if pattern_descriptions:
                            matched_descriptions = sorted(pattern_descriptions)
                    except Exception as e:
                        logger.error(
                            "[Stitching Backfill] Error processing design %s: %s",
                            getattr(design, "filename", None),
                            e,
                        )
                else:
                    logger.warning(
                        "[Stitching Backfill] Could not resolve pattern path for design %s",
                        design.filename,
                    )

                non_stitching_tags = [
                    tag for tag in design.tags if getattr(tag, "tag_group", None) != "stitching"
                ]
                if not matched_descriptions:
                    if clear_existing_stitching:
                        current_ids = {getattr(tag, "id", None) for tag in design.tags}
                        cleared_ids = {getattr(tag, "id", None) for tag in non_stitching_tags}
                        if current_ids != cleared_ids:
                            result.cleared_only += 1
                            if not dry_run:
                                design.tags = non_stitching_tags
                                design.tags_checked = False
                                design.tagging_tier = None
                        else:
                            result.no_match += 1
                    else:
                        result.no_match += 1
                    result.still_untagged += 1
                    continue

                for description in matched_descriptions:
                    result.tag_breakdown[description] = result.tag_breakdown.get(description, 0) + 1

                matched_tags = _unique_tags_from_descriptions(matched_descriptions, desc_to_tag)
                replacement_tags = non_stitching_tags + [
                    tag for tag in matched_tags if tag not in non_stitching_tags
                ]

                current_ids = {getattr(tag, "id", None) for tag in design.tags}
                replacement_ids = {getattr(tag, "id", None) for tag in replacement_tags}
                if current_ids == replacement_ids:
                    result.already_matched += 1
                    continue

                result.tier1_tagged += 1
                result.total_tagged += 1

                if not dry_run:
                    design.tags = replacement_tags
                    design.tags_checked = False
                    design.tagging_tier = 1

            # Track all processed design IDs to prevent re-processing
            for design in designs:
                processed_design_ids.add(design.id)

            # Commit after each batch iteration so the next query
            # sees updated tags and doesn't re-process the same designs.
            if not dry_run:
                db.commit()

            # Check for stop signal after each batch commit
            if is_stop_requested():
                logger.info("[Stitching Backfill] Stop signal detected — committing and exiting.")
                if not dry_run:
                    db.commit()
                break
    except Exception as exc:
        import traceback

        logger.error(f"[Stitching Backfill] UNEXPECTED ERROR: {exc}\n{traceback.format_exc()}")
        raise

    result.designs_considered = total_designs_considered
    if not dry_run:
        db.commit()
    return result


def run_tagging_action(
    db,
    action: str,
    tiers: list[int],
    api_key: str,
    batch_size: int | None = None,
    delay: float = 5.0,
    vision_delay: float = 2.0,
    overwrite_verified: bool = False,
    dry_run: bool = False,
    design_ids: list[int] | None = None,
) -> TaggingActionResult:
    """Run a tagging action on existing designs in the database.

    Args:
        db: SQLAlchemy session.
        action: One of ``"tag_untagged"``, ``"retag_all_unverified"``,
            ``"retag_all"``.
        tiers: List of tier numbers to run, e.g. ``[1, 2]`` or ``[1, 2, 3]``.
        api_key: Google AI API key (required for tiers 2 and 3).
        batch_size: Maximum number of designs to process (``None`` = all).
        delay: Seconds between Gemini text calls (tier 2).
        vision_delay: Seconds between Gemini vision calls (tier 3).
        overwrite_verified: Deprecated — kept for backward compatibility.
            Use ``"retag_all"`` (includes verified) or ``"retag_all_unverified"``
            (skips verified) instead.
        dry_run: If ``True``, compute tag suggestions but do not write to the DB.
        design_ids: Optional pre-filtered list of design IDs to process.
            When provided, skips the database query for selecting designs.

    Returns:
        A :class:`TaggingActionResult` summarising what happened.
    """
    from src.models import Design, Tag
    from src.services.unified_backfill import is_stop_requested

    result = TaggingActionResult(action=action, tiers_run=list(tiers))

    # --- Load tag vocabulary ---
    all_tags: list[Tag] = db.query(Tag).order_by(Tag.description).all()
    desc_to_id: dict[str, int] = {tag.description: tag.id for tag in all_tags}
    # Only use image-group tags for AI tagging (stitching tags are handled
    # separately by StitchIdentifier pattern analysis).
    image_tag_descriptions = [tag.description for tag in all_tags if tag.tag_group == "image"]
    valid_descriptions = image_tag_descriptions or list(desc_to_id.keys())
    valid_set = set(valid_descriptions)

    # --- Select designs based on action ---
    if design_ids is not None:
        # Pre-filtered list provided by caller — skip the query
        designs = db.query(Design).filter(Design.id.in_(design_ids)).order_by(Design.filename).all()
        result.designs_considered = len(designs)
        if not designs:
            return result
    else:
        query = db.query(Design)

        if action == "tag_untagged":
            # Only designs that have no tags in the "image" tag group
            # (they may have "stitching" tags — those are unrelated to AI keyword tagging)
            image_tag_ids = [t.id for t in all_tags if t.tag_group == "image"]
            if image_tag_ids:
                query = query.filter(~Design.tags.any(Tag.id.in_(image_tag_ids)))
            # If there are no image tags at all, no designs match
        elif action == "retag_all":
            # All designs — overwrite everything, including verified
            pass
        elif action == "retag_all_unverified":
            # All unverified designs — overwrite their tags
            query = query.filter(Design.tags_checked.is_(False) | Design.tags_checked.is_(None))
        else:
            result.errors.append(f"Unknown action: {action!r}")
            return result

        # Log the SQL statement for diagnostics
        try:
            sql_str = str(
                query.order_by(Design.filename).statement.compile(
                    compile_kwargs={"literal_binds": True}
                )
            )
            logger.info(f"[Tagging] SQL used to select designs: {sql_str}")
        except Exception as e:
            logger.warning(f"[Tagging] Could not log SQL statement: {e}")

        designs: list[Design] = query.order_by(Design.filename).all()

    # Apply batch_size limit
    if batch_size and batch_size > 0:
        designs = designs[:batch_size]

    result.designs_considered = len(designs)

    if not designs:
        return result

    # --- Tier 1 ---
    tier1_results: dict[int, list[str]] = {}  # design.id → descriptions
    needs_tier2: list[Design] = []

    if 1 in tiers:
        for design in designs:
            if is_stop_requested():
                logger.info("[Tagging] Stop signal detected during Tier 1 — exiting.")
                if not dry_run:
                    db.commit()
                return result
            matched = suggest_tier1(design.filename, valid_set)
            tier1_results[design.id] = matched
            if not matched:
                needs_tier2.append(design)
    else:
        needs_tier2 = list(designs)

    # Check stop signal before Tier 2
    if is_stop_requested():
        logger.info("[Tagging] Stop signal detected before Tier 2 — exiting.")
        if not dry_run:
            db.commit()
        return result

    # --- Tier 2 ---
    tier2_results: dict[str, list[str]] = {}  # filename stem → descriptions
    needs_tier3: list[Design] = []

    if 2 in tiers and needs_tier2:
        if not api_key:
            result.errors.append("No Google API key configured — Tier 2 skipped.")
            needs_tier3 = [d for d in needs_tier2 if not tier1_results.get(d.id)]
        else:
            filenames = [d.filename for d in needs_tier2]
            try:
                tier2_results = suggest_tier2_batch(
                    filenames,
                    valid_descriptions,
                    api_key,
                    batch_size=20,
                    delay_seconds=delay,
                )
            except Exception as exc:  # noqa: BLE001
                logger.exception(
                    "Tier 2 tagging failed for %d design(s); continuing to Tier 3 where possible.",
                    len(needs_tier2),
                )
                result.errors.append(f"Tier 2 error: {exc}")
            needs_tier3 = [d for d in needs_tier2 if not tier2_results.get(_stem(d.filename))]
    elif 2 not in tiers:
        needs_tier3 = [d for d in needs_tier2 if not tier1_results.get(d.id)]

    # Check stop signal before Tier 3
    if is_stop_requested():
        logger.info("[Tagging] Stop signal detected before Tier 3 — exiting.")
        if not dry_run:
            db.commit()
        return result

    # --- Tier 3 ---
    tier3_results: dict[int, list[str]] = {}  # design.id → descriptions

    if 3 in tiers and needs_tier3:
        if not api_key:
            result.errors.append("No Google API key configured — Tier 3 skipped.")
        else:
            candidates = [d for d in needs_tier3 if d.image_data is not None]
            if candidates:
                try:
                    tier3_results = suggest_tier3_vision(
                        candidates,
                        valid_descriptions,
                        api_key,
                        delay_seconds=vision_delay,
                    )
                except Exception as exc:  # noqa: BLE001
                    logger.exception(
                        "Tier 3 tagging failed for %d design(s); leaving them untagged.",
                        len(candidates),
                    )
                    result.errors.append(f"Tier 3 error: {exc}")

    # --- Apply results ---
    for design in designs:
        # Check for stop signal before each design
        if is_stop_requested():
            logger.info("[Tagging] Stop signal detected — committing and exiting.")
            if not dry_run:
                db.commit()
            break

        descriptions: list[str] = tier1_results.get(design.id, [])
        tier: int | None = None
        if descriptions:
            tier = 1
            result.tier1_tagged += 1

        if not descriptions:
            descriptions = tier2_results.get(_stem(design.filename), [])
            if descriptions:
                tier = 2
                result.tier2_tagged += 1

        if not descriptions:
            descriptions = tier3_results.get(design.id, [])
            if descriptions:
                tier = 3
                result.tier3_tagged += 1

        tag_ids = [desc_to_id[d] for d in descriptions if d in desc_to_id]

        if descriptions:
            result.total_tagged += 1
        else:
            result.still_untagged += 1

        if not dry_run:
            if tag_ids:
                if action in ("retag_all", "retag_all_unverified"):
                    # Overwrite existing tags
                    design.tags = db.query(Tag).filter(Tag.id.in_(tag_ids)).all()
                    design.tagging_tier = tier
                    design.tags_checked = False
                elif action == "tag_untagged":
                    # Only set tags if design currently has none
                    if not design.tags:
                        design.tags = db.query(Tag).filter(Tag.id.in_(tag_ids)).all()
                        design.tagging_tier = tier
                        design.tags_checked = False
            elif action == "tag_untagged":
                # Always mark as checked=False for newly processed designs
                design.tags_checked = False

    if not dry_run:
        db.commit()

    return result
