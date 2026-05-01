"""Ensure current tag names and ``tags_checked`` exist.

Revision ID: 0003_tags_types
Revises: 0002_settings
Create Date: 2026-02-28
"""
from __future__ import annotations

from alembic import op
import sqlalchemy as sa

revision = "0003_tags_types"
down_revision = "0002_settings"
branch_labels = None
depends_on = None

NEW_TAGS = [
    "Cats",
    "Celtic and Tribal",
    "Dogs",
    "Diwali",
    "Easter",
    "Eid",
    "Fantasy",
    "Father's Day",
    "Hanukkah",
    "Horses",
    "ITH Accessories",
    "Monogram",
    "Mother's Day",
    "Religious",
    "Sketchy and Vintage",
    "Thanksgiving",
    "Valentine's Day",
    "Wedding",
    "Wreaths",
]

RENAMES = {
    "Floral": "Flowers",
    "Sea": "Nautical",
}


def _table_names() -> set[str]:
    return set(sa.inspect(op.get_bind()).get_table_names())


def _column_names(table_name: str) -> set[str]:
    return {column["name"] for column in sa.inspect(op.get_bind()).get_columns(table_name)}


def upgrade() -> None:
    tables = _table_names()

    if "designs" in tables and "tags_checked" not in _column_names("designs"):
        op.add_column(
            "designs",
            sa.Column("tags_checked", sa.Boolean(), nullable=False, server_default="false"),
        )

    if "tags" not in tables:
        return

    conn = op.get_bind()
    for old_name, new_name in RENAMES.items():
        conn.execute(
            sa.text("UPDATE tags SET description = :new WHERE description = :old"),
            {"new": new_name, "old": old_name},
        )

    for description in NEW_TAGS:
        exists = conn.execute(
            sa.text("SELECT 1 FROM tags WHERE description = :d"),
            {"d": description},
        ).fetchone()
        if not exists:
            conn.execute(
                sa.text("INSERT INTO tags (description) VALUES (:d)"),
                {"d": description},
            )


def downgrade() -> None:
    tables = _table_names()

    if "designs" in tables and "tags_checked" in _column_names("designs"):
        op.drop_column("designs", "tags_checked")

    if "tags" not in tables:
        return

    conn = op.get_bind()
    for old_name, new_name in RENAMES.items():
        conn.execute(
            sa.text("UPDATE tags SET description = :old WHERE description = :new"),
            {"old": old_name, "new": new_name},
        )
    for description in NEW_TAGS:
        conn.execute(
            sa.text("DELETE FROM tags WHERE description = :d"),
            {"d": description},
        )
