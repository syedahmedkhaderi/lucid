import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CredentialProfile } from "../lib/types";
import { useConfigStore } from "../stores/configStore";

const api = vi.hoisted(() => ({
  add: vi.fn(),
  activate: vi.fn(),
  remove: vi.fn(),
  list: vi.fn(),
  enable: vi.fn(),
}));

vi.mock("../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../lib/ipc")>()),
  addCredentialProfile: (...args: unknown[]) => api.add(...args),
  activateCredentialProfile: (...args: unknown[]) => api.activate(...args),
  deleteCredentialProfile: (...args: unknown[]) => api.remove(...args),
  listCredentialProfiles: (...args: unknown[]) => api.list(...args),
  setCredentialProfileEnabled: (...args: unknown[]) => api.enable(...args),
}));

import { CredentialProfiles } from "./CredentialProfiles";

const profile: CredentialProfile = {
  id: "profile-1",
  provider: "gemini",
  label: "Primary",
  project_id: "project-alpha",
  key_suffix: "1234",
  enabled: true,
  active: false,
  verified_at: 1_700_000_000_000,
  last_used_at: null,
  last_error: null,
};

describe("CredentialProfiles", () => {
  beforeEach(() => {
    api.add.mockReset();
    api.activate.mockReset();
    api.remove.mockReset();
    api.list.mockReset().mockResolvedValue([]);
    api.enable.mockReset();
    useConfigStore.setState({ llmProvider: "ollama" });
  });

  afterEach(() => cleanup());

  it("validates and stores a key, then clears the password field without displaying the secret", async () => {
    api.add.mockResolvedValue(profile);
    api.list.mockResolvedValueOnce([]).mockResolvedValue([profile]);
    render(<CredentialProfiles provider="gemini" providerLabel="Google Gemini" />);
    const secret = "private-test-key-value";
    fireEvent.change(screen.getByLabelText("Google Gemini profile API key"), { target: { value: secret } });
    fireEvent.change(screen.getByLabelText("Credential label"), { target: { value: "Primary" } });
    fireEvent.change(screen.getByLabelText("Provider project ID"), { target: { value: "project-alpha" } });
    fireEvent.click(screen.getByRole("button", { name: "Validate & Store" }));

    await screen.findByText("Credential validated and stored securely.");
    expect(api.add).toHaveBeenCalledWith("gemini", secret, "Primary", "project-alpha");
    expect(screen.getByLabelText("Google Gemini profile API key")).toHaveValue("");
    expect(screen.queryByText(secret)).not.toBeInTheDocument();
    expect(await screen.findByText("••••••••1234 · project-alpha")).toBeInTheDocument();
  });

  it("imports pasted lines sequentially and clears the textarea when finished", async () => {
    api.add.mockResolvedValue(profile);
    render(<CredentialProfiles provider="gemini" providerLabel="Google Gemini" />);
    fireEvent.click(screen.getByRole("button", { name: "Import several keys" }));
    const input = screen.getByLabelText("Bulk credential import");
    fireEvent.change(input, { target: { value: "Primary\tproject-one\tkey-one\nBackup\t\tkey-two" } });
    fireEvent.click(screen.getByRole("button", { name: "Validate each line" }));

    await screen.findByText("2 credentials validated and stored.");
    expect(api.add).toHaveBeenNthCalledWith(1, "gemini", "key-one", "Primary", "project-one");
    expect(api.add).toHaveBeenNthCalledWith(2, "gemini", "key-two", "Backup", undefined);
    await waitFor(() => expect(input).toHaveValue(""));
    expect(screen.queryByText("key-one")).not.toBeInTheDocument();
    expect(screen.queryByText("key-two")).not.toBeInTheDocument();
  });

  it("keeps profile activation as a deliberate user action", async () => {
    api.list.mockResolvedValue([profile]);
    api.activate.mockResolvedValue({ ...profile, active: true });
    render(<CredentialProfiles provider="gemini" providerLabel="Google Gemini" />);
    fireEvent.click(await screen.findByRole("button", { name: "Use this" }));
    await waitFor(() => expect(api.activate).toHaveBeenCalledWith("profile-1"));
    expect(await screen.findByText("Primary is now the selected Google Gemini credential.")).toBeInTheDocument();
  });
});
