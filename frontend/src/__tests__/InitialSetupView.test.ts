import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import InitialSetupView from "../lib/InitialSetupView.svelte";

// ---------------------------------------------------------------------------
// Mock the command adapter so we control the completion of setup.
// ---------------------------------------------------------------------------
const completeInitialSetupMock = vi.hoisted(() => vi.fn());
vi.mock("../lib/api/commandAdapter", () => ({
  completeInitialSetup: completeInitialSetupMock,
}));

// Mock the embedded admin views so the wizard's step gating can be tested in
// isolation. Each stub exposes a data-testid so we can assert which step is
// currently visible.
vi.mock("../lib/views/AdminDesignersView.svelte", async () => {
  const { default: AdminDesignersView } = await import(
    "./__mocks__/AdminDesignersView.svelte"
  );
  return { default: AdminDesignersView };
});

vi.mock("../lib/views/AdminSourcesView.svelte", async () => {
  const { default: AdminSourcesView } = await import(
    "./__mocks__/AdminSourcesView.svelte"
  );
  return { default: AdminSourcesView };
});

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
describe("InitialSetupView.svelte", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    completeInitialSetupMock.mockResolvedValue(undefined);
  });

  it("renders the Designers step by default", () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });

    expect(
      screen.getByText("Welcome to Embroidery Catalogue!")
    ).toBeInTheDocument();
    expect(
      screen.getByText("Let's set up your catalogue")
    ).toBeInTheDocument();
    expect(screen.getByText("Step 1 of 2 — Designers")).toBeInTheDocument();
    expect(screen.getByText("What are Designers?")).toBeInTheDocument();
    expect(
      screen.getByText(/Designers are the digitizers or creators/)
    ).toBeInTheDocument();
    expect(screen.getByText(/Why do this now\?/)).toBeInTheDocument();
    expect(
      screen.getByText(/Setting up your frequent designers now/)
    ).toBeInTheDocument();
    expect(screen.getByText("Are they mandatory?")).toBeInTheDocument();
    expect(
      screen.getByText(/Not at all! This step is completely optional/)
    ).toBeInTheDocument();

    expect(screen.getByTestId("admin-designers-view")).toBeInTheDocument();
    expect(screen.queryByTestId("admin-sources-view")).not.toBeInTheDocument();

    const button = screen.getByRole("button", { name: "Continue →" });
    expect(button).toBeEnabled();
  });

  it("advances to the Sources step when Continue is clicked", async () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });
    expect(screen.getByTestId("admin-designers-view")).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    expect(screen.getByText("Step 2 of 2 — Sources")).toBeInTheDocument();
    expect(screen.getByText("What are Sources?")).toBeInTheDocument();
    expect(
      screen.getByText(/Sources describe where your embroidery designs came from/)
    ).toBeInTheDocument();
    expect(screen.getByText(/Setting up your common sources now/)).toBeInTheDocument();
    expect(
      screen.getByText(/Not at all! This step is completely optional/)
    ).toBeInTheDocument();

    expect(screen.getByTestId("admin-sources-view")).toBeInTheDocument();
    expect(screen.queryByTestId("admin-designers-view")).not.toBeInTheDocument();

    expect(completeInitialSetupMock).not.toHaveBeenCalled();
  });

  it("completes setup and invokes the callback on the final step", async () => {
    const onInitialSetupCompleted = vi.fn();
    render(InitialSetupView, { props: { onInitialSetupCompleted } });

    // Move from Designers to Sources.
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    // Finish setup on the Sources step.
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    expect(completeInitialSetupMock).toHaveBeenCalledTimes(1);
    await waitFor(() => {
      expect(onInitialSetupCompleted).toHaveBeenCalledTimes(1);
    });
  });

  it("disables the button and shows 'Saving…' while finishing setup", async () => {
    let resolveSetup: (() => void) | undefined;
    completeInitialSetupMock.mockReturnValue(
      new Promise<void>((resolve) => {
        resolveSetup = resolve;
      })
    );

    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });

    // Move to the final step.
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    const finishButton = screen.getByRole("button", { name: "Continue →" });
    await fireEvent.click(finishButton);

    await waitFor(() => {
      expect(screen.getByText("Saving…")).toBeInTheDocument();
    });
    const savingButton = screen.getByRole("button", { name: "Saving…" });
    expect(savingButton).toBeDisabled();

    // Resolve the pending setup promise and confirm we return to normal state.
    resolveSetup?.();
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Continue →" })
      ).toBeEnabled();
    });
  });

  it("ignores a second continue while already finishing setup", async () => {
    // Keep the promise pending so `finishing` stays true between clicks. A
    // resolved promise would clear `finishing` before the second click fires.
    completeInitialSetupMock.mockReturnValue(new Promise<void>(() => {}));

    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });

    // Move to the final step.
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    // Fire two clicks while the setup promise is still pending. The second
    // invocation must be ignored by the `finishing` guard so
    // completeInitialSetup is called exactly once.
    const finishButton = screen.getByRole("button", { name: "Continue →" });
    await fireEvent.click(finishButton);
    await fireEvent.click(finishButton);

    expect(completeInitialSetupMock).toHaveBeenCalledTimes(1);
    // The button should remain disabled and show "Saving…" because the setup
    // promise is still pending.
    const savingButton = screen.getByRole("button", { name: "Saving…" });
    expect(savingButton).toBeDisabled();
  });

  it("shows an error message when setup completion fails", async () => {
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});
    completeInitialSetupMock.mockRejectedValue(new Error("db locked"));

    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });

    // Move to the final step and attempt to finish.
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));
    await fireEvent.click(screen.getByRole("button", { name: "Continue →" }));

    await waitFor(() => {
      expect(
        screen.getByText(
          "Failed to save setup status: Error: db locked. Please try again."
        )
      ).toBeInTheDocument();
    });
    expect(consoleError).toHaveBeenCalledWith(
      "complete_initial_setup failed:",
      expect.any(Error)
    );
    expect(screen.getByText("Continue →")).toBeInTheDocument();
    expect(screen.queryByText("Saving…")).not.toBeInTheDocument();

    consoleError.mockRestore();
  });

  it("does not show an error message initially", () => {
    render(InitialSetupView, {
      props: { onInitialSetupCompleted: vi.fn() },
    });

    expect(
      screen.queryByText(/Failed to save setup status/)
    ).not.toBeInTheDocument();
  });
});