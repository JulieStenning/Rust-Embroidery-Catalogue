import { test, expect } from "./fixtures";
import { clickNav, gotoRoute, mainMenu } from "./helpers";

/**
 * End-to-end tests for the "Settings" tab on the Application Settings page
 * (accessed via top navigation "Admin: System" or `#/admin/system/settings`).
 *
 * Exercises the real Tauri WebView2 application, IPC layer, and SQLite database.
 * Uses a mock Gemini API key for safe AI configuration testing.
 */
test.describe.serial("application settings", () => {
  test("navigates to Application Settings from the top menu link and defaults to Settings tab", async ({
    page,
  }) => {
    // Navigate via top menu link
    await clickNav(page, "System");

    // Page title and header are rendered
    await expect(
      page.getByRole("heading", { name: "Application Settings", exact: true }),
    ).toBeVisible();

    // Tablist has Settings selected by default
    const tablist = page.getByTestId("system-maintenance-tablist");
    await expect(tablist).toBeVisible();

    const settingsTab = page.getByTestId("system-tab-settings");
    const backupTab = page.getByTestId("system-tab-backup");
    const orphansTab = page.getByTestId("system-tab-orphans");

    await expect(settingsTab).toBeVisible();
    await expect(settingsTab).toHaveAttribute("aria-selected", "true");
    await expect(backupTab).toBeVisible();
    await expect(backupTab).toHaveAttribute("aria-selected", "false");
    await expect(orphansTab).toBeVisible();
    await expect(orphansTab).toHaveAttribute("aria-selected", "false");

    // System admin link in navbar is active
    const systemNavlink = mainMenu(page).getByRole("link", { name: "System" });
    await expect(systemNavlink).toHaveClass(/menu-link-active/);
  });

  test("deep links to #/admin/system and supports tab switching", async ({
    page,
  }) => {
    // Bare hub URL defaults to settings
    await gotoRoute(page, "#/admin/system");
    await expect(
      page.getByRole("heading", { name: "Application Settings", exact: true }),
    ).toBeVisible();

    const tablist = page.getByTestId("system-maintenance-tablist");
    const settingsTab = page.getByTestId("system-tab-settings");
    const backupTab = page.getByTestId("system-tab-backup");
    const orphansTab = page.getByTestId("system-tab-orphans");

    // Switch to Backup & Restore tab
    await backupTab.click();
    await expect(backupTab).toHaveAttribute("aria-selected", "true");
    await expect(settingsTab).toHaveAttribute("aria-selected", "false");
    await expect(
      page.getByRole("heading", { name: "Backup & Restore" }),
    ).toBeVisible();

    // Switch to Orphaned Files tab
    await orphansTab.click();
    await expect(orphansTab).toHaveAttribute("aria-selected", "true");
    await expect(backupTab).toHaveAttribute("aria-selected", "false");
    await expect(
      page.getByRole("heading", { name: "Orphans" }),
    ).toBeVisible();

    // Switch back to Settings tab
    await settingsTab.click();
    await expect(settingsTab).toHaveAttribute("aria-selected", "true");
    await expect(orphansTab).toHaveAttribute("aria-selected", "false");
    await expect(
      page.getByRole("heading", { name: "Application Settings", exact: true }),
    ).toBeVisible();
  });

  test("renders initial clean form state with Save button disabled", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/settings");
    await expect(
      page.getByRole("heading", { name: "Application Settings", exact: true }),
    ).toBeVisible();

    // Key input controls exist
    await expect(page.locator("#settings-google-api-key")).toBeVisible();
    await expect(page.locator("#settings-ai-batch-size")).toBeVisible();
    await expect(page.locator("#settings-ai-commit-every")).toBeVisible();
    await expect(page.locator("#settings-ai-workers")).toBeVisible();
    await expect(page.locator("#settings-ai-delay")).toBeVisible();
    await expect(page.locator("#settings-ai-model")).toBeVisible();
    await expect(page.locator("#settings-db-idle-check-interval")).toBeVisible();

    // Save button is initially disabled and dirty hint is hidden
    const saveButton = page.getByRole("button", { name: "Save settings" });
    await expect(saveButton).toBeVisible();
    await expect(saveButton).toBeDisabled();
    await expect(page.getByTestId("settings-dirty-hint")).toBeHidden();

    // AI Model controls are disabled when no API key is present
    await expect(page.locator("#settings-ai-model")).toBeDisabled();
    await expect(
      page.getByRole("button", { name: "Refresh", exact: true }),
    ).toBeDisabled();
    await expect(
      page.getByRole("button", { name: "Test model", exact: true }),
    ).toBeDisabled();
  });

  test("toggles API key password visibility", async ({ page }) => {
    await gotoRoute(page, "#/admin/system/settings");
    const apiKeyInput = page.locator("#settings-google-api-key");
    const toggleButton = page.getByRole("button", {
      name: "Show or hide API key",
    });

    // Default: masked password
    await expect(apiKeyInput).toHaveAttribute("type", "password");
    await expect(toggleButton).toHaveAttribute("aria-pressed", "false");

    // Click toggle to reveal
    await toggleButton.click();
    await expect(apiKeyInput).toHaveAttribute("type", "text");
    await expect(toggleButton).toHaveAttribute("aria-pressed", "true");

    // Click toggle to re-mask
    await toggleButton.click();
    await expect(apiKeyInput).toHaveAttribute("type", "password");
    await expect(toggleButton).toHaveAttribute("aria-pressed", "false");
  });

  test("tracks dirty state and reverts when value is restored", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/settings");
    const batchSizeInput = page.locator("#settings-ai-batch-size");
    const saveButton = page.getByRole("button", { name: "Save settings" });
    const dirtyHint = page.getByTestId("settings-dirty-hint");

    const originalValue = await batchSizeInput.inputValue();

    // Modify value -> marks dirty
    await batchSizeInput.fill("123");
    await expect(dirtyHint).toBeVisible();
    await expect(saveButton).toBeEnabled();

    // Revert value -> marks clean
    await batchSizeInput.fill(originalValue);
    await expect(dirtyHint).toBeHidden();
    await expect(saveButton).toBeDisabled();
  });

  test("toggles free-tier checkbox and updates placeholder hints dynamically", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/settings");
    const freeTierCheckbox = page.locator(
      'input[type="checkbox"]',
    );
    const workersInput = page.locator("#settings-ai-workers");
    const delayInput = page.locator("#settings-ai-delay");

    // Initially unchecked (standard tier: workers 4, delay 0)
    if (await freeTierCheckbox.isChecked()) {
      await freeTierCheckbox.uncheck();
      const saveBtn = page.getByRole("button", { name: "Save settings" });
      if (await saveBtn.isEnabled()) await saveBtn.click();
    }

    await expect(workersInput).toHaveAttribute("placeholder", /e\.g\. 4/);
    await expect(delayInput).toHaveAttribute("placeholder", /e\.g\. 0/);

    // Check free tier -> updates placeholders (workers 2 (free tier), delay 10)
    await freeTierCheckbox.check();
    await expect(page.getByTestId("settings-dirty-hint")).toBeVisible();
    await expect(workersInput).toHaveAttribute(
      "placeholder",
      /e\.g\. 2 \(free tier\)/,
    );
    await expect(delayInput).toHaveAttribute("placeholder", /e\.g\. 10/);

    // Uncheck free tier -> restores placeholders
    await freeTierCheckbox.uncheck();
    await expect(workersInput).toHaveAttribute("placeholder", /e\.g\. 4/);
    await expect(delayInput).toHaveAttribute("placeholder", /e\.g\. 0/);
  });

  test("configures mock Gemini API key, tests UI enablement, and persists across reload", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/settings");

    const apiKeyInput = page.locator("#settings-google-api-key");
    const modelSelect = page.locator("#settings-ai-model");
    const refreshButton = page.getByRole("button", {
      name: "Refresh",
      exact: true,
    });
    const testModelButton = page.getByRole("button", {
      name: "Test model",
      exact: true,
    });
    const saveButton = page.getByRole("button", { name: "Save settings" });
    const dirtyHint = page.getByTestId("settings-dirty-hint");

    // Enter mock Gemini API key
    const mockApiKey = "mock-gemini-api-key-playwright-e2e";
    await apiKeyInput.fill(mockApiKey);

    // Form is now dirty
    await expect(dirtyHint).toBeVisible();
    await expect(saveButton).toBeEnabled();

    // Model controls are now dynamically enabled
    await expect(modelSelect).toBeEnabled();
    await expect(refreshButton).toBeEnabled();
    await expect(testModelButton).toBeEnabled();

    // Helper text reflects that a key is now present
    await expect(
      page.getByText(
        "A key is currently saved. You can leave it as-is or replace it here.",
      ),
    ).toBeVisible();

    // Test model button triggers test without crashing
    await testModelButton.click();
    // After clicking Test model with an empty model selection, it warns to select a model
    await expect(
      page.getByText("Enter or pick a Gemini model to test."),
    ).toBeVisible();

    // Set other settings values
    await page.locator("#settings-ai-batch-size").fill("150");
    await page.locator("#settings-ai-commit-every").fill("75");
    await page.locator("#settings-ai-workers").fill("3");
    await page.locator("#settings-ai-delay").fill("1.5");
    await page.locator("#settings-db-idle-check-interval").fill("900");

    // Save settings
    await saveButton.click();

    // Verify save completed and dirty hint cleared
    await expect(dirtyHint).toBeHidden();
    await expect(
      page.getByText("Settings saved successfully."),
    ).toBeVisible();

    // Reload page to verify SQLite round-trip persistence
    await page.reload();
    await expect(
      page.getByRole("heading", { name: "Application Settings", exact: true }),
    ).toBeVisible();

    await expect(page.locator("#settings-google-api-key")).toHaveValue(
      mockApiKey,
    );
    await expect(page.locator("#settings-ai-batch-size")).toHaveValue("150");
    await expect(page.locator("#settings-ai-commit-every")).toHaveValue("75");
    await expect(page.locator("#settings-ai-workers")).toHaveValue("3");
    await expect(page.locator("#settings-ai-delay")).toHaveValue("1.5");
    await expect(
      page.locator("#settings-db-idle-check-interval"),
    ).toHaveValue("900");

    // Model dropdown remains enabled on re-mount
    await expect(page.locator("#settings-ai-model")).toBeEnabled();
  });

  test("runs manual database compaction and refreshes statistics", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/settings");

    // Maintenance & diagnostics section is present
    await expect(
      page.getByRole("heading", { name: "Maintenance & diagnostics" }),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Database Maintenance" }),
    ).toBeVisible();

    // DB stats badges render
    await expect(page.getByText("Database size")).toBeVisible();
    await expect(page.getByText("Recoverable")).toBeVisible();

    // Click "Optimize & Compact Database"
    const compactButton = page.getByRole("button", {
      name: "Optimize & Compact Database",
    });
    await expect(compactButton).toBeVisible();
    await expect(compactButton).toBeEnabled();

    await compactButton.click();

    // Assert completion toast with pages reclaimed
    await expect(
      page.getByText(/Database compacted — \d+ pages reclaimed/),
    ).toBeVisible();
  });

  test("renders read-only storage locations and documentation links", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/settings");

    // Storage locations section
    await expect(
      page.getByRole("heading", { name: "Storage locations" }),
    ).toBeVisible();
    await expect(
      page.getByText("The catalogue database and imported embroidery files live under"),
    ).toBeVisible();

    const metaCard = page.locator(".settings-meta");

    // Monospace code blocks for paths
    const codeBlocks = metaCard.locator("code.settings-code");
    await expect(codeBlocks).toHaveCount(3);

    // Catalogue data location
    await expect(
      metaCard.getByText("Catalogue data location", { exact: true }),
    ).toBeVisible();
    // Database path
    await expect(
      metaCard.getByText("Database", { exact: true }),
    ).toBeVisible();
    // Log folder
    await expect(metaCard.getByText("Log folder", { exact: true })).toBeVisible();

    // Contextual cross-links in the form
    const form = page.locator("form.settings-form");
    const helpLink = form.getByRole("link", {
      name: "Press here for more information.",
    });
    await expect(helpLink).toHaveAttribute("href", "#/help");

    const batchOpsLink = form.getByRole("link", {
      name: "Batch Operations",
    });
    await expect(batchOpsLink).toHaveAttribute(
      "href",
      "#/admin/batch-operations",
    );
  });

  test("cleans up mock API key and restores original settings", async ({
    page,
  }) => {
    await gotoRoute(page, "#/admin/system/settings");

    const apiKeyInput = page.locator("#settings-google-api-key");
    await apiKeyInput.fill("");

    // Restore numeric defaults
    await page.locator("#settings-ai-batch-size").fill("");
    await page.locator("#settings-ai-commit-every").fill("");
    await page.locator("#settings-ai-workers").fill("");
    await page.locator("#settings-ai-delay").fill("");
    await page.locator("#settings-db-idle-check-interval").fill("1800");

    const saveButton = page.getByRole("button", { name: "Save settings" });
    await expect(saveButton).toBeEnabled();
    await saveButton.click();

    await expect(page.getByTestId("settings-dirty-hint")).toBeHidden();
    await expect(
      page.getByText("Settings saved successfully."),
    ).toBeVisible();

    // Verify clean state on reload
    await page.reload();
    await expect(page.locator("#settings-google-api-key")).toHaveValue("");
    await expect(page.locator("#settings-ai-model")).toBeDisabled();
  });
});