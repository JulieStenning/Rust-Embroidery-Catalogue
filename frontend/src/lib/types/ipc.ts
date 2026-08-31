export interface BrowseAdditionalFilters {
  designer_filters?: string[];
  image_tag_filters?: string[];
  stitching_tag_filters?: string[];
  source_filters?: string[];
  hoop_size?: string | null;
  min_rating?: number | null;
  stitched_status?: "all" | "yes" | "no" | null;
}

export interface SearchPayload {
  q?: string;
  search_file_name?: boolean;
  search_tags?: boolean;
  search_folder_name?: boolean;
  unverified_only?: boolean;
  additional_filters?: BrowseAdditionalFilters;
  page?: number;
  page_size?: number;
  sort_by?: string;
  sort_dir?: string;
}

/**
 * Frontend-only browse filter state (kept in the browse session store so the
 * browse view survives the detail round-trip). Mirrors BrowseView's local
 * `browseFilters` object.
 */
export interface BrowseFilterState {
  q: string;
  allWords: string;
  exactPhrase: string;
  anyWords: string;
  noneWords: string;
  filename: string;
  designerFilters: string[];
  imageTagFilters: string[];
  stitchingTagFilters: string[];
  hoop: string;
  sourceFilters: string[];
  rating: string;
  stitched: string;
  unverifiedOnly: boolean;
  searchFilename: boolean;
  searchTags: boolean;
  searchFolder: boolean;
  sortBy: string;
  sortDir: string;
}

/** Progress streamed from Rust during catalogue storage migration. */
export interface StorageMigrationProgress {
  current_phase:
    | "preflight"
    | "database"
    | "assets"
    | "finalising"
    | "completed"
    | "cancelled"
    | "error"
    | string;
  items_copied: number;
  total_items: number;
  bytes_copied: number;
  total_bytes: number;
  status_message: string;
  percent: number;
  error: string | null;
}

/** Final result returned by the migration command. */
export interface StorageMigrationSummary {
  success: boolean;
  source_root: string;
  target_root: string;
  database_bytes: number;
  asset_items: number;
  asset_bytes: number;
  requires_restart: boolean;
}

export interface BrowseDesignSummaryWire {
  id: number;
  filename: string;
  filepath: string;
  designer: string;
  source: string;
  hoop: string | null;
  projects: string[];
  tags: string[];
  image_tags: string[];
  stitching_tags: string[];
  is_stitched: boolean;
  image_tags_verified: boolean;
  stitching_tags_verified: boolean;
  rating: number | null;
  date_added?: string | null;
}

export interface BrowseDesignCard {
  id: number;
  filename: string;
  filepath: string;
  designer: string;
  source: string;
  hoop: string;
  projects: string[];
  tags: string[];
  imageTags: string[];
  stitchingTags: string[];
  isStitched: boolean;
  imageTagsVerified: boolean;
  stitchingTagsVerified: boolean;
  rating: number | null;
  folder: string;
  dateAdded: string;
}

export interface MutationPatch {
  designer?: string;
  source?: string;
  hoop?: string | null;
  projects?: string[];
  tags?: string[];
  imageTags?: string[];
  stitchingTags?: string[];
  rating?: number | null;
  isStitched?: boolean;
  imageTagsVerified?: boolean;
  stitchingTagsVerified?: boolean;
}

export interface ProjectListItem {
  id: number;
  name: string;
}

export interface BrowseTagOption {
  id: number;
  description: string;
  tag_group: string | null;
  is_system: boolean | null;
}

export interface DesignLookupOption {
  id: number;
  name: string;
}

export interface DesignTagDetail {
  id: number;
  description: string;
  tag_group: string | null;
}

export interface DesignDetailWire {
  id: number;
  filename: string;
  filepath: string;
  image_type: string | null;
  image_data_url: string | null;
  width_mm: number | null;
  height_mm: number | null;
  stitch_count: number | null;
  color_count: number | null;
  color_change_count: number | null;
  designer: string;
  designer_id: number | null;
  source: string;
  source_id: number | null;
  hoop: string | null;
  hoop_id: number | null;
  notes: string | null;
  rating: number | null;
  is_stitched: boolean;
  image_tags_verified: boolean;
  stitching_tags_verified: boolean;
  tagging_mode: string | null;
  date_added: string | null;
  tags: DesignTagDetail[];
  projects: ProjectListItem[];
  available_projects: ProjectListItem[];
  all_tags: BrowseTagOption[];
  designers: DesignLookupOption[];
  sources: DesignLookupOption[];
  hoops: DesignLookupOption[];
}

export interface DesignDetail {
  id: number;
  filename: string;
  filepath: string;
  imageType: string | null;
  imageDataUrl: string | null;
  widthMm: number | null;
  heightMm: number | null;
  stitchCount: number | null;
  colorCount: number | null;
  colorChangeCount: number | null;
  designer: string;
  designerId: number | null;
  source: string;
  sourceId: number | null;
  hoop: string | null;
  hoopId: number | null;
  notes: string | null;
  rating: number | null;
  isStitched: boolean;
  imageTagsVerified: boolean;
  stitchingTagsVerified: boolean;
  taggingMode: string | null;
  dateAdded: string | null;
  tags: DesignTagDetail[];
  projects: ProjectListItem[];
  availableProjects: ProjectListItem[];
  allTags: BrowseTagOption[];
  designers: DesignLookupOption[];
  sources: DesignLookupOption[];
  hoops: DesignLookupOption[];
}

export interface UpdateDesignMetadataRequest {
  notes?: string | null;
  designer_id?: number | null;
  source_id?: number | null;
  hoop_id?: number | null;
}

export interface SetDesignRatingRequest {
  rating: number | null;
}

export interface SetDesignStitchedRequest {
  is_stitched: boolean;
}

export interface SetDesignVerificationRequest {
  image_tags_verified?: boolean | null;
  stitching_tags_verified?: boolean | null;
}

export interface SetDesignTagsRequest {
  tag_ids: number[];
  image_tags_verified?: boolean | null;
  stitching_tags_verified?: boolean | null;
}

/**
 * Wire shape for `bulk_set_tags_for_designs`. Explicit add/remove lists so
 * tags left untouched (indeterminate / mixed in the bulk UI) are never
 * touched by the backend.
 */
export interface BulkApplyTagsRequest {
  tags_to_add: number[];
  tags_to_remove: number[];
  clear_all_tags: boolean;
  image_tags_verified?: boolean | null;
  stitching_tags_verified?: boolean | null;
}

export interface DesignCommandResult {
  design_id: number;
  message: string;
}

export interface SettingsViewModel {
  preview_3d_profile: string;
  google_api_key: string;
  has_google_api_key: boolean;
  ai_vision_auto: boolean;
  ai_batch_size: string;
  ai_delay: string;
  ai_gemini_model: string;
  ai_commit_every: string;
  ai_workers: string;
  ai_free_tier: boolean;
  import_commit_batch_size: string;
  import_last_browse_folder: string;
  can_configure_data_root: boolean;
  data_root: string;
  library_root: string;
  database_path: string;
  log_folder: string;
  app_mode: string;
  ai_tagging_help_url: string;
  db_idle_check_interval_secs: string;
}

export interface SaveSettingsRequest {
  preview_3d_profile?: string;
  google_api_key: string;
  ai_vision_auto: boolean;
  ai_batch_size: string;
  ai_delay: string;
  ai_gemini_model: string;
  ai_commit_every: string;
  ai_workers: string;
  ai_free_tier: boolean;
  import_commit_batch_size: string;
  data_root: string;
  db_idle_check_interval_secs?: string;
}

/** Result of the Settings "Test model" button. */
export interface GeminiModelTestResult {
  ok: boolean;
  message: string;
}

export interface SaveSettingsResult {
  saved: boolean;
  message: string;
}

export interface BrowseDataRootResult {
  path: string | null;
  error: string | null;
}

export interface AdapterGoogleApiKeyResponse {
  source: string;
  key: string;
  persisted?: boolean;
  error?: string;
}

export interface DbStats {
  file_size_bytes: number;
  page_count: number;
  freelist_count: number;
  page_size: number;
  free_ratio: number;
  reclaimable_bytes: number;
}

export interface CompactResult {
  file_size_before: number;
  file_size_after: number;
  pages_reclaimed: number;
  duration_ms: number;
}

export interface AdapterDbStatsResponse {
  source: string;
  stats: DbStats | null;
  error?: string;
}

export interface AdapterCompactResponse {
  source: string;
  result: CompactResult | null;
  message: string;
  error?: string;
}

export interface AppStatus {
  execution_mode: "dev" | "installed";
  data_root: string;
  embroidery_dir: string;
  database_path: string;
  data_root_missing: boolean;
  /** True when a configured data root exists but the database file is missing. */
  database_missing: boolean;
}

/** Tri-state status of the configured database reported at startup. */
export type DatabaseStatusKind = "uninitialized" | "connected" | "missing";

/** Detailed database status used by the recovery flow. */
export interface DatabaseStatus {
  status: DatabaseStatusKind;
  configured_data_root: string | null;
  database_path: string | null;
  embroidery_dir: string | null;
  data_root_missing: boolean;
}

/** Result of a drive-letter relocation scan. */
export interface DetectedDataRoot {
  data_root: string | null;
  relative_subpath: string;
}

/** Validation result for a candidate data root. */
export interface DatabaseValidation {
  valid: boolean;
  data_root: string;
  database_path: string;
  embroidery_dir: string;
  embroidery_dir_exists: boolean;
  error: string | null;
}

export interface BrowseDesignPreview {
  id: number;
  data_url: string | null;
}

export interface DesignImageData {
  design_id: number;
  image_type: string | null;
  data_url: string | null;
}

export interface ReparseDesignResultWire {
  design_id: number;
  width_mm: number | null;
  height_mm: number | null;
  stitch_count: number | null;
  color_count: number | null;
  color_change_count: number | null;
  hoop_id: number | null;
  hoop: string | null;
  message: string;
}

export interface ReparseDesignResult {
  designId: number;
  widthMm: number | null;
  heightMm: number | null;
  stitchCount: number | null;
  colorCount: number | null;
  colorChangeCount: number | null;
  hoopId: number | null;
  hoop: string | null;
  message: string;
}

export interface AdapterReparseDesignResponse {
  source: string;
  persisted: boolean;
  result: ReparseDesignResult | null;
  message: string;
  error?: string;
}

export interface AdapterListResponse<TItem> {
  source: string;
  items: TItem[];
  error?: string;
}

export interface AdapterItemResponse<TItem> {
  source: string;
  item: TItem | null;
  error?: string;
}

export interface AdapterMutationResponse {
  source: string;
  persisted: boolean;
  design_id: number;
  message: string;
  error?: string;
}

export interface AdapterSettingsViewModelResponse {
  source: string;
  model: SettingsViewModel;
  error?: string;
}

export interface AdapterSaveSettingsResponse {
  source: string;
  saved: boolean;
  message: string;
  persisted: boolean;
}

export interface AdapterBrowseDataRootResponse {
  source: string;
  path: string | null;
  error: string | null;
}

export interface AdapterAppStatusResponse {
  source: string;
  status: AppStatus | null;
  error?: string;
}

export interface ProjectSummary {
  id: number;
  name: string;
  description?: string | null;
  design_count?: number;
  date_created?: string | null;
}

export interface ProjectDetailModel {
  id: number;
  name: string;
  description: string | null;
}

export interface ProjectDesignItem {
  id: number;
  filename: string;
  filepath: string;
  image_data_url?: string | null;
  designer?: string;
  designer_name?: string | null;
  source?: string;
  hoop?: string | null;
  rating?: number | null;
  is_stitched?: boolean;
  tags_checked?: boolean;
  has_image?: boolean;
  width_mm?: number | null;
  height_mm?: number | null;
  stitch_count?: number | null;
  color_count?: number | null;
  color_change_count?: number | null;
  notes?: string | null;
  date_added?: string | null;
}

export interface ProjectDetailView {
  project: ProjectDetailModel | null;
  designs: ProjectDesignItem[];
}

export interface ProjectMutationResult {
  project_id: number;
  message: string;
}

export interface RemoveProjectDesignResult {
  project_id: number;
  design_id: number;
  message: string;
}

export interface AdapterProjectListResponse {
  source: string;
  items: ProjectSummary[];
  error?: string;
}

export interface AdapterProjectDetailResponse {
  source: string;
  item: ProjectDetailView | null;
  error?: string;
}

export interface AdapterProjectMutationResponse {
  source: string;
  persisted: boolean;
  project_id: number;
  message: string;
  error?: string;
}

export interface AdapterProjectDesignMutationResponse {
  source: string;
  persisted: boolean;
  project_id: number;
  design_id: number;
  message: string;
  error?: string;
}

export interface BulkImportScannedFile {
  full_path?: string;
  [key: string]: unknown;
}

export interface BulkImportPreview {
  discovered_count: number;
  selected_count: number;
  folder_count: number;
  scanned_files: BulkImportScannedFile[];
  resolved_assignments: unknown[];
  missing_root: boolean;
  no_supported_files: boolean;
  invalid_root: boolean;
}

export interface AdapterImportPreviewResponse {
  source: string;
  preview: BulkImportPreview;
  message: string;
}

export interface BrowseImportFolderResult {
  path?: string | null;
  paths?: string[];
}

export interface AdapterBrowseImportFolderResponse {
  source: string;
  path: string;
  paths: string[];
  message: string;
}

export interface ImportPrecheckResult {
  context_token: string;
  context_token_present: boolean;
  ready_for_confirm: boolean;
  is_first_import: boolean;
  needs_hoop_setup: boolean;
  root_path_count: number;
  selected_file_count: number;
  resolved_assignments: unknown[];
}

export interface AdapterImportPrecheckResponse {
  source: string;
  precheck: ImportPrecheckResult;
  message: string;
}

export interface ImportConfirmResult {
  persisted_design_count?: number;
  [key: string]: unknown;
}

export interface ImportPrecheckActionResult {
  action: string;
  context_token_present: boolean;
  consumed_context: boolean;
  requires_skip_hoops_confirmation: boolean;
  next_route: string | null;
  confirm_result: ImportConfirmResult | null;
}

export interface AdapterImportPrecheckActionResponse {
  source: string;
  actionResult: ImportPrecheckActionResult;
  message: string;
}

export interface AdapterStopBulkImportResponse {
  source: string;
  stopRequested: boolean;
  message: string;
}

export interface TaggingActionsViewModel {
  has_google_api_key: boolean;
  ai_vision_auto: boolean;
  ai_batch_size: string;
  ai_delay: string;
  ai_commit_every: string;
  ai_workers: string;
  ai_free_tier: boolean;
  import_commit_batch_size: string;
  default_batch_size: number;
  default_commit_every: number;
  default_workers: number;
  default_delay: number;
}

export interface AdapterTaggingActionsViewModelResponse {
  source: string;
  model: TaggingActionsViewModel;
  error?: string;
}

/** Total / unverified / verified candidate counts for a tagging scope. */
export interface TaggingScopeCounts {
  total_count: number;
  unverified_count: number;
  verified_count: number;
}

export interface AdapterTaggingCandidateCountResponse {
  source: string;
  /** The backend scope action the count reflects (`tag_untagged`, `retag_all_unverified`, `retag_all`). */
  action: string;
  counts: TaggingScopeCounts;
  error?: string;
}

/**
 * Flat view-model describing the options the Tagging Actions screen passes to
 * the command adapter. The adapter translates this into the nested wire shape
 * expected by the Rust `run_unified_backfill` command (see
 * `UnifiedBackfillWireRequest`).
 */
export interface UnifiedBackfillRequest {
  /**
   * Tagging scope action: `"tag_untagged"` (designs with no image tags),
   * `"retag_all_unverified"` (designs not yet scanned with Visual AI), or
   * `"retag_all"` (every design).
   */
  action_mode: string;
  /**
   * Tagging modes to run: `"path_rule"` (File & Folder Rules) and/or `"ai_vision"`
   * (Visual AI). When omitted the adapter falls back to the legacy
   * `run_vision`-derived modes (`path_rule`, plus `ai_vision` when run_vision).
   */
  modes?: string[];
  /**
   * How existing image-group tags are handled: `"add"` (append only, keep
   * existing) or `"reset"` (clear and re-tag). Non-image / manually-added tags
   * are never touched.
   */
  merge_mode?: string;
  /**
   * When `true`, human-verified designs (`image_tags_verified = 1`) are excluded
   * from the candidate pool. Defaults to `true` (Recommended).
   */
  exclude_verified?: boolean;
  run_vision: boolean;
  run_images: boolean;
  image_redo: boolean;
  run_color_counts: boolean;
  run_hoop_dimensions: boolean;
  commit_every: number;
  batch_size: number;
  workers: number;
}

/**
 * Nested wire shape matching the Rust `backfill::UnifiedBackfillRequest` +
 * `backfill::UnifiedBackfillActions` structs exactly. Serde field names are
 * snake_case, and optional sections are represented via null when disabled.
 */
export interface UnifiedBackfillActionsWire {
  tagging?: {
    action?: string;
    modes?: string[];
    merge_mode?: string;
    exclude_verified?: boolean;
    enabled?: boolean;
  } | null;
  stitching?: {
    clear_stitching_mode?: string;
    enabled?: boolean;
  } | null;
  images?: {
    redo?: boolean;
    enabled?: boolean;
  } | null;
  color_counts?: {
    enabled?: boolean;
  } | null;
  hoop_dimensions?: {
    enabled?: boolean;
  } | null;
  fingerprinting?: {
    enabled?: boolean;
  } | null;
}

export interface UnifiedBackfillWireRequest {
  actions?: UnifiedBackfillActionsWire | null;
  batch_size?: number | null;
  commit_every?: number | null;
  workers?: number | null;
  delay_seconds?: number | null;
  vision_delay_seconds?: number | null;
}

export interface UnifiedBackfillResult {
  source: string;
  processed: number;
  errors: number;
  stopped: boolean;
  actions: string[];
  commit_every?: number;
  batch_size?: number;
  workers?: number;
  stitching_tag_count_before?: number;
  stitching_tag_count_after?: number;
  error?: string;
}

export interface AdapterStopUnifiedBackfillResponse {
  source: string;
  status: string;
  error?: string;
}

export interface BackfillLogEntry {
  level: string;
  message: string;
}

export interface AdapterBackfillLogEntriesResponse {
  source: string;
  entries: BackfillLogEntry[];
}

export interface RunStitchingBackfillOptions {
  commit_every?: number;
  batch_size?: number;
  workers?: number;
  clear_stitching_mode?: string;
  image_redo?: boolean;
}

export interface BackupViewModel {
  db_destination: string;
  designs_destination: string;
  db_source_path: string;
  designs_source_path: string;
  /** Epoch-seconds string of the last successful database backup (if any). */
  db_last_backup_at: string;
  /** Epoch-seconds string of the last successful designs backup (if any). */
  designs_last_backup_at: string;
}

export interface AdapterBackupViewModelResponse {
  source: string;
  model: BackupViewModel;
  error?: string;
}

export interface SaveBackupSettingsRequest {
  dbDestination: string;
  designsDestination: string;
}

export interface AdapterSaveBackupSettingsResponse {
  source: string;
  persisted: boolean;
  saved: boolean;
  message: string;
  db_destination?: string;
  designs_destination?: string;
  error?: string;
}

export interface AdapterBrowseBackupFolderResponse {
  source: string;
  path: string | null;
  error: string | null;
}

export interface DatabaseBackupResult {
  success: boolean;
  backup_path: string;
  size_bytes: number;
  completed_at: string;
  error: string;
  cancelled: boolean;
}

export interface DesignsBackupResult {
  success: boolean;
  scanned: number;
  copied: number;
  updated: number;
  unchanged: number;
  archived: number;
  total_bytes_copied: number;
  completed_at: string;
  error: string;
  cancelled: boolean;
}

/** Result of raising the cooperative backup cancellation flag in Rust. */
export interface CancelBackupResult {
  cancel_requested: boolean;
}

export interface AdapterRunBothBackupsResponse {
  source: string;
  database: DatabaseBackupResult | null;
  designs: DesignsBackupResult | null;
  error?: string;
}

export interface AdapterScanOrphansResponse {
  source: string;
  checked: number;
  found: number;
  error?: string;
}

// ---------------------------------------------------------------------------
// Restore (inverse of backup)
// ---------------------------------------------------------------------------

export interface BrowseRestoreFileResponse {
  source: string;
  path: string | null;
  error: string | null;
}

export interface RestoreDatabaseResult {
  success: boolean;
  restored_path: string;
  rollback_copy_path: string | null;
  design_count: number;
  /** `PRAGMA user_version` of the restored database (schema hint). */
  schema_version_hint: number | null;
  /** `PRAGMA user_version` of the live database before the swap. */
  previous_schema_version_hint: number | null;
  rolled_back: boolean;
  error: string | null;
}

export interface RestoreDesignsResult {
  success: boolean;
  scanned: number;
  copied: number;
  updated: number;
  skipped: number;
  total_bytes_copied: number;
  error: string | null;
}

export interface RestoreBothResult {
  database: RestoreDatabaseResult | null;
  designs: RestoreDesignsResult | null;
  unmatched: DetectUnmatchedFilesResult | null;
  /** Set when the whole restore_both call fails (e.g. designs folder missing). */
  error?: string;
}

/** Payload streamed on `catalogue-restore-progress`. */
export interface RestoreProgress {
  phase: string;
  db_status: string;
  scanned: number;
  copied: number;
  skipped: number;
  total_bytes: number;
  percent: number;
  error: string | null;
}

/**
 * Payload streamed on `backfill-progress` from the unified backfill run.
 * `stage` is one of `started`, `batch_committed`, `stopped` or `completed`.
 */
export interface BackfillProgress {
  stage: string;
  processed: number;
  errors: number;
  current_action: string;
}

export interface DetectUnmatchedFilesResult {
  checked: number;
  unmatched: number;
  sample: string[];
}

export interface ImportUnmatchedFilesResult {
  detected: number;
  imported: number;
  failed: number;
  failed_samples: string[];
}

export interface CancelRestoreResult {
  cancel_requested: boolean;
}

export interface OrphanPageItem {
  id: number;
  filename: string;
  filepath: string;
  designer: string;
  date_added: string | null;
}

export interface AdapterOrphansPageResponse {
  source: string;
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
  items: OrphanPageItem[];
  error?: string;
}

export interface AdapterBrowseDesignsPageResponse {
  source: string;
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
  items: BrowseDesignSummaryWire[];
  error?: string;
}

export interface AdapterDeleteOrphansResponse {
  source: string;
  persisted: boolean;
  deleted: number;
  error?: string;
}

export interface AdapterBrowseOrphanPathResponse {
  source: string;
  ok: boolean;
  opened: string;
  error?: string;
}

export interface AdminEntitySummary {
  id: number;
  name: string;
  design_count: number;
}

export interface AdminTagSummary {
  id: number;
  description: string;
  tag_group: string;
  design_count: number;
  is_system: boolean;
}

export interface AdminHoopSummary {
  id: number;
  name: string;
  max_width_mm: number;
  max_height_mm: number;
  design_count: number;
}

export interface ConfigureDataRootResult {
  data_root: string;
  existing_database_detected: boolean;
  database_path: string;
}

export interface AdapterConfigureDataRootResponse extends AdapterPersistedResponse {
  data_root?: string;
  existing_database_detected?: boolean;
  database_path?: string;
}

export interface AdapterPersistedResponse {
  source: string;
  persisted: boolean;
  error?: string;
}

export interface AdapterPersistedItemResponse<TItem> extends AdapterPersistedResponse {
  item?: TItem;
}

export function mapReparseDesignFromWire(wire: ReparseDesignResultWire): ReparseDesignResult {
  return {
    designId: Number(wire.design_id),
    widthMm: wire.width_mm ?? null,
    heightMm: wire.height_mm ?? null,
    stitchCount: wire.stitch_count ?? null,
    colorCount: wire.color_count ?? null,
    colorChangeCount: wire.color_change_count ?? null,
    hoopId: wire.hoop_id ?? null,
    hoop: wire.hoop ?? null,
    message: String(wire.message || ""),
  };
}

export function mapDesignDetailFromWire(wire: DesignDetailWire): DesignDetail {
  return {
    id: Number(wire.id),
    filename: String(wire.filename || ""),
    filepath: String(wire.filepath || ""),
    imageType: wire.image_type ?? null,
    imageDataUrl: wire.image_data_url ?? null,
    widthMm: wire.width_mm ?? null,
    heightMm: wire.height_mm ?? null,
    stitchCount: wire.stitch_count ?? null,
    colorCount: wire.color_count ?? null,
    colorChangeCount: wire.color_change_count ?? null,
    designer: String(wire.designer || ""),
    designerId: wire.designer_id ?? null,
    source: String(wire.source || ""),
    sourceId: wire.source_id ?? null,
    hoop: wire.hoop ?? null,
    hoopId: wire.hoop_id ?? null,
    notes: wire.notes ?? null,
    rating: wire.rating ?? null,
    isStitched: Boolean(wire.is_stitched),
    imageTagsVerified: Boolean(wire.image_tags_verified),
    stitchingTagsVerified: Boolean(wire.stitching_tags_verified),
    taggingMode: wire.tagging_mode ?? null,
    dateAdded: wire.date_added ?? null,
    tags: Array.isArray(wire.tags) ? wire.tags : [],
    projects: Array.isArray(wire.projects) ? wire.projects : [],
    availableProjects: Array.isArray(wire.available_projects) ? wire.available_projects : [],
    allTags: Array.isArray(wire.all_tags) ? wire.all_tags : [],
    designers: Array.isArray(wire.designers) ? wire.designers : [],
    sources: Array.isArray(wire.sources) ? wire.sources : [],
    hoops: Array.isArray(wire.hoops) ? wire.hoops : [],
  };
}
