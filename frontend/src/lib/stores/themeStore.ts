// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Global User Theme Store.
 *
 * Supports three modes:
 * - "system": Follows OS prefers-color-scheme setting.
 * - "light": Explicit light theme.
 * - "dark": Explicit dark theme.
 *
 * Automatically syncs with `localStorage` and binds `data-theme` to
 * `document.documentElement` for instant reactive CSS application.
 */

import { writable } from "svelte/store";

export type ThemeMode = "system" | "light" | "dark";

const STORAGE_KEY = "embroidery_catalogue_theme";

function getInitialTheme(): ThemeMode {
  if (typeof window === "undefined" || !window.localStorage) {
    return "system";
  }
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "system") {
      return stored;
    }
  } catch {
    // Ignore localStorage access errors (e.g. sandboxed / private windows)
  }
  return "system";
}

export const themeStore = writable<ThemeMode>(getInitialTheme());

/**
 * Apply the data-theme attribute on <html> element based on current theme setting.
 */
export function applyThemeToDocument(theme: ThemeMode): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  if (theme === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", theme);
  }
}

/**
 * Update the user theme preference, store in localStorage, and apply to DOM.
 */
export function setTheme(theme: ThemeMode): void {
  themeStore.set(theme);
  if (typeof window !== "undefined" && window.localStorage) {
    try {
      window.localStorage.setItem(STORAGE_KEY, theme);
    } catch {
      // Ignore storage write errors
    }
  }
  applyThemeToDocument(theme);
}

/**
 * Initialize theme bindings on app mount.
 */
export function initTheme(): () => void {
  const initial = getInitialTheme();
  applyThemeToDocument(initial);

  const unsubscribe = themeStore.subscribe((value) => {
    applyThemeToDocument(value);
  });

  return unsubscribe;
}
