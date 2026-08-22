import { describe, it, expect } from "vitest";
import {
  DB_MAINTENANCE_STARTED,
  DB_MAINTENANCE_FINISHED,
  type DbMaintenanceStartedEvent,
  type DbMaintenanceFinishedEvent,
} from "../../lib/types/dbMaintenance";

describe("DB_MAINTENANCE_STARTED", () => {
  it("exposes the exact Tauri event name for the start payload", () => {
    expect(DB_MAINTENANCE_STARTED).toBe("db-maintenance-started");
  });
});

describe("DB_MAINTENANCE_FINISHED", () => {
  it("exposes the exact Tauri event name for the finish payload", () => {
    expect(DB_MAINTENANCE_FINISHED).toBe("db-maintenance-finished");
  });
});

describe("DbMaintenanceStartedEvent", () => {
  it("describes the wire fields used by the start payload", () => {
    const event: DbMaintenanceStartedEvent = {
      page_count: 152,
      freelist_pages: 12,
      free_ratio: 0.0789,
      reclaimable_bytes: 98304,
    };

    expect(typeof event.page_count).toBe("number");
    expect(typeof event.freelist_pages).toBe("number");
    expect(typeof event.free_ratio).toBe("number");
    expect(typeof event.reclaimable_bytes).toBe("number");
    expect(event.page_count).toBe(152);
    expect(event.freelist_pages).toBe(12);
    expect(event.free_ratio).toBe(0.0789);
    expect(event.reclaimable_bytes).toBe(98304);
  });
});

describe("DbMaintenanceFinishedEvent", () => {
  it("describes the wire fields used by the finish payload", () => {
    const event: DbMaintenanceFinishedEvent = {
      reclaimed_pages: 12,
      reclaimable_bytes_before: 98304,
      reclaimable_bytes_after: 1024,
      duration_ms: 45,
    };

    expect(typeof event.reclaimed_pages).toBe("number");
    expect(typeof event.reclaimable_bytes_before).toBe("number");
    expect(typeof event.reclaimable_bytes_after).toBe("number");
    expect(typeof event.duration_ms).toBe("number");
    expect(event.reclaimed_pages).toBe(12);
    expect(event.reclaimable_bytes_before).toBe(98304);
    expect(event.reclaimable_bytes_after).toBe(1024);
    expect(event.duration_ms).toBe(45);
  });
});
