import { useCallback, useEffect, useState } from "react";
import { Check, KeyRound, Loader2, Plus, Trash2 } from "lucide-react";
import {
  activateCredentialProfile,
  addCredentialProfile,
  deleteCredentialProfile,
  listCredentialProfiles,
  setCredentialProfileEnabled,
} from "../lib/ipc";
import type { CredentialProfile } from "../lib/types";
import { useConfigStore } from "../stores/configStore";

interface Props {
  provider: string;
  providerLabel: string;
  onProfileActivated?: () => void;
}

function errorText(error: unknown): string {
  return typeof error === "string" ? error : error instanceof Error ? error.message : "Credential action failed";
}

/** User-managed profiles; secrets stay in the operating-system credential store. */
export function CredentialProfiles({ provider, providerLabel, onProfileActivated }: Props) {
  const [profiles, setProfiles] = useState<CredentialProfile[]>([]);
  const [label, setLabel] = useState("");
  const [projectId, setProjectId] = useState("");
  const [keyValue, setKeyValue] = useState("");
  const [bulkText, setBulkText] = useState("");
  const [bulkOpen, setBulkOpen] = useState(false);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const result = await listCredentialProfiles(provider);
      setProfiles(Array.isArray(result) ? result : []);
    } catch (cause) {
      setError(errorText(cause));
    } finally {
      setLoading(false);
    }
  }, [provider]);

  useEffect(() => {
    setError("");
    setMessage("");
    void refresh();
  }, [refresh]);

  const addOne = async (value: string, profileLabel: string, profileProject?: string) => {
    await addCredentialProfile(provider, value, profileLabel, profileProject || undefined);
  };

  const saveOne = async () => {
    const value = keyValue.trim();
    if (!value) return;
    setBusy(true);
    setError("");
    setMessage("");
    try {
      await addOne(value, label.trim() || `${providerLabel} key ${profiles.length + 1}`, projectId.trim());
      setMessage("Credential validated and stored securely.");
      setLabel("");
      setProjectId("");
      await refresh();
    } catch (cause) {
      setError(errorText(cause));
    } finally {
      setKeyValue("");
      setBusy(false);
    }
  };

  const importMany = async () => {
    const lines = bulkText.split(/\r?\n/).map((line, index) => ({ line: line.trim(), number: index + 1 })).filter((row) => row.line);
    if (lines.length === 0) return;
    setBusy(true);
    setError("");
    setMessage("");
    const failures: string[] = [];
    let imported = 0;
    try {
      // Keep checks sequential so a bulk paste cannot fan out requests or
      // exceed a provider's documented request rate.
      for (const row of lines) {
        const cells = row.line.split("\t");
        if (cells.length < 3 || !cells.slice(2).join("\t").trim()) {
          failures.push(`Line ${row.number}: expected label, project ID, and API key separated by tabs.`);
          continue;
        }
        const [profileLabel, project, ...secretParts] = cells;
        const secret = secretParts.join("\t").trim();
        try {
          await addOne(secret, profileLabel.trim() || `${providerLabel} imported ${row.number}`, project.trim());
          imported += 1;
        } catch (cause) {
          failures.push(`Line ${row.number}: ${errorText(cause)}`);
        }
      }
      setMessage(`${imported} credential${imported === 1 ? "" : "s"} validated and stored.`);
      if (failures.length) setError(failures.join("\n"));
      await refresh();
    } finally {
      // Never keep a pasted batch of secrets sitting in a mounted textarea.
      setBulkText("");
      setBusy(false);
    }
  };

  const activate = async (profile: CredentialProfile) => {
    setBusy(true);
    setError("");
    setMessage("");
    try {
      await activateCredentialProfile(profile.id);
      if (useConfigStore.getState().llmProvider === provider) {
        await useConfigStore.getState().activateLLMProvider(provider);
      }
      onProfileActivated?.();
      setMessage(`${profile.label} is now the selected ${providerLabel} credential.`);
      await refresh();
    } catch (cause) {
      setError(errorText(cause));
    } finally {
      setBusy(false);
    }
  };

  const toggleEnabled = async (profile: CredentialProfile) => {
    setBusy(true);
    setError("");
    setMessage("");
    try {
      await setCredentialProfileEnabled(profile.id, !profile.enabled);
      await refresh();
    } catch (cause) {
      setError(errorText(cause));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (profile: CredentialProfile) => {
    setBusy(true);
    setError("");
    setMessage("");
    try {
      await deleteCredentialProfile(profile.id);
      setMessage(`${profile.label} was removed from this device.`);
      await refresh();
    } catch (cause) {
      setError(errorText(cause));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="mt-4 rounded-lg border border-border/40 bg-background/50 p-4" aria-label={`${providerLabel} credential profiles`}>
      <div className="flex items-start gap-2.5">
        <KeyRound className="mt-0.5 h-4 w-4 shrink-0 text-primary/80" />
        <div className="min-w-0 flex-1">
          <h4 className="text-sm font-semibold text-foreground">Credential profiles</h4>
          <p className="mt-1 text-xs leading-relaxed text-muted-foreground">
            Keys are validated before storage. The keychain holds each secret; Lucid keeps only its label, project association and masked ending. Quotas apply to the associated project, and quota errors never switch profiles automatically.
          </p>
        </div>
      </div>

      <div className="mt-4 grid gap-2 sm:grid-cols-2">
        <input aria-label="Credential label" value={label} onChange={(event) => setLabel(event.target.value)} maxLength={80} placeholder="Label (for example, Procurement)" className="rounded-md border border-border/50 bg-background px-3 py-2 text-xs text-foreground placeholder:text-muted-foreground/50 focus:border-primary focus:outline-none" />
        <input aria-label="Provider project ID" value={projectId} onChange={(event) => setProjectId(event.target.value)} maxLength={128} placeholder="Project ID (optional; enter it yourself)" className="rounded-md border border-border/50 bg-background px-3 py-2 text-xs text-foreground placeholder:text-muted-foreground/50 focus:border-primary focus:outline-none" />
      </div>
      <div className="mt-2 flex gap-2">
        <input aria-label={`${providerLabel} profile API key`} type="password" autoComplete="off" value={keyValue} onChange={(event) => setKeyValue(event.target.value)} maxLength={4096} placeholder={`Enter a ${providerLabel} key`} className="min-w-0 flex-1 rounded-md border border-border/50 bg-background px-3 py-2 text-xs text-foreground placeholder:text-muted-foreground/50 focus:border-primary focus:outline-none" />
        <button type="button" onClick={() => void saveOne()} disabled={!keyValue.trim() || busy} className="inline-flex shrink-0 items-center gap-1.5 rounded-md bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground disabled:opacity-50">
          {busy ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Plus className="h-3.5 w-3.5" />}
          Validate & Store
        </button>
      </div>

      <div className="mt-4">
        <button type="button" onClick={() => setBulkOpen((open) => !open)} aria-expanded={bulkOpen} className="text-xs font-medium text-primary hover:underline">
          {bulkOpen ? "Hide bulk import" : "Import several keys"}
        </button>
        {bulkOpen && (
          <div className="mt-2">
            <p className="mb-1.5 text-xs text-muted-foreground">One key per line: label, project ID, then API key, separated by tabs. Leave the project field empty if unknown.</p>
            <textarea aria-label="Bulk credential import" value={bulkText} onChange={(event) => setBulkText(event.target.value)} rows={4} spellCheck={false} autoComplete="off" placeholder={`Primary\tmy-gemini-project\t<API key>\nBackup\t\t<API key>`} className="w-full resize-y rounded-md border border-border/50 bg-background px-3 py-2 font-mono text-xs text-foreground placeholder:text-muted-foreground/50 focus:border-primary focus:outline-none" />
            <button type="button" onClick={() => void importMany()} disabled={!bulkText.trim() || busy} className="mt-2 inline-flex items-center gap-1.5 rounded-md border border-border/50 px-3 py-2 text-xs font-medium text-foreground disabled:opacity-50">
              {busy ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Plus className="h-3.5 w-3.5" />}
              Validate each line
            </button>
          </div>
        )}
      </div>

      {message && <p role="status" className="mt-3 flex items-center gap-1.5 text-xs text-success"><Check className="h-3.5 w-3.5" />{message}</p>}
      {error && <p role="alert" className="mt-3 whitespace-pre-line text-xs text-destructive">{error}</p>}

      <div className="mt-4 border-t border-border/30 pt-3">
        <div className="mb-2 flex items-center justify-between">
          <span className="text-xs font-medium text-foreground">Saved profiles</span>
          <button type="button" onClick={() => void refresh()} disabled={loading || busy} className="text-xs text-muted-foreground hover:text-foreground disabled:opacity-50">Refresh</button>
        </div>
        {loading ? <p className="text-xs text-muted-foreground">Reading keychain profiles…</p> : profiles.length === 0 ? <p className="text-xs text-muted-foreground">No profiles saved yet.</p> : (
          <ul className="space-y-2">
            {profiles.map((profile) => (
              <li key={profile.id} className="flex flex-wrap items-center gap-2 rounded-md border border-border/30 px-3 py-2">
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="truncate text-xs font-medium text-foreground">{profile.label}</span>
                    {profile.active && <span className="rounded-full bg-success/10 px-1.5 py-0.5 text-[10px] font-semibold text-success">Selected</span>}
                    {!profile.enabled && <span className="rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">Disabled</span>}
                  </div>
                  <p className="mt-0.5 text-[11px] text-muted-foreground">
                    ••••••••{profile.key_suffix}{profile.project_id ? ` · ${profile.project_id}` : " · project not linked"}
                  </p>
                </div>
                {!profile.active && profile.enabled && <button type="button" disabled={busy} onClick={() => void activate(profile)} className="rounded border border-border/50 px-2 py-1 text-[11px] text-foreground disabled:opacity-50">Use this</button>}
                {!profile.active && <button type="button" disabled={busy} onClick={() => void toggleEnabled(profile)} className="rounded border border-border/50 px-2 py-1 text-[11px] text-muted-foreground disabled:opacity-50">{profile.enabled ? "Disable" : "Enable"}</button>}
                {!profile.active && <button type="button" aria-label={`Remove ${profile.label}`} disabled={busy} onClick={() => void remove(profile)} className="rounded p-1 text-muted-foreground hover:text-destructive disabled:opacity-50"><Trash2 className="h-3.5 w-3.5" /></button>}
              </li>
            ))}
          </ul>
        )}
        <p className="mt-2 text-[11px] leading-relaxed text-muted-foreground">Lucid cannot read provider quota balances. Removing a profile deletes its local key; revoke the key with its provider if it was exposed.</p>
      </div>
    </section>
  );
}
