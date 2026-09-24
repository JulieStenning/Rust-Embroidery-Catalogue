// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import { tick } from "svelte";
import LicenceActivationView from "../LicenceActivationView.svelte";

const activateLicenceMock = vi.hoisted(() => vi.fn());

vi.mock("../api/commandAdapter", () => ({
  activateLicence: activateLicenceMock,
}));

describe("LicenceActivationView.svelte", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the activation form", () => {
    const onLicenceActivated = vi.fn();
    render(LicenceActivationView, { props: { onLicenceActivated } });

    expect(screen.getByTestId("licence-email-input")).toBeInTheDocument();
    expect(screen.getByTestId("licence-key-input")).toBeInTheDocument();
    expect(screen.getByTestId("activate-licence-button")).toBeInTheDocument();
  });

  it("shows error message when activating with empty email", async () => {
    const onLicenceActivated = vi.fn();
    render(LicenceActivationView, { props: { onLicenceActivated } });

    const btn = screen.getByTestId("activate-licence-button");
    await fireEvent.click(btn);
    await tick();

    expect(screen.getByTestId("licence-error-alert")).toHaveTextContent(
      "Please enter your registered email address."
    );
    expect(activateLicenceMock).not.toHaveBeenCalled();
    expect(onLicenceActivated).not.toHaveBeenCalled();
  });

  it("shows error message when activating with empty key", async () => {
    const onLicenceActivated = vi.fn();
    render(LicenceActivationView, { props: { onLicenceActivated } });

    const emailInput = screen.getByTestId("licence-email-input");
    await fireEvent.input(emailInput, { target: { value: "tester@example.com" } });

    const btn = screen.getByTestId("activate-licence-button");
    await fireEvent.click(btn);
    await tick();

    expect(screen.getByTestId("licence-error-alert")).toHaveTextContent(
      "Please enter your licence key."
    );
    expect(activateLicenceMock).not.toHaveBeenCalled();
  });

  it("calls onLicenceActivated on successful activation", async () => {
    const mockStatus = {
      is_active: true,
      is_valid: true,
      email: "tester@example.com",
      tier: "beta",
      expires_at: 1798058308,
      expires_at_formatted: "2026-12-23",
      error_message: null,
    };
    activateLicenceMock.mockResolvedValueOnce(mockStatus);

    const onLicenceActivated = vi.fn();
    render(LicenceActivationView, { props: { onLicenceActivated } });

    const emailInput = screen.getByTestId("licence-email-input");
    const keyInput = screen.getByTestId("licence-key-input");
    const btn = screen.getByTestId("activate-licence-button");

    await fireEvent.input(emailInput, { target: { value: "tester@example.com" } });
    await fireEvent.input(keyInput, { target: { value: "EMB1.abc.def" } });
    await fireEvent.click(btn);
    await tick();

    expect(activateLicenceMock).toHaveBeenCalledWith("tester@example.com", "EMB1.abc.def");
    expect(onLicenceActivated).toHaveBeenCalledWith(mockStatus);
  });

  it("shows backend error message on failed activation", async () => {
    const mockStatus = {
      is_active: true,
      is_valid: false,
      email: "tester@example.com",
      tier: null,
      expires_at: null,
      expires_at_formatted: null,
      error_message: "Licence key does not match the provided email address",
    };
    activateLicenceMock.mockResolvedValueOnce(mockStatus);

    const onLicenceActivated = vi.fn();
    render(LicenceActivationView, { props: { onLicenceActivated } });

    const emailInput = screen.getByTestId("licence-email-input");
    const keyInput = screen.getByTestId("licence-key-input");
    const btn = screen.getByTestId("activate-licence-button");

    await fireEvent.input(emailInput, { target: { value: "wrong@example.com" } });
    await fireEvent.input(keyInput, { target: { value: "EMB1.abc.def" } });
    await fireEvent.click(btn);
    await tick();

    expect(screen.getByTestId("licence-error-alert")).toHaveTextContent(
      "Licence key does not match the provided email address"
    );
    expect(onLicenceActivated).not.toHaveBeenCalled();
  });
});
