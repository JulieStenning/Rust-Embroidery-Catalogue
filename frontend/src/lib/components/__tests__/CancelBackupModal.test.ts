import "@testing-library/jest-dom/vitest";
import { describe, it, expect, afterEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/svelte";
import CancelBackupModal from "../CancelBackupModal.svelte";

/**
 * The explanatory notes render both as <p> text nodes and inside the modal
 * body container, so assert on the normalized textContent of the body (see
 * .clinerules: scope the query, don't fight multi-region text).
 */
function bodyText(): string {
  const dialog = screen.getByRole("dialog");
  const modalBody = dialog.querySelector(".cancel-backup-modal-body");
  return (modalBody?.textContent ?? "").replace(/\s+/g, " ").trim();
}

const DB_RUNNING =
  "The database copy is currently running. If you proceed, the database backup will be aborted and the incomplete database file will be deleted.";
const DB_COMPLETED = "The database copy has completed.";
const DESIGNS_NOTE = "Any design files already copied up to the point of cancellation";

describe("CancelBackupModal", () => {
  afterEach(() => {
    cleanup();
  });

  it("database-only backup shows the in-progress note and no design-file note", () => {
    render(CancelBackupModal, { open: true, activeKind: "database" });
    const text = bodyText();
    expect(text).toContain(DB_RUNNING);
    expect(text).not.toContain(DB_COMPLETED);
    expect(text).not.toContain(DESIGNS_NOTE);
  });

  it("designs-only backup shows the design-file note and no database note", () => {
    render(CancelBackupModal, { open: true, activeKind: "designs" });
    const text = bodyText();
    expect(text).toContain(DESIGNS_NOTE);
    expect(text).not.toContain("database copy is currently running");
    expect(text).not.toContain(DB_COMPLETED);
  });

  it("combined backup while the database copy is still in progress shows only the in-progress note", () => {
    render(CancelBackupModal, {
      open: true,
      activeKind: "both",
      databaseCopyDone: false,
    });
    const text = bodyText();
    expect(text).toContain(DB_RUNNING);
    expect(text).not.toContain(DB_COMPLETED);
    // Design files are only copied after the database phase finishes.
    expect(text).not.toContain(DESIGNS_NOTE);
  });

  it("combined backup after the database copy completes shows the completed note and keeps the design-file note", () => {
    render(CancelBackupModal, {
      open: true,
      activeKind: "both",
      databaseCopyDone: true,
    });
    const text = bodyText();
    expect(text).toContain(DB_COMPLETED);
    expect(text).not.toContain(DB_RUNNING);
    expect(text).toContain(DESIGNS_NOTE);
  });
});
