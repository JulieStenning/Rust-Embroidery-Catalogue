"""Add composite browse-query indexes.

Revision ID: 0007_composite_indexes
Revises: 0006_search_indexes
Create Date: 2026-04-05
"""
from __future__ import annotations

from alembic import op

revision = "0007_composite_indexes"
down_revision = "0006_search_indexes"
branch_labels = None
depends_on = None


def upgrade() -> None:
    # Common browse queries filter by designer/source and then sort by filename.
    op.create_index(
        "ix_designs_designer_id_filename",
        "designs",
        ["designer_id", "filename"],
        unique=False,
    )
    op.create_index(
        "ix_designs_source_id_filename",
        "designs",
        ["source_id", "filename"],
        unique=False,
    )

    # Sorting by date added should not require a full table sort.
    op.create_index("ix_designs_date_added", "designs", ["date_added"], unique=False)

    # Reverse many-to-many traversals benefit from a composite in lookup order.
    op.create_index(
        "ix_design_tags_tag_id_design_id",
        "design_tags",
        ["tag_id", "design_id"],
        unique=False,
    )
    op.create_index(
        "ix_project_designs_design_id_project_id",
        "project_designs",
        ["design_id", "project_id"],
        unique=False,
    )


def downgrade() -> None:
    op.drop_index("ix_project_designs_design_id_project_id", table_name="project_designs")
    op.drop_index(
        "ix_design_tags_tag_id_design_id",
        table_name="design_tags",
    )
    op.drop_index("ix_designs_date_added", table_name="designs")
    op.drop_index("ix_designs_source_id_filename", table_name="designs")
    op.drop_index("ix_designs_designer_id_filename", table_name="designs")
