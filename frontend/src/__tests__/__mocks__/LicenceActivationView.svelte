<!-- SPDX-FileCopyrightText: 2026 Julie Stenning -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

<script>
  let { onLicenceActivated } = $props();
</script>

<div data-testid="licence-activation-view">
  <button
    data-testid="mock-activate-licence-btn"
    onclick={() => onLicenceActivated?.({ is_valid: true, is_active: true })}
  >
    Activate
  </button>
</div>
