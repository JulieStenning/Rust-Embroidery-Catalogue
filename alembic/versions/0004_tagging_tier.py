"""Ensure ``tagging_tier`` exists on designs.

Revision ID: 0004_tagging_tier
Revises: 0003_tags_types
Create Date: 2026-03-01
"""
from __future__ import annotations

from alembic import op
import sqlalchemy as sa

revision = "0004_tagging_tier"
down_revision = "0003_tags_types"
branch_labels = None
depends_on = None


def _column_names(table_name: str) -> set[str]:
    return {column["name"] for column in sa.inspect(op.get_bind()).get_columns(table_name)}


def upgrade() -> None:
    if "tagging_tier" not in _column_names("designs"):
        op.add_column(
            "designs",
            sa.Column("tagging_tier", sa.SmallInteger(), nullable=True),
        )


def downgrade() -> None:
    if "tagging_tier" in _column_names("designs"):
        op.drop_column("designs", "tagging_tier")
