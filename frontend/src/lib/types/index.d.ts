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

export interface BrowseDesignSummary {
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

/**
 * A partial subset of BrowseDesignSummary fields that can be mutated
 * during a DesignDetails session and subsequently patched into the
 * browse card grid without a full database re-fetch.
 */
export interface MutationPatch {
  designer?: string;
  source?: string;
  hoop?: string | null;
  projects?: string[];
  tags?: string[];
  imageTags?: string[];
  stitchingTags?: string[];
  rating?: number | null;
  is_stitched?: boolean;
  tagsChecked?: boolean;
}
