// Meeting Audio Presets — one-click configuration profiles.

import type {
  AudioDevice,
  MeetingAudioConfig,
  STTProviderType,
} from "../lib/types";

export interface MeetingPreset {
  name: string;
  description: string;
  badge?: string;
  you: { hint: DeviceHint; stt: STTProviderType };
  them: { hint: DeviceHint; stt: STTProviderType };
  requiresKey?: string;
  requiresDownload?: string;
  windowsOnly?: boolean;
}

/** Hints for resolving device IDs at apply time. */
export type DeviceHint =
  | "default_mic"    // Default input device
  | "default_output" // Default output device
  | "mic_2";         // Second mic (for in-person with two mics)

export const BUILT_IN_PRESETS: MeetingPreset[] = [
  {
    name: "Whisper Turbo",
    description: "Use Whisper Large v3 Turbo for both sides of the meeting",
    badge: "Local",
    you: { hint: "default_mic", stt: "whisper_cpp" },
    them: { hint: "default_output", stt: "whisper_cpp" },
    requiresDownload: "whisper_cpp",
  },
  {
    name: "Parakeet",
    description: "Use NVIDIA Parakeet TDT 0.6B v3 for both sides of the meeting",
    badge: "Local",
    you: { hint: "default_mic", stt: "parakeet_tdt" },
    them: { hint: "default_output", stt: "parakeet_tdt" },
    requiresDownload: "parakeet_tdt",
  },
  {
    name: "Groq Whisper",
    description: "Use Groq Whisper for both sides; requires a Groq API key and cloud processing",
    badge: "Cloud",
    you: { hint: "default_mic", stt: "groq_whisper" },
    them: { hint: "default_output", stt: "groq_whisper" },
    requiresKey: "groq_whisper",
  },
];

/** Resolve a device hint to an actual device ID. */
function resolveHint(hint: DeviceHint, devices: AudioDevice[]): { device_id: string; is_input: boolean } {
  switch (hint) {
    case "default_mic": {
      const mic = devices.find((d) => d.is_input && d.is_default)
        ?? devices.find((d) => d.is_input);
      return { device_id: mic?.id ?? "default", is_input: true };
    }
    case "default_output": {
      const out = devices.find((d) => !d.is_input && d.is_default)
        ?? devices.find((d) => !d.is_input);
      return { device_id: out?.id ?? "default", is_input: false };
    }
    case "mic_2": {
      // Second microphone — skip the default
      const mics = devices.filter((d) => d.is_input);
      const second = mics.find((d) => !d.is_default) ?? mics[0];
      return { device_id: second?.id ?? "default", is_input: true };
    }
  }
}

/** Apply a preset using the current list of audio devices. */
export function applyPreset(
  preset: MeetingPreset,
  devices: AudioDevice[]
): MeetingAudioConfig {
  const youDevice = resolveHint(preset.you.hint, devices);
  const themDevice = resolveHint(preset.them.hint, devices);

  return {
    you: {
      role: "You",
      device_id: youDevice.device_id,
      is_input_device: youDevice.is_input,
      stt_provider: preset.you.stt,
    },
    them: {
      role: "Them",
      device_id: themDevice.device_id,
      is_input_device: themDevice.is_input,
      stt_provider: preset.them.stt,
    },
    recording_enabled: false,
    preset_name: preset.name,
  };
}
