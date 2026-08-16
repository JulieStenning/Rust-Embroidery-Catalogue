<script>
  import { onMount } from "svelte";
  import { getAboutDocument } from "../api/commandAdapter";
  import { renderMarkdown } from "../utils/markdown.js";

  let { slug } = $props();

  /** @type {{ slug: string, title: string, description: string, filename: string, document_text: string } | null} */
  let documentItem = $state(null);
  let loading = $state(false);
  let error = $state("");

  /**
   * Determine whether the loaded document should be rendered as Markdown.
   * @param {{ slug?: string, filename?: string } | null | undefined} item
   */
  function shouldRenderAsMarkdown(item) {
    if (!item || typeof item !== "object") return false;
    const slugName = String(item.slug || "").toLowerCase();
    const filename = String(item.filename || "").toLowerCase();
    return slugName === "ai-tagging" || filename.endsWith(".md");
  }

  /**
   * @param {{ slug?: string, filename?: string } | null | undefined} item
   */
  function shouldRenderAsHtml(item) {
    if (!item || typeof item !== "object") return false;
    const slugName = String(item.slug || "").toLowerCase();
    const filename = String(item.filename || "").toLowerCase();
    return slugName === "disclaimer" || filename.endsWith(".html");
  }

  /**
   * @param {string} slugName
   */
  async function loadAboutDocumentView(slugName) {
    const normalizedSlug = String(slugName || "").trim().toLowerCase();
    if (!normalizedSlug) {
      documentItem = null;
      error = "Document not found.";
      return;
    }

    loading = true;
    error = "";

    try {
      const result = await getAboutDocument(normalizedSlug);
      if (normalizedSlug !== String(slug || "").trim().toLowerCase()) return;

      documentItem = result?.item || null;
      if (!documentItem) {
        error = String(result?.error || "Document not found.");
      }
    } catch (e) {
      documentItem = null;
      error = `Could not load document: ${e}`;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    if (slug) {
      loadAboutDocumentView(slug);
    }
  });
</script>

<div class="max-w-5xl mx-auto space-y-4 font-sans">
  <div class="bg-white rounded-xl shadow p-6">
    {#if loading}
      <p class="text-sm text-gray-500">Loading document...</p>
    {:else if error}
      <p class="text-sm text-red-650 bg-red-50 border border-red-200 rounded p-3">{error}</p>
    {:else if documentItem?.document_text}
      {#if shouldRenderAsHtml(documentItem)}
        <div class="text-sm text-gray-700 bg-gray-50 border rounded-lg p-4 space-y-4 shadow-inner">
          {@html documentItem.document_text}
        </div>
      {:else if shouldRenderAsMarkdown(documentItem)}
        <div class="text-sm text-gray-700 bg-gray-50 border rounded-lg p-4 prose prose-gray max-w-none shadow-inner document-markdown">
          {@html renderMarkdown(documentItem.document_text)}
        </div>
      {:else}
        <pre class="whitespace-pre-wrap text-sm text-gray-700 bg-gray-50 border rounded-lg p-4 overflow-x-auto font-mono shadow-inner">{documentItem.document_text}</pre>
      {/if}
    {:else}
      <p class="text-sm text-gray-500 italic">Document content is unavailable.</p>
    {/if}
  </div>
</div>

<style>
  /* Injected {@html} Markdown is not Svelte-scoped, so target it via a
     dedicated class with :global() selectors. Tailwind Typography (the
     "prose" classes) is not installed, so restore heading hierarchy and
     vertical rhythm manually. */
  :global(.document-markdown h1) {
    font-size: 1.75rem;
    line-height: 1.25;
    font-weight: 700;
    margin: 0 0 0.75rem;
    color: #111827;
  }

  :global(.document-markdown h2) {
    font-size: 1.375rem;
    line-height: 1.3;
    font-weight: 650;
    margin: 1.5rem 0 0.5rem;
    padding-bottom: 0.25rem;
    border-bottom: 1px solid #e5e7eb;
    color: #1f2937;
  }

  :global(.document-markdown h3) {
    font-size: 1.125rem;
    line-height: 1.35;
    font-weight: 650;
    margin: 1.25rem 0 0.5rem;
    color: #1f2937;
  }

  :global(.document-markdown h4) {
    font-size: 1rem;
    line-height: 1.4;
    font-weight: 650;
    margin: 1rem 0 0.5rem;
    color: #1f2937;
  }

  :global(.document-markdown p) {
    margin: 0.5rem 0;
  }

  :global(.document-markdown ul),
  :global(.document-markdown ol) {
    margin: 0.5rem 0 0.5rem 1.25rem;
    padding-left: 1rem;
  }

  :global(.document-markdown li) {
    margin: 0.25rem 0;
  }

  :global(.document-markdown blockquote) {
    margin: 0.75rem 0;
    padding-left: 0.75rem;
    border-left: 3px solid #c7d2fe;
    color: #4b5563;
  }

  :global(.document-markdown a) {
    color: #4f46e5;
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  :global(.document-markdown code) {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.875em;
    background: #f3f4f6;
    border-radius: 0.25rem;
    padding: 0.125rem 0.25rem;
  }

  :global(.document-markdown pre) {
    margin: 0.75rem 0;
    padding: 0.75rem;
    border-radius: 0.375rem;
    background: #1f2937;
    color: #e5e7eb;
    overflow-x: auto;
  }

  @media (prefers-color-scheme: dark) {
    :global(.document-markdown h1),
    :global(.document-markdown h2),
    :global(.document-markdown h3),
    :global(.document-markdown h4) {
      color: #f1f5f9;
    }

    :global(.document-markdown h2) {
      border-bottom-color: #334155;
    }

    :global(.document-markdown blockquote) {
      color: #cbd5e1;
      border-left-color: #6366f1;
    }

    :global(.document-markdown code) {
      background: #1e293b;
      color: #e2e8f0;
    }

    :global(.document-markdown a) {
      color: #818cf8;
    }
  }
</style>
