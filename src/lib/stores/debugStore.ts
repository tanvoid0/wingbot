import { writable } from 'svelte/store';

export const isDebugModalOpen = writable(false);

export function openDebugModal() {
  console.log('Opening debug modal');
  isDebugModalOpen.set(true);
}

export function closeDebugModal() {
  console.log('Closing debug modal');
  isDebugModalOpen.set(false);
}
