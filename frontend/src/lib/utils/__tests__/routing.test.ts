import { describe, it, expect } from "vitest";
import {
  normalizeHash,
  resolveCurrentUiKind,
  parseProjectDetailId,
  parseReferenceDataTab,
  parseSystemTab,
  shouldShowBackButton,
} from "../routing.js";

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

  it("maps #/admin/batch-operations to batch-operations", () => {
    expect(resolveCurrentUiKind("#/admin/batch-operations")).toBe("batch-operations");
  });

  it("maps every Reference Data sub-tab to the stable reference-data kind", () => {
    expect(resolveCurrentUiKind("#/admin/data")).toBe("reference-data");
    expect(resolveCurrentUiKind("#/admin/data/designers")).toBe("reference-data");
    expect(resolveCurrentUiKind("#/admin/data/tags")).toBe("reference-data");
    expect(resolveCurrentUiKind("#/admin/data/sources")).toBe("reference-data");
    expect(resolveCurrentUiKind("#/admin/data/hoops")).toBe("reference-data");
  });

  it("maps every System sub-tab to the stable system kind", () => {
    expect(resolveCurrentUiKind("#/admin/system")).toBe("system");
    expect(resolveCurrentUiKind("#/admin/system/settings")).toBe("system");
    expect(resolveCurrentUiKind("#/admin/system/backup")).toBe("system");
    expect(resolveCurrentUiKind("#/admin/system/orphans")).toBe("system");
  });

  it("maps an unrecognised route to null", () => {
    expect(resolveCurrentUiKind("#/unknown/route")).toBeNull();
  });
});

describe("parseReferenceDataTab", () => {
  it("defaults the bare hub root to designers", () => {
    expect(parseReferenceDataTab("#/admin/data")).toBe("designers");
  });

  it("parses each sub-tab route", () => {
    expect(parseReferenceDataTab("#/admin/data/designers")).toBe("designers");
    expect(parseReferenceDataTab("#/admin/data/tags")).toBe("tags");
    expect(parseReferenceDataTab("#/admin/data/sources")).toBe("sources");
    expect(parseReferenceDataTab("#/admin/data/hoops")).toBe("hoops");
  });

  it("returns null outside the Reference Data hub", () => {
    expect(parseReferenceDataTab("#/designs")).toBeNull();
    expect(parseReferenceDataTab("#/admin/system/settings")).toBeNull();
    expect(parseReferenceDataTab("#/admin/data/unknown")).toBeNull();
  });
});

describe("parseSystemTab", () => {
  it("defaults the bare hub root to settings", () => {
    expect(parseSystemTab("#/admin/system")).toBe("settings");
  });

  it("parses each sub-tab route", () => {
    expect(parseSystemTab("#/admin/system/settings")).toBe("settings");
    expect(parseSystemTab("#/admin/system/backup")).toBe("backup");
    expect(parseSystemTab("#/admin/system/orphans")).toBe("orphans");
  });

  it("returns null outside the System hub", () => {
    expect(parseSystemTab("#/help")).toBeNull();
    expect(parseSystemTab("#/admin/data/tags")).toBeNull();
    expect(parseSystemTab("#/admin/system/unknown")).toBeNull();
  });
});

describe("parseProjectDetailId", () => {
  it("does not treat the new-project route as a numeric id", () => {
    expect(parseProjectDetailId("#/projects/new")).toBeNull();
  });
});

describe("shouldShowBackButton", () => {
  const prev = "#/designs";

  it("shows on content pages when arrived from elsewhere", () => {
    const routes = ["#/about", "#/about/licence", "#/about/document/licence", "#/help"];
    for (const route of routes) {
      expect(shouldShowBackButton(route, prev)).toBe(true);
    }
  });

  it("hides on work surfaces and Admin hubs even when arrived from elsewhere", () => {
    const routes = [
      "#/designs",
      "#/import",
      "#/import/step2",
      "#/projects",
      "#/projects/new",
      "#/projects/7",
      "#/projects/7/print",
      "#/designs/123",
      "#/designs/123/print",
      "#/admin/data/tags",
      "#/admin/batch-operations",
      "#/admin/system/settings",
      "#/admin/system/backup",
      "#/admin/system/orphans",
    ];
    for (const route of routes) {
      expect(shouldShowBackButton(route, prev)).toBe(false);
    }
  });

  it("hides on cold launch / deep link (no previous route)", () => {
    expect(shouldShowBackButton("#/help", "")).toBe(false);
  });

  it("hides when the previous route equals the current route", () => {
    expect(shouldShowBackButton("#/help", "#/help")).toBe(false);
  });

  it("hides on an unrecognised route", () => {
    expect(shouldShowBackButton("#/unknown/route", prev)).toBe(false);
  });
});
