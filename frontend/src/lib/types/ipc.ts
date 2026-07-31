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
  image_preference: string;
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
  image_preference: string;
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
