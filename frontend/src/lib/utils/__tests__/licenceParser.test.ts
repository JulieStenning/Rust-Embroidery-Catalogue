// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, it, expect } from "vitest";
import {
  parseRustLicencesHtml,
  parseRustLicencesHtmlRegex,
} from "../licenceParser";

const SAMPLE_HTML = `
<div class="licenses-list">
  <details class="license-card">
    <summary class="license-header">
      <span class="license-name">Apache License 2.0</span>
      <span class="license-used-by">Crates using this license:</span>
    </summary>
    <div class="license-crates">
      <h4>Used by:</h4>
      <ul>
        <li>
          <strong>tokio</strong> (v1.38.0) — <a href="https://github.com/tokio-rs/tokio" target="_blank" rel="noopener">https://github.com/tokio-rs/tokio</a>
        </li>
        <li>
          <strong>bytes</strong> (v1.6.0) — <a href="https://github.com/tokio-rs/bytes" target="_blank" rel="noopener">https://github.com/tokio-rs/bytes</a>
        </li>
      </ul>
    </div>
    <div class="license-text">
      <pre>Apache License Version 2.0 text here</pre>
    </div>
  </details>
  <details class="license-card">
    <summary class="license-header">
      <span class="license-name">MIT</span>
      <span class="license-used-by">Crates using this license:</span>
    </summary>
    <div class="license-crates">
      <h4>Used by:</h4>
      <ul>
        <li>
          <strong>serde</strong> (v1.0.200) — <a href="https://github.com/serde-rs/serde" target="_blank" rel="noopener">https://github.com/serde-rs/serde</a>
        </li>
      </ul>
    </div>
    <div class="license-text">
      <pre>MIT License text here</pre>
    </div>
  </details>
</div>
`;

describe("licenceParser", () => {
  it("returns an empty array for empty or whitespace-only HTML", () => {
    expect(parseRustLicencesHtml("")).toEqual([]);
    expect(parseRustLicencesHtml("   \n\t  ")).toEqual([]);
    expect(parseRustLicencesHtmlRegex("")).toEqual([]);
  });

  it("correctly parses multiple cards and extracts crate records sorted by name", () => {
    const items = parseRustLicencesHtml(SAMPLE_HTML);
    expect(items).toHaveLength(3);

    // Sorted alphabetically: bytes, serde, tokio
    expect(items[0]).toEqual({
      name: "bytes",
      version: "1.6.0",
      licence: "Apache License 2.0",
      repository: "https://github.com/tokio-rs/bytes",
      text: "Apache License Version 2.0 text here",
    });

    expect(items[1]).toEqual({
      name: "serde",
      version: "1.0.200",
      licence: "MIT",
      repository: "https://github.com/serde-rs/serde",
      text: "MIT License text here",
    });

    expect(items[2]).toEqual({
      name: "tokio",
      version: "1.38.0",
      licence: "Apache License 2.0",
      repository: "https://github.com/tokio-rs/tokio",
      text: "Apache License Version 2.0 text here",
    });
  });

  it("produces consistent results using the regex fallback parser", () => {
    const items = parseRustLicencesHtmlRegex(SAMPLE_HTML);
    expect(items).toHaveLength(3);
    expect(items.map((i) => i.name)).toEqual(["bytes", "serde", "tokio"]);
  });

  it("handles crates with missing repository links gracefully", () => {
    const noRepoHtml = `
      <details class="license-card">
        <summary><span class="license-name">BSD-3-Clause</span></summary>
        <div class="license-crates">
          <ul>
            <li><strong>local-crate</strong> (v0.1.0)</li>
          </ul>
        </div>
        <div class="license-text"><pre>BSD text</pre></div>
      </details>
    `;
    const items = parseRustLicencesHtml(noRepoHtml);
    expect(items).toHaveLength(1);
    expect(items[0].name).toBe("local-crate");
    expect(items[0].version).toBe("0.1.0");
    expect(items[0].repository).toBe("");
    expect(items[0].licence).toBe("BSD-3-Clause");
  });
});

