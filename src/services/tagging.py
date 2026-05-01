"""
Tagging helpers and AI integration for embroidery import.
"""

import logging
from pathlib import Path

from src.models import Tag

logger = logging.getLogger(__name__)

DEFAULT_TAGGING_COMMIT_BATCH_SIZE = 1000


def _coerce_batch_size(value: int | None, default: int = DEFAULT_TAGGING_COMMIT_BATCH_SIZE) -> int:
    """Return a safe positive batch size."""
    if value is None:
        return default
    try:
        parsed = int(value)
    except (TypeError, ValueError):
        return default
    return parsed if parsed > 0 else default


def _unique_tags_from_descriptions(
    descriptions: list[str], desc_to_tag: dict[str, Tag]
) -> list[Tag]:
    """Return a list of unique Tag objects for the given descriptions."""
    seen = set()
    tags = []
    for desc in descriptions:
        tag = desc_to_tag.get(desc)
        if tag and tag.id not in seen:
            tags.append(tag)
            seen.add(tag.id)
    return tags


def _apply_tier2_tags(
    db,
    created: list,
    desc_to_tag: dict[str, Tag],
    valid_descriptions_list: list[str],
    api_key: str,
    commit_batch_size: int = DEFAULT_TAGGING_COMMIT_BATCH_SIZE,
) -> None:
    from src.services.auto_tagging import suggest_tier2_batch

    needs_tier2 = [d for d in created if not d.tags]
    if not needs_tier2:
        return
    logger.info("Tier 2: Gemini filename AI on %d untagged designs", len(needs_tier2))
    batch_size = _coerce_batch_size(commit_batch_size)
    for start in range(0, len(needs_tier2), batch_size):
        batch = needs_tier2[start : start + batch_size]
        filenames = [d.filename for d in batch]
        tier2_results = suggest_tier2_batch(filenames, valid_descriptions_list, api_key)
        updated = 0
        for design in batch:
            stem = Path(design.filename).stem.lower()
            descriptions = tier2_results.get(stem, [])
            if descriptions:
                design.tags = _unique_tags_from_descriptions(descriptions, desc_to_tag)
                design.tagging_tier = 2
                updated += 1
        if updated:
            db.commit()


def _apply_tier3_tags(
    db,
    created: list,
    desc_to_tag: dict[str, Tag],
    valid_descriptions_list: list[str],
    api_key: str,
    commit_batch_size: int = DEFAULT_TAGGING_COMMIT_BATCH_SIZE,
) -> None:
    from src.services.auto_tagging import suggest_tier3_vision

    needs_tier3 = [d for d in created if not d.tags and d.image_data is not None]
    if not needs_tier3:
        return
    logger.info("Tier 3: vision AI on %d still-untagged designs", len(needs_tier3))
    batch_size = _coerce_batch_size(commit_batch_size)
    for start in range(0, len(needs_tier3), batch_size):
        batch = needs_tier3[start : start + batch_size]
        tier3_results = suggest_tier3_vision(batch, valid_descriptions_list, api_key)
        updated = 0
        for design in batch:
            descriptions = tier3_results.get(design.id, [])
            if descriptions:
                design.tags = _unique_tags_from_descriptions(descriptions, desc_to_tag)
                design.tagging_tier = 3
                updated += 1
        if updated:
            db.commit()
