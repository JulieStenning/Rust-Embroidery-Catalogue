<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script lang="ts">
  import type { DependencyItem } from "../types/licence";
  import LicenceTermsModal from "./LicenceTermsModal.svelte";

  interface Props {
    items: DependencyItem[];
    dependencyType?: "Rust" | "Frontend";
    emptyMessage?: string;
  }

  let {
    items = [],
    dependencyType = "Rust",
    emptyMessage = "No dependencies found.",
  }: Props = $props();

  let searchQuery = $state("");
  let selectedLicence = $state("all");
  let activeModalItem = $state<DependencyItem | null>(null);
  let isModalOpen = $state(false);

  /**
   * Distinct licence categories sorted by popularity (item count) descending,
   * then alphabetically.
   */
  const licenceCategories = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const item of items) {
      const lic = item.licence.trim() || "Unknown";
      counts.set(lic, (counts.get(lic) || 0) + 1);
    }
    return Array.from(counts.entries())
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count || a.name.localeCompare(b.name));
  });

  const filteredItems = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    return items.filter((item) => {
      if (selectedLicence !== "all" && item.licence !== selectedLicence) {
        return false;
      }
      if (!q) return true;
      return (
        item.name.toLowerCase().includes(q) ||
        item.licence.toLowerCase().includes(q) ||
        (item.version && item.version.toLowerCase().includes(q))
      );
    });
  });

  function openTermsModal(item: DependencyItem) {
    activeModalItem = item;
    isModalOpen = true;
  }

  function closeTermsModal() {
    isModalOpen = false;
    activeModalItem = null;
  }

  function clearSearch() {
    searchQuery = "";
  }
</script>

<div class="space-y-3 font-sans">
  {#if items.length > 0}
    <!-- Filter and Search Toolbar -->
    <div
      class="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 p-3 bg-[var(--surface-card-subtle)] border border-[var(--border-default)] rounded-lg"
    >
      <div class="flex flex-1 flex-col sm:flex-row items-stretch sm:items-center gap-2">
        <div class="relative flex-1">
          <input
            type="search"
            class="input-field w-full text-xs py-1.5 pl-8 pr-7 rounded bg-[var(--surface-card)] text-[var(--text-primary)] border border-[var(--border-default)]"
            placeholder="Filter by name or licence..."
            bind:value={searchQuery}
            data-testid="dependency-search-input"
            aria-label="Filter dependencies"
          />
          <svg
            class="w-4 h-4 text-[var(--text-muted)] absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
          {#if searchQuery}
            <button
              type="button"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-[var(--text-muted)] hover:text-[var(--text-primary)] text-xs"
              aria-label="Clear search"
              onclick={clearSearch}
            >
              ✕
            </button>
          {/if}
        </div>

        <div class="w-full sm:w-auto">
          <select
            class="input-field w-full sm:w-auto text-xs py-1.5 px-2 rounded bg-[var(--surface-card)] text-[var(--text-primary)] border border-[var(--border-default)]"
            bind:value={selectedLicence}
            data-testid="dependency-licence-filter"
            aria-label="Filter by licence category"
          >
            <option value="all">All licences ({items.length})</option>
            {#each licenceCategories as cat}
              <option value={cat.name}>{cat.name} ({cat.count})</option>
            {/each}
          </select>
        </div>
      </div>

      <div
        class="text-xs text-[var(--text-secondary)] whitespace-nowrap self-end sm:self-center"
        data-testid="dependency-count-summary"
      >
        {#if filteredItems.length === items.length}
          Showing {items.length} {dependencyType === "Rust" ? "crates" : "packages"}
        {:else}
          Showing {filteredItems.length} of {items.length}
          {dependencyType === "Rust" ? "crates" : "packages"}
        {/if}
      </div>
    </div>

    <!-- Data Table -->
    {#if filteredItems.length > 0}
      <div
        class="border border-[var(--border-default)] rounded-lg overflow-hidden bg-[var(--surface-card)] shadow-sm"
      >
        <div class="overflow-x-auto max-h-[600px] overflow-y-auto">
          <table class="w-full text-left text-xs border-collapse">
            <thead
              class="sticky top-0 bg-[var(--surface-card-subtle)] border-b border-[var(--border-default)] text-[var(--text-secondary)] z-10"
            >
              <tr>
                <th scope="col" class="py-2.5 px-4 font-semibold">
                  {dependencyType === "Rust" ? "Crate" : "Package"}
                </th>
                <th scope="col" class="py-2.5 px-4 font-semibold">Licence</th>
                <th scope="col" class="py-2.5 px-4 font-semibold">Repository</th>
                <th scope="col" class="py-2.5 px-4 font-semibold text-right">Terms</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-[var(--border-default)]">
              {#each filteredItems as item}
                <tr class="hover:bg-[var(--surface-hover)] transition-colors">
                  <td class="py-2.5 px-4 font-medium text-[var(--text-primary)]">
                    <span class="dependency-name font-semibold">{item.name}</span>
                    {#if item.version}
                      <span class="text-[var(--text-muted)] text-[11px] ml-1">
                        (v{item.version})
                      </span>
                    {/if}
                  </td>
                  <td class="py-2.5 px-4">
                    <span
                      class="inline-flex items-center text-[11px] font-semibold px-2 py-0.5 rounded bg-[var(--surface-card-subtle)] text-[var(--text-brand)] border border-[var(--border-default)]"
                    >
                      {item.licence}
                    </span>
                  </td>
                  <td class="py-2.5 px-4 max-w-xs truncate">
                    {#if item.repository}
                      <a
                        href={item.repository}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-[var(--text-brand)] hover:underline inline-flex items-center gap-1 font-mono text-[11px]"
                        title={item.repository}
                      >
                        <span class="truncate">{item.repository.replace(/^https?:\/\//, "")}</span>
                        <svg
                          class="w-3 h-3 flex-shrink-0 opacity-70"
                          fill="none"
                          viewBox="0 0 24 24"
                          stroke="currentColor"
                        >
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                          />
                        </svg>
                      </a>
                    {:else}
                      <span class="text-[var(--text-muted)] italic text-[11px]">—</span>
                    {/if}
                  </td>
                  <td class="py-2.5 px-4 text-right whitespace-nowrap">
                    <button
                      type="button"
                      class="menu-button-secondary text-xs px-2.5 py-1"
                      onclick={() => openTermsModal(item)}
                      data-testid="view-terms-button"
                      aria-label="View licence terms for {item.name}"
                    >
                      {item.text ? "View Terms" : "Details"}
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {:else}
      <div
        class="p-6 text-center text-xs text-[var(--text-muted)] bg-[var(--surface-card-subtle)] border border-[var(--border-default)] rounded-lg"
        data-testid="dependency-no-results"
      >
        No dependencies found matching "{searchQuery}"
        {#if selectedLicence !== "all"}
          under {selectedLicence}
        {/if}.
      </div>
    {/if}
  {:else}
    <p class="text-sm text-[var(--text-muted)]" data-testid="dependency-empty-state">
      {emptyMessage}
    </p>
  {/if}
</div>

<LicenceTermsModal
  open={isModalOpen}
  item={activeModalItem}
  onClose={closeTermsModal}
/>

