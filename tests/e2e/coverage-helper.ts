// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import type { Page } from "@playwright/test";
import MCR from "monocart-coverage-reports";

export function getCoverageInstance() {
  return MCR({
    name: "Frontend E2E Coverage Report",
    outputDir: "./coverage/e2e",
    entryFilter: (entry: { url?: string }) => {
      return Boolean(
        entry.url &&
        (entry.url.includes("assets") ||
          entry.url.includes("tauri") ||
          entry.url.includes("localhost") ||
          entry.url.includes("index")),
      );
    },
    sourceFilter: (sourcePath: string) => {
      return (
        (sourcePath.includes("src/") || sourcePath.includes("src\\")) &&
        !sourcePath.includes("node_modules") &&
        !sourcePath.includes(".test.")
      );
    },
    reports: [
      "console-summary",
      ["v8"],
      ["html", { subdir: "html" }],
      ["lcovonly", { outputFile: "./coverage/e2e/lcov.info" }],
    ],
  });
}

export async function startCoverage(page: Page): Promise<void> {
  if (page.coverage) {
    await page.coverage.startJSCoverage({ resetOnNavigation: false });
  }
}

export async function stopCoverage(page: Page): Promise<void> {
  if (page.coverage) {
    const coverage = await page.coverage.stopJSCoverage().catch(() => []);
    if (coverage && coverage.length > 0) {
      const mcr = getCoverageInstance();
      await mcr.add(coverage);
    }
  }
}
