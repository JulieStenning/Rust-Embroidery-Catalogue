// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import BatchOperationsView from "../BatchOperationsView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter — prevents real Tauri `invoke` calls.
// ---------------------------------------------------------------------------
const adapterMocks = vi.hoisted(() => ({
  getBatchOperationsViewModel: vi.fn(),
  runUnifiedBackfill: vi.fn(),
  stopUnifiedBackfill: vi.fn(),
  getBackfillLogEntries: vi.fn(),
  runMaintenanceBackfill: vi.fn(),
  countMissingPreviews: vi.fn(),
  countTaggingCandidates: vi.fn(),
}));

vi.mock("../../api/commandAdapter", () => adapterMocks);

// Mock the toast store — the view calls addToast() on every run.
const toastMock = vi.hoisted(() => ({ addToast: vi.fn() }));
vi.mock("../../stores/toastStore.js", () => toastMock);

// The view installs a backfill-progress listener on mount. Provide a benign
// mock so it resolves cleanly and never touches real Tauri internals in jsdom.
vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

const viewModel = () => ({
  source: "rust",
  model: {
    has_google_api_key: false,
    ai_vision_auto: false,
    ai_batch_size: "",
    ai_delay: "",
    import_commit_batch_size: "",
    default_batch_size: 100,
    default_commit_every: 100,
    default_workers: 4,
  },
});

/** Helper that constructs a backfill result. */
const backfillResult = (overrides = {}) => ({
  source: "rust",
  processed: 0,
  errors: 0,
  stopped: false,
  actions: [],
  ...overrides,
});

async function gotoMaintenanceTab() {
  const user = userEvent.setup();
  await user.click(screen.getByRole("tab", { name: /Maintenance & File Processing/i }));
}

async function startMaintenanceRun() {
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Review & Start Maintenance" }));
  await user.click(screen.getByRole("button", { name: "Start Maintenance" }));
}

describe("BatchOperationsView stitching tags on maintenance tab", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    adapterMocks.getBatchOperationsViewModel.mockResolvedValue(viewModel());
    adapterMocks.getBackfillLogEntries.mockResolvedValue({
      source: "rust",
      entries: [],
    });
    adapterMocks.countTaggingCandidates.mockResolvedValue({
      source: "rust",
      action: "tag_untagged",
      counts: { total_count: 12, unverified_count: 10, verified_count: 2 },
    });
    adapterMocks.countMissingPreviews.mockResolvedValue(0);
    adapterMocks.runMaintenanceBackfill.mockResolvedValue(backfillResult());
  });

  it("switches to Maintenance tab from the callout link on Tagging tab", async () => {
    render(BatchOperationsView);
    const user = userEvent.setup();

    const callout = screen.getByTestId("tagging-maintenance-callout");
    expect(callout).toBeInTheDocument();

    const link = screen.getByRole("button", { name: /Maintenance & File Processing/i });
    await user.click(link);

    expect(screen.getByRole("tab", { name: /Maintenance & File Processing/i })).toHaveAttribute(
      "aria-selected",
      "true"
    );
  });

  it("calls runMaintenanceBackfill with detect_stitching_tags and unverified clear mode by default", async () => {
    render(BatchOperationsView);
    await gotoMaintenanceTab();

    const user = userEvent.setup();
    await user.click(
      screen.getByRole("checkbox", { name: /Detect \/ recalculate stitching tags/ })
    );

    await startMaintenanceRun();

    await waitFor(() => {
      expect(adapterMocks.runMaintenanceBackfill).toHaveBeenCalledWith(
        expect.objectContaining({
          detect_stitching_tags: true,
          stitching_clear_mode: "unverified",
          scope: "all",
        })
      );
    });
  });

  it("passes stitching_clear_mode all when overwrite human-verified is checked", async () => {
    render(BatchOperationsView);
    await gotoMaintenanceTab();

    const user = userEvent.setup();
    await user.click(
      screen.getByRole("checkbox", { name: /Detect \/ recalculate stitching tags/ })
    );
    await user.click(
      screen.getByRole("checkbox", {
        name: /Overwrite human-verified stitching tags/,
      })
    );

    await startMaintenanceRun();

    await waitFor(() => {
      expect(adapterMocks.runMaintenanceBackfill).toHaveBeenCalledWith(
        expect.objectContaining({
          detect_stitching_tags: true,
          stitching_clear_mode: "all",
        })
      );
    });
  });
});
