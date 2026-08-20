import "@testing-library/jest-dom/vitest";
import { describe, it, expect } from "vitest";
import { render, screen, within } from "@testing-library/svelte";
import AboutView from "../AboutView.svelte";

/** Type-guard helper so querySelector results can be used as HTMLElements. */
function element<T extends Element>(
  value: T | null | undefined,
  message?: string
): T {
  if (!value) {
    throw new Error(message ?? "Expected element to exist.");
  }
  return value;
}

describe("AboutView", () => {
  describe("page chrome", () => {
    it("renders the app name, logo, and version", () => {
      render(AboutView);

      expect(
        screen.getByRole("heading", { name: "Embroidery Catalogue" })
      ).toBeInTheDocument();
      expect(screen.getByText("Version v0.1.0")).toBeInTheDocument();
      expect(screen.getByText("🧵")).toBeInTheDocument();
    });

    it("renders the primary application description", () => {
      render(AboutView);

      // Scope to the exact description paragraph. The pyembroidery attribution
      // card also contains file-extension names, so match a unique phrase that
      // only appears in the app description.
      const description = element(
        document.querySelector("p.text-gray-700"),
        "Expected the app description paragraph."
      );
      const normalized = (description.textContent ?? "")
        .replace(/\s+/g, " ")
        .trim();
      expect(normalized).toContain(
        "Embroidery Catalogue is a local, desktop catalogue for browsing, tagging, and managing a collection of digital embroidery designs."
      );
      expect(normalized).toContain(".jef");
      expect(normalized).toContain(".pes");
      expect(normalized).toContain(".hus");
      expect(normalized).toContain(".vp3");
      expect(normalized).toContain("local database");
    });

    it("renders the primary copyright notice", () => {
      render(AboutView);

      expect(screen.getByText("Primary Copyright")).toBeInTheDocument();
      expect(screen.getByText("Copyright (C) 2026 Julie Stenning")).toBeInTheDocument();
    });
  });

  describe("acknowledgements & code porting attributions", () => {
    it("credits pyembroidery with an MIT license and a repository link", () => {
      render(AboutView);

      expect(
        screen.getByRole("heading", { name: "pyembroidery (MIT License)" })
      ).toBeInTheDocument();

      const pyembroideryHeading = screen.getByRole("heading", {
        name: "pyembroidery (MIT License)",
      });
      const card = pyembroideryHeading.closest("div");
      expect(card).not.toBeNull();

      const link = within(card as HTMLElement).getByRole("link", {
        name: "https://github.com/EmbroidePy/pyembroidery",
      });
      expect(link).toHaveAttribute("href", "https://github.com/EmbroidePy/pyembroidery");
      expect(link).toHaveAttribute("target", "_blank");
      expect(link).toHaveAttribute("rel", "noopener noreferrer");
    });

    it("describes pyembroidery as the basis for Rust binary parsing routines", () => {
      render(AboutView);

      const heading = screen.getByRole("heading", {
        name: "pyembroidery (MIT License)",
      });
      const card = heading.closest("div");
      expect(card).not.toBeNull();
      expect((card as HTMLElement).textContent).toContain("ported/derived into native Rust modules");
      expect((card as HTMLElement).textContent).toContain(".pes");
      expect((card as HTMLElement).textContent).toContain(".jef");
      expect((card as HTMLElement).textContent).toContain(".vp3");
      expect((card as HTMLElement).textContent).toContain(".hus");
      expect((card as HTMLElement).textContent).toContain(".exp");
      expect((card as HTMLElement).textContent).toContain(".dst");
    });
  });

  describe("View Full Licences navigation", () => {
    it("links to the dedicated licence route", () => {
      render(AboutView);

      const button = screen.getByRole("link", { name: "View Full Licences" });
      expect(button).toHaveAttribute("href", "#/about/licence");
    });
  });
});