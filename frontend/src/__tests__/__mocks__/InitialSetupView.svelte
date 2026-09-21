<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  /** Mock InitialSetupView used by App.svelte tests. */
  let { onInitialSetupCompleted } = $props();
</script>

<div data-testid="initial-setup-view">
  <button onclick={onInitialSetupCompleted} data-testid="complete-initial-setup">
    Complete initial setup
  </button>
</div>
