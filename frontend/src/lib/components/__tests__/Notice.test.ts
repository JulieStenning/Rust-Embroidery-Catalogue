import "@testing-library/jest-dom/vitest";
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import Notice from "../Notice.svelte";

// ---------------------------------------------------------------------------
// Notice.svelte — a small presentational component with no external
// dependencies, so no mocks are required.
//
// Behaviour under test:
//   - Renders only when `message` is truthy.
//   - Always applies the base classes: border, rounded, px-3, py-2, text-sm.
//   - `error=true` forces the error styling regardless of `type`.
//   - Otherwise, `resolvedType` is driven by `type`:
//       * "success"          → green
//       * "error" / "danger" → red
//       * "info" / "default" → blue
//       * anything else     → base classes only, no colour styling
// ---------------------------------------------------------------------------

describe("Notice", () => {
  describe("rendering", () => {
    it("renders the message text when a message prop is provided", () => {
      render(Notice, { props: { message: "Operation completed" } });

      expect(screen.getByText("Operation completed")).toBeInTheDocument();
    });

    it("renders nothing when the message prop is an empty string", () => {
      render(Notice, { props: { message: "" } });

      // Svelte 5 leaves an inert comment node behind, so assert on the
      // absence of any rendered notice element rather than an empty container.
      expect(screen.queryByText(/./)).not.toBeInTheDocument();
    });

    it("always applies the base styling classes", () => {
      render(Notice, { props: { message: "Hello", type: "info" } });

      const div = screen.getByText("Hello");
      expect(div).toHaveClass("border", "rounded", "px-3", "py-2", "text-sm");
    });
  });

  describe("type styling", () => {
    it("applies green styling for type='success'", () => {
      render(Notice, { props: { message: "Saved", type: "success" } });

      const div = screen.getByText("Saved");
      expect(div).toHaveClass("bg-green-50", "border-green-300", "text-green-800");
    });

    it("applies red styling for type='error'", () => {
      render(Notice, { props: { message: "Failed", type: "error" } });

      const div = screen.getByText("Failed");
      expect(div).toHaveClass("bg-red-50", "border-red-300", "text-red-800");
    });

    it("treats type='danger' the same as 'error'", () => {
      render(Notice, { props: { message: "Careful", type: "danger" } });

      const div = screen.getByText("Careful");
      expect(div).toHaveClass("bg-red-50", "border-red-300", "text-red-800");
    });

    it("applies blue styling for type='info'", () => {
      render(Notice, { props: { message: "Heads up", type: "info" } });

      const div = screen.getByText("Heads up");
      expect(div).toHaveClass("bg-blue-50", "border-blue-200", "text-blue-800");
    });

    it("treats type='default' the same as 'info'", () => {
      render(Notice, { props: { message: "Heads up", type: "default" } });

      const div = screen.getByText("Heads up");
      expect(div).toHaveClass("bg-blue-50", "border-blue-200", "text-blue-800");
    });

    it("defaults to info styling when no type prop is supplied", () => {
      render(Notice, { props: { message: "Default notice" } });

      const div = screen.getByText("Default notice");
      expect(div).toHaveClass("bg-blue-50", "border-blue-200", "text-blue-800");
    });

    it("applies no colour styling for an unknown type value", () => {
      render(Notice, { props: { message: "Odd type", type: "warning" } });

      const div = screen.getByText("Odd type");
      expect(div).toHaveClass("border", "rounded", "px-3", "py-2", "text-sm");
      expect(div).not.toHaveClass(
        "bg-green-50",
        "border-green-300",
        "text-green-800",
        "bg-red-50",
        "border-red-300",
        "text-red-800",
        "bg-blue-50",
        "border-blue-200",
        "text-blue-800"
      );
    });
  });

  describe("error prop override", () => {
    it("forces red styling when error=true even with type='success'", () => {
      render(Notice, { props: { message: "Overridden", type: "success", error: true } });

      const div = screen.getByText("Overridden");
      expect(div).toHaveClass("bg-red-50", "border-red-300", "text-red-800");
      expect(div).not.toHaveClass("bg-green-50", "border-green-300", "text-green-800");
    });

    it("forces red styling when error=true even with type='info'", () => {
      render(Notice, { props: { message: "Overridden", type: "info", error: true } });

      const div = screen.getByText("Overridden");
      expect(div).toHaveClass("bg-red-50", "border-red-300", "text-red-800");
      expect(div).not.toHaveClass("bg-blue-50", "border-blue-200", "text-blue-800");
    });

    it("does not force red styling when error is false", () => {
      render(Notice, { props: { message: "Not an error", type: "success", error: false } });

      const div = screen.getByText("Not an error");
      expect(div).toHaveClass("bg-green-50", "border-green-300", "text-green-800");
      expect(div).not.toHaveClass("bg-red-50", "border-red-300", "text-red-800");
    });
  });
});