import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import ReferenceDataView from "../ReferenceDataView.svelte";

vi.mock("../AdminDesignersView.svelte", async () => {
  const { default: C } = await import("./__mocks__/hubs/ReferenceDataDesigners.svelte");
  return { default: C };
});
vi.mock("../TagsView.svelte", async () => {
  const { default: C } = await import("./__mocks__/hubs/ReferenceDataTags.svelte");
  return { default: C };
});
vi.mock("../AdminSourcesView.svelte", async () => {
  const { default: C } = await import("./__mocks__/hubs/ReferenceDataSources.svelte");
  return { default: C };
});
vi.mock("../AdminHoopsView.svelte", async () => {
  const { default: C } = await import("./__mocks__/hubs/ReferenceDataHoops.svelte");
  return { default: C };
});

/** Set location.hash and dispatch a hashchange (mirrors real navigation). */
const setHash = (hash: string) => {
  window.location.hash = hash;
  window.dispatchEvent(new HashChangeEvent("hashchange"));
};

describe("ReferenceDataView", () => {
  beforeEach(() => {
    window.location.hash = "#/admin/data/designers";
  });

  it("renders the four sub-tab links with the canonical URLs", async () => {
    render(ReferenceDataView);

    expect(screen.getByTestId("reference-data-tablist")).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Designers" })).toHaveAttribute(
      "href",
      "#/admin/data/designers"
    );
    expect(screen.getByRole("tab", { name: "Tags" })).toHaveAttribute("href", "#/admin/data/tags");
    expect(screen.getByRole("tab", { name: "Sources" })).toHaveAttribute(
      "href",
      "#/admin/data/sources"
    );
    expect(screen.getByRole("tab", { name: "Hoops" })).toHaveAttribute("href", "#/admin/data/hoops");
  });

  it("defaults to the designers sub-view on the bare hub root", async () => {
    window.location.hash = "#/admin/data";
    render(ReferenceDataView);

    await waitFor(() => {
      expect(screen.getByTestId("rd-designers")).toBeInTheDocument();
    });
    expect(screen.getByRole("tab", { name: "Designers" })).toHaveAttribute("aria-selected", "true");
  });

  it("mounts the correct child for each deep-link URL", async () => {
    setHash("#/admin/data/tags");
    render(ReferenceDataView);
    await waitFor(() => {
      expect(screen.getByTestId("rd-tags")).toBeInTheDocument();
    });
    expect(screen.getByRole("tab", { name: "Tags" })).toHaveAttribute("aria-selected", "true");
  });

  it("switches children when the hash URL changes between sub-tabs", async () => {
    render(ReferenceDataView);
    await waitFor(() => {
      expect(screen.getByTestId("rd-designers")).toBeInTheDocument();
    });

    setHash("#/admin/data/sources");
    await waitFor(() => {
      expect(screen.getByTestId("rd-sources")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("rd-designers")).not.toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Sources" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tab", { name: "Designers" })).toHaveAttribute(
      "aria-selected",
      "false"
    );
  });

  it("mounts the hoops child for the hoops URL", async () => {
    setHash("#/admin/data/hoops");
    render(ReferenceDataView);
    await waitFor(() => {
      expect(screen.getByTestId("rd-hoops")).toBeInTheDocument();
    });
    expect(screen.getByRole("tab", { name: "Hoops" })).toHaveAttribute("aria-selected", "true");
  });
});
