import { describe, expect, it } from "vitest";
import { compactOverlayBounds } from "./compactOverlay";

describe("compact overlay placement", () => {
  it("centers the marker in the usable top edge of a scaled monitor", () => {
    expect(compactOverlayBounds({
      scaleFactor: 2,
      position: { x: 0, y: 0 },
      size: { width: 3024, height: 1964 },
      workArea: { position: { x: 0, y: 76 }, size: { width: 3024, height: 1888 } },
    })).toEqual({ x: 1364, y: 96, width: 296, height: 176 });
  });

  it("uses the monitor bounds when a work area is unavailable", () => {
    expect(compactOverlayBounds({
      scaleFactor: 1,
      position: { x: -1920, y: 0 },
      size: { width: 1920, height: 1080 },
    })).toEqual({ x: -1034, y: 10, width: 148, height: 88 });
  });

  it("safely falls back to a unit scale factor", () => {
    expect(compactOverlayBounds({
      scaleFactor: 0,
      position: { x: 0, y: 0 },
      size: { width: 1920, height: 1080 },
    }).width).toBe(148);
  });
});
