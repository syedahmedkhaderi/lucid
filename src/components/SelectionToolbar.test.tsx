import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { SelectionToolbar } from "./SelectionToolbar";

afterEach(cleanup);

it("offers copy for selected text without a translation action", async () => {
  const writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText } });
  render(
    <>
      <p>Fact from the meeting brief</p>
      <SelectionToolbar />
    </>,
  );

  const text = screen.getByText("Fact from the meeting brief");
  const selection = window.getSelection()!;
  const range = document.createRange();
  range.selectNodeContents(text);
  Object.defineProperty(range, "getBoundingClientRect", {
    value: () => ({ top: 80, left: 40, width: 100, height: 18, right: 140, bottom: 98, x: 40, y: 80, toJSON: () => ({}) }),
  });
  selection.removeAllRanges();
  selection.addRange(range);
  fireEvent.mouseUp(document);

  const copy = screen.getByRole("button", { name: "Copy selected text" });
  expect(screen.queryByRole("button", { name: /translate/i })).toBeNull();
  fireEvent.click(copy);
  expect(writeText).toHaveBeenCalledWith("Fact from the meeting brief");
});
