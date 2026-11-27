import { describe, expect, it } from "vitest";
import { responsiveTranscriptFontSize } from "./TranscriptPanel";

describe("live transcript text size", () => {
  it("stays readable in a narrow overlay and scales with wider panels", () => {
    expect(responsiveTranscriptFontSize(260)).toBe(12);
    expect(responsiveTranscriptFontSize(320)).toBe(12);
    expect(responsiveTranscriptFontSize(440)).toBe(13);
    expect(responsiveTranscriptFontSize(680)).toBe(15);
    expect(responsiveTranscriptFontSize(900)).toBe(16);
  });

  it("uses a stable default until the panel has been measured", () => {
    expect(responsiveTranscriptFontSize(0)).toBe(13);
  });
});
