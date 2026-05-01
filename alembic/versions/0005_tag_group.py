"""Ensure ``tag_group`` exists on tags and delivered tags are up to date.

Revision ID: 0005_tag_group
Revises: 0004_tagging_tier
Create Date: 2026-03-31
"""
from __future__ import annotations

from alembic import op
import sqlalchemy as sa

revision = "0005_tag_group"
down_revision = "0004_tagging_tier"
branch_labels = None
depends_on = None

RENAMES = {
    "Floral": "Flowers",
    "Sea": "Nautical",
}

TAG_ROWS = [
    {"id": 1, "description": "Cross Stitch", "tag_group": "stitching"},
    {"id": 2, "description": "In The Hoop", "tag_group": "stitching"},
    {"id": 3, "description": "Filled", "tag_group": "stitching"},
    {"id": 4, "description": "Redwork", "tag_group": "stitching"},
    {"id": 5, "description": "Blackwork", "tag_group": "stitching"},
    {"id": 6, "description": "Stfumato", "tag_group": "stitching"},
    {"id": 7, "description": "Cutwork", "tag_group": "stitching"},
    {"id": 8, "description": "Don't Know", "tag_group": "image"},
    {"id": 9, "description": "Line Outline", "tag_group": "stitching"},
    {"id": 10, "description": "Satin Stitch", "tag_group": "stitching"},
    {"id": 11, "description": "Applique", "tag_group": "stitching"},
    {"id": 12, "description": "Silhouette", "tag_group": "image"},
    {"id": 13, "description": "Light Fills", "tag_group": "stitching"},
    {"id": 14, "description": "Lace", "tag_group": "stitching"},
    {"id": 15, "description": "Trapunto", "tag_group": "stitching"},
    {"id": 16, "description": "For Quilting", "tag_group": "image"},
    {"id": 17, "description": "Handstitched Look", "tag_group": "stitching"},
    {"id": 18, "description": "Animals", "tag_group": "image"},
    {"id": 19, "description": "Flowers", "tag_group": "image"},
    {"id": 20, "description": "People and Work", "tag_group": "image"},
    {"id": 21, "description": "Job", "tag_group": "image"},
    {"id": 22, "description": "House", "tag_group": "image"},
    {"id": 23, "description": "Garden", "tag_group": "image"},
    {"id": 24, "description": "Music", "tag_group": "image"},
    {"id": 25, "description": "Nautical", "tag_group": "image"},
    {"id": 26, "description": "Landscapes and Travel", "tag_group": "image"},
    {"id": 27, "description": "Toys", "tag_group": "image"},
    {"id": 28, "description": "Hearts and Lips", "tag_group": "image"},
    {"id": 29, "description": "Sport", "tag_group": "image"},
    {"id": 30, "description": "Borders", "tag_group": "image"},
    {"id": 31, "description": "Paisley", "tag_group": "image"},
    {"id": 32, "description": "Butterflies and Insects", "tag_group": "image"},
    {"id": 33, "description": "Words and Letters", "tag_group": "image"},
    {"id": 34, "description": "Christmas", "tag_group": "image"},
    {"id": 35, "description": "Patterns", "tag_group": "image"},
    {"id": 36, "description": "Corners", "tag_group": "image"},
    {"id": 37, "description": "Bow and Ribbons", "tag_group": "image"},
    {"id": 38, "description": "Frames", "tag_group": "image"},
    {"id": 39, "description": "Trees", "tag_group": "image"},
    {"id": 40, "description": "Transport", "tag_group": "image"},
    {"id": 42, "description": "Birds", "tag_group": "image"},
    {"id": 43, "description": "Monsters", "tag_group": "image"},
    {"id": 44, "description": "Utility - Testing", "tag_group": "image"},
    {"id": 45, "description": "Food", "tag_group": "image"},
    {"id": 46, "description": "Scrolls", "tag_group": "image"},
    {"id": 47, "description": "Footwear", "tag_group": "image"},
    {"id": 48, "description": "For Clothes", "tag_group": "image"},
    {"id": 49, "description": "Faces", "tag_group": "image"},
    {"id": 50, "description": "Handbags", "tag_group": "image"},
    {"id": 51, "description": "Fairies, Elves etc.", "tag_group": "image"},
    {"id": 52, "description": "Hobbies", "tag_group": "image"},
    {"id": 53, "description": "Ornaments", "tag_group": "image"},
    {"id": 54, "description": "Collars", "tag_group": "image"},
    {"id": 55, "description": "Household", "tag_group": "image"},
    {"id": 56, "description": "Fashion", "tag_group": "image"},
    {"id": 57, "description": "Halloween", "tag_group": "image"},
    {"id": 58, "description": "Sun Moon and Stars", "tag_group": "image"},
    {"id": 59, "description": "Angels", "tag_group": "image"},
    {"id": 60, "description": "Babies", "tag_group": "image"},
    {"id": 61, "description": "Quilting", "tag_group": "stitching"},
    {"id": 62, "description": "Jewellery", "tag_group": "image"},
    {"id": 63, "description": "Buildings and Structures", "tag_group": "image"},
    {"id": 64, "description": "Crests", "tag_group": "image"},
    {"id": 65, "description": "Badges and Crests", "tag_group": "image"},
    {"id": 66, "description": "Fish and Seashells", "tag_group": "image"},
    {"id": 67, "description": "Flags", "tag_group": "image"},
    {"id": 68, "description": "Children", "tag_group": "image"},
    {"id": 69, "description": "Cartoon", "tag_group": "image"},
    {"id": 70, "description": "Banners", "tag_group": "image"},
    {"id": 71, "description": "Celebrations", "tag_group": "image"},
    {"id": 72, "description": "Ghosts", "tag_group": "image"},
    {"id": 73, "description": "Winter", "tag_group": "image"},
    {"id": 74, "description": "Zodiac", "tag_group": "image"},
    {"id": 75, "description": "Alphabets", "tag_group": "image"},
    {"id": 95, "description": "Cats", "tag_group": "image"},
    {"id": 96, "description": "Celtic and Tribal", "tag_group": "image"},
    {"id": 97, "description": "Dogs", "tag_group": "image"},
    {"id": 98, "description": "Diwali", "tag_group": "image"},
    {"id": 99, "description": "Easter", "tag_group": "image"},
    {"id": 100, "description": "Eid", "tag_group": "image"},
    {"id": 101, "description": "Fantasy", "tag_group": "image"},
    {"id": 102, "description": "Father's Day", "tag_group": "image"},
    {"id": 103, "description": "Hanukkah", "tag_group": "image"},
    {"id": 104, "description": "Horses", "tag_group": "image"},
    {"id": 105, "description": "ITH Accessories", "tag_group": "stitching"},
    {"id": 106, "description": "Monogram", "tag_group": "image"},
    {"id": 107, "description": "Mother's Day", "tag_group": "image"},
    {"id": 108, "description": "Religious", "tag_group": "image"},
    {"id": 109, "description": "Sketchy and Vintage", "tag_group": "stitching"},
    {"id": 110, "description": "Thanksgiving", "tag_group": "image"},
    {"id": 111, "description": "Valentine's Day", "tag_group": "image"},
    {"id": 112, "description": "Wedding", "tag_group": "image"},
    {"id": 113, "description": "Wreaths", "tag_group": "image"},
    {"id": 114, "description": "Steampunk", "tag_group": "image"},
    {"id": 115, "description": "Sewing", "tag_group": "image"},
    {"id": 116, "description": "Clothes", "tag_group": "image"},
    {"id": 117, "description": "Netfill", "tag_group": "stitching"},
    {"id": 118, "description": "Dancing", "tag_group": "image"},
]


def _table_names() -> set[str]:
    return set(sa.inspect(op.get_bind()).get_table_names())


def _column_names(table_name: str) -> set[str]:
    return {column["name"] for column in sa.inspect(op.get_bind()).get_columns(table_name)}


def upgrade() -> None:
    tables = _table_names()
    if "tags" not in tables:
        return

    if "tag_group" not in _column_names("tags"):
        op.add_column(
            "tags",
            sa.Column("tag_group", sa.String(20), nullable=True),
        )

    conn = op.get_bind()

    for old_name, new_name in RENAMES.items():
        conn.execute(
            sa.text("UPDATE tags SET description = :new WHERE description = :old"),
            {"new": new_name, "old": old_name},
        )

    for row in TAG_ROWS:
        params = {
            "id": row["id"],
            "description": row["description"],
            "tag_group": row["tag_group"],
        }
        exists = conn.execute(
            sa.text("SELECT 1 FROM tags WHERE id = :id OR description = :description"),
            {"id": params["id"], "description": params["description"]},
        ).fetchone()
        if exists:
            conn.execute(
                sa.text(
                    "UPDATE tags "
                    "SET description = :description, tag_group = :tag_group "
                    "WHERE id = :id OR description = :description"
                ),
                params,
            )
        else:
            conn.execute(
                sa.text(
                    "INSERT INTO tags (id, description, tag_group) "
                    "VALUES (:id, :description, :tag_group)"
                ),
                params,
            )


def downgrade() -> None:
    tables = _table_names()
    if "tags" in tables and "tag_group" in _column_names("tags"):
        op.drop_column("tags", "tag_group")
