import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { MeetingSetupModal } from "./MeetingSetupModal";
import { useConfigStore } from "../stores/configStore";
import { useScenarioStore } from "../stores/scenarioStore";
import { FALLBACK_GENERAL, useWorkspaceStore } from "../stores/workspaceStore";
import type { AIScenario, ScenarioTemplate } from "../lib/types";

const boardReview = {
  id: "custom_1",
  name: "Board review",
  description: "Custom scenario",
  is_custom: true,
} as ScenarioTemplate;

const start = async () => {
  await waitFor(() => expect(screen.getByRole("button", { name: "Start Meeting" })).toBeEnabled());
  fireEvent.click(screen.getByRole("button", { name: "Start Meeting" }));
};

function remember(scenario: AIScenario) {
  useConfigStore.setState({ rememberedMeetingSetup: { audioMode: "online", scenario } });
}

beforeAll(() => {
  mockWindows("launcher");
  mockIPC(() => []);
});

beforeEach(() => {
  useScenarioStore.setState({ customScenarios: [boardReview] });
});

afterEach(() => {
  cleanup();
  mockIPC(() => []);
  useConfigStore.setState({ rememberedMeetingSetup: null });
  useScenarioStore.setState({ customScenarios: [] });
});

describe("MeetingSetupModal scenarios", () => {
  it("offers custom scenarios and starts the chosen one", async () => {
    const onStart = vi.fn();
    render(<MeetingSetupModal open onStart={onStart} onCancel={() => {}} />);
    fireEvent.click(screen.getByRole("button", { name: "Team Meeting" }));
    fireEvent.click(screen.getByRole("button", { name: /Board review/ }));
    await start();
    expect(onStart).toHaveBeenCalledWith("online", "custom_1", { workspaceId: "general", borrowed: [], documentIds: [], meetingContext: "" });
  });

  it("passes optional meeting context with the selected documents", async () => {
    const onStart = vi.fn();
    render(<MeetingSetupModal open onStart={onStart} onCancel={() => {}} />);
    fireEvent.click(screen.getByText("Add optional meeting context"));
    fireEvent.change(screen.getByRole("textbox", { name: "Optional meeting context" }), {
      target: { value: "Review Q3 renewals and explain each variance." },
    });
    await start();
    expect(onStart).toHaveBeenCalledWith("online", "team_meeting", {
      workspaceId: "general",
      borrowed: [],
      documentIds: [],
      meetingContext: "Review Q3 renewals and explain each variance.",
    });
  });

  it("shows a remembered custom scenario by name", async () => {
    remember("custom_1");
    const onStart = vi.fn();
    render(<MeetingSetupModal open onStart={onStart} onCancel={() => {}} />);
    expect(screen.getByText("Board review")).toBeInTheDocument();
    await start();
    expect(onStart).toHaveBeenCalledWith("online", "custom_1", { workspaceId: "general", borrowed: [], documentIds: [], meetingContext: "" });
  });

  it("falls back to Team Meeting when the remembered scenario was deleted", async () => {
    remember("custom_deleted");
    const onStart = vi.fn();
    render(<MeetingSetupModal open onStart={onStart} onCancel={() => {}} />);
    expect(screen.getByText("Team Meeting")).toBeInTheDocument();
    expect(screen.queryByText("custom_deleted")).toBeNull();
    await start();
    expect(onStart).toHaveBeenCalledWith("online", "team_meeting", { workspaceId: "general", borrowed: [], documentIds: [], meetingContext: "" });
  });
});

describe("MeetingSetupModal workspaces", () => {
  const start2 = async () => {
    await waitFor(() => expect(screen.getByRole("button", { name: "Start Meeting" })).toBeEnabled());
    fireEvent.click(screen.getByRole("button", { name: "Start Meeting" }));
  };
  const a = { id: "a", name: "Acme", description: "", default_scenario: null, keep_separate: false, archived: false, is_default: false } as never;
  const b = { id: "b", name: "Biology", description: "", default_scenario: "interview", keep_separate: false, archived: false, is_default: false } as never;
  const k = { id: "k", name: "Vault", description: "", default_scenario: null, keep_separate: true, archived: false, is_default: false } as never;
  const general = { id: "general", name: "General", description: "", default_scenario: null, keep_separate: false, archived: false, is_default: true } as never;

  beforeEach(() => {
    useWorkspaceStore.setState({ workspaces: [general, a, b, k] });
  });
  afterEach(() => {
    useWorkspaceStore.setState({ workspaces: [FALLBACK_GENERAL] });
  });

  it("hides the choice when there is only General", () => {
    useWorkspaceStore.setState({ workspaces: [FALLBACK_GENERAL] });
    render(<MeetingSetupModal open onStart={() => {}} onCancel={() => {}} />);
    expect(screen.queryByRole("radiogroup", { name: "Workspace" })).toBeNull();
  });

  it("preselects at most ten workspace documents and passes the edited selection", async () => {
    mockIPC((command) => command === "get_workspace"
      ? { items: Array.from({ length: 12 }, (_, i) => ({
          workspace_id: "general",
          kind: "document",
          ref_id: `doc-${i}`,
          label: null,
          name: `Proposal ${i}`,
          pinned: false,
          status: "ready",
          detail: "pdf",
          tokens: 5,
          added_at: 0,
        })) }
      : []);
    useWorkspaceStore.setState({ workspaces: [FALLBACK_GENERAL] });
    const onStart = vi.fn();
    render(<MeetingSetupModal open onStart={onStart} onCancel={() => {}} />);

    expect(await screen.findByText("10/10 selected")).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "Proposal 9 in General" })).toBeChecked();
    expect(screen.getByRole("checkbox", { name: "Proposal 10 in General" })).toBeDisabled();
    fireEvent.click(screen.getByRole("checkbox", { name: "Proposal 0 in General" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Proposal 10 in General" }));
    await start();

    expect(onStart).toHaveBeenCalledWith("online", "team_meeting", {
      workspaceId: "general",
      borrowed: [],
      documentIds: ["doc-1", "doc-2", "doc-3", "doc-4", "doc-5", "doc-6", "doc-7", "doc-8", "doc-9", "doc-10"],
      meetingContext: "",
    });
  });

  it("starts in the workspace that is open, and can change it", async () => {
    const onStart = vi.fn();
    render(<MeetingSetupModal open workspaceId="a" onStart={onStart} onCancel={() => {}} />);
    expect(screen.getByRole("radio", { name: "Acme" })).toHaveAttribute("aria-checked", "true");
    fireEvent.click(screen.getByRole("radio", { name: "Biology" }));
    await start2();
    expect(onStart).toHaveBeenCalledWith("online", "interview", { workspaceId: "b", borrowed: [], documentIds: [], meetingContext: "" });
  });

  it("the workspace's scenario applies until one is picked here", async () => {
    const onStart = vi.fn();
    render(<MeetingSetupModal open workspaceId="b" onStart={onStart} onCancel={() => {}} />);
    expect(screen.getByRole("button", { name: "Interview" })).toBeInTheDocument();
    // An explicit pick wins, and survives a change of workspace.
    fireEvent.click(screen.getByRole("button", { name: "Interview" }));
    fireEvent.click(screen.getByRole("button", { name: /Board review/ }));
    fireEvent.click(screen.getByRole("radio", { name: "Acme" }));
    await start2();
    expect(onStart).toHaveBeenCalledWith("online", "custom_1", { workspaceId: "a", borrowed: [], documentIds: [], meetingContext: "" });
  });

  it("offers Also use, blocks kept-separate combinations with the reason, and drops a borrowed home", async () => {
    const onStart = vi.fn();
    render(<MeetingSetupModal open workspaceId="a" onStart={onStart} onCancel={() => {}} />);
    const vault = screen.getByRole("checkbox", { name: /Vault/ });
    expect(vault).toBeDisabled();
    expect(screen.getByText("Kept separate")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("checkbox", { name: /Biology/ }));
    await start2();
    expect(onStart).toHaveBeenLastCalledWith("online", expect.any(String), { workspaceId: "a", borrowed: ["b"], documentIds: [], meetingContext: "" });
    // Moving the home onto the borrowed one removes it from "Also use".
    fireEvent.click(screen.getByRole("radio", { name: "Biology" }));
    await start2();
    expect(onStart).toHaveBeenLastCalledWith("online", expect.any(String), { workspaceId: "b", borrowed: [], documentIds: [], meetingContext: "" });
  });
});
