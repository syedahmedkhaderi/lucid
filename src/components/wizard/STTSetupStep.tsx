// Wizard Step 3: choose a supported local or Groq speech-to-text engine.

import { useEffect, useState, type KeyboardEvent } from "react";
import { useConfigStore } from "../../stores/configStore";
import { getLocalSTTCapabilities, hasApiKey, storeApiKey } from "../../lib/ipc";
import { LocalModelManager } from "../../settings/LocalModelManager";
import type { STTProviderType, MeetingAudioConfig, LocalSTTCapabilities } from "../../lib/types";
import { CheckCircle, Cpu, Eye, EyeOff, HardDrive, Loader2, Zap } from "lucide-react";

export interface ProviderChoice {
  value: STTProviderType;
  label: string;
  icon: React.ReactNode;
  description: string;
  requiresKey: boolean;
  free: boolean;
}

export const WIZARD_STT_PROVIDERS: ProviderChoice[] = [
  {
    value: "whisper_cpp",
    label: "Whisper Large v3 Turbo",
    icon: <HardDrive className="h-4 w-4" />,
    description: "Local and private · one-time 1.6 GB download",
    requiresKey: false,
    free: true,
  },
  {
    value: "parakeet_tdt",
    label: "NVIDIA Parakeet TDT 0.6B v3",
    icon: <Cpu className="h-4 w-4" />,
    description: "Local and private · one-time 465 MB download",
    requiresKey: false,
    free: true,
  },
  {
    value: "groq_whisper",
    label: "Groq Whisper",
    icon: <Zap className="h-4 w-4" />,
    description: "Cloud transcription · requires a Groq API key",
    requiresKey: true,
    free: false,
  },
];

const LOCAL_PROVIDERS: STTProviderType[] = ["whisper_cpp", "parakeet_tdt"];

function radioArrowKeys(e: KeyboardEvent<HTMLDivElement>) {
  if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
  const radios = Array.from(e.currentTarget.querySelectorAll<HTMLElement>('[role="radio"]'));
  const current = radios.indexOf(document.activeElement as HTMLElement);
  if (current < 0 || radios.length === 0) return;
  let next: number;
  switch (e.key) {
    case "ArrowDown":
    case "ArrowRight": next = (current + 1) % radios.length; break;
    case "ArrowUp":
    case "ArrowLeft": next = (current - 1 + radios.length) % radios.length; break;
    case "Home": next = 0; break;
    case "End": next = radios.length - 1; break;
    default: return;
  }
  e.preventDefault();
  radios[next].focus();
  radios[next].click();
}

export function STTSetupStep() {
  const meetingAudioConfig = useConfigStore((s) => s.meetingAudioConfig);
  const setMeetingAudioConfig = useConfigStore((s) => s.setMeetingAudioConfig);
  const privacyMode = useConfigStore((s) => s.privacyMode);
  const [capabilities, setCapabilities] = useState<LocalSTTCapabilities | null>(null);
  const [youSTT, setYouSTT] = useState<STTProviderType>("groq_whisper");
  const [themSTT, setThemSTT] = useState<STTProviderType>("groq_whisper");
  const [hasGroqKey, setHasGroqKey] = useState(false);
  const [groqKey, setGroqKey] = useState("");
  const [showGroqKey, setShowGroqKey] = useState(false);
  const [savingKey, setSavingKey] = useState(false);
  const [keySaved, setKeySaved] = useState(false);
  const [keyError, setKeyError] = useState("");

  useEffect(() => {
    let cancelled = false;
    Promise.all([
      getLocalSTTCapabilities().catch(() => null),
      hasApiKey("groq_whisper").catch(() => false),
    ]).then(([detected, savedKey]) => {
      if (cancelled) return;
      const cap = detected ?? {
        platform: "Unknown",
        has_gpu: false,
        local_models_available: false,
        reason: "Hardware detection is unavailable. Use Groq Whisper on this device.",
      };
      setCapabilities(cap);
      setHasGroqKey(savedKey);
      const canUseLocal = cap.local_models_available;
      const defaultProvider: STTProviderType = canUseLocal ? "whisper_cpp" : "groq_whisper";
      const available = canUseLocal ? [...LOCAL_PROVIDERS, "groq_whisper"] : ["groq_whisper"];
      const you = (meetingAudioConfig?.you.stt_provider && available.includes(meetingAudioConfig.you.stt_provider))
        ? meetingAudioConfig.you.stt_provider : defaultProvider;
      const them = (meetingAudioConfig?.them.stt_provider && available.includes(meetingAudioConfig.them.stt_provider))
        ? meetingAudioConfig.them.stt_provider : defaultProvider;
      setYouSTT(you);
      setThemSTT(them);
      if (meetingAudioConfig) {
        setMeetingAudioConfig({
          ...meetingAudioConfig,
          you: { ...meetingAudioConfig.you, stt_provider: you, local_model_id: LOCAL_PROVIDERS.includes(you) ? (you === "whisper_cpp" ? "large-v3-turbo" : "parakeet-tdt-0.6b-v3-int8") : undefined },
          them: { ...meetingAudioConfig.them, stt_provider: them, local_model_id: LOCAL_PROVIDERS.includes(them) ? (them === "whisper_cpp" ? "large-v3-turbo" : "parakeet-tdt-0.6b-v3-int8") : undefined },
          preset_name: null,
        });
      }
    });
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    if (!meetingAudioConfig || !capabilities) return;
    const modelId = (provider: STTProviderType) => provider === "whisper_cpp"
      ? "large-v3-turbo"
      : provider === "parakeet_tdt" ? "parakeet-tdt-0.6b-v3-int8" : undefined;
    setMeetingAudioConfig({
      ...meetingAudioConfig,
      you: { ...meetingAudioConfig.you, stt_provider: youSTT, local_model_id: modelId(youSTT) },
      them: { ...meetingAudioConfig.them, stt_provider: themSTT, local_model_id: modelId(themSTT) },
      preset_name: null,
    });
  }, [youSTT, themSTT, capabilities]);

  const choices = capabilities?.local_models_available
    ? WIZARD_STT_PROVIDERS
    : WIZARD_STT_PROVIDERS.filter((provider) => provider.value === "groq_whisper");
  const groqSelected = youSTT === "groq_whisper" || themSTT === "groq_whisper";

  async function saveGroqKey() {
    const key = groqKey.trim();
    if (!key) return;
    setSavingKey(true);
    setKeyError("");
    try {
      await storeApiKey("groq_whisper", key);
      setHasGroqKey(true);
      setGroqKey("");
      setKeySaved(true);
    } catch (error) {
      setKeyError(String(error));
    } finally {
      setSavingKey(false);
    }
  }

  return (
    <div className="flex flex-col items-center">
      <div className="mb-8 text-center">
        <div className="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-primary/10 shadow-md shadow-primary/10">
          <HardDrive className="h-7 w-7 text-primary" />
        </div>
        <h2 className="text-2xl font-bold text-foreground">Speech-to-Text Setup</h2>
        <p className="mt-2 text-sm leading-relaxed text-muted-foreground">
          Choose a transcriber for your microphone and meeting audio.
        </p>
      </div>

      <div className="w-full max-w-lg space-y-6">
        {!capabilities ? (
          <div role="status" className="flex items-center gap-2 rounded-xl border border-border/30 px-4 py-3 text-sm text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" /> Checking this device for local transcription support…
          </div>
        ) : capabilities.local_models_available ? (
          <>
            <div className="rounded-xl border border-success/20 bg-success/5 px-4 py-3 text-xs leading-relaxed text-success">
              This {capabilities.platform} supports local transcription. Audio stays on this device when either local model is selected.
            </div>
            <section aria-labelledby="model-download-title" className="rounded-xl border border-border/30 bg-card/50 p-4">
              <h3 id="model-download-title" className="mb-1 text-sm font-semibold text-foreground">Install local models here</h3>
              <p className="mb-4 text-xs leading-relaxed text-muted-foreground">
                Download in Lucid. Lucid verifies the file and activates the model automatically when setup finishes.
              </p>
              <div className="space-y-4">
                <div>
                  <p className="mb-2 flex items-center gap-2 text-xs font-medium text-foreground"><HardDrive className="h-3.5 w-3.5" />Whisper Large v3 Turbo</p>
                  <LocalModelManager compact engineFilter="whisper_cpp" />
                </div>
                <div>
                  <p className="mb-2 flex items-center gap-2 text-xs font-medium text-foreground"><Cpu className="h-3.5 w-3.5" />NVIDIA Parakeet TDT 0.6B v3</p>
                  <LocalModelManager compact engineFilter="parakeet_tdt" />
                </div>
              </div>
            </section>
          </>
        ) : (
          <div className="rounded-xl border border-info/20 bg-info/5 px-4 py-3 text-xs leading-relaxed text-muted-foreground">
            {capabilities.reason} The local model downloads are hidden on this device.
          </div>
        )}

        {(["you", "them"] as const).map((side) => {
          const selected = side === "you" ? youSTT : themSTT;
          const label = side === "you" ? "You" : "Them";
          return (
            <section key={side}>
              <h3 id={`stt-${side}-label`} className="mb-3 flex items-center gap-2.5 text-sm font-medium text-foreground">
                <span className={`rounded-lg px-2 py-1 text-meta font-semibold uppercase tracking-wide ${side === "you" ? "bg-primary/10 text-primary" : "bg-muted text-muted-foreground"}`}>{label}</span>
                Transcriber
              </h3>
              <div role="radiogroup" aria-labelledby={`stt-${side}-label`} onKeyDown={radioArrowKeys} className="grid grid-cols-1 gap-2">
                {choices.map((provider, index) => (
                  <ProviderButton
                    key={provider.value}
                    provider={provider}
                    selected={selected === provider.value}
                    tabStop={selected === provider.value || (index === 0 && !choices.some((choice) => choice.value === selected))}
                    onSelect={() => side === "you" ? setYouSTT(provider.value) : setThemSTT(provider.value)}
                  />
                ))}
              </div>
            </section>
          );
        })}

        {groqSelected && (
          <section aria-labelledby="groq-key-title" className="rounded-xl border border-border/30 bg-card/50 p-4">
            <div className="mb-2 flex items-center justify-between gap-3">
              <h3 id="groq-key-title" className="text-sm font-semibold text-foreground">Groq API key</h3>
              {hasGroqKey && <span className="inline-flex items-center gap-1 text-xs text-success"><CheckCircle className="h-3.5 w-3.5" />Saved securely</span>}
            </div>
            <p className="mb-3 text-xs leading-relaxed text-muted-foreground">Saved in the system credential store on this device. Add it here without leaving setup. Groq account limits depend on your plan.</p>
            <div className="flex gap-2">
              <div className="relative min-w-0 flex-1">
                <input
                  aria-label="Groq API key"
                  type={showGroqKey ? "text" : "password"}
                  value={groqKey}
                  onChange={(event) => { setGroqKey(event.target.value); setKeySaved(false); setKeyError(""); }}
                  autoComplete="off"
                  placeholder={hasGroqKey ? "Replace saved key" : "gsk_…"}
                  className="w-full rounded-lg border border-border/40 bg-background px-3 py-2 pr-9 text-sm text-foreground outline-none focus:border-primary"
                />
                <button type="button" aria-label={showGroqKey ? "Hide Groq API key" : "Show Groq API key"} onClick={() => setShowGroqKey((value) => !value)} className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground">
                  {showGroqKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
              <button type="button" disabled={!groqKey.trim() || savingKey} onClick={() => void saveGroqKey()} className="rounded-lg bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground disabled:opacity-50">
                {savingKey ? "Saving…" : keySaved ? "Saved" : "Save key"}
              </button>
            </div>
            {keyError && <p role="alert" className="mt-2 text-xs text-destructive">Could not save key: {keyError}</p>}
            {privacyMode === "private_local" && <p className="mt-3 text-xs text-warning">Cloud transcription is blocked while Private Local is on. Change the privacy mode in setup before sending audio to Groq.</p>}
          </section>
        )}
      </div>
    </div>
  );
}

function ProviderButton({
  provider,
  selected,
  tabStop,
  onSelect,
}: {
  provider: ProviderChoice;
  selected: boolean;
  tabStop: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      role="radio"
      aria-checked={selected}
      tabIndex={tabStop ? 0 : -1}
      onClick={onSelect}
      className={`flex w-full items-center gap-3 rounded-xl border px-4 py-3 text-left transition-colors ${selected ? "border-primary bg-primary/5 ring-1 ring-primary" : "border-border/30 bg-card/30 hover:border-border/60"}`}
    >
      <span className={selected ? "text-primary" : "text-muted-foreground"}>{provider.icon}</span>
      <span className="min-w-0 flex-1">
        <span className="flex items-center gap-2 text-sm font-medium text-foreground">{provider.label}
          {provider.free && <span className="rounded-full bg-success/10 px-1.5 py-0.5 text-meta font-semibold uppercase tracking-wide text-success">Local</span>}
          {provider.value === "groq_whisper" && <span className="rounded-full bg-info/10 px-1.5 py-0.5 text-meta font-semibold uppercase tracking-wide text-info">Cloud</span>}
        </span>
        <span className="mt-0.5 block text-xs text-muted-foreground">{provider.description}</span>
      </span>
    </button>
  );
}
