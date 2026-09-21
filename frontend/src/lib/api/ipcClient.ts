// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

export type LooseRecord = Record<string, unknown>;

export function invokeLoose<T = LooseRecord>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    if (typeof window !== "undefined") {
      const e2eStubs = (
        window as unknown as {
          __E2E_IPC_STUBS__?: Record<string, (args?: Record<string, unknown>) => unknown>;
        }
      )?.__E2E_IPC_STUBS__;
      if (e2eStubs && typeof e2eStubs[command] === "function") {
        return Promise.resolve(e2eStubs[command](args) as T);
      }
    }
    const result = args === undefined ? invoke(command) : invoke(command, args);

    if (result && typeof (result as PromiseLike<T>).then === "function") {
      return new Promise<T>((resolve, reject) => {
        try {
          (result as PromiseLike<T>).then(
            (value) => resolve(value as T),
            (reason) => reject(reason)
          );
        } catch (error) {
          reject(error);
        }
      });
    }

    return Promise.resolve(result as T);
  } catch (error) {
    return Promise.reject(error);
  }
}
