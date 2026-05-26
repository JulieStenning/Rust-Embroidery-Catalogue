// SQLx does not require a generated schema module.
// Table name constants are defined here for code readability and to avoid
// hard-coding strings throughout the codebase.

pub mod tables {
    pub const SETTINGS: &str = "settings";
    pub const DESIGNS: &str = "designs";
    pub const DESIGNERS: &str = "designers";
    pub const SOURCES: &str = "sources";
    pub const HOOPS: &str = "hoops";
    pub const TAGS: &str = "tags";
    pub const PROJECTS: &str = "projects";
    pub const DESIGN_TAGS: &str = "design_tags";
    pub const PROJECT_DESIGNS: &str = "project_designs";
}
