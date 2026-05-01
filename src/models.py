"""
SQLAlchemy ORM models for the Embroidery Catalogue application.

Tables:
  designers, sources, hoops, tags, designs,
  design_tags, projects, project_designs
"""

from datetime import date
from typing import Optional

from sqlalchemy import (
    Boolean,
    Column,
    Date,
    ForeignKey,
    Index,
    Integer,
    LargeBinary,
    Numeric,
    SmallInteger,
    String,
    Table,
    Text,
)
from sqlalchemy.orm import Mapped, mapped_column, relationship

from src.database import Base

# ---------------------------------------------------------------------------
# Reference / lookup tables
# ---------------------------------------------------------------------------


class Designer(Base):
    __tablename__ = "designers"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, index=True)
    name: Mapped[str] = mapped_column(String(255), nullable=False, unique=True)

    designs: Mapped[list["Design"]] = relationship("Design", back_populates="designer")

    def __repr__(self) -> str:
        return f"<Designer id={self.id} name={self.name!r}>"


class Source(Base):
    __tablename__ = "sources"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, index=True)
    name: Mapped[str] = mapped_column(String(255), nullable=False, unique=True)

    designs: Mapped[list["Design"]] = relationship("Design", back_populates="source")

    def __repr__(self) -> str:
        return f"<Source id={self.id} name={self.name!r}>"


class Hoop(Base):
    __tablename__ = "hoops"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, index=True)
    name: Mapped[str] = mapped_column(String(100), nullable=False, unique=True)
    max_width_mm: Mapped[float] = mapped_column(Numeric(8, 2), nullable=False)
    max_height_mm: Mapped[float] = mapped_column(Numeric(8, 2), nullable=False)

    designs: Mapped[list["Design"]] = relationship("Design", back_populates="hoop")

    def __repr__(self) -> str:
        return f"<Hoop id={self.id} name={self.name!r}>"


class Tag(Base):
    __tablename__ = "tags"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, index=True)
    description: Mapped[str] = mapped_column(String(255), nullable=False, unique=True)
    tag_group: Mapped[str | None] = mapped_column(String(20), nullable=True)

    designs: Mapped[list["Design"]] = relationship(
        "Design",
        secondary="design_tags",
        back_populates="tags",
    )

    def __repr__(self) -> str:
        return f"<Tag id={self.id} description={self.description!r} tag_group={self.tag_group!r}>"


# ---------------------------------------------------------------------------
# Junction tables
# ---------------------------------------------------------------------------

design_tags = Table(
    "design_tags",
    Base.metadata,
    Column("design_id", Integer, ForeignKey("designs.id", ondelete="CASCADE"), primary_key=True),
    Column(
        "tag_id",
        Integer,
        ForeignKey("tags.id", ondelete="CASCADE"),
        primary_key=True,
        index=True,
    ),
)
Index(
    "ix_design_tags_tag_id_design_id",
    design_tags.c.tag_id,
    design_tags.c.design_id,
)

project_designs = Table(
    "project_designs",
    Base.metadata,
    Column("project_id", Integer, ForeignKey("projects.id", ondelete="CASCADE"), primary_key=True),
    Column(
        "design_id",
        Integer,
        ForeignKey("designs.id", ondelete="CASCADE"),
        primary_key=True,
        index=True,
    ),
)
Index(
    "ix_project_designs_design_id_project_id",
    project_designs.c.design_id,
    project_designs.c.project_id,
)


# ---------------------------------------------------------------------------
# Core tables
# ---------------------------------------------------------------------------


class Design(Base):
    __tablename__ = "designs"
    __table_args__ = (
        Index("ix_designs_designer_id_filename", "designer_id", "filename"),
        Index("ix_designs_source_id_filename", "source_id", "filename"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True, index=True)
    filename: Mapped[str] = mapped_column(String(500), nullable=False, index=True)
    filepath: Mapped[str] = mapped_column(String(1000), nullable=False, index=True)
    image_data: Mapped[bytes | None] = mapped_column(LargeBinary, nullable=True)
    image_type: Mapped[str | None] = mapped_column(String(10), nullable=True)
    width_mm: Mapped[float | None] = mapped_column(Numeric(8, 2), nullable=True)
    height_mm: Mapped[float | None] = mapped_column(Numeric(8, 2), nullable=True)
    stitch_count: Mapped[int | None] = mapped_column(Integer, nullable=True)
    color_count: Mapped[int | None] = mapped_column(Integer, nullable=True)
    color_change_count: Mapped[int | None] = mapped_column(Integer, nullable=True)
    notes: Mapped[str | None] = mapped_column(Text, nullable=True)
    rating: Mapped[int | None] = mapped_column(SmallInteger, nullable=True)
    is_stitched: Mapped[bool] = mapped_column(Boolean, nullable=False, default=False)
    tags_checked: Mapped[bool] = mapped_column(Boolean, nullable=False, default=False)
    tagging_tier: Mapped[int | None] = mapped_column(SmallInteger, nullable=True)
    date_added: Mapped[date | None] = mapped_column(Date, nullable=True, index=True)

    designer_id: Mapped[int | None] = mapped_column(
        Integer,
        ForeignKey("designers.id", ondelete="SET NULL"),
        nullable=True,
        index=True,
    )
    source_id: Mapped[int | None] = mapped_column(
        Integer,
        ForeignKey("sources.id", ondelete="SET NULL"),
        nullable=True,
        index=True,
    )
    hoop_id: Mapped[int | None] = mapped_column(
        Integer, ForeignKey("hoops.id", ondelete="SET NULL"), nullable=True
    )

    designer: Mapped[Optional["Designer"]] = relationship("Designer", back_populates="designs")
    source: Mapped[Optional["Source"]] = relationship("Source", back_populates="designs")
    hoop: Mapped[Optional["Hoop"]] = relationship("Hoop", back_populates="designs")

    tags: Mapped[list["Tag"]] = relationship(
        "Tag",
        secondary="design_tags",
        back_populates="designs",
    )
    projects: Mapped[list["Project"]] = relationship(
        "Project",
        secondary="project_designs",
        back_populates="designs",
    )

    def __repr__(self) -> str:
        return f"<Design id={self.id} filename={self.filename!r}>"


class Project(Base):
    __tablename__ = "projects"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, index=True)
    name: Mapped[str] = mapped_column(String(255), nullable=False, unique=True)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    date_created: Mapped[date | None] = mapped_column(Date, nullable=True)

    designs: Mapped[list["Design"]] = relationship(
        "Design",
        secondary="project_designs",
        back_populates="projects",
    )

    def __repr__(self) -> str:
        return f"<Project id={self.id} name={self.name!r}>"


# ---------------------------------------------------------------------------
# Application settings (key/value store)
# ---------------------------------------------------------------------------


class Setting(Base):
    __tablename__ = "settings"

    key: Mapped[str] = mapped_column(String(100), primary_key=True)
    value: Mapped[str] = mapped_column(Text, nullable=False)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)

    def __repr__(self) -> str:
        return f"<Setting key={self.key!r} value={self.value!r}>"
