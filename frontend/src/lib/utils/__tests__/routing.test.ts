import { describe, it, expect } from "vitest";
import { normalizeHash, resolveCurrentUiKind, parseProjectDetailId } from "../routing.js";

describe("normalizeHash", () => {
  it("preserves the new-project route", () => {
    expect(normalizeHash("#/projects/new")).toBe("#/projects/new");
  });

  it("strips query strings from known routes", () => {
    expect(normalizeHash("#/help?section=projects")).toBe("#/help");
  });

  it("falls back to the browse route for unknown routes", () => {
    expect(normalizeHash("#/unknown/route")).toBe("#/designs");
  });

  it("falls back to the browse route for empty input", () => {
    expect(normalizeHash("")).toBe("#/designs");
  });
});

describe("resolveCurrentUiKind", () => {
  it("maps #/projects/new to project-new", () => {
    expect(resolveCurrentUiKind("#/projects/new")).toBe("project-new");
  });

  it("maps an unrecognised route to null", () => {
    expect(resolveCurrentUiKind("#/unknown/route")).toBeNull();
  });
});

describe("parseProjectDetailId", () => {
  it("does not treat the new-project route as a numeric id", () => {
    expect(parseProjectDetailId("#/projects/new")).toBeNull();
  });
});
