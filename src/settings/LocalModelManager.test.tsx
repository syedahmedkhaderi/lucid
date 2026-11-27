import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const localModels = vi.hoisted(() => ({
  engines: [] as Array<Record<string, unknown>>,
  delete: vi.fn(),
}));

vi.mock("../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../lib/ipc")>()),
  listLocalSTTEngines: async () => localModels.engines,
  deleteLocalSTTModel: async (engine: string, modelId: string) => {
    localModels.delete(engine, modelId);
    localModels.engines = localModels.engines.map((entry) => ({
      ...entry,
      models: (entry.models as Array<Record<string, unknown>>).filter((model) => model.id !== modelId),
    }));
  },
}));

vi.mock("../hooks/useModelDownload", () => ({
  useModelDownload: () => ({ downloads: {}, startDownload: vi.fn(), cancelDownload: vi.fn() }),
}));

import { LocalModelManager } from "./LocalModelManager";
import { useMeetingStore } from "../stores/meetingStore";
import { useConfigStore } from "../stores/configStore";

const engine = () => ({
  engine: "parakeet_tdt",
  name: "NVIDIA Parakeet",
  description: "Local English transcription",
  models: [{
    id: "parakeet-tdt-0.6b-v3-int8",
    engine: "parakeet_tdt",
    name: "Parakeet TDT 0.6B v3",
    size_bytes: 487_211_443,
    disk_usage_bytes: 670_576_784,
    accuracy_rating: 5,
    speed_rating: 5,
    is_streaming: true,
    is_downloaded: true,
  }],
});

beforeEach(() => {
  localModels.engines = [engine()];
  localModels.delete.mockReset();
  useMeetingStore.setState({ isRecording: false, meetingStarting: false });
  useConfigStore.setState({ activeWhisperModel: null, activeModelPerEngine: {}, meetingAudioConfig: null });
});

afterEach(cleanup);

describe("LocalModelManager", () => {
  it("shows actual storage and confirms before deleting a downloaded model", async () => {
    const user = userEvent.setup();
    render(<LocalModelManager />);

    await screen.findByText("Downloaded models use 640 MB. Delete a model to free space.");
    await user.click(await screen.findByRole("button", { name: /Delete Parakeet TDT 0\.6B v3 and free 640 MB/i }));
    expect(screen.getByRole("alertdialog", { name: "Delete Parakeet TDT 0.6B v3?" })).toBeInTheDocument();
    expect(screen.getByText(/removes about 640 MB from this device/i)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Delete model" }));
    await waitFor(() => expect(localModels.delete).toHaveBeenCalledWith("parakeet_tdt", "parakeet-tdt-0.6b-v3-int8"));
    await waitFor(() => expect(screen.queryByText("Parakeet TDT 0.6B v3")).toBeNull());
  });

  it("prevents model deletion during a meeting", async () => {
    useMeetingStore.setState({ isRecording: true });
    render(<LocalModelManager />);

    expect(await screen.findByRole("button", { name: /Delete Parakeet TDT 0\.6B v3/i })).toBeDisabled();
    expect(localModels.delete).not.toHaveBeenCalled();
  });
});
