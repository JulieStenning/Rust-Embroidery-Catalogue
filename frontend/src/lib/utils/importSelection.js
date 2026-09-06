/**
 * Pure helpers for the import wizard's Step-2 file selection model.
 *
 * Problem being solved: a first import can hold tens of thousands of scanned
 * files. Keeping that selection as a flat list of full paths makes every single
 * toggle O(total) and forces the review page to materialise one object per file.
 *
 * Instead each scanned folder is selected by default ("select all"). We only
 * remember the *exceptions* to that default:
 *
 *   state.selectedOnly[folder] : list of paths explicitly SELECTED when the
 *                                user has switched the folder to "none".
 *                                An empty array is a sentinel meaning "none".
 *   state.deselected[folder]   : list of paths explicitly DESELECTED when the
 *                                folder is otherwise "all". Never stored empty.
 *
 * A folder absent from both maps means "every file in it is selected", so the
 * default (40k files in one folder) costs almost nothing. All predicates are
 * O(1); only materialiseSelectedPaths() is O(total) and is called once when the
 * Confirm wire is built.
 */

export const IMPORT_UNKNOWN_FOLDER = "Unknown folder";

/**
 * @param {string} fullPath
 * @returns {string}
 */
function folderPathOf(fullPath) {
  const normalized = String(fullPath || "").trim().replace(/\\/g, "/");
  const splitIndex = normalized.lastIndexOf("/");
  if (splitIndex <= 0) return "";
  return normalized.slice(0, splitIndex);
}

/**
 * @param {string} folderPath
 * @returns {string}
 */
function folderLabelOf(folderPath) {
  const value = String(folderPath || "").trim().replace(/\\/g, "/").replace(/\/+$/g, "");
  if (!value) return IMPORT_UNKNOWN_FOLDER;
  const segments = value.split("/").filter(Boolean);
  return segments.length > 0 ? segments[segments.length - 1] : value;
}

/**
 * @param {string} fullPath
 * @returns {string}
 */
function filenameOf(fullPath) {
  const normalized = String(fullPath || "").trim().replace(/\\/g, "/");
  const segments = normalized.split("/").filter(Boolean);
  return segments.length > 0 ? segments[segments.length - 1] : String(fullPath || "");
}

/**
 * Group a raw `preview.scanned_files` array (each with a `full_path`) into
 * folders. Returns a stable list sorted by folder path; each folder holds its
 * file paths sorted by filename.
 *
 * @param {Array<Record<string, any>> | null | undefined} scannedFiles
 * @returns {Array<{folderPath: string, label: string, filePaths: string[]}>}
 */
export function buildImportFolderCatalog(scannedFiles) {
  const grouped = new Map();
  for (const rawFile of Array.isArray(scannedFiles) ? scannedFiles : []) {
    const fullPath = String(rawFile?.full_path || "").trim();
    if (!fullPath) continue;
    const folderPath = folderPathOf(fullPath) || IMPORT_UNKNOWN_FOLDER;
    if (!grouped.has(folderPath)) {
      grouped.set(folderPath, {
        folderPath,
        label: folderLabelOf(folderPath) || folderPath,
        filePaths: [],
      });
    }
    grouped.get(folderPath).filePaths.push(fullPath);
  }
  const records = Array.from(grouped.values());
  for (const record of records) {
    record.filePaths.sort((left, right) =>
      filenameOf(left).localeCompare(filenameOf(right), undefined, { sensitivity: "base" })
    );
  }
  records.sort((left, right) => left.folderPath.localeCompare(right.folderPath));
  return records;
}

/**
 * @returns {{deselected: Record<string, string[]>, selectedOnly: Record<string, string[]>}}
 */
export function selCreate() {
  return { deselected: {}, selectedOnly: {} };
}

function hasKey(map, key) {
  return Object.prototype.hasOwnProperty.call(map, key);
}

function withArrayValue(map, key, value) {
  const next = { ...map };
  next[key] = value;
  return next;
}

function withoutKey(map, key) {
  if (!hasKey(map, key)) return map;
  const next = { ...map };
  delete next[key];
  return next;
}

/**
 * Sanitise a serialised selection (e.g. restored from the session store) so it
 * is safe to use regardless of shape.
 * @param {any} raw
 * @returns {{deselected: Record<string, string[]>, selectedOnly: Record<string, string[]>}}
 */
export function selFromSerialized(raw) {
  /** @type {Record<string, string[]>} */
  const deselected = {};
  /** @type {Record<string, string[]>} */
  const selectedOnly = {};
  if (!raw || typeof raw !== "object") return { deselected, selectedOnly };
  const copyOf = (value) => {
    /** @type {Record<string, string[]>} */
    const out = {};
    if (value && typeof value === "object") {
      for (const key of Object.keys(value)) {
        const list = Array.isArray(value[key])
          ? value[key]
              .map((item) => String(item || "").trim())
              .filter((item) => item.length > 0)
          : [];
        if (list.length > 0) out[key] = list;
      }
    }
    return out;
  };
  return { deselected: copyOf(raw.deselected), selectedOnly: copyOf(raw.selectedOnly) };
}

/**
 * Is the given file selected?
 * @param {any} state
 * @param {string} folderPath
 * @param {string} fullPath
 * @returns {boolean}
 */
export function selIsSelected(state, folderPath, fullPath) {
  if (hasKey(state.selectedOnly, folderPath)) {
    return state.selectedOnly[folderPath].includes(fullPath);
  }
  if (hasKey(state.deselected, folderPath)) {
    return !state.deselected[folderPath].includes(fullPath);
  }
  return true;
}

/**
 * Number of selected files in a folder with `total` scanned files.
 * @param {any} state
 * @param {string} folderPath
 * @param {number} total
 * @returns {number}
 */
export function selCount(state, folderPath, total) {
  if (hasKey(state.selectedOnly, folderPath)) {
    return state.selectedOnly[folderPath].length;
  }
  if (hasKey(state.deselected, folderPath)) {
    return Math.max(0, total - state.deselected[folderPath].length);
  }
  return total;
}

/**
 * Toggle a single file. Immutable: returns a new selection state.
 * @param {any} state
 * @param {string} folderPath
 * @param {string} fullPath
 * @param {boolean} selected
 * @returns {any}
 */
export function selToggleFile(state, folderPath, fullPath, selected) {
  let { deselected, selectedOnly } = state;
  if (selected) {
    if (hasKey(selectedOnly, folderPath)) {
      if (selectedOnly[folderPath].includes(fullPath)) return state;
      selectedOnly = withArrayValue(selectedOnly, folderPath, [
        ...selectedOnly[folderPath],
        fullPath,
      ]);
    } else if (hasKey(deselected, folderPath)) {
      const remaining = deselected[folderPath].filter((item) => item !== fullPath);
      if (remaining.length === 0) deselected = withoutKey(deselected, folderPath);
      else deselected = withArrayValue(deselected, folderPath, remaining);
    } else {
      // Already selected by default.
      return state;
    }
  } else {
    if (hasKey(selectedOnly, folderPath)) {
      const remaining = selectedOnly[folderPath].filter((item) => item !== fullPath);
      // Keep an empty list as the "none selected" sentinel.
      selectedOnly = withArrayValue(selectedOnly, folderPath, remaining);
    } else if (hasKey(deselected, folderPath)) {
      if (deselected[folderPath].includes(fullPath)) return state;
      deselected = withArrayValue(deselected, folderPath, [
        ...deselected[folderPath],
        fullPath,
      ]);
    } else {
      deselected = withArrayValue(deselected, folderPath, [fullPath]);
    }
  }
  return { deselected, selectedOnly };
}

/**
 * Select every file in a folder.
 * @param {any} state
 * @param {string} folderPath
 * @returns {any}
 */
export function selSelectAllInFolder(state, folderPath) {
  return {
    deselected: withoutKey(state.deselected, folderPath),
    selectedOnly: withoutKey(state.selectedOnly, folderPath),
  };
}

/**
 * Select none of the files in a folder.
 * @param {any} state
 * @param {string} folderPath
 * @returns {any}
 */
export function selDeselectAllInFolder(state, folderPath) {
  return {
    deselected: withoutKey(state.deselected, folderPath),
    selectedOnly: withArrayValue(state.selectedOnly, folderPath, []),
  };
}

/**
 * @param {any} state
 * @param {string[]} folderPaths
 * @returns {any}
 */
export function selSelectAllFolders(state, folderPaths) {
  let deselected = state.deselected;
  let selectedOnly = state.selectedOnly;
  for (const folderPath of folderPaths) {
    deselected = withoutKey(deselected, folderPath);
    selectedOnly = withoutKey(selectedOnly, folderPath);
  }
  return { deselected, selectedOnly };
}

/**
 * @param {any} state
 * @param {string[]} folderPaths
 * @returns {any}
 */
export function selDeselectAllFolders(state, folderPaths) {
  let deselected = state.deselected;
  let selectedOnly = state.selectedOnly;
  for (const folderPath of folderPaths) {
    deselected = withoutKey(deselected, folderPath);
    selectedOnly = withArrayValue(selectedOnly, folderPath, []);
  }
  return { deselected, selectedOnly };
}

/**
 * Flatten the selected files to the concrete list of full paths required by the
 * Confirm wire. O(total); call only when the wire is about to be built.
 * @param {any} state
 * @param {Array<Record<string, any>>} catalog
 * @returns {string[]}
 */
export function selMaterializeSelectedPaths(state, catalog) {
  const result = [];
  for (const record of catalog) {
    const folderPath = record.folderPath;
    if (hasKey(state.selectedOnly, folderPath)) {
      for (const fullPath of state.selectedOnly[folderPath]) result.push(fullPath);
    } else if (hasKey(state.deselected, folderPath)) {
      const deselectedSet = new Set(state.deselected[folderPath]);
      for (const fullPath of record.filePaths) {
        if (!deselectedSet.has(fullPath)) result.push(fullPath);
      }
    } else {
      for (const fullPath of record.filePaths) result.push(fullPath);
    }
  }
  return result;
}

