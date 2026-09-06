import { describe, it, expect } from "vitest";
import {
  IMPORT_UNKNOWN_FOLDER,
  buildImportFolderCatalog,
  selCreate,
  selIsSelected,
  selCount,
  selToggleFile,
  selSelectAllInFolder,
  selDeselectAllInFolder,
  selSelectAllFolders,
  selDeselectAllFolders,
  selMaterializeSelectedPaths,
  selFromSerialized,
} from "../importSelection.js";

const FOLDER = "C:/Designs/Rose Studio";
const files = (n = 3) =>
  buildImportFolderCatalog(
    Array.from({ length: n }, (_, i) => ({
      full_path: `${FOLDER}/design-${i}.pes`,
    }))
  )[0];

describe("importSelection helpers", () => {
  it("builds a folder catalog, sorting folders and file paths, skipping blank paths", () => {
    const catalog = buildImportFolderCatalog([
      { full_path: "folder-b/three.pes" },
      { full_path: "folder-a/one.pes" },
      { full_path: "folder-a/two.pes" },
      { full_path: "solo.pes" },
      { full_path: "" },
      {},
    ]);

    // Sorted folder list; rootless file is grouped under "Unknown folder".
    expect(catalog.map((c) => c.folderPath)).toEqual([
      "folder-a",
      "folder-b",
      IMPORT_UNKNOWN_FOLDER,
    ]);

    const a = catalog.find((c) => c.folderPath === "folder-a");
    expect(a?.filePaths).toEqual(["folder-a/one.pes", "folder-a/two.pes"]);

    const unknown = catalog.find((c) => c.folderPath === IMPORT_UNKNOWN_FOLDER);
    expect(unknown?.label).toBe(IMPORT_UNKNOWN_FOLDER);
    expect(unknown?.filePaths).toEqual(["solo.pes"]);
  });

  it("defaults every scanned file to selected with no stored exceptions", () => {
    const record = files();
    const state = selCreate();
    expect(record.filePaths.every((p) => selIsSelected(state, FOLDER, p))).toBe(true);
    expect(selCount(state, FOLDER, record.filePaths.length)).toBe(record.filePaths.length);
  });

  it("deselects and re-selects a single file", () => {
    const record = files();
    const [a, b, c] = record.filePaths;
    let state = selCreate();

    state = selToggleFile(state, FOLDER, a, false);
    expect(selIsSelected(state, FOLDER, a)).toBe(false);
    expect(selIsSelected(state, FOLDER, b)).toBe(true);
    expect(selCount(state, FOLDER, record.filePaths.length)).toBe(2);

    state = selToggleFile(state, FOLDER, a, true);
    expect(selIsSelected(state, FOLDER, a)).toBe(true);
    expect(selCount(state, FOLDER, record.filePaths.length)).toBe(3);
  });

  it("select-all/deselect-all operate on a whole folder without enumerating files", () => {
    const record = files();
    let state = selCreate();
    const [a, b, c] = record.filePaths;

    state = selToggleFile(state, FOLDER, a, false);
    state = selToggleFile(state, FOLDER, b, false);
    expect(selCount(state, FOLDER, record.filePaths.length)).toBe(1);

    state = selSelectAllInFolder(state, FOLDER);
    expect(selCount(state, FOLDER, record.filePaths.length)).toBe(3);

    state = selDeselectAllInFolder(state, FOLDER);
    expect(selCount(state, FOLDER, record.filePaths.length)).toBe(0);
    expect(selIsSelected(state, FOLDER, a)).toBe(false);
    expect(selIsSelected(state, FOLDER, c)).toBe(false);

    // Selecting from a "none" folder adds to the selected-only list.
    state = selToggleFile(state, FOLDER, b, true);
    expect(selCount(state, FOLDER, record.filePaths.length)).toBe(1);
    expect(selIsSelected(state, FOLDER, b)).toBe(true);
    expect(selIsSelected(state, FOLDER, c)).toBe(false);
  });

  it("materialises the concrete selected path list across all folder states", () => {
    const catalog = buildImportFolderCatalog([
      { full_path: "C:/Big/design-0.pes" },
      { full_path: "C:/Big/design-1.pes" },
      { full_path: "C:/Big/design-2.pes" },
      { full_path: "C:/Small/only.pes" },
    ]);
    let state = selCreate();
    const bigFolder = "C:/Big";
    const smallFolder = "C:/Small";

    // Default: everything selected.
    expect(selMaterializeSelectedPaths(state, catalog)).toHaveLength(4);

    // Big folder: deselect two, leaving one. Small folder: select none.
    state = selDeselectAllFolders(state, [smallFolder]);
    state = selToggleFile(state, bigFolder, "C:/Big/design-0.pes", false);
    state = selToggleFile(state, bigFolder, "C:/Big/design-2.pes", false);

    const selected = selMaterializeSelectedPaths(state, catalog);
    expect(selected).toEqual(["C:/Big/design-1.pes"]);

    // Then select exactly one file inside an otherwise-none folder.
    state = selSelectAllFolders(state, [smallFolder]); // reset
    state = selDeselectAllFolders(state, [bigFolder, smallFolder]);
    state = selToggleFile(state, smallFolder, "C:/Small/only.pes", true);
    expect(selMaterializeSelectedPaths(state, catalog)).toEqual(["C:/Small/only.pes"]);
  });

  it("sanitises arbitrary serialised selections", () => {
    const state = selFromSerialized({
      deselected: { [FOLDER]: ["rose.pes", ""] },
      selectedOnly: { [FOLDER]: ["snow.pes"] },
    });
    expect(state.deselected[FOLDER]).toEqual(["rose.pes"]);
    expect(state.selectedOnly[FOLDER]).toEqual(["snow.pes"]);

    const empty = selFromSerialized(null);
    expect(empty).toEqual({ deselected: {}, selectedOnly: {} });
  });
});
