import { describe, expect, it } from "vitest";
import type { WorkspaceCitation } from "../../lib/types";
import { citationWhere } from "./citations";

describe("citationWhere", () => {
  const citation: WorkspaceCitation = {
    tag: "S1",
    kind: "document",
    ref_id: "d1",
    sub_ref: "page:12",
    title: "Pricing.pdf",
    start_ms: null,
    snippet: "",
  };

  it("shows a PDF page number in the source label", () => {
    expect(citationWhere(citation)).toBe("Pricing.pdf, page 12");
  });

  it("does not interpret arbitrary sub references as page numbers", () => {
    expect(citationWhere({ ...citation, sub_ref: "page:../12" })).toBe("Pricing.pdf");
  });
});
