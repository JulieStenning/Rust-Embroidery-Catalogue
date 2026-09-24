// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, it, expect } from "vitest";
import { resolveDocumentLinks, renderMarkdown, DOCUMENT_SLUG_BY_FILENAME } from "../markdown.js";

describe("markdown utilities", () => {
  describe("DOCUMENT_SLUG_BY_FILENAME", () => {
    it("contains mappings for all expected user-facing guidance files", () => {
      expect(DOCUMENT_SLUG_BY_FILENAME["import_workflow.md"]).toBe("import-workflow");
      expect(DOCUMENT_SLUG_BY_FILENAME["first_import_actions.md"]).toBe("first-import-actions");
      expect(DOCUMENT_SLUG_BY_FILENAME["import_folder_assignment.md"]).toBe(
        "import-folder-assignment"
      );
      expect(DOCUMENT_SLUG_BY_FILENAME["batch_operations_backfill.md"]).toBe("ai-tagging");
      expect(DOCUMENT_SLUG_BY_FILENAME["data_storage_guide.md"]).toBe("data-storage");
      expect(DOCUMENT_SLUG_BY_FILENAME["getting_started.md"]).toBe("getting-started");
      expect(DOCUMENT_SLUG_BY_FILENAME["tag_word_matches.md"]).toBe("tag-word-matches");
    });
  });

  describe("resolveDocumentLinks", () => {
    it("rewrites relative links to registered guides into in-app about document routes", () => {
      const input =
        "See [Import Workflow](IMPORT_WORKFLOW.md) and [First Import](FIRST_IMPORT_ACTIONS.md).";
      const output = resolveDocumentLinks(input);

      expect(output).toBe(
        "See [Import Workflow](#/about/document/import-workflow) and [First Import](#/about/document/first-import-actions)."
      );
    });

    it("handles parent directory prefixes like ../SUPPORTED_FORMATS.md", () => {
      const input = "Check [formats](../SUPPORTED_FORMATS.md) for details.";
      const output = resolveDocumentLinks(input);

      expect(output).toBe("Check [formats](#/about/document/supported-formats) for details.");
    });

    it("preserves in-app hash links", () => {
      const input = "Go to [Word Matches](#/admin/data/tag-matches).";
      expect(resolveDocumentLinks(input)).toBe(input);
    });

    it("preserves absolute HTTP and HTTPS URLs", () => {
      const input = "Visit [Google](https://google.com) or [Local](http://localhost:3000).";
      expect(resolveDocumentLinks(input)).toBe(input);
    });

    it("neutralizes unrecognised relative links to plain text", () => {
      const input = "See [Unrecognised File](nonexistent_file.pdf) for details.";
      expect(resolveDocumentLinks(input)).toBe("See Unrecognised File for details.");
    });
  });

  describe("renderMarkdown", () => {
    it("renders valid HTML with clickable in-app links for guidance files", () => {
      const markdown = "Read [Import Workflow](IMPORT_WORKFLOW.md) for help.";
      const html = renderMarkdown(markdown);

      expect(html).toContain('<a href="#/about/document/import-workflow">Import Workflow</a>');
    });
  });
});
