"""Add indexes for searchable design fields.

Revision ID: 0006_search_indexes
Revises: 0005_tag_group
Create Date: 2026-04-05
"""
from __future__ import annotations

from alembic import op

revision = "0006_search_indexes"
down_revision = "0005_tag_group"
branch_labels = None
depends_on = None


def upgrade() -> None:
    # Frequently filtered design fields
    op.create_index("ix_designs_filename", "designs", ["filename"], unique=False)
    op.create_index("ix_designs_filepath", "designs", ["filepath"], unique=False)
    op.create_index("ix_designs_designer_id", "designs", ["designer_id"], unique=False)
    op.create_index("ix_designs_source_id", "designs", ["source_id"], unique=False)

    # Reverse many-to-many lookups used by tag/project linking
    op.create_index(
        "ix_design_tags_tag_id",
        "design_tags",
        ["tag_id"],
        unique=False,
    )
    op.create_index(
        "ix_project_designs_design_id",
        "project_designs",
        ["design_id"],
        unique=False,
    )


def downgrade() -> None:
    op.drop_index("ix_project_designs_design_id", table_name="project_designs")
    op.drop_index("ix_design_tags_tag_id", table_name="design_tags")
    op.drop_index("ix_designs_source_id", table_name="designs")
    op.drop_index("ix_designs_designer_id", table_name="designs")
    op.drop_index("ix_designs_filepath", table_name="designs")
    op.drop_index("ix_designs_filename", table_name="designs")
