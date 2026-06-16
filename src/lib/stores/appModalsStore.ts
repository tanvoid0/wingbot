import { writable } from 'svelte/store';

export const isLauncherModalOpen = writable(false);
export const isOverlaysModalOpen = writable(false);

export function openLauncherModal() {
	isLauncherModalOpen.set(true);
}

export function closeLauncherModal() {
	isLauncherModalOpen.set(false);
}

export function openOverlaysModal() {
	isOverlaysModalOpen.set(true);
}

export function closeOverlaysModal() {
	isOverlaysModalOpen.set(false);
}
