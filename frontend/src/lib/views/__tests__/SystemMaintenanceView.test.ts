// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import SystemMaintenanceView from "../SystemMaintenanceView.svelte";

vi.mock("../SettingsView.svelte", async () => {
  const { default: C } = await import("./__mocks__/hubs/SystemSettings.svelte");
  return { default: C };
});
vi.mock("../BackupView.svelte", async () => {
  const { default: C } = await import("./__mocks__/hubs/SystemBackup.svelte");
  return { default: C };
});
vi.mock("../OrphansView.svelte", async () => {
  const { default: C } = await import("./__mocks__/hubs/SystemOrphans.svelte");
  return { default: C };
});

/** Set location.hash and dispatch a hashchange (mirrors real navigation). */
const setHash = (hash: string) => {
  window.location.hash = hash;
  window.dispatchEvent(new HashChangeEvent("hashchange"));
};

describe("SystemMaintenanceView", () => {
  beforeEach(() => {
    window.location.hash = "#/admin/system/settings";
  });

  it("renders the three sub-tab links with the canonical URLs", async () => {
    render(SystemMaintenanceView);

    expect(screen.getByTestId("system-maintenance-tablist")).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Settings" })).toHaveAttribute(
      "href",
      "#/admin/system/settings"
    );
    expect(screen.getByRole("tab", { name: "Backup & Restore" })).toHaveAttribute(
      "href",
      "#/admin/system/backup"
    );
    expect(screen.getByRole("tab", { name: "Orphaned Files" })).toHaveAttribute(
      "href",
      "#/admin/system/orphans"
    );
  });

  it("defaults to the settings sub-view on the bare hub root", async () => {
    window.location.hash = "#/admin/system";
    render(SystemMaintenanceView);

    await waitFor(() => {
      expect(screen.getByTestId("sys-settings")).toBeInTheDocument();
    });
    expect(screen.getByRole("tab", { name: "Settings" })).toHaveAttribute("aria-selected", "true");
  });

  it("mounts the correct child for each deep-link URL", async () => {
    setHash("#/admin/system/backup");
    render(SystemMaintenanceView);
    await waitFor(() => {
      expect(screen.getByTestId("sys-backup")).toBeInTheDocument();
    });
    expect(screen.getByRole("tab", { name: "Backup & Restore" })).toHaveAttribute(
      "aria-selected",
      "true"
    );
  });

  it("switches children when the hash URL changes between sub-tabs", async () => {
    render(SystemMaintenanceView);
    await waitFor(() => {
      expect(screen.getByTestId("sys-settings")).toBeInTheDocument();
    });

    setHash("#/admin/system/orphans");
    await waitFor(() => {
      expect(screen.getByTestId("sys-orphans")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("sys-settings")).not.toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Orphaned Files" })).toHaveAttribute(
      "aria-selected",
      "true"
    );
    expect(screen.getByRole("tab", { name: "Settings" })).toHaveAttribute("aria-selected", "false");
  });

  it("mounts the orphans child for the orphans URL", async () => {
    setHash("#/admin/system/orphans");
    render(SystemMaintenanceView);
    await waitFor(() => {
      expect(screen.getByTestId("sys-orphans")).toBeInTheDocument();
    });
    expect(screen.getByRole("tab", { name: "Orphaned Files" })).toHaveAttribute(
      "aria-selected",
      "true"
    );
  });

  it("disables tabs and guards clicks when busyState is active", async () => {
    const { beginBusy, resetBusy } = await import("../../stores/busyStore");
    beginBusy("Running backup");

    const { unmount } = render(SystemMaintenanceView);

    const settingsTab = screen.getByRole("tab", { name: "Settings" });
    const backupTab = screen.getByRole("tab", { name: "Backup & Restore" });

    expect(settingsTab).toHaveAttribute("aria-disabled", "true");
    expect(settingsTab.className).toContain("opacity-50");

    // Clicking while busy should prevent navigation
    const clickEvent = new MouseEvent("click", { cancelable: true, bubbles: true });
    backupTab.dispatchEvent(clickEvent);
    expect(clickEvent.defaultPrevented).toBe(true);

    resetBusy();
    unmount();
  });
});
