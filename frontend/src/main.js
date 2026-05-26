import { mount } from "svelte";
import App from "./App.svelte";

function ensureStartupErrorBanner(message) {
  const bannerId = "startup-error-banner";
  let banner = document.getElementById(bannerId);

  if (!banner) {
    banner = document.createElement("div");
    banner.id = bannerId;
    banner.setAttribute("role", "alert");
    banner.setAttribute("aria-live", "assertive");
    banner.style.position = "fixed";
    banner.style.top = "0";
    banner.style.left = "0";
    banner.style.right = "0";
    banner.style.zIndex = "9999";
    banner.style.padding = "10px 14px";
    banner.style.background = "#7f1d1d";
    banner.style.color = "#fee2e2";
    banner.style.borderBottom = "1px solid #ef4444";
    banner.style.fontFamily = "Segoe UI, Arial, sans-serif";
    banner.style.fontSize = "13px";
    banner.style.lineHeight = "1.4";
    banner.style.whiteSpace = "pre-wrap";
    banner.style.wordBreak = "break-word";
    document.body.appendChild(banner);
  }

  banner.textContent = `[Startup Error] ${message}`;
}

function formatError(error) {
  if (!error) {
    return "Unknown startup error";
  }
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

window.addEventListener("error", (event) => {
  ensureStartupErrorBanner(formatError(event.error || event.message));
});

window.addEventListener("unhandledrejection", (event) => {
  ensureStartupErrorBanner(formatError(event.reason));
});

const target = document.getElementById("app");

if (!target) {
  const message = "Could not find #app mount node.";
  ensureStartupErrorBanner(message);
  throw new Error(message);
}

try {
  mount(App, { target });
} catch (error) {
  ensureStartupErrorBanner(formatError(error));
  throw error;
}
