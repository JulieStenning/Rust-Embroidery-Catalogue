"""Advanced search: query parsing and SQLAlchemy filter building.

Supports Google-like syntax in the ``q`` field:
  - ``"exact phrase"``  → exact substring match
  - ``-word``           → exclusion
  - ``word1 OR word2``  → at least one must match
  - ``word``            → required (AND)

The structured fields (``all_words``, ``exact_phrase``, ``any_words``,
``none_words``) are simpler: each is interpreted as described by its name.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field

# ---------------------------------------------------------------------------
# Parsed query data class
# ---------------------------------------------------------------------------


@dataclass
class ParsedQuery:
    """Holds the decomposed parts of an advanced search request."""

    required_words: list[str] = field(default_factory=list)
    """Words that MUST appear somewhere in the result (AND logic)."""

    exact_phrases: list[str] = field(default_factory=list)
    """Phrases that must appear verbatim (AND logic; each phrase is one item)."""

    any_words: list[str] = field(default_factory=list)
    """At least one of these words must appear (OR logic)."""

    excluded_words: list[str] = field(default_factory=list)
    """Words that must NOT appear (NOT logic)."""

    def is_empty(self) -> bool:
        return not (
            self.required_words or self.exact_phrases or self.any_words or self.excluded_words
        )


# ---------------------------------------------------------------------------
# Parsing helpers
# ---------------------------------------------------------------------------


def _normalize_query_text(value: str) -> str:
    """Normalise user-entered search text for reliable parsing."""
    return (value or "").translate(
        str.maketrans(
            {
                "\u2018": "'",
                "\u2019": "'",
                "\u201c": '"',
                "\u201d": '"',
                "\u00a0": " ",
            }
        )
    )


def _term_to_like_pattern(term: str) -> str:
    """Convert a user search term into a SQL LIKE pattern.

    Plain text becomes a substring search. `*` and `?` are treated as wildcard
    operators so users can search for patterns like `*.hus` in the general box.
    """
    normalized = " ".join(term.strip().split()).lower()
    if not normalized:
        return ""
    if "*" in normalized or "?" in normalized:
        return normalized.replace("*", "%").replace("?", "_")
    return f"%{normalized}%"


def _tokenize(q: str) -> list[str]:
    """Tokenise a Google-like query string.

    Quoted strings are kept as single tokens (with their quotes).
    ``-word`` tokens are returned with the leading dash.
    """
    q = _normalize_query_text(q)
    tokens: list[str] = []
    for m in re.finditer(r'"[^"]*"|-?\S+', q):
        tokens.append(m.group())
    return tokens


def _strip_optional_quotes(token: str) -> str:
    """Trim whitespace and remove surrounding double quotes if present."""
    token = " ".join(_normalize_query_text(token).strip().split())
    if len(token) >= 2 and token[0] == '"' and token[-1] == '"':
        token = token[1:-1].strip()
    return token


def _parse_google_syntax(q: str, result: ParsedQuery) -> None:
    """Parse a Google-like query string and populate *result* in place."""
    tokens = _tokenize(q)
    i = 0
    while i < len(tokens):
        token = tokens[i]

        # Skip bare OR keyword
        if token.upper() == "OR":
            i += 1
            continue

        # Exclusion: -word or -"phrase"
        if token.startswith("-") and len(token) > 1:
            inner = token[1:]
            if inner.startswith('"') and inner.endswith('"'):
                result.excluded_words.append(inner[1:-1])
            else:
                result.excluded_words.append(inner)
            i += 1
            continue

        # Quoted phrase: "some text"
        if token.startswith('"') and token.endswith('"') and len(token) >= 2:
            phrase = token[1:-1]
            if phrase:
                result.exact_phrases.append(phrase)
            i += 1
            continue

        # Check for OR chain: word OR word OR ...
        if i + 1 < len(tokens) and tokens[i + 1].upper() == "OR":
            or_group = [token]
            j = i + 1
            while j < len(tokens) and tokens[j].upper() == "OR":
                j += 1  # skip the OR keyword
                if j < len(tokens) and tokens[j].upper() != "OR":
                    or_group.append(tokens[j])
                    j += 1
            result.any_words.extend(or_group)
            i = j
            continue

        # Plain word — required (AND)
        result.required_words.append(token)
        i += 1


# ---------------------------------------------------------------------------
# Public parsing entry point
# ---------------------------------------------------------------------------


def parse_advanced_query(
    q: str = "",
    all_words: str = "",
    exact_phrase: str = "",
    any_words: str = "",
    none_words: str = "",
) -> ParsedQuery:
    """Build a :class:`ParsedQuery` from advanced search form fields.

    Parameters
    ----------
    q:
        General query box — supports Google-like syntax.
    all_words:
        Space-separated words that must ALL appear (AND).
    exact_phrase:
        A phrase that must appear verbatim.
    any_words:
        Space-separated words of which AT LEAST ONE must appear (OR).
    none_words:
        Space-separated words that must NOT appear.
    """
    result = ParsedQuery()

    q = _normalize_query_text(q)
    all_words = _normalize_query_text(all_words)
    exact_phrase = _normalize_query_text(exact_phrase)
    any_words = _normalize_query_text(any_words)
    none_words = _normalize_query_text(none_words)

    for token in _tokenize(all_words):
        cleaned = _strip_optional_quotes(token)
        if not cleaned:
            continue
        if token.startswith('"') and token.endswith('"'):
            result.exact_phrases.append(cleaned)
        else:
            result.required_words.append(cleaned)

    phrase = _strip_optional_quotes(exact_phrase)
    if phrase:
        result.exact_phrases.append(phrase)

    for token in _tokenize(any_words):
        cleaned = _strip_optional_quotes(token)
        if cleaned:
            result.any_words.append(cleaned)

    for token in _tokenize(none_words):
        cleaned = _strip_optional_quotes(token[1:] if token.startswith("-") else token)
        if cleaned:
            result.excluded_words.append(cleaned)

    if q.strip():
        _parse_google_syntax(q.strip(), result)

    return result


# ---------------------------------------------------------------------------
# SQLAlchemy filter builder
# ---------------------------------------------------------------------------


def build_search_filters(
    pq: ParsedQuery,
    *,
    search_filename: bool = True,
    search_tags: bool = True,
    search_folder: bool = True,
) -> list:
    """Translate a :class:`ParsedQuery` into a list of SQLAlchemy conditions.

    Every item in the returned list must be passed to ``Query.filter()``
    (they are combined with AND by SQLAlchemy).

    Parameters
    ----------
    pq:
        The parsed query to translate.
    search_filename:
        Include the design filename in field matching.
    search_tags:
        Include tag descriptions in field matching.
    search_folder:
        Include the folder portion of the stored file path in field matching.
    """
    from sqlalchemy import func, not_, or_

    from src.models import Design, Tag

    _folder_expr = func.folder_path(Design.filepath)

    def _field_condition(term: str) -> object | None:
        """Return an OR condition matching *term* across selected fields."""
        normalized = " ".join(term.strip().split())
        if not normalized:
            return None

        pattern = _term_to_like_pattern(normalized)
        clauses = []

        if search_filename:
            full_name_expr = func.lower(
                func.trim(Design.filename + func.file_extension(Design.filepath))
            )
            clauses.append(func.lower(func.trim(Design.filename)).like(pattern))
            clauses.append(func.lower(func.file_extension(Design.filepath)).like(pattern))
            clauses.append(full_name_expr.like(pattern))
        if search_folder:
            clauses.append(func.lower(func.trim(_folder_expr)).like(pattern))
        if search_tags:
            clauses.append(Design.tags.any(func.lower(func.trim(Tag.description)).like(pattern)))

        if not clauses:
            return None
        return or_(*clauses) if len(clauses) > 1 else clauses[0]

    conditions = []

    # Required words — every word must match in at least one field
    for word in pq.required_words:
        cond = _field_condition(word)
        if cond is not None:
            conditions.append(cond)

    # Exact phrases — each phrase must appear verbatim in at least one field
    for phrase in pq.exact_phrases:
        cond = _field_condition(phrase)
        if cond is not None:
            conditions.append(cond)

    # Any words — at least one of the words must match
    if pq.any_words:
        or_clauses = [_field_condition(w) for w in pq.any_words]
        or_clauses = [c for c in or_clauses if c is not None]
        if or_clauses:
            conditions.append(or_(*or_clauses))

    # Excluded words — none must match in any field
    for word in pq.excluded_words:
        cond = _field_condition(word)
        if cond is not None:
            conditions.append(not_(cond))

    return conditions
