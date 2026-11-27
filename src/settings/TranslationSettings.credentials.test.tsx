import "@testing-library/jest-dom/vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const ipc = vi.hoisted(() => ({
  hasApiKey: vi.fn(),
  getCredentialSetting: vi.fn(),
  setTranslationProvider: vi.fn(),
  testTranslationConnection: vi.fn(),
  getTranslationLanguages: vi.fn(),
  listOpusMtModels: vi.fn(),
  storeApiKey: vi.fn(),
}));

vi.mock("../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../lib/ipc")>()),
  hasApiKey: (...args: unknown[]) => ipc.hasApiKey(...args),
  getCredentialSetting: (...args: unknown[]) => ipc.getCredentialSetting(...args),
  setTranslationProvider: (...args: unknown[]) => ipc.setTranslationProvider(...args),
  testTranslationConnection: (...args: unknown[]) => ipc.testTranslationConnection(...args),
  getTranslationLanguages: (...args: unknown[]) => ipc.getTranslationLanguages(...args),
  listOpusMtModels: (...args: unknown[]) => ipc.listOpusMtModels(...args),
  storeApiKey: (...args: unknown[]) => ipc.storeApiKey(...args),
}));

import { TranslationSettings } from "./TranslationSettings";

describe("TranslationSettings credentials", () => {
  beforeEach(() => {
    ipc.hasApiKey.mockReset().mockImplementation(async (slot: string) => slot === "translation_microsoft");
    ipc.getCredentialSetting.mockReset().mockResolvedValue("westeurope");
    ipc.setTranslationProvider.mockReset().mockResolvedValue(undefined);
    ipc.testTranslationConnection.mockReset().mockResolvedValue({
      connected: true,
      language_count: 100,
      response_ms: 12,
    });
    ipc.getTranslationLanguages.mockReset().mockResolvedValue([]);
    ipc.listOpusMtModels.mockReset().mockResolvedValue([]);
    ipc.storeApiKey.mockReset().mockResolvedValue(undefined);
  });

  afterEach(cleanup);

  it("keeps a saved key out of the input and validates a typed replacement before saving", async () => {
    const user = userEvent.setup();
    render(<TranslationSettings />);

    const key = await screen.findByPlaceholderText("Key stored — type to replace");
    await waitFor(() => expect(key).toHaveValue(""));
    expect(ipc.getCredentialSetting).toHaveBeenCalledWith("translation_microsoft_region");

    await user.type(key, "replacement-key");
    await user.click(screen.getByRole("button", { name: "Save & Test" }));

    await waitFor(() => expect(ipc.testTranslationConnection).toHaveBeenCalledWith(
      "microsoft",
      "replacement-key",
      "westeurope",
    ));
    expect(ipc.setTranslationProvider).toHaveBeenCalledWith(
      "microsoft",
      "westeurope",
      "replacement-key",
    );
    expect(ipc.storeApiKey).not.toHaveBeenCalled();
    await waitFor(() => expect(key).toHaveValue(""));
  });
});
