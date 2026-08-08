import "@testing-library/jest-dom/vitest";
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import AdminSourcesView from "./AdminSourcesView.svelte";

describe("AdminSourcesView mock", () => {
  it("renders with data-embedded=false when embedded is omitted", () => {
    render(AdminSourcesView);

    const view = screen.getByTestId("admin-sources-view");
    expect(view).toBeInTheDocument();
    expect(view).toHaveAttribute("data-embedded", "false");
    expect(view).toHaveTextContent("Admin Sources");
  });

  it("renders with data-embedded=false when embedded is false", () => {
    render(AdminSourcesView, { embedded: false });

    const view = screen.getByTestId("admin-sources-view");
    expect(view).toBeInTheDocument();
    expect(view).toHaveAttribute("data-embedded", "false");
    expect(view).toHaveTextContent("Admin Sources");
  });

  it("renders with data-embedded=true when embedded is true", () => {
    render(AdminSourcesView, { embedded: true });

    const view = screen.getByTestId("admin-sources-view");
    expect(view).toBeInTheDocument();
    expect(view).toHaveAttribute("data-embedded", "true");
    expect(view).toHaveTextContent("Admin Sources");
  });
});