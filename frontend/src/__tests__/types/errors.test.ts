// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, it, expect } from "vitest";
import { isIpcError, type IpcError } from "../../lib/types/errors";

describe("IpcError types and guards", () => {
  it("identifies valid IpcError objects", () => {
    const err: IpcError = {
      code: "not_found",
      message: "Design 42 not found",
    };
    expect(isIpcError(err)).toBe(true);
  });

  it("rejects non-object or null values", () => {
    expect(isIpcError(null)).toBe(false);
    expect(isIpcError(undefined)).toBe(false);
    expect(isIpcError("some string")).toBe(false);
    expect(isIpcError(123)).toBe(false);
  });

  it("rejects objects missing required fields", () => {
    expect(isIpcError({ code: "database" })).toBe(false);
    expect(isIpcError({ message: "error message" })).toBe(false);
    expect(isIpcError({ code: 123, message: "msg" })).toBe(false);
  });
});
