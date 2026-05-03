// Rust Diesel models for Embroidery Catalogue
// This file is auto-generated from the Python SQLAlchemy models

use diesel::prelude::*;
use diesel::sql_types::*;
use chrono::NaiveDate;

#[derive(Queryable, Identifiable, Insertable, Associations, Debug)]
#[diesel(table_name = designers)]
pub struct Designer {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug)]
#[diesel(table_name = sources)]
pub struct Source {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug)]
#[diesel(table_name = hoops)]
pub struct Hoop {
    pub id: i32,
    pub name: String,
    pub max_width_mm: f64,
    pub max_height_mm: f64,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug)]
#[diesel(table_name = tags)]
pub struct Tag {
    pub id: i32,
    pub description: String,
    pub tag_group: Option<String>,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug)]
#[diesel(table_name = designs)]
#[diesel(belongs_to(Designer))]
#[diesel(belongs_to(Source))]
#[diesel(belongs_to(Hoop))]
pub struct Design {
    pub id: i32,
    pub filename: String,
    pub filepath: String,
    pub image_data: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub stitch_count: Option<i32>,
    pub color_count: Option<i32>,
    pub color_change_count: Option<i32>,
    pub notes: Option<String>,
    pub rating: Option<i16>,
    pub is_stitched: bool,
    pub tags_checked: bool,
    pub tagging_tier: Option<i16>,
    pub date_added: Option<NaiveDate>,
    pub designer_id: Option<i32>,
    pub source_id: Option<i32>,
    pub hoop_id: Option<i32>,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug)]
#[diesel(table_name = projects)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub date_created: Option<NaiveDate>,
}

#[derive(Queryable, Identifiable, Insertable, Debug)]
#[diesel(table_name = settings)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
}

#[derive(Identifiable, Queryable, Insertable, Associations, Debug)]
#[diesel(table_name = design_tags)]
#[diesel(primary_key(design_id, tag_id))]
#[diesel(belongs_to(Design, foreign_key = design_id))]
#[diesel(belongs_to(Tag, foreign_key = tag_id))]
pub struct DesignTag {
    pub design_id: i32,
    pub tag_id: i32,
}

#[derive(Identifiable, Queryable, Insertable, Associations, Debug)]
#[diesel(table_name = project_designs)]
#[diesel(primary_key(project_id, design_id))]
#[diesel(belongs_to(Project, foreign_key = project_id))]
#[diesel(belongs_to(Design, foreign_key = design_id))]
pub struct ProjectDesign {
    pub project_id: i32,
    pub design_id: i32,
}
