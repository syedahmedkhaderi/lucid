import { useState } from "react";
import { AlertTriangle, Cloud, Laptop, ShieldCheck } from "lucide-react";
import { useConfigStore } from "../stores/configStore";
import type { PrivacyMode } from "../lib/types";

const OPTIONS: {
  value: PrivacyMode;
  title: string;
  description: string;
  icon: typeof ShieldCheck;
}[] = [
  {
    value: "private_local",
    title: "Private Local",
    description: "Keep meeting content on this device. Only offline speech models and local AI endpoints are allowed.",
    icon: Laptop,
  },
  {
    value: "cloud_allowed",
    title: "Cloud processing allowed",
    description: "Allow configured providers to process meeting content. Provider charges or usage limits may apply.",
    icon: Cloud,
  },
];

export function PrivacySettings() {
  const mode = useConfigStore((state) => state.privacyMode);
  const setPrivacyMode = useConfigStore((state) => state.setPrivacyMode);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const selectMode = async (value: PrivacyMode) => {
    if (value === mode || saving) return;
    setSaving(true);
    setError(null);
    try {
      await setPrivacyMode(value);
      if (value === "cloud_allowed") {
        const current = useConfigStore.getState();
        await current.activateLLMProvider(current.llmProvider);
      }
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-5">
      <div>
        <h3 className="text-sm font-semibold text-foreground">Privacy and content handling</h3>
        <p className="mt-1 text-xs leading-relaxed text-muted-foreground">
          Choose whether Lucid may send audio, transcripts, notes, documents, or images to network services.
          The Rust backend enforces this policy for provider requests.
        </p>
      </div>

      <div className="space-y-2" role="radiogroup" aria-label="Privacy mode">
        {OPTIONS.map(({ value, title, description, icon: Icon }) => {
          const selected = mode === value;
          return (
            <button
              key={value}
              type="button"
              role="radio"
              aria-checked={selected}
              disabled={saving}
              onClick={() => void selectMode(value)}
              className={`flex w-full items-start gap-3 rounded-xl border p-4 text-left transition-colors disabled:cursor-wait disabled:opacity-70 ${
                selected
                  ? "border-primary/70 bg-primary/5"
                  : "border-border/40 bg-card/40 hover:bg-accent/40"
              }`}
            >
              <span className={`mt-0.5 rounded-lg p-2 ${selected ? "bg-primary/10 text-primary" : "bg-secondary text-muted-foreground"}`}>
                <Icon className="h-4 w-4" aria-hidden="true" />
              </span>
              <span className="min-w-0 flex-1">
                <span className="flex items-center gap-2 text-sm font-medium text-foreground">
                  {title}
                  {selected && <ShieldCheck className="h-3.5 w-3.5 text-emerald-500" aria-label="Selected" />}
                </span>
                <span className="mt-1 block text-xs leading-relaxed text-muted-foreground">{description}</span>
              </span>
            </button>
          );
        })}
      </div>

      <div className="rounded-lg border border-border/30 bg-secondary/20 p-3 text-xs leading-relaxed text-muted-foreground">
        {mode === "private_local" ? (
          <>
            <p className="font-medium text-foreground">Allowed in Private Local</p>
            <p className="mt-1">Whisper.cpp, Sherpa ONNX, ORT, Parakeet, and Ollama or LM Studio on this device. Browser speech, Windows Speech dictation, subscription CLIs, and remote endpoints are blocked. Windows dictation can use Microsoft's online service.</p>
          </>
        ) : (
          <>
            <p className="font-medium text-foreground">Cloud providers can receive content</p>
            <p className="mt-1">Review each provider’s data handling and billing settings. Provider charges or usage limits may apply.</p>
          </>
        )}
      </div>

      <div className="flex items-start gap-2 text-[11px] leading-relaxed text-muted-foreground">
        <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" aria-hidden="true" />
        <p>Change this setting only after the meeting and audio capture have stopped. Local models and services you install are managed separately by you.</p>
      </div>
      {error && <p role="alert" className="text-xs text-destructive">{error}</p>}
      {saving && <p role="status" className="text-xs text-muted-foreground">Applying privacy setting…</p>}
    </div>
  );
}
