"""Add stitch_count, color_count, color_change_count columns to designs table.

Revision ID: 0010_add_stitch_color_counts
Revises: 0009_remove_legacy_designs_base_path_setting
Create Date: 2026-04-25
"""
from __future__ import annotations

from collections.abc import Sequence

import sqlalchemy as sa

from alembic import op

revision: str = "0010_add_stitch_color_counts"
down_revision: str | None = "0009_remove_legacy_designs_base_path_setting"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.add_column("designs", sa.Column("stitch_count", sa.Integer(), nullable=True))
    op.add_column("designs", sa.Column("color_count", sa.Integer(), nullable=True))
    op.add_column("designs", sa.Column("color_change_count", sa.Integer(), nullable=True))


def downgrade() -> None:
    op.drop_column("designs", "color_change_count")
    op.drop_column("designs", "color_count")
    op.drop_column("designs", "stitch_count")
