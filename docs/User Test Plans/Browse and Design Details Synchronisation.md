## Test Plan: 

### Context & Objective

Verify that edits made to one or more designs while navigating sequentially (`Previous` / `Next`) within the `DesignDetails` view correctly persist, update the `Browse` grid UI immediately upon return, and properly trigger active search/filter re-evaluation without requiring a full manual window refresh.

---

### Prerequisites & Setup

* [ ] Launch **Embroidery Catalogue** in debug/dev mode.
* [ ] Ensure the catalog contains at least **10–15 indexed designs**.
* [ ] Apply an active search query or filter on the `Browse` screen (e.g., filter by a specific tag like `Floral` or a specific `Rating` threshold) so you can verify that filtering re-evaluates edited records.

---

### Test Cases

#### 1. Single Record Edit (Baseline)

* [ ] **1.1 Single-field update on active card**
* Select a design card from the `Browse` window.
* In `DesignDetails`, change the **Rating** or **Verified** toggle.
* Return to `Browse`.
* **Expected Result:** The card immediately displays the updated Rating or Verified badge without a manual refresh.


* [ ] **1.2 Filter eviction on single-field update**
* Set a filter in `Browse` for `Verified: False`.
* Open a card matching this filter.
* In `DesignDetails`, mark **Verified** as `True` and save.
* Return to `Browse`.
* **Expected Result:** The card is immediately removed/evicted from the active `Browse` list because it no longer satisfies the `Verified: False` filter predicate.



---

#### 2. Multi-Record Sequential Navigation (`Next` / `Previous`)

* [ ] **2.1 Batch edits across multiple cards**
* Open a card from the `Browse` window (Record A).
* Modify a tag (e.g., add `Needlework`) on Record A.
* Click **Next** to navigate to Record B.
* Modify the **Rating** on Record B.
* Click **Next** to navigate to Record C.
* Toggle **Verified** on Record C.
* Click **Previous** back to Record B and verify your unsaved/saved state holds.
* Return to the `Browse` window.
* **Expected Result:** Cards A, B, and C all render their updated states (tags, rating, verified badge) in the grid simultaneously.


* [ ] **2.2 Search & filter reactivity on batch edits**
* Filter `Browse` by a search term or tag (e.g., `Project: Holiday`).
* Open the first matching card.
* Sequentially edit 3 cards using **Next**:
* Add the tag `Holiday` to a design that didn't have it previously.
* Remove the tag `Holiday` from a design that had it.


* Return to `Browse`.
* **Expected Result:** The new design now appears in the filtered grid view, and the removed design is filtered out. The active filter count updates correctly.



---

#### 3. Edge Cases & Performance

* [ ] **3.1 Selective card re-rendering**
* Open `DesignDetails`, navigate through 5 designs, but only edit **1** design.
* Return to `Browse`.
* **Expected Result:** Only the single modified card component re-renders/patches state; non-edited cards remain untouched with no visual flicker or full grid resetting.


* [ ] **3.2 Rapid sequential navigation without edits**
* Open `DesignDetails` and rapidly click **Next** 10 times without changing any fields.
* Return to `Browse`.
* **Expected Result:** No unnecessary state updates or database sync events are dispatched; grid state remains clean and performant.


* [ ] **3.3 Future-Proofing: Asynchronous background updates (Hoop Recalculation)**
* *(For testing when hoop recalculation is active)*
* Trigger a hoop recalculation on a design and immediately hit **Next** to move to another design or return to `Browse`.
* **Expected Result:** The backend background task completes, emits the update, and the corresponding card reflects the recalculated hoop dimensions in the background without locking the UI or corrupting adjacent card states.



---

### Post-Test Validation

* [ ] All `- [ ]` checkboxes passed.
* [ ] No regression errors or console warnings regarding duplicate key updates in Svelte list loops.