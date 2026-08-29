## 📋 User Test Suite: [Tri-State Bulk Tagging and Dual Verification]

### 🔄 Core Workflows to Test

### Scope
Testing tri-state checkbox behavior, tag preservation, and independent dual-verification (image_tags_verified and stitching_tags_verified) across @MainView.svelte, @TagSelectionModal.svelte, @TaggingActionsView.svelte, and @DesignDetailView.svelte.

---

### Test Suite 1: Single Design Tagging & Verification

- [ ] **1.1 Single Design - Full Review**  
  Open a single design in @DesignDetailView.svelte or open @TagSelectionModal.svelte for 1 selected design. Make no tag changes and click Save. Verify that both image_tags_verified and stitching_tags_verified are set to true, and verify that the ✓ tick icon appears on the design card in @MainView.svelte.

- [ ] **1.2 Single Design - Add/Remove Tags**  
  Select 1 design, open @TagSelectionModal.svelte, add an Image tag, and remove a Stitching tag. Click Save. Verify tags are correctly updated in SQLite and reflected in @DesignDetailView.svelte. Verify both verification flags are set to true and the ✓ tick icon is visible in @MainView.svelte.

---

### Test Suite 2: Tri-State Checkbox Interactions in Modal

- [ ] **2.1 Initial Mixed-State Display**  
  Select 3 designs where Design A has tag Floral, Design B has tag Floral, and Design C does not have Floral. Open @TagSelectionModal.svelte. Verify that the Floral checkbox displays the indeterminate state [-].

- [ ] **2.2 Tri-State Click Cycling**  
  Click the indeterminate [-] checkbox once: verify it transitions to Checked [✓]. Click the checkbox a second time: verify it transitions to Unchecked [ ]. Click the checkbox a third time: verify it returns to Checked [✓] (or cycles as defined).

- [ ] **2.3 Indeterminate Tag Preservation (No Accidental Deletions)**  
  Select 3 designs with mixed tags (some have satin stitch, some do not). Open @TagSelectionModal.svelte. Add a new image tag Geometric [✓], but leave satin stitch in its indeterminate [-] state. Click Save. Verify that Geometric is added to all 3 designs. Verify that designs that previously had satin stitch still retain it, and designs without it did not receive it.

---

### Test Suite 3: Bulk Tagging Verification Logic Matrix

- [ ] **3.1 Multiple Designs - Uniform Tags (Share Same Tags = Y)**  
  Select multiple designs that share the exact same Image tags and Stitching tags (no indeterminate checkboxes shown). Open @TagSelectionModal.svelte and click Save without making changes. Verify both image_tags_verified and stitching_tags_verified become true for all selected designs. Verify all selected cards display the ✓ tick in @MainView.svelte.

- [ ] **3.2 Multiple Designs - Mixed Tags: Only Image Tags Modified**  
  Select multiple designs with mixed tags across both categories. Open @TagSelectionModal.svelte, add an Image tag (e.g., Border), and leave all Stitching tags untouched ([-] / unchanged). Click Save. Verify image_tags_verified is set to true on all selected designs. Verify stitching_tags_verified retains each design's previous verification state unchanged.

- [ ] **3.3 Multiple Designs - Mixed Tags: Only Stitching Tags Modified**  
  Select multiple designs with mixed tags across both categories. Open @TagSelectionModal.svelte, add a Stitching tag (e.g., fill stitch), and leave all Image tags untouched ([-] / unchanged). Click Save. Verify stitching_tags_verified is set to true on all selected designs. Verify image_tags_verified retains each design's previous verification state unchanged.

- [ ] **3.4 Multiple Designs - Mixed Tags: No Changes Made (Row 11)**  
  Select multiple designs with mixed Image tags and mixed Stitching tags. Open @TagSelectionModal.svelte and click Save without altering any checkboxes. Verify that neither image_tags_verified nor stitching_tags_verified are altered for any of the selected designs.

- [ ] **3.5 Multiple Designs - Mixed Tags: Both Categories Modified**  
  Select multiple designs with mixed tags. Modify at least one Image tag and at least one Stitching tag. Click Save. Verify both image_tags_verified and stitching_tags_verified are set to true across all selected designs.

---

### Test Suite 4: UI Presentation & Indicators

- [ ] **4.1 Browse View Badge (@MainView.svelte)**  
  Check a design where image_tags_verified = true and stitching_tags_verified = false: verify no overall ✓ tick is shown on the card. Check a design where image_tags_verified = false and stitching_tags_verified = true: verify no overall ✓ tick is shown on the card. Check a design where both are true: verify the ✓ tick is visible. Hover over card indicators and check tooltips for accurate status descriptions.

- [ ] **4.2 Design Detail View Breakdown (@DesignDetailView.svelte)**  
  Open a design in @DesignDetailView.svelte. Verify discrete status labels/badges are shown for Image Tags and Stitching Tags independently. Toggle individual verification buttons (if present) and verify local SQLite updates without side effects on the other category.

### ❌ Failed Tests / Discovered Friction
