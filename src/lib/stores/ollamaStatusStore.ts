import { get, writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export type OllamaStatusState = 'disabled' | 'checking' | 'online' | 'offline';

export type OllamaFeatureFlags = {
  liveCoach: boolean;
  matchHistory: boolean;
  scout: boolean;
  rune: boolean;
  lobby: boolean;
};

const DEFAULT_FLAGS: OllamaFeatureFlags = {
  liveCoach: true,
  matchHistory: true,
  scout: true,
  rune: true,
  lobby: true,
};

export const ollamaEnabled = writable(false);
export const ollamaStatus = writable<OllamaStatusState>('disabled');
export const ollamaModelName = writable('wingbot-lol');
export const ollamaLastError = writable<string | null>(null);
export const ollamaFeatureFlags = writable<OllamaFeatureFlags>({ ...DEFAULT_FLAGS });

let pollTimer: ReturnType<typeof setInterval> | undefined;
let refreshInFlight = false;

export function isOllamaFeatureEnabled(feature: keyof OllamaFeatureFlags): boolean {
  if (!get(ollamaEnabled)) return false;
  return get(ollamaFeatureFlags)[feature] ?? true;
}

function applyOllamaConfig(ollama?: {
  enabled?: boolean;
  base_url?: string;
  model_name?: string;
  poll_interval_sec?: number;
  live_coach?: boolean;
  insights_match_history?: boolean;
  insights_scout?: boolean;
  insights_rune?: boolean;
  insights_lobby?: boolean;
}) {
  const enabled = ollama?.enabled ?? false;
  ollamaEnabled.set(enabled);
  ollamaModelName.set(ollama?.model_name ?? 'wingbot-lol');
  ollamaFeatureFlags.set({
    liveCoach: ollama?.live_coach ?? true,
    matchHistory: ollama?.insights_match_history ?? true,
    scout: ollama?.insights_scout ?? true,
    rune: ollama?.insights_rune ?? true,
    lobby: ollama?.insights_lobby ?? true,
  });
}

export async function refreshOllamaStatus() {
  if (refreshInFlight) return;
  refreshInFlight = true;

  try {
    const config = await invoke<{ ollama?: Parameters<typeof applyOllamaConfig>[0] }>('get_config');
    applyOllamaConfig(config?.ollama);

    if (!get(ollamaEnabled)) {
      ollamaStatus.set('disabled');
      ollamaLastError.set(null);
      return;
    }

    const ollama = config?.ollama;
    ollamaStatus.set('checking');
    await invoke('test_ollama_connection', {
      config: {
        enabled: true,
        base_url: ollama?.base_url ?? 'http://127.0.0.1:11434',
        model_name: ollama?.model_name ?? 'wingbot-lol',
        poll_interval_sec: ollama?.poll_interval_sec ?? 30,
        live_coach: ollama?.live_coach ?? true,
        insights_match_history: ollama?.insights_match_history ?? true,
        insights_scout: ollama?.insights_scout ?? true,
        insights_rune: ollama?.insights_rune ?? true,
        insights_lobby: ollama?.insights_lobby ?? true,
      },
    });

    ollamaStatus.set('online');
    ollamaLastError.set(null);
  } catch (error) {
    if (get(ollamaEnabled)) {
      ollamaStatus.set('offline');
      ollamaLastError.set(error instanceof Error ? error.message : String(error));
    } else {
      ollamaStatus.set('disabled');
      ollamaLastError.set(null);
    }
  } finally {
    refreshInFlight = false;
  }
}

export function startOllamaStatusPolling(intervalMs = 30_000) {
  stopOllamaStatusPolling();
  void refreshOllamaStatus();
  pollTimer = setInterval(() => void refreshOllamaStatus(), intervalMs);
}

export function stopOllamaStatusPolling() {
  if (pollTimer !== undefined) {
    clearInterval(pollTimer);
    pollTimer = undefined;
  }
}
