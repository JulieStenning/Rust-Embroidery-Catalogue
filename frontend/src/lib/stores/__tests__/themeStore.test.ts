// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import {
  themeStore,
  setTheme,
  applyThemeToDocument,
  initTheme,
  type ThemeMode,
} from "../themeStore";

function currentTheme(): ThemeMode {
  let snapshot: ThemeMode = "system";
  const unsubscribe = themeStore.subscribe((value) => {
    snapshot = value;
  });
  unsubscribe();
  return snapshot;
}

describe("themeStore", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
    themeStore.set("system");
  });

  afterEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("defaults to system theme", () => {
    expect(currentTheme()).toBe("system");
  });

  it("applies theme to document attribute", () => {
    applyThemeToDocument("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");

    applyThemeToDocument("light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");

    applyThemeToDocument("system");
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
  });

  it("updates store and localStorage on setTheme", () => {
    setTheme("dark");
    expect(currentTheme()).toBe("dark");
    expect(localStorage.getItem("embroidery_catalogue_theme")).toBe("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");

    setTheme("light");
    expect(currentTheme()).toBe("light");
    expect(localStorage.getItem("embroidery_catalogue_theme")).toBe("light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");

    setTheme("system");
    expect(currentTheme()).toBe("system");
    expect(localStorage.getItem("embroidery_catalogue_theme")).toBe("system");
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
  });

  it("syncs document on initTheme lifecycle", () => {
    setTheme("dark");
    const unsubscribe = initTheme();
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");

    themeStore.set("light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");

    unsubscribe();
  });
});
