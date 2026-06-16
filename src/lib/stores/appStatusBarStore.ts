import { writable } from 'svelte/store';

/** Page-specific overrides for the global status bar title/badge. */
export type AppStatusBarContext = {
  title?: string;
  badge?: string;
  badgeClass?: string;
};

/** Optional page toolbar actions (e.g. Settings save/reset). */
export type AppStatusBarToolbar = {
  hasChanges?: boolean;
  isLoading?: boolean;
  onSave?: () => void | Promise<void>;
  onReset?: () => void | Promise<void>;
};

export const appStatusBarContext = writable<AppStatusBarContext | null>(null);
export const appStatusBarToolbar = writable<AppStatusBarToolbar | null>(null);

export function clearAppStatusBarPageState() {
  appStatusBarContext.set(null);
  appStatusBarToolbar.set(null);
}
