// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, it, expect, vi, beforeEach } from "vitest";
import { getLicenceStatus, activateLicence, deactivateLicence } from "../licenceAdapter";
import * as ipcClient from "../ipcClient";

describe("licenceAdapter", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("calls get_licence_status without args", async () => {
    const mockStatus = {
      is_active: true,
      is_valid: true,
      email: "tester@example.com",
      tier: "beta",
      expires_at: 1798058308,
      expires_at_formatted: "2026-12-23",
      error_message: null,
    };
    const spy = vi.spyOn(ipcClient, "invokeLoose").mockResolvedValueOnce(mockStatus);

    const result = await getLicenceStatus();
    expect(spy).toHaveBeenCalledWith("get_licence_status");
    expect(result).toEqual(mockStatus);
  });

  it("calls activate_licence with camelCase payload keys", async () => {
    const mockStatus = {
      is_active: true,
      is_valid: true,
      email: "tester@example.com",
      tier: "beta",
      expires_at: 1798058308,
      expires_at_formatted: "2026-12-23",
      error_message: null,
    };
    const spy = vi.spyOn(ipcClient, "invokeLoose").mockResolvedValueOnce(mockStatus);

    const result = await activateLicence("tester@example.com", "EMB1.abc.def");
    expect(spy).toHaveBeenCalledWith("activate_licence", {
      email: "tester@example.com",
      licenceKey: "EMB1.abc.def",
    });
    expect(result).toEqual(mockStatus);
  });

  it("calls deactivate_licence without args", async () => {
    const mockStatus = {
      is_active: false,
      is_valid: false,
      email: null,
      tier: null,
      expires_at: null,
      expires_at_formatted: null,
      error_message: null,
    };
    const spy = vi.spyOn(ipcClient, "invokeLoose").mockResolvedValueOnce(mockStatus);

    const result = await deactivateLicence();
    expect(spy).toHaveBeenCalledWith("deactivate_licence");
    expect(result).toEqual(mockStatus);
  });
});
