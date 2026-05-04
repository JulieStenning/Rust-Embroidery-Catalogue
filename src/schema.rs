// @generated automatically by Diesel CLI.

diesel::table! {
    design_tags (design_id, tag_id) {
        design_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::table! {
    designers (id) {
        id -> Nullable<Integer>,
        name -> Text,
    }
}

diesel::table! {
    designs (id) {
        id -> Nullable<Integer>,
        filename -> Text,
        filepath -> Text,
        image_data -> Nullable<Binary>,
        image_type -> Nullable<Text>,
        width_mm -> Nullable<Double>,
        height_mm -> Nullable<Double>,
        stitch_count -> Nullable<Integer>,
        color_count -> Nullable<Integer>,
        color_change_count -> Nullable<Integer>,
        notes -> Nullable<Text>,
        rating -> Nullable<SmallInt>,
        is_stitched -> Bool,
        tags_checked -> Bool,
        tagging_tier -> Nullable<SmallInt>,
        date_added -> Nullable<Date>,
        designer_id -> Nullable<Integer>,
        source_id -> Nullable<Integer>,
        hoop_id -> Nullable<Integer>,
    }
}

diesel::table! {
    hoops (id) {
        id -> Nullable<Integer>,
        name -> Text,
        max_width_mm -> Double,
        max_height_mm -> Double,
    }
}

diesel::table! {
    project_designs (project_id, design_id) {
        project_id -> Integer,
        design_id -> Integer,
    }
}

diesel::table! {
    projects (id) {
        id -> Nullable<Integer>,
        name -> Text,
        description -> Nullable<Text>,
        date_created -> Nullable<Date>,
    }
}

diesel::table! {
    settings (key) {
        key -> Nullable<Text>,
        value -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    sources (id) {
        id -> Nullable<Integer>,
        name -> Text,
    }
}

diesel::table! {
    tags (id) {
        id -> Nullable<Integer>,
        description -> Text,
        tag_group -> Nullable<Text>,
    }
}

diesel::joinable!(design_tags -> designs (design_id));
diesel::joinable!(design_tags -> tags (tag_id));
diesel::joinable!(designs -> designers (designer_id));
diesel::joinable!(designs -> hoops (hoop_id));
diesel::joinable!(designs -> sources (source_id));
diesel::joinable!(project_designs -> designs (design_id));
diesel::joinable!(project_designs -> projects (project_id));

diesel::allow_tables_to_appear_in_same_query!(
    design_tags,
    designers,
    designs,
    hoops,
    project_designs,
    projects,
    settings,
    sources,
    tags,
);
