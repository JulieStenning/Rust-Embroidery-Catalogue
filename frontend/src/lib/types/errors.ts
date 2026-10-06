// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

export type IpcErrorCode =
  | "not_found"
  | "invalid_input"
  | "database"
  | "io"
  | "parse"
  | "unsupported"
  | "cancelled"
  | "internal";

export interface IpcError {
  code: IpcErrorCode;
  message: string;
}

export function isIpcError(error: unknown): error is IpcError {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    "message" in error &&
    typeof (error as Record<string, unknown>).code === "string" &&
    typeof (error as Record<string, unknown>).message === "string"
  );
}
