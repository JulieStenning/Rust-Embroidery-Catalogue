// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { invokeLoose } from "./ipcClient";
import type {
  AdapterPersistedItemResponse,
  AdapterPersistedResponse,
  AdapterListResponse,
  AdminTagSummary,
  AdminTagSynonymGroup,
  AdminTagSynonymKeyword,
  BrowseTagOption,
} from "../types/ipc";

/**
 * Load tag options for browse filters and bulk-tag modal.
 * Falls back to tag names derived from mock designs while migration is in progress.
 */
export async function getBrowseTags(): Promise<AdapterListResponse<BrowseTagOption>> {
  try {
    const tags = await invokeLoose<BrowseTagOption[]>("get_tags_for_browse");
    if (Array.isArray(tags)) {
      return {
        items: tags.map((tag) => ({
          id: Number(tag?.id),
          description: String(tag?.description || ""),
          tag_group: tag?.tag_group == null ? null : String(tag.tag_group),
          is_system: tag?.is_system == null ? false : Boolean(tag.is_system),
        })),
        source: "rust",
      };
    }
    return {
      items: [],
      source: "rust",
      error: "get_tags_for_browse returned an unexpected payload.",
    };
  } catch (error) {
    return { items: [], source: "mock", error: String(error) };
  }
}

/**
 * Apply an explicit add/remove tag diff across a batch of designs.
 *
 * Tags left untouched (indeterminate / mixed in the UI) are simply excluded
 * from both lists, so the backend never touches them. This prevents the
 * previous blanket "replace all tags" behaviour from accidentally removing
 * tags that existed on only some selected designs.
 *
 * @param {Array<number | string>} designIds
 * @param {Array<number | string>} tagsToAdd - Tags to add to ALL selected designs.
 * @param {Array<number | string>} [tagsToRemove=[]] - Tags to remove from ALL selected designs.
 * @param {boolean} [clearAllTags=false] - Clear all tags from the selected designs first.
 * @param {{ imageTagsVerified?: boolean | null, stitchingTagsVerified?: boolean | null }} [verification]
 *   Optional per-category verification overrides. `null` / absent means "leave
 *   that category's existing verification flag untouched" — the backend will
 *   NOT clear a prior verified status for a category with no change.
 */
export async function bulkSetTagsForDesigns(
  designIds: Array<number | string>,
  tagsToAdd: Array<number | string>,
  tagsToRemove: Array<number | string> = [],
  clearAllTags = false,
  verification?: { imageTagsVerified?: boolean | null; stitchingTagsVerified?: boolean | null }
) {
  const normalizedDesignIds =
    designIds && typeof designIds[Symbol.iterator] === "function"
      ? Array.from(designIds)
          .map((id) => Number(id))
          .filter((id) => Number.isFinite(id) && id > 0)
      : [];
  const normalizedAddIds =
    tagsToAdd && typeof tagsToAdd[Symbol.iterator] === "function"
      ? Array.from(
          new Set(
            Array.from(tagsToAdd)
              .map((id) => Number(id))
              .filter((id) => Number.isFinite(id) && id > 0)
          )
        )
      : [];
  const normalizedRemoveIds =
    tagsToRemove && typeof tagsToRemove[Symbol.iterator] === "function"
      ? Array.from(
          new Set(
            Array.from(tagsToRemove)
              .map((id) => Number(id))
              .filter((id) => Number.isFinite(id) && id > 0)
          )
        )
      : [];

  if (normalizedDesignIds.length === 0) {
    return {
      source: "mock",
      requested_count: 0,
      updated_count: 0,
      persisted: false,
    };
  }

  try {
    const result = await invokeLoose("bulk_set_tags_for_designs", {
      designIds: normalizedDesignIds,
      request: {
        tags_to_add: normalizedAddIds,
        tags_to_remove: normalizedRemoveIds,
        clear_all_tags: Boolean(clearAllTags),
        image_tags_verified: verification?.imageTagsVerified ?? null,
        stitching_tags_verified: verification?.stitchingTagsVerified ?? null,
      },
    });
    return {
      source: "rust",
      requested_count: Number(result?.requested_count ?? normalizedDesignIds.length),
      updated_count: Number(result?.updated_count ?? 0),
      persisted: true,
    };
  } catch (error) {
    console.info("bulk_set_tags_for_designs unavailable or failed, using local fallback.", error);
    return {
      source: "mock",
      requested_count: normalizedDesignIds.length,
      updated_count: normalizedDesignIds.length,
      persisted: false,
    };
  }
}

export async function listTags(): Promise<AdapterListResponse<AdminTagSummary>> {
  try {
    const items = await invokeLoose<AdminTagSummary[]>("list_tags");
    if (Array.isArray(items)) {
      return {
        source: "rust",
        items: items.map((item) => ({
          id: Number(item?.id),
          description: String(item?.description || ""),
          tag_group: item?.tag_group == null ? "" : String(item.tag_group),
          design_count: Number(item?.design_count ?? 0),
          is_system: Boolean(item?.is_system ?? false),
        })),
      };
    }
    return {
      source: "rust",
      items: [],
      error: "list_tags returned an unexpected payload.",
    };
  } catch (error) {
    return {
      source: "mock",
      items: [],
      error: String(error),
    };
  }
}

/**
 * @param {string} description
 * @param {string | null} tagGroup
 */
export async function createTag(
  description: string,
  tagGroup: string | null
): Promise<AdapterPersistedItemResponse<AdminTagSummary>> {
  try {
    const item = await invokeLoose<AdminTagSummary>("create_tag", {
      request: {
        description,
        tag_group: tagGroup,
      },
    });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        description: String(item?.description || ""),
        tag_group: item?.tag_group == null ? "" : String(item.tag_group),
        design_count: Number(item?.design_count ?? 0),
        is_system: Boolean(item?.is_system ?? false),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * @param {number | string} tagId
 * @param {string | null} tagGroup
 */
export async function setTagGroup(
  tagId: number | string,
  tagGroup: string | null
): Promise<AdapterPersistedItemResponse<AdminTagSummary>> {
  try {
    const item = await invokeLoose<AdminTagSummary>("set_tag_group", {
      request: { tag_id: Number(tagId), tag_group: tagGroup },
    });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        description: String(item?.description || ""),
        tag_group: item?.tag_group == null ? "" : String(item.tag_group),
        design_count: Number(item?.design_count ?? 0),
        is_system: Boolean(item?.is_system ?? false),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * @param {number | string} tagId
 * @param {string} description
 */
export async function updateTag(
  tagId: number | string,
  description: string
): Promise<AdapterPersistedItemResponse<AdminTagSummary>> {
  try {
    const item = await invokeLoose<AdminTagSummary>("update_tag", {
      request: {
        tag_id: Number(tagId),
        description,
      },
    });
    return {
      source: "rust",
      persisted: true,
      item: {
        id: Number(item?.id),
        description: String(item?.description || ""),
        tag_group: item?.tag_group == null ? "" : String(item.tag_group),
        design_count: Number(item?.design_count ?? 0),
        is_system: Boolean(item?.is_system ?? false),
      },
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * @param {number | string} tagId
 */
export async function deleteTag(tagId: number | string): Promise<AdapterPersistedResponse> {
  try {
    await invokeLoose("delete_tag", { tagId: Number(tagId) });
    return { source: "rust", persisted: true };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * List all tags and their configured word matches grouped by tag.
 */
export async function listTagSynonymsGrouped(): Promise<AdapterListResponse<AdminTagSynonymGroup>> {
  try {
    const groups = await invokeLoose<AdminTagSynonymGroup[]>("list_tag_synonyms_grouped");
    if (Array.isArray(groups)) {
      return {
        items: groups.map((g) => ({
          tag_id: Number(g?.tag_id),
          tag_description: String(g?.tag_description || ""),
          tag_group: g?.tag_group == null ? null : String(g.tag_group),
          keywords: Array.isArray(g?.keywords)
            ? g.keywords.map((k) => ({
                id: Number(k?.id),
                keyword: String(k?.keyword || ""),
              }))
            : [],
        })),
        source: "rust",
      };
    }
    return { items: [], source: "rust", error: "Unexpected payload for tag synonyms." };
  } catch (error) {
    return { items: [], source: "mock", error: String(error) };
  }
}

/**
 * Add word matches for a specific tag.
 * @param {number | string} tagId
 * @param {string} wordsInput
 */
export async function addTagSynonyms(
  tagId: number | string,
  wordsInput: string
): Promise<AdapterPersistedItemResponse<AdminTagSynonymKeyword[]>> {
  try {
    const keywords = await invokeLoose<AdminTagSynonymKeyword[]>("add_tag_synonyms", {
      request: {
        tag_id: Number(tagId),
        words_input: wordsInput,
      },
    });
    return {
      source: "rust",
      persisted: true,
      item: Array.isArray(keywords)
        ? keywords.map((k) => ({
            id: Number(k?.id),
            keyword: String(k?.keyword || ""),
          }))
        : [],
    };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * Delete a single word match by synonym ID.
 * @param {number | string} synonymId
 */
export async function deleteTagSynonym(
  synonymId: number | string
): Promise<AdapterPersistedResponse> {
  try {
    await invokeLoose("delete_tag_synonym", { synonymId: Number(synonymId) });
    return { source: "rust", persisted: true };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}

/**
 * Delete all word matches for a specific tag.
 * @param {number | string} tagId
 */
export async function deleteAllTagSynonymsForTag(
  tagId: number | string
): Promise<AdapterPersistedResponse> {
  try {
    await invokeLoose("delete_all_tag_synonyms_for_tag", { tagId: Number(tagId) });
    return { source: "rust", persisted: true };
  } catch (error) {
    return { source: "mock", persisted: false, error: String(error) };
  }
}
