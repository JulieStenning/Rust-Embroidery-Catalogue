import { writable } from "svelte/store";

const STORAGE_KEY = "embroidery_catalogue_first_import_banner_dismissed";

function getInitialDismissedState(): boolean {
  if (typeof window === "undefined" || !window.localStorage) {
    return false;
  }
  try {
    return window.localStorage.getItem(STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

export function isFirstImportBannerDismissed(): boolean {
  return getInitialDismissedState();
}

export const firstImportBannerVisible = writable(false);

export function triggerFirstImportBanner(): void {
  if (!isFirstImportBannerDismissed()) {
    firstImportBannerVisible.set(true);
  }
}

export function dismissFirstImportBanner(): void {
  firstImportBannerVisible.set(false);
  if (typeof window !== "undefined" && window.localStorage) {
    try {
      window.localStorage.setItem(STORAGE_KEY, "true");
    } catch (e) {
      console.warn("Could not save first import banner dismissal:", e);
    }
  }
}
