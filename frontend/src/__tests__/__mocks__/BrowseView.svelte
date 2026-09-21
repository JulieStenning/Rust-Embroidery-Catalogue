<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  let { browseNeedsRefresh } = $props();
</script>

<div data-testid="browse-view">
  Browse View
  {#if browseNeedsRefresh}
    <p data-testid="browse-needs-refresh">refresh-requested</p>
  {/if}
</div>
