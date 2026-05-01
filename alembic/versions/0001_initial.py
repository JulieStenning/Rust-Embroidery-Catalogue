"""initial current schema

Revision ID: 0001_initial
Revises:
Create Date: 2026-02-28
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "0001_initial"
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

TAG_ROWS = [
    {"id": 1, "description": "Cross Stitch"},
    {"id": 2, "description": "In The Hoop"},
    {"id": 3, "description": "Filled"},
    {"id": 4, "description": "Redwork"},
    {"id": 5, "description": "Blackwork"},
    {"id": 6, "description": "Stfumato"},
    {"id": 7, "description": "Cutwork"},
    {"id": 8, "description": "Don't Know"},
    {"id": 9, "description": "Line Outline"},
    {"id": 10, "description": "Satin Stitch"},
    {"id": 11, "description": "Applique"},
    {"id": 12, "description": "Silhouette"},
    {"id": 13, "description": "Light Fills"},
    {"id": 14, "description": "Lace"},
    {"id": 15, "description": "Trapunto"},
    {"id": 16, "description": "For Quilting"},
    {"id": 17, "description": "Handstitched Look"},
    {"id": 18, "description": "Animals"},
    {"id": 19, "description": "Flowers"},
    {"id": 20, "description": "People and Work"},
    {"id": 21, "description": "Job"},
    {"id": 22, "description": "House"},
    {"id": 23, "description": "Garden"},
    {"id": 24, "description": "Music"},
    {"id": 25, "description": "Nautical"},
    {"id": 26, "description": "Landscapes and Travel"},
    {"id": 27, "description": "Toys"},
    {"id": 28, "description": "Hearts and Lips"},
    {"id": 29, "description": "Sport"},
    {"id": 30, "description": "Borders"},
    {"id": 31, "description": "Paisley"},
    {"id": 32, "description": "Butterflies and Insects"},
    {"id": 33, "description": "Words and Letters"},
    {"id": 34, "description": "Christmas"},
    {"id": 35, "description": "Patterns"},
    {"id": 36, "description": "Corners"},
    {"id": 37, "description": "Bow and Ribbons"},
    {"id": 38, "description": "Frames"},
    {"id": 39, "description": "Trees"},
    {"id": 40, "description": "Transport"},
    {"id": 42, "description": "Birds"},
    {"id": 43, "description": "Monsters"},
    {"id": 44, "description": "Utility - Testing"},
    {"id": 45, "description": "Food"},
    {"id": 46, "description": "Scrolls"},
    {"id": 47, "description": "Footwear"},
    {"id": 48, "description": "For Clothes"},
    {"id": 49, "description": "Faces"},
    {"id": 50, "description": "Handbags"},
    {"id": 51, "description": "Fairies, Elves etc."},
    {"id": 52, "description": "Hobbies"},
    {"id": 53, "description": "Ornaments"},
    {"id": 54, "description": "Collars"},
    {"id": 55, "description": "Household"},
    {"id": 56, "description": "Fashion"},
    {"id": 57, "description": "Halloween"},
    {"id": 58, "description": "Sun Moon and Stars"},
    {"id": 59, "description": "Angels"},
    {"id": 60, "description": "Babies"},
    {"id": 61, "description": "Quilting"},
    {"id": 62, "description": "Jewellery"},
    {"id": 63, "description": "Buildings and Structures"},
    {"id": 64, "description": "Crests"},
    {"id": 65, "description": "Badges and Crests"},
    {"id": 66, "description": "Fish and Seashells"},
    {"id": 67, "description": "Flags"},
    {"id": 68, "description": "Children"},
    {"id": 69, "description": "Cartoon"},
    {"id": 70, "description": "Banners"},
    {"id": 71, "description": "Celebrations"},
    {"id": 72, "description": "Ghosts"},
    {"id": 73, "description": "Winter"},
    {"id": 74, "description": "Zodiac"},
    {"id": 75, "description": "Alphabets"},
    {"id": 95, "description": "Cats"},
    {"id": 96, "description": "Celtic and Tribal"},
    {"id": 97, "description": "Dogs"},
    {"id": 98, "description": "Diwali"},
    {"id": 99, "description": "Easter"},
    {"id": 100, "description": "Eid"},
    {"id": 101, "description": "Fantasy"},
    {"id": 102, "description": "Father's Day"},
    {"id": 103, "description": "Hanukkah"},
    {"id": 104, "description": "Horses"},
    {"id": 105, "description": "ITH Accessories"},
    {"id": 106, "description": "Monogram"},
    {"id": 107, "description": "Mother's Day"},
    {"id": 108, "description": "Religious"},
    {"id": 109, "description": "Sketchy and Vintage"},
    {"id": 110, "description": "Thanksgiving"},
    {"id": 111, "description": "Valentine's Day"},
    {"id": 112, "description": "Wedding"},
    {"id": 113, "description": "Wreaths"},
    {"id": 114, "description": "Steampunk"},
    {"id": 115, "description": "Sewing"},
    {"id": 116, "description": "Clothes"},
    {"id": 117, "description": "Netfill"},
    {"id": 118, "description": "Dancing"},
]


def upgrade() -> None:
    op.create_table(
        "designers",
        sa.Column("id", sa.Integer(), primary_key=True),
        sa.Column("name", sa.String(255), nullable=False, unique=True),
    )

    op.create_table(
        "sources",
        sa.Column("id", sa.Integer(), primary_key=True),
        sa.Column("name", sa.String(255), nullable=False, unique=True),
    )

    op.create_table(
        "hoops",
        sa.Column("id", sa.Integer(), primary_key=True),
        sa.Column("name", sa.String(100), nullable=False, unique=True),
        sa.Column("max_width_mm", sa.Numeric(8, 2), nullable=False),
        sa.Column("max_height_mm", sa.Numeric(8, 2), nullable=False),
    )

    op.create_table(
        "tags",
        sa.Column("id", sa.Integer(), primary_key=True),
        sa.Column("description", sa.String(255), nullable=False, unique=True),
        sa.Column("tag_group", sa.String(20), nullable=True),
    )

    op.create_table(
        "projects",
        sa.Column("id", sa.Integer(), primary_key=True),
        sa.Column("name", sa.String(255), nullable=False, unique=True),
        sa.Column("description", sa.Text(), nullable=True),
        sa.Column("date_created", sa.Date(), nullable=True),
    )

    op.create_table(
        "designs",
        sa.Column("id", sa.Integer(), primary_key=True),
        sa.Column("filename", sa.String(500), nullable=False),
        sa.Column("filepath", sa.String(1000), nullable=False),
        sa.Column("image_data", sa.LargeBinary(), nullable=True),
        sa.Column("width_mm", sa.Numeric(8, 2), nullable=True),
        sa.Column("height_mm", sa.Numeric(8, 2), nullable=True),
        sa.Column("notes", sa.Text(), nullable=True),
        sa.Column("rating", sa.SmallInteger(), nullable=True),
        sa.Column("is_stitched", sa.Boolean(), nullable=False, server_default="false"),
        sa.Column("tags_checked", sa.Boolean(), nullable=False, server_default="false"),
        sa.Column("tagging_tier", sa.SmallInteger(), nullable=True),
        sa.Column("date_added", sa.Date(), nullable=True),
        sa.Column("designer_id", sa.Integer(), sa.ForeignKey("designers.id", ondelete="SET NULL"), nullable=True),
        sa.Column("source_id", sa.Integer(), sa.ForeignKey("sources.id", ondelete="SET NULL"), nullable=True),
        sa.Column("hoop_id", sa.Integer(), sa.ForeignKey("hoops.id", ondelete="SET NULL"), nullable=True),
    )

    op.create_table(
        "design_tags",
        sa.Column("design_id", sa.Integer(), sa.ForeignKey("designs.id", ondelete="CASCADE"), primary_key=True),
        sa.Column("tag_id", sa.Integer(), sa.ForeignKey("tags.id", ondelete="CASCADE"), primary_key=True),
    )

    op.create_table(
        "project_designs",
        sa.Column("project_id", sa.Integer(), sa.ForeignKey("projects.id", ondelete="CASCADE"), primary_key=True),
        sa.Column("design_id", sa.Integer(), sa.ForeignKey("designs.id", ondelete="CASCADE"), primary_key=True),
    )

    # Hoops are intentionally not pre-seeded because users need to configure
    # the machine hoops they actually own.

    tags_table = sa.table(
        "tags",
        sa.column("id", sa.Integer()),
        sa.column("description", sa.String(255)),
    )
    op.bulk_insert(tags_table, TAG_ROWS)


def downgrade() -> None:
    op.drop_table("project_designs")
    op.drop_table("design_tags")
    op.drop_table("designs")
    op.drop_table("projects")
    op.drop_table("tags")
    op.drop_table("hoops")
    op.drop_table("sources")
    op.drop_table("designers")
