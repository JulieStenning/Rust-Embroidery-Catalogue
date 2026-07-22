// Database models (SQLx FromRow structs)

use sqlx::FromRow;

/// A key/value setting row from the settings table.
#[derive(FromRow, Debug, Clone)]
pub struct Setting {
    pub key: Option<String>,
    pub value: String,
    pub description: Option<String>,
}

/// A designer record.
#[derive(FromRow, Debug, Clone)]
pub struct Designer {
    pub id: Option<i64>,
    pub name: String,
}

/// A source record.
#[derive(FromRow, Debug, Clone)]
pub struct Source {
    pub id: Option<i64>,
    pub name: String,
}

/// A hoop record.
#[derive(FromRow, Debug, Clone)]
pub struct Hoop {
    pub id: Option<i64>,
    pub name: String,
    pub max_width_mm: f64,
    pub max_height_mm: f64,
}

/// A tag record.
#[derive(FromRow, Debug, Clone)]
pub struct Tag {
    pub id: Option<i64>,
    pub description: String,
    pub tag_group: Option<String>,
}

/// A design record.
#[derive(FromRow, Debug, Clone)]
pub struct Design {
    pub id: Option<i64>,
    pub filename: String,
    pub filepath: String,
    pub image_data: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub stitch_count: Option<i64>,
    pub color_count: Option<i64>,
    pub color_change_count: Option<i64>,
    pub notes: Option<String>,
    pub rating: Option<i64>,
    pub is_stitched: bool,
    pub tags_checked: bool,
    pub tagging_tier: Option<i64>,
    pub date_added: Option<String>,
    pub designer_id: Option<i64>,
    pub source_id: Option<i64>,
    pub hoop_id: Option<i64>,
    pub file_size_bytes: Option<i64>,
    pub file_hash_blake3: Option<String>,
}

/// A project record.
#[derive(FromRow, Debug, Clone)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub date_created: Option<String>,
}
