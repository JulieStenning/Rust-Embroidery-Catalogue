import { writable } from "svelte/store";

// ---------------------------------------------------------------------------
// Import wizard session state
// ---------------------------------------------------------------------------
// The wizard lives inside ImportView and its `$state` is destroyed whenever the
// route leaves "import" (e.g. clicking Admin Settings, AI Tagging Guide, About
// or Licence in the step 3 "Before You Import" panel).  This store holds a
// serializable snapshot of the wizard so the user can return to step 2/3 with
// all their selections intact after visiting the top-level nav.
//
// ImportView keeps its local `$state` as the live source of truth while the
// component is mounted, and mirrors it into this store every time it changes.
// On remount it restores from the store.  resetImportWizard()/cancel call
// clear() to discard the snapshot.
// ---------------------------------------------------------------------------

export interface ImportSessionState {
  /** Source folder rows (primary path + additional rows). */
  rootPath: string;
  /** Additional source folder rows. */
  rootPaths: string[];

  /** Preview result from previewImportFromRoots, or null. */
  preview: Record<string, unknown> | null;
  previewSource: string;
  previewMessage: string;

  /** Precheck result from precheckImportWire, or null. */
  precheck: Record<string, unknown> | null;
  precheckSource: string;
  precheckMessage: string;

  /** Full paths of the files selected for import. */
  selectedFiles: string[];

  /** Backend token minted by precheck_bulk_import_wire. */
  contextToken: string;

  /** Global designer/source overrides applied to every folder. */
  globalDesignerId: string;
  globalSourceId: string;

  /** Per-folder designer/source overrides keyed by normalized folder path. */
  perFolderAssignmentByPath: Record<string, { designerId: string; sourceId: string }>;

  /** Last precheck action message surfaced on the step 3 panel. */
  actionMessage: string;
  actionSource: string;
  actionNeedsSkipHoopsConfirm: boolean;
}

function createInitialSessionState(): ImportSessionState {
  return {
    rootPath: "",
    rootPaths: [],
    preview: null,
    previewSource: "mock",
    previewMessage: "",
    precheck: null,
    precheckSource: "mock",
    precheckMessage: "Run precheck after selecting files.",
    selectedFiles: [],
    contextToken: "",
    globalDesignerId: "",
    globalSourceId: "",
    perFolderAssignmentByPath: {},
    actionMessage: "",
    actionSource: "mock",
    actionNeedsSkipHoopsConfirm: false,
  };
}

function createImportSessionStore() {
  const { subscribe, set, update } = writable<ImportSessionState>(createInitialSessionState());

  return {
    subscribe,

    /**
     * Replace the entire wizard snapshot.  Used by ImportView on every state
     * change (via an effect) so navigating away and back preserves the wizard.
     * @param {ImportSessionState} state
     */
    setSession(state: ImportSessionState) {
      set(state);
    },

    /**
     * Patch a subset of the wizard snapshot.  Convenient for targeted writes
     * without re-supplying the full object.
     * @param {Partial<ImportSessionState>} patch
     */
    patchSession(patch: Partial<ImportSessionState>) {
      update((state) => ({ ...state, ...patch }));
    },

    /** Discard the wizard snapshot (used by resetImportWizard()/cancel). */
    clear() {
      set(createInitialSessionState());
    },
  };
}

/** Singleton session store for the bulk import wizard. */
export const importSessionStore = createImportSessionStore();