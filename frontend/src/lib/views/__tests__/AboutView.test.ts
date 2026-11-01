// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, within, fireEvent, waitFor } from "@testing-library/svelte";
import AboutView from "../AboutView.svelte";
import type { LicenceStatus } from "../../types/licence";

const getLicenceStatusMock = vi.hoisted(() => vi.fn());
const deactivateLicenceMock = vi.hoisted(() => vi.fn());
const addToastMock = vi.hoisted(() => vi.fn());

vi.mock("../../api/licenceAdapter", () => ({
  getLicenceStatus: getLicenceStatusMock,
  deactivateLicence: deactivateLicenceMock,
}));

vi.mock("../../stores/toastStore.js", () => ({
  addToast: addToastMock,
}));

/** Type-guard helper so querySelector results can be used as HTMLElements. */
function element<T extends Element>(value: T | null | undefined, message?: string): T {
  if (!value) {
    throw new Error(message ?? "Expected element to exist.");
  }
  return value;
}

describe("AboutView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getLicenceStatusMock.mockResolvedValue({
      is_active: true,
      is_valid: true,
      email: "user@example.com",
      tier: "lifetime",
      expires_at: null,
      expires_at_formatted: null,
      error_message: null,
    } satisfies LicenceStatus);
  });

  describe("page chrome", () => {
    it("renders the app name, logo, and version", () => {
      render(AboutView);

      expect(screen.getByRole("heading", { name: "Embroidery Catalogue" })).toBeInTheDocument();
      expect(screen.getByText("Version v0.1.0")).toBeInTheDocument();
      expect(screen.getByText("🧵")).toBeInTheDocument();
    });

    it("renders the primary application description", () => {
      render(AboutView);

      const description = element(
        screen.getByText(/Embroidery Catalogue is a local, desktop catalogue/i),
        "Expected the app description paragraph."
      );
      const normalized = (description.textContent ?? "").replace(/\s+/g, " ").trim();
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

  describe("product licence & activation", () => {
    it("renders the licence details when valid", async () => {
      render(AboutView);

      await waitFor(() => {
        expect(getLicenceStatusMock).toHaveBeenCalled();
      });

      expect(screen.getByText("Product Licence & Activation")).toBeInTheDocument();
      expect(screen.getByText("LIFETIME")).toBeInTheDocument();
      expect(screen.getByTestId("about-licence-email")).toHaveTextContent("user@example.com");
      expect(screen.getByTestId("about-licence-expiry")).toHaveTextContent("Lifetime (No Expiry)");
      expect(screen.getByTestId("about-deactivate-licence-button")).toBeInTheDocument();
    });

    it("renders expiry date when present", async () => {
      getLicenceStatusMock.mockResolvedValue({
        is_active: true,
        is_valid: true,
        email: "subscriber@example.com",
        tier: "annual",
        expires_at: 1780000000,
        expires_at_formatted: "01/12/2026",
        error_message: null,
      });

      render(AboutView);

      await waitFor(() => {
        expect(screen.getByTestId("about-licence-expiry")).toHaveTextContent(
          "Valid until 01/12/2026"
        );
      });
      expect(screen.getByText("ANNUAL")).toBeInTheDocument();
    });

    it("shows no active licence detected when not valid", async () => {
      getLicenceStatusMock.mockResolvedValue({
        is_active: false,
        is_valid: false,
        email: null,
        tier: null,
        expires_at: null,
        expires_at_formatted: null,
        error_message: null,
      });

      render(AboutView);

      await waitFor(() => {
        expect(screen.getByText("No active licence detected.")).toBeInTheDocument();
      });
    });

    it("opens deactivation confirmation modal on button click", async () => {
      render(AboutView);

      await waitFor(() => {
        expect(screen.getByTestId("about-deactivate-licence-button")).toBeInTheDocument();
      });

      const deactivateBtn = screen.getByTestId("about-deactivate-licence-button");
      await fireEvent.click(deactivateBtn);

      expect(screen.getByTestId("confirm-deactivate-licence-modal")).toBeInTheDocument();
    });
  });

  describe("acknowledgements & code porting attributions", () => {
    it("credits pyembroidery with an MIT license and a repository link", () => {
      render(AboutView);

      expect(
        screen.getByRole("heading", {
          name: "Acknowledgements & Code Porting Attributions",
        })
      ).toBeInTheDocument();
      expect(
        screen.getByRole("heading", { name: "pyembroidery (MIT License)" })
      ).toBeInTheDocument();
      const link = screen.getByRole("link", {
        name: "https://github.com/EmbroidePy/pyembroidery",
      });
      expect(link).toHaveAttribute("href", "https://github.com/EmbroidePy/pyembroidery");
    });

    it("identifies ported binary parsing routines and formats in attribution", () => {
      render(AboutView);

      const heading = screen.getByRole("heading", {
        name: "pyembroidery (MIT License)",
      });
      const card = heading.closest("div");
      expect(card).not.toBeNull();
      const text = ((card as HTMLElement).textContent ?? "").replace(/\s+/g, " ");
      expect(text).toContain(".jef");
      expect(text).toContain(".pes");
      expect(text).toContain(".vp3");
      expect(text).toContain(".hus");
      expect(text).toContain(".exp");
      expect(text).toContain(".dst");
      expect(text).toContain("PngWriter");
    });

    it("thanks the pyembroidery contributors for the reference implementations", () => {
      render(AboutView);

      const heading = screen.getByRole("heading", {
        name: "pyembroidery (MIT License)",
      });
      const card = heading.closest("div");
      expect(card).not.toBeNull();
      const text = ((card as HTMLElement).textContent ?? "").replace(/\s+/g, " ");
      expect(text).toContain(
        "Our thanks go to Tatarize and the EmbroidePy pyembroidery contributors for publishing their work under a permissive licence"
      );
    });

    it("discloses the verbatim MIT notice required by pyembroidery behind a disclosure", () => {
      render(AboutView);

      const details = element(
        screen.getByTestId("pyembroidery-mit-notice"),
        "Expected the pyembroidery MIT notice disclosure."
      );
      expect(details.tagName).toBe("DETAILS");

      expect(
        within(details).getByText("View the MIT License notice required by pyembroidery")
      ).toBeInTheDocument();

      const notice = element(details.querySelector("pre"), "Expected the MIT notice <pre>.");
      const text = notice.textContent ?? "";
      expect(text).toContain("MIT License");
      expect(text).toContain("Copyright (c) 2018");
      expect(text).toContain(
        "Permission is hereby granted, free of charge, to any person obtaining a copy"
      );
      expect(text).toContain(
        "The above copyright notice and this permission notice shall be included in all"
      );
      expect(text).toContain('THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND');
    });
  });

  describe("View Open-Source Licences navigation", () => {
    it("links to the dedicated licence route", () => {
      render(AboutView);

      const button = screen.getByRole("link", { name: "View Open-Source Licences & Credits" });
      expect(button).toHaveAttribute("href", "#/about/licence");
    });
  });
});
