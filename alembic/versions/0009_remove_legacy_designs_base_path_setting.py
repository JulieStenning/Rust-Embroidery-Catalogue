"""Remove obsolete designs_base_path setting for managed-only storage.

Revision ID: 0009_remove_legacy_designs_base_path_setting
Revises: 0008_rename_tag_tables
Create Date: 2026-04-05
"""
from __future__ import annotations

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "0009_remove_legacy_designs_base_path_setting"
down_revision: Union[str, None] = "0008_rename_tag_tables"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    tables = set(sa.inspect(op.get_bind()).get_table_names())
    if "settings" in tables:
        op.execute(sa.text("DELETE FROM settings WHERE key = 'designs_base_path'"))


def downgrade() -> None:
    # The legacy setting is intentionally not restored. Managed-only storage
    # uses a fixed application-relative folder and no longer relies on this key.
    pass
