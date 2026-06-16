import { get, writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export type OllamaStatusState = 'disabled' | 'checking' | 'online' | 'offline';

export const ollamaEnabled = writable(false);
export const ollamaStatus = writable<OllamaStatusState>('disabled');
export const ollamaModelName = writable('wingbot-lol');
export const ollamaLastError = writable<string | null>(null);

let pollTimer: ReturnType<typeof setInterval> | undefined;
let refreshInFlight = false;

export async function refreshOllamaStatus() {
  if (refreshInFlight) return;
  refreshInFlight = true;

  try {
    const config = await invoke<{
      ollama?: {
        enabled?: boolean;
        base_url?: string;
        model_name?: string;
        poll_interval_sec?: number;
      };
    }>('get_config');

    const ollama = config?.ollama;
    const enabled = ollama?.enabled ?? false;
    ollamaEnabled.set(enabled);
    ollamaModelName.set(ollama?.model_name ?? 'wingbot-lol');

    if (!enabled) {
      ollamaStatus.set('disabled');
      ollamaLastError.set(null);
      return;
    }

    ollamaStatus.set('checking');
    await invoke('test_ollama_connection', {
      config: {
        enabled: true,
        base_url: ollama?.base_url ?? 'http://127.0.0.1:11434',
        model_name: ollama?.model_name ?? 'wingbot-lol',
        poll_interval_sec: ollama?.poll_interval_sec ?? 30,
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
