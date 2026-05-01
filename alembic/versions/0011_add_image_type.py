"""Add image_type column to designs table.

Revision ID: 0011_add_image_type
Revises: 0010_add_stitch_color_counts
Create Date: 2026-04-28
"""
from __future__ import annotations

from collections.abc import Sequence

import sqlalchemy as sa

from alembic import op

revision: str = "0011_add_image_type"
down_revision: str | None = "0010_add_stitch_color_counts"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.add_column("designs", sa.Column("image_type", sa.String(10), nullable=True))
    # Backfill existing images as '2d' — the 2D preview option was recently added
    # and is the default for bulk operations, so existing images are most likely 2D.
    op.execute(
        "UPDATE designs SET image_type = '2d' WHERE image_data IS NOT NULL"
    )


def downgrade() -> None:
    op.drop_column("designs", "image_type")
