// Folder assignment resolution contract for import scaffolding.

#[derive(Debug, Clone, Default)]
pub struct FolderAssignment {
    pub folder_path: String,
    pub designer_id: Option<i64>,
    pub source_id: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct AssignmentFallback {
    pub designer_id: Option<i64>,
    pub source_id: Option<i64>,
}

pub fn resolve_assignment(
    per_folder: &FolderAssignment,
    fallback: &AssignmentFallback,
) -> FolderAssignment {
    FolderAssignment {
        folder_path: per_folder.folder_path.clone(),
        designer_id: per_folder.designer_id.or(fallback.designer_id),
        source_id: per_folder.source_id.or(fallback.source_id),
    }
}