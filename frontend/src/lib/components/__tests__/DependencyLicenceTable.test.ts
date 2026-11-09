// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect } from "vitest";
import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { tick } from "svelte";
import DependencyLicenceTable from "../DependencyLicenceTable.svelte";
import type { DependencyItem } from "../../types/licence";

const SAMPLE_ITEMS: DependencyItem[] = [
  {
    name: "rusqlite",
    version: "0.32.0",
    licence: "MIT",
    repository: "https://github.com/rusqlite/rusqlite",
    text: "MIT License text for rusqlite",
  },
  {
    name: "tauri",
    version: "2.0.0",
    licence: "Apache-2.0",
    repository: "https://github.com/tauri-apps/tauri",
    text: "Apache-2.0 text for tauri",
  },
  {
    name: "serde",
    version: "1.0.200",
    licence: "MIT",
    repository: "https://github.com/serde-rs/serde",
    text: "MIT License text for serde",
  },
];

describe("DependencyLicenceTable", () => {
  it("renders table with all provided items and summary counter", () => {
    render(DependencyLicenceTable, {
      props: {
        items: SAMPLE_ITEMS,
        dependencyType: "Rust",
      },
    });

    expect(screen.getByTestId("dependency-count-summary")).toHaveTextContent("Showing 3 crates");
    expect(screen.getByText("rusqlite")).toBeInTheDocument();
    expect(screen.getByText("(v0.32.0)")).toBeInTheDocument();
    expect(screen.getByText("tauri")).toBeInTheDocument();
    expect(screen.getByText("serde")).toBeInTheDocument();
  });

  it("filters items in real time via the search box", async () => {
    render(DependencyLicenceTable, {
      props: {
        items: SAMPLE_ITEMS,
        dependencyType: "Rust",
      },
    });

    const searchInput = screen.getByTestId("dependency-search-input");
    await fireEvent.input(searchInput, { target: { value: "rusq" } });
    await tick();

    expect(screen.getByTestId("dependency-count-summary")).toHaveTextContent(
      "Showing 1 of 3 crates"
    );
    expect(screen.getByText("rusqlite")).toBeInTheDocument();
    expect(screen.queryByText("tauri")).not.toBeInTheDocument();
    expect(screen.queryByText("serde")).not.toBeInTheDocument();
  });

  it("filters items by licence category dropdown", async () => {
    render(DependencyLicenceTable, {
      props: {
        items: SAMPLE_ITEMS,
        dependencyType: "Rust",
      },
    });

    const select = screen.getByTestId("dependency-licence-filter");
    await fireEvent.change(select, { target: { value: "Apache-2.0" } });
    await tick();

    expect(screen.getByTestId("dependency-count-summary")).toHaveTextContent(
      "Showing 1 of 3 crates"
    );
    expect(screen.getByText("tauri")).toBeInTheDocument();
    expect(screen.queryByText("rusqlite")).not.toBeInTheDocument();
  });

  it("opens the terms modal when clicking View Terms and closes it", async () => {
    render(DependencyLicenceTable, {
      props: {
        items: SAMPLE_ITEMS,
        dependencyType: "Rust",
      },
    });

    const viewTermsBtn = screen.getByRole("button", { name: "View licence terms for rusqlite" });
    await fireEvent.click(viewTermsBtn);
    await tick();

    expect(screen.getByTestId("licence-terms-modal")).toBeInTheDocument();
    expect(screen.getByTestId("licence-terms-text")).toHaveTextContent(
      "MIT License text for rusqlite"
    );

    const closeBtn = screen.getByTestId("close-licence-terms-modal");
    await fireEvent.click(closeBtn);
    await tick();

    expect(screen.queryByTestId("licence-terms-modal")).not.toBeInTheDocument();
  });

  it("displays no-match notice when query matches nothing", async () => {
    render(DependencyLicenceTable, {
      props: {
        items: SAMPLE_ITEMS,
        dependencyType: "Frontend",
      },
    });

    const searchInput = screen.getByTestId("dependency-search-input");
    await fireEvent.input(searchInput, { target: { value: "nonexistent-pkg" } });
    await tick();

    expect(screen.getByTestId("dependency-no-results")).toHaveTextContent(
      'No dependencies found matching "nonexistent-pkg"'
    );
  });

  it("displays emptyMessage when items array is empty", () => {
    render(DependencyLicenceTable, {
      props: {
        items: [],
        dependencyType: "Frontend",
        emptyMessage: "No frontend packages recorded.",
      },
    });

    expect(screen.getByTestId("dependency-empty-state")).toHaveTextContent(
      "No frontend packages recorded."
    );
  });
});
