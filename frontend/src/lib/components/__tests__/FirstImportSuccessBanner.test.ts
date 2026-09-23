// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import { tick } from "svelte";
import FirstImportSuccessBanner from "../FirstImportSuccessBanner.svelte";
import { firstImportBannerVisible, triggerFirstImportBanner } from "../../stores/firstImportStore";

describe("FirstImportSuccessBanner.svelte", () => {
  beforeEach(() => {
    window.localStorage.clear();
    firstImportBannerVisible.set(false);
  });

  it("does not render when the banner is not visible", () => {
    render(FirstImportSuccessBanner);
    expect(screen.queryByTestId("first-import-success-banner")).not.toBeInTheDocument();
  });

  it("renders when visible and displays tagged copy, links, and dismiss button", async () => {
    firstImportBannerVisible.set(true);
    render(FirstImportSuccessBanner);
    await tick();

    expect(screen.getByTestId("first-import-success-banner")).toBeInTheDocument();
    expect(screen.getByText("First import complete!")).toBeInTheDocument();
    expect(
      screen.getByText(/Your designs are tagged with fast, offline File & Folder rules/)
    ).toBeInTheDocument();

    const settingsLink = screen.getByRole("link", { name: "Settings" });
    expect(settingsLink).toHaveAttribute("href", "#/admin/system/settings");

    const batchLink = screen.getByRole("link", { name: "Batch Operations" });
    expect(batchLink).toHaveAttribute("href", "#/admin/batch-operations");

    const dismissBtn = screen.getByRole("button", { name: "Dismiss first import notice" });
    expect(dismissBtn).toBeInTheDocument();
  });

  it("dismisses the banner when dismiss button is clicked and stores in localStorage", async () => {
    triggerFirstImportBanner();
    render(FirstImportSuccessBanner);
    await tick();

    expect(screen.getByTestId("first-import-success-banner")).toBeInTheDocument();

    const dismissBtn = screen.getByRole("button", { name: "Dismiss first import notice" });
    await fireEvent.click(dismissBtn);
    await tick();

    expect(screen.queryByTestId("first-import-success-banner")).not.toBeInTheDocument();
    expect(window.localStorage.getItem("embroidery_catalogue_first_import_banner_dismissed")).toBe(
      "true"
    );

    // Triggering again after dismissal does not reopen it
    triggerFirstImportBanner();
    await tick();
    expect(screen.queryByTestId("first-import-success-banner")).not.toBeInTheDocument();
  });
});
