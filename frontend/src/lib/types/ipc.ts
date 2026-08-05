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
  tags_checked: boolean;
  rating: number | null;
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
  tagsChecked: boolean;
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
  tagsChecked?: boolean;
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
  tags_checked: boolean;
  tagging_tier: number | null;
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
  tagsChecked: boolean;
  taggingTier: number | null;
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

export interface SetDesignTagsCheckedRequest {
  tags_checked: boolean;
}

export interface SetDesignTagsRequest {
  tag_ids: number[];
}

export interface DesignCommandResult {
  design_id: number;
  message: string;
}

export interface SettingsViewModel {
  preview_3d_profile: string;
  google_api_key: string;
  has_google_api_key: boolean;
  ai_tier2_auto: boolean;
  ai_tier3_auto: boolean;
  ai_batch_size: string;
  ai_delay: string;
  import_commit_batch_size: string;
  import_last_browse_folder: string;
  can_configure_data_root: boolean;
  data_root: string;
  database_path: string;
  log_folder: string;
  app_mode: string;
  ai_tagging_help_url: string;
}

export interface SaveSettingsRequest {
  preview_3d_profile?: string;
  google_api_key: string;
  ai_tier2_auto: boolean;
  ai_tier3_auto: boolean;
  ai_batch_size: string;
  ai_delay: string;
  import_commit_batch_size: string;
  data_root: string;
}

export interface SaveSettingsResult {
  saved: boolean;
  message: string;
}

export interface BrowseDataRootResult {
  path: string | null;
  error: string | null;
}

export interface AppStatus {
  execution_mode: "portable" | "installed";
  data_root: string;
  embroidery_dir: string;
  database_path: string;
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
  source?: string;
  hoop?: string | null;
  rating?: number | null;
  is_stitched?: boolean;
  tags_checked?: boolean;
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
  ai_tier2_auto: boolean;
  ai_tier3_auto: boolean;
  ai_batch_size: string;
  ai_delay: string;
  import_commit_batch_size: string;
  default_batch_size: number;
  default_commit_every: number;
  default_workers: number;
}

export interface AdapterTaggingActionsViewModelResponse {
  source: string;
  model: TaggingActionsViewModel;
  error?: string;
}

/**
 * Flat view-model describing the options the Tagging Actions screen passes to
 * the command adapter. The adapter translates this into the nested wire shape
 * expected by the Rust `run_unified_backfill` command (see
 * `UnifiedBackfillWireRequest`).
 */
export interface UnifiedBackfillRequest {
  action_mode: string;
  run_tier2: boolean;
  run_tier3: boolean;
  run_images: boolean;
  image_redo: boolean;
  run_color_counts: boolean;
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
    tiers?: number[];
    enabled?: boolean;
  } | null;
  stitching?: {
    clear_existing_stitching?: boolean;
    enabled?: boolean;
  } | null;
  images?: {
    redo?: boolean;
    enabled?: boolean;
  } | null;
  color_counts?: {
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
  clear_existing?: boolean;
  image_redo?: boolean;
}

export interface BackupViewModel {
  db_destination: string;
  designs_destination: string;
  db_source_path: string;
  designs_source_path: string;
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
    tagsChecked: Boolean(wire.tags_checked),
    taggingTier: wire.tagging_tier ?? null,
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
