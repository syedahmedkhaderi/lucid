import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/plugin-store", () => ({
  load: async () => ({ set: async () => {}, get: async () => undefined, onKeyChange: async () => () => {} }),
}));
vi.mock("./MeetingAudioSettings", () => ({ MeetingAudioSettings: () => <div>Audio settings content</div> }));
vi.mock("./LLMSettings", () => ({ LLMSettings: () => <div>AI answers content</div> }));
vi.mock("./STTSettings", () => ({ STTSettings: () => <div>Speech models content</div> }));
vi.mock("./PrivacySettings", () => ({ PrivacySettings: () => <div>Privacy content</div> }));
vi.mock("./AboutSettings", () => ({ AboutSettings: () => <div>About content</div> }));
vi.mock("../sources/PersonalSourcesEditor", () => ({ PersonalSourcesEditor: () => <div>Documents content</div> }));

import { SettingsOverlay } from "./SettingsOverlay";
import { useMeetingStore } from "../stores/meetingStore";

describe("simplified settings navigation", () => {
  beforeEach(() => {
    useMeetingStore.setState({ currentView: "settings", previousView: "launcher", settingsTab: null });
  });
  afterEach(cleanup);

  it("shows the six essentials first and keeps legacy settings collapsed under Advanced", () => {
    render(<SettingsOverlay />);

    const nav = screen.getByRole("tablist", { name: "Settings navigation" });
    for (const label of ["Audio & Devices", "Speech & Models", "AI Answers", "Documents", "Privacy", "About"]) {
      expect(nav).toHaveTextContent(label);
    }
    expect(nav).not.toHaveTextContent("Translation");
    expect(screen.getByText("Advanced").closest("details")).not.toHaveAttribute("open");

    fireEvent.click(screen.getByText("Advanced"));
    expect(screen.getByRole("tab", { name: "AI Actions" })).toBeInTheDocument();
    expect(screen.queryByRole("tab", { name: "Translation" })).toBeNull();
  });
});
