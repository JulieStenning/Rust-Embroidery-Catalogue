"""Reserved revision after the tag-schema cleanup.

Revision ID: 0008_rename_tag_tables
Revises: 0007_composite_indexes
Create Date: 2026-04-05
"""
from __future__ import annotations

revision = "0008_rename_tag_tables"
down_revision = "0007_composite_indexes"
branch_labels = None
depends_on = None


def upgrade() -> None:
    # Fresh databases already use the current tag/table names.
    pass


def downgrade() -> None:
    pass
